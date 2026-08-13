# Phase 1 Data Model: Live Diagnostic Updates on Config File Edits

No new persistent data of any kind (spec.md FR-009, Key Entities: "Watched
Configuration Change... not a persistent record — exists only as a trigger for
immediate re-evaluation, then discarded"). This feature adds one new accessor to
existing state and otherwise only changes *control flow* — when existing functions
get called, never what they compute.

## `ServerState` (existing type, `document_store.rs`) — one new accessor

| Member | Change |
|---|---|
| `open_uris(&self) -> impl Iterator<Item = &Uri>` | **New.** Returns every currently-open document's URI. Read-only; no new stored field — `documents: HashMap<Uri, OpenDocument>` already holds everything needed, just wasn't previously enumerable from outside the module. |

## Registration state (transient, not stored in `ServerState`)

The watcher registration itself is not tracked as state anywhere — it's a one-time
action taken during `run()`'s startup sequence (send the request, or don't, based
on the capability check) and never referenced again. There is no "is the watcher
currently registered" flag anywhere; if the client's response to the registration
request is an error, that's logged (research.md §1) but doesn't change any stored
state — the server simply continues operating with the same graceful-degradation
behavior as a client that never supported this at all.

## Control-flow change (the actual "model" of this feature)

```
initialize
   │
   ├─▶ parse InitializeParams once (research.md §3)
   │        │
   │        ├─▶ workspace_root  (unchanged from 012)
   │        │
   │        └─▶ workspace.did_change_watched_files.dynamic_registration == Some(true)?
   │                 │yes                              │no
   │                 ▼                                  ▼
   │        send client/registerCapability      do nothing (FR-004)
   │        for **/drut.toml (research.md §4)
   │
   ▼
main message loop (unchanged shape, two new-but-small branches)
   │
   ├─▶ Message::Response(_)  →  log on error, else no-op (research.md §1)
   │
   └─▶ Message::Notification(note)
            │
            ├─▶ DidOpen/DidChange/DidClose  →  unchanged from today
            │
            └─▶ DidChangeWatchedFiles (NEW)
                     │
                     └─▶ for uri in state.open_uris() { diagnostics::publish(uri) }
                              (re-resolves drut-config fresh per document, per
                              012's own already-existing, unmodified logic)
```

Every box below "main message loop" that existed before this feature is completely
unmodified in its own internal logic — only the *set of things that can trigger*
`diagnostics::publish` grows, from "this one document changed" to also include
"some `drut.toml` somewhere changed, so re-check every open document."
