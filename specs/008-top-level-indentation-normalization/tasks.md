---

description: "Task list for Top-Level Indentation Normalization"
---

# Tasks: Top-Level Indentation Normalization

**Input**: Design documents from `/specs/008-top-level-indentation-normalization/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/,
quickstart.md (all present)

**Tests**: Included — a formatter behavioral reversal touching real corpus
output requires real coverage: `voyager-core`'s own unit tests for every
new/changed case, the residue-resolution regression, the `007`-interaction
case, and human-reviewed golden-fixture regeneration as this feature's own
Definition of Done (constitution Principle III's existing gate).

**Organization**: One primary user story (the policy change itself, US1)
plus a second, narrower story (US2 — the residue-scenario proof this
whole debugging thread was chasing), matching spec.md exactly. No
Foundational phase — the entire code change is one function, with no
shared prerequisite work any story needs before the other.

**Everything in this file's scope was measured against a real, working
prototype during planning (research.md §1/§3), not estimated**:

- **Exactly one pre-existing test breaks**: `format::tests::
  top_level_baseline_is_left_untouched` (`crates/voyager-core/src/
  format.rs`) — its own name and assertion directly encode the old
  policy. Every other test in the entire workspace (114 other
  `voyager-core` lib tests, all of `format_sequence.rs`, all of
  `drut-lsp`/`drut-cli`/`drut-mcp`) passes unchanged.
- **Exactly 7 of 9 `real_corpus/` golden fixtures drift**; zero
  hand-written `valid/` fixtures are affected. Full file list below (T008).
- **T006 (added after the initial task-generation pass, before
  implementation started)**: the one `007`/`008` interaction point not
  obviously covered by either feature's own tests in isolation — a
  genuinely diagnosed block whose own opener sits at non-zero
  indentation. Verified via the same prototype: opener corrected to 0,
  every child (both legitimate body content and any swallowed trailing
  content) left completely untouched, `UnmatchedProcess` still reported.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an
  incomplete sibling task)
- **[Story]**: US1/US2 — omitted for Setup/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/src/format.rs` — the actual change
  (`plan_indentation`) plus its own test module.
- `crates/voyager-core/tests/format_sequence.rs` — new residue-resolution
  test (US2).
- `crates/voyager-core/tests/fixtures/golden/real_corpus/` — regenerated
  golden fixtures (US1, human-reviewed).
- `specs/002-cli-check-format/spec.md` +
  `specs/002-cli-check-format/contracts/formatting-api.md` — amended per
  `contracts/top-level-indentation.md`'s exact replacement text.

---

## Phase 1: Setup

- [ ] T001 Confirm baseline: `cargo build --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings` both clean,
      on this fresh branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: User Story 1 - Every top-level statement lands at column 0, always (Priority: P1) 🎯 MVP

**Goal**: `plan_indentation` unconditionally force-plans every top-level
node (statement or block) to column 0, per research.md §1's exact,
prototype-verified one-line change.

**Independent Test**: Format a script with a top-level statement at a
non-zero column and confirm it's corrected to column 0, with children
still correctly indented relative to that new base.

### Implementation for User Story 1

- [ ] T002 [US1] Implement the change in `plan_indentation`
      (`crates/voyager-core/src/format.rs`), per research.md §1's exact
      diff: `plan.insert(node.span().start.line, 0);` before the existing
      `if let Node::Block(block) = node { plan_block(...) }` check. No
      other function changes.
- [ ] T003 [US1] Update `plan_block`'s and `diagnosed_block_openers`'s own
      doc comments (`crates/voyager-core/src/format.rs`) to state the
      narrowed `007` rationale (research.md §1, contracts/
      top-level-indentation.md): the skip never protected a diagnosed
      block's opener line (this feature's new rule now owns that,
      independently); it only ever protects the block's *children*.
      Depends on T002 (describes the post-T002 state accurately).
- [ ] T004 [P] [US1] Rewrite the one broken pre-existing test,
      `top_level_baseline_is_left_untouched` →
      `top_level_baseline_is_always_normalized_to_zero`
      (`crates/voyager-core/src/format.rs`), asserting the new expected
      output for the same input (`"        RUN PGM=MATRIX\n        X = 1\n        ENDRUN\n"`
      → `"RUN PGM=MATRIX\n    X = 1\nENDRUN\n"`). Depends on T002.
- [ ] T005 [US1] Add new unit tests to `crates/voyager-core/src/format.rs`'s
      own `#[cfg(test)] mod tests`, covering spec.md Acceptance Scenarios
      1-3: a bare top-level statement (no enclosing block at all) gets
      normalized to column 0 (the previously-entirely-untouched case,
      research.md §1); a top-level block opener with stale-indented
      children gets both the opener *and* its children corrected together
      in one pass; an already-column-0 file stays byte-identical
      (`changed: false`, idempotence). Depends on T002.
- [ ] T006 [US1] Add a new unit test to `crates/voyager-core/src/format.rs`'s
      own `#[cfg(test)] mod tests`,
      `diagnosed_block_opener_is_normalized_but_children_stay_untouched`
      — the explicit `007`/`008` interaction case confirmed live during
      planning, not left to be inferred from the two features' own
      isolated test suites. Input: a genuinely unclosed `PROCESS` whose
      own opener sits at non-zero indentation, with both its legitimate
      body content (`FILEI`) and a swallowed trailing `RUN` block also at
      non-zero indentation —
      `"    PROCESS PHASE=INPUT\n        FILEI = ni.1\n\n    RUN PGM=HWYASSIGN\n        FILEI NETI = 'net.net'\n    ENDRUN\n"`.
      Asserts: `changed: true`; exactly one diagnostic,
      `DiagnosticKind::UnmatchedProcess`; the output's `PROCESS` line has
      zero leading spaces (corrected); every other line — `FILEI = ni.1`,
      `RUN PGM=HWYASSIGN`, `FILEI NETI = ...`, `ENDRUN` — is byte-for-byte
      identical to the input (untouched, `007`'s skip still applies to
      the diagnosed block's children). Depends on T002.
- [ ] T007 [P] [US1] Amend `specs/002-cli-check-format/spec.md`'s FR-012
      bullet and `specs/002-cli-check-format/contracts/formatting-api.md`,
      using `contracts/top-level-indentation.md`'s exact replacement text
      verbatim. Independent of T002-T006 — documentation only, different
      files.
- [ ] T008 [US1] Regenerate and individually human-review the 7 affected
      golden fixtures (quickstart.md step 4):
      ```powershell
      $env:UPDATE_GOLDEN = "1"
      cargo test -p voyager-core --test format_corpus
      Remove-Item Env:\UPDATE_GOLDEN
      ```
      Exact file list (research.md §3 — measured, not estimated):
      - `tests/fixtures/golden/real_corpus/AssignHwy/02_Assign_AM_MD_PM_EV.s`
      - `tests/fixtures/golden/real_corpus/AssignHwy/09_TAZ_Based_Metrics.s`
      - `tests/fixtures/golden/real_corpus/Distribute/3_SumToDistricts_GRAVITY.s`
      - `tests/fixtures/golden/real_corpus/Distribute/4pd_mainbody_distribution.block`
      - `tests/fixtures/golden/real_corpus/InputProcessing/1_InputSetup.s`
      - `tests/fixtures/golden/real_corpus/InputProcessing/2_UrbanizationTermTime.s`
      - `tests/fixtures/golden/real_corpus/ModeChoice/06_HBW_logsums.s`
      For each file, `git diff` its golden copy and confirm explicitly:
      only leading-whitespace changes, every changed line's new leading
      whitespace is `0` for what was previously a top-level line, nothing
      reordered/added/removed/corrupted elsewhere in the file. Report each
      file's review outcome individually — this is FR-006/SC-003's own
      Definition of Done, not a mechanical regenerate-and-commit step.
      Depends on T002 (the golden output must reflect the real fix).

**Checkpoint**: Top-level normalization fully implemented, unit-tested
(including the explicit `007` interaction), and every affected golden
fixture regenerated and reviewed.

---

## Phase 3: User Story 2 - The known residue scenario fully self-resolves (Priority: P2)

**Goal**: Prove, with a new regression test using the *stale-indentation*
shape research.md §1's prototype specifically validated (not the
already-correct shape `007`'s own tests already cover), that the
`PROCESS`/`RUN` residue sequence fully resolves within the second format
pass alone.

**Independent Test**: Reproduce the sequence with `RUN` left at stale
non-zero indentation after `ENDPROCESS` is added, and confirm one format
pass corrects everything.

### Implementation for User Story 2

- [ ] T009 [US2] Add a new test to
      `crates/voyager-core/tests/format_sequence.rs`,
      `process_run_residue_with_stale_run_indentation_resolves_in_one_pass`:
      unlike the existing `process_run_residue_is_fixed_after_endprocess_
      is_added` (whose `RUN` was already correctly positioned, per `007`'s
      own no-speculative-write behavior), this test's `step2` input has
      `RUN`/`FILEI NETI`/`ENDRUN` at *stale*, non-zero indentation (e.g.
      `RUN` at 4 spaces, `FILEI NETI` at 8, `ENDRUN` at 4) after
      `ENDPROCESS` is added — the shape `007` alone never corrected, and
      the shape this feature's own unconditional top-level rule fixes
      directly (research.md §1's prototype table, row 4). Asserts
      `changed: true` and the fully-corrected output in that single pass.
      Depends on T002 (US1's implementation).

**Checkpoint**: The residue scenario's hardest variant (stale, not just
absent, indentation) is proven to self-resolve in one pass.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Whole-workspace and full-corpus re-proof, once the policy
change and its golden-fixture regeneration are both done.

- [ ] T010 `cargo test --release --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings`, both
      clean — confirms zero regressions anywhere (research.md §2 already
      confirmed zero adapter code changes are needed; this re-proves
      nothing else broke).
- [ ] T011 [P] Full 161-file corpus revalidation across all three adapter
      surfaces (quickstart.md step 5), each reported individually:
      ```powershell
      $env:DRUT_CORPUS_PATH = "path\to\WF-TDM-Official-Releases"
      cargo test --release -p drut-cli --test fixture_corpus_e2e -- --ignored
      cargo test --release -p drut-lsp --test diagnostics_corpus -- --ignored
      cargo test --release -p drut-mcp --test diagnostics_corpus -- --ignored
      ```
      Expected and required: still 161/161 clean (SC-004) — this is a
      whitespace-shifting change only, zero new diagnostics anywhere.

**Checkpoint**: Feature-complete against spec.md; golden fixtures reflect
reality; full corpus re-proven clean.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **User Story 1 (Phase 2)**: Depends on Setup. No dependency on US2 —
  the whole policy change and its own regression coverage/golden
  regeneration stand alone.
- **User Story 2 (Phase 3)**: Depends on US1's T002 (the implementation
  must exist for the new stale-indentation regression test to have
  anything to prove). Not required for US1 to be complete/mergeable on
  its own — but per spec.md, both stories ship together in this feature.
- **Polish (Phase 4)**: Depends on both stories being complete.

### Within User Story 1

- T002 (implementation) before T003 (doc comments describing the
  post-T002 state), T004 (rewritten test), T005/T006 (new tests), T008
  (golden regeneration, needs the real fixed output).
- T007 (spec/contract doc amendment) is independent of T002-T006/T008 —
  different files, no code dependency — but logically describes the same
  change, so keep it in the same review pass.

### Parallel Opportunities

- T004, T005, T006, T007 can all proceed in parallel once T002 lands
  (T004/T005/T006 share `format.rs` but are additive, non-conflicting
  edits; T007 is a different file entirely).
- T011's three corpus-validation commands are independent of each other
  (different crates) and of T010 (different scope) — genuinely parallel,
  though T010 is listed first as the cheaper, faster gate.

---

## Parallel Example: Once T002 Lands

```bash
Task: "T004: rewrite top_level_baseline_is_left_untouched"
Task: "T005: add new format.rs unit test cases"
Task: "T006: add the diagnosed-opener/untouched-children interaction test"
Task: "T007: amend spec.md FR-012 + formatting-api.md"
```

---

## Implementation Strategy

### Single Pass (both stories are small and directly coupled)

1. Setup → baseline confirmed clean.
2. User Story 1 → the policy change, its tests (including the explicit
   `007` interaction, T006), its doc amendment, and its human-reviewed
   golden-fixture regeneration (T008 is the one step in this whole
   feature that cannot be rushed or automated away).
3. User Story 2 → the stale-indentation residue proof.
4. Polish → whole-workspace and full-corpus re-proof, reported explicitly.

---

## Notes

- T008's human-review step is this feature's real bottleneck and its
  actual Definition of Done for FR-006/SC-003 — not a formality. Report
  each of the 7 files' diff review individually before considering this
  feature done, matching the original `T023b` discipline those golden
  files were first created under.
- T006 was added after this file's initial generation, in response to an
  explicit review question about the one `007`/`008` interaction point
  neither feature's own isolated tests obviously covered — already
  verified against the real prototype before being written into this
  file, not a placeholder.
- Commit after each task or logical group.
