---

description: "Task list for Unused @token@ Diagnostic"
---

# Tasks: Unused `@token@` Diagnostic

**Input**: Design documents from `/specs/029-unused-token-diagnostic/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md
(all present)

**Tests**: Included — a new `voyager-core` public function plus a new LSP diagnostic stream,
touching real user-facing behavior, needs real coverage per spec.md's six Acceptance Scenarios.

**Organization**: Single user story (this feature has one priority tier, same shape as its
`020-undefined-token-diagnostic` sibling). Foundational carries the new opener-aware enumeration
function; User Story 1 is the actual diagnostic stream itself, built on Foundational's piece plus
`020`'s already-existing `all_assignments`/`hover::collect_included_files` reuse.

**Everything in this file's scope was measured against the real, current codebase during
planning (research.md), not estimated**:

- `drut-lsp/src/diagnostics.rs::publish` already builds four chained diagnostic-iterator streams
  (structural `DiagnosticKind`-based, the `; FMT: OFF` marker, the malformed `drut.toml` warning,
  `UndefinedToken`) — confirmed by reading the actual function. This feature adds a fifth,
  identically shaped.
- `Block::opener_tokens` already exists (added this session for an unrelated casing fix) and
  already carries exactly the token stream this feature needs to scan — confirmed directly
  against `block.rs`. No new `Block` field, no parser change.
- `all_assignments`, `hover::collect_included_files` already exist and need zero changes
  (research.md §3-4) — this feature reuses both verbatim.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete sibling task)
- **[Story]**: US1 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/src/token_resolution.rs` — `all_variable_refs_including_openers`, and its
  own test module.
- `crates/drut-lsp/src/unused_token.rs` (new) — `unused_token_assignments`, its own test module.
- `crates/drut-lsp/src/diagnostics.rs` — the fifth chained stream, extended test coverage.
- `crates/drut-cli/tests/`, `crates/drut-mcp/src/diagnose.rs`'s own tests — the one negative
  ("never reaches here") confirmation task.
- `ROADMAP.md` — new item marked done (Polish).

---

## Phase 1: Setup

- [X] T001 Confirm baseline: `cargo build --workspace` and `cargo clippy --workspace
      --all-targets -- -D warnings` both clean, on this branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The opener-aware enumeration function this feature needs, and nothing else — it's
the one genuinely new piece of resolution logic (research.md §1-2). Not independently
user-visible on its own — US1 is what actually publishes anything.

- [X] T002 In `crates/voyager-core/src/token_resolution.rs`: add a private helper
      `collect_opener_token_slices(nodes: &[Node], out: &mut Vec<&[Token]>)` (data-model.md §1)
      — structurally identical to the existing `collect_if_condition_token_slices`, but pushing
      each non-empty `Block::opener_tokens` slice instead of each `IfBranch.condition`. Depends
      on nothing (existing module).
- [X] T003 In `crates/voyager-core/src/token_resolution.rs`: add `pub fn
      all_variable_refs_including_openers(nodes: &[Node]) -> Vec<VariableRefAt>` (data-model.md
      §1) — identical body to the existing `all_variable_refs`, plus one more loop over T002's
      `collect_opener_token_slices` output before the final source-order sort. `all_variable_refs`
      itself is not modified. Depends on T002.
- [X] T004 [P] Add unit tests to `crates/voyager-core/src/token_resolution.rs`'s own test
      module: everything `all_variable_refs` already covers is also covered by
      `all_variable_refs_including_openers`; a `@token@` on a block-opener line
      (`RUN PGM=@Prog@`) IS present in `all_variable_refs_including_openers`'s result (the one
      behavioral difference); `all_variable_refs`'s own existing tests, including
      `all_variable_refs_excludes_a_block_opener_reference`, still pass unmodified. Depends on
      T003.

**Checkpoint**: `all_variable_refs_including_openers` exists, compiles, is tested;
`all_variable_refs` and every existing `020` consumer remain byte-for-byte unchanged. `cargo
build --workspace` succeeds (nothing calls the new function yet — unaffected).

---

## Phase 3: User Story 1 - A developer notices a dead token assignment while editing (Priority: P1)

**Goal**: Opening or editing a `.s`/`.block` file with an `Assignment` whose target name is
never referenced via `@name@` anywhere in scope shows a Hint/Information-severity underline at
that assignment — live, LSP-only, every dead assignment site flagged independently, applied
unconditionally regardless of `READ FILE` participation (Clarifications Q1/Q2).

**Independent Test**: Open a document containing one assignment with no reference anywhere and
one assignment that is referenced later; confirm exactly the unreferenced one receives a
Hint-severity notice.

### Implementation for User Story 1

- [X] T005 [US1] Create `crates/drut-lsp/src/unused_token.rs`: `struct UnusedAssignment { target:
      String, value_span: Span, statement_span: Span }` and `unused_token_assignments(uri:
      &lsp_types::Uri, doc: &OpenDocument) -> Vec<UnusedAssignment>` (data-model.md §2) — builds
      a case-insensitive `HashSet` of every referenced name by calling
      `all_variable_refs_including_openers` on `doc`'s own nodes plus each of
      `hover::collect_included_files(uri, doc)`'s included files' nodes, then returns every
      `all_assignments(&doc.parse_result.nodes)` entry whose target isn't in that set. Depends on
      T003.
- [X] T006 [US1] In `crates/drut-lsp/src/diagnostics.rs::publish`: add a fifth chained stream,
      `unused_token_diagnostics` (data-model.md §3) — maps each `unused_token_assignments`
      result to an `lsp_types::Diagnostic` with `range: a.statement_span`, `severity:
      DiagnosticSeverity::HINT`, `code: "UnusedToken"`, `source: "drut-token"`, and the hedged
      message wording from data-model.md §3. Chains it alongside the existing
      `structural_diagnostics`/`fmt_marker_diagnostics`/`config_warnings`/
      `undefined_token_diagnostics` streams into the final `diagnostics` list. Depends on T005.

### Tests for User Story 1

- [X] T007 [P] [US1] Add unit tests to `crates/drut-lsp/src/unused_token.rs`'s own test module,
      covering every spec.md US1 Acceptance Scenario directly: an assignment with no reference
      anywhere is returned (AS1); an assignment referenced only on a block-opener line is NOT
      returned (AS2, the correctness fix); an assignment reassigned twice with one reference
      after both is not returned for either site (AS3); a document with no references and no
      `READ FILE` statements returns its one unreferenced assignment (AS4); an assignment
      reassigned twice with zero references anywhere returns BOTH sites independently (AS5,
      Clarification Q1); an assignment with no reference in a file that also has a `READ FILE`
      statement is still returned (AS6, Clarification Q2). Also: an assignment resolvable through
      one level of `READ FILE` inclusion is not returned (the positive one-level case). Depends
      on T005.
- [X] T008 [P] [US1] Add tests to `crates/drut-lsp/src/diagnostics.rs`'s own test module: a
      published diagnostics list for a document with one unused assignment includes exactly one
      `HINT`-severity, `"drut-token"`-sourced, `"UnusedToken"`-coded entry spanning that
      assignment statement; the six real `DiagnosticKind`-based diagnostics and `UndefinedToken`
      in the same document still publish unaffected by this feature's addition (SC-004); editing
      the document to add a reference and re-publishing removes the notice, with no other
      diagnostic stream affected (FR-007). Depends on T006.
- [X] T009 [US1] Add a test confirming SC-005 directly: on a document containing at least one
      unused assignment, `drut-cli`'s `check` command output and `drut-mcp`'s `diagnose` tool
      output both contain exactly the pre-existing `DiagnosticKind` category names and nothing
      else — this stream never reaches either surface. Depends on T006.

**Checkpoint**: User Story 1 independently proven — the notice appears exactly where spec.md's
Acceptance Scenarios say it should, at the right severity, on the right surface only, and
disappears live once a reference is added.

---

## Phase 4: Polish & Cross-Cutting Concerns

- [X] T010 [P] In `ROADMAP.md`: add and mark done a new item for this feature, dated, pointing
      at this feature's spec directory — same pattern every other completed `ROADMAP.md` item
      already follows.
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

- T004 depends on T003 (needs the function to test), but is otherwise independent of any other
  file.
- T007, T008 can run in parallel once T005/T006 land (different test files).

---

## Parallel Example: Once Foundational (T002-T004) Lands

```bash
Task: "T005: unused_token.rs — unused_token_assignments"
Task: "T010: ROADMAP.md new item marked done"
```

---

## Implementation Strategy

### MVP First (this feature IS the MVP — single story)

1. Setup → baseline confirmed clean.
2. Foundational → opener-aware enumeration function exists and is tested in isolation.
3. User Story 1 → the notice publishes correctly, at the right severity, on the right surface,
   proven against every Acceptance Scenario.
4. **STOP and VALIDATE**: run T007-T009 against real fixtures for each of the six scenarios.

### Incremental Delivery

1. Foundational → foundation ready.
2. US1 → feature complete (there is no second increment for this feature).
3. Polish → `ROADMAP.md` update, full re-proof.

---

## Notes

- This feature deliberately has no golden-fixture/real-corpus revalidation phase, matching
  `020`'s own precedent — it changes no formatting or parsing output, only what gets published
  as LSP diagnostics (plan.md's Scale/Scope). Coverage is targeted fixtures per Acceptance
  Scenario instead.
- T005's set-difference approach (not a per-reference `resolve_token_value` call) is deliberate,
  not a shortcut — this feature only needs "is this name referenced anywhere," never "which
  specific assignment does this specific reference resolve to" (research.md §4).
- Commit after each task or logical group.
