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
