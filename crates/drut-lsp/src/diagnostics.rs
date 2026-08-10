//! `textDocument/publishDiagnostics` (FR-005–FR-007, `contracts/
//! lsp-capabilities.md`).
//!
//! Covers six of `voyager-core`'s seven `DiagnosticKind` values.
//! `InvalidEncoding` never appears in `parse_result.diagnostics` here by
//! construction — `document_store.rs` always calls `parse()`, never
//! `parse_bytes()` — not something this module needs to filter out
//! (data-model.md §3, research.md §12).

use lsp_server::{Connection, Message};
use lsp_types::notification::{Notification as _, PublishDiagnostics};
use lsp_types::{DiagnosticSeverity, PublishDiagnosticsParams, Uri};

use crate::document_store::ServerState;
use crate::position::to_lsp_range;

fn kind_name(kind: voyager_core::DiagnosticKind) -> &'static str {
    use voyager_core::DiagnosticKind::*;
    match kind {
        UnmatchedIf => "UnmatchedIf",
        UnmatchedLoop => "UnmatchedLoop",
        UnclosedBlockComment => "UnclosedBlockComment",
        InvalidContinuation => "InvalidContinuation",
        UnmatchedRun => "UnmatchedRun",
        MisplacedBreak => "MisplacedBreak",
        InvalidEncoding => "InvalidEncoding", // structurally unreachable here; kept exhaustive
    }
}

/// Publishes every diagnostic for `uri`'s currently-open document — or an
/// empty list, if the document isn't open (defensive; callers only invoke
/// this right after inserting/updating the document).
pub fn publish(connection: &Connection, state: &ServerState, uri: &Uri) {
    let Some(doc) = state.get(uri) else {
        publish_empty(connection, uri);
        return;
    };

    let diagnostics = doc
        .parse_result
        .diagnostics
        .iter()
        .map(|d| lsp_types::Diagnostic {
            range: to_lsp_range(&doc.text, d.span),
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(lsp_types::NumberOrString::String(
                kind_name(d.kind).to_string(),
            )),
            code_description: None,
            source: Some("drut".to_string()),
            message: d.message.clone(),
            related_information: None,
            tags: None,
            data: None,
        })
        .collect();

    send(connection, uri.clone(), diagnostics, Some(doc.version));
}

/// Publishes an empty diagnostics list for `uri` (FR-006: clear on close, or
/// as a defensive fallback for a URI with no tracked document).
pub fn publish_empty(connection: &Connection, uri: &Uri) {
    send(connection, uri.clone(), Vec::new(), None);
}

fn send(
    connection: &Connection,
    uri: Uri,
    diagnostics: Vec<lsp_types::Diagnostic>,
    version: Option<i32>,
) {
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version,
    };
    let note = lsp_server::Notification::new(PublishDiagnostics::METHOD.to_string(), params);
    let _ = connection.sender.send(Message::Notification(note));
}
