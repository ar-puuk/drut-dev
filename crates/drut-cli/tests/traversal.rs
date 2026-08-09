//! FR-001–FR-005 coverage: directory recursion, `.gitignore` respecting
//! (including nested `.gitignore`), extension filtering, invalid paths, and
//! unreadable files.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use drut_cli::traverse::traverse;

/// A fresh, uniquely-named scratch directory under the OS temp dir, cleaned
/// up on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("drut-cli-test-{label}-{nanos}"));
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

fn matched_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = traverse(dir)
        .matched_files
        .into_iter()
        .map(|f| f.path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn directory_recursion_finds_nested_files() {
    let dir = TempDir::new("recursion");
    fs::write(dir.path().join("top.s"), "RUN PGM=MATRIX\nENDRUN\n").unwrap();
    let nested = dir.path().join("sub").join("deeper");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("deep.block"), "PROCESS PHASE=1\nENDPROCESS\n").unwrap();

    let names = matched_names(dir.path());
    assert_eq!(names, vec!["deep.block".to_string(), "top.s".to_string()]);
}

#[test]
fn gitignore_excludes_matching_paths() {
    let dir = TempDir::new("gitignore");
    // `.gitignore` only applies inside an actual repo — the `ignore` crate's
    // `require_git` default (true) mirrors real `git`'s own behavior, which
    // FR-002 explicitly asks this tool to match ("the same way git itself
    // would decide"). A bare `.git` directory is enough; no real repo needed.
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join(".gitignore"), "ignored_dir/\n").unwrap();
    fs::write(dir.path().join("kept.s"), "kept\n").unwrap();
    let ignored_dir = dir.path().join("ignored_dir");
    fs::create_dir_all(&ignored_dir).unwrap();
    fs::write(ignored_dir.join("excluded.s"), "excluded\n").unwrap();

    // Nested .gitignore excludes one specific file within a kept directory.
    let kept_dir = dir.path().join("kept_dir");
    fs::create_dir_all(&kept_dir).unwrap();
    fs::write(kept_dir.join(".gitignore"), "excluded_nested.s\n").unwrap();
    fs::write(kept_dir.join("excluded_nested.s"), "excluded\n").unwrap();
    fs::write(kept_dir.join("kept_nested.s"), "kept\n").unwrap();

    let names = matched_names(dir.path());
    assert_eq!(
        names,
        vec!["kept.s".to_string(), "kept_nested.s".to_string()]
    );
}

#[test]
fn extension_filtering_is_case_insensitive_and_scoped_to_s_and_block() {
    let dir = TempDir::new("extfilter");
    fs::write(dir.path().join("lower.s"), "x").unwrap();
    fs::write(dir.path().join("upper.S"), "x").unwrap();
    fs::write(dir.path().join("mixed.BlOcK"), "x").unwrap();
    for skipped in ["binary.mat", "net.net", "db.dbd", "proj.prj", "notes.txt"] {
        fs::write(dir.path().join(skipped), "x").unwrap();
    }

    let names = matched_names(dir.path());
    assert_eq!(
        names,
        vec!["lower.s".to_string(), "mixed.BlOcK".to_string(), "upper.S".to_string()]
    );
}

#[test]
fn nonexistent_path_sets_invalid_target_not_an_error_file() {
    let missing = std::env::temp_dir().join("drut-cli-test-does-not-exist-xyz");
    let outcome = traverse(&missing);
    assert!(outcome.invalid_target.is_some());
    assert!(outcome.matched_files.is_empty());
    assert!(outcome.read_failures.is_empty());
}

#[test]
fn empty_directory_is_zero_matched_files_not_an_error() {
    let dir = TempDir::new("empty");
    let outcome = traverse(dir.path());
    assert!(outcome.invalid_target.is_none());
    assert!(outcome.matched_files.is_empty());
    assert!(outcome.read_failures.is_empty());
}

#[cfg(windows)]
#[test]
fn unreadable_file_becomes_a_read_failure_not_a_panic() {
    use std::os::windows::fs::OpenOptionsExt;

    let dir = TempDir::new("unreadable");
    let path = dir.path().join("locked.s");
    fs::write(&path, "content").unwrap();

    // Open with FILE_SHARE_NONE (share_mode(0)) so a concurrent read from
    // traverse() fails with a sharing-violation error, simulating FR-005's
    // "matched a filter but couldn't be read" case without needing ACL edits.
    let _locked = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&path)
        .unwrap();

    let outcome = traverse(dir.path());
    assert!(outcome.matched_files.is_empty());
    assert_eq!(outcome.read_failures.len(), 1);
    assert_eq!(outcome.read_failures[0].path, path);
}

#[cfg(unix)]
#[test]
fn unreadable_file_becomes_a_read_failure_not_a_panic() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new("unreadable");
    let path = dir.path().join("locked.s");
    fs::write(&path, "content").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

    let outcome = traverse(dir.path());
    // Restore permissions so TempDir's Drop can remove the file.
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

    assert!(outcome.matched_files.is_empty());
    assert_eq!(outcome.read_failures.len(), 1);
    assert_eq!(outcome.read_failures[0].path, path);
}
