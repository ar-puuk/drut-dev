//! Full-corpus diagnostic parity through the LSP protocol layer (SC-002,
//! first slice of FR-028's Definition of Done) — mirrors
//! `drut-cli/tests/fixture_corpus_e2e.rs`'s established pattern, but driving
//! real `textDocument/didOpen` JSON-RPC via `Connection::memory()` instead
//! of spawning the built binary.
//!
//! Two parts:
//! 1. The committed `voyager-core/tests/fixtures/broken/` set (no external
//!    dependency) — every deliberately-broken fixture publishes its
//!    expected diagnostic, **excluding** `undecodable_byte.s`
//!    (`InvalidEncoding`), which cannot and is not expected to reproduce
//!    through `didOpen` (FR-005/FR-028 carve-out, research.md §12) — opening
//!    it should publish zero diagnostics for the six reachable categories.
//! 2. The external WF-TDM-Official-Releases corpus (SC-002's other half) —
//!    gated behind `DRUT_CORPUS_PATH` and `#[ignore]`'d unconditionally, the
//!    same three-state gating `drut-cli`'s own e2e test already establishes.

mod common;

use std::path::{Path, PathBuf};

use common::*;
use serde_json::json;

fn broken_fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../voyager-core/tests/fixtures/broken")
}

fn expected_kinds(text: &str) -> Vec<String> {
    let first_line = text.lines().next().unwrap_or("");
    let marker = "; EXPECT:";
    if let Some(rest) = first_line.strip_prefix(marker) {
        rest.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        Vec::new()
    }
}

#[test]
fn broken_fixtures_publish_their_expected_diagnostic_via_did_open() {
    let dir = broken_fixtures_dir();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {dir:?}: {e}"))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "s" || ext == "block"))
        .collect();
    entries.sort();
    assert!(!entries.is_empty(), "expected at least one broken fixture under {dir:?}");

    let (client, _handle) = spawn_server();
    initialize(&client);

    for path in entries {
        let raw = std::fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
        let uri = format!("file:///{}", path.file_name().unwrap().to_string_lossy());

        if path.file_name().unwrap() == "undecodable_byte.s" {
            // FR-005/FR-028 carve-out (research.md §12): this fixture's raw
            // bytes are not valid UTF-8 by design, so — exactly like a real
            // editor — the only way its content can ever reach `didOpen` at
            // all is already-lossily-decoded (String::from_utf8_lossy,
            // substituting U+FFFD), the same non-fatal decoding VS Code
            // itself performs before ever handing text to a language
            // server. That decoded text has no other structural defect, so
            // zero diagnostics is the correct outcome — `InvalidEncoding`
            // itself is never reachable this way, by construction.
            let lossy = String::from_utf8_lossy(&raw).into_owned();
            let note = did_open(&client, &uri, &lossy);
            let diagnostics = note.params["diagnostics"].as_array().unwrap();
            assert!(
                diagnostics.is_empty(),
                "undecodable_byte.s (lossily decoded, as a real editor would) should publish \
                 zero diagnostics through didOpen, got: {diagnostics:?}"
            );
            continue;
        }

        let text = String::from_utf8(raw).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
        let note = did_open(&client, &uri, &text);
        let diagnostics = note.params["diagnostics"].as_array().unwrap();
        let expected = expected_kinds(&text);
        assert!(
            !expected.is_empty(),
            "fixture {path:?} is missing a valid '; EXPECT: Kind' marker on its first line"
        );
        for kind in &expected {
            assert!(
                diagnostics.iter().any(|d| d["code"] == json!(kind)),
                "expected a {kind} diagnostic for {path:?}, got: {diagnostics:#?}"
            );
        }
    }

    shutdown(&client);
}

fn corpus_path() -> PathBuf {
    match std::env::var("DRUT_CORPUS_PATH") {
        Ok(value) if !value.trim().is_empty() => PathBuf::from(value),
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

/// SC-002 reproduced through the LSP protocol layer: every valid corpus
/// file, opened via `didOpen`, publishes zero diagnostics — the same
/// 161/161-clean result already proven at the `voyager-core` (library) and
/// `drut-cli` (CLI) layers, now proven through the server too.
#[test]
#[ignore = "requires DRUT_CORPUS_PATH pointing at a local WF-TDM-Official-Releases checkout"]
fn full_corpus_did_open_is_clean_through_the_lsp_protocol() {
    let corpus = corpus_path();
    let mut files = Vec::new();
    collect_script_files(&corpus, &mut files);
    assert!(!files.is_empty(), "expected at least one .s/.block file under {corpus:?}");

    let (client, _handle) = spawn_server();
    initialize(&client);

    let mut failures = Vec::new();
    for (i, path) in files.iter().enumerate() {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => continue, // non-UTF-8 on disk: didOpen can't carry it anyway (research.md §12).
        };
        let uri = format!("file:///corpus-{i}.s");
        let note = did_open(&client, &uri, &text);
        let diagnostics = note.params["diagnostics"].as_array().unwrap();
        if !diagnostics.is_empty() {
            failures.push(format!("{path:?}: {diagnostics:?}"));
        }
    }

    shutdown(&client);
    assert!(
        failures.is_empty(),
        "expected zero diagnostics across the full corpus, got failures in {} file(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
