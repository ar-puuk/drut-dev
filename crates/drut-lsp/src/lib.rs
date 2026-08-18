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

/// Fixed registration ID for the first request this server ever sends
/// (013-lsp-config-file-watch) — safe as a fixed constant since only one
/// request of *this* kind is ever outstanding at a time; distinguished from
/// `CLIENT_FORMAT_DEFAULTS_ID` below (021-editor-settings-config's own,
/// second, server-initiated request kind) by `handle_response`'s own match
/// on the response's ID.
const DRUT_TOML_WATCHER_ID: &str = "drut-toml-watcher";

/// Fixed ID for the (now second) request this server ever initiates
/// (021-editor-settings-config, research.md §2/§4) — asking the client for
/// its `"drut.format"` `workspace/configuration` section. Re-sent (still
/// under this same fixed ID) on every `workspace/didChangeConfiguration`
/// notification, not just once at startup — safe as a single fixed constant
/// for the identical reason `DRUT_TOML_WATCHER_ID` already is: only one
/// pull is ever outstanding for this request kind at a time; a stale
/// response from a superseded pull is simply the most recent one
/// `handle_response` sees, matching the existing "whatever arrives last
/// wins" cache-replacement semantics of `ServerState::set_client_format_
/// defaults`.
const CLIENT_FORMAT_DEFAULTS_ID: &str = "drut-client-format-defaults";

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

    // 021-editor-settings-config: the second server-initiated request this
    // server ever sends, identical fire-and-forget shape as the watcher
    // registration immediately above — no wait here either.
    if workspace_configuration_supported(init_params.as_ref()) {
        request_client_format_defaults(&connection);
    }

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
                handle_response(&connection, response, &mut state);
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

/// Whether the client advertised support for `workspace/configuration`
/// (021-editor-settings-config, research.md §2) — same "no static-
/// capability alternative, confirmed against `lsp-types`' own capability
/// doc comment" shape `did_change_watched_files_supported` already
/// established for the file watcher. `false` for a client that omits this,
/// doesn't support it, or whose params failed to parse — the request is
/// then never attempted (FR-004), and formatting behaves exactly as before
/// this feature.
fn workspace_configuration_supported(params: Option<&lsp_types::InitializeParams>) -> bool {
    params
        .and_then(|p| p.capabilities.workspace.as_ref())
        .and_then(|w| w.configuration)
        .unwrap_or(false)
}

/// Sends the (now second) request this server ever initiates: asking the
/// client for its merged `"drut.format"` `workspace/configuration` section
/// (021-editor-settings-config, research.md §2/§4, `contracts/
/// editor-settings-config.md`). Only called when `workspace_configuration_
/// supported` already returned `true` — never attempted against a client
/// that hasn't advertised support (FR-004). Fire-and-forget, identical
/// shape to `register_drut_toml_watcher`: does not wait for a response —
/// the eventual response, if any, is handled generically by
/// `handle_response` whenever it arrives on the main loop, and a response
/// that never arrives simply means the cache stays at its previous value
/// (research.md §2).
fn request_client_format_defaults(connection: &Connection) {
    use lsp_types::request::{Request as _, WorkspaceConfiguration};

    let params = lsp_types::ConfigurationParams {
        items: vec![lsp_types::ConfigurationItem {
            // 021-editor-settings-config, research.md §5: one single,
            // global pull — no per-document/per-workspace-folder scoping.
            scope_uri: None,
            section: Some("drut.format".to_string()),
        }],
    };
    let request = lsp_server::Request::new(
        lsp_server::RequestId::from(CLIENT_FORMAT_DEFAULTS_ID.to_string()),
        WorkspaceConfiguration::METHOD.to_string(),
        params,
    );
    let _ = connection.sender.send(Message::Request(request));
}

/// Handles a response to one of the (now two) requests this server ever
/// sends (013-lsp-config-file-watch, 021-editor-settings-config, FR-010).
/// Never blocks anything — called generically from the main loop's own
/// unified message dispatch, which has no per-message-type blocking wait of
/// any kind; a response that never arrives simply means this is never
/// called for that ID, and every other message continues to be handled
/// normally regardless. Distinguishes the two request kinds by ID:
/// `CLIENT_FORMAT_DEFAULTS_ID` updates `state`'s cache on success and is
/// silently left alone on failure/malformed content (the cache simply stays
/// at its previous value, research.md §2); anything else (i.e. the
/// `drut.toml` watcher registration) keeps its original error-result-is-
/// logged-but-otherwise-inert behavior.
fn handle_response(connection: &Connection, response: Response, state: &mut ServerState) {
    if response.id == lsp_server::RequestId::from(CLIENT_FORMAT_DEFAULTS_ID.to_string()) {
        if let Ok(value) = response.response_result {
            // `WorkspaceConfiguration::Result` is `Vec<Value>`, one entry
            // per requested `ConfigurationItem` — exactly one item was ever
            // requested (research.md §4), so only its first entry matters.
            // A client that can't provide the section returns `null` there
            // (per `lsp-types`' own doc comment on `WorkspaceConfiguration`)
            // -- `parse_client_format_defaults` treats that identically to
            // an object with every field absent (spec.md Edge Cases).
            if let Some(section) = value.as_array().and_then(|items| items.first()) {
                state.set_client_format_defaults(parse_client_format_defaults(section));
            }
        }
        // An error result, or a response whose shape didn't match at all,
        // simply leaves the cache at its previous value — never a hard
        // failure, matching every other malformed-config-value contract in
        // this project.
        return;
    }

    if let Err(error) = response.response_result {
        let params = lsp_types::LogMessageParams {
            typ: lsp_types::MessageType::WARNING,
            message: format!("drut.toml watcher registration failed: {error:?}"),
        };
        let note = lsp_server::Notification::new(lsp_types::notification::LogMessage::METHOD.to_string(), params);
        let _ = connection.sender.send(Message::Notification(note));
    }
}

/// Parses a `workspace/configuration` response's `"drut.format"` section
/// (a JSON object whose keys are this feature's own camelCase VS Code
/// setting names, data-model.md §3) into an `ExplicitFormatOverride`
/// (021-editor-settings-config T008). Every field is looked up
/// independently — a missing key, a key of the wrong JSON type, or an
/// unrecognized string value all resolve to that one field staying `None`,
/// never a hard failure and never affecting any other field (data-model.md
/// §2). `value` being anything other than a JSON object (e.g. `null`, for a
/// client that can't provide the section at all) also resolves to every
/// field `None` — the same outcome as an object with every key individually
/// absent (spec.md Edge Cases).
fn parse_client_format_defaults(value: &serde_json::Value) -> drut_config::ExplicitFormatOverride {
    let field = |key: &str| value.get(key);
    drut_config::ExplicitFormatOverride {
        casing_control_words: field("casingControlWords").and_then(parse_client_casing),
        casing_pair_keywords: field("casingPairKeywords").and_then(parse_client_casing),
        casing_data_references: field("casingDataReferences").and_then(parse_client_casing),
        indent_top_level: field("indentTopLevel").and_then(parse_client_indent_top_level),
        indent_width: field("indentWidth").and_then(parse_client_u8),
        operator_spacing: field("operatorSpacing").and_then(parse_client_operator_spacing),
        blank_lines: field("blankLines").and_then(parse_client_blank_lines),
        blank_lines_top_cap: field("blankLinesTopCap").and_then(parse_client_u8),
        blank_lines_nested_cap: field("blankLinesNestedCap").and_then(parse_client_u8),
    }
}

/// Shared by the three granular `*Casing` fields — identical accepted-value
/// shape (`"preserve"`/`"upper"`/`"lower"`) every one of them already has
/// at the `drut.toml` parsing layer (`drut_config::parse::parse_casing`);
/// any other string, or any non-string JSON value, is simply unrecognized
/// here (`None`), not a hard failure.
fn parse_client_casing(value: &serde_json::Value) -> Option<voyager_core::CasingConvention> {
    match value.as_str()? {
        "preserve" => Some(voyager_core::CasingConvention::Preserve),
        "upper" => Some(voyager_core::CasingConvention::Upper),
        "lower" => Some(voyager_core::CasingConvention::Lower),
        _ => None,
    }
}

fn parse_client_indent_top_level(value: &serde_json::Value) -> Option<voyager_core::IndentTopLevelMode> {
    match value.as_str()? {
        "preserve" => Some(voyager_core::IndentTopLevelMode::Preserve),
        "auto" => Some(voyager_core::IndentTopLevelMode::Auto),
        _ => None,
    }
}

fn parse_client_operator_spacing(value: &serde_json::Value) -> Option<voyager_core::OperatorSpacing> {
    match value.as_str()? {
        "preserve" => Some(voyager_core::OperatorSpacing::Preserve),
        "fixed" => Some(voyager_core::OperatorSpacing::Fixed),
        "auto" => Some(voyager_core::OperatorSpacing::Auto),
        _ => None,
    }
}

fn parse_client_blank_lines(value: &serde_json::Value) -> Option<voyager_core::BlankLineMode> {
    match value.as_str()? {
        "preserve" => Some(voyager_core::BlankLineMode::Preserve),
        "auto" => Some(voyager_core::BlankLineMode::Auto),
        _ => None,
    }
}

/// Shared by `indentWidth`/`blankLinesTopCap`/`blankLinesNestedCap` — any
/// JSON number that fits in a `u8` is accepted here; the 1–16 (or
/// 1–50) valid-range bound is enforced later, at `drut_config::
/// resolve_format_options`'s own resolve layer, the same two-stage
/// parse-then-validate split `drut_config::parse` already uses for
/// `drut.toml`'s own `indent_width`/blank-line-cap fields. A value that
/// doesn't even fit in a `u8` (negative, or too large) is left `None`
/// here rather than silently clamped.
fn parse_client_u8(value: &serde_json::Value) -> Option<u8> {
    value.as_u64().and_then(|n| u8::try_from(n).ok())
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
    use lsp_types::notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument,
    };

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
    let note = match note.extract::<lsp_types::DidChangeWatchedFilesParams>(DidChangeWatchedFiles::METHOD) {
        Ok(_params) => {
            let uris: Vec<lsp_types::Uri> = state.open_uris().cloned().collect();
            for uri in &uris {
                diagnostics::publish(connection, state, uri);
            }
            return;
        }
        Err(lsp_server::ExtractError::MethodMismatch(note)) => note,
        Err(lsp_server::ExtractError::JsonError { .. }) => return,
    };

    // 021-editor-settings-config FR-002/FR-006, research.md §3: a client
    // setting changed somewhere -- re-fire the same fire-and-forget
    // workspace/configuration pull, never read this notification's own
    // `settings` payload (the modern LSP client convention sends it as a
    // bare `null` re-pull trigger, not a real data source). The refreshed
    // cache is picked up by the *next* format request against any open
    // document, with no reopen needed (SC-004) -- no document-level action
    // is taken here.
    if note.extract::<lsp_types::DidChangeConfigurationParams>(DidChangeConfiguration::METHOD).is_ok() {
        request_client_format_defaults(connection);
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Builds `Option<InitializeParams>` the same way `run()` itself does
    /// (`serde_json::from_value(...).ok()`), for `workspace_configuration_
    /// supported`'s own unit tests (021-editor-settings-config T010).
    fn init_params(capabilities: serde_json::Value) -> Option<lsp_types::InitializeParams> {
        serde_json::from_value(json!({"capabilities": capabilities})).ok()
    }

    #[test]
    fn workspace_configuration_supported_true_when_the_client_advertises_it() {
        let params = init_params(json!({"workspace": {"configuration": true}}));
        assert!(workspace_configuration_supported(params.as_ref()));
    }

    #[test]
    fn workspace_configuration_supported_false_when_the_workspace_key_is_absent() {
        let params = init_params(json!({}));
        assert!(!workspace_configuration_supported(params.as_ref()));
    }

    #[test]
    fn workspace_configuration_supported_false_when_explicitly_advertised_false() {
        let params = init_params(json!({"workspace": {"configuration": false}}));
        assert!(!workspace_configuration_supported(params.as_ref()));
    }

    #[test]
    fn workspace_configuration_supported_false_for_no_params_at_all() {
        assert!(!workspace_configuration_supported(None));
    }

    #[test]
    fn parse_client_format_defaults_parses_every_known_field() {
        let value = json!({
            // The legacy flat "casing" key is no longer looked up at all --
            // present here to prove it's harmlessly ignored, not that it
            // still does anything.
            "casing": "upper",
            "casingControlWords": "lower",
            "casingPairKeywords": "preserve",
            "casingDataReferences": "upper",
            "indentTopLevel": "auto",
            "indentWidth": 2,
            "operatorSpacing": "fixed",
            "blankLines": "auto",
            "blankLinesTopCap": 3,
            "blankLinesNestedCap": 1
        });
        let result = parse_client_format_defaults(&value);
        assert_eq!(result.casing_control_words, Some(voyager_core::CasingConvention::Lower));
        assert_eq!(result.casing_pair_keywords, Some(voyager_core::CasingConvention::Preserve));
        assert_eq!(result.casing_data_references, Some(voyager_core::CasingConvention::Upper));
        assert_eq!(result.indent_top_level, Some(voyager_core::IndentTopLevelMode::Auto));
        assert_eq!(result.indent_width, Some(2));
        assert_eq!(result.operator_spacing, Some(voyager_core::OperatorSpacing::Fixed));
        assert_eq!(result.blank_lines, Some(voyager_core::BlankLineMode::Auto));
        assert_eq!(result.blank_lines_top_cap, Some(3));
        assert_eq!(result.blank_lines_nested_cap, Some(1));
    }

    /// T010's own dedicated regression case: a malformed/partially-invalid
    /// pulled JSON object leaves only the affected field `None`, not the
    /// whole cache.
    #[test]
    fn parse_client_format_defaults_leaves_only_the_malformed_field_none() {
        let value = json!({
            "casingControlWords": "sideways",
            "indentWidth": 4
        });
        let result = parse_client_format_defaults(&value);
        assert_eq!(result.casing_control_words, None, "an unrecognized string must leave only this field None");
        assert_eq!(result.indent_width, Some(4), "a sibling valid field must be unaffected");
    }

    #[test]
    fn parse_client_format_defaults_of_a_non_object_value_resolves_to_every_field_none() {
        // A client that can't provide the section returns `null` for that
        // item (lsp-types' own WorkspaceConfiguration doc comment) --
        // treated identically to an object with every key absent (spec.md
        // Edge Cases).
        let result = parse_client_format_defaults(&json!(null));
        assert_eq!(result.casing_control_words, None);
        assert_eq!(result.indent_width, None);
    }

    #[test]
    fn parse_client_format_defaults_of_an_empty_object_resolves_to_every_field_none() {
        let result = parse_client_format_defaults(&json!({}));
        assert_eq!(result.casing_control_words, None);
        assert_eq!(result.indent_width, None);
    }
}
