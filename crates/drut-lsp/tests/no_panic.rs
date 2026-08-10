//! FR-004's no-panic guarantee, specifically for `drut-lsp`'s own new code
//! paths (hover, completion, spellcheck, semantic tokens) — not only the
//! guarantee `voyager-core` already proves for its own parsing logic
//! (`/speckit-analyze` finding E1). Every request in this suite is expected
//! to return a well-formed response (possibly `null`/empty), never crash
//! the server thread.

mod common;

use common::*;
use serde_json::json;

/// Malformed/edge-case document *text* content — never raw invalid bytes,
/// since those are structurally unreachable via `didOpen`/`didChange`
/// (research.md §12); this sweep is about malformed-but-valid Unicode text.
const EDGE_CASE_DOCUMENTS: &[(&str, &str)] = &[
    ("empty document", ""),
    ("only whitespace", "   \n\n\t\n"),
    ("only a comment", "; just a comment, nothing else"),
    ("truncated mid-statement", "IF (a=b"),
    ("unterminated block comment", "/* never closed"),
    ("replacement character as ordinary text", "PRINT LIST=\u{FFFD}\n"),
    (
        "supplementary-plane character at a boundary",
        "IF (a=b) PRINT LIST=😀\n",
    ),
    ("deeply nested block comments", "/*/*/*/*/* five deep */*/*/*/*/\n"),
    ("only a BREAK, misplaced", "BREAK\n"),
    ("stray closer with nothing open", "ENDIF\nENDLOOP\nENDRUN\n"),
];

fn exercise_every_handler(client: &lsp_server::Connection, uri: &str) {
    // hover, at a handful of positions across the (short) document.
    for line in 0..3u32 {
        for character in [0u32, 5, 50] {
            send_request(
                client,
                10,
                "textDocument/hover",
                json!({
                    "textDocument": {"uri": uri},
                    "position": {"line": line, "character": character}
                }),
            );
            let _ = recv_response(client); // must respond, ok() or not — never silence.
        }
    }

    // completion, at the same spread of positions.
    for line in 0..3u32 {
        send_request(
            client,
            11,
            "textDocument/completion",
            json!({
                "textDocument": {"uri": uri},
                "position": {"line": line, "character": 0}
            }),
        );
        let _ = recv_response(client);
    }

    // semantic tokens, whole document.
    send_request(
        client,
        12,
        "textDocument/semanticTokens/full",
        json!({"textDocument": {"uri": uri}}),
    );
    let _ = recv_response(client);

    // formatting, whole document (added 2026-08-10, formatting.rs).
    send_request(
        client,
        13,
        "textDocument/formatting",
        json!({
            "textDocument": {"uri": uri},
            "options": {"tabSize": 4, "insertSpaces": true}
        }),
    );
    let _ = recv_response(client);
}

#[test]
fn every_handler_survives_every_edge_case_document_without_panicking() {
    let (client, _handle) = spawn_server();
    initialize(&client);

    for (i, (name, text)) in EDGE_CASE_DOCUMENTS.iter().enumerate() {
        let uri = format!("file:///edge-case-{i}.s");
        // didOpen itself must not panic and must publish a well-formed
        // (possibly empty) diagnostics notification.
        let note = did_open(&client, &uri, text);
        assert!(
            note.params["diagnostics"].is_array(),
            "case {name:?}: publishDiagnostics must always carry an array, even if empty"
        );

        exercise_every_handler(&client, &uri);

        // A didChange back to empty content, and to the same content again,
        // exercises the re-parse path without panicking either.
        send_notification(
            &client,
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": uri, "version": 2},
                "contentChanges": [{"text": ""}]
            }),
        );
        let _ = recv_notification(&client, "textDocument/publishDiagnostics");
    }

    // If the server thread panicked at any point above, the channel would
    // now be disconnected — one final round trip proves it's still alive.
    let note = did_open(&client, "file:///still-alive.s", "IF (a=b)\nENDIF\n");
    assert!(note.params["diagnostics"].as_array().unwrap().is_empty());

    shutdown(&client);
}
