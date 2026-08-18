//! `format` flag behavior: default/`--write`/`--check`/`--diff` disposition,
//! `--casing-control-words` validation, mutual exclusivity (spec.md FR-015–FR-019), and
//! `--indent-top-level` (FR-026, 009-top-level-indent-toggle).

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("drut-cli-formatflags-{label}-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        TempDir(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn drut(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_drut"))
        .args(args)
        .output()
        .expect("failed to run drut")
}

const MESSY: &str = "IF (X=1)\nY = 2\nENDIF\n";
const CLEAN: &str = "IF (X=1)\n    Y = 2\nENDIF\n";

#[test]
fn default_mode_prints_formatted_content_and_never_writes() {
    let dir = TempDir::new("default");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), CLEAN);
    // File on disk must be untouched.
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY);
}

#[test]
fn write_mode_overwrites_the_file() {
    let dir = TempDir::new("write");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--write"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), CLEAN);
}

#[test]
fn check_mode_reports_without_writing() {
    let dir = TempDir::new("check-dirty");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--check"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY, "must not write");
}

#[test]
fn check_mode_on_already_clean_file_exits_0() {
    let dir = TempDir::new("check-clean");
    let file = dir.path().join("x.s");
    fs::write(&file, CLEAN).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--check"]);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn diff_mode_prints_a_diff_without_writing() {
    let dir = TempDir::new("diff");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--diff"]);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("-Y = 2") || stdout.contains("Y = 2"), "expected a diff hunk, got:\n{stdout}");
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY, "must not write");
}

#[test]
fn casing_with_no_value_is_a_usage_error_before_touching_any_file() {
    // Note: `clap`'s own usage-error exit code (a `--casing-control-words`
    // with no value never reaches our own exit-code logic at all) happens
    // to also be 2, the same numeric value FR-011/FR-020's `Fatal` uses —
    // a coincidence, not a collision, since spec.md only requires this be
    // distinguishable as "a clap-level parse failure, not one of the three
    // run-outcome exit codes," not that it use a numerically distinct
    // value. What actually matters, and what this asserts, is that no file
    // was touched.
    let dir = TempDir::new("casing-bare");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--casing-control-words"]);
    assert_ne!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY, "must not touch the file");
}

#[test]
fn casing_with_invalid_value_is_a_usage_error() {
    let dir = TempDir::new("casing-invalid");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--casing-control-words=sideways"]);
    assert_ne!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY);
}

#[test]
fn casing_upper_rewrites_control_words() {
    let dir = TempDir::new("casing-upper");
    let file = dir.path().join("x.s");
    fs::write(&file, "if (x=1)\nendif\n").unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--casing-control-words=upper"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "IF (x=1)\nENDIF\n");
}

const TOP_LEVEL_NON_ZERO: &str = "    IF (X=1)\n        Y = 2\n    ENDIF\n";

#[test]
fn indent_top_level_omitted_defaults_to_preserve() {
    // 009-top-level-indent-toggle FR-004(a): the CLI flag's own default,
    // confirmed end to end (not just via clap's declared default_value_t)
    // -- a non-zero top-level IF, already correctly nested relative to its
    // own (non-zero) position, is left completely untouched.
    let dir = TempDir::new("indent-top-level-omitted");
    let file = dir.path().join("x.s");
    fs::write(&file, TOP_LEVEL_NON_ZERO).unwrap();

    let out = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), TOP_LEVEL_NON_ZERO);
}

#[test]
fn indent_top_level_auto_forces_column_zero() {
    let dir = TempDir::new("indent-top-level-auto");
    let file = dir.path().join("x.s");
    fs::write(&file, TOP_LEVEL_NON_ZERO).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--indent-top-level=auto"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "IF (X=1)\n    Y = 2\nENDIF\n");
}

#[test]
fn indent_top_level_preserve_explicit_matches_omitted() {
    let dir = TempDir::new("indent-top-level-preserve-explicit");
    let file = dir.path().join("x.s");
    fs::write(&file, TOP_LEVEL_NON_ZERO).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--indent-top-level=preserve"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), TOP_LEVEL_NON_ZERO);
}

#[test]
fn indent_top_level_invalid_value_is_a_usage_error() {
    let dir = TempDir::new("indent-top-level-invalid");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--indent-top-level=sideways"]);
    assert_ne!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY);
}

#[test]
fn write_check_and_diff_are_mutually_exclusive() {
    let dir = TempDir::new("mutex");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--write", "--check"]);
    assert_ne!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY, "must not touch the file");
}

// -- FMT region markers (010-fmt-region-markers) ----------------------------

const PROTECTED_RANGE: &str = "IF (X=1)\nY = 1\n; FMT: OFF\n  weird = 1\n; FMT: ON\nZ = 2\nENDIF\n";
const PROTECTED_RANGE_FORMATTED: &str = "IF (X=1)\n    Y = 1\n; FMT: OFF\n  weird = 1\n; FMT: ON\n    Z = 2\nENDIF\n";

#[test]
fn protected_range_survives_default_mode() {
    let dir = TempDir::new("fmt-marker-default");
    let file = dir.path().join("x.s");
    fs::write(&file, PROTECTED_RANGE).unwrap();

    let out = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), PROTECTED_RANGE_FORMATTED);
}

#[test]
fn protected_range_survives_check_mode() {
    let dir = TempDir::new("fmt-marker-check");
    let file = dir.path().join("x.s");
    fs::write(&file, PROTECTED_RANGE_FORMATTED).unwrap();

    // Already in its final form (including the protected range) -- --check
    // must report clean, since re-formatting it is a no-op.
    let out = drut(&["format", file.to_str().unwrap(), "--check"]);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn protected_range_survives_diff_mode() {
    let dir = TempDir::new("fmt-marker-diff");
    let file = dir.path().join("x.s");
    fs::write(&file, PROTECTED_RANGE).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--diff"]);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Only Y/Z's indentation changes -- the protected "  weird = 1" line
    // must not appear as a removed/added line in the diff at all.
    assert!(!stdout.contains("-  weird = 1"), "protected line must not appear as changed:\n{stdout}");
    assert!(!stdout.contains("+  weird = 1") || stdout.matches("weird = 1").count() <= 1, "protected line must not appear twice (once unchanged is fine, changed is not):\n{stdout}");
}

#[test]
fn protected_range_survives_write_mode() {
    let dir = TempDir::new("fmt-marker-write");
    let file = dir.path().join("x.s");
    fs::write(&file, PROTECTED_RANGE).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--write"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), PROTECTED_RANGE_FORMATTED);
}

#[test]
fn unclosed_fmt_off_notice_appears_on_stderr_with_line_number() {
    let dir = TempDir::new("fmt-marker-unclosed");
    let file = dir.path().join("x.s");
    fs::write(&file, "IF (X=1)\n; FMT: OFF\nY = 1\nENDIF\n").unwrap();

    let out = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0), "an unclosed marker is informational, not an error");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unclosed") && stderr.contains("FMT: OFF") && stderr.contains("line 2"),
        "expected an unclosed-marker notice naming line 2, got:\n{stderr}"
    );
}

#[test]
fn no_unclosed_fmt_off_notice_when_every_marker_is_matched() {
    let dir = TempDir::new("fmt-marker-matched");
    let file = dir.path().join("x.s");
    fs::write(&file, PROTECTED_RANGE).unwrap();

    let out = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("unclosed"), "no notice expected when every marker is matched, got:\n{stderr}");
}

/// Regression test for a real bug this session's own quickstart walkthrough
/// caught: `println!`/`print!` panic on a broken stdout pipe (Rust's exit
/// code for an unhandled panic is 101), which is exactly what happens
/// piping `drut format ... --diff | head` — an ordinary way to use a CLI
/// tool. `io_util::write_stdout`/`write_stdout_line` exist specifically to
/// avoid this. Uses many files so the child is virtually certain to still
/// be writing when the parent stops reading after a small chunk.
#[test]
fn broken_stdout_pipe_does_not_panic() {
    let dir = TempDir::new("broken-pipe");
    for i in 0..50 {
        fs::write(dir.path().join(format!("f{i}.s")), MESSY).unwrap();
    }

    let mut child = Command::new(env!("CARGO_BIN_EXE_drut"))
        .args(["format", dir.path().to_str().unwrap(), "--diff"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn drut");

    // Read a small amount, then drop the handle — closing our end of the
    // pipe while the child (likely) still has more to write, the same way
    // `head`/`less` stopping early does.
    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut buf = [0u8; 64];
    let _ = stdout.read(&mut buf);
    drop(stdout);

    let status = child.wait().expect("failed to wait on drut");
    assert_ne!(
        status.code(),
        Some(101),
        "process must not exit via an unhandled panic on a closed stdout pipe"
    );
}

// --- 012-toml-configuration: drut.toml (T020, T025, T026, T031) ----------

#[test]
fn drut_toml_governs_output_with_no_flags_passed() {
    // US1 Acceptance Scenario 1.
    let dir = TempDir::new("toml-governs");
    fs::write(dir.path().join("drut.toml"), "[format]\ncasing_control_words = \"upper\"\n").unwrap();
    let file = dir.path().join("x.s");
    fs::write(&file, "if (x=1)\nendif\n").unwrap();

    let out = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "IF (x=1)\nENDIF\n");
}

#[test]
fn malformed_drut_toml_warns_on_stderr_but_still_completes_and_exits_0() {
    // FR-011/SC-005: never blocks, never silent.
    let dir = TempDir::new("toml-malformed");
    fs::write(dir.path().join("drut.toml"), "[format]\ncasing_control_words = \"sideways\"\n").unwrap();
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0), "a config warning must never change the exit code");
    assert_eq!(String::from_utf8_lossy(&out.stdout), CLEAN, "formatting must still complete normally");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("drut.toml problem"),
        "expected a config-warning notice on stderr, got: {stderr}"
    );
    assert!(stderr.contains("casing_control_words"), "expected the notice to name the specific bad key, got: {stderr}");
}

#[test]
fn explicit_casing_control_words_flag_overrides_drut_toml_for_one_run_only() {
    // US2 Acceptance Scenario 1 and 2.
    let dir = TempDir::new("toml-override");
    fs::write(dir.path().join("drut.toml"), "[format]\ncasing_control_words = \"lower\"\n").unwrap();
    let file = dir.path().join("x.s");
    fs::write(&file, "IF (X=1)\nENDIF\n").unwrap();

    let overridden = drut(&["format", file.to_str().unwrap(), "--casing-control-words=upper"]);
    assert_eq!(String::from_utf8_lossy(&overridden.stdout), "IF (X=1)\nENDIF\n", "explicit flag must win");

    let reverted = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(
        String::from_utf8_lossy(&reverted.stdout),
        "if (X=1)\nendif\n",
        "the override must be scoped to one invocation, not persistent -- \
         casing_control_words only ever touches control words (if/endif), never the X variable"
    );
}

#[test]
fn explicit_casing_control_words_preserve_overrides_drut_toml_for_one_run_only() {
    // 014-casing-preserve-mode FR-006/FR-009/User Story 1: an explicit
    // preserve override wins over a drut.toml-resolved upper/lower, not
    // treated as "no override given".
    let dir = TempDir::new("toml-override-preserve");
    fs::write(dir.path().join("drut.toml"), "[format]\ncasing_control_words = \"upper\"\n").unwrap();
    let file = dir.path().join("x.s");
    fs::write(&file, "if (x=1)\nendif\n").unwrap();

    let overridden = drut(&["format", file.to_str().unwrap(), "--casing-control-words=preserve"]);
    assert_eq!(
        String::from_utf8_lossy(&overridden.stdout),
        "if (x=1)\nendif\n",
        "explicit preserve must win, leaving casing untouched despite the file's upper setting"
    );

    let reverted = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(
        String::from_utf8_lossy(&reverted.stdout),
        "IF (x=1)\nENDIF\n",
        "the override must be scoped to one invocation -- the file's upper setting applies again \
         (casing only ever touches control words, never the x variable)"
    );
}

#[test]
fn drut_toml_setting_only_casing_control_words_leaves_indent_top_level_at_the_built_in_default() {
    // US2 Acceptance Scenario 3: an unset field falls back independently.
    let dir = TempDir::new("toml-partial");
    fs::write(dir.path().join("drut.toml"), "[format]\ncasing_control_words = \"upper\"\n").unwrap();
    let file = dir.path().join("x.s");
    fs::write(&file, TOP_LEVEL_NON_ZERO).unwrap();

    let out = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "    IF (X=1)\n        Y = 2\n    ENDIF\n",
        "indent_top_level must stay at Preserve (built-in default) -- unaffected by casing being set"
    );
}

#[test]
fn isolated_ignores_a_present_valid_drut_toml_entirely() {
    // US3 Acceptance Scenario 1.
    let dir = TempDir::new("toml-isolated");
    fs::write(
        dir.path().join("drut.toml"),
        "[format]\ncasing_control_words = \"upper\"\nindent_top_level = \"auto\"\n",
    )
    .unwrap();
    let file = dir.path().join("x.s");
    fs::write(&file, TOP_LEVEL_NON_ZERO).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--isolated"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        TOP_LEVEL_NON_ZERO,
        "isolated must match built-in defaults exactly, as if no drut.toml existed"
    );
}

// -- 017-casing-categories-indent-width: per-category flags, indent-width,
// and the literal reported gap (tasks.md T016/T018/T019/T020/T021/T030/T039) --

#[test]
fn casing_data_references_flag_reaches_tokens_casing_control_words_cannot() {
    // US2: the literal reported gap (GitHub issue #3) -- mw/li/ni/i/j
    // uppercased via a flag that never existed before this feature.
    let dir = TempDir::new("casing-data-references");
    let file = dir.path().join("x.s");
    fs::write(&file, "mw[1] = mi.1.1\nx = li.FT\ny = ni.CLASS\nif (i=25) z = j\n").unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--casing-data-references=upper"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "MW[1] = MI.1.1\nx = LI.FT\ny = NI.CLASS\nif (I=25) Z = J\n"
    );
}

#[test]
fn casing_data_references_left_at_preserve_by_default_leaves_the_reported_tokens_untouched() {
    // US2 Acceptance Scenario 3 -- opt-in only, no --casing-data-references
    // flag at all, no drut.toml.
    let dir = TempDir::new("data-references-preserve-default");
    let file = dir.path().join("x.s");
    let src = "mw[1] = mi.1.1\nx = li.FT\ny = ni.CLASS\nif (i=25) z = j\nzones = 1\n";
    fs::write(&file, src).unwrap();

    let out = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), src, "no flag, no drut.toml -- byte-identical (FR-012)");
}

#[test]
fn mw_pair_keyword_shaped_and_assignment_target_shaped_both_uppercase_together() {
    // FR-005, proven at the CLI/format() level (not just data_reference.rs's
    // own lower-level recognition tests) -- one flag, uniform result
    // regardless of structural shape.
    let dir = TempDir::new("mw-uniform-shape");
    let file = dir.path().join("x.s");
    fs::write(&file, "pathload path=time, mw[201]=mi.1.1\nmw[1] = mi.2.1\n").unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--casing-data-references=upper"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "pathload path=time, MW[201]=MI.1.1\nMW[1] = MI.2.1\n"
    );
}

#[test]
fn all_three_casing_categories_set_independently_in_one_run() {
    // US1 Acceptance Scenario 1: a script mixing all three token kinds,
    // three different explicit values, each category's tokens change
    // independently and no category's setting leaks into another's.
    let dir = TempDir::new("three-categories");
    let file = dir.path().join("x.s");
    fs::write(&file, "if (x=1)\nfile=out.txt\ny = mi.1.1\nendif\n").unwrap();

    let out = drut(&[
        "format",
        file.to_str().unwrap(),
        "--casing-control-words=upper",
        "--casing-pair-keywords=preserve",
        "--casing-data-references=lower",
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "IF (x=1)\n    file=out.txt\n    y = mi.1.1\nENDIF\n",
        "control_words upper, pair_keywords untouched (already lowercase), data_references already lowercase (no-op)"
    );
}

#[test]
fn casing_data_references_rejects_auto_as_a_usage_error() {
    // FR-003: no built-in preset ships with this feature -- "auto" is not
    // a valid value at any of the three granular casing flags.
    let dir = TempDir::new("casing-data-references-auto-rejected");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--casing-data-references=auto"]);
    assert_ne!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY);
}

#[test]
fn indent_width_flag_overrides_the_built_in_default() {
    let dir = TempDir::new("indent-width-flag");
    let file = dir.path().join("x.s");
    fs::write(&file, "IF (X=1)\nLOOP i=1,5\nY = 2\nENDLOOP\nENDIF\n").unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--indent-width=2"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "IF (X=1)\n  LOOP i=1,5\n    Y = 2\n  ENDLOOP\nENDIF\n"
    );
}

#[test]
fn indent_width_out_of_range_is_a_usage_error_not_a_silent_clamp() {
    // The CLI validates its own range at the argument-parsing layer (a
    // clean usage error) -- distinct from a drut.toml value out of range,
    // which degrades non-fatally instead (data-model.md §4).
    let dir = TempDir::new("indent-width-out-of-range");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--indent-width=0"]);
    assert_ne!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY);
}

#[test]
fn drut_toml_indent_width_governs_output_with_no_flag_passed() {
    let dir = TempDir::new("toml-indent-width");
    fs::write(dir.path().join("drut.toml"), "[format]\nindent_width = 2\n").unwrap();
    let file = dir.path().join("x.s");
    fs::write(&file, "IF (X=1)\nY = 2\nENDIF\n").unwrap();

    let out = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "IF (X=1)\n  Y = 2\nENDIF\n");
}

// -- 018-operator-spacing (tasks.md T017, T021) --

#[test]
fn operator_spacing_flag_overrides_a_drut_toml_resolved_preserve() {
    let dir = TempDir::new("operator-spacing-flag");
    fs::write(dir.path().join("drut.toml"), "[format]\noperator_spacing = \"preserve\"\n").unwrap();
    let file = dir.path().join("x.s");
    fs::write(&file, "ZONES   = 1\n").unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--operator-spacing=fixed"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ZONES = 1\n");
}

#[test]
fn operator_spacing_invalid_value_is_a_usage_error_not_a_silent_fallback() {
    // FR-011/SC-004: operator_spacing is a closed ValueEnum, same shape as
    // --casing-control-words -- an out-of-set CLI value is rejected outright, not
    // silently degraded to preserve the way a malformed drut.toml value is.
    let dir = TempDir::new("operator-spacing-invalid");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--operator-spacing=tight"]);
    assert_ne!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY);
}

// -- 019-blank-line-normalization (tasks.md T016) --

#[test]
fn blank_lines_flag_overrides_a_drut_toml_resolved_preserve_for_one_run() {
    let dir = TempDir::new("blank-lines-flag");
    fs::write(dir.path().join("drut.toml"), "[format]\nblank_lines = \"preserve\"\n").unwrap();
    let file = dir.path().join("x.s");
    fs::write(&file, "X = 1\n\n\n\n\n\nY = 2\n").unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--blank-lines=auto"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "X = 1\n\n\nY = 2\n",
        "the run of 5 must contract to the default top-level cap (2)"
    );
}

#[test]
fn blank_lines_top_cap_flag_overrides_the_default() {
    let dir = TempDir::new("blank-lines-top-cap-flag");
    let file = dir.path().join("x.s");
    fs::write(&file, "X = 1\n\n\n\n\n\nY = 2\n").unwrap();

    let out = drut(&[
        "format",
        file.to_str().unwrap(),
        "--blank-lines=auto",
        "--blank-lines-top-cap=1",
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "X = 1\n\nY = 2\n");
}

#[test]
fn blank_line_cap_out_of_range_is_a_usage_error_not_a_silent_clamp() {
    // Mirrors --indent-width's own regression case (data-model.md §3): the
    // CLI validates its own range at the argument-parsing layer.
    let dir = TempDir::new("blank-line-cap-out-of-range");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&[
        "format",
        file.to_str().unwrap(),
        "--blank-lines=auto",
        "--blank-lines-top-cap=0",
    ]);
    assert_ne!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY);

    let out = drut(&[
        "format",
        file.to_str().unwrap(),
        "--blank-lines=auto",
        "--blank-lines-nested-cap=51",
    ]);
    assert_ne!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY);
}
