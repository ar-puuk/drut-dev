---

description: "Task list for Undefined @token@ Diagnostic"
---

# Tasks: Undefined `@token@` Diagnostic

**Input**: Design documents from `/specs/020-undefined-token-diagnostic/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md
(all present)

**Tests**: Included — a new `voyager-core` public function plus a new LSP diagnostic stream,
touching real user-facing behavior, needs real coverage per spec.md's five Acceptance Scenarios
and three "never flag a blind spot" exclusions.

**Organization**: Single user story (no bundling this time — this feature has one priority
tier). Foundational carries the new enumeration function and the visibility widening that makes
`hover.rs`'s existing disk-I/O/resolution machinery reachable from a sibling module; User Story
1 is the actual diagnostic stream itself, built entirely on Foundational's pieces with no new
resolution logic of its own (research.md §3's central finding — every "never flag a blind spot"
requirement is already satisfied by reusing existing functions unmodified).

**Everything in this file's scope was measured against the real, current codebase during
planning (research.md), not estimated**:

- `drut-lsp/src/diagnostics.rs::publish` already builds three chained diagnostic-iterator
  streams (structural `DiagnosticKind`-based, the `; FMT: OFF` marker, the malformed `drut.toml`
  warning) — confirmed by reading the actual function, not assumed. This feature adds a fourth,
  identically shaped.
- `hover.rs::collect_included_files`/`struct IncludedFile` already do all the disk-I/O this
  feature needs (read a literal-path `READ FILE` target, parse it, gracefully omit on any
  failure) — both are currently private to `hover.rs`, confirmed directly in the source.
- Each of FR-003's three "never flag a resolver blind spot" exclusions (block-opener position,
  multi-level inclusion, token-built inclusion path) is already true by construction of the
  existing functions this feature reuses unmodified — confirmed per-case against the actual
  code (research.md §3), not a new suppression rule to write.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete sibling task)
- **[Story]**: US1 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/src/token_resolution.rs` — `all_variable_refs`, and its own test module.
- `crates/drut-lsp/src/hover.rs` — visibility widening only, no behavior change.
- `crates/drut-lsp/src/undefined_token.rs` (new) — `undefined_token_positions`, its own test
  module.
- `crates/drut-lsp/src/diagnostics.rs` — the fourth chained stream, extended test coverage.
- `crates/drut-cli/tests/`, `crates/drut-mcp/src/diagnose.rs`'s own tests — the one negative
  ("never reaches here") confirmation task.
- `ROADMAP.md` — item 14 marked done (Polish).

---

## Phase 1: Setup

- [X] T001 Confirm baseline: `cargo build --workspace` and `cargo clippy --workspace
      --all-targets -- -D warnings` both clean, on this branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The enumeration function and the reused-resolution-machinery reach this feature
needs. Not independently user-visible on its own — US1 is what actually publishes anything.

- [X] T002 In `crates/voyager-core/src/token_resolution.rs`: add `all_variable_refs(nodes:
      &[Node]) -> Vec<VariableRefAt>` (data-model.md §1) — reuses `collect_statements`/
      `collect_if_condition_token_slices` exactly as `variable_ref_at` does, collecting every
      `VariableRef` match instead of stopping at the first one containing a given position.
      Pure, no I/O, never panics. Depends on nothing (existing module, existing helpers).
- [X] T003 [P] In `crates/drut-lsp/src/hover.rs`: widen `collect_included_files` and `struct
      IncludedFile` from private to `pub(crate)` (data-model.md §2, research.md §4) — no
      behavior change, `hover.rs`'s own call sites unaffected. Depends on nothing.
- [X] T004 [P] Add unit tests to `crates/voyager-core/src/token_resolution.rs`'s own test
      module: multiple `@token@` references in one document are all returned, in source order;
      an `IfBranch.condition` reference is included; a `@token@` on a block-opener line
      (`RUN PGM=@Prog@`) is absent from the result (research.md §3 — confirms this holds by
      construction, not by a new filter); a document with no `@token@` references returns an
      empty `Vec`. Depends on T002.

**Checkpoint**: `all_variable_refs` exists, compiles, is tested; `collect_included_files`/
`IncludedFile` are reachable from a sibling module. `cargo build --workspace` succeeds (nothing
calls the new function or the widened items yet — unaffected).

---

## Phase 3: User Story 1 - A developer notices an unresolvable `@token@` reference while editing (Priority: P1)

**Goal**: Opening or editing a `.s`/`.block` file with a `@token@` reference the existing
resolution logic can't resolve shows a Hint/Information-severity underline at that reference —
live, LSP-only, never overclaiming certainty on a known resolver blind spot.

**Independent Test**: Open a document containing one unresolvable `@token@` reference and one
resolvable one; confirm exactly the unresolvable one receives a Hint-severity notice.

### Implementation for User Story 1

- [X] T005 [US1] Create `crates/drut-lsp/src/undefined_token.rs`: `undefined_token_positions
      (uri: &lsp_types::Uri, doc: &OpenDocument) -> Vec<VariableRefAt>` (data-model.md §2) —
      calls `hover::collect_included_files` once, builds the `included` list T003's widening now
      allows, then filters `token_resolution::all_variable_refs(&doc.parse_result.nodes)` down
      to references where `token_resolution::resolve_token_value(&doc.parse_result.nodes,
      var_ref.span.start, &included, &var_ref.name)` returns `None`. No new resolution logic —
      every exclusion in FR-003 is inherited automatically from the reused functions (research.md
      §3). Depends on T002, T003.
- [X] T006 [US1] In `crates/drut-lsp/src/diagnostics.rs::publish`: add a fourth chained stream,
      `undefined_token_diagnostics` (data-model.md §3) — maps each `undefined_token_positions`
      result to an `lsp_types::Diagnostic` with `severity: DiagnosticSeverity::HINT`, `code:
      "UndefinedToken"`, `source: "drut-token"`, and the hedged message wording from
      data-model.md §3 (never asserts non-existence outright). Chains it alongside the existing
      `structural_diagnostics`/`fmt_marker_diagnostics`/`config_warnings` streams into the final
      `diagnostics` list. Depends on T005.

### Tests for User Story 1

- [X] T007 [P] [US1] Add unit tests to `crates/drut-lsp/src/undefined_token.rs`'s own test
      module, covering every spec.md US1 Acceptance Scenario directly: a `@token@` with no
      same-file assignment and no `READ FILE` inclusion is returned (AS1); a `@token@` with a
      same-file assignment is not returned (AS2); a `@token@` on a block-opener line is not
      returned (AS3); a `@token@` resolvable only through two levels of `READ FILE` inclusion is
      returned — correctly not resolved, since only one level is followed (AS4); a `@token@`
      resolvable only through a token-built `READ FILE` path is returned — correctly not
      resolved (AS5). Also: a `@token@` resolvable through one direct, literal-path `READ FILE`
      inclusion is not returned (the positive one-level case). Depends on T005.
- [X] T008 [P] [US1] Add tests to `crates/drut-lsp/src/diagnostics.rs`'s own test module: a
      published diagnostics list for a document with one unresolvable `@token@` includes exactly
      one `HINT`-severity, `"drut-token"`-sourced, `"UndefinedToken"`-coded entry at that
      reference's exact span; the six real `DiagnosticKind`-based diagnostics in the same
      document still publish at `ERROR` severity, source `"drut"`, completely unaffected by this
      feature's addition (SC-004); editing the document to add the missing assignment and
      re-publishing removes the notice, with no other diagnostic stream affected (FR-007).
      Depends on T006.
- [X] T009 [US1] Add a test confirming SC-005 directly: on a document containing at least one
      unresolvable `@token@`, `drut-cli`'s `check` command output and `drut-mcp`'s `diagnose`
      tool output both contain exactly the pre-existing six/seven `DiagnosticKind` category
      names and nothing else — this stream never reaches either surface. Depends on T006.

**Checkpoint**: User Story 1 independently proven — the notice appears exactly where spec.md's
Acceptance Scenarios say it should, at the right severity, on the right surface only, and
disappears live once the missing definition is added.

---

## Phase 4: Polish & Cross-Cutting Concerns

- [X] T010 [P] In `ROADMAP.md`: mark item 14 done, dated, pointing at this feature's spec
      directory — same pattern every other completed `ROADMAP.md` item already follows.
- [X] T011 `cargo test --release --workspace` and `cargo clippy --workspace --all-targets --
      -D warnings`, both clean.
- [X] T012 Run `quickstart.md` end-to-end as written, confirming every step's expected result
      holds against the actual shipped code, not just against the individual task-level tests
      above in isolation.

**Checkpoint**: Feature-complete against spec.md; `ROADMAP.md` consistent with shipped code;
full workspace re-proven clean.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS User Story 1.
- **User Story 1 (Phase 3)**: Depends on Foundational only.
- **Polish (Phase 4)**: T010 is independent of the code phases; T011-T012 depend on User Story 1
  being complete.

### Parallel Opportunities

- T003, T004 can run in parallel with each other and (T003 only) with T002 — different files/
  concerns, T004 itself depends on T002 landing first.
- T007, T008 can run in parallel once T005/T006 land (different test files).

---

## Parallel Example: Once Foundational (T002-T004) Lands

```bash
Task: "T005: undefined_token.rs — undefined_token_positions"
Task: "T010: ROADMAP.md item 14 marked done"
```

---

## Implementation Strategy

### MVP First (this feature IS the MVP — single story)

1. Setup → baseline confirmed clean.
2. Foundational → enumeration function and reused resolution machinery reachable.
3. User Story 1 → the notice publishes correctly, at the right severity, on the right surface,
   proven against every Acceptance Scenario.
4. **STOP and VALIDATE**: run T007-T009 against real fixtures for each of the five scenarios.

### Incremental Delivery

1. Foundational → foundation ready.
2. US1 → feature complete (there is no second increment for this feature).
3. Polish → `ROADMAP.md` update, full re-proof.

---

## Notes

- This feature deliberately has no golden-fixture/real-corpus revalidation phase, unlike every
  formatting feature this session — it changes no formatting or parsing output, only what gets
  published as LSP diagnostics (plan.md's Scale/Scope). Coverage is targeted fixtures per
  Acceptance Scenario instead.
- T005's "no new resolution logic" framing is load-bearing, not incidental — if a future change
  to this feature ever needs to add its own exclusion rule beyond what `all_variable_refs`/
  `collect_included_files`/`resolve_token_value` already provide, that's a signal the reused
  functions' own behavior has drifted from what this feature assumed, worth re-checking against
  research.md §3 rather than layering a patch on top.
- Commit after each task or logical group.
