//! `textDocument/foldingRange` (011-code-folding, `contracts/
//! folding-range-api.md`).
//!
//! Block ranges come from `voyager_core::all_blocks` — a thin enumeration
//! wrapper reusing `block_at`'s own unchanged derivation rules
//! (`011-code-folding/research.md` §1). Block-comment ranges come from
//! `voyager_core::tokenize`'s already-public `TokenKind::BlockComment`
//! directly — no `voyager-core` change needed for that half
//! (research.md §2).

use voyager_core::{BlockFold, TokenKind};

use crate::document_store::ServerState;
use crate::position::to_lsp_position;

/// Handles a `textDocument/foldingRange` request. Returns `None` only when
/// the requested document isn't open (matches `hover::handle`'s own
/// "unknown document" behavior) — a document with nothing foldable returns
/// `Some(vec![])` (FR-011), never `None`.
pub fn handle(state: &ServerState, params: &lsp_types::FoldingRangeParams) -> Option<Vec<lsp_types::FoldingRange>> {
    let uri = &params.text_document.uri;
    let doc = state.get(uri)?;

    let mut ranges: Vec<lsp_types::FoldingRange> = Vec::new();

    for fold in voyager_core::all_blocks(&doc.parse_result.nodes, &doc.parse_result.diagnostics) {
        if let Some(range) = block_folding_range(&doc.text, &fold) {
            ranges.push(range);
        }
    }

    for token in voyager_core::tokenize(&doc.text) {
        if matches!(token.kind, TokenKind::BlockComment { unterminated: false }) {
            if let Some(range) = comment_folding_range(&doc.text, token.span) {
                ranges.push(range);
            }
        }
    }

    Some(ranges)
}

/// Builds a `Region`-kind range for a block with a resolvable counterpart,
/// or `None` for a short-`IF`/genuinely-unmatched block (`counterpart ==
/// None`, FR-004/FR-005) or a zero-span result (FR-008 — defensive for
/// blocks; no block kind's current rules produce one, research.md §5).
fn block_folding_range(text: &str, fold: &BlockFold) -> Option<lsp_types::FoldingRange> {
    let counterpart = fold.info.counterpart?;
    let start_line = to_lsp_position(text, fold.opener).line;
    let end_line = to_lsp_position(text, counterpart.start).line;
    if start_line >= end_line {
        return None;
    }
    Some(lsp_types::FoldingRange {
        start_line,
        start_character: None,
        end_line,
        end_character: None,
        kind: Some(lsp_types::FoldingRangeKind::Region),
        collapsed_text: None,
    })
}

/// Builds a `Comment`-kind range for a terminated block-comment token, or
/// `None` for a single-line comment (FR-008 — load-bearing for this stream,
/// research.md §5: a single-line `/* note */` has `span.start.line ==
/// span.end.line` and nothing upstream of this check excludes it).
fn comment_folding_range(text: &str, span: voyager_core::Span) -> Option<lsp_types::FoldingRange> {
    let start_line = to_lsp_position(text, span.start).line;
    let end_line = to_lsp_position(text, span.end).line;
    if start_line >= end_line {
        return None;
    }
    Some(lsp_types::FoldingRange {
        start_line,
        start_character: None,
        end_line,
        end_character: None,
        kind: Some(lsp_types::FoldingRangeKind::Comment),
        collapsed_text: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn params(uri: &str) -> lsp_types::FoldingRangeParams {
        lsp_types::FoldingRangeParams {
            text_document: lsp_types::TextDocumentIdentifier {
                uri: lsp_types::Uri::from_str(uri).unwrap(),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        }
    }

    fn open(state: &mut ServerState, uri: &str, text: &str) {
        state.did_open(lsp_types::Uri::from_str(uri).unwrap(), text.to_string(), 1);
    }

    fn region_range(ranges: &[lsp_types::FoldingRange], start_line: u32) -> &lsp_types::FoldingRange {
        ranges
            .iter()
            .find(|r| r.start_line == start_line && r.kind == Some(lsp_types::FoldingRangeKind::Region))
            .unwrap_or_else(|| panic!("expected a Region range starting at line {start_line}, got {ranges:?}"))
    }

    #[test]
    fn if_block_folds_from_opener_to_endif() {
        let mut state = ServerState::new();
        open(&mut state, "file:///a.s", "IF (a=b)\nX = 1\nENDIF\n");
        let ranges = handle(&state, &params("file:///a.s")).unwrap();
        let r = region_range(&ranges, 0);
        assert_eq!(r.end_line, 2);
    }

    #[test]
    fn loop_block_folds_from_opener_to_endloop() {
        let mut state = ServerState::new();
        open(&mut state, "file:///a.s", "LOOP i=1,5\nX = 1\nENDLOOP\n");
        let ranges = handle(&state, &params("file:///a.s")).unwrap();
        let r = region_range(&ranges, 0);
        assert_eq!(r.end_line, 2);
    }

    #[test]
    fn implicitly_closed_run_folds_to_the_line_before_the_next_run() {
        let mut state = ServerState::new();
        open(
            &mut state,
            "file:///a.s",
            "RUN PGM=MATRIX\nZONES=5\nRUN PGM=HWYASSIGN\nENDRUN\n",
        );
        let ranges = handle(&state, &params("file:///a.s")).unwrap();
        let r = region_range(&ranges, 0);
        assert_eq!(r.end_line, 1);
    }

    #[test]
    fn implicitly_closed_process_folds_correctly() {
        let mut state = ServerState::new();
        open(
            &mut state,
            "file:///a.s",
            "PROCESS PHASE=INPUT\nFILEI=ni.1\nPROCESS PHASE=OUTPUT\nENDPROCESS\n",
        );
        let ranges = handle(&state, &params("file:///a.s")).unwrap();
        let r = region_range(&ranges, 0);
        assert_eq!(r.end_line, 1);
    }

    #[test]
    fn short_if_produces_no_range() {
        let mut state = ServerState::new();
        open(&mut state, "file:///a.s", "IF (a=b) PRINT LIST=1\n");
        let ranges = handle(&state, &params("file:///a.s")).unwrap();
        assert!(ranges.is_empty(), "expected no range, got {ranges:?}");
    }

    #[test]
    fn genuinely_unmatched_if_produces_no_range() {
        let mut state = ServerState::new();
        open(&mut state, "file:///a.s", "IF (a=b)\nX = 1\n");
        let ranges = handle(&state, &params("file:///a.s")).unwrap();
        assert!(ranges.is_empty(), "expected no range, got {ranges:?}");
    }

    #[test]
    fn multi_line_block_comment_produces_a_comment_range() {
        let mut state = ServerState::new();
        open(&mut state, "file:///a.s", "/* line one\n   line two */\nX = 1\n");
        let ranges = handle(&state, &params("file:///a.s")).unwrap();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].kind, Some(lsp_types::FoldingRangeKind::Comment));
        assert_eq!(ranges[0].start_line, 0);
        assert_eq!(ranges[0].end_line, 1);
    }

    #[test]
    fn single_line_block_comment_produces_no_range() {
        let mut state = ServerState::new();
        open(&mut state, "file:///a.s", "/* note */\nX = 1\n");
        let ranges = handle(&state, &params("file:///a.s")).unwrap();
        assert!(ranges.is_empty(), "expected no range for a single-line block comment, got {ranges:?}");
    }

    #[test]
    fn unclosed_block_comment_produces_no_range() {
        let mut state = ServerState::new();
        open(&mut state, "file:///a.s", "/* never closed\nX = 1\n");
        let ranges = handle(&state, &params("file:///a.s")).unwrap();
        assert!(ranges.is_empty(), "expected no range, got {ranges:?}");
    }

    #[test]
    fn nested_blocks_each_get_independent_ranges() {
        let mut state = ServerState::new();
        open(&mut state, "file:///a.s", "IF (a=b)\nLOOP i=1,5\nENDLOOP\nENDIF\n");
        let ranges = handle(&state, &params("file:///a.s")).unwrap();
        assert_eq!(ranges.len(), 2);
        let outer = region_range(&ranges, 0);
        assert_eq!(outer.end_line, 3);
        let inner = region_range(&ranges, 1);
        assert_eq!(inner.end_line, 2);
    }

    #[test]
    fn document_with_nothing_foldable_returns_empty_vec() {
        let mut state = ServerState::new();
        open(&mut state, "file:///a.s", "X = 1\nY = 2\n");
        let ranges = handle(&state, &params("file:///a.s")).unwrap();
        assert_eq!(ranges, Vec::new());
    }

    #[test]
    fn unopened_document_returns_none() {
        let state = ServerState::new();
        let result = handle(&state, &params("file:///never-opened.s"));
        assert!(result.is_none());
    }
}
