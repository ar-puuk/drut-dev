# Phase 0 Research: Live Diagnostic Updates on Config File Edits

All findings below are measured against the real, current codebase on this branch
(`crates/drut-lsp/src/lib.rs`, `document_store.rs`, `diagnostics.rs`) — which
already includes `012-toml-configuration`'s work, since `013` branches directly from
`012`'s tip — plus the vendored `lsp-server 0.10.0` and `lsp-types 0.97.0` crate
source. Not estimated.

## §1. `run()`'s main loop discards `Message::Response` today — this is the first time that assumption breaks

**Finding**: `lib.rs::run`'s main loop has:

```rust
Message::Response(_) => {
    // This server never sends requests of its own, so it never
    // expects a response back — nothing to do.
}
```

This is true today and has been true since `003`. Sending a `client/
registerCapability` request is the **first** request `drut-lsp` has ever
initiated — the comment's own premise stops holding the moment this ships.

**Decision**: The `Message::Response(_)` arm gets a minimal, non-panicking handler —
since the only request this server will ever send back-references a known, fixed ID
and the result type is `()` (per `RegisterCapability::Result = ()`, confirmed in
`lsp-types`' own source), there is nothing meaningful to *do* with a successful
response beyond confirming it wasn't an error; log a `window/logMessage` on error
(consistent with `010`/`011`'s "surface visibly, don't be silent" precedent), stay
silent on success (matches `initialize`'s own handshake, which also doesn't narrate
success). No response-to-request correlation table is needed for a server that ever
only sends exactly one kind of request with one fixed, known ID.

**FR-010 ("never blocks on this response"), traced structurally, not assumed**:
`run()`'s main loop is a single `for msg in &connection.receiver` with no
per-message-type blocking wait of any kind — it processes whatever message the
channel yields next, unconditionally, regardless of type. There is no code path,
before or after this feature, where the loop pauses to wait specifically for one
expected response before continuing. This means the three failure modes FR-010
names (response never arrives; arrives indicating failure; arrives late) are
*structurally* incapable of blocking the loop — not because of a timeout, retry
limit, or other defensive mechanism added for this feature, but because the
architecture never blocks on any single message in the first place. This was
confirmed by reading `run()`'s actual current code before writing FR-010, per the
owner's explicit "confirm before deciding" standard — not inferred from general
async/message-loop intuition.

**Alternatives considered**: A timeout-and-retry mechanism for the registration
request was considered and rejected as unnecessary — it would be solving a problem
(the loop blocking) that doesn't exist given the architecture's actual shape, and
would add real complexity (tracking elapsed time, a retry counter, a give-up
threshold) for a scenario that already degrades gracefully to `US2`'s own already-
specified behavior with zero extra code.

## §2. No static-capability alternative exists — dynamic registration is mandatory, confirmed directly

**Finding**: `lsp-types`' own doc comment on `DidChangeWatchedFilesClientCapabilities`
states directly: *"the current protocol doesn't support static configuration for
file changes from the server side."* Unlike `folding_range_provider` or
`document_formatting_provider` (both plain booleans in `ServerCapabilities`), there
is no `ServerCapabilities` field for "I want to watch files" at all — the only path
is: check `InitializeParams.capabilities.workspace.did_change_watched_files.
dynamic_registration`, and if `Some(true)`, send `client/registerCapability`.

**Decision**: Gate registration on this exact field. `None`/`Some(false)` (client
doesn't support or doesn't advertise the capability) means: send nothing, register
nothing — spec.md FR-004's "must not attempt to activate" is satisfied by simply
never constructing the request, not by sending-and-catching-a-rejection.

## §3. Reusing, not duplicating, `012`'s own `InitializeParams` parsing

**Finding**: `012` already added `workspace_root_from_initialize_params(params:
serde_json::Value) -> Option<PathBuf>` to `lib.rs`, which parses the raw
`initialize` response value into `lsp_types::InitializeParams` once. Parsing it a
second time for the capability check would be pure duplication.

**Decision**: Restructure so `run()` parses `InitializeParams` once into a local
binding, then both (a) extracts `root_uri`/`workspace_folders` for `workspace_root`
(unchanged from `012`) and (b) reads
`capabilities.workspace.as_ref().and_then(|w| w.did_change_watched_files.as_ref())
.and_then(|d| d.dynamic_registration).unwrap_or(false)` for the registration gate —
one parse, two consumers, matching this crate's existing "translate once, reuse"
pattern (`position.rs`'s own stated rationale for centralizing translation).

## §4. Registration payload — verified directly against `lsp-types`' own struct shapes

**Decision**:

```rust
let registration = lsp_types::Registration {
    id: "drut-toml-watcher".to_string(),
    method: <lsp_types::notification::DidChangeWatchedFiles as lsp_types::notification::Notification>::METHOD.to_string(),
    register_options: Some(serde_json::to_value(lsp_types::DidChangeWatchedFilesRegistrationOptions {
        watchers: vec![lsp_types::FileSystemWatcher {
            glob_pattern: lsp_types::GlobPattern::String("**/drut.toml".to_string()),
            kind: None, // defaults to Create | Change | Delete per lsp-types' own doc comment
        }],
    }).unwrap()),
};
```

sent as a `lsp_server::Request::new(RequestId::from("drut-toml-watcher".to_string()),
<RegisterCapability as Request>::METHOD.to_string(), RegistrationParams {
registrations: vec![registration] })` via `connection.sender.send(Message::
Request(...))`. Every type referenced here (`Registration`, `RegistrationParams`,
`DidChangeWatchedFilesRegistrationOptions`, `FileSystemWatcher`, `GlobPattern`,
`RegisterCapability`, `DidChangeWatchedFiles`) confirmed present in the already-
vendored `lsp-types 0.97.0` source — no dependency version bump.

**Alternatives considered**: Narrowing the glob to each open document's own
resolved-config ancestry was considered and rejected per spec.md's own Assumptions
(FR-007) — `**/drut.toml` (workspace-wide) is the deliberate choice, not an
oversight.

## §5. Notification handling and the new `ServerState` accessor

**Decision**: `handle_notification` gains a new arm alongside the existing three,
extracting `lsp_types::DidChangeWatchedFilesParams` for method
`<DidChangeWatchedFiles as Notification>::METHOD`. On receipt (regardless of the
specific `FileChangeType` — Create/Change/Delete are all treated identically, per
spec.md's Edge Cases: any of the three could change which config applies), iterate
every currently-open document and call `diagnostics::publish` for each.

This requires a new accessor on `ServerState` — it currently exposes no way to
enumerate open documents, only `get(uri)` for one at a time:

```rust
pub fn open_uris(&self) -> impl Iterator<Item = &Uri> {
    self.documents.keys()
}
```

`diagnostics::publish` itself needs **zero changes** — it already re-resolves
`drut-config` fresh internally (confirmed in `012`'s own code, no caching); this
feature only changes *how often* and *for how many documents* it gets called, never
its own logic. Confirms spec.md's own framing: "this feature changes only how
promptly an already-correct mechanism is triggered."

## §6. FR-008 (no visible change for an unaffected document) falls out for free

**Finding**: `diagnostics::publish` already sends a full `PublishDiagnosticsParams`
on every call, and every mainstream LSP client (including VS Code) treats a
republish with an identical diagnostics list as a silent no-op — no flicker, no
visible event, confirmed by extension of `010`'s and `012`'s own established
"republish is safe and idempotent when nothing actually changed" pattern (both
already republish deterministically from live state on every `didChange`, with no
reported flicker issue). No new de-duplication logic is needed to satisfy FR-008;
re-publishing unconditionally for every open document already satisfies it as a
structural consequence of how `publish()` already works, not because of any new
guard added here.

## §7. Manual verification note

VS Code's own `vscode-languageclient` supports dynamic `workspace/
didChangeWatchedFiles` registration out of the box — no extension-side code change
is needed in `editors/vscode/` for this feature to work end-to-end in the primary
target editor, confirmed by this being a standard, widely-implemented LSP client
capability (not something specific to a custom client configuration this project
would need to opt into separately).
