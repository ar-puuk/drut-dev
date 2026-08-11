//! Cross-adapter parity: `drut-mcp`'s `query_structure` tool vs. `drut-lsp`'s
//! hover, for the same real corpus positions (spec.md SC-003,
//! `004-mcp-server/quickstart.md` step 6, folded into that feature's T021
//! extraction-verification report).
//!
//! **Deliberately lives here, in `drut-cli`, not in `crates/drut-mcp/tests/`**
//! (a placement correction made during implementation, not the original
//! plan): `drut-mcp` structurally cannot depend on `drut-lsp` (FR-011,
//! verified by that feature's own T004) — a test comparing the two head to
//! head therefore cannot live inside `drut-mcp`'s own test suite. `drut-cli`
//! already depends on both crates as a regular dependency, so it's the one
//! place in the workspace that can drive both sides of this comparison
//! without violating that same isolation. Both now call the identical
//! `voyager_core::block_at` (`004-mcp-server/research.md` §5) — this test
//! is parity on the *wiring*, not a re-verification of the derivation
//! itself (already proven correct in `voyager-core/tests/block_resolution.rs`
//! and `drut-lsp`'s own pre-existing hover tests, both passing unmodified
//! after the extraction).

use std::path::{Path, PathBuf};
use std::thread;

use drut_mcp::query_structure::{query_structure, StructuralQueryInput};
use drut_mcp::source::ScriptSource;
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use serde_json::json;

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

/// Lines where `block_at` reports *something* — the only lines worth
/// spending a real LSP round trip on; every other line trivially agrees
/// (`kind: null` on both sides) since both read the identical function.
fn interesting_lines(text: &str) -> Vec<u32> {
    let result = voyager_core::parse(text);
    let line_count = text.lines().count() as u32;
    (1..=line_count)
        .filter(|&line| voyager_core::block_at(&result.nodes, &result.diagnostics, voyager_core::Position::new(line, 1)).is_some())
        .collect()
}

// --- Minimal LSP protocol driver (drut-lsp/tests/common's own helpers
// aren't importable from outside that crate, so this is a small, test-only
// duplication -- not grammar/parsing logic, Principle I doesn't apply to
// test scaffolding). ---

fn spawn_lsp() -> (Connection, thread::JoinHandle<()>) {
    let (server, client) = Connection::memory();
    let handle = thread::spawn(move || {
        drut_lsp::run(server);
    });
    (client, handle)
}

fn send_request(client: &Connection, id: i32, method: &str, params: serde_json::Value) {
    client
        .sender
        .send(Message::Request(Request {
            id: RequestId::from(id),
            method: method.to_string(),
            params,
        }))
        .unwrap();
}

fn send_notification(client: &Connection, method: &str, params: serde_json::Value) {
    let _ = client.sender.send(Message::Notification(Notification {
        method: method.to_string(),
        params,
    }));
}

fn recv_response(client: &Connection) -> Response {
    loop {
        match client.receiver.recv_timeout(std::time::Duration::from_secs(5)).unwrap() {
            Message::Response(r) => return r,
            _ => continue,
        }
    }
}

fn initialize(client: &Connection) {
    send_request(client, 1, "initialize", json!({"capabilities": {}}));
    recv_response(client);
    send_notification(client, "initialized", json!({}));
}

fn hover_at(client: &Connection, id: i32, uri: &str, line: u32) -> Option<String> {
    // 0-based LSP line, character 1 (matches this test's own 1-based column
    // 1 convention used for query_structure below).
    send_request(
        client,
        id,
        "textDocument/hover",
        json!({
            "textDocument": {"uri": uri},
            "position": {"line": line.saturating_sub(1), "character": 1}
        }),
    );
    let response = recv_response(client);
    let result = response.response_result.ok()?;
    if result.is_null() {
        return None;
    }
    result["contents"]["value"].as_str().map(|s| s.to_string())
}

#[test]
#[ignore = "requires DRUT_CORPUS_PATH pointing at a local WF-TDM-Official-Releases checkout"]
fn query_structure_matches_hover_for_every_block_line_across_the_corpus() {
    let corpus = corpus_path();
    let mut files = Vec::new();
    collect_script_files(&corpus, &mut files);
    assert!(!files.is_empty(), "expected at least one .s/.block file under {corpus:?}");

    let (client, _handle) = spawn_lsp();
    initialize(&client);

    let mut checked = 0usize;
    let mut mismatches = Vec::new();
    let mut req_id = 100;

    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue; // Non-UTF-8 on disk: didOpen can't carry it anyway.
        };
        let uri = format!("file:///corpus-{}.s", req_id);
        send_notification(
            &client,
            "textDocument/didOpen",
            json!({"textDocument": {"uri": &uri, "languageId": "drut-voyager", "version": 1, "text": &text}}),
        );
        // Drain the publishDiagnostics notification didOpen always sends.
        loop {
            match client.receiver.recv_timeout(std::time::Duration::from_secs(5)).unwrap() {
                Message::Notification(n) if n.method == "textDocument/publishDiagnostics" => break,
                _ => continue,
            }
        }

        for line in interesting_lines(&text) {
            req_id += 1;
            let hover_value = hover_at(&client, req_id, &uri, line);

            let qs_result = query_structure(&StructuralQueryInput {
                source: ScriptSource {
                    text: Some(text.clone()),
                    path: None,
                },
                line,
                column: 1,
            })
            .unwrap();

            checked += 1;
            match (&hover_value, &qs_result.kind) {
                (None, None) => {}
                (Some(h), Some(k)) if h.contains(k.as_str()) => {}
                _ => mismatches.push(format!(
                    "{path:?}:{line}: hover={hover_value:?}, query_structure.kind={:?}",
                    qs_result.kind
                )),
            }
        }
    }

    assert!(checked > 0, "expected at least one block-containing line across the corpus");
    assert!(
        mismatches.is_empty(),
        "expected query_structure and hover to agree on all {checked} checked line(s); {} mismatch(es):\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}
