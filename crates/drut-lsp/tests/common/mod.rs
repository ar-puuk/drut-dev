//! Shared `Connection::memory()`-driving helpers for `drut-lsp`'s LSP-level
//! test suite (research.md §9) — used by every `tests/*.rs` file that needs
//! a real JSON-RPC round trip rather than calling handler functions
//! directly.

use std::thread;

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use serde_json::json;

pub fn spawn_server() -> (Connection, thread::JoinHandle<()>) {
    let (server, client) = Connection::memory();
    let handle = thread::spawn(move || {
        drut_lsp::run(server);
    });
    (client, handle)
}

pub fn send_request(client: &Connection, id: i32, method: &str, params: serde_json::Value) {
    client
        .sender
        .send(Message::Request(Request {
            id: RequestId::from(id),
            method: method.to_string(),
            params,
        }))
        .unwrap();
}

pub fn send_notification(client: &Connection, method: &str, params: serde_json::Value) {
    // Best-effort: after a `shutdown` response, this server's loop has
    // already broken and its receiver is dropped, so a trailing `exit`
    // send may legitimately fail — that's expected, not a test bug.
    let _ = client.sender.send(Message::Notification(Notification {
        method: method.to_string(),
        params,
    }));
}

pub fn recv_response(client: &Connection) -> Response {
    loop {
        match client.receiver.recv_timeout(std::time::Duration::from_secs(5)).unwrap() {
            Message::Response(r) => return r,
            _ => continue, // skip any notifications (e.g. publishDiagnostics) in between.
        }
    }
}

pub fn recv_notification(client: &Connection, method: &str) -> Notification {
    loop {
        match client.receiver.recv_timeout(std::time::Duration::from_secs(5)).unwrap() {
            Message::Notification(n) if n.method == method => return n,
            _ => continue,
        }
    }
}

/// Runs the full `initialize`/`initialized` handshake, draining the
/// response — every test starts here.
pub fn initialize(client: &Connection) {
    send_request(client, 1, "initialize", json!({"capabilities": {}}));
    recv_response(client);
    send_notification(client, "initialized", json!({}));
}

/// Opens `uri` with `text` and returns the resulting
/// `textDocument/publishDiagnostics` notification (every `didOpen` produces
/// exactly one) — callers that don't care about diagnostics can just
/// discard the return value.
pub fn did_open(client: &Connection, uri: &str, text: &str) -> Notification {
    send_notification(
        client,
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "drut-voyager",
                "version": 1,
                "text": text
            }
        }),
    );
    recv_notification(client, "textDocument/publishDiagnostics")
}

pub fn shutdown(client: &Connection) {
    send_request(client, 999, "shutdown", json!(null));
    recv_response(client);
    send_notification(client, "exit", json!(null));
}
