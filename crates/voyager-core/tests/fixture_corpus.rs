//! Walks `tests/fixtures/valid/**` and `tests/fixtures/broken/**`, asserting
//! SC-001 (zero false positives on valid scripts) and SC-002/SC-003 (every
//! broken fixture correctly flags its injected defect category), per
//! constitution Principle IV and FR-025.
//!
//! Broken fixtures declare which `DiagnosticKind`(s) they expect via a
//! `; EXPECT: Kind1, Kind2` marker on their first line — this is a test-only
//! convention, not part of the crate's grammar.

use std::fs;
use std::path::{Path, PathBuf};

use voyager_core::{parse, DiagnosticKind};

fn is_fixture_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("s") | Some("block")
    )
}

fn collect_fixtures(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(collect_fixtures(&path));
        } else if is_fixture_file(&path) {
            out.push(path);
        }
    }
    out.sort();
    out
}

fn fixtures_dir(sub: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(sub)
}

fn parse_diagnostic_kind(name: &str) -> Option<DiagnosticKind> {
    match name.trim() {
        "UnmatchedIf" => Some(DiagnosticKind::UnmatchedIf),
        "UnmatchedLoop" => Some(DiagnosticKind::UnmatchedLoop),
        "UnclosedBlockComment" => Some(DiagnosticKind::UnclosedBlockComment),
        "InvalidContinuation" => Some(DiagnosticKind::InvalidContinuation),
        "UnmatchedRun" => Some(DiagnosticKind::UnmatchedRun),
        "MisplacedBreak" => Some(DiagnosticKind::MisplacedBreak),
        _ => None,
    }
}

fn expected_kinds(source: &str) -> Vec<DiagnosticKind> {
    let first_line = source.lines().next().unwrap_or("");
    let marker = "; EXPECT:";
    let rest = first_line
        .find(marker)
        .map(|idx| &first_line[idx + marker.len()..])
        .unwrap_or("");
    rest.split(',').filter_map(parse_diagnostic_kind).collect()
}

#[test]
fn valid_fixtures_produce_zero_diagnostics() {
    let dir = fixtures_dir("valid");
    let fixtures = collect_fixtures(&dir);
    assert!(
        !fixtures.is_empty(),
        "expected at least one valid fixture under {dir:?}"
    );
    for path in fixtures {
        let source = fs::read_to_string(&path).expect("fixture should be readable UTF-8 text");
        let result = parse(&source);
        assert!(
            result.diagnostics.is_empty(),
            "expected zero diagnostics for valid fixture {path:?}, got {:#?}",
            result.diagnostics
        );
    }
}

#[test]
fn broken_fixtures_each_produce_their_expected_diagnostic() {
    let dir = fixtures_dir("broken");
    let fixtures = collect_fixtures(&dir);
    assert!(
        !fixtures.is_empty(),
        "expected at least one broken fixture under {dir:?}"
    );
    for path in fixtures {
        let source = fs::read_to_string(&path).expect("fixture should be readable UTF-8 text");
        let expected = expected_kinds(&source);
        assert!(
            !expected.is_empty(),
            "fixture {path:?} is missing a valid '; EXPECT: Kind' marker on its first line"
        );
        let result = parse(&source);
        for kind in &expected {
            assert!(
                result.diagnostics.iter().any(|d| d.kind == *kind),
                "expected a {kind:?} diagnostic for {path:?}, got {:#?}",
                result.diagnostics
            );
        }
    }
}

#[test]
fn every_diagnostic_category_has_at_least_one_broken_fixture() {
    let dir = fixtures_dir("broken");
    let fixtures = collect_fixtures(&dir);
    let mut seen = std::collections::HashSet::new();
    for path in &fixtures {
        let source = fs::read_to_string(path).expect("fixture should be readable UTF-8 text");
        seen.extend(expected_kinds(&source));
    }
    for kind in [
        DiagnosticKind::UnmatchedIf,
        DiagnosticKind::UnmatchedLoop,
        DiagnosticKind::UnclosedBlockComment,
        DiagnosticKind::InvalidContinuation,
        DiagnosticKind::UnmatchedRun,
        DiagnosticKind::MisplacedBreak,
    ] {
        assert!(
            seen.contains(&kind),
            "no broken fixture declares {kind:?} (FR-025)"
        );
    }
}

/// SC-005: at least one bare-fragment `.block` and one self-contained,
/// `RUN`/`ENDRUN`-wrapped `.block` shape are present in the valid corpus.
#[test]
fn block_extension_covers_both_observed_shapes() {
    let dir = fixtures_dir("valid");
    let fixtures = collect_fixtures(&dir);
    let block_files: Vec<&PathBuf> = fixtures
        .iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("block"))
        .collect();
    assert!(
        block_files.len() >= 2,
        "expected at least two .block fixtures (bare-fragment and self-contained shapes)"
    );
    let mut saw_bare_fragment = false;
    let mut saw_self_contained = false;
    for path in block_files {
        let source = fs::read_to_string(path).expect("fixture should be readable UTF-8 text");
        let result = parse(&source);
        let has_top_level_run = result.nodes.iter().any(|n| {
            matches!(n, voyager_core::Node::Block(b) if matches!(b.kind, voyager_core::BlockKind::Run { .. }))
        });
        if has_top_level_run {
            saw_self_contained = true;
        } else {
            saw_bare_fragment = true;
        }
    }
    assert!(
        saw_bare_fragment,
        "expected a bare-fragment .block fixture (no top-level RUN)"
    );
    assert!(
        saw_self_contained,
        "expected a self-contained, RUN-wrapped .block fixture"
    );
}

/// User Story 3 (quickstart.md Scenario 3): `tokenize()` alone exposes
/// comments, `@variable@` references, and continuation markers as distinct,
/// correctly-positioned tokens.
#[test]
fn token_detail_fixture_exposes_expected_token_kinds() {
    let path = fixtures_dir("valid").join("token_detail.s");
    let source = fs::read_to_string(&path).expect("token_detail.s fixture should exist");
    let tokens = voyager_core::tokenize(&source);

    assert!(
        tokens
            .iter()
            .any(|t| t.kind == voyager_core::TokenKind::LineComment),
        "expected a LineComment token"
    );
    assert!(
        tokens.iter().any(
            |t| matches!(t.kind, voyager_core::TokenKind::BlockComment { .. })
                && t.span.start.line != t.span.end.line
        ),
        "expected a multi-line BlockComment token"
    );
    let block_comments: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t.kind, voyager_core::TokenKind::BlockComment { .. }))
        .collect();
    assert!(
        block_comments.len() >= 3,
        "expected the nested comment to yield 2 tokens on top of the multi-line one"
    );
    assert!(
        tokens.iter().any(|t| matches!(&t.kind, voyager_core::TokenKind::VariableRef { name } if name == "ParentDir")),
        "expected a VariableRef token for @ParentDir@"
    );
    assert!(
        tokens.iter().any(|t| t.kind == voyager_core::TokenKind::ContinuationMarker),
        "expected a ContinuationMarker token (the @variable@ reference splits across a continuation)"
    );
}
