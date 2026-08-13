# Implementation Plan: Live Diagnostic Updates on Config File Edits

**Branch**: `013-lsp-config-file-watch` | **Date**: 2026-08-13 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/013-lsp-config-file-watch/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Register a `workspace/didChangeWatchedFiles` watcher for `**/drut.toml`, gated on
the client actually advertising `workspace.didChangeWatchedFiles.dynamicRegistration`
support (checked from `InitializeParams`, already partially parsed for
`012-toml-configuration`'s own workspace-root capture). On a matching file event,
re-run `diagnostics::publish` for every currently-open document. This is the first
request `drut-lsp` has ever sent to a client (dynamic capability registration via
`client/registerCapability`) and the first workspace-scoped (not per-document)
notification it has ever handled — both real, new, but narrow additions on top of
`012`'s already-built `drut-config`/`ServerState` infrastructure.

## Technical Context

**Language/Version**: Rust 2021 edition (matches every other crate in this workspace).

**Primary Dependencies**: `lsp-types` 0.97.0 and `lsp-server` 0.10.0 (already
`drut-lsp` dependencies) — confirmed directly against their vendored source that
every type needed (`Registration`, `RegistrationParams`,
`DidChangeWatchedFilesRegistrationOptions`, `FileSystemWatcher`, `GlobPattern`,
`DidChangeWatchedFilesClientCapabilities`, `request::RegisterCapability`,
`notification::DidChangeWatchedFiles`) already exists; no dependency change.

**Storage**: N/A — no new persistent state (spec.md FR-009); reuses `012`'s existing
`ServerState`/`drut-config` resolution, unmodified.

**Testing**: `cargo test -p drut-lsp` — new unit tests for the capability-gating
logic and the new `ServerState::open_uris()` accessor, plus a real protocol test
over `Connection::memory()` reproducing spec.md US1 Acceptance Scenario 1's exact
sequence (the owner's own repro) end to end.

**Target Platform**: Cross-platform LSP server; any LSP-capable editor client, VS
Code (which does support dynamic `didChangeWatchedFiles` registration) used for
manual verification per constitution Principle VI.

**Project Type**: Adapter-only change within the existing workspace — `drut-lsp`
only. No change to `voyager-core`, `drut-config`, `drut-cli`, or `drut-mcp`.

**Performance Goals**: Re-publishing diagnostics for every open document on a
`drut.toml` change must stay imperceptibly fast at drut's real target scale (spec.md
SC-004, Assumptions) — matches the existing per-document re-publish cost already
paid on every ordinary `didChange`, just fanned out across open documents instead of
one.

**Constraints**: This is the first server-initiated request `drut-lsp` has ever
sent — `run()`'s main loop currently discards `Message::Response` unconditionally
with a comment stating exactly that assumption no longer holds once this ships
(research.md §1). Registration MUST be skipped entirely, not attempted-and-ignored,
for a client that doesn't advertise support (spec.md FR-004) — confirmed there is no
static-capability alternative to dynamic registration for this LSP method at all
(research.md §2).

**Scale/Scope**: Small — one new capability-negotiation step at startup, one new
notification handler, one new `ServerState` accessor. Comparable in size to `011`'s
folding-capability addition, smaller than `012`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Single Source of Truth)**: PASS — no grammar/parsing/lint-rule logic
  anywhere in this feature; it only changes when `drut-lsp` re-invokes
  `diagnostics::publish` (already the single source of truth for diagnostics
  assembly) and when `drut-config`'s already-existing resolution is re-run
  (unmodified).
- **Principle II (No Verbatim Vendor Docs)**: N/A — no vendor-doc-derived content.
- **Principle III (Formatter Idempotence)**: N/A — this feature never touches
  formatting; `formatting.rs`/`range_formatting.rs` are unmodified (they already
  resolve config fresh per request, spec.md's own "not a caching bug" finding).
- **Principle IV (False Negatives Over False Positives)**: Applies by analogy —
  FR-008 (an unaffected document's re-evaluation must never produce a visible
  diagnostic change) exists specifically so this feature cannot introduce diagnostic
  flicker/noise on documents nothing actually changed for.
- **Principle V (Vertical Increments)**: PASS — one independently-shippable,
  independently-testable increment, matching this project's own explicit history
  (confirmed via git log research before this cycle: 002/003/007 each fixed a bug
  found during that phase's own verification before the phase was considered done,
  never merged with a known bug as tracked follow-up) — `013` exists precisely
  because `012`'s own manual verification isn't complete until this is fixed.
- **Principle VI (LSP-Standard Mechanisms)**: Directly load-bearing — this is the
  entire point: `workspace/didChangeWatchedFiles` + `client/registerCapability` are
  the LSP-standard mechanism for exactly this need; no editor-proprietary file-watch
  API is used.
- **Principle VII (Naming Honesty)**: PASS — no overclaiming; "detects config file
  changes" is exactly what this does.
- **Principle VIII (Public/Private Boundary)**: N/A.

No violations; Complexity Tracking table is not needed.

## Project Structure

### Documentation (this feature)

```text
specs/013-lsp-config-file-watch/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/
└── drut-lsp/
    └── src/
        ├── lib.rs               # register the watcher at initialize (gated on
        │                        #   client capability); handle its response;
        │                        #   handle DidChangeWatchedFiles notifications
        ├── document_store.rs    # + ServerState::open_uris() accessor
        └── diagnostics.rs       # unmodified — publish() already does exactly
                                  #   what's needed, just needs to be called more

    └── tests/
        └── protocol_smoke.rs    # + the exact US1 Acceptance Scenario 1 repro,
                                  #   over Connection::memory()
```

**Structure Decision**: Entirely within `drut-lsp` — no other crate changes at all,
confirmed by the bug's own root-cause analysis (formatting was already correct;
`drut-config`'s resolution was already fresh/uncached; only the diagnostics-push
trigger was missing). Matches this repo's existing structure exactly.

## Complexity Tracking

Not applicable — no Constitution Check violations (see above).
