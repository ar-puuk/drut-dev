// Embeds a build/version identifier into the compiled binary so a running
// drut-lsp process can report exactly which build it is, directly from
// inside its own startup log line (src/lib.rs's `run()`) -- not inferred
// from PATH resolution in a separate shell, but reported by the process
// itself. Added 2026-08-11 to make "which drut-lsp is VS Code actually
// running" answerable definitively from inside the editor's own Output
// panel, after a live debugging session where PATH-resolution divergence
// between environments was the leading (unconfirmed) suspect.
//
// Runs from D:\GitHub\drut-dev (or wherever the workspace is checked out)
// -- see docs/known-environment-quirks.md's Application Control entry
// before assuming a build-script failure here is a real bug; its own
// findings confirm build scripts succeed from the trusted repo path.

use std::process::Command;

fn main() {
    // Re-run if the git HEAD moves, so the embedded commit never goes
    // stale silently across commits within the same otherwise-cached build.
    println!("cargo:rerun-if-changed=../../.git/HEAD");

    let git_commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=DRUT_GIT_COMMIT={git_commit}");

    // Unix epoch seconds, not a formatted calendar date -- avoids hand-
    // rolling UTC calendar math or adding a chrono-style dependency purely
    // for a build-identification log line; precise and directly comparable
    // for "which build is newer" without needing to be human-pretty.
    let build_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=DRUT_BUILD_TIMESTAMP={build_timestamp}");
}
