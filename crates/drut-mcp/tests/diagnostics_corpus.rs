//! Full-corpus diagnostic parity for the `diagnose` tool (SC-006) — mirrors
//! `drut-lsp/tests/diagnostics_corpus.rs`'s own established pattern (itself
//! mirroring `drut-cli/tests/fixture_corpus_e2e.rs`): the real corpus is
//! already proven 100% clean at the `voyager-core`/CLI/LSP layers, so
//! "`diagnose` reports zero diagnostics for every file" *is* parity with
//! `drut check`'s own output (also zero) — the same simplification every
//! prior phase's corpus test already relies on, not a new one invented here.

use std::path::{Path, PathBuf};

use drut_mcp::diagnose::{diagnose, DiagnosticsInput};
use drut_mcp::source::ScriptSource;

fn corpus_path() -> PathBuf {
    match std::env::var("DRUT_CORPUS_PATH") {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => panic!(
            "set DRUT_CORPUS_PATH to a local WF-TDM-Official-Releases checkout to run this test \
             (e.g. $env:DRUT_CORPUS_PATH = \"D:\\GitHub\\WF-TDM-Official-Releases\")"
        ),
    }
}

fn collect_script_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_script_files(&path, out);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("s") || ext.eq_ignore_ascii_case("block"))
        {
            out.push(path);
        }
    }
}

#[test]
#[ignore = "requires DRUT_CORPUS_PATH pointing at a local WF-TDM-Official-Releases checkout"]
fn full_corpus_diagnose_is_clean() {
    let corpus = corpus_path();
    let mut files = Vec::new();
    collect_script_files(&corpus, &mut files);
    assert!(!files.is_empty(), "expected at least one .s/.block file under {corpus:?}");

    let mut failures = Vec::new();
    for path in &files {
        let input = DiagnosticsInput {
            source: ScriptSource {
                text: None,
                path: Some(path.to_string_lossy().to_string()),
            },
        };
        match diagnose(&input) {
            Ok(diagnostics) if diagnostics.is_empty() => {}
            Ok(diagnostics) => failures.push(format!("{path:?}: {diagnostics:?}")),
            Err(err) => failures.push(format!("{path:?}: tool error: {err}")),
        }
    }

    assert!(
        failures.is_empty(),
        "expected zero diagnostics across the full corpus via the diagnose tool, got failures in {} file(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
