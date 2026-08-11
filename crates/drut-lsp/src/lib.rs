//! `drut-lsp`: a thin LSP adapter over `voyager-core` (constitution Principle
//! I) — diagnostics, hover, completion, spell-check, and semantic tokens for
//! Cube Voyager control-statement scripts. No grammar/parsing/lint-rule logic
//! lives here; every fact this crate reports is derived from `voyager-core`'s
//! public entry points (see specs/003-lsp-vscode-extension/plan.md).

pub mod completion;
pub mod diagnostics;
pub mod document_store;
pub mod formatting;
pub mod hover;
pub mod position;
pub mod range_formatting;
pub mod semantic_tokens;
pub mod spellcheck;

use lsp_server::{Connection, Message, Notification as ServerNotification, Request as ServerRequest, Response};
use lsp_types::notification::Notification as _;
use lsp_types::request::Request as _;

use document_store::ServerState;

/// The custom semantic-token names beyond `lsp-types`' standard set
/// (research.md §6, `contracts/lsp-capabilities.md`). `STATEMENT` is a
/// generic base type for a whole-statement span carrying only a modifier
/// (`unreachable`) — semantic tokens always need a base type, and this
/// feature declares no general-syntax types at all (that's the static
/// TextMate grammar's job, FR-021), so a minimal "statement" type covers
/// the modifier-only case distinctly from `shortIf`'s own dedicated type.
pub const SHORT_IF_TOKEN_TYPE: &str = "shortIf";
pub const STATEMENT_TOKEN_TYPE: &str = "statement";
pub const UNREACHABLE_TOKEN_MODIFIER: &str = "unreachable";

/// Builds the `ServerCapabilities` this server declares at `initialize`
/// (`contracts/lsp-capabilities.md`).
fn server_capabilities() -> lsp_types::ServerCapabilities {
    lsp_types::ServerCapabilities {
        // Fixed constant, never negotiated — vscode-languageclient rejects
        // anything but utf-16 (research.md §1, point 4;
        // contracts/position-encoding.md).
        position_encoding: Some(lsp_types::PositionEncodingKind::UTF16),
        text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Kind(
            lsp_types::TextDocumentSyncKind::FULL,
        )),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        // Added 2026-08-10 (see formatting.rs's own module docs) --
        // whole-document formatting only, no range/on-type variants.
        document_formatting_provider: Some(lsp_types::OneOf::Left(true)),
        // Added 2026-08-11 (see range_formatting.rs's own module docs) --
        // serves VS Code's editor.formatOnPaste
        // (specs/005-format-on-save-paste).
        document_range_formatting_provider: Some(lsp_types::OneOf::Left(true)),
        completion_provider: Some(lsp_types::CompletionOptions {
            trigger_characters: Some(vec![" ".to_string(), "=".to_string()]),
            ..Default::default()
        }),
        semantic_tokens_provider: Some(
            lsp_types::SemanticTokensOptions {
                legend: lsp_types::SemanticTokensLegend {
                    token_types: vec![
                        lsp_types::SemanticTokenType::new(SHORT_IF_TOKEN_TYPE),
                        lsp_types::SemanticTokenType::new(STATEMENT_TOKEN_TYPE),
                        // Added 2026-08-10 (semantic_tokens.rs): a
                        // *standard* LSP semantic token type, not a custom
                        // one like the two above -- VS Code's editor ships
                        // a built-in baseline color for the ~20 standard
                        // types (including `variable`) that applies even
                        // when the active color theme has no rule of its
                        // own for it, unlike this extension's custom
                        // TextMate scopes (`variable.other.readwrite.
                        // drut-voyager`), which only render distinctly
                        // under a theme that happens to already color that
                        // exact scope -- found not to be reliably true via
                        // real manual VS Code testing this same day.
                        lsp_types::SemanticTokenType::VARIABLE,
                    ],
                    token_modifiers: vec![lsp_types::SemanticTokenModifier::new(
                        UNREACHABLE_TOKEN_MODIFIER,
                    )],
                },
                full: Some(lsp_types::SemanticTokensFullOptions::Bool(true)),
                ..Default::default()
            }
            .into(),
        ),
        ..Default::default()
    }
}

/// Runs the LSP server loop over `connection` until the client disconnects.
///
/// This is the entry point both `drut-cli`'s `server` subcommand (over real
/// stdio) and this crate's own tests (over `lsp_server::Connection::memory()`,
/// research.md §9) call — no LSP protocol logic lives in `drut-cli` itself.
pub fn run(connection: Connection) {
    let caps = serde_json::to_value(server_capabilities()).expect("ServerCapabilities always serializes");
    if connection.initialize(caps).is_err() {
        // Client disconnected before completing the handshake — nothing more
        // to do (FR-004: never panic).
        return;
    }

    let mut state = ServerState::new();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                match connection.handle_shutdown(&req) {
                    Ok(true) => break,
                    Ok(false) => {}
                    Err(_) => break,
                }
                handle_request(&connection, req, &mut state);
            }
            Message::Notification(note) => {
                handle_notification(&connection, note, &mut state);
            }
            Message::Response(_) => {
                // This server never sends requests of its own, so it never
                // expects a response back — nothing to do.
            }
        }
    }
}

fn handle_notification(connection: &Connection, note: ServerNotification, state: &mut ServerState) {
    use lsp_types::notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument};

    let note = match note.extract::<lsp_types::DidOpenTextDocumentParams>(DidOpenTextDocument::METHOD) {
        Ok(params) => {
            state.did_open(
                params.text_document.uri.clone(),
                params.text_document.text,
                params.text_document.version,
            );
            diagnostics::publish(connection, state, &params.text_document.uri);
            return;
        }
        Err(lsp_server::ExtractError::MethodMismatch(note)) => note,
        Err(lsp_server::ExtractError::JsonError { .. }) => return,
    };

    let note = match note.extract::<lsp_types::DidChangeTextDocumentParams>(DidChangeTextDocument::METHOD) {
        Ok(params) => {
            let uri = params.text_document.uri.clone();
            if let Some(change) = params.content_changes.into_iter().next() {
                state.did_change(&uri, change.text, params.text_document.version);
                diagnostics::publish(connection, state, &uri);
            }
            return;
        }
        Err(lsp_server::ExtractError::MethodMismatch(note)) => note,
        Err(lsp_server::ExtractError::JsonError { .. }) => return,
    };

    if let Ok(params) = note.extract::<lsp_types::DidCloseTextDocumentParams>(DidCloseTextDocument::METHOD) {
        state.did_close(&params.text_document.uri);
        diagnostics::publish_empty(connection, &params.text_document.uri);
    }
}

fn handle_request(connection: &Connection, req: ServerRequest, state: &mut ServerState) {
    use lsp_types::request::{Completion, Formatting, HoverRequest, RangeFormatting, SemanticTokensFullRequest};

    let id = req.id.clone();
    let method = req.method.clone();

    match method.as_str() {
        HoverRequest::METHOD => match serde_json::from_value::<lsp_types::HoverParams>(req.params) {
            Ok(params) => send_ok(connection, id, &hover::handle(state, &params)),
            Err(e) => send_err(connection, id, e.to_string()),
        },
        Completion::METHOD => match serde_json::from_value::<lsp_types::CompletionParams>(req.params) {
            Ok(params) => send_ok(connection, id, &completion::handle(state, &params)),
            Err(e) => send_err(connection, id, e.to_string()),
        },
        SemanticTokensFullRequest::METHOD => {
            match serde_json::from_value::<lsp_types::SemanticTokensParams>(req.params) {
                Ok(params) => send_ok(connection, id, &semantic_tokens::handle(state, &params)),
                Err(e) => send_err(connection, id, e.to_string()),
            }
        }
        Formatting::METHOD => match serde_json::from_value::<lsp_types::DocumentFormattingParams>(req.params) {
            Ok(params) => send_ok(connection, id, &formatting::handle(state, &params)),
            Err(e) => send_err(connection, id, e.to_string()),
        },
        RangeFormatting::METHOD => {
            match serde_json::from_value::<lsp_types::DocumentRangeFormattingParams>(req.params) {
                Ok(params) => send_ok(connection, id, &range_formatting::handle(state, &params)),
                Err(e) => send_err(connection, id, e.to_string()),
            }
        }
        other => send_err(connection, id, format!("unhandled method: {other}")),
    }
}

fn send_ok<T: serde::Serialize>(connection: &Connection, id: lsp_server::RequestId, result: &T) {
    let response = Response::new_ok(id, result);
    let _ = connection.sender.send(Message::Response(response));
}

fn send_err(connection: &Connection, id: lsp_server::RequestId, message: String) {
    let response = Response::new_err(id, lsp_server::ErrorCode::MethodNotFound as i32, message);
    let _ = connection.sender.send(Message::Response(response));
}
