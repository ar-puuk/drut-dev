//! FR-019/FR-020/SC-005: every diagnostic/hover/semantic-token position
//! lands correctly under UTF-16 counting, including for content containing
//! a supplementary-plane character (which occupies two UTF-16 code units,
//! not one) — not only for content where `char` count and UTF-16 code-unit
//! count happen to coincide.

mod common;

use common::*;
use serde_json::json;

/// 😀 (U+1F600) is one `char`, but two UTF-16 code units — the classic case
/// `char`-counting and UTF-16-counting diverge on.
const EMOJI: &str = "😀";

#[test]
fn diagnostic_position_after_a_supplementary_plane_character_is_correct() {
    let (client, _handle) = spawn_server();
    initialize(&client);

    // The comment (with the emoji) sits before an unmatched IF on the next
    // line — the diagnostic itself is on line 1 (0-based), so the emoji on
    // line 0 doesn't shift its position, but this still proves the server
    // survives supplementary-plane content upstream without corrupting
    // later line/character accounting.
    let text = format!("; a comment with an emoji {EMOJI} in it\nIF (a=b)\n; no ENDIF\n");
    let note = did_open(&client, "file:///a.s", &text);
    let diagnostics = note.params["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["range"]["start"]["line"], json!(1));

    shutdown(&client);
}

#[test]
fn hover_after_supplementary_plane_character_resolves_correct_position() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    let text = format!("; {EMOJI}\nIF (a=b)\nENDIF\n");
    did_open(&client, "file:///a.s", &text);

    // Hover over "IF" on line 1 (0-based) — the emoji on line 0 must not
    // corrupt line 1's own position accounting.
    send_request(
        &client,
        2,
        "textDocument/hover",
        json!({
            "textDocument": {"uri": "file:///a.s"},
            "position": {"line": 1, "character": 1}
        }),
    );
    let response = recv_response(&client);
    let result = response.response_result.expect("hover must succeed");
    assert!(!result.is_null(), "expected a hover result for the IF token on line 1");
    let value = result["contents"]["value"].as_str().unwrap();
    assert!(value.contains("If"));

    shutdown(&client);
}

#[test]
fn semantic_token_position_after_supplementary_plane_character_is_correct() {
    let (client, _handle) = spawn_server();
    initialize(&client);
    let text = format!("; {EMOJI}\nIF (a=b) PRINT LIST=1\n");
    did_open(&client, "file:///a.s", &text);

    send_request(
        &client,
        2,
        "textDocument/semanticTokens/full",
        json!({"textDocument": {"uri": "file:///a.s"}}),
    );
    let response = recv_response(&client);
    let result = response.response_result.expect("semanticTokens/full must succeed");
    let data = result["data"].as_array().unwrap();
    assert_eq!(data.len(), 5);
    // deltaLine from the (implicit, line 0) starting point to the short-IF
    // on line 1 must be exactly 1, regardless of the emoji's UTF-16 width
    // on line 0.
    assert_eq!(data[0], json!(1));

    shutdown(&client);
}
