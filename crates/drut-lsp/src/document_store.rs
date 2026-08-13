//! Open-document session state (data-model.md §2, FR-002).
//!
//! Always calls `voyager_core::parse`, never `parse_bytes` — a live LSP
//! document's `text` is a Rust `String`, already guaranteed valid UTF-8, and
//! `didOpen`/`didChange`'s JSON payload cannot carry anything else
//! (research.md §12). There is no encoding-fallback branch here.

use std::collections::HashMap;

use lsp_types::Uri;
use voyager_core::ParseResult;

/// One currently-open document, re-derived on every content change.
#[derive(Debug, Clone)]
pub struct OpenDocument {
    /// The document's current in-editor content — not necessarily saved to
    /// disk (FR-002).
    pub text: String,
    /// Always `voyager_core::parse(&text)` — see module docs.
    pub parse_result: ParseResult,
    /// The LSP document version last applied, so a stale/out-of-order
    /// `didChange` can be detected and ignored (FR-002, FR-006).
    pub version: i32,
}

impl OpenDocument {
    fn new(text: String, version: i32) -> Self {
        let parse_result = voyager_core::parse(&text);
        OpenDocument {
            text,
            parse_result,
            version,
        }
    }

    /// Replaces this document's content and re-derives `parse_result` in the
    /// same step — there is no window where `parse_result` could be read
    /// against stale `text` (data-model.md §2 validation rule).
    fn replace(&mut self, text: String, version: i32) {
        self.parse_result = voyager_core::parse(&text);
        self.text = text;
        self.version = version;
    }
}

/// Owns every open document for the running `drut server` process (FR-002).
#[derive(Debug, Default)]
pub struct ServerState {
    documents: HashMap<Uri, OpenDocument>,
    /// The client's workspace root, captured once at `initialize` time
    /// (012-toml-configuration/research.md §5) — used only as a `drut.toml`
    /// discovery fallback for a document with no real on-disk location
    /// (e.g. an unsaved buffer). `None` for a client that sends neither
    /// `rootUri` nor `workspaceFolders`, or before `initialize` completes —
    /// not a startup failure either way.
    workspace_root: Option<std::path::PathBuf>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState::default()
    }

    pub fn set_workspace_root(&mut self, root: Option<std::path::PathBuf>) {
        self.workspace_root = root;
    }

    pub fn workspace_root(&self) -> Option<&std::path::Path> {
        self.workspace_root.as_deref()
    }

    /// `textDocument/didOpen`: inserts (or replaces) the document.
    pub fn did_open(&mut self, uri: Uri, text: String, version: i32) {
        self.documents.insert(uri, OpenDocument::new(text, version));
    }

    /// `textDocument/didChange` (full sync — `TextDocumentSyncKind::Full`):
    /// replaces the document's content with `text`. A `version` not greater
    /// than the document's currently-tracked version is ignored (a stale or
    /// out-of-order notification), per data-model.md §2's staleness guard.
    pub fn did_change(&mut self, uri: &Uri, text: String, version: i32) {
        if let Some(doc) = self.documents.get_mut(uri) {
            if version > doc.version {
                doc.replace(text, version);
            }
        }
        // A didChange for a document we don't have open is silently
        // ignored — nothing to update, and no panic (FR-004).
    }

    /// `textDocument/didClose`: removes the document. Callers are
    /// responsible for publishing an empty diagnostics list afterward
    /// (FR-006) — that's a protocol concern, not state-storage.
    pub fn did_close(&mut self, uri: &Uri) {
        self.documents.remove(uri);
    }

    pub fn get(&self, uri: &Uri) -> Option<&OpenDocument> {
        self.documents.get(uri)
    }

    /// Every currently-open document's URI (013-lsp-config-file-watch,
    /// data-model.md) — used to re-publish diagnostics for every open
    /// document on a `workspace/didChangeWatchedFiles` event, not just one.
    /// No new stored field; a read-only view over `documents`.
    pub fn open_uris(&self) -> impl Iterator<Item = &Uri> {
        self.documents.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn uri(s: &str) -> Uri {
        Uri::from_str(s).unwrap()
    }

    #[test]
    fn did_open_inserts_and_parses() {
        let mut state = ServerState::new();
        state.did_open(uri("file:///a.s"), "IF (a=b)\nENDIF\n".to_string(), 1);
        let doc = state.get(&uri("file:///a.s")).unwrap();
        assert_eq!(doc.version, 1);
        assert!(doc.parse_result.diagnostics.is_empty());
    }

    #[test]
    fn did_change_replaces_and_reparses() {
        let mut state = ServerState::new();
        state.did_open(uri("file:///a.s"), "IF (a=b)\n".to_string(), 1);
        state.did_change(&uri("file:///a.s"), "IF (a=b)\nENDIF\n".to_string(), 2);
        let doc = state.get(&uri("file:///a.s")).unwrap();
        assert_eq!(doc.version, 2);
        assert_eq!(doc.text, "IF (a=b)\nENDIF\n");
    }

    #[test]
    fn stale_did_change_is_ignored() {
        let mut state = ServerState::new();
        state.did_open(uri("file:///a.s"), "original\n".to_string(), 5);
        state.did_change(&uri("file:///a.s"), "stale\n".to_string(), 3);
        let doc = state.get(&uri("file:///a.s")).unwrap();
        assert_eq!(doc.version, 5);
        assert_eq!(doc.text, "original\n");
    }

    #[test]
    fn did_change_for_unknown_document_does_not_panic() {
        let mut state = ServerState::new();
        state.did_change(&uri("file:///never-opened.s"), "text".to_string(), 1);
        assert!(state.get(&uri("file:///never-opened.s")).is_none());
    }

    #[test]
    fn did_close_removes_document() {
        let mut state = ServerState::new();
        state.did_open(uri("file:///a.s"), "text\n".to_string(), 1);
        state.did_close(&uri("file:///a.s"));
        assert!(state.get(&uri("file:///a.s")).is_none());
    }

    #[test]
    fn open_uris_is_empty_with_no_documents_open() {
        let state = ServerState::new();
        assert_eq!(state.open_uris().count(), 0);
    }

    #[test]
    fn open_uris_returns_every_open_document() {
        let mut state = ServerState::new();
        state.did_open(uri("file:///a.s"), "IF (a=b)\nENDIF\n".to_string(), 1);
        state.did_open(uri("file:///b.s"), "IF (a=b)\nENDIF\n".to_string(), 1);
        let mut uris: Vec<Uri> = state.open_uris().cloned().collect();
        uris.sort();
        let mut expected = vec![uri("file:///a.s"), uri("file:///b.s")];
        expected.sort();
        assert_eq!(uris, expected);
    }
}
