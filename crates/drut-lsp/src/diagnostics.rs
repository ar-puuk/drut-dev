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
use crate::undefined_token;
use crate::unused_token;
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

    // 020-undefined-token-diagnostic: a fourth, independently-sourced stream
    // for @token@ references with no resolvable definition — same
    // "additive, non-Diagnostic-kind, distinct source/severity" treatment
    // the two streams above already established. Never a broader claim of
    // non-existence than the resolver itself can back up (constitution
    // Principle IV): every one of the resolver's own documented blind spots
    // (block-opener position, multi-level READ FILE inclusion, a
    // token-built inclusion path) is inherited automatically by reusing
    // `undefined_token_positions` unmodified, not suppressed by a separate
    // rule here (research.md §3).
    let undefined_token_diagnostics: Vec<lsp_types::Diagnostic> =
        undefined_token::undefined_token_positions(uri, doc)
            .into_iter()
            .map(|var_ref| lsp_types::Diagnostic {
                range: to_lsp_range(&doc.text, var_ref.span),
                severity: Some(DiagnosticSeverity::HINT),
                code: Some(lsp_types::NumberOrString::String("UndefinedToken".to_string())),
                code_description: None,
                source: Some("drut-token".to_string()),
                message: format!(
                    "'@{}@' has no assignment this tool can find in this file or a directly \
                     included one — it may still be defined elsewhere Drut can't see",
                    var_ref.name
                ),
                related_information: None,
                tags: None,
                data: None,
            })
            .collect();

    // 029-unused-token-diagnostic: a fifth, independently-sourced stream for
    // Assignment statements whose target name is never referenced via
    // @name@ anywhere in scope -- the exact inverse of UndefinedToken above.
    // Same "additive, non-Diagnostic-kind, distinct code" treatment, sharing
    // UndefinedToken's own "drut-token" source (same conceptual domain,
    // different diagnostic code). Applies unconditionally regardless of
    // whether this document participates in any READ FILE relationship --
    // a documented, accepted false-positive risk for the shared-parameters-
    // file pattern (spec.md Clarification Q2), not a bug.
    let unused_token_diagnostics: Vec<lsp_types::Diagnostic> =
        unused_token::unused_token_assignments(uri, doc)
            .into_iter()
            .map(|a| lsp_types::Diagnostic {
                range: to_lsp_range(&doc.text, a.statement_span),
                severity: Some(DiagnosticSeverity::HINT),
                code: Some(lsp_types::NumberOrString::String("UnusedToken".to_string())),
                code_description: None,
                source: Some("drut-token".to_string()),
                message: format!(
                    "'{}' is assigned but never referenced via '@{}@' in this file or a \
                     directly included one — it may still be used elsewhere Drut can't see",
                    a.target, a.target
                ),
                related_information: None,
                tags: None,
                data: None,
            })
            .collect();

    let diagnostics = structural_diagnostics
        .chain(fmt_marker_diagnostics)
        .chain(config_warnings)
        .chain(undefined_token_diagnostics)
        .chain(unused_token_diagnostics)
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
