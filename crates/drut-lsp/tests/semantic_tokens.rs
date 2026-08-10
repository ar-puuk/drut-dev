//! Story 6 acceptance scenarios, exercised through real
//! `textDocument/semanticTokens/full` JSON-RPC requests (spec.md User Story
//! 6), including the no-flag-on-`MisplacedBreak` case.

mod common;

use common::*;
use serde_json::json;

const SHORT_IF_TYPE: u64 = 0;
const STATEMENT_TYPE: u64 = 1;
const VARIABLE_TYPE: u64 = 2;
const UNREACHABLE_BIT: u64 = 1;

fn semantic_tokens(client: &lsp_server::Connection, uri: &str) -> Vec<u64> {
    send_request(
        client,
        2,
        "textDocument/semanticTokens/full",
        json!({"textDocument": {"uri": uri}}),
    );
    let result = recv_response(client)
        .response_result
        .expect("semanticTokens/full must succeed");
    result["data"]
        .as_array()
        .expect("SemanticTokens.data present")
        .iter()
        .map(|v| v.as_u64().unwrap())
        .collect()
}

#[test]
fn short_if_gets_distinguishable_token_type() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "IF (a=b) PRINT LIST=1\n");

    let data = semantic_tokens(&client, "file:///a.s");
    // 5 values per token: deltaLine, deltaStart, length, tokenType, modifiers.
    assert_eq!(data.len(), 5);
    assert_eq!(data[3], SHORT_IF_TYPE);

    shutdown(&client);
}

#[test]
fn block_style_if_is_not_flagged_as_short() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "IF (a=b)\nPRINT LIST=1\nENDIF\n");

    let data = semantic_tokens(&client, "file:///a.s");
    assert!(data.is_empty(), "no short-IF or unreachable tokens expected");

    shutdown(&client);
}

#[test]
fn statement_after_break_is_flagged_unreachable() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "LOOP\nBREAK\nPRINT LIST=1\nENDLOOP\n");

    let data = semantic_tokens(&client, "file:///a.s");
    assert_eq!(data.len(), 5);
    assert_eq!(data[3], STATEMENT_TYPE);
    assert_eq!(data[4] & UNREACHABLE_BIT, UNREACHABLE_BIT);

    shutdown(&client);
}

#[test]
fn variable_ref_gets_the_standard_variable_token_type() {
    // Added 2026-08-10 -- see semantic_tokens.rs's own comment for why:
    // theme-independent coloring, real corpus term, real manual-testing
    // finding (a custom TextMate scope alone wasn't reliably colored under
    // every theme).
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "IF (@MODE@ = 1)\nENDIF\n");

    let data = semantic_tokens(&client, "file:///a.s");
    assert_eq!(data.len(), 5);
    assert_eq!(data[3], VARIABLE_TYPE);

    shutdown(&client);
}

#[test]
fn misplaced_break_does_not_flag_anything() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "BREAK\nPRINT LIST=1\n");

    let data = semantic_tokens(&client, "file:///a.s");
    assert!(data.is_empty());

    shutdown(&client);
}
