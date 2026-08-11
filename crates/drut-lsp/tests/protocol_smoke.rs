//! LSP-level protocol smoke test (research.md §9): drives `drut_lsp::run`
//! through a real `initialize`/`initialized`/`textDocument/didOpen` round
//! trip over `lsp_server::Connection::memory()` — no subprocess, but real
//! JSON-RPC messages, catching wire-format bugs a purely in-process
//! function-call test would miss.

mod common;

use common::*;
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
