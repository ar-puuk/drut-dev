//! `drut-lsp`: a thin LSP adapter over `voyager-core` (constitution Principle
//! I) — diagnostics, hover, completion, spell-check, and semantic tokens for
//! Cube Voyager control-statement scripts. No grammar/parsing/lint-rule logic
//! lives here; every fact this crate reports is derived from `voyager-core`'s
//! public entry points (see specs/003-lsp-vscode-extension/plan.md).

pub mod completion;
pub mod diagnostics;
pub mod document_store;
pub mod folding;
pub mod formatting;
pub mod hover;
pub mod position;
pub mod range_formatting;
pub mod semantic_tokens;
pub mod spellcheck;
pub mod undefined_token;
pub mod workspace;

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

/// Fixed registration ID for the one and only request this server ever
/// sends (013-lsp-config-file-watch) — safe as a single fixed constant
/// since there is only ever one outstanding server-initiated request kind;
/// a second one would need real per-request ID generation.
const DRUT_TOML_WATCHER_ID: &str = "drut-toml-watcher";

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
        // Added 2026-08-12 (011-code-folding, folding.rs's own module docs).
        folding_range_provider: Some(lsp_types::FoldingRangeProviderCapability::Simple(true)),
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
    let init_params_raw = match connection.initialize(caps) {
        Ok(params) => params,
        Err(_) => {
            // Client disconnected before completing the handshake — nothing
            // more to do (FR-004: never panic).
            return;
        }
    };

    log_startup_info(&connection);

    // Parsed once (013-lsp-config-file-watch/research.md §3), reused for both
    // the workspace-root fallback (012) and the file-watch capability check
    // (013) — `None` for a client whose params fail to parse, treated the
    // same as "supports nothing extra," not a startup failure.
    let init_params: Option<lsp_types::InitializeParams> = serde_json::from_value(init_params_raw).ok();

    let mut state = ServerState::new();
    state.set_workspace_root(workspace_root_from_initialize_params(init_params.as_ref()));

    if did_change_watched_files_supported(init_params.as_ref()) {
        register_drut_toml_watcher(&connection);
    }
    // No wait for a response here, by design (FR-010, research.md §1) — the
    // main loop below never blocks on any single message, so an unconfirmed
    // registration can never stall anything else.

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
            Message::Response(response) => {
                handle_response(&connection, response);
            }
        }
    }
}

/// Extracts the client's workspace root from the already-parsed `initialize`
/// params, for the untitled-buffer `drut.toml` discovery fallback
/// (012-toml-configuration/research.md §5). `rootUri` wins when present
/// (the LSP spec's own note: "If both rootPath and rootUri are set, rootUri
/// wins" — and `workspaceFolders` is the modern replacement specifically
/// for `rootUri`, so it's the fallback here, not the primary), falling back
/// to the first `workspaceFolders` entry. `None` for a client that sends
/// neither, or params that failed to parse — not a startup failure either
/// way.
#[allow(deprecated)]
fn workspace_root_from_initialize_params(params: Option<&lsp_types::InitializeParams>) -> Option<std::path::PathBuf> {
    let params = params?;
    let uri = params
        .root_uri
        .clone()
        .or_else(|| params.workspace_folders.as_ref()?.first().map(|f| f.uri.clone()))?;
    workspace::uri_to_path(&uri)
}

/// Whether the client advertised support for asking it to report file
/// changes on this server's behalf (013-lsp-config-file-watch, research.md
/// §2) — the *only* mechanism that exists for this at all; there is no
/// static-capability alternative (confirmed directly against `lsp-types`'
/// own `DidChangeWatchedFilesClientCapabilities` doc comment). `false` for
/// a client that omits this, doesn't support it, or whose params failed to
/// parse — registration is then never attempted (FR-004).
fn did_change_watched_files_supported(params: Option<&lsp_types::InitializeParams>) -> bool {
    params
        .and_then(|p| p.capabilities.workspace.as_ref())
        .and_then(|w| w.did_change_watched_files.as_ref())
        .and_then(|d| d.dynamic_registration)
        .unwrap_or(false)
}

/// Sends the one and only request this server ever initiates: asking the
/// client to report `drut.toml` changes anywhere in the workspace
/// (013-lsp-config-file-watch, research.md §4, `contracts/
/// config-watch-api.md`). Only called when `did_change_watched_files_
/// supported` already returned `true` — never attempted against a client
/// that hasn't advertised support (FR-004). Fire-and-forget: does not wait
/// for a response (FR-010) — the eventual response, if any, is handled
/// generically by `handle_response` whenever it arrives on the main loop.
fn register_drut_toml_watcher(connection: &Connection) {
    use lsp_types::request::{RegisterCapability, Request as _};

    let watcher = lsp_types::FileSystemWatcher {
        glob_pattern: lsp_types::GlobPattern::String("**/drut.toml".to_string()),
        kind: None, // defaults to Create | Change | Delete (lsp-types' own doc comment).
    };
    let registration = lsp_types::Registration {
        id: DRUT_TOML_WATCHER_ID.to_string(),
        method: lsp_types::notification::DidChangeWatchedFiles::METHOD.to_string(),
        register_options: Some(
            serde_json::to_value(lsp_types::DidChangeWatchedFilesRegistrationOptions { watchers: vec![watcher] })
                .expect("DidChangeWatchedFilesRegistrationOptions always serializes"),
        ),
    };
    let request = lsp_server::Request::new(
        lsp_server::RequestId::from(DRUT_TOML_WATCHER_ID.to_string()),
        RegisterCapability::METHOD.to_string(),
        lsp_types::RegistrationParams { registrations: vec![registration] },
    );
    let _ = connection.sender.send(Message::Request(request));
}

/// Handles a response to the one request this server ever sends
/// (013-lsp-config-file-watch, research.md §1/FR-010). Never blocks
/// anything — called generically from the main loop's own unified message
/// dispatch, which has no per-message-type blocking wait of any kind; a
/// response that never arrives simply means this is never called for that
/// ID, and every other message continues to be handled normally regardless.
/// An error result is logged (never silent, matching `010`/`011`/`012`'s
/// own precedent) but otherwise changes nothing about how the session
/// continues.
fn handle_response(connection: &Connection, response: Response) {
    if let Err(error) = response.response_result {
        let params = lsp_types::LogMessageParams {
            typ: lsp_types::MessageType::WARNING,
            message: format!("drut.toml watcher registration failed: {error:?}"),
        };
        let note = lsp_server::Notification::new(lsp_types::notification::LogMessage::METHOD.to_string(), params);
        let _ = connection.sender.send(Message::Notification(note));
    }
}

/// Reports exactly which binary/build is running, directly from inside the
/// process itself — via the LSP-standard `window/logMessage` notification
/// (constitution Principle VI: LSP-standard over editor-proprietary),
/// visible in any LSP-capable client's own log/Output surface (VS Code's
/// `vscode-languageclient` routes it into the language client's own Output
/// channel automatically, with no extension-side code needed). Added
/// 2026-08-11 so "which drut-lsp is VS Code actually running" is answerable
/// definitively from inside the editor, rather than inferred from PATH
/// resolution in a separate shell — found necessary during a live
/// debugging session where PATH-resolution divergence between the terminal
/// and VS Code's own spawned process was the leading, unconfirmed suspect
/// for a reported bug that turned out (per a real LSP-protocol-level test)
/// not to be a code defect at all.
fn log_startup_info(connection: &Connection) {
    let exe_path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|e| format!("<unavailable: {e}>"));
    let message = format!(
        "drut-lsp starting — binary: {exe_path} | commit: {} | built: {} (unix epoch seconds)",
        env!("DRUT_GIT_COMMIT"),
        env!("DRUT_BUILD_TIMESTAMP"),
    );
    let params = lsp_types::LogMessageParams {
        typ: lsp_types::MessageType::INFO,
        message,
    };
    let note = lsp_server::Notification::new(
        lsp_types::notification::LogMessage::METHOD.to_string(),
        params,
    );
    let _ = connection.sender.send(Message::Notification(note));
}

fn handle_notification(connection: &Connection, note: ServerNotification, state: &mut ServerState) {
    use lsp_types::notification::{DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument};

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

    let note = match note.extract::<lsp_types::DidCloseTextDocumentParams>(DidCloseTextDocument::METHOD) {
        Ok(params) => {
            state.did_close(&params.text_document.uri);
            diagnostics::publish_empty(connection, &params.text_document.uri);
            return;
        }
        Err(lsp_server::ExtractError::MethodMismatch(note)) => note,
        Err(lsp_server::ExtractError::JsonError { .. }) => return,
    };

    // 013-lsp-config-file-watch FR-001/FR-002: a `drut.toml` changed
    // somewhere in the workspace -- re-publish diagnostics for every
    // currently-open document, not just one. `diagnostics::publish` is
    // unmodified; it already re-resolves `drut-config` fresh internally
    // (research.md §5). Every `FileEvent` in `changes` is treated
    // identically regardless of its own `typ` (Created/Changed/Deleted) --
    // deliberate, per spec.md's Edge Cases.
    if note.extract::<lsp_types::DidChangeWatchedFilesParams>(DidChangeWatchedFiles::METHOD).is_ok() {
        let uris: Vec<lsp_types::Uri> = state.open_uris().cloned().collect();
        for uri in &uris {
            diagnostics::publish(connection, state, uri);
        }
    }
}

fn handle_request(connection: &Connection, req: ServerRequest, state: &mut ServerState) {
    use lsp_types::request::{
        Completion, FoldingRangeRequest, Formatting, HoverRequest, RangeFormatting, SemanticTokensFullRequest,
    };

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
        FoldingRangeRequest::METHOD => match serde_json::from_value::<lsp_types::FoldingRangeParams>(req.params) {
            Ok(params) => send_ok(connection, id, &folding::handle(state, &params)),
            Err(e) => send_err(connection, id, e.to_string()),
        },
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
