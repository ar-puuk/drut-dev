//! Story 4 acceptance scenarios, exercised through real
//! `textDocument/completion` JSON-RPC requests (spec.md User Story 4),
//! including the context-scoped-vs-general-fallback split and the
//! FR-019-boundary regression guard (T025).

mod common;

use common::*;
use serde_json::json;

fn complete(client: &lsp_server::Connection, uri: &str, line: u32, character: u32) -> Vec<String> {
    send_request(
        client,
        2,
        "textDocument/completion",
        json!({
            "textDocument": {"uri": uri},
            "position": {"line": line, "character": character}
        }),
    );
    let result = recv_response(client).response_result.expect("completion must succeed");
    result
        .as_array()
        .expect("Array(CompletionItem) response shape")
        .iter()
        .map(|item| item["label"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn completion_at_start_of_statement_offers_general_control_words() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "\n");

    let labels = complete(&client, "file:///a.s", 0, 0);
    assert!(labels.contains(&"IF".to_string()));
    assert!(labels.contains(&"RUN".to_string()));

    shutdown(&client);
}

#[test]
fn completion_inside_comment_offers_nothing() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "; a comment here\n");

    let labels = complete(&client, "file:///a.s", 0, 5);
    assert!(labels.is_empty());

    shutdown(&client);
}

#[test]
fn completion_inside_quoted_string_offers_nothing() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "PRINT LIST='hello world'\n");

    let labels = complete(&client, "file:///a.s", 0, 16); // inside the quotes.
    assert!(labels.is_empty());

    shutdown(&client);
}

#[test]
fn run_pgm_hwyassign_and_run_pgm_matrix_offer_identical_completions() {
    // Regression guard: completion scoping MUST NOT vary by a control
    // word's PGM= value (FR-012's explicit FR-019 boundary).
    let (client, _handle) = spawn_server();
    initialize(&client);
    did_open(&client, "file:///a.s", "RUN PGM=HWYASSIGN\nENDRUN\n");
    did_open(&client, "file:///b.s", "RUN PGM=MATRIX\nENDRUN\n");

    let labels_a = complete(&client, "file:///a.s", 0, 5);
    let labels_b = complete(&client, "file:///b.s", 0, 5);
    assert_eq!(labels_a, labels_b);

    shutdown(&client);
}
