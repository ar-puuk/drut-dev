//! Shared file/directory traversal: `.gitignore`-aware walking plus `.s`/
//! `.block` extension filtering, used by both `check` and `format`
//! (spec.md FR-001–FR-005; data-model.md §3).

use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

/// One file selected for processing — a path plus its raw bytes, read once
/// and reused for `parse_bytes`/`format_bytes` (data-model.md §3).
#[derive(Debug, Clone)]
pub struct MatchedFile {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

/// A file that matched the `.s`/`.block` extension filter but couldn't be
/// read (FR-005).
#[derive(Debug, Clone)]
pub struct ReadFailure {
    pub path: PathBuf,
    pub message: String,
}

/// The result of resolving a target path into concrete files (FR-001–FR-005).
#[derive(Debug, Default)]
pub struct TraversalOutcome {
    pub matched_files: Vec<MatchedFile>,
    pub read_failures: Vec<ReadFailure>,
    /// Set instead of the above two when `path` itself doesn't exist or is
    /// neither a file nor a directory (FR-004).
    pub invalid_target: Option<String>,
}

/// `.s`/`.block`, case-insensitive on the extension (FR-003). Every other
/// extension — including known binary Cube types (`.mat`/`.net`/`.dbd`/
/// `.prj`) and anything else — is never opened, read, or reported on.
fn has_script_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("s") || ext.eq_ignore_ascii_case("block"))
}

/// Walks `target` (file or directory), honoring `.gitignore` the same way
/// `git` would (FR-002, via the `ignore` crate's default `WalkBuilder`
/// behavior — nested `.gitignore` files, global excludes, etc.), filters to
/// `.s`/`.block` files (FR-003), and reads each matched file's raw bytes.
pub fn traverse(target: &Path) -> TraversalOutcome {
    let mut outcome = TraversalOutcome::default();

    if !target.exists() {
        outcome.invalid_target = Some(format!("path does not exist: {}", target.display()));
        return outcome;
    }

    if target.is_file() {
        // FR-003 skips a non-matching file silently; that includes the
        // explicit single-file target itself, which simply yields zero
        // matched files rather than an error.
        if has_script_extension(target) {
            read_one(target, &mut outcome);
        }
        return outcome;
    }

    if !target.is_dir() {
        outcome.invalid_target = Some(format!(
            "path is neither a file nor a directory: {}",
            target.display()
        ));
        return outcome;
    }

    for entry in WalkBuilder::new(target).build() {
        let Ok(entry) = entry else {
            // A walk-level error (e.g. a broken symlink) is neither a
            // matched file nor a read failure — FR-005 only covers a file
            // that already matched the extension filter.
            continue;
        };
        let is_file = entry.file_type().is_some_and(|ft| ft.is_file());
        if is_file && has_script_extension(entry.path()) {
            read_one(entry.path(), &mut outcome);
        }
    }

    outcome
}

fn read_one(path: &Path, outcome: &mut TraversalOutcome) {
    match fs::read(path) {
        Ok(bytes) => outcome.matched_files.push(MatchedFile {
            path: path.to_path_buf(),
            bytes,
        }),
        Err(err) => outcome.read_failures.push(ReadFailure {
            path: path.to_path_buf(),
            message: err.to_string(),
        }),
    }
}
