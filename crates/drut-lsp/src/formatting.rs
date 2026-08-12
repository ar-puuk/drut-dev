//! `textDocument/formatting` — added 2026-08-10 during manual VS Code
//! verification (see `specs/003-lsp-vscode-extension/spec.md`'s dated
//! Assumptions entry for the full rationale: this capability didn't exist
//! in the original phase scope, but "Format Document"/format-on-save not
//! working at all, with `voyager_core::format` already fully built and
//! tested by `002-cli-check-format`, was a real, concrete gap surfaced by
//! hands-on testing, not a hypothetical).
//!
//! Thin wrapper over `voyager_core::format` (Principle I) — no
//! whitespace/casing logic lives here. Always returns a single `TextEdit`
//! spanning the whole document (never a set of minimal diffs): simplest to
//! reason about, and correct regardless of how much or little changed,
//! since `voyager_core::format` itself already guarantees idempotence
//! (`002-cli-check-format`'s own golden-fixture corpus) — reformatting an
//! already-formatted document is always a safe no-op edit.

use crate::document_store::ServerState;
use crate::position::to_lsp_range;

/// Handles a `textDocument/formatting` request.
///
/// Casing is deliberately left untouched (`FormatOptions::default()`,
/// `casing: None`) — this phase wires up whitespace/structure formatting
/// only; an opt-in casing setting is a `drut-cli`-only concern today (FR-015)
/// and out of scope for LSP-triggered formatting until a real settings
/// surface exists for it (spec.md Assumptions rules out any configuration
/// surface this phase, `003`'s own precedent).
pub fn handle(
    state: &ServerState,
    params: &lsp_types::DocumentFormattingParams,
) -> Option<Vec<lsp_types::TextEdit>> {
    let uri = &params.text_document.uri;
    let doc = state.get(uri)?;

    let result = voyager_core::format(&doc.text, voyager_core::FormatOptions::default());
    if !result.changed {
        // Already formatted -- an empty edit list, not `None` (`None` would
        // mean "this document has no formatter opinion at all", which isn't
        // true here; there's just nothing left to change).
        return Some(Vec::new());
    }

    let range = to_lsp_range(&doc.text, whole_document_span(&doc.text));
    Some(vec![lsp_types::TextEdit {
        range,
        new_text: result.text,
    }])
}

/// The `voyager-core` `Span` covering all of `text`, start to end — built by
/// walking every char once (1-based line/column, matching `Span`'s own
/// convention, `end` one past the last char). Deliberately local to this
/// module rather than a sentinel-position trick (e.g. `Position::MAX`)
/// through `position.rs`'s existing clamping: that clamping only clamps a
/// requested *column* to its line's real length, it does not clamp an
/// out-of-range *line* down to the document's real last line (verified
/// directly against `position.rs`'s own `out_of_range_line_clamps_rather_
/// than_panicking` test) — a huge sentinel line number would silently
/// survive translation as a bogus, too-large `Range`, not a safely-clamped
/// one.
fn whole_document_span(text: &str) -> voyager_core::Span {
    use voyager_core::Position;

    let mut line = 1u32;
    let mut column = 1u32;
    for c in text.chars() {
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    voyager_core::Span::new(Position::new(1, 1), Position::new(line, column))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn params(uri: &str) -> lsp_types::DocumentFormattingParams {
        lsp_types::DocumentFormattingParams {
            text_document: lsp_types::TextDocumentIdentifier {
                uri: lsp_types::Uri::from_str(uri).unwrap(),
            },
            options: lsp_types::FormattingOptions {
                tab_size: 4,
                insert_spaces: true,
                ..Default::default()
            },
            work_done_progress_params: Default::default(),
        }
    }

    #[test]
    fn misindented_body_statement_is_corrected_relative_to_its_opener() {
        // format.rs's own documented design (FR-012): a *top-level*
        // statement's own indentation is deliberately left untouched (see
        // format.rs's `plan_indentation` doc comment) -- only a nested
        // child's indentation is normalized, relative to its block's own
        // (possibly-untouched) opener line. `PRINT` here is wrongly flush
        // with `IF` instead of one level in.
        let mut state = ServerState::new();
        state.did_open(
            lsp_types::Uri::from_str("file:///a.s").unwrap(),
            "IF (a=b)\nPRINT LIST=1\nENDIF\n".to_string(),
            1,
        );
        let edits = handle(&state, &params("file:///a.s")).unwrap();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].new_text, "IF (a=b)\n    PRINT LIST=1\nENDIF\n");
    }

    #[test]
    fn already_formatted_document_returns_no_edits() {
        let mut state = ServerState::new();
        let text = "IF (a=b)\n    PRINT LIST=1\nENDIF\n".to_string();
        state.did_open(lsp_types::Uri::from_str("file:///a.s").unwrap(), text, 1);
        let edits = handle(&state, &params("file:///a.s")).unwrap();
        assert!(edits.is_empty(), "expected no edits for an already-formatted document, got {edits:?}");
    }

    #[test]
    fn non_zero_top_level_indentation_is_left_untouched_by_default() {
        // 009-top-level-indent-toggle FR-004(c)/User Story 3: no compiler
        // forcing function exists for this call site (it's a bare
        // FormatOptions::default(), not a struct literal) -- confirmed
        // directly rather than inferred from any other adapter's own test
        // passing.
        let mut state = ServerState::new();
        let text = "    IF (a=b)\n        PRINT LIST=1\n    ENDIF\n".to_string();
        state.did_open(lsp_types::Uri::from_str("file:///a.s").unwrap(), text.clone(), 1);
        let edits = handle(&state, &params("file:///a.s")).unwrap();
        assert!(edits.is_empty(), "non-zero top-level indentation must be left untouched by default, got {edits:?}");
    }

    #[test]
    fn unopened_document_returns_none() {
        let state = ServerState::new();
        assert!(handle(&state, &params("file:///never-opened.s")).is_none());
    }

    #[test]
    fn whole_document_span_covers_a_multiline_document() {
        let text = "IF (a=b)\nENDIF\n";
        let span = whole_document_span(text);
        assert_eq!(span.start, voyager_core::Position::new(1, 1));
        // Three lines by char-count: "IF (a=b)\n", "ENDIF\n", "" (the empty
        // tail after the final newline) -- line 3, column 1.
        assert_eq!(span.end, voyager_core::Position::new(3, 1));
    }
}
