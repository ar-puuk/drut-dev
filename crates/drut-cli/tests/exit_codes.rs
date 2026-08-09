//! FR-011 (check) / FR-020 (format) exit-code coverage, run against the
//! actual built `drut` binary. The `format` portion is added in T032 (US2).

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
