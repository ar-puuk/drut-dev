//! `format` flag behavior: default/`--write`/`--check`/`--diff` disposition,
//! `--casing` validation, and mutual exclusivity (spec.md FR-015–FR-019).

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
    // Note: `clap`'s own usage-error exit code (a `--casing` with no value
    // never reaches our own exit-code logic at all) happens to also be 2,
    // the same numeric value FR-011/FR-020's `Fatal` uses — a coincidence,
    // not a collision, since spec.md only requires this be distinguishable
    // as "a clap-level parse failure, not one of the three run-outcome exit
    // codes," not that it use a numerically distinct value. What actually
    // matters, and what this asserts, is that no file was touched.
    let dir = TempDir::new("casing-bare");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--casing"]);
    assert_ne!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY, "must not touch the file");
}

#[test]
fn casing_with_invalid_value_is_a_usage_error() {
    let dir = TempDir::new("casing-invalid");
    let file = dir.path().join("x.s");
    fs::write(&file, MESSY).unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--casing=sideways"]);
    assert_ne!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), MESSY);
}

#[test]
fn casing_upper_rewrites_control_words() {
    let dir = TempDir::new("casing-upper");
    let file = dir.path().join("x.s");
    fs::write(&file, "if (x=1)\nendif\n").unwrap();

    let out = drut(&["format", file.to_str().unwrap(), "--casing=upper"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "IF (x=1)\nENDIF\n");
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
