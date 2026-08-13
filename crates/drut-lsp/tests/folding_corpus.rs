//! Full-corpus proof for 011-code-folding's SC-003, gated behind
//! `DRUT_CORPUS_PATH` and `#[ignore]`'d unconditionally — the same
//! three-state gating `diagnostics_corpus.rs` already establishes.
//!
//! Two independent full-corpus assertions, matching SC-003's own "block
//! **or** comment" wording (T012, extended during `/speckit-analyze`
//! remediation to close finding E1 — the original draft covered only (1)):
//!
//! 1. **Blocks**: `voyager_core::all_blocks` and the existing
//!    `voyager_core::block_at` agree with each other on every real block in
//!    every corpus file — proof that the new full-document enumeration
//!    function never diverges from the already-trusted single-position
//!    query it's built on top of (research.md §1).
//! 2. **Block comments**: every terminated, multi-line `BlockComment` token
//!    across the corpus is confirmed foldable and every unterminated or
//!    single-line one is confirmed not, matching FR-006/FR-007/FR-008 —
//!    proof that the comment-folding half of this feature (which has no
//!    second `voyager-core` entry point to cross-check against, unlike
//!    blocks) is still verified at full corpus scale, not only at the
//!    hand-written-fixture level (T006).

use std::path::{Path, PathBuf};
use std::str::FromStr;

use drut_lsp::document_store::ServerState;
use lsp_types::Uri;
use voyager_core::{all_blocks, block_at, parse, tokenize, TokenKind};

fn corpus_path() -> PathBuf {
    match std::env::var("DRUT_CORPUS_PATH") {
        Ok(value) if !value.trim().is_empty() => PathBuf::from(value),
        _ => panic!(
            "set DRUT_CORPUS_PATH to a local WF-TDM-Official-Releases checkout to run this test \
             (e.g. $env:DRUT_CORPUS_PATH = \"D:\\GitHub\\WF-TDM-Official-Releases\")"
        ),
    }
}

fn collect_script_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_script_files(&path, out);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("s") || ext.eq_ignore_ascii_case("block"))
        {
            out.push(path);
        }
    }
}

#[test]
#[ignore = "requires DRUT_CORPUS_PATH pointing at a local WF-TDM-Official-Releases checkout"]
fn all_blocks_agrees_with_block_at_across_the_full_corpus() {
    let corpus = corpus_path();
    let mut files = Vec::new();
    collect_script_files(&corpus, &mut files);
    assert!(!files.is_empty(), "expected at least one .s/.block file under {corpus:?}");

    let mut disagreements = Vec::new();
    let mut blocks_checked: usize = 0;

    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue; // non-UTF-8 on disk: not this test's concern (research.md §12).
        };
        let result = parse(&text);
        for fold in all_blocks(&result.nodes, &result.diagnostics) {
            blocks_checked += 1;
            let Some(via_block_at) = block_at(&result.nodes, &result.diagnostics, fold.opener) else {
                disagreements.push(format!(
                    "{path:?}: all_blocks reported a block at {:?} but block_at found none there",
                    fold.opener
                ));
                continue;
            };
            if via_block_at.counterpart != fold.info.counterpart {
                disagreements.push(format!(
                    "{path:?}: at {:?}, all_blocks counterpart={:?} but block_at counterpart={:?}",
                    fold.opener, fold.info.counterpart, via_block_at.counterpart
                ));
            }
            if via_block_at.kind != fold.info.kind {
                disagreements.push(format!(
                    "{path:?}: at {:?}, all_blocks kind={:?} but block_at kind={:?}",
                    fold.opener, fold.info.kind, via_block_at.kind
                ));
            }
        }
    }

    assert!(
        disagreements.is_empty(),
        "expected zero disagreements between all_blocks and block_at across {} blocks in {} file(s), \
         got {} disagreement(s):\n{}",
        blocks_checked,
        files.len(),
        disagreements.len(),
        disagreements.join("\n")
    );
}

#[test]
#[ignore = "requires DRUT_CORPUS_PATH pointing at a local WF-TDM-Official-Releases checkout"]
fn block_comment_foldability_is_correct_across_the_full_corpus() {
    let corpus = corpus_path();
    let mut files = Vec::new();
    collect_script_files(&corpus, &mut files);
    assert!(!files.is_empty(), "expected at least one .s/.block file under {corpus:?}");

    let mut failures = Vec::new();
    let mut comments_checked: usize = 0;

    for (i, path) in files.iter().enumerate() {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue; // non-UTF-8 on disk: not this test's concern (research.md §12).
        };

        // Independently derive the expected foldable-comment start lines
        // straight from the token stream (FR-006/FR-007/FR-008's rule:
        // terminated AND spans more than one line), then confirm the real
        // handler — not a reimplementation of its logic — produces exactly
        // that set, end to end through ServerState.
        let mut expected_start_lines: Vec<u32> = Vec::new();
        for token in tokenize(&text) {
            let TokenKind::BlockComment { unterminated } = token.kind else {
                continue;
            };
            comments_checked += 1;
            let is_multi_line = token.span.start.line != token.span.end.line;
            if !unterminated && is_multi_line {
                expected_start_lines.push(token.span.start.line - 1); // 1-based -> 0-based (to_lsp_position).
            }
        }

        let mut state = ServerState::new();
        let uri = Uri::from_str(&format!("file:///corpus-{i}.s")).unwrap();
        state.did_open(uri.clone(), text, 1);
        let params = lsp_types::FoldingRangeParams {
            text_document: lsp_types::TextDocumentIdentifier { uri },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        let ranges = drut_lsp::folding::handle(&state, &params).unwrap_or_default();
        let actual_start_lines: Vec<u32> = ranges
            .iter()
            .filter(|r| r.kind == Some(lsp_types::FoldingRangeKind::Comment))
            .map(|r| r.start_line)
            .collect();

        if actual_start_lines != expected_start_lines {
            failures.push(format!(
                "{path:?}: expected comment-range start lines {expected_start_lines:?}, got {actual_start_lines:?}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "expected zero mismatches across {} block comments in {} file(s), got {} failure(s):\n{}",
        comments_checked,
        files.len(),
        failures.len(),
        failures.join("\n")
    );
}
