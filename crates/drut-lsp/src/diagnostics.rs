//! `textDocument/publishDiagnostics` (FR-005–FR-007, `contracts/
//! lsp-capabilities.md`).
//!
//! Covers seven of `voyager-core`'s eight `DiagnosticKind` values.
//! `InvalidEncoding` never appears in `parse_result.diagnostics` here by
//! construction — `document_store.rs` always calls `parse()`, never
//! `parse_bytes()` — not something this module needs to filter out
//! (data-model.md §3, research.md §12).

use lsp_server::{Connection, Message};
use lsp_types::notification::{Notification as _, PublishDiagnostics};
use lsp_types::{DiagnosticSeverity, PublishDiagnosticsParams, Uri};
use voyager_core::{Position, Span};

use crate::document_store::ServerState;
use crate::position::to_lsp_range;
use crate::workspace::resolve_path;

fn kind_name(kind: voyager_core::DiagnosticKind) -> &'static str {
    use voyager_core::DiagnosticKind::*;
    match kind {
        UnmatchedIf => "UnmatchedIf",
        UnmatchedLoop => "UnmatchedLoop",
        UnclosedBlockComment => "UnclosedBlockComment",
        InvalidContinuation => "InvalidContinuation",
        UnmatchedRun => "UnmatchedRun",
        UnmatchedProcess => "UnmatchedProcess",
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

    let structural_diagnostics = doc.parse_result.diagnostics.iter().map(|d| lsp_types::Diagnostic {
        range: to_lsp_range(&doc.text, d.span),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(lsp_types::NumberOrString::String(kind_name(d.kind).to_string())),
        code_description: None,
        source: Some("drut".to_string()),
        message: d.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    });

    // 010-fmt-region-markers FR-010: a second, independently-sourced stream
    // for unclosed '; FMT: OFF' markers — deliberately not a
    // voyager_core::Diagnostic/DiagnosticKind (spec.md Assumptions), so it's
    // built here directly from the standalone unclosed_fmt_off_markers()
    // scan rather than folded into parse_result.diagnostics above. HINT
    // severity and a distinct "drut-fmt" source keep it visually and
    // programmatically separate from the seven/eight real DiagnosticKind
    // values, which all publish at ERROR above.
    let fmt_marker_diagnostics =
        voyager_core::unclosed_fmt_off_markers(&doc.text)
            .into_iter()
            .map(|pos| lsp_types::Diagnostic {
                range: to_lsp_range(&doc.text, unclosed_marker_line_span(pos)),
                severity: Some(DiagnosticSeverity::HINT),
                code: Some(lsp_types::NumberOrString::String("UnclosedFmtOff".to_string())),
                code_description: None,
                source: Some("drut-fmt".to_string()),
                message: "'; FMT: OFF' has no matching '; FMT: ON' — formatting is suppressed through end of file"
                    .to_string(),
                related_information: None,
                tags: None,
                data: None,
            });

    // 012-toml-configuration FR-011: a third, independently-sourced stream
    // for a malformed drut.toml governing this document — same "additive,
    // non-Diagnostic-kind, distinct source/severity" treatment 010's own
    // fmt-marker stream above established. Never blocks: the document still
    // formats/parses normally regardless of this stream's contents
    // (research.md §6).
    let config_warnings: Vec<lsp_types::Diagnostic> = resolve_path(uri, state)
        .and_then(|path| drut_config::discover(&path))
        .map(|config_path| drut_config::parse::parse(&config_path).1)
        .unwrap_or_default()
        .into_iter()
        .map(|warning| lsp_types::Diagnostic {
            range: to_lsp_range(&doc.text, Span::new(Position::new(1, 1), Position::new(1, u32::MAX))),
            severity: Some(DiagnosticSeverity::HINT),
            code: Some(lsp_types::NumberOrString::String("DrutTomlProblem".to_string())),
            code_description: None,
            source: Some("drut-config".to_string()),
            message: warning.to_string(),
            related_information: None,
            tags: None,
            data: None,
        })
        .collect();

    let diagnostics = structural_diagnostics
        .chain(fmt_marker_diagnostics)
        .chain(config_warnings)
        .collect();

    send(connection, uri.clone(), diagnostics, Some(doc.version));
}

/// Widens a single marker position into a span covering the rest of its
/// line — `to_lsp_position`'s own column-clamping (already tested in
/// `position.rs`) takes care of stopping at the line's real end, so this
/// needs no line-length lookup of its own. A zero-width range at just
/// `pos` would be a valid but poorly-visible diagnostic in many editors.
fn unclosed_marker_line_span(pos: Position) -> Span {
    Span::new(pos, Position::new(pos.line, u32::MAX))
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
