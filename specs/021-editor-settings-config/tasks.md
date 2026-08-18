---

description: "Task list for Editor-Settings Exposure for [format] Config Fields"
---

# Tasks: Editor-Settings Exposure for `[format]` Config Fields

**Input**: Design documents from `/specs/021-editor-settings-config/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md
(all present)

**Tests**: Included — a `drut-config` public-API signature change touching four crates, plus a
genuinely new `drut-lsp` server capability (a second-ever server-initiated request), needs real
coverage at every layer.

**Organization**: Per plan.md's own framing, spec.md's User Story 1 ("personal preference, no
`drut.toml`") and User Story 2 ("`drut.toml` still wins") are two acceptance angles on one
capability, not separable increments — both are proven together in one story phase. Foundational
carries the entire `drut-config` precedence-tier addition (the new parameter, the resolution
logic, every existing call site's mechanical fix) — this alone already proves the *precedence
logic itself* is correct, fully testable without any LSP wiring at all. The User Story phase is
the LSP-side delivery mechanism (pull, cache, live-refresh) that gets a real value into that
already-proven logic; Polish is the VS Code declaration plus final re-proof.

**Everything in this file's scope was measured against the real, current codebase during
planning (research.md), not estimated**:

- `drut_config::resolve_casing_and_indent`'s existing per-field chains end in
  `.unwrap_or_default()` with exactly one prior fallback source (`drut.toml`) today — confirmed
  by reading the actual chains, not assumed. Adding a fourth-parameter fallback is a small,
  mechanical extension of a pattern already there twice (explicit, then config).
- `drut-lsp/src/lib.rs::register_drut_toml_watcher` is, per its own doc comment, "the one and
  only request this server ever initiates" — fire-and-forget, never blocking the main loop. This
  feature's new request follows the identical shape; no new blocking-wait machinery is built.
- `workspace/didChangeConfiguration`'s payload is not a reliable data source in the modern LSP
  client convention (confirmed via `vscode-languageclient`'s own bundled source) — this feature
  never reads it, treating the notification purely as a re-pull trigger.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete sibling task)
- **[Story]**: US1 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/drut-config/src/lib.rs` — the new `client_defaults` parameter and its resolution logic.
- `crates/drut-config/tests/` — new precedence coverage.
- `crates/drut-cli/src/format_cmd.rs`, `crates/drut-mcp/src/format.rs` — mechanical call-site
  fixes only (always pass `ExplicitFormatOverride::default()`).
- `crates/drut-lsp/src/document_store.rs` — `ServerState`'s new cache field.
- `crates/drut-lsp/src/lib.rs` — the new request/response/notification plumbing.
- `crates/drut-lsp/src/formatting.rs`, `src/range_formatting.rs` — the real call-site wiring.
- `crates/drut-lsp/tests/protocol_smoke.rs` — end-to-end round-trip coverage.
- `editors/vscode/package.json` — the 10 new `drut.format.*` settings declarations.
- `ROADMAP.md` — item 15 marked done (Polish).

---

## Phase 1: Setup

- [X] T001 Confirm baseline: `cargo build --workspace` and `cargo clippy --workspace
      --all-targets -- -D warnings` both clean, on this branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The entire `drut-config` precedence-tier addition — the new parameter threaded
through every field's resolution, and every existing caller's mechanical fix. Proves the
precedence logic itself is correct (spec.md US1 AS1/AS3, US2 AS1/AS2) before any LSP delivery
mechanism is built on top of it.

- [X] T002 In `crates/drut-config/src/lib.rs`: add `client_defaults: ExplicitFormatOverride` as
      `resolve_format_options`'s 4th parameter (data-model.md §1, contracts/
      editor-settings-config.md). In `resolve_casing_and_indent`: add one more
      `.or(client_defaults.field)` to each of the 6 simple per-field chains (`data_references`,
      `top_level_indent`, `operator_spacing`, `blank_lines`), immediately before
      `.unwrap_or_default()`. For `control_words`/`pair_keywords` specifically, add **two** more
      steps each — `.or(client_defaults.control_words_casing).or(client_defaults.casing)` and
      `.or(client_defaults.pair_keywords_casing).or(client_defaults.casing)` respectively —
      mirroring the legacy-then-granular fallback both the `explicit` and `config` tiers already
      apply for these two fields (research.md §1's precision correction, caught during checklist
      review — get this exactly right, not "one more `.or()` for every field" uniformly). In
      `resolve_indent_width`/`resolve_blank_line_cap`: add `client_defaults`'s corresponding
      field as a third fallback argument, validated identically to how a `drut.toml` value
      already is (out-of-range degrades to the built-in default with a non-blocking notice).
      Depends on nothing (existing function, existing helpers).
- [X] T003 [P] In `crates/drut-cli/src/format_cmd.rs`: update the `resolve_format_options` call
      site to pass `ExplicitFormatOverride::default()` as the new 4th argument — CLI behavior is
      completely unaffected (spec.md FR-007). Depends on T002.
- [X] T004 [P] In `crates/drut-mcp/src/format.rs`: same fix as T003, MCP's own call site. Depends
      on T002.
- [X] T005 [P] In `crates/drut-lsp/src/formatting.rs` and `src/range_formatting.rs`: same fix as
      T003, both LSP call sites — `ExplicitFormatOverride::default()` as a **temporary**
      placeholder, explicitly noted in a comment as swapped for the real cached value in T009 once
      the pull/cache mechanism exists. Depends on T002.
- [X] T006 [P] Add tests to `crates/drut-config/tests/` (new file or an existing precedence test
      file): a `client_defaults` value applies when neither `explicit` nor `drut.toml` set a
      field (US1 AS1/AS3 shape); a `drut.toml` value wins over a conflicting `client_defaults`
      value for the same field (US2 AS1); a `client_defaults` value wins for a *different* field
      `drut.toml` doesn't set, in the same resolution call (US2 AS2); an out-of-range
      `client_defaults` numeric value falls back to the built-in default with a non-blocking
      notice; **a `client_defaults.casing` (legacy, no granular field set) value resolves both
      `control_words` and `pair_keywords` correctly** (the CHK001/CHK002 regression case —
      confirms the two-step fallback from T002, not just the single-step shape every other field
      gets); every existing precedence test updated to pass `ExplicitFormatOverride::default()`
      as the new 4th argument and still passes unmodified. Depends on T002.

**Checkpoint**: The precedence logic itself is proven correct in isolation — `client_defaults`
resolves exactly where spec.md says it should, `drut.toml` still wins where it should, CLI/MCP
are untouched. `cargo build --workspace` succeeds (drut-lsp's two call sites compile via T005's
temporary placeholder, not yet reading anything real).

---

## Phase 3: User Story 1 - Editor-settings deliver a real value into the proven precedence logic (Priority: P1)

**Goal**: `drut-lsp` pulls the client's `drut.format` settings via the standard LSP
`workspace/configuration` mechanism, caches them, and feeds them into the already-proven
precedence logic from Foundational — live, refreshing on `workspace/didChangeConfiguration`,
gracefully absent for a client that doesn't support the capability at all.

**Independent Test**: With a client setting configured for one field and no `drut.toml`
anywhere, format a document and confirm the client-configured value is applied; with a
conflicting `drut.toml` present, confirm `drut.toml` wins instead.

### Implementation for User Story 1

- [X] T007 [US1] In `crates/drut-lsp/src/document_store.rs`: add `client_format_defaults:
      drut_config::ExplicitFormatOverride` to `ServerState` (same shape as the existing
      `workspace_root` field), plus `set_client_format_defaults`/`client_format_defaults`
      get/set methods (data-model.md §2). Depends on T002 (needs the type in scope).
- [X] T008 [US1] In `crates/drut-lsp/src/lib.rs`: add `workspace_configuration_supported(params)
      -> bool` (same "advertised-capability-gated, confirmed against `lsp-types`' own capability
      doc comments" shape `did_change_watched_files_supported` already established — checks
      `capabilities.workspace.configuration`). Add `request_client_format_defaults(connection)`
      — sends a `workspace/configuration` request for section `"drut.format"`, fire-and-forget,
      same shape as the existing `register_drut_toml_watcher` (research.md §2, §4); called once
      at startup when `workspace_configuration_supported` is true. Extend `handle_response` with
      a match arm for this request's ID: parses the returned JSON object's known field names into
      an `ExplicitFormatOverride`, calling `ServerState::set_client_format_defaults` — an
      unparseable/missing field is simply left `None` for that field, never a hard failure. Extend
      `handle_notification` with a `workspace/didChangeConfiguration` arm that calls
      `request_client_format_defaults` again — the notification's own payload is never read
      (research.md §3). Depends on T007.
- [X] T009 [US1] In `crates/drut-lsp/src/formatting.rs` and `src/range_formatting.rs`: replace
      T005's temporary `ExplicitFormatOverride::default()` placeholder with
      `state.client_format_defaults()` — the real wiring. Depends on T007, T008.

### Tests for User Story 1

- [X] T010 [P] [US1] Add unit tests: `workspace_configuration_supported` for both a client that
      advertises the capability and one that doesn't (`crates/drut-lsp/src/lib.rs`'s own test
      module); `ServerState`'s cache get/set round-trips correctly, defaulting to `Default`
      before any pull (`crates/drut-lsp/src/document_store.rs`'s own test module); a malformed/
      partially-invalid pulled JSON object leaves only the affected field(s) `None`, not the
      whole cache. Depends on T007, T008.
- [X] T011 [US1] Add protocol-level round-trip tests to `crates/drut-lsp/tests/protocol_smoke.rs`
      (real `initialize`/`workspace/configuration`/`textDocument/formatting` round trip over
      `Connection::memory()`, matching this file's existing pattern): a client advertising
      `workspace.configuration` support, with a `drut.format` setting for one field per config
      category (casing, indentation, operator spacing, blank lines) and no `drut.toml`, formats a
      document reflecting each client-set value (US1 AS1); a `drut.toml` present in the same
      scenario, setting one of those same fields to a conflicting value, wins over the client
      setting for that field only (US2 AS1/AS2); a `workspace/didChangeConfiguration`
      notification after an initial pull triggers a re-pull, and the *next* format request
      against the already-open document reflects the refreshed value, no reopen needed (US1 AS2,
      SC-004); a client that never advertises `workspace.configuration` support never receives
      the request at all, and formatting behaves exactly as before this feature (FR-004, SC-005).
      Depends on T009.

**Checkpoint**: User Story 1 independently proven — client settings actually reach formatting
output through the standard LSP mechanism, `drut.toml` still wins where it sets a field, live
updates work without reopening, and a non-supporting client sees zero behavior change.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: The VS Code-visible declaration, `ROADMAP.md` update, and final re-proof, once the
capability itself is proven.

- [X] T012 [P] In `editors/vscode/package.json`: add all 10 fields to `contributes.configuration.
      properties` under `drut.format.*` (camelCase — data-model.md §3), matching each field's own
      accepted-value set (enum for casing/indent/spacing/blank-line-mode fields, integer with
      range for `indentWidth`/`topLevelBlankLineCap`/`nestedBlankLineCap`) — no `"default"` on
      any property, so an unset VS Code setting correctly means "not present" from the server's
      point of view.
- [X] T013 [P] In `ROADMAP.md`: mark item 15 done, dated, pointing at this feature's spec
      directory — same pattern every other completed `ROADMAP.md` item already follows.
- [X] T014 `cargo test --release --workspace` and `cargo clippy --workspace --all-targets --
      -D warnings`, both clean.
- [X] T015 Full 161-file real-corpus revalidation via the CLI surface with no `drut.toml` and no
      client settings (the CLI always passes `ExplicitFormatOverride::default()` for the new
      parameter) — expected zero diagnostic/output change from before this feature (SC-003),
      reported as its own explicit result.
- [X] T016 Run `quickstart.md` end-to-end as written, including step 5's direct inspection of
      `package.json`'s declared configuration schema (SC-006), confirming every step's expected
      result holds against the actual shipped code.

**Checkpoint**: Feature-complete against spec.md; `ROADMAP.md` consistent with shipped code;
full workspace and full corpus re-proven clean; all 10 fields visible in VS Code's Settings UI.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS User Story 1.
- **User Story 1 (Phase 3)**: Depends on Foundational only.
- **Polish (Phase 4)**: T012/T013 are independent of the code phases; T014-T016 depend on User
  Story 1 being complete.

### Parallel Opportunities

- T003, T004, T005, T006 can all run in parallel once T002 lands (different files).
- T010 can run in parallel with T009's own implementation once T007/T008 land (different files:
  unit tests vs. the two handler call-site swaps).
- T012, T013 can run in parallel with each other and with T014-T016 not yet started.

---

## Parallel Example: Once Foundational (T002-T006) Lands

```bash
Task: "T007: ServerState gains client_format_defaults cache field"
Task: "T012: package.json gains 10 drut.format.* properties"
```

---

## Implementation Strategy

### MVP First (this feature IS the MVP — one capability, two acceptance angles)

1. Setup → baseline confirmed clean.
2. Foundational → the precedence logic itself proven correct, in isolation, before any LSP
   wiring exists.
3. User Story 1 → the real delivery mechanism, proven end-to-end via protocol-level tests.
4. **STOP and VALIDATE**: run T011 against both the no-`drut.toml` and `drut.toml`-present
   scenarios.

### Incremental Delivery

1. Foundational → precedence logic ready and independently tested.
2. User Story 1 → feature complete (there is no second increment for this feature).
3. Polish → VS Code declaration, `ROADMAP.md` update, full re-proof.

---

## Notes

- T005's placeholder-then-swap shape (fixed for real in T003/T004, left temporary for T009 to
  finish) is deliberate — it keeps the build green at the end of Foundational without requiring
  the LSP-side capability to exist yet, the same incremental-buildability discipline every prior
  Foundational phase in this project's specs has followed.
- T008 is this feature's single highest-risk task: it's the one place a genuinely new mechanism
  (a second server-initiated request, response parsing, notification-triggered re-pull) is added
  to `drut-lsp`'s main loop. T010's tests exist specifically to catch a capability-detection or
  parsing mistake here, not as mechanical coverage.
- Commit after each task or logical group.
