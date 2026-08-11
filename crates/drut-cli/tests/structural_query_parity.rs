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
//!
//! **Strengthened 2026-08-10, a follow-up correction found during review,
//! not part of the original implementation pass**: the original comparison
//! only checked block-kind-name substring containment in hover's rendered
//! text -- it never extracted or compared the actual counterpart span from
//! either side. Since this whole extraction exists specifically because of
//! the implicitly-closed `Run`/`Process` counterpart derivation
//! (CHK015/finding I1, Phase 3), a kind-only check could not have caught a
//! regression in exactly that logic. `parse_hover_fact` below now extracts
//! the counterpart line (and `is_short_if`) from hover's markdown text, and
//! the comparison fails on any mismatch there too, not just kind. Real
//! investigation (`checked 7388 lines across 161 files, 0 implicit-close
//! cases`) further found, and independently confirmed via a raw text scan
//! with no dependency on `block_at` at all, that the real corpus as of this
//! date genuinely contains **zero** implicitly-closed `Run`/`Process`
//! blocks -- every opener in all 161 files has its own explicit closer.
//! That's a real property of this disciplined corpus, not a test bug, so
//! `implicit_close_parity_on_a_synthetic_case` below proves cross-adapter
//! parity on that specific derivation using a synthetic case instead, since
//! the real corpus provably cannot supply that evidence today.

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

/// A hover value's own resolved fact, parsed back out of its markdown text
/// -- the exact three shapes `hover.rs`'s own `handle` produces (`"**Kind**"`,
/// `"**Kind** (self-closing short-IF — no separate closer)"`, or `"**Kind**
/// — matched counterpart at line N"`). Parsing the rendered text back into
/// structured data is inherently a little fragile compared to comparing
/// structured values directly, but `drut-lsp`'s hover response *is*
/// markdown text by protocol design (`contracts/lsp-capabilities.md`) --
/// this is the only data actually on the wire to compare against.
#[derive(Debug, PartialEq, Eq)]
struct HoverFact {
    kind: String,
    is_short_if: bool,
    counterpart_line: Option<u32>,
}

fn parse_hover_fact(value: &str) -> HoverFact {
    let kind = value
        .strip_prefix("**")
        .and_then(|rest| rest.split_once("**"))
        .map(|(kind, _)| kind.to_string())
        .unwrap_or_default();

    let is_short_if = value.contains("self-closing short-IF");

    let counterpart_line = value.split("matched counterpart at line ").nth(1).and_then(|tail| tail.trim().parse::<u32>().ok());

    HoverFact {
        kind,
        is_short_if,
        counterpart_line,
    }
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
    // Specifically: how many checked lines were an implicitly-closed
    // Run/Process block (kind Run/Process, a counterpart present, and not
    // itself the explicit-closer line -- i.e. genuinely exercising the
    // implicit-close derivation this whole extraction exists for, CHK015/
    // finding I1, not just any Run/Process line). Reported explicitly so a
    // 100% pass can't be mistaken for "trivially passed because the corpus
    // never actually hit this case."
    let mut implicit_close_cases = 0usize;

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
            match &hover_value {
                None => {
                    if qs_result.kind.is_some() {
                        mismatches.push(format!(
                            "{path:?}:{line}: hover=None, query_structure.kind={:?}",
                            qs_result.kind
                        ));
                    }
                }
                Some(h) => {
                    let hover_fact = parse_hover_fact(h);
                    let Some(kind) = &qs_result.kind else {
                        mismatches.push(format!(
                            "{path:?}:{line}: hover={hover_fact:?}, query_structure.kind=None"
                        ));
                        continue;
                    };
                    if &hover_fact.kind != kind {
                        mismatches.push(format!(
                            "{path:?}:{line}: kind mismatch -- hover={:?}, query_structure={:?}",
                            hover_fact.kind, kind
                        ));
                    }
                    if hover_fact.is_short_if != qs_result.is_short_if {
                        mismatches.push(format!(
                            "{path:?}:{line}: is_short_if mismatch -- hover={}, query_structure={}",
                            hover_fact.is_short_if, qs_result.is_short_if
                        ));
                    }
                    // The counterpart location itself -- the whole reason
                    // this extraction exists (CHK015/finding I1, Phase 3):
                    // an implicitly-closed RUN/PROCESS block's resolved
                    // location is exactly the derivation a kind-only check
                    // would never catch a regression in.
                    if hover_fact.counterpart_line != qs_result.counterpart_start_line {
                        mismatches.push(format!(
                            "{path:?}:{line}: counterpart line mismatch -- hover={:?}, query_structure={:?}",
                            hover_fact.counterpart_line, qs_result.counterpart_start_line
                        ));
                    }

                    // Evidence tracking: is this specifically an
                    // implicitly-closed Run/Process case? Counted from
                    // query_structure's own result alone (never gated on
                    // hover agreeing) -- this needs to be real, independent
                    // evidence that the corpus *contains* such cases,
                    // available even if it turns out the two disagree on
                    // them. The counterpart line's own text not containing
                    // an explicit closer keyword is the practical signal
                    // (an explicit close's counterpart *is* that
                    // ENDRUN/ENDPROCESS/ENDPHASE line, by rule 1; an
                    // implicit close's counterpart is just wherever the
                    // block's last real content ends).
                    if kind == "Run" || kind == "Process" {
                        if let Some(counterpart_line) = qs_result.counterpart_start_line {
                            let counterpart_text = text.lines().nth((counterpart_line - 1) as usize).unwrap_or("").to_uppercase();
                            let is_explicit_closer = counterpart_text.contains("ENDRUN")
                                || counterpart_text.contains("ENDPROCESS")
                                || counterpart_text.contains("ENDPHASE");
                            if !is_explicit_closer {
                                implicit_close_cases += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    assert!(checked > 0, "expected at least one block-containing line across the corpus");
    // NOTE, established 2026-08-10 by direct investigation (both via this
    // heuristic and, independently, a raw text scan with no dependency on
    // block_at at all): the real WF-TDM-Official-Releases corpus, as of
    // this pass, genuinely contains **zero** implicitly-closed Run/Process
    // blocks -- every RUN and PROCESS/PHASE opener in all 161 files is
    // followed by its own explicit ENDRUN/ENDPROCESS/ENDPHASE before the
    // next opener. This is a real property of this specific, disciplined
    // corpus, not a bug in this test or in `block_at` -- so this count is
    // reported as evidence, not asserted `> 0`; see
    // `implicit_close_parity_on_a_synthetic_case` below for where this
    // derivation's cross-adapter parity is actually proven, since the real
    // corpus provably cannot provide that evidence today.
    println!(
        "structural_query_parity: checked {checked} block-containing line(s) across {} file(s), \
         {implicit_close_cases} of them real implicitly-closed Run/Process case(s) (see note above \
         if this is 0 -- expected for this corpus as of 2026-08-10), {} mismatch(es)",
        files.len(),
        mismatches.len()
    );
    assert!(
        mismatches.is_empty(),
        "expected query_structure and hover to agree (kind, is_short_if, and counterpart location) on all {checked} checked line(s); {} mismatch(es):\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

/// The real corpus (as of 2026-08-10) provably contains zero implicitly-
/// closed Run/Process blocks (see the note in the test above) -- so cross-
/// adapter parity on that *specific* derivation needs a synthetic case
/// instead, using the exact scenario `drut-lsp`'s own
/// `hover_over_implicitly_closed_run_reports_resolved_location` test
/// already covers on the `drut-lsp` side alone. This is the test that
/// actually proves what `/speckit-analyze`-style scrutiny asked for: that
/// `query_structure` and `hover` agree specifically on an implicitly-closed
/// case, not just that neither crashes on one.
#[test]
fn implicit_close_parity_on_a_synthetic_case() {
    let text = "RUN PGM=MATRIX\nZONES=5\nRUN PGM=HWYASSIGN\nENDRUN\n";
    let uri = "file:///synthetic-implicit-close.s";

    let (client, _handle) = spawn_lsp();
    initialize(&client);
    send_notification(
        &client,
        "textDocument/didOpen",
        json!({"textDocument": {"uri": uri, "languageId": "drut-voyager", "version": 1, "text": text}}),
    );
    loop {
        match client.receiver.recv_timeout(std::time::Duration::from_secs(5)).unwrap() {
            Message::Notification(n) if n.method == "textDocument/publishDiagnostics" => break,
            _ => continue,
        }
    }

    // Line 1: the first RUN's own opener -- implicitly closed by the
    // second RUN on line 3, with no ENDRUN of its own.
    let hover_value = hover_at(&client, 1, uri, 1).expect("hover must resolve a fact on the first RUN's own opener line");
    let hover_fact = parse_hover_fact(&hover_value);

    let qs_result = query_structure(&StructuralQueryInput {
        source: ScriptSource {
            text: Some(text.to_string()),
            path: None,
        },
        line: 1,
        column: 1,
    })
    .unwrap();

    assert_eq!(hover_fact.kind, "Run");
    assert_eq!(qs_result.kind.as_deref(), Some("Run"));
    assert!(!hover_fact.is_short_if);
    assert!(!qs_result.is_short_if);
    // The first RUN's body ends at line 2 (ZONES=5) -- not line 3, where
    // the second RUN's own opener sits. Both adapters must agree on this
    // exact resolved location, the whole point of the extraction.
    assert_eq!(hover_fact.counterpart_line, Some(2));
    assert_eq!(qs_result.counterpart_start_line, Some(2));
    assert_eq!(
        hover_fact.counterpart_line, qs_result.counterpart_start_line,
        "hover and query_structure must agree on the implicitly-closed RUN's resolved counterpart location"
    );
}
