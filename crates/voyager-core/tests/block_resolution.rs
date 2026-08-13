//! Unit tests for `voyager_core::block_at` (T019, `contracts/
//! block-resolution-api.md`) — ports every case `drut-lsp/src/hover.rs`'s
//! own pre-extraction test module covered, directly against `block_at`,
//! independent of either caller (`drut-lsp`'s hover or `drut-mcp`'s
//! `query_structure`).
//!
//! Also covers `voyager_core::all_blocks` (011-code-folding T003,
//! `contracts/folding-range-api.md`) — the full-document enumeration
//! `drut-lsp`'s folding capability depends on.

use voyager_core::{all_blocks, block_at, parse, BlockKindName, Position};

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

// --- all_blocks (011-code-folding T003) ---

#[test]
fn all_blocks_reports_every_explicitly_closed_kind() {
    let cases: &[(&str, BlockKindName, u32)] = &[
        ("IF (a=b)\nENDIF\n", BlockKindName::If, 2),
        ("LOOP i=1,5\nENDLOOP\n", BlockKindName::Loop, 2),
        ("RUN PGM=MATRIX\nZONES=5\nENDRUN\n", BlockKindName::Run, 3),
        ("PROCESS PHASE=INPUT\nFILEI=ni.1\nENDPROCESS\n", BlockKindName::Process, 3),
        (
            "IF (I=1)\nJLOOP\nX = 1\nENDJLOOP\nENDIF\n",
            BlockKindName::JLoop,
            4,
        ),
        (
            "LOOP i=1,5\nLINKLOOP\nX = 1\nENDLINKLOOP\nENDLOOP\n",
            BlockKindName::LinkLoop,
            4,
        ),
        (
            "DISTRIBUTEMULTISTEP PROCESSNUM=4\nX = 1\nENDDISTRIBUTEMULTISTEP\n",
            BlockKindName::DistributeMultistep,
            3,
        ),
    ];
    for (source, kind, expected_closer_line) in cases {
        let result = parse(source);
        let folds = all_blocks(&result.nodes, &result.diagnostics);
        let fold = folds
            .iter()
            .find(|f| f.info.kind == *kind)
            .unwrap_or_else(|| panic!("expected a {kind:?} BlockFold in {source:?}, got {folds:?}"));
        let counterpart = fold.info.counterpart.unwrap_or_else(|| panic!("expected a counterpart for {kind:?}"));
        assert_eq!(counterpart.start.line, *expected_closer_line);
    }
}

#[test]
fn all_blocks_reports_implicitly_closed_run_and_process() {
    let result = parse("RUN PGM=MATRIX\nZONES=5\nRUN PGM=HWYASSIGN\nENDRUN\n");
    let folds = all_blocks(&result.nodes, &result.diagnostics);
    let first_run = folds
        .iter()
        .find(|f| f.opener.line == 1)
        .expect("expected a BlockFold for the first RUN");
    assert_eq!(first_run.info.kind, BlockKindName::Run);
    let counterpart = first_run.info.counterpart.expect("implicitly-closed RUN should resolve a counterpart");
    assert_eq!(counterpart.start.line, 2);

    let result = parse("PROCESS PHASE=INPUT\nFILEI=ni.1\nPROCESS PHASE=OUTPUT\nENDPROCESS\n");
    let folds = all_blocks(&result.nodes, &result.diagnostics);
    let first_process = folds
        .iter()
        .find(|f| f.opener.line == 1)
        .expect("expected a BlockFold for the first PROCESS");
    assert_eq!(first_process.info.kind, BlockKindName::Process);
    assert!(first_process.info.counterpart.is_some());
}

#[test]
fn all_blocks_reports_short_if_with_no_counterpart() {
    let result = parse("IF (a=b) PRINT LIST=1\n");
    let folds = all_blocks(&result.nodes, &result.diagnostics);
    assert_eq!(folds.len(), 1);
    assert!(folds[0].info.is_short_if);
    assert!(folds[0].info.counterpart.is_none());
}

#[test]
fn all_blocks_reports_genuinely_unmatched_blocks_with_no_counterpart() {
    for source in ["IF (a=b)\nPRINT LIST=1\n", "LOOP i=1,5\nPRINT LIST=1\n", "RUN PGM=MATRIX\n"] {
        let result = parse(source);
        let folds = all_blocks(&result.nodes, &result.diagnostics);
        assert_eq!(folds.len(), 1, "source: {source:?}");
        assert!(!folds[0].info.is_short_if, "source: {source:?}");
        assert!(folds[0].info.counterpart.is_none(), "source: {source:?}");
    }
}

#[test]
fn all_blocks_reports_nested_blocks_independently() {
    let result = parse("IF (a=b)\nLOOP i=1,5\nENDLOOP\nENDIF\n");
    let folds = all_blocks(&result.nodes, &result.diagnostics);
    assert_eq!(folds.len(), 2);
    assert!(folds.iter().any(|f| f.info.kind == BlockKindName::If && f.opener.line == 1));
    let inner_loop = folds
        .iter()
        .find(|f| f.info.kind == BlockKindName::Loop)
        .expect("expected the nested LOOP to be reported independently");
    assert_eq!(inner_loop.opener.line, 2);
    assert_eq!(inner_loop.info.counterpart.unwrap().start.line, 3);
}

#[test]
fn all_blocks_on_empty_document_returns_empty() {
    let result = parse("");
    assert!(all_blocks(&result.nodes, &result.diagnostics).is_empty());
}
