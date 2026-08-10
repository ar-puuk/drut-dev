//! Story 5 acceptance scenarios, exercised through real `textDocument/hover`
//! JSON-RPC requests — spell-check rides on hover, not a distinct method
//! (spec.md User Story 5, `contracts/lsp-capabilities.md`).

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
fn close_typo_gets_a_did_you_mean_nudge() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    // "FI" is a transposition of "IF" — parsed as an Assignment (no `=`
    // makes it ambiguous, so this uses a clearly-non-control-word context
    // instead: a bare word statement).
    did_open(&client, "file:///a.s", "FI\n");

    let result = hover(&client, "file:///a.s", 0, 0);
    let value = result["contents"]["value"].as_str().unwrap();
    assert!(value.to_lowercase().contains("did you mean"), "value was: {value}");
    assert!(value.contains("IF"));

    shutdown(&client);
}

#[test]
fn exact_match_gets_no_nudge() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "IF (a=b)\nENDIF\n");

    // Hovering the IF itself returns block-structure info, not a spell
    // nudge — confirms no false-positive nudge fires for a correctly
    // spelled keyword.
    let result = hover(&client, "file:///a.s", 0, 1);
    let value = result["contents"]["value"].as_str().unwrap();
    assert!(!value.to_lowercase().contains("did you mean"));

    shutdown(&client);
}

#[test]
fn unrelated_token_gets_no_nudge() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "XYZZY123NOTHINGLIKEIT\n");

    let result = hover(&client, "file:///a.s", 0, 0);
    assert!(result.is_null());

    shutdown(&client);
}
