---

description: "Task list for UnmatchedProcess Diagnostic"
---

# Tasks: UnmatchedProcess Diagnostic

**Input**: Design documents from `/specs/006-unmatched-process-diagnostic/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/,
quickstart.md (all present)

**Tests**: Included — constitution Principle IV's zero-false-positive bar
for a new diagnostic category requires real test coverage, not just
"tests pass": `voyager-core`'s own unit tests for every Acceptance
Scenario, a real-shaped fixture-corpus regression test (FR-009), and full
161-file corpus revalidation reported as its own explicit result (this
session's established standard for core-crate changes, matching 004's
`block_at` extraction verification).

**Organization**: One primary user story (this feature has no smaller
independently-valuable increment beneath it), preceded by a genuinely
**compile-coupled** Foundational phase — not the usual "shared
infrastructure" kind, but a hard Rust-level coupling: research.md §1 found
three adapter crates maintain *exhaustive* `DiagnosticKind` matches with no
wildcard arm, so adding the new variant (`diagnostic.rs`) breaks
compilation of `drut-cli`/`drut-lsp`/`drut-mcp` until each gets its own new
match arm. These cannot be staged as independently-buildable increments the
way Foundational work usually can — they are listed as separate tasks for
clarity, but the phase's own checkpoint (not each task) is where the
workspace compiles again.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an
  incomplete sibling task)
- **[Story]**: US1 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/` — the actual feature (`diagnostic.rs`, `block.rs`)
  plus its own test updates (`tests/fixture_corpus.rs`, a new broken
  fixture).
- `crates/drut-cli/`, `crates/drut-lsp/`, `crates/drut-mcp/` — mechanical,
  non-decision-making adapter updates (research.md §1's exact inventory).
- `specs/001-voyager-script-parser/contracts/diagnostics.md` — amended in
  place (research.md §5), not a new competing contract file.

---

## Phase 1: Setup

- [X] T001 Confirm baseline: `cargo build --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings` both clean,
      on this fresh branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: Foundational (compile-coupled — see this file's own Organization note above)

**Purpose**: Land the new `DiagnosticKind` variant and every exhaustive
adapter match it breaks, together, so the workspace compiles again before
any real logic or tests are added.

**⚠️ CRITICAL**: `cargo build --workspace` does not pass again until every
task in this phase is done — there is no meaningful intermediate
checkpoint partway through it.

- [X] T002 Add the `UnmatchedProcess` variant to `DiagnosticKind` in
      `crates/voyager-core/src/diagnostic.rs`, with a doc comment matching
      the existing variants' convention (data-model.md, referencing FR-002).
- [X] T003 Add the matching arm to all three exhaustive adapter matches,
      using the final wording from `contracts/unmatched-process-
      diagnostic.md` (depends on T002 — these three edits are listed as one
      task because none of them compiles independently once T002 lands):
      - `crates/drut-cli/src/report/sarif.rs`: `ALL_KINDS` (7 → 8),
        `rule_id` (`"unmatched-process"`), `short_description`.
      - `crates/drut-lsp/src/diagnostics.rs`: `kind_name`
        (`"UnmatchedProcess"`); update the module doc's "six of
        `voyager-core`'s seven `DiagnosticKind` values" to seven of eight.
      - `crates/drut-mcp/src/diagnose.rs`: `category_name`
        (`"UnmatchedProcess"`); update `DiagnosticDto.category`'s doc
        comment count.
      Confirm `cargo build --workspace` and `cargo test --workspace`
      (pre-existing tests, unaffected — the new variant has no firing logic
      yet) both pass clean.

**Checkpoint**: Workspace compiles and all pre-existing tests pass with the
new variant in place but not yet fired by anything.

---

## Phase 3: User Story 1 - Getting warned about a PROCESS block that never closes (Priority: P1) 🎯 MVP

**Goal**: `parse_process` fires `UnmatchedProcess` under the exact
condition `parse_run` already uses for `UnmatchedRun`, with a real-shaped
regression fixture proving it against the actual scenario that surfaced
this gap.

**Independent Test**: Parse a script with an unclosed `PROCESS PHASE=...`
followed by real content and confirm exactly one `UnmatchedProcess`
diagnostic, pointing at the `PROCESS`/`PHASE=` statement.

### Implementation for User Story 1

- [X] T004 [US1] Implement the firing logic in `parse_process`
      (`crates/voyager-core/src/block.rs`), per research.md §3's exact diff
      — add the `Role::Process` implicit-close branch (now explicit, no
      diagnostic) and the new `UnmatchedProcess` diagnostic push for
      everything else (true EOF or an enclosing block's own closer forcing
      an early stop). Depends on T002/T003 (Foundational).
- [X] T005 [US1] Unit tests in `crates/voyager-core/src/block.rs`'s own
      `#[cfg(test)] mod tests`, mirroring `parse_run`'s existing test
      shape, covering all four spec.md Acceptance Scenarios: explicit
      `ENDPROCESS`/`ENDPHASE` close → no diagnostic; implicit close by a
      following `PROCESS`/`PHASE=` → no diagnostic (unchanged behavior,
      regression-proofed); genuinely unmatched at true EOF → exactly one
      `UnmatchedProcess`, span on the opener; `PROCESS` nested inside an
      `IF` whose `ENDIF` arrives first → `UnmatchedProcess` fires (the
      nested-early-stop case, proving research.md §3's "falls out for
      free" claim isn't just asserted). Depends on T004.
- [X] T006 [P] [US1] Add `"UnmatchedProcess" =>
      Some(DiagnosticKind::UnmatchedProcess)` to `parse_diagnostic_kind` in
      `crates/voyager-core/tests/fixture_corpus.rs`, and add
      `DiagnosticKind::UnmatchedProcess` to
      `every_diagnostic_category_has_at_least_one_broken_fixture`'s
      hardcoded array. Depends on T002 only (not on T004/T005) — parallel
      with them, different file.
- [X] T007 [US1] Add the real-shaped broken fixture,
      `crates/voyager-core/tests/fixtures/broken/
      unmatched_process_with_trailing_content.s` (FR-009): an unclosed
      `PROCESS PHASE=...` followed by multiple real subsequent statements
      (not the minimal one-line `process_block_reports_unconditional_
      counterpart` shape) — declares `; EXPECT: UnmatchedProcess` on its
      first line. Depends on T004 (the fixture must actually produce the
      diagnostic) and T006 (the marker must be recognized). Confirm
      `cargo test -p voyager-core --test fixture_corpus` passes, including
      `every_diagnostic_category_has_at_least_one_broken_fixture`.
- [X] T008 [P] [US1] Amend `specs/001-voyager-script-parser/contracts/
      diagnostics.md`: add the `UnmatchedProcess` table row, and rewrite
      the "Note on block kinds without a diagnostic category" paragraph to
      name only `JLoop`/`LinkLoop`/`DistributeMultistep` as still deferred,
      pointing to this feature as precedent (`contracts/
      unmatched-process-diagnostic.md`'s own exact text). Depends on T004
      (should describe final, implemented behavior) but is otherwise
      independent of T005-T007 — different file.

**Checkpoint**: `UnmatchedProcess` fully functional, unit-tested, and
covered by its own real-shaped fixture-corpus regression case.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Empirical re-proof of the zero-false-positive claim this
whole feature is based on, plus whole-workspace regression checks.

- [X] T009 Full real-corpus revalidation (quickstart.md step 4) —
      **reported as its own explicit, standalone result, per this
      session's own established standard for core-crate changes (not
      folded into a general "tests pass" summary)**:
      ```powershell
      $env:DRUT_CORPUS_PATH = "path\to\WF-TDM-Official-Releases"
      cargo test -p drut-cli --test fixture_corpus_e2e -- --ignored
      cargo test -p drut-lsp --test diagnostics_corpus -- --ignored
      cargo test -p drut-mcp --test diagnostics_corpus -- --ignored
      ```
      Expected and required: still 161/161 clean across all three — the
      empirical claim this feature's entire low-risk framing rests on
      (FR-008), re-verified through every adapter's own path, not just
      `voyager-core::parse` in isolation.
- [X] T010 [P] Manual adapter spot check (quickstart.md step 5, FR-007):
      run `drut check` against a fixture containing an unclosed `PROCESS`
      in both text and `--format sarif` modes; confirm text output shows
      `UnmatchedProcess` and SARIF output includes a
      `"ruleId": "unmatched-process"` result with the correct
      `shortDescription` in the rule catalog.
- [X] T011 [P] `cargo test --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings`, both clean —
      confirms zero regressions anywhere in the four-crate workspace.

**Checkpoint**: Feature-complete against spec.md; corpus claim re-proven,
not just carried forward from the motivating investigation.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup. **Hard compile-level
  block** on everything else — see this phase's own note above.
- **User Story 1 (Phase 3)**: Depends on Foundational. No other story to
  depend on or be independent of — this is the whole feature.
- **Polish (Phase 4)**: Depends on User Story 1 being complete (T009 needs
  real firing logic to revalidate against; T010 needs a real binary with
  the new diagnostic wired; T011 needs the whole workspace in its final
  state).

### Within User Story 1

- T004 (logic) before T005 (tests of that logic).
- T006 (test-helper support for the new kind) is parallel with T004/T005 —
  different file, only depends on T002.
- T007 (the real fixture) depends on both T004 (logic must exist to fire)
  and T006 (marker must be recognized) — genuinely sequential, not
  parallelizable against either.
- T008 (contracts doc) parallel with T005-T007 — different file, only
  really needs T004's logic to be stable to describe accurately.

### Parallel Opportunities

- T006 and T008 can proceed in parallel with T004/T005 and with each
  other — three different files, no shared dependency beyond Foundational.
- T010 and T011 in Polish are parallel with each other (different
  concerns); T009 is listed first as the empirical proof step but has no
  file-level conflict with either.

---

## Parallel Example: Once Foundational Completes

```bash
# T004/T005 (block.rs logic + its own tests) proceed alongside:
Task: "T006: fixture_corpus.rs test-helper updates"
Task: "T008: contracts/diagnostics.md amendment"

# T007 waits for both T004 and T006 to land before it can be written
# meaningfully.
```

---

## Implementation Strategy

### Single Pass (this feature has no smaller MVP slice)

1. Setup → baseline confirmed clean.
2. Foundational → workspace compiles again with the new variant in place.
3. User Story 1 → the actual diagnostic, fully tested, with its real-shaped
   regression fixture.
4. Polish → full corpus revalidation reported explicitly, whole-workspace
   regression check.
5. Ship — no partial/incremental delivery slice smaller than the whole
   feature makes sense here.

---

## Notes

- This feature has no Foundational-phase-then-independent-stories shape
  the way 004/005 did — it's small enough, and its one real complexity
  (the compile-coupled adapter matches) is a Foundational-phase concern
  in the literal sense, not an artificial one.
- Commit after each task or logical group.
- T009's explicit, standalone-reported result is a hard requirement from
  this feature's own motivating instruction (full corpus revalidation,
  reported as its own item) — not optional polish.
