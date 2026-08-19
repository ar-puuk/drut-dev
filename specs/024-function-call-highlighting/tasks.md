---

description: "Task list for Function-Call Syntax Highlighting (024-function-call-highlighting)"
---

# Tasks: Function-Call Syntax Highlighting

**Input**: Design documents from `/specs/024-function-call-highlighting/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/function-call-highlighting.md, quickstart.md

**Tests**: Not explicitly requested in `spec.md`, but included anyway — this project's
established convention (every prior grammar change, e.g. `003-lsp-vscode-extension`'s own
`grammar.test.ts` suite) always adds coverage to that same test harness before merge.

**Organization**: Tasks are grouped by user story (spec.md: US1 P1, US2 P2). Both stories
are satisfied by the same single grammar-pattern addition (Phase 2) — a `(`-lookahead-gated
pattern simultaneously delivers "colors everywhere" (US1) and "never fires without a call"
(US2) — so each story's phase here is test coverage proving its half of that one change.

## Path Conventions

Single existing project, `editors/vscode/` (plan.md Project Structure) — no new directory.

---

## Phase 1: Setup

**Purpose**: Confirm the existing dev environment; no new dependencies for this feature.

- [X] T001 Confirm `editors/vscode` deps are installed (`npm install` in `editors/vscode/`,
      already a prerequisite of the shipped `003-lsp-vscode-extension`) — no `package.json`
      change needed (plan.md Technical Context: no new dependency).

---

## Phase 2: Foundational (Blocking Prerequisite)

**Purpose**: The one grammar change every user story's tests validate.

**⚠️ CRITICAL**: No user story task can be meaningfully tested until this is complete.

- [X] T002 Add the `#function-calls` repository pattern to
      `editors/vscode/syntaxes/drut.tmLanguage.json`: the 138-name alternation from
      `data-model.md` §1 (case-insensitive), a `(?=\()` lookahead requiring `(` to
      immediately follow with no intervening whitespace, scoped `support.function.drut`
      (`data-model.md` §2, `contracts/function-call-highlighting.md`). Wire it into the
      top-level `patterns` array alongside `#control-words`/`#statement-words`.

**Checkpoint**: Grammar loads without error (`npm run compile` in `editors/vscode/`
succeeds); user story test tasks can now be added.

---

## Phase 3: User Story 1 - A built-in function reads the same color everywhere (Priority: P1) 🎯 MVP

**Goal**: Every recognized function name renders `support.function.drut` regardless of
statement position (spec.md Acceptance Scenarios 1-4).

**Independent Test**: Tokenize each line below with `editors/vscode/test/grammar.test.ts`'s
harness and confirm the named token's scope includes `support.function`.

- [X] T003 [P] [US1] Add grammar test in `editors/vscode/test/grammar.test.ts`:
      `if (RIGHTSTR(TRIM(RouteName),1)='-')` — both `RIGHTSTR` and `TRIM` scope as
      `support.function` (spec.md AS2; quickstart.md §2).
- [X] T004 [P] [US1] Add grammar test in `editors/vscode/test/grammar.test.ts`:
      `if (STRLEN(TRIM(@SEGIDExField@))>0)` — both `STRLEN` and `TRIM` scope as
      `support.function`, and `@SEGIDExField@` still scopes as `variable.other.readwrite`
      (spec.md AS3; quickstart.md §2).
- [X] T005 [P] [US1] Add grammar test in `editors/vscode/test/grammar.test.ts`:
      `RouteName = REPLACESTR(RouteName,'-','',0)` — `REPLACESTR` scopes as
      `support.function` (spec.md AS1; contract table row 1 — same visual result as today,
      now via `#function-calls` instead of the `#pair-values` accident).
- [X] T006 [P] [US1] Add grammar test in `editors/vscode/test/grammar.test.ts`:
      `ANGLE = ROUND(_L.S_Angle * 10) / 10` — `ROUND` scopes as `support.function`;
      `_L.S_Angle` does not (spec.md AS4).
- [X] T007 [P] [US1] Add grammar test in `editors/vscode/test/grammar.test.ts`: a
      case-insensitive occurrence, e.g. `CmpNumRetNum(V,'=',0,1,V)`, scopes as
      `support.function` (FR-003; research.md §4 cross-check).
- [X] T008 [P] [US1] Add grammar test in `editors/vscode/test/grammar.test.ts`: a
      vendor-reference-only function with no `WF-TDM-Official-Releases` corpus occurrence
      at all, e.g. `SUBSTR(street,4,6)` and `ARCSIN(0.5)`, scopes as `support.function` —
      validates FR-005's broadened, not-corpus-gated scope (quickstart.md §2; this is the
      check that would have failed under this feature's original, corpus-only 21-name
      draft — research.md §1).
- [X] T009 [P] [US1] Add grammar test in `editors/vscode/test/grammar.test.ts`: a real
      CONVERGE-phase usage line from the reference guide,
      `IF (GAPCHANGEAVE(3) < 0.006 && GAPCHANGEMAX(3) < 0.009) BALANCE = 1` — all three of
      `GAPCHANGEAVE`, `GAPCHANGEMAX` scope as `support.function`; `BALANCE` does not
      (research.md §2, CONVERGE-phase family; data-model.md §1).
- [X] T010 [US1] Add one data-driven grammar test in `editors/vscode/test/grammar.test.ts`
      that iterates the full 138-name list (`data-model.md` §1, copied or imported as a
      plain array in the test file) and asserts `NAME(x)` scopes as `support.function` for
      every single one — closes the gap the original draft's spot-check-only coverage left
      (this task exists specifically because SC-001 promises *every* name is verified, and
      T003-T009 alone only spot-check ~13 of 138). Depends on T003-T009 landing first only
      in the sense that it's the broader net catching the same pattern; no file conflict.

**Checkpoint**: User Story 1 is independently testable — `npm test` in `editors/vscode/`
passes T003-T010.

---

## Phase 4: User Story 2 - A non-call position keeps its ordinary color (Priority: P2)

**Goal**: A recognized function name never renders `support.function.drut` unless `(`
immediately follows (spec.md FR-006, Edge Cases).

**Independent Test**: Tokenize each line below and confirm the named token's scope does
NOT include `support.function`.

- [X] T011 [P] [US2] Add grammar test in `editors/vscode/test/grammar.test.ts`:
      `MAX = 100` — `MAX` does not scope as `support.function` (no following `(`; spec.md
      User Story 2 Acceptance Scenario 1; quickstart.md §2).
- [X] T012 [P] [US2] Add grammar test in `editors/vscode/test/grammar.test.ts`: `BESTJRNY`
      used bare, with no trailing `(...)` — does not scope as `support.function`, validating
      the deliberate exclusion of parenthesis-less skim values from the 138-name list
      (data-model.md §1; research.md §2; quickstart.md §2).
- [X] T013 [P] [US2] Add grammar test in `editors/vscode/test/grammar.test.ts`: a
      function-shaped substring inside a quoted string, e.g.
      `PRINT LIST='calling REPLACESTR(x) here'` — the text inside the quotes does not scope
      as `support.function` (protects the string-safety guarantee `#pair-values` already
      documents for itself; `#function-calls` is top-level-only, same as every other
      word-list pattern in this grammar).

**Checkpoint**: User Stories 1 AND 2 both pass independently — `npm test` in
`editors/vscode/` is fully green.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Regression re-proof and project bookkeeping.

- [X] T014 Run `npm test` in `editors/vscode/` (full `grammar.test.ts` suite, plus
      `formatOnSave.test.ts`/`binaryBootstrap.test.ts` already in the `test` script) and
      confirm zero regressions on every pre-existing check (spec.md SC-003).
- [X] T015 Manual spot-check: open
      `crates/voyager-core/tests/fixtures/valid/real_corpus/InputProcessing/1_InputSetup.s`
      in an Extension Development Host (`F5` from `editors/vscode/`) and confirm line 118
      (`if (STRLEN(TRIM(@SEGIDExField@))>0)`) renders `STRLEN`/`TRIM` in the function color
      (quickstart.md §4).
- [X] T016 [P] Update `ROADMAP.md`'s "Resolved queued items" log and `CHANGELOG.md`'s
      `## [Unreleased]` section with this feature's summary, matching the entry style
      already used for `023-range-dash-spacing`.

---

## Dependencies & Execution Order

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2, T002)**: Depends on Setup. BLOCKS every test task in Phase 3/4
  (a test asserting a scope the grammar doesn't yet produce will fail, by design — this is
  the one task where "write the test first" doesn't apply, since the pattern IS the fix
  both stories test).
- **User Story 1 (Phase 3)**: Can start once T002 lands. No dependency on US2.
- **User Story 2 (Phase 4)**: Can start once T002 lands. No dependency on US1 (independent
  negative-case coverage of the same pattern).
- **Polish (Phase 5)**: Depends on Phase 3 + Phase 4 both being green.

## Parallel Example: Phase 3 + Phase 4 together

```bash
# T002 (Foundational) must land first. After that, every test task below is [P] --
# different assertions in the same file, no inter-task dependency:
Task: "RIGHTSTR(TRIM(RouteName),1) both scope as support.function"
Task: "STRLEN(TRIM(@SEGIDExField@)) both scope, @var@ unaffected"
Task: "REPLACESTR on assignment RHS scopes as support.function"
Task: "ROUND scopes, _L.S_Angle does not"
Task: "CmpNumRetNum case-insensitive scopes"
Task: "SUBSTR/ARCSIN (no corpus evidence) still scope"
Task: "GAPCHANGEAVE(3)/GAPCHANGEMAX(3) CONVERGE-phase example scopes"
Task: "all 138 names, data-driven, scope as support.function"
Task: "MAX = 100 does NOT scope"
Task: "bare BESTJRNY does NOT scope"
Task: "function name inside a quoted string does NOT scope"
```

## Implementation Strategy

### MVP First (User Story 1 only)

1. Complete Phase 1 (trivial) + Phase 2 (T002 — the actual fix).
2. Complete Phase 3 (T003-T010) — validates the bug report is fixed, and (via T010) that
   every one of the 138 recognized names actually renders correctly, not just a sample.
3. **STOP and VALIDATE**: `npm test` green for T003-T010 is already a shippable MVP (every
   Acceptance Scenario in spec.md's User Story 1 passes).

### Incremental Delivery

1. Phase 1 + Phase 2 → grammar pattern exists.
2. Phase 3 → User Story 1 proven → could ship here.
3. Phase 4 → User Story 2 proven (regression protection) → ship.
4. Phase 5 → full regression re-proof + bookkeeping → merge.

## Notes

- No new file is created — every task edits one of the two files in plan.md's Project
  Structure (`drut.tmLanguage.json`, `grammar.test.ts`).
- [P] tasks in Phase 3/Phase 4 are independent `check(...)` assertions within the same
  `grammar.test.ts` file — "parallel" here means no ordering dependency between them, not
  literal concurrent file edits (a human or agent still adds them one at a time to the same
  file, same as `023-range-dash-spacing`'s own Rust unit-test tasks did).
