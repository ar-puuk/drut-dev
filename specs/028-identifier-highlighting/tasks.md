---

description: "Task list for Data-Reference & User-Variable Highlighting (028-identifier-highlighting)"
---

# Tasks: Data-Reference & User-Variable Highlighting

**Input**: Design documents from `/specs/028-identifier-highlighting/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/identifier-highlighting.md, quickstart.md

**Tests**: Not explicitly requested in `spec.md`, but included anyway — this project's
established convention (`026`/`027` before it) always adds `grammar.test.ts`/
`highlightCustomization.test.ts` coverage before merge.

**Organization**: Tasks are grouped by user story (spec.md: US1 `dataReferences` P1, US2
`userVariables` P2), preceded by a Foundational phase (the two line-scoped
`Label`/`ShellEscape` exclusion patterns both stories' FR-004a depends on) and followed
by a Polish phase.

## Path Conventions

Single existing project, `editors/vscode/` (plan.md Project Structure) — no new
directory, no other crate touched.

---

## Phase 1: Setup

**Purpose**: Confirm the existing dev environment; no new dependencies for this feature.

- [X] T001 Confirm `npm install && npm run compile` are clean on the base branch before
      starting (plan.md: no new dependency).

---

## Phase 2: Foundational (Blocking Prerequisite)

**Purpose**: The `Label`/`ShellEscape` line-scoped exclusion patterns both user stories'
FR-004a depends on — without these, the new catch-all in Phase 4 (and, to a lesser
degree, the family match in Phase 3) would reach into non-Voyager-syntax content.

**⚠️ CRITICAL**: US2's catch-all pattern in particular cannot be safely added until this
lands.

- [X] T002 In `editors/vscode/syntaxes/drut.tmLanguage.json`'s `repository`, add
      `#shell-escape` (`match: "^[ \t]*(\*.*)$"`, scope
      `meta.embedded.shell-escape.drut`) and `#label` (`match: "^[ \t]*(:)[ \t]*
      ([A-Za-z_][A-Za-z0-9_]*)"`, scope `entity.name.label.drut`) as two new whole-line/
      whole-shape `match` patterns (research.md §5), and include both in the top-level
      `patterns` array immediately after `#comments` (before `#strings`).

**Checkpoint**: `npm run compile` succeeds; a `ShellEscape`/`Label` line now tokenizes
under its own new scope. User story test tasks can now be added.

---

## Phase 3: User Story 1 - Data-reference family highlighted everywhere (Priority: P1) 🎯 MVP

**Goal**: `DBA`/`MI`/`MW`/... render in one consistent, distinct color regardless of
position — function-call argument, `LOOP`/`RUN`/`PROCESS` opener line, or plain
expression (spec.md Acceptance Scenarios 1-4).

**Independent Test**: Tokenize a script containing the family both after `=` and inside
a function call via `grammar.test.ts`; assert both occurrences share one scope.

- [X] T003 [US1] In `editors/vscode/syntaxes/drut.tmLanguage.json`'s `repository`, add
      `#data-references` (`match:
      "(?i)(?<![A-Za-z0-9_])(MI|MO|MW|LI|LW|NI|NW|ZI|ZONES|Z|DBI|DBA|RO|A|B|I|J)(?=\\.|(?![A-Za-z0-9_]))"`,
      scope `variable.language.data-reference.drut`, research.md §2), and insert it into
      the top-level `patterns` array between `#function-calls` and `#pair-keywords` —
      the array-order precedence FR-003 depends on (research.md §3).
- [X] T004 [US1] In `editors/vscode/src/highlightCustomization.ts`, add `"dataReferences"`
      to the `HighlightCategory` union and `dataReferences:
      "variable.language.data-reference.drut"` to `CATEGORY_SCOPES` (data-model.md §1).
- [X] T005 [US1] Add `drut.highlight.dataReferences` to `editors/vscode/package.json`'s
      `contributes.configuration.properties` — same shape as the 9 existing
      `drut.highlight.*` string settings (data-model.md §3).
- [X] T006 [P] [US1] Add test in `test/grammar.test.ts`: `DBA` scopes as
      `variable.language.data-reference.drut` both in `X = DBA.2.field` and inside
      `ROUND(DBA.2.VOL[numrec])` — the exact reported gap (spec.md AS1).
- [X] T007 [P] [US1] Add test in `test/grammar.test.ts`: `DBI` in `LOOP NUMREC = counter,
      DBI.2.NUMRECORDS` scopes as `variable.language.data-reference.drut` (spec.md AS2).
- [X] T008 [P] [US1] Add test in `test/grammar.test.ts`: `ZONES` in `RUN PGM=MATRIX
      ZONES=5` scopes as `variable.language.data-reference.drut`, not
      `variable.parameter.drut` (spec.md AS4, FR-003).
- [X] T009 [P] [US1] Add test in `test/grammar.test.ts`: inside a `ShellEscape` line
      (`*copy A B`), `A`/`B` do **not** scope as `variable.language.data-reference.drut`
      — the whole line scopes as `meta.embedded.shell-escape.drut` instead (FR-004a).
- [X] T010 [US1] In `test/highlightCustomization.test.ts`, extend the existing
      data-driven "every `HighlightCategory` value produces a correctly-scoped rule"
      test to include `dataReferences` (spec.md AS3, SC-003).

**Checkpoint**: User Story 1 is independently testable and shippable — `npm test`
(grammar) and `npx ts-node test/highlightCustomization.test.ts` both pass for
`dataReferences` alone, with User Story 2 not yet started.

---

## Phase 4: User Story 2 - User-defined variables highlighted consistently (Priority: P2)

**Goal**: A user-defined identifier (e.g. `_BNode`) renders consistently as an expression
operand regardless of position, without reclassifying any name already owned by a more
specific category (spec.md Acceptance Scenarios 1-3).

**Independent Test**: Tokenize `LINKID = _ANode + '_' + _BNode` via `grammar.test.ts`;
assert `_BNode` gets the new scope and no previously-scoped token changes scope.

- [X] T011 [US2] In `editors/vscode/syntaxes/drut.tmLanguage.json`'s `repository`, add
      `#user-identifiers` (`match: "(?<![A-Za-z0-9_])[A-Za-z_][A-Za-z0-9_]*(?![A-Za-z0-9_])"`,
      scope `variable.other.identifier.drut`, research.md §4), and append it as the
      **last** entry in the top-level `patterns` array (after `#punctuation`) — array
      position is the entire filtering mechanism.
- [X] T012 [US2] In `editors/vscode/src/highlightCustomization.ts`, add
      `"userVariables"` to the `HighlightCategory` union and `userVariables:
      "variable.other.identifier.drut"` to `CATEGORY_SCOPES` (data-model.md §1).
- [X] T013 [US2] Add `drut.highlight.userVariables` to `editors/vscode/package.json`'s
      `contributes.configuration.properties` (data-model.md §3).
- [X] T014 [P] [US2] Add test in `test/grammar.test.ts`: in `LINKID = _ANode + '_' +
      _BNode`, `_BNode` scopes as `variable.other.identifier.drut` — the exact reported
      gap (spec.md AS1). Also assert `_ANode` keeps scoping as `constant.other.drut`
      (`values`/`pairValues`, unchanged) — the documented `=`-adjacency trade-off
      (spec.md Assumptions), not a regression.
- [X] T015 [P] [US2] Add test in `test/grammar.test.ts`: a recognized control word,
      statement word, function-call name (in call position), pair-keyword name, pair
      value, and data-reference name each individually still scope under their own
      existing category, never `variable.other.identifier.drut` (spec.md AS3, FR-004's
      full exclusion list — comprehensive negative check).
- [X] T016 [P] [US2] Add test in `test/grammar.test.ts`: `:STEP0` (a `Label`) scopes as
      `entity.name.label.drut`, not `variable.other.identifier.drut` (FR-004a).
- [X] T017 [US2] In `test/highlightCustomization.test.ts`, extend the same data-driven
      test from T010 to also include `userVariables` (spec.md AS2, SC-003).
- [X] T018 [US2] Add test in `test/grammar.test.ts`: inside a quoted string (e.g.
      `PRINT LIST='DBA and _BNode'`), neither a data-reference-shaped nor a generic-
      identifier-shaped substring scopes as `variable.language.data-reference.drut` or
      `variable.other.identifier.drut` — the whole quoted run stays
      `string.quoted.single.drut` (FR-008; `/speckit-analyze` finding E1 — the
      mechanism is structurally inherited from `#strings`' existing nesting,
      research.md §2, but had no direct regression test for the two new categories).

**Checkpoint**: User Stories 1 AND 2 both pass independently — the full `npm test` +
`highlightCustomization.test.ts` suite is green.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Full regression re-proof and project bookkeeping.

- [X] T019 Run `npm test` in `editors/vscode/` (grammar + formatOnSave + binaryBootstrap
      + highlightCustomization, including this feature's new cases) and confirm zero
      regressions on every pre-existing check — in particular that none of the 9
      original `drut.highlight.*` categories or `drut.highlight.namedVariables` changed
      behavior (spec.md SC-004).
- [ ] T020 Manual spot-check per `quickstart.md` §4 in a real Extension Development Host
      (`F5`), ideally against an excerpt of the actual production script that surfaced
      this feature — confirm all three originally-reported symptoms (unhighlighted `DBA`
      in a function call, uncapitalized `DBI` on a `LOOP` line — already fixed by the
      separate `voyager-core` casing bug fix already applied in this working tree, not
      part of this feature's own task list — and inconsistently-highlighted
      `_ANode`/`_BNode`) read correctly now.
- [X] T021 [P] Update `ROADMAP.md`'s "Resolved queued items" log and `CHANGELOG.md`'s
      `## [Unreleased]` section with this feature's summary, matching the entry style
      already used for `026`/`027`.

---

## Dependencies & Execution Order

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2, T002)**: Depends on Setup. BLOCKS T009 (US1) and T011/T016
  (US2) — the `ShellEscape`/`Label` exclusion tests/pattern can't pass or be added
  meaningfully before it lands.
- **User Story 1 (Phase 3)**: Can start once Phase 2 lands. No dependency on US2.
- **User Story 2 (Phase 4)**: Can start once Phase 2 lands. Independent of US1's own
  pattern/setting, but T015's negative check is strongest once US1's `#data-references`
  pattern (T003) already exists to check against — sequence Phase 4 after Phase 3 in
  practice even though there's no hard blocking dependency.
- **Polish (Phase 5)**: Depends on Phase 3 + Phase 4 both being green.

## Parallel Example: Phase 3 + Phase 4 test tasks

```bash
# T002 (Foundational) must land first. T003-T005 / T011-T013 (pattern + wiring) are
# sequential within their own story (same file, ordered edits). Once each story's
# wiring is in place, its own test tasks run in parallel:
Task: "DBA scopes consistently after = and inside ROUND(...)"
Task: "DBI on a LOOP opener bound expression scopes correctly"
Task: "ZONES precedence: dataReferences wins over pairKeywords"
Task: "data-reference family inside a ShellEscape line is excluded"
Task: "_BNode scopes as userVariables; _ANode's existing scope is unchanged"
Task: "every other category's names never scope as userVariables"
Task: "a Label declaration scopes as entity.name.label, not userVariables"
```

## Implementation Strategy

### MVP First (User Story 1 only)

1. Complete Phase 1 (trivial) + Phase 2 (T002 — the shared exclusion mechanism).
2. Complete Phase 3 (T003-T010) — validates `dataReferences` works end to end.
3. **STOP and VALIDATE**: the grammar + unit-test suite passing is already a shippable
   MVP (every Acceptance Scenario in spec.md's User Story 1 passes) — closes the more
   evidence-backed, narrower of the two reported gaps on its own.

### Incremental Delivery

1. Phase 1 + Phase 2 → shared exclusion mechanism exists.
2. Phase 3 → User Story 1 (`dataReferences`) proven → could ship here.
3. Phase 4 → User Story 2 (`userVariables`) proven → ship.
4. Phase 5 → manual real-editor proof + bookkeeping → merge.

## Notes

- No `voyager-core`/`drut-config`/`drut-cli`/`drut-mcp`/`drut-lsp` file is touched by any
  task here (spec.md FR-007) — everything lives under `editors/vscode/`.
- `extension.ts` is not modified by any task — `applyHighlightCustomizations` already
  iterates `CATEGORY_SCOPES` generically (research.md §1), so both new categories wire
  up for free once T004/T012 land.
- `ensureVariableColorCustomization`/`decideVariableColorSync` (`027`,
  `drut.highlight.namedVariables`) are not modified by any task here.
