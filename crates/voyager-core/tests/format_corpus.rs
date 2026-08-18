//! Golden-file, idempotency, and structural-equivalence checks for
//! `format`/`format_bytes` against the fixture corpus (spec.md FR-021 in
//! `002-cli-check-format`; constitution Principle III).
//!
//! Golden files live in `tests/fixtures/golden/`, mirroring
//! `tests/fixtures/valid/`'s layout — **including `real_corpus/`**, whose
//! golden copies needed human review before they could exist at all (T023b;
//! see `specs/002-cli-check-format/tasks.md`'s "Human-in-the-loop
//! dependency" — all 9 real files were reviewed, one required a lexer fix
//! first, see `001-voyager-script-parser/spec.md`'s FR-004/FR-005 amendment
//! and Assumptions entry). `encoding_fallback/` (T025's `Recovered`/`Lossy`
//! fixtures) is covered by its own tests below, separately from the
//! golden-diff mechanism, since there's no single "golden text" for a file
//! `format` refuses to persist.
//!
//! Regenerate golden files (only after reviewing the diff!) with:
//! `UPDATE_GOLDEN=1 cargo test -p voyager-core --test format_corpus`

use std::fs;
use std::path::{Path, PathBuf};

use voyager_core::format::{format_bytes, CasingConvention, CasingSettings, EncodingFidelity, FormatOptions};
use voyager_core::{parse, BlankLineMode, BlockKind, Node, OperatorSpacing, Statement, StatementKind, TopLevelIndentMode};

const VALID_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/valid");
const GOLDEN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/golden");
const REAL_CORPUS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/valid/real_corpus");
const REAL_CORPUS_GOLDEN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/golden/real_corpus");
const ENCODING_FALLBACK_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/encoding_fallback");

// 009-top-level-indent-toggle: a second, separate fixture set holding 008's
// already-committed, already-human-reviewed golden output verbatim (copied
// before GOLDEN_DIR was regenerated to preserve-mode output) -- proves
// explicit Normalize mode reproduces 008's shipped behavior exactly, with
// no second human-review pass needed since this content never changed.
const GOLDEN_NORMALIZE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/golden_normalize");
const REAL_CORPUS_GOLDEN_NORMALIZE_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/golden_normalize/real_corpus");

// -- 017-casing-categories-indent-width, T036: a third variant, mirroring
// golden_normalize/'s own precedent exactly -- applied only to the 9
// already-reviewed real_corpus fixtures (not a new redaction review; same
// underlying already-approved source text, just reformatted differently,
// the same reasoning golden_normalize/'s own module comment already uses).
const REAL_CORPUS_GOLDEN_DATA_REFERENCES_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/golden_data_references/real_corpus");

fn data_references_upper_indent_2_options() -> FormatOptions {
    FormatOptions {
        casing: CasingSettings {
            data_references: CasingConvention::Upper,
            ..CasingSettings::default()
        },
        top_level_indent: TopLevelIndentMode::default(),
        indent_width: 2,
        operator_spacing: OperatorSpacing::default(),
        ..FormatOptions::default()
    }
}

// -- 018-operator-spacing, T031: two more variants, same golden-directory
// pattern golden_normalize/ and golden_data_references/ already established
// -- applied only to the 9 already-reviewed real_corpus fixtures.
const REAL_CORPUS_GOLDEN_OPERATOR_SPACING_FIXED_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/golden_operator_spacing_fixed/real_corpus");
const REAL_CORPUS_GOLDEN_OPERATOR_SPACING_AUTO_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/golden_operator_spacing_auto/real_corpus");

fn operator_spacing_fixed_options() -> FormatOptions {
    FormatOptions {
        casing: CasingSettings::default(),
        top_level_indent: TopLevelIndentMode::default(),
        indent_width: 4,
        operator_spacing: OperatorSpacing::Fixed,
        ..FormatOptions::default()
    }
}

fn operator_spacing_auto_options() -> FormatOptions {
    FormatOptions {
        casing: CasingSettings::default(),
        top_level_indent: TopLevelIndentMode::default(),
        indent_width: 4,
        operator_spacing: OperatorSpacing::Auto,
        ..FormatOptions::default()
    }
}

// -- 019-blank-line-normalization, T025: one more variant, same
// golden-directory pattern already established -- applied only to the 9
// already-reviewed real_corpus fixtures.
const REAL_CORPUS_GOLDEN_BLANK_LINES_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/golden_blank_lines/real_corpus");

fn blank_lines_auto_options() -> FormatOptions {
    FormatOptions {
        casing: CasingSettings::default(),
        top_level_indent: TopLevelIndentMode::default(),
        indent_width: 4,
        operator_spacing: OperatorSpacing::default(),
        blank_lines: BlankLineMode::Auto,
        ..FormatOptions::default()
    }
}

fn normalize_options() -> FormatOptions {
    FormatOptions {
        casing: CasingSettings::default(),
        top_level_indent: TopLevelIndentMode::Normalize,
        indent_width: 4,
        operator_spacing: OperatorSpacing::default(),
        ..FormatOptions::default()
    }
}

fn script_files_in(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let entry = entry.expect("readable dir entry");
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext.eq_ignore_ascii_case("s") || ext.eq_ignore_ascii_case("block") {
            out.push(path);
        }
    }
    out
}

/// Hand-written, project-authored fixtures only — `real_corpus/` (a
/// directory) is skipped by construction here.
fn hand_written_fixtures() -> Vec<PathBuf> {
    let mut out = script_files_in(Path::new(VALID_DIR));
    out.sort();
    out
}

fn golden_path_for(fixture: &Path) -> PathBuf {
    Path::new(GOLDEN_DIR).join(fixture.file_name().expect("fixture has a filename"))
}

/// The 9 real, curated, redaction-checked WF-TDM files (recursively —
/// `real_corpus/` has its own subdirectories, e.g. `AssignHwy/`,
/// `Distribute/`), reviewed and approved per T023b.
fn real_corpus_fixtures() -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let entry = entry.expect("readable dir entry");
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext.eq_ignore_ascii_case("s") || ext.eq_ignore_ascii_case("block") {
                    out.push(path);
                }
            }
        }
    }
    walk(Path::new(REAL_CORPUS_DIR), &mut out);
    out.sort();
    out
}

/// Mirrors `real_corpus/`'s own subdirectory structure under
/// `golden/real_corpus/`, since (unlike the flat hand-written set) its
/// filenames aren't guaranteed unique across subdirectories.
fn golden_path_for_real_corpus(fixture: &Path) -> PathBuf {
    let rel = fixture
        .strip_prefix(REAL_CORPUS_DIR)
        .expect("real_corpus_fixtures() only returns paths under REAL_CORPUS_DIR");
    Path::new(REAL_CORPUS_GOLDEN_DIR).join(rel)
}

fn check_golden(
    fixtures: Vec<PathBuf>,
    golden_dir: &Path,
    golden_path_for: impl Fn(&Path) -> PathBuf,
    options: FormatOptions,
) {
    let update = std::env::var_os("UPDATE_GOLDEN").is_some();
    if update {
        fs::create_dir_all(golden_dir).unwrap();
    }

    let mut mismatches = Vec::new();
    for fixture in fixtures {
        let bytes = fs::read(&fixture).unwrap();
        let result = format_bytes(&bytes, options);
        let golden_path = golden_path_for(&fixture);

        if update {
            fs::create_dir_all(golden_path.parent().unwrap()).unwrap();
            fs::write(&golden_path, &result.text).unwrap();
            continue;
        }

        let expected = fs::read_to_string(&golden_path).unwrap_or_else(|_| {
            panic!(
                "no golden file at {} — run `UPDATE_GOLDEN=1 cargo test -p voyager-core --test format_corpus` \
                 to generate it (review the diff before committing)",
                golden_path.display()
            )
        });
        if result.text != expected {
            mismatches.push(fixture.display().to_string());
        }
    }

    assert!(
        mismatches.is_empty(),
        "formatted output drifted from its golden copy for: {mismatches:#?}\n\
         run with UPDATE_GOLDEN=1 to review and regenerate if this drift is intentional"
    );
}

fn check_idempotent(fixtures: Vec<PathBuf>, options: FormatOptions) {
    for fixture in fixtures {
        let bytes = fs::read(&fixture).unwrap();
        let once = format_bytes(&bytes, options);
        let twice = format_bytes(once.text.as_bytes(), options);
        assert_eq!(once.text, twice.text, "not idempotent: {}", fixture.display());
        assert!(
            !twice.changed,
            "second format pass must be a no-op: {}",
            fixture.display()
        );
    }
}

fn check_structure_and_diagnostics_preserved(fixtures: Vec<PathBuf>, options: FormatOptions) {
    for fixture in fixtures {
        let bytes = fs::read(&fixture).unwrap();
        let source = String::from_utf8_lossy(&bytes).into_owned();
        let formatted = format_bytes(&bytes, options);

        let before = parse(&source);
        let after = parse(&formatted.text);

        assert_eq!(
            shape_signature(&before.nodes),
            shape_signature(&after.nodes),
            "statement/block shape changed: {}",
            fixture.display()
        );

        let mut before_kinds: Vec<_> = before.diagnostics.iter().map(|d| d.kind).collect();
        let mut after_kinds: Vec<_> = after.diagnostics.iter().map(|d| d.kind).collect();
        before_kinds.sort_by_key(|k| format!("{k:?}"));
        after_kinds.sort_by_key(|k| format!("{k:?}"));
        assert_eq!(
            before_kinds, after_kinds,
            "diagnostic kinds changed: {}",
            fixture.display()
        );
    }
}

#[test]
fn hand_written_fixtures_match_golden_output() {
    check_golden(hand_written_fixtures(), Path::new(GOLDEN_DIR), golden_path_for, FormatOptions::default());
}

#[test]
fn hand_written_fixtures_are_idempotent() {
    check_idempotent(hand_written_fixtures(), FormatOptions::default());
}

#[test]
fn hand_written_fixtures_preserve_structure_and_diagnostics() {
    check_structure_and_diagnostics_preserved(hand_written_fixtures(), FormatOptions::default());
}

#[test]
fn real_corpus_fixtures_match_golden_output() {
    check_golden(
        real_corpus_fixtures(),
        Path::new(REAL_CORPUS_GOLDEN_DIR),
        golden_path_for_real_corpus,
        FormatOptions::default(),
    );
}

#[test]
fn real_corpus_fixtures_are_idempotent() {
    check_idempotent(real_corpus_fixtures(), FormatOptions::default());
}

#[test]
fn real_corpus_fixtures_preserve_structure_and_diagnostics() {
    check_structure_and_diagnostics_preserved(real_corpus_fixtures(), FormatOptions::default());
}

#[test]
fn real_corpus_fixture_count_is_the_known_nine() {
    // A tripwire: if this ever drifts, someone added/removed a real_corpus
    // file without updating this comment (or the golden set silently grew
    // stale) — see T023b's provenance note.
    assert_eq!(real_corpus_fixtures().len(), 9);
}

// -- 009-top-level-indent-toggle: explicit Normalize mode reproduces 008's
// already-committed, already-human-reviewed golden_normalize/ output
// byte-for-byte (FR-006/SC-002) -- no second human-review pass needed,
// since golden_normalize/ is a verbatim copy of what 008 already had
// reviewed, never regenerated by this feature's own work.

fn golden_normalize_path_for(fixture: &Path) -> PathBuf {
    Path::new(GOLDEN_NORMALIZE_DIR).join(fixture.file_name().expect("fixture has a filename"))
}

fn golden_normalize_path_for_real_corpus(fixture: &Path) -> PathBuf {
    let rel = fixture
        .strip_prefix(REAL_CORPUS_DIR)
        .expect("real_corpus_fixtures() only returns paths under REAL_CORPUS_DIR");
    Path::new(REAL_CORPUS_GOLDEN_NORMALIZE_DIR).join(rel)
}

#[test]
fn hand_written_fixtures_match_golden_output_under_normalize() {
    check_golden(
        hand_written_fixtures(),
        Path::new(GOLDEN_NORMALIZE_DIR),
        golden_normalize_path_for,
        normalize_options(),
    );
}

#[test]
fn real_corpus_fixtures_match_golden_output_under_normalize() {
    check_golden(
        real_corpus_fixtures(),
        Path::new(REAL_CORPUS_GOLDEN_NORMALIZE_DIR),
        golden_normalize_path_for_real_corpus,
        normalize_options(),
    );
}

#[test]
fn real_corpus_fixtures_are_idempotent_under_normalize() {
    check_idempotent(real_corpus_fixtures(), normalize_options());
}

// -- 017-casing-categories-indent-width, T036 --

fn golden_data_references_path_for_real_corpus(fixture: &Path) -> PathBuf {
    let rel = fixture
        .strip_prefix(REAL_CORPUS_DIR)
        .expect("real_corpus_fixtures() only returns paths under REAL_CORPUS_DIR");
    Path::new(REAL_CORPUS_GOLDEN_DATA_REFERENCES_DIR).join(rel)
}

#[test]
fn real_corpus_fixtures_match_golden_output_under_data_references_upper_indent_2() {
    check_golden(
        real_corpus_fixtures(),
        Path::new(REAL_CORPUS_GOLDEN_DATA_REFERENCES_DIR),
        golden_data_references_path_for_real_corpus,
        data_references_upper_indent_2_options(),
    );
}

#[test]
fn real_corpus_fixtures_are_idempotent_under_data_references_upper_indent_2() {
    check_idempotent(real_corpus_fixtures(), data_references_upper_indent_2_options());
}

#[test]
fn real_corpus_fixtures_preserve_structure_and_diagnostics_under_data_references_upper_indent_2() {
    check_structure_and_diagnostics_preserved(real_corpus_fixtures(), data_references_upper_indent_2_options());
}

// -- 018-operator-spacing, T031 --

fn golden_operator_spacing_fixed_path_for_real_corpus(fixture: &Path) -> PathBuf {
    let rel = fixture
        .strip_prefix(REAL_CORPUS_DIR)
        .expect("real_corpus_fixtures() only returns paths under REAL_CORPUS_DIR");
    Path::new(REAL_CORPUS_GOLDEN_OPERATOR_SPACING_FIXED_DIR).join(rel)
}

#[test]
fn real_corpus_fixtures_match_golden_output_under_operator_spacing_fixed() {
    check_golden(
        real_corpus_fixtures(),
        Path::new(REAL_CORPUS_GOLDEN_OPERATOR_SPACING_FIXED_DIR),
        golden_operator_spacing_fixed_path_for_real_corpus,
        operator_spacing_fixed_options(),
    );
}

#[test]
fn real_corpus_fixtures_are_idempotent_under_operator_spacing_fixed() {
    check_idempotent(real_corpus_fixtures(), operator_spacing_fixed_options());
}

#[test]
fn real_corpus_fixtures_preserve_structure_and_diagnostics_under_operator_spacing_fixed() {
    check_structure_and_diagnostics_preserved(real_corpus_fixtures(), operator_spacing_fixed_options());
}

fn golden_operator_spacing_auto_path_for_real_corpus(fixture: &Path) -> PathBuf {
    let rel = fixture
        .strip_prefix(REAL_CORPUS_DIR)
        .expect("real_corpus_fixtures() only returns paths under REAL_CORPUS_DIR");
    Path::new(REAL_CORPUS_GOLDEN_OPERATOR_SPACING_AUTO_DIR).join(rel)
}

#[test]
fn real_corpus_fixtures_match_golden_output_under_operator_spacing_auto() {
    check_golden(
        real_corpus_fixtures(),
        Path::new(REAL_CORPUS_GOLDEN_OPERATOR_SPACING_AUTO_DIR),
        golden_operator_spacing_auto_path_for_real_corpus,
        operator_spacing_auto_options(),
    );
}

#[test]
fn real_corpus_fixtures_are_idempotent_under_operator_spacing_auto() {
    check_idempotent(real_corpus_fixtures(), operator_spacing_auto_options());
}

#[test]
fn real_corpus_fixtures_preserve_structure_and_diagnostics_under_operator_spacing_auto() {
    check_structure_and_diagnostics_preserved(real_corpus_fixtures(), operator_spacing_auto_options());
}

// -- 019-blank-line-normalization, T025 --

fn golden_blank_lines_path_for_real_corpus(fixture: &Path) -> PathBuf {
    let rel = fixture
        .strip_prefix(REAL_CORPUS_DIR)
        .expect("real_corpus_fixtures() only returns paths under REAL_CORPUS_DIR");
    Path::new(REAL_CORPUS_GOLDEN_BLANK_LINES_DIR).join(rel)
}

#[test]
fn real_corpus_fixtures_match_golden_output_under_blank_lines_auto() {
    check_golden(
        real_corpus_fixtures(),
        Path::new(REAL_CORPUS_GOLDEN_BLANK_LINES_DIR),
        golden_blank_lines_path_for_real_corpus,
        blank_lines_auto_options(),
    );
}

#[test]
fn real_corpus_fixtures_are_idempotent_under_blank_lines_auto() {
    check_idempotent(real_corpus_fixtures(), blank_lines_auto_options());
}

#[test]
fn real_corpus_fixtures_preserve_structure_and_diagnostics_under_blank_lines_auto() {
    check_structure_and_diagnostics_preserved(real_corpus_fixtures(), blank_lines_auto_options());
}

/// A structural "shape" — statement kinds/words/pair-keys and block
/// kinds/nesting, in order, deliberately ignoring `Span` (which shifts by
/// construction whenever whitespace width changes) and casing (uppercased
/// for comparison, since a casing difference is exactly what FR-015 is
/// allowed to introduce and is covered by its own tests, not this one).
fn shape_signature(nodes: &[Node]) -> String {
    node_sigs(nodes)
}

fn node_sigs(nodes: &[Node]) -> String {
    nodes.iter().map(node_sig).collect::<Vec<_>>().join(",")
}

fn node_sig(node: &Node) -> String {
    match node {
        Node::Statement(s) => stmt_sig(s),
        Node::Block(b) => match &b.kind {
            BlockKind::If { branches } => format!(
                "If[{}]",
                branches
                    .iter()
                    .map(|br| format!("({})", node_sigs(&br.children)))
                    .collect::<Vec<_>>()
                    .join(";")
            ),
            BlockKind::Loop {} => format!("Loop[{}]", node_sigs(&b.children)),
            BlockKind::Run { pgm, disabled } => format!(
                "Run({},{})[{}]",
                pgm.as_deref().unwrap_or("").to_ascii_uppercase(),
                disabled,
                node_sigs(&b.children)
            ),
            BlockKind::Process { name } => format!(
                "Process({})[{}]",
                name.as_deref().unwrap_or("").to_ascii_uppercase(),
                node_sigs(&b.children)
            ),
            BlockKind::JLoop {} => format!("JLoop[{}]", node_sigs(&b.children)),
            BlockKind::LinkLoop {} => format!("LinkLoop[{}]", node_sigs(&b.children)),
            BlockKind::DistributeMultistep { process_num } => format!(
                "DMS({})[{}]",
                process_num.as_deref().unwrap_or("").to_ascii_uppercase(),
                node_sigs(&b.children)
            ),
        },
    }
}

fn stmt_sig(s: &Statement) -> String {
    match &s.kind {
        StatementKind::Control { word, pairs } => format!(
            "C:{}({})",
            word.to_ascii_uppercase(),
            pairs
                .iter()
                .map(|(k, _)| k.to_ascii_uppercase())
                .collect::<Vec<_>>()
                .join(",")
        ),
        // Uppercased for the same reason Control's word/pairs already are:
        // 017-casing-categories-indent-width's data_references category can
        // legitimately recase an assignment target (e.g. `mw[1]` ->
        // `MW[1]`) — a real gap this file's own new data-references golden
        // variant surfaced, not a pre-existing case that happened to never
        // trigger before this feature existed.
        StatementKind::Assignment { target, .. } => format!("A:{}", target.to_ascii_uppercase()),
        StatementKind::Label { name } => format!("L:{name}"),
        StatementKind::ShellEscape { .. } => "S".to_string(),
    }
}

// -- Encoding fallback (T025 fixtures; FR-013(b), FR-024, FR-025) ----------

#[test]
fn recovered_fixture_is_classified_recovered_and_written_through() {
    let path = Path::new(ENCODING_FALLBACK_DIR).join("recovered.s");
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let result = format_bytes(&bytes, FormatOptions::default());
    assert_eq!(result.encoding_fidelity, EncodingFidelity::Recovered);
    assert!(result.diagnostics.is_empty(), "a recovered byte produces no diagnostic");
}

#[test]
fn lossy_fixture_is_classified_lossy_and_diagnosed() {
    let path = Path::new(ENCODING_FALLBACK_DIR).join("lossy.s");
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let result = format_bytes(&bytes, FormatOptions::default());
    assert_eq!(result.encoding_fidelity, EncodingFidelity::Lossy);
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.kind == voyager_core::DiagnosticKind::InvalidEncoding));
}
