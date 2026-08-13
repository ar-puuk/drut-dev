//! LSP-level protocol smoke test (research.md §9): drives `drut_lsp::run`
//! through a real `initialize`/`initialized`/`textDocument/didOpen` round
//! trip over `lsp_server::Connection::memory()` — no subprocess, but real
//! JSON-RPC messages, catching wire-format bugs a purely in-process
//! function-call test would miss.

mod common;

use common::*;
use lsp_server::Connection;
use serde_json::json;

#[test]
fn initialize_handshake_declares_utf16_position_encoding() {
    let (client, _handle) = spawn_server();

    send_request(&client, 1, "initialize", json!({"capabilities": {}}));
    let response = recv_response(&client);
    let result = response.response_result.expect("initialize must succeed");
    assert_eq!(result["capabilities"]["positionEncoding"], json!("utf-16"));
    // Added 2026-08-10 (formatting.rs).
    assert_eq!(result["capabilities"]["documentFormattingProvider"], json!(true));

    send_notification(&client, "initialized", json!({}));
    shutdown(&client);
}

#[test]
fn startup_logs_the_running_binary_path_and_build_identifier() {
    // Added 2026-08-11: a real bug report turned out not to be a code
    // defect (confirmed via a real LSP-protocol test), leaving PATH-
    // resolution divergence between environments as the leading suspect --
    // this proves the server actually reports which binary/build it is,
    // via the LSP-standard window/logMessage notification, not just that
    // the code compiles.
    let (client, _handle) = spawn_server();

    send_request(&client, 1, "initialize", json!({"capabilities": {}}));
    recv_response(&client);
    // lsp_server::Connection::initialize() blocks server-side until it
    // receives this notification -- log_startup_info() only runs after,
    // so it must be sent before waiting for the log message.
    send_notification(&client, "initialized", json!({}));

    let note = recv_notification(&client, "window/logMessage");
    let message = note.params["message"].as_str().expect("message must be a string");
    assert!(
        message.contains("binary:"),
        "expected the startup log to report the running binary's path, got: {message}"
    );
    assert!(
        message.contains("commit:"),
        "expected the startup log to report a build/commit identifier, got: {message}"
    );
    // The binary path must be this test's own freshly-built executable,
    // not some other drut-lsp resolved from elsewhere -- proves the log
    // reports reality, not a hardcoded placeholder.
    let exe_path = std::env::current_exe().unwrap();
    let exe_name = exe_path.file_name().unwrap().to_string_lossy();
    assert!(
        message.contains(exe_name.as_ref()),
        "expected the logged binary path to reference this test binary ({exe_name}), got: {message}"
    );

    shutdown(&client);
}

#[test]
fn formatting_request_round_trips_a_real_edit() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "IF (a=b)\nPRINT LIST=1\nENDIF\n");

    send_request(
        &client,
        2,
        "textDocument/formatting",
        json!({
            "textDocument": {"uri": "file:///a.s"},
            "options": {"tabSize": 4, "insertSpaces": true}
        }),
    );
    let response = recv_response(&client);
    let result = response.response_result.expect("formatting must succeed");
    let edits = result.as_array().expect("edits array");
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0]["newText"], json!("IF (a=b)\n    PRINT LIST=1\nENDIF\n"));

    shutdown(&client);
}

#[test]
fn did_open_publishes_diagnostics_for_a_broken_document() {
    let (client, _handle) = spawn_server();
    initialize(&client);

    let note = did_open(&client, "file:///broken.s", "IF (a=b)\n; no ENDIF\n");
    let diagnostics = note.params["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["code"], json!("UnmatchedIf"));

    shutdown(&client);
}

#[test]
fn formatting_after_a_diagnosed_block_is_closed_no_longer_leaves_residue() {
    // 007-formatter-diagnosed-block-indent-fix, exercised through the real
    // LSP protocol -- textDocument/didOpen, textDocument/formatting,
    // textDocument/didChange, textDocument/formatting again -- not just
    // voyager-core::format directly and not just drut-cli. The exact
    // PROCESS/RUN sequence that surfaced the bug during manual VS Code
    // verification.
    let (client, _handle) = spawn_server();
    initialize(&client);

    let step1 = "PROCESS PHASE=INPUT\n    FILEI = ni.1\n    LOOP DAY = 1, 5\n        PRINT LIST='Day = ', DAY\n    ENDLOOP\n\nRUN PGM=HWYASSIGN\n    FILEI NETI = 'net.net'\nENDRUN\n";
    did_open(&client, "file:///residue.s", step1);

    send_request(
        &client,
        2,
        "textDocument/formatting",
        json!({
            "textDocument": {"uri": "file:///residue.s"},
            "options": {"tabSize": 4, "insertSpaces": true}
        }),
    );
    let response = recv_response(&client);
    let edits = response.response_result.expect("formatting must succeed");
    assert_eq!(
        edits,
        json!([]),
        "pass 1 (PROCESS still unclosed) must leave RUN untouched via the real LSP path too, got {edits:?}"
    );

    // Simulate the user typing ENDPROCESS by hand -- full-sync didChange,
    // matching this server's declared TextDocumentSyncKind::FULL.
    let step2 = step1.replacen("    ENDLOOP\n\n", "    ENDLOOP\nENDPROCESS\n\n", 1);
    send_notification(
        &client,
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": "file:///residue.s", "version": 2},
            "contentChanges": [{"text": step2}]
        }),
    );
    recv_notification(&client, "textDocument/publishDiagnostics"); // the didChange's own diagnostics push

    send_request(
        &client,
        3,
        "textDocument/formatting",
        json!({
            "textDocument": {"uri": "file:///residue.s"},
            "options": {"tabSize": 4, "insertSpaces": true}
        }),
    );
    let response = recv_response(&client);
    let edits = response.response_result.expect("formatting must succeed");
    assert_eq!(
        edits,
        json!([]),
        "pass 2 (PROCESS now closed) must report the file already correctly formatted via the real LSP path -- RUN must not be stuck at a stale nested indent, got {edits:?}"
    );

    shutdown(&client);
}

#[test]
fn did_open_publishes_unmatched_process_for_a_genuinely_unclosed_phase() {
    // 006-unmatched-process-diagnostic FR-007: proves drut-lsp's real
    // publishDiagnostics path surfaces the new kind end to end, not just
    // that voyager-core::parse itself reports it.
    let (client, _handle) = spawn_server();
    initialize(&client);

    let note = did_open(&client, "file:///unclosed_phase.s", "PROCESS PHASE=INPUT\nFILEI=ni.1\n");
    let diagnostics = note.params["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["code"], json!("UnmatchedProcess"));

    shutdown(&client);
}

#[test]
fn did_open_on_valid_document_publishes_zero_diagnostics() {
    let (client, _handle) = spawn_server();
    initialize(&client);

    let note = did_open(&client, "file:///clean.s", "IF (a=b)\nENDIF\n");
    let diagnostics = note.params["diagnostics"].as_array().unwrap();
    assert!(diagnostics.is_empty());

    shutdown(&client);
}

#[test]
fn did_close_clears_diagnostics() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "IF (a=b)\n; no ENDIF\n");

    send_notification(
        &client,
        "textDocument/didClose",
        json!({"textDocument": {"uri": "file:///a.s"}}),
    );
    let note = recv_notification(&client, "textDocument/publishDiagnostics");
    let diagnostics = note.params["diagnostics"].as_array().unwrap();
    assert!(diagnostics.is_empty());

    shutdown(&client);
}

#[test]
fn did_open_publishes_an_unclosed_fmt_off_hint_distinct_from_structural_diagnostics() {
    // 010-fmt-region-markers T020: an unmatched '; FMT: OFF' publishes
    // through the same textDocument/publishDiagnostics cycle as structural
    // diagnostics, but as its own additive, HINT-severity, "drut-fmt"-
    // sourced stream — never a voyager_core::DiagnosticKind.
    let (client, _handle) = spawn_server();
    initialize(&client);

    let note = did_open(&client, "file:///unclosed_marker.s", "IF (X=1)\n; FMT: OFF\nY = 1\nENDIF\n");
    let diagnostics = note.params["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1, "expected exactly one diagnostic, got: {diagnostics:?}");
    assert_eq!(diagnostics[0]["code"], json!("UnclosedFmtOff"));
    assert_eq!(diagnostics[0]["source"], json!("drut-fmt"));
    assert_eq!(diagnostics[0]["severity"], json!(4), "HINT is severity 4 in the LSP spec, distinct from ERROR (1)");

    shutdown(&client);
}

#[test]
fn did_open_publishes_zero_fmt_off_hints_for_a_clean_document() {
    let (client, _handle) = spawn_server();
    initialize(&client);

    let note = did_open(&client, "file:///clean_markers.s", "IF (X=1)\n; FMT: OFF\nY = 1\n; FMT: ON\nENDIF\n");
    let diagnostics = note.params["diagnostics"].as_array().unwrap();
    assert!(diagnostics.is_empty(), "every marker matched -- expected zero diagnostics, got: {diagnostics:?}");

    shutdown(&client);
}

fn folding_ranges(client: &Connection, id: i32, uri: &str) -> Vec<serde_json::Value> {
    send_request(client, id, "textDocument/foldingRange", json!({"textDocument": {"uri": uri}}));
    let response = recv_response(client);
    let result = response.response_result.expect("foldingRange must succeed");
    result.as_array().expect("folding ranges array").clone()
}

#[test]
fn initialize_handshake_declares_folding_range_support() {
    let (client, _handle) = spawn_server();
    send_request(&client, 1, "initialize", json!({"capabilities": {}}));
    let response = recv_response(&client);
    let result = response.response_result.expect("initialize must succeed");
    assert_eq!(result["capabilities"]["foldingRangeProvider"], json!(true));
    send_notification(&client, "initialized", json!({}));
    shutdown(&client);
}

#[test]
fn folding_range_us1_scenario_1_if_block_hides_exactly_the_lines_between() {
    // spec.md US1 Acceptance Scenario 1.
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "IF (a=b)\nX = 1\nY = 2\nENDIF\n");

    let ranges = folding_ranges(&client, 2, "file:///a.s");
    assert_eq!(ranges.len(), 1, "got {ranges:?}");
    assert_eq!(ranges[0]["startLine"], json!(0));
    assert_eq!(ranges[0]["endLine"], json!(3));
    assert_eq!(ranges[0]["kind"], json!("region"));

    shutdown(&client);
}

#[test]
fn folding_range_us1_scenario_2_nested_loop_inside_if_produces_two_independent_ranges() {
    // spec.md US1 Acceptance Scenario 2.
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "IF (a=b)\nLOOP i=1,5\nX = 1\nENDLOOP\nENDIF\n");

    let ranges = folding_ranges(&client, 2, "file:///a.s");
    assert_eq!(ranges.len(), 2, "got {ranges:?}");
    let outer = ranges.iter().find(|r| r["startLine"] == json!(0)).expect("outer IF range");
    assert_eq!(outer["endLine"], json!(4));
    let inner = ranges.iter().find(|r| r["startLine"] == json!(1)).expect("inner LOOP range");
    assert_eq!(inner["endLine"], json!(3));
    // The inner range is fully contained within the outer's line span.
    assert!(inner["startLine"].as_u64().unwrap() > outer["startLine"].as_u64().unwrap());
    assert!(inner["endLine"].as_u64().unwrap() < outer["endLine"].as_u64().unwrap());

    shutdown(&client);
}

#[test]
fn folding_range_us1_scenario_3_block_comment_folds_from_open_to_close() {
    // spec.md US1 Acceptance Scenario 3.
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "/* first line\n   second line\n   third line */\nX = 1\n");

    let ranges = folding_ranges(&client, 2, "file:///a.s");
    assert_eq!(ranges.len(), 1, "got {ranges:?}");
    assert_eq!(ranges[0]["kind"], json!("comment"));
    assert_eq!(ranges[0]["startLine"], json!(0));
    assert_eq!(ranges[0]["endLine"], json!(2));

    shutdown(&client);
}

#[test]
fn folding_range_us2_scenario_1_live_edit_extends_the_range() {
    // spec.md US2 Acceptance Scenario 1 / FR-010: a folding-range request
    // after a didChange reflects the document's current text, not a stale
    // parse.
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "LOOP i=1,5\nX = 1\nENDLOOP\n");

    let before = folding_ranges(&client, 2, "file:///a.s");
    assert_eq!(before[0]["endLine"], json!(2));

    send_notification(
        &client,
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": "file:///a.s", "version": 2},
            "contentChanges": [{"text": "LOOP i=1,5\nX = 1\nY = 2\nENDLOOP\n"}]
        }),
    );
    recv_notification(&client, "textDocument/publishDiagnostics");

    let after = folding_ranges(&client, 3, "file:///a.s");
    assert_eq!(
        after[0]["endLine"],
        json!(3),
        "the added line must shift ENDLOOP's line, and the range must reflect it, got {after:?}"
    );

    shutdown(&client);
}

#[test]
fn folding_range_us2_scenario_2_deleting_the_closer_removes_the_range() {
    // spec.md US2 Acceptance Scenario 2 / FR-005, proven as a live-edit
    // scenario specifically (distinct from the static-document unmatched-
    // block coverage in folding.rs's own unit tests).
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "IF (a=b)\nX = 1\nENDIF\n");

    let before = folding_ranges(&client, 2, "file:///a.s");
    assert_eq!(before.len(), 1);

    send_notification(
        &client,
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": "file:///a.s", "version": 2},
            "contentChanges": [{"text": "IF (a=b)\nX = 1\n"}]
        }),
    );
    recv_notification(&client, "textDocument/publishDiagnostics");

    let after = folding_ranges(&client, 3, "file:///a.s");
    assert!(after.is_empty(), "an IF with no ENDIF must offer no fold range, got {after:?}");

    shutdown(&client);
}

#[test]
fn folding_range_us3_fold_all_coverage_matches_every_foldable_construct_exactly() {
    // spec.md US3 / SC-002: one of every block kind, a nested block, and a
    // block comment -- the returned set must match exactly, with no
    // omission beyond FR-004/FR-005/FR-007/FR-008's documented exceptions.
    let (client, _handle) = spawn_server();
    initialize(&client);
    let source = "/* header comment */\nIF (a=b)\nLOOP i=1,5\nJLOOP\nX = 1\nENDJLOOP\nENDLOOP\nENDIF\nRUN PGM=MATRIX\nZONES=5\nENDRUN\nPROCESS PHASE=INPUT\nFILEI=ni.1\nENDPROCESS\nDISTRIBUTEMULTISTEP PROCESSNUM=4\nX = 1\nENDDISTRIBUTEMULTISTEP\nIF (a=b) PRINT LIST=1\n";
    // header comment is a single line -> excluded (FR-008); short-IF on the
    // last line -> excluded (FR-004). Every other construct is foldable.
    did_open(&client, "file:///a.s", source);

    let ranges = folding_ranges(&client, 2, "file:///a.s");
    let regions = ranges.iter().filter(|r| r["kind"] == json!("region")).count();
    let comments = ranges.iter().filter(|r| r["kind"] == json!("comment")).count();
    assert_eq!(regions, 6, "IF, LOOP, JLOOP, RUN, PROCESS, DISTRIBUTEMULTISTEP -- got {ranges:?}");
    assert_eq!(comments, 0, "the header comment is single-line, correctly excluded -- got {ranges:?}");
    assert_eq!(ranges.len(), 6, "total must match exactly, got {ranges:?}");

    shutdown(&client);
}

#[test]
fn did_change_reparses_and_republishes() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "IF (a=b)\n; no ENDIF\n");

    send_notification(
        &client,
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": "file:///a.s", "version": 2},
            "contentChanges": [{"text": "IF (a=b)\nENDIF\n"}]
        }),
    );
    let note = recv_notification(&client, "textDocument/publishDiagnostics");
    let diagnostics = note.params["diagnostics"].as_array().unwrap();
    assert!(diagnostics.is_empty(), "fixed content should clear the diagnostic without reopening");

    shutdown(&client);
}
