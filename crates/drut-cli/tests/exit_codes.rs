//! FR-011 (check) / FR-020 (format) exit-code coverage, run against the
//! actual built `drut` binary.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("drut-cli-exitcode-{label}-{nanos}"));
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

#[test]
fn check_clean_directory_exits_0() {
    let out = drut(&["check", "../voyager-core/tests/fixtures/valid"]);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn check_broken_directory_exits_1() {
    let out = drut(&["check", "../voyager-core/tests/fixtures/broken"]);
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn check_nonexistent_path_exits_2() {
    let missing = std::env::temp_dir().join("drut-cli-exitcode-does-not-exist-xyz");
    let out = drut(&["check", missing.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
#[cfg(windows)]
fn check_fatal_takes_precedence_over_diagnostics_found() {
    use std::os::windows::fs::OpenOptionsExt;

    let dir = TempDir::new("precedence");
    // A file with a real diagnostic (ProblemsFound-worthy on its own).
    fs::write(dir.path().join("broken.s"), "IF (1=1)\n").unwrap();
    // A second, unreadable file (Fatal-worthy on its own).
    let locked_path = dir.path().join("locked.s");
    fs::write(&locked_path, "content").unwrap();
    let _locked = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&locked_path)
        .unwrap();

    let out = drut(&["check", dir.path().to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "Fatal must win when both a diagnostic and a read failure occur in the same run"
    );
}

// -- format (FR-020) ----------------------------------------------------

const MESSY: &str = "IF (X=1)\nY = 2\nENDIF\n";

#[test]
fn format_default_mode_clean_already_formatted_file_exits_0() {
    let dir = TempDir::new("fmt-clean");
    let file = dir.path().join("x.s");
    fs::write(&file, "IF (X=1)\n    Y = 2\nENDIF\n").unwrap();
    let out = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn format_write_mode_applying_a_real_change_exits_0() {
    // --write's exit 0 covers both "nothing needed a change" and
    // "every needed write succeeded" (FR-020(a)) — not just the no-op case.
    let dir = TempDir::new("fmt-write-clean");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();
    let out = drut(&["format", file.to_str().unwrap(), "--write"]);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn format_check_mode_finding_a_change_exits_1() {
    let dir = TempDir::new("fmt-check-dirty");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();
    let out = drut(&["format", file.to_str().unwrap(), "--check"]);
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn format_default_and_diff_modes_finding_a_change_are_not_themselves_a_failure() {
    // FR-020(b) is specific to --check; default/--diff finding changes is
    // not itself a failure (cli-contract.md's exit-code table footnote).
    let dir = TempDir::new("fmt-default-dirty");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let default_out = drut(&["format", file.to_str().unwrap()]);
    assert_eq!(default_out.status.code(), Some(0));

    let diff_out = drut(&["format", file.to_str().unwrap(), "--diff"]);
    assert_eq!(diff_out.status.code(), Some(0));
}

#[test]
fn format_nonexistent_path_exits_2() {
    let missing = std::env::temp_dir().join("drut-cli-exitcode-format-does-not-exist-xyz");
    let out = drut(&["format", missing.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn format_lossy_file_exits_2_in_every_mode_not_only_write() {
    // FR-025: a Lossy-encoding file makes the whole run Fatal regardless of
    // which mode encountered it — even default/--check/--diff, none of
    // which ever attempt a write themselves.
    let lossy_fixture = Path::new("../voyager-core/tests/fixtures/encoding_fallback/lossy.s");
    assert!(lossy_fixture.exists(), "expected the T025 lossy fixture to exist");

    for args in [
        vec!["format", lossy_fixture.to_str().unwrap()],
        vec!["format", lossy_fixture.to_str().unwrap(), "--check"],
        vec!["format", lossy_fixture.to_str().unwrap(), "--diff"],
    ] {
        let out = drut(&args);
        assert_eq!(out.status.code(), Some(2), "args={args:?}");
    }
}

#[test]
fn format_lossy_file_write_mode_refuses_and_leaves_file_untouched() {
    let dir = TempDir::new("fmt-lossy-write");
    let file = dir.path().join("lossy.s");
    fs::copy("../voyager-core/tests/fixtures/encoding_fallback/lossy.s", &file).unwrap();
    let before = fs::read(&file).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--write"]);
    assert_eq!(out.status.code(), Some(2));
    let after = fs::read(&file).unwrap();
    assert_eq!(before, after, "a refused write must leave the file byte-identical");
}
