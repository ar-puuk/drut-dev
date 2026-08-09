//! Walks `tests/fixtures/valid/**` and `tests/fixtures/broken/**`, asserting
//! SC-001 (zero false positives on valid scripts) and SC-002/SC-003 (every
//! broken fixture correctly flags its injected defect category), per
//! constitution Principle IV and FR-025.
//!
//! Fixtures are read as raw bytes and parsed via `parse_bytes` (FR-034)
//! uniformly, not `fs::read_to_string`/`parse` — real production Voyager
//! scripts are not guaranteed to be valid UTF-8 (T049's real fixture corpus
//! found exactly one that wasn't), and for pure-UTF-8 fixtures the two paths
//! are equivalent, so there's no reason to special-case one file.
//!
//! Broken fixtures declare which `DiagnosticKind`(s) they expect via a
//! `; EXPECT: Kind1, Kind2` marker on their first line — this is a test-only
//! convention, not part of the crate's grammar. The marker itself is always
//! pure ASCII, so it's read via a lossy decode even for the one fixture whose
//! *content* is deliberately not valid UTF-8.

use std::fs;
use std::path::{Path, PathBuf};

use voyager_core::{parse_bytes, DiagnosticKind};

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

fn read_fixture_bytes(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| panic!("fixture {path:?} should be readable: {e}"))
}

fn parse_diagnostic_kind(name: &str) -> Option<DiagnosticKind> {
    match name.trim() {
        "UnmatchedIf" => Some(DiagnosticKind::UnmatchedIf),
        "UnmatchedLoop" => Some(DiagnosticKind::UnmatchedLoop),
        "UnclosedBlockComment" => Some(DiagnosticKind::UnclosedBlockComment),
        "InvalidContinuation" => Some(DiagnosticKind::InvalidContinuation),
        "UnmatchedRun" => Some(DiagnosticKind::UnmatchedRun),
        "MisplacedBreak" => Some(DiagnosticKind::MisplacedBreak),
        "InvalidEncoding" => Some(DiagnosticKind::InvalidEncoding),
        _ => None,
    }
}

fn expected_kinds(bytes: &[u8]) -> Vec<DiagnosticKind> {
    // The marker line is always pure ASCII by convention; a lossy decode is
    // fine here even for the one fixture whose real content isn't UTF-8.
    let text = String::from_utf8_lossy(bytes);
    let first_line = text.lines().next().unwrap_or("");
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
        let bytes = read_fixture_bytes(&path);
        let result = parse_bytes(&bytes);
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
        let bytes = read_fixture_bytes(&path);
        let expected = expected_kinds(&bytes);
        assert!(
            !expected.is_empty(),
            "fixture {path:?} is missing a valid '; EXPECT: Kind' marker on its first line"
        );
        let result = parse_bytes(&bytes);
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
        seen.extend(expected_kinds(&read_fixture_bytes(path)));
    }
    for kind in [
        DiagnosticKind::UnmatchedIf,
        DiagnosticKind::UnmatchedLoop,
        DiagnosticKind::UnclosedBlockComment,
        DiagnosticKind::InvalidContinuation,
        DiagnosticKind::UnmatchedRun,
        DiagnosticKind::MisplacedBreak,
        DiagnosticKind::InvalidEncoding,
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
        let bytes = read_fixture_bytes(path);
        let result = parse_bytes(&bytes);
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
    let bytes = read_fixture_bytes(&path);
    let tokens = voyager_core::tokenize_bytes(&bytes);

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

/// T049 / FR-034: the one real, non-UTF-8 file found in the real fixture
/// corpus (a single Windows-1252 byte inside a comment) decodes silently —
/// zero diagnostics, since that byte resolves successfully under the
/// fallback encoding.
#[test]
fn real_non_utf8_fixture_decodes_silently() {
    let path = fixtures_dir("valid").join("real_corpus/Distribute/4pd_mainbody_distribution.block");
    let bytes = read_fixture_bytes(&path);
    assert!(
        std::str::from_utf8(&bytes).is_err(),
        "this fixture is expected to contain a genuine non-UTF-8 byte; if this fails, the file changed"
    );
    let result = parse_bytes(&bytes);
    assert!(
        result.diagnostics.is_empty(),
        "expected the one real non-UTF-8 byte to decode silently under Windows-1252 fallback, got {:#?}",
        result.diagnostics
    );
}

/// FR-023 (subscripted assignment targets): a plain "zero diagnostics" check
/// would NOT have caught the classify_statement bug this fixture regresses —
/// misclassifying `MW[1] = ...` as `Control{word:"MW"}` instead of
/// `Assignment` produces zero diagnostics either way, since "MW" isn't a
/// recognized block keyword. `StatementKind` has to be checked directly.
#[test]
fn subscripted_assignment_targets_are_classified_as_assignment() {
    let path = fixtures_dir("valid").join("subscripted_assignment_targets.s");
    let bytes = read_fixture_bytes(&path);
    let result = parse_bytes(&bytes);
    assert!(
        result.diagnostics.is_empty(),
        "got {:#?}",
        result.diagnostics
    );

    // The fixture's one top-level node is the RUN block; walk its children.
    let run_block = result
        .nodes
        .iter()
        .find_map(|n| match n {
            voyager_core::Node::Block(b)
                if matches!(b.kind, voyager_core::BlockKind::Run { .. }) =>
            {
                Some(b)
            }
            _ => None,
        })
        .expect("expected a top-level RUN block");

    let targets: Vec<&str> = run_block
        .children
        .iter()
        .filter_map(|n| match n {
            voyager_core::Node::Statement(s) => match &s.kind {
                voyager_core::StatementKind::Assignment { target, .. } => Some(target.as_str()),
                _ => None,
            },
            _ => None,
        })
        .collect();

    for expected in [
        "MW[1]",
        "MW[2]",
        "MW[3]",
        "SUBAREAID[Seg_Idx][idx_SUBAREAID]",
    ] {
        assert!(
            targets.contains(&expected),
            "expected an Assignment with target {expected:?}, got targets {targets:?}"
        );
    }
    assert!(
        !targets.iter().any(|t| *t == "MW" || *t == "SUBAREAID"),
        "a subscripted target must never be classified with just the bare identifier as its target"
    );
}
