---

description: "Task list for Live Diagnostic Updates on Config File Edits"
---

# Tasks: Live Diagnostic Updates on Config File Edits

**Input**: Design documents from `/specs/013-lsp-config-file-watch/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/,
quickstart.md (all present)

**Tests**: Included — matches this project's established discipline for every
prior LSP-protocol feature (`003`, `010`, `011`, `012`): real protocol tests over
`Connection::memory()`, not just unit-level assertions, for anything that changes
what the server sends or reacts to over the wire.

**Organization**: Two user stories, matching spec.md exactly — US1 (P1, the bug
fix itself: diagnostics update live on a `drut.toml` edit), US2 (P2, graceful
degradation — both for a client that never supported this, and for a client that
does but never confirms activation, FR-010). A Foundational phase carries the
shared mechanism (capability check, request construction, response handling, the
new `ServerState` accessor) both stories build on.

**Everything in this file's scope was measured against the real, current codebase
on this branch — which already includes `012`'s work, since `013` branches
directly from `012`'s tip — not estimated (research.md §1-§7)**:

- `run()`'s main loop is a single, unified `for msg in &connection.receiver` with
  no per-message-type blocking wait anywhere — confirmed by reading the actual
  code before writing FR-010, not assumed. This is what makes FR-010 (never blocks
  on the registration response) a structural guarantee rather than a new
  timeout/retry mechanism that needed to be built.
- No static-capability alternative to dynamic registration exists for
  `workspace/didChangeWatchedFiles` at all — confirmed directly against
  `lsp-types`' own doc comment. Registration is skipped entirely for an
  unsupporting client (FR-004), never attempted-and-caught.
- `012`'s `workspace_root_from_initialize_params` already parses `InitializeParams`
  once; this feature reuses that same parse for the capability check rather than
  parsing twice (research.md §3).
- `diagnostics::publish` needs **zero logic changes** — it already re-resolves
  `drut-config` fresh internally with no caching (confirmed in `012`'s own code).
  This feature only changes how often, and for how many documents, it gets called.
- **Post-`/speckit-tasks`-request remediation**: FR-010/SC-005/US2 Acceptance
  Scenario 3 were added to spec.md (and research.md §1, contracts.md) after the
  owner flagged, before task generation, that "what happens if the registration
  response never arrives or arrives malformed" wasn't covered by anything decided
  so far. T015 below is that scenario's own dedicated, required test — not folded
  into general error-handling coverage.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete
  sibling task)
- **[Story]**: US1/US2 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/drut-lsp/src/lib.rs` — capability check, request construction/sending,
  response handling, the new notification handler.
- `crates/drut-lsp/src/document_store.rs` — the new `open_uris()` accessor.
- `crates/drut-lsp/tests/protocol_smoke.rs` — every protocol-level test in this
  file (all of them, per this project's `Connection::memory()` convention).

---

## Phase 1: Setup

- [x] T001 Confirm baseline: `cargo build --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings` both clean, on this
      branch (which already includes `012`'s committed work) before any new change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The shared mechanism — capability negotiation, request construction,
response handling, and document enumeration — both user stories depend on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T002 Add `pub fn open_uris(&self) -> impl Iterator<Item = &Uri>` to
      `crates/drut-lsp/src/document_store.rs`'s `ServerState` (data-model.md):
      returns `self.documents.keys()` — no new stored field, purely a read-only
      view over what's already there. Never panics; returns an empty iterator for
      a session with no open documents.
- [x] T003 In `crates/drut-lsp/src/lib.rs`'s `run()`: restructure so
      `InitializeParams` is parsed once into a local binding (reusing the exact
      parse `012`'s `workspace_root_from_initialize_params` already performs, not
      a second independent parse — research.md §3), and compute
      `did_change_watched_files_supported: bool` from
      `params.capabilities.workspace.as_ref().and_then(|w|
      w.did_change_watched_files.as_ref()).and_then(|d|
      d.dynamic_registration).unwrap_or(false)`.
- [x] T004 In `run()`: if `did_change_watched_files_supported`, construct and send
      exactly one `client/registerCapability` request (contracts/
      config-watch-api.md, research.md §4) — ID `"drut-toml-watcher"`, registering
      `workspace/didChangeWatchedFiles` for glob `**/drut.toml` (default `kind`,
      i.e. Create | Change | Delete) — via
      `connection.sender.send(Message::Request(...))`. If not supported, send
      nothing at all (FR-004 — never attempted, not attempted-and-ignored). Either
      way, `run()` proceeds immediately into the main loop with no wait. Depends
      on T003.
- [x] T005 In `run()`'s main loop: replace the current no-op `Message::
      Response(_)` arm with a minimal handler — on `Err(...)`, log via
      `window/logMessage` (`MessageType::WARNING`, matching `010`/`011`/`012`'s
      "surface visibly, never silently" precedent); on `Ok(...)`, no action.
      **Add no blocking wait, timeout, or retry logic of any kind** — the loop's
      existing unified-`for`-over-every-message-type shape already guarantees
      FR-010 structurally (research.md §1); this task must not compromise that by
      introducing per-message special-casing that could pause the loop. Depends
      on T004.
- [x] T006 [P] Add unit tests to `crates/drut-lsp/src/document_store.rs`'s own
      test module for `open_uris()`: zero documents open returns an empty
      iterator; multiple `did_open` calls followed by `open_uris()` returns every
      one of their URIs (order not significant). Depends on T002.
- [x] T007 [P] Add a protocol test to `crates/drut-lsp/tests/protocol_smoke.rs`:
      an `initialize` handshake advertising
      `workspace.didChangeWatchedFiles.dynamicRegistration: true` produces exactly
      one outgoing `client/registerCapability` request, asserted directly against
      its actual content — method `"workspace/didChangeWatchedFiles"` and a
      `**/drut.toml` glob pattern inside `registerOptions.watchers` — not merely
      "a request of some kind was sent." Depends on T004.

**Checkpoint**: The shared mechanism exists, is correctly gated, sends the correct
request when supported, and can never block the server on its response.

---

## Phase 3: User Story 1 - A configuration fix or mistake shows up immediately, without reopening anything (Priority: P1) 🎯 MVP

**Goal**: Editing `drut.toml` directly refreshes diagnostics for every open
document it could affect, without closing or reopening any of them.

**Independent Test**: With a script open showing a config-related diagnostic, edit
the governing `drut.toml` to a different value without touching the script file
itself; confirm the diagnostic updates to match.

### Implementation for User Story 1

- [x] T008 [US1] In `crates/drut-lsp/src/lib.rs`'s `handle_notification`: add a
      new extraction arm for `notification::DidChangeWatchedFiles`
      (`DidChangeWatchedFilesParams { changes: Vec<FileEvent> }`), chained after
      the existing three (`DidOpenTextDocument`/`DidChangeTextDocument`/
      `DidCloseTextDocument`) in the same `extract`/`MethodMismatch` style. On a
      successful extraction, for every URI in `state.open_uris()`, call
      `diagnostics::publish(connection, state, uri)` — unconditionally, regardless
      of each `FileEvent.typ` (Created/Changed/Deleted all treated identically,
      per spec.md's Edge Cases) and regardless of which specific path changed
      (the deliberate broad-scope choice, FR-007). `diagnostics::publish` itself
      is not modified — it already re-resolves `drut-config` fresh internally.
      Depends on T002, T005.

### Tests for User Story 1

- [x] T009 [US1] Add the **primary, required** regression test to
      `crates/drut-lsp/tests/protocol_smoke.rs`, reproducing spec.md US1
      Acceptance Scenario 1 exactly — the owner's own reported sequence, not a
      simplified variant: a real on-disk `drut.toml` (temp directory) with an
      invalid `casing` value; a script file in the same directory, opened via
      `didOpen`, confirmed to carry a diagnostic naming that specific bad value;
      the `drut.toml` file edited on disk to a *different* invalid value; a
      simulated `workspace/didChangeWatchedFiles` notification sent for that
      path; confirm the republished diagnostic names the **new** bad value —
      **without any `didChange`, `didClose`, or `didOpen` sent for the script
      file at any point**. Depends on T008.
- [x] T010 [P] [US1] Add a test to `protocol_smoke.rs`: two script files open at
      once, both governed by the same `drut.toml`; one simulated
      `didChangeWatchedFiles` notification; confirm **both** documents'
      diagnostics refresh correctly (US1 Acceptance Scenario 2 — not only
      whichever document was opened or focused last). Depends on T008.
- [x] T011 [P] [US1] Add tests to `protocol_smoke.rs`: a script file open with a
      **valid** `drut.toml` (no diagnostic) — editing `drut.toml` to introduce a
      mistake produces a new diagnostic with no action on the script file (US1
      Acceptance Scenario 3); a script file open with an existing config
      diagnostic — fixing the underlying `drut.toml` mistake clears the
      diagnostic with no action on the script file (US1 Acceptance Scenario 4).
      Depends on T008.
- [x] T012 [P] [US1] Add a test to `protocol_smoke.rs` proving FR-008: a
      `didChangeWatchedFiles` event for a `drut.toml` edit that does **not**
      change a given open document's effective resolved configuration (e.g. a
      second, unrelated `drut.toml` elsewhere in the workspace changes, or a
      change to `drut.toml` that doesn't affect the field currently in error)
      produces **no visible diagnostic change** for that document — same
      diagnostics list before and after, not a duplicate or a flicker. Depends
      on T008.

**Checkpoint**: The bug is fixed and proven via its own exact repro sequence, plus
the multiple-document and appear/disappear variants named in spec.md.

---

## Phase 4: User Story 2 - Predictable, unbroken behavior on an editor that doesn't support this (Priority: P2)

**Goal**: An editor session that doesn't support (or doesn't confirm) this
capability is never worse off than before this feature existed — no crash, no
hang, no behavior change beyond the known, accepted detection-delay limitation.

**Independent Test**: Using a session that doesn't indicate support, confirm the
session starts normally and that a `drut.toml`-only edit doesn't update
diagnostics until the affected document is itself reopened or edited.

### Tests for User Story 2

- [x] T013 [US2] Add a protocol test to `protocol_smoke.rs`: an `initialize`
      handshake with `workspace.didChangeWatchedFiles.dynamicRegistration`
      **absent or `false`** produces **zero** outgoing requests from the server
      before the first client-issued request/notification — confirmed by
      asserting no `client/registerCapability` request (or any other) arrives on
      the client's receiver before the test itself sends something (US2
      Acceptance Scenario 1). Depends on T004.
- [x] T014 [US2] Add a protocol test to `protocol_smoke.rs`: with
      `dynamicRegistration` unsupported (so no watcher is ever registered and no
      `didChangeWatchedFiles` notification would ever legitimately arrive from a
      well-behaved client), a script file's diagnostics remain unchanged after
      its governing `drut.toml` is edited on disk with no notification sent —
      diagnostics only update once the script file itself receives a `didChange`
      or is reopened, matching pre-`013` behavior exactly (US2 Acceptance
      Scenario 2). Depends on T004.
- [x] T015 [US2] **FR-010's own dedicated test** — add a protocol test to
      `protocol_smoke.rs`: an `initialize` handshake **with**
      `dynamicRegistration: true` (so the registration request genuinely is
      sent, confirmed via T007's own assertion pattern), but the test harness
      never sends any response to it at all; immediately follow with an ordinary,
      unrelated request (e.g. `textDocument/hover` or `textDocument/formatting`
      against an open document) and confirm **that request still receives its
      own correct response**, promptly, with no hang or delay attributable to the
      never-answered registration request (US2 Acceptance Scenario 3, SC-005).
      This is the specific failure mode flagged as uncovered before task
      generation — do not treat it as satisfied by T007 or T013 alone, both of
      which test *whether* a request is sent, not what happens when a sent
      request's response never comes. Depends on T005.
- [x] T016 [P] [US2] Add a protocol test to `protocol_smoke.rs`: the test harness
      responds to the `client/registerCapability` request with an error result;
      confirm the server logs a `window/logMessage` notification
      (`MessageType::WARNING`) and continues operating normally afterward (an
      immediately-following unrelated request still succeeds) — no crash, no
      hang. Depends on T005.

**Checkpoint**: Every failure mode this feature could introduce — unsupported
client, registration failure, registration silence — degrades to exactly US2's
specified limitation and nothing worse, each proven by its own test.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Whole-workspace re-proof and quickstart execution, once both stories
are done.

- [x] T017 `cargo test --release --workspace` and `cargo clippy --workspace
      --all-targets -- -D warnings`, both clean.
- [x] T018 Run quickstart.md end-to-end (all 5 steps, including the manual VS
      Code step); confirm each step's expected outcome individually before
      reporting the feature done. (Steps 1, 2, 3, 5 run and confirmed; step 4,
      the manual VS Code check, is left for the owner per this project's
      established pattern for every prior LSP-facing feature.)

**Checkpoint**: Feature-complete against spec.md; every FR (including FR-010) and
every acceptance scenario in both user stories independently proven.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup. Blocks both user stories — the
  capability check, request/response mechanism, and `open_uris()` accessor must
  all exist and be individually proven (T002-T007) before either story's own work
  begins.
- **User Story 1 (Phase 3)**: Depends on Foundational in full.
- **User Story 2 (Phase 4)**: Depends on Foundational in full (specifically T004's
  conditional-send logic and T005's response handling — US2 has no new
  implementation of its own, only dedicated proof that Foundational's behavior
  holds under every failure mode).
- **Polish (Phase 5)**: Depends on both stories being complete.

### Parallel Opportunities

- T006 and T007 can proceed in parallel once their respective dependencies (T002,
  T004) land — different files.
- T010, T011, T012 can proceed in parallel once T008 lands — same file
  (`protocol_smoke.rs`), coordinate insertion order rather than true concurrent
  editing.
- T016 can proceed in parallel with T013/T014/T015 once T005 lands — same
  caveat.

---

## Implementation Strategy

### Single Pass (small feature, one shared mechanism, two thin story layers)

1. Setup → baseline confirmed clean.
2. Foundational → the entire real implementation surface of this feature: capability
   negotiation, request construction, response handling (including FR-010's
   structural guarantee), and the one new `ServerState` accessor.
3. User Story 1 → the one new notification handler (a few lines, since
   `diagnostics::publish` needs no changes) plus its own regression proof —
   including the exact bug repro sequence as the primary, required test.
4. User Story 2 → no new implementation, only dedicated proof that Foundational's
   gating and non-blocking behavior hold under every failure mode named in
   spec.md, including the specific "response never arrives" case flagged before
   task generation.
5. Polish → whole-workspace re-proof and quickstart execution (manual VS Code
   step left for the owner, matching every prior LSP-facing feature's pattern).

---

## Notes

- T009 (the exact bug repro) and T015 (FR-010's never-blocks proof) are this
  feature's two most important tests — one proves the bug is actually fixed in
  its originally-reported form, the other proves the fix's own new failure mode
  (an unconfirmed request) can't make things worse than before. Neither should be
  treated as redundant with the other's neighboring coverage.
- Commit after each task or logical group.
