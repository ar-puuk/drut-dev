//! Full external-corpus end-to-end checks (T017; T033 for `format` is added
//! in US2). Gated behind `DRUT_CORPUS_PATH` and `#[ignore]`'d unconditionally
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

use std::path::PathBuf;
use std::process::Command;

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
