//! Unit tests for `voyager_core::block_at` (T019, `contracts/
//! block-resolution-api.md`) — ports every case `drut-lsp/src/hover.rs`'s
//! own pre-extraction test module covered, directly against `block_at`,
//! independent of either caller (`drut-lsp`'s hover or `drut-mcp`'s
//! `query_structure`).

use voyager_core::{block_at, parse, BlockKindName, Position};

#[test]
fn block_style_if_reports_kind_and_matched_endif() {
    let result = parse("IF (a=b)\nENDIF\n");
    let info = block_at(&result.nodes, &result.diagnostics, Position::new(1, 2)).unwrap();
    assert_eq!(info.kind, BlockKindName::If);
    assert!(!info.is_short_if);
    let counterpart = info.counterpart.unwrap();
    assert_eq!(counterpart.start.line, 2);
}

#[test]
fn short_if_has_no_separate_closer() {
    let result = parse("IF (a=b) PRINT LIST=1\n");
    let info = block_at(&result.nodes, &result.diagnostics, Position::new(1, 2)).unwrap();
    assert_eq!(info.kind, BlockKindName::If);
    assert!(info.is_short_if);
    assert!(info.counterpart.is_none());
}

#[test]
fn implicitly_closed_run_reports_resolved_location() {
    let result = parse("RUN PGM=MATRIX\nZONES=5\nRUN PGM=HWYASSIGN\nENDRUN\n");
    let info = block_at(&result.nodes, &result.diagnostics, Position::new(1, 2)).unwrap();
    assert_eq!(info.kind, BlockKindName::Run);
    assert!(!info.is_short_if);
    // The first RUN's body ends at line 2 (ZONES=5), right before the
    // second RUN implicitly closes it.
    let counterpart = info.counterpart.unwrap();
    assert_eq!(counterpart.start.line, 2);
}

#[test]
fn explicitly_closed_run_reports_its_own_endrun() {
    let result = parse("RUN PGM=MATRIX\nZONES=5\nENDRUN\n");
    let info = block_at(&result.nodes, &result.diagnostics, Position::new(1, 2)).unwrap();
    assert_eq!(info.kind, BlockKindName::Run);
    let counterpart = info.counterpart.unwrap();
    assert_eq!(counterpart.start.line, 3);
}

#[test]
fn process_block_reports_unconditional_counterpart() {
    let result = parse("PROCESS PHASE=INPUT\nFILEI=ni.1\n");
    let info = block_at(&result.nodes, &result.diagnostics, Position::new(1, 2)).unwrap();
    assert_eq!(info.kind, BlockKindName::Process);
    assert!(info.counterpart.is_some());
}

#[test]
fn position_with_no_enclosing_block_returns_none() {
    let result = parse("IF (a=b)\nXYZZY LIST=1\nENDIF\n");
    let info = block_at(&result.nodes, &result.diagnostics, Position::new(2, 2));
    assert!(info.is_none());
}
