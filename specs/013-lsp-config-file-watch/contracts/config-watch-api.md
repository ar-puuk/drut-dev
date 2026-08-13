# Contract: Config File Watch

Covers `drut-lsp`'s startup registration sequence and its new notification handler.
No public API beyond `drut-lsp`'s own protocol surface — this is entirely internal
to the LSP server's own request/notification handling.

## Startup sequence (`lib.rs::run`)

1. `connection.initialize(caps)` — unchanged call, but its `Ok` value is parsed
   into `lsp_types::InitializeParams` **once** (already true as of `012`;
   extended here to also drive the capability check, research.md §3).
2. If `params.capabilities.workspace.as_ref().and_then(|w|
   w.did_change_watched_files.as_ref()).and_then(|d| d.dynamic_registration) ==
   Some(true)`: send exactly one `client/registerCapability` request, ID
   `"drut-toml-watcher"`, registering `workspace/didChangeWatchedFiles` for glob
   `**/drut.toml` (Create | Change | Delete — the default `kind` when omitted).
   Otherwise: send nothing (FR-004 — never attempted, not attempted-and-failed).
3. Continue into the main message loop exactly as before.

**Never blocks startup**: whether or not this step sends a request, `run()`
proceeds to the main loop immediately after — there is no synchronous wait for the
registration response before the server starts handling other messages.

## Main loop additions

- `Message::Response(_)`: if the response is `Err(...)`, log it via
  `window/logMessage` (`MessageType::WARNING` — informational, matching this
  project's "surface visibly, never silently" precedent, `010`/`011`/`012`); on
  `Ok(...)`, no action (nothing meaningful to do with a successful, empty-result
  registration ack). Never panics on an unexpected response shape.
- **FR-010 (never blocks on this response)**: no code path anywhere waits
  specifically for this response before doing anything else — `run()`'s loop
  processes every message generically, in arrival order, regardless of type
  (research.md §1). A response that never arrives simply means
  `Message::Response(_)` is never reached for that ID; every `Message::Request`/
  `Message::Notification` in the meantime and afterward is handled exactly as if
  registration had never been attempted. This is a structural property of the
  existing loop shape, not a new timeout/retry mechanism — none is added.
- `Message::Notification` gains a new extraction attempt for
  `notification::DidChangeWatchedFiles` (`DidChangeWatchedFilesParams { changes:
  Vec<FileEvent> }`), tried after the existing three (`DidOpenTextDocument`/
  `DidChangeTextDocument`/`DidCloseTextDocument`), same `extract`/`MethodMismatch`
  chaining style already used for the existing three. On a successful extraction,
  for every URI in `state.open_uris()`, call `diagnostics::publish(connection,
  state, uri)` — unconditionally, regardless of `FileEvent.typ`
  (Created/Changed/Deleted are treated identically) and regardless of the specific
  changed path (any `**/drut.toml` match triggers a full re-check of every open
  document — the deliberate broad-scope choice, spec.md Assumptions/FR-007).

## `ServerState` (new accessor)

```rust
pub fn open_uris(&self) -> impl Iterator<Item = &Uri>
```

Never panics; returns an empty iterator for a session with no open documents (a
normal, common state — e.g. immediately after `initialize`, before any `didOpen`).

## Non-goals (explicitly out of contract)

- No narrowing of the watch glob to per-document resolved-config paths (spec.md
  Assumptions — deliberate).
- No de-duplication logic for "this document's diagnostics didn't actually
  change" — republishing is already safe/idempotent by construction (research.md
  §6); no new guard is added.
- No `workspace/unregisterCapability` call — the watcher, once registered, stays
  registered for the life of the session; there's no scenario in this feature
  where deregistering mid-session would be correct.
- No change to `drut-config`'s own resolution logic, `formatting.rs`,
  `range_formatting.rs`, or any crate other than `drut-lsp`.
