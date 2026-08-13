//! Per-file upward directory walk-up for the nearest `drut.toml`
//! (012-toml-configuration/research.md §7).

use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = "drut.toml";

/// Walk upward from `start` (a file or directory) for the nearest
/// `drut.toml`. Stops at the first file found, a `.git` boundary (a file or
/// a directory — presence only, no worktree-redirect parsing needed), or
/// the filesystem root. Never panics, including when `start` doesn't exist.
pub fn discover(start: &Path) -> Option<PathBuf> {
    let mut dir = if start.is_dir() {
        Some(start.to_path_buf())
    } else {
        start.parent().map(Path::to_path_buf)
    };

    while let Some(current) = dir {
        let candidate = current.join(CONFIG_FILE_NAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        if current.join(".git").exists() {
            return None;
        }
        dir = current.parent().map(Path::to_path_buf);
    }

    None
}
