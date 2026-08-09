//! Full external-corpus end-to-end checks (T017 for `check`, T033 for
//! `format`). Gated behind `DRUT_CORPUS_PATH` and `#[ignore]`'d unconditionally
//! — the WF-TDM-Official-Releases corpus (161 real `.s`/`.block` files) is
//! external and not committed to this repo (licensing still an open item,
//! `001-voyager-script-parser/research.md` §3), so this can't be a normal
//! `cargo test` assertion that every checkout is expected to satisfy.
//!
//! Three distinguishable states, not two — see
//! specs/002-cli-check-format/tasks.md's "Human-in-the-loop dependency" note:
//! 1. Plain `cargo test`: this test doesn't run at all; `cargo test` reports
//!    it under a separate "ignored" count, never mixed into "passed".
//! 2. `cargo test -- --ignored` with `DRUT_CORPUS_PATH` unset: the test runs
//!    (since `--ignored` was explicitly requested) and immediately panics
//!    with a clear message — reported FAILED, not silently skipped or passed.
//! 3. `cargo test -- --ignored` with `DRUT_CORPUS_PATH` set: it actually
//!    walks the corpus and asserts against it.
//!
//! Run locally with:
//! `$env:DRUT_CORPUS_PATH = "D:\GitHub\WF-TDM-Official-Releases"`
//! `cargo test -p drut-cli --test fixture_corpus_e2e -- --ignored`

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn corpus_path() -> PathBuf {
    match std::env::var("DRUT_CORPUS_PATH") {
        Ok(value) if !value.trim().is_empty() => PathBuf::from(value),
        _ => panic!(
            "set DRUT_CORPUS_PATH to a local WF-TDM-Official-Releases checkout to run this test \
             (e.g. $env:DRUT_CORPUS_PATH = \"D:\\GitHub\\WF-TDM-Official-Releases\")"
        ),
    }
}

/// SC-001 reproduced end-to-end through the CLI: the full 161-file corpus
/// parses with zero diagnostics and exit code 0 — the same result
/// `voyager-core`'s own library-level test suite already proves, now proven
/// through traversal + byte-reading + `parse_bytes` + exit-code wiring too.
#[test]
#[ignore = "requires DRUT_CORPUS_PATH pointing at a local WF-TDM-Official-Releases checkout"]
fn full_corpus_check_is_clean_through_the_cli() {
    let corpus = corpus_path();
    let out = Command::new(env!("CARGO_BIN_EXE_drut"))
        .arg("check")
        .arg(&corpus)
        .output()
        .expect("failed to run drut");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.trim().is_empty(),
        "expected zero diagnostics across the full corpus, got:\n{stdout}"
    );
    assert_eq!(out.status.code(), Some(0));
}

// T018's broken-fixture assertion (SC-002, distinguishable from a read
// failure) is already covered by tests/exit_codes.rs's
// `check_broken_directory_exits_1` against the committed
// `voyager-core/tests/fixtures/broken/` set — the external 161-file corpus
// itself is documented as containing zero broken files (001's own full-corpus
// validation), so there's no broken-fixture case to add *here* without
// hand-injecting a defect into a real production file, which the fixture
// corpus's own licensing/redaction discipline argues against. No duplicate
// test added.

fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to);
        } else {
            fs::copy(&from, &to).unwrap_or_else(|e| panic!("copy {} -> {}: {e}", from.display(), to.display()));
        }
    }
}

/// SC-004/SC-005 reproduced end-to-end through the CLI: `format --write`
/// applied to the full corpus, twice, with the second pass a genuine no-op
/// (CLI-level idempotency — `format_bytes`'s own idempotency guarantee is
/// already exhaustively unit- and golden-file-tested at the `voyager-core`
/// layer, per research.md §7; this test is only proving the CLI's
/// traversal → format_bytes → write → re-traversal wiring, not re-deriving
/// that guarantee), followed by a `check` pass confirming the now-formatted
/// corpus is still 100% clean (SC-001, reproduced again post-format).
///
/// Operates on a **temporary copy** of the corpus, never the real
/// `DRUT_CORPUS_PATH` checkout — `--write` is destructive, and mutating an
/// external, unrelated repository as a side effect of running this test
/// suite would be exactly the kind of surprising, hard-to-reverse action
/// that has no place in an automated test, opt-in or not.
#[test]
#[ignore = "requires DRUT_CORPUS_PATH pointing at a local WF-TDM-Official-Releases checkout"]
fn full_corpus_format_write_is_idempotent_and_stays_clean_through_the_cli() {
    let source_corpus = corpus_path();

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let scratch = std::env::temp_dir().join(format!("drut-cli-e2e-format-scratch-{nanos}"));
    copy_dir_recursive(&source_corpus, &scratch);

    let write = |dir: &Path| {
        Command::new(env!("CARGO_BIN_EXE_drut"))
            .arg("format")
            .arg(dir)
            .arg("--write")
            .output()
            .expect("failed to run drut")
    };

    let first = write(&scratch);
    assert_eq!(first.status.code(), Some(0), "first --write pass should succeed cleanly");

    let second = write(&scratch);
    assert_eq!(second.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&second.stdout).trim().is_empty(),
        "second --write pass must report nothing further to format (CLI-level idempotency)"
    );

    let recheck = Command::new(env!("CARGO_BIN_EXE_drut"))
        .arg("check")
        .arg(&scratch)
        .output()
        .expect("failed to run drut");
    assert_eq!(
        recheck.status.code(),
        Some(0),
        "the formatted corpus must still parse 100% clean (SC-001, post-format)"
    );

    let _ = fs::remove_dir_all(&scratch);
}
