//! `textDocument/rangeFormatting` — serves VS Code's `editor.formatOnPaste`
//! (specs/005-format-on-save-paste, contracts/range-formatting-api.md).
//!
//! Thin wrapper over `voyager_core::format`, same as `formatting.rs`
//! (constitution Principle I) — no independent formatting/grammar logic
//! here. Runs a normal whole-document format internally, then returns only
//! the edits whose line falls within the requested range — research.md
//! §2's resolution of spec.md's deferred "how" question, verified end to
//! end against real block-boundary fixtures (this module's own tests
//! below, mirroring contracts/range-formatting-api.md's Tests section).

use crate::document_store::ServerState;
use crate::position::to_lsp_range;

/// One line whose content `voyager_core::format` changed, relative to the
/// original document (data-model.md §1). Deliberately local to this
/// module — exists purely to make the line-diff step independently
/// testable from the LSP-shape translation around it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct LineEdit {
    /// 0-based line number — LSP convention, matches `Range.start.line`/
    /// `end.line` directly, no translation needed for the range-filter
    /// comparison in [`filter_to_range`].
    line_index: u32,
    /// The formatted line's full content, replacing the original line's
    /// content at `line_index` (never including a line terminator — see
    /// [`handle`]'s own note).
    new_content: String,
}

/// Compares `original` and `formatted` line by line, emitting a
/// [`LineEdit`] for every index where they disagree.
///
/// Safe as an *exact*, line-count-preserving comparison — never a generic
/// diff algorithm — because `voyager_core::format` only ever rewrites a
/// line's leading whitespace (and, opt-in only, keyword casing, never
/// enabled by any LSP caller): it never inserts, removes, reorders, or
/// merges lines (`format.rs`'s own documented scope, research.md §2).
/// `.lines()` on both texts therefore always yields the same number of
/// items — confirmed concretely, not just assumed, by
/// `contracts/range-formatting-api.md`'s two block-boundary fixtures
/// below, which preserve line count even while producing a genuine
/// `UnmatchedIf` diagnostic.
fn diff_lines(original: &str, formatted: &str) -> Vec<LineEdit> {
    original
        .lines()
        .zip(formatted.lines())
        .enumerate()
        .filter(|(_, (orig, fmt))| orig != fmt)
        .map(|(i, (_, fmt))| LineEdit {
            line_index: i as u32,
            new_content: fmt.to_string(),
        })
        .collect()
}

/// Keeps only the [`LineEdit`]s whose `line_index` falls within
/// `[range.start.line, range.end.line]` — inclusive on both ends
/// (data-model.md §1), covering a *structural* (block opener/closer) line
/// sitting exactly at either boundary the same as an ordinary
/// body-statement line (proven, not just asserted, by this module's own
/// `paste_that_opens_a_block_only_returns_the_in_range_edit`/
/// `paste_that_closes_a_block_only_returns_the_in_range_edit` tests, both
/// of which use a single-line range sitting exactly on a structural line).
fn filter_to_range(edits: Vec<LineEdit>, range: lsp_types::Range) -> Vec<LineEdit> {
    edits
        .into_iter()
        .filter(|edit| edit.line_index >= range.start.line && edit.line_index <= range.end.line)
        .collect()
}

/// The `voyager-core` `Span` covering line `line_index`'s *content* only —
/// column 1 through one past its last char, never the line terminator
/// itself. Deliberately excludes the terminator from both the computed
/// range and (in [`handle`]) the replacement text: including it would
/// require hardcoding a terminator string in `new_text`, silently
/// converting a CRLF-line-ended document to LF on every touched line —
/// directly contradicting `format.rs`'s own documented guarantee that
/// line-ending style is copied through unchanged. Excluding those bytes
/// from the edit entirely sidesteps the question — they are simply never
/// touched, correct regardless of which line-ending convention the
/// document actually uses (an implementation-time correction made before
/// writing this function, not the original contract's own first-drafted
/// design — see `contracts/range-formatting-api.md`'s own note on this).
fn line_content_span(text: &str, line_index: u32) -> voyager_core::Span {
    use voyager_core::Position;

    let line_number = line_index + 1; // voyager-core's Position is 1-based.
    let char_count = text.lines().nth(line_index as usize).map_or(0, |l| l.chars().count()) as u32;
    voyager_core::Span::new(Position::new(line_number, 1), Position::new(line_number, char_count + 1))
}

/// Handles a `textDocument/rangeFormatting` request.
///
/// Casing is deliberately left untouched (`FormatOptions::default()`),
/// same rationale as `formatting.rs`'s whole-document handler: no
/// configuration surface exists yet for LSP-triggered formatting (spec.md
/// Assumptions).
pub fn handle(
    state: &ServerState,
    params: &lsp_types::DocumentRangeFormattingParams,
) -> Option<Vec<lsp_types::TextEdit>> {
    let uri = &params.text_document.uri;
    let doc = state.get(uri)?;

    let result = voyager_core::format(&doc.text, voyager_core::FormatOptions::default());
    if !result.changed {
        // Already formatted -- an empty edit list, not `None` (`None`
        // would mean "this document has no formatter opinion at all",
        // which isn't true here; there's just nothing left to change),
        // same convention `formatting.rs` already uses.
        return Some(Vec::new());
    }

    let line_edits = diff_lines(&doc.text, &result.text);
    let in_range = filter_to_range(line_edits, params.range);

    let edits = in_range
        .into_iter()
        .map(|edit| lsp_types::TextEdit {
            range: to_lsp_range(&doc.text, line_content_span(&doc.text, edit.line_index)),
            new_text: edit.new_content,
        })
        .collect();

    Some(edits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn params(uri: &str, start_line: u32, end_line: u32) -> lsp_types::DocumentRangeFormattingParams {
        lsp_types::DocumentRangeFormattingParams {
            text_document: lsp_types::TextDocumentIdentifier {
                uri: lsp_types::Uri::from_str(uri).unwrap(),
            },
            range: lsp_types::Range {
                start: lsp_types::Position { line: start_line, character: 0 },
                end: lsp_types::Position { line: end_line, character: 0 },
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
    fn misindented_line_within_range_is_corrected() {
        let mut state = ServerState::new();
        state.did_open(
            lsp_types::Uri::from_str("file:///a.s").unwrap(),
            "IF (a=b)\nPRINT LIST=1\nENDIF\n".to_string(),
            1,
        );
        // Line 1 (0-based) is "PRINT LIST=1", misindented.
        let edits = handle(&state, &params("file:///a.s", 1, 1)).unwrap();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].new_text, "    PRINT LIST=1");
    }

    #[test]
    fn already_formatted_document_returns_empty_edit_list() {
        let mut state = ServerState::new();
        let text = "IF (a=b)\n    PRINT LIST=1\nENDIF\n".to_string();
        state.did_open(lsp_types::Uri::from_str("file:///a.s").unwrap(), text, 1);
        let edits = handle(&state, &params("file:///a.s", 0, 2)).unwrap();
        assert!(edits.is_empty(), "expected no edits for an already-formatted document, got {edits:?}");
    }

    #[test]
    fn unopened_document_returns_none() {
        let state = ServerState::new();
        assert!(handle(&state, &params("file:///never-opened.s", 0, 0)).is_none());
    }

    #[test]
    fn change_outside_requested_range_is_not_returned() {
        let mut state = ServerState::new();
        // Two unrelated, independently-misindented lines inside two
        // separate top-level IF blocks.
        let text = "IF (a=b)\nPRINT LIST=1\nENDIF\nIF (c=d)\nPRINT LIST=2\nENDIF\n".to_string();
        state.did_open(lsp_types::Uri::from_str("file:///a.s").unwrap(), text, 1);
        // Range covers only line 1 ("PRINT LIST=1"), not line 4
        // ("PRINT LIST=2").
        let edits = handle(&state, &params("file:///a.s", 1, 1)).unwrap();
        assert_eq!(edits.len(), 1, "expected exactly one in-range edit, got {edits:?}");
        assert_eq!(edits[0].new_text, "    PRINT LIST=1");
    }

    #[test]
    fn change_at_exact_range_boundary_is_included() {
        let mut state = ServerState::new();
        let text = "IF (a=b)\nPRINT LIST=1\nENDIF\n".to_string();
        state.did_open(lsp_types::Uri::from_str("file:///a.s").unwrap(), text, 1);
        // Range starts and ends exactly on line 1 -- the misindented line
        // itself, at both boundaries simultaneously.
        let edits = handle(&state, &params("file:///a.s", 1, 1)).unwrap();
        assert_eq!(edits.len(), 1);
    }

    /// contracts/range-formatting-api.md's verified block-boundary fixture:
    /// a paste that opens a block (a lone `IF (c=3)`, no closer within the
    /// paste) between two body statements of an already-open block. The
    /// whole-document reformat needs to change four lines total (3, 4, 5,
    /// 6) and reports one `UnmatchedIf` diagnostic -- only line 3 (the
    /// pasted line itself) is within the requested range and must be
    /// returned.
    #[test]
    fn paste_that_opens_a_block_only_returns_the_in_range_edit() {
        let mut state = ServerState::new();
        let text = "IF (a=1)\n    IF (b=2)\n        PRINT LIST=1\nIF (c=3)\n        PRINT LIST=2\n    ENDIF\nENDIF\n".to_string();
        state.did_open(lsp_types::Uri::from_str("file:///a.s").unwrap(), text, 1);
        let edits = handle(&state, &params("file:///a.s", 3, 3)).unwrap();
        assert_eq!(
            edits.len(),
            1,
            "expected exactly one in-range edit (line 3), got {edits:?} -- lines 4/5/6 also \
             change in the whole-document reformat but must be filtered out"
        );
        assert_eq!(edits[0].new_text, "        IF (c=3)");
    }

    /// The mirror case: a paste that closes a block (a lone `ENDIF`, no
    /// opener within the paste) between the same two body statements. The
    /// whole-document reformat needs to change three lines total (3, 4, 5)
    /// and reports one `UnmatchedIf` diagnostic (a stray closer at the
    /// end) -- only line 3 is within the requested range.
    #[test]
    fn paste_that_closes_a_block_only_returns_the_in_range_edit() {
        let mut state = ServerState::new();
        let text = "IF (a=1)\n    IF (b=2)\n        PRINT LIST=1\nENDIF\n        PRINT LIST=2\n    ENDIF\nENDIF\n".to_string();
        state.did_open(lsp_types::Uri::from_str("file:///a.s").unwrap(), text, 1);
        let edits = handle(&state, &params("file:///a.s", 3, 3)).unwrap();
        assert_eq!(
            edits.len(),
            1,
            "expected exactly one in-range edit (line 3), got {edits:?} -- lines 4/5 also \
             change in the whole-document reformat but must be filtered out"
        );
        assert_eq!(edits[0].new_text, "    ENDIF");
    }
}
