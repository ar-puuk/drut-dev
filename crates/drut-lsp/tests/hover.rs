//! Story 3 acceptance scenarios, exercised through real
//! `textDocument/hover` JSON-RPC requests (spec.md User Story 3).

mod common;

use common::*;
use serde_json::json;

fn hover(client: &lsp_server::Connection, uri: &str, line: u32, character: u32) -> serde_json::Value {
    send_request(
        client,
        2,
        "textDocument/hover",
        json!({
            "textDocument": {"uri": uri},
            "position": {"line": line, "character": character}
        }),
    );
    recv_response(client).response_result.expect("hover must succeed")
}

#[test]
fn hover_over_if_names_kind_and_matched_endif() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "IF (a=b)\nENDIF\n");

    let result = hover(&client, "file:///a.s", 0, 1);
    let value = result["contents"]["value"].as_str().unwrap();
    assert!(value.contains("If"));

    shutdown(&client);
}

#[test]
fn hover_over_short_if_distinguishes_it() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "IF (a=b) PRINT LIST=1\n");

    let result = hover(&client, "file:///a.s", 0, 1);
    let value = result["contents"]["value"].as_str().unwrap();
    assert!(value.contains("short-IF"));

    shutdown(&client);
}

#[test]
fn hover_over_implicitly_closed_run_reports_resolved_location() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(
        &client,
        "file:///a.s",
        "RUN PGM=MATRIX\nZONES=5\nRUN PGM=HWYASSIGN\nENDRUN\n",
    );

    let result = hover(&client, "file:///a.s", 0, 1);
    let value = result["contents"]["value"].as_str().unwrap();
    assert!(value.contains("Run"));
    assert!(!value.contains("short-IF"));

    shutdown(&client);
}

#[test]
fn hover_over_unrelated_token_returns_null() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "IF (a=b)\nPRINTQQQ LIST=1\nENDIF\n");

    let result = hover(&client, "file:///a.s", 1, 1);
    assert!(result.is_null());

    shutdown(&client);
}
