---

description: "Task list for Editor Highlight Color Customization (026-highlight-customization)"
---

# Tasks: Editor Highlight Color Customization

**Input**: Design documents from `/specs/026-highlight-customization/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/highlight-customization.md, quickstart.md

**Tests**: Not explicitly requested in `spec.md`, but included anyway — this project's
established convention (every prior extension-client feature, e.g.
`formatOnSaveDecision.ts`/`formatOnSave.test.ts`) always adds standalone-`ts-node`
coverage for pure logic before merge.

**Organization**: Tasks are grouped by user story (spec.md: US1 P1, US2 P2), preceded by
a Foundational phase (grammar split + the pure merge module + settings contributions
both stories need) and followed by a Polish phase.

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

**Purpose**: The grammar split and the pure category/merge logic every user story's
tests validate.

**⚠️ CRITICAL**: No user story task can be meaningfully tested until this is complete.

- [X] T002 In `editors/vscode/syntaxes/drut.tmLanguage.json`, rename `#statement-words`'s
      `"name"` from `support.function.drut` to `support.function.statement.drut`, and
      `#function-calls`'s `"name"` from `support.function.drut` to
      `support.function.builtin.drut` (research.md §5) — a pure rename, no match-pattern
      change to either pattern.
- [X] T003 Create `editors/vscode/src/highlightCustomization.ts`: the
      `HighlightCategory` type, `CATEGORY_SCOPES` table, `mergeHighlightRules`,
      `isEmptyTokenColorCustomizations`, and a small structural `deepEqual` helper
      (order-insensitive for object keys, order-sensitive for arrays — data-model.md
      §1-§3) — zero `vscode` import, mirrors `formatOnSaveDecision.ts`'s
      standalone-testable convention exactly. `deepEqual` backs T005's no-op guard
      (`/speckit-analyze` finding: skip the `editor.tokenColorCustomizations` write
      entirely when the computed result wouldn't actually change anything, so an
      activation for a user who never touches `drut.highlight.*` never rewrites
      `settings.json` at all).
- [X] T004 Add the 9 `drut.highlight.<category>` settings to `editors/vscode/
      package.json`'s `contributes.configuration.properties` (`type: "string"`, no
      default, a `markdownDescription` naming the underlying TextMate scope(s) and
      noting the expected CSS-color format) — same shape as the existing
      `drut.format.casing*` string-enum settings, but free-form string, not `enum`.
- [X] T005 In `editors/vscode/src/extension.ts`, add `applyHighlightCustomizations()`
      (data-model.md §4, mirrors `ensureVariableColorCustomization`'s try/catch/
      best-effort shape), including the `deepEqual`-backed no-op write guard, and call
      it once from `activate()`, plus register a `workspace.onDidChangeConfiguration`
      listener (added to `context.subscriptions`) that re-runs it whenever
      `e.affectsConfiguration("drut.highlight")` is true.

**Checkpoint**: `npm run compile` succeeds; user story test tasks can now be added.

---

## Phase 3: User Story 1 - A script author recolors one category (Priority: P1) 🎯 MVP

**Goal**: Setting/unsetting any `drut.highlight.<category>` correctly upserts/removes
its rule, independently of every other category, with no window reload needed (spec.md
Acceptance Scenarios 1-4).

**Independent Test**: Call `mergeHighlightRules` directly with various `desired` maps
and assert the resulting `TokenColorCustomizations`.

- [X] T006 [P] [US1] Add test in `test/highlightCustomization.test.ts`: `desired = {}`
      (nothing set) against an empty `current` returns something
      `isEmptyTokenColorCustomizations` reports true for — matches "unset is a strict
      no-op" (spec.md AS1, SC-002).
- [X] T007 [P] [US1] Add test: `desired = { functionCalls: "#FF6B35" }` against an empty
      `current` returns exactly one rule, `scope: "support.function.builtin.drut"`,
      `settings.foreground: "#FF6B35"` (spec.md AS2).
- [X] T008 [P] [US1] Add test: starting from the result of T007, calling
      `mergeHighlightRules` again with `desired = {}` removes that rule and returns an
      empty result (spec.md AS3 — reverts, doesn't strand).
- [X] T009 [P] [US1] Add test: `desired = { controlWords: "#C586C0", functionCalls:
      "#FF6B35" }` against an empty `current` returns exactly two independent rules,
      each with its own correct scope and color (spec.md AS4).
- [X] T010 [US1] Add one data-driven test iterating all 9 `HighlightCategory` values from
      `CATEGORY_SCOPES`, asserting each produces a rule with the correct scope(s) when
      set alone — closes the same "verify every category, not a sample" gap `024`'s
      T010 and `025`'s T010 each closed for their own lists (spec.md SC-001).
- [X] T011 [P] [US1] Add grammar test in `test/grammar.test.ts`: `functionCalls` and
      `statementWords` now scope as `support.function.builtin`/`support.function.
      statement` respectively (not the old shared `support.function.drut`) — proves the
      T002 split (spec.md SC-005).

**Checkpoint**: User Story 1 is independently testable — `npx ts-node test/
highlightCustomization.test.ts` and the grammar-split checks in `npm test` both pass.

---

## Phase 4: User Story 2 - Unrelated customizations are preserved (Priority: P2)

**Goal**: A pre-existing, unrelated `editor.tokenColorCustomizations` rule survives
every `drut.highlight.*` set/unset cycle unchanged (spec.md FR-004, SC-003).

**Independent Test**: Seed `current` with unrelated content, run `mergeHighlightRules`
through a set-then-unset cycle, assert the unrelated content is byte-for-byte unchanged
at every step.

- [X] T012 [P] [US2] Add test in `test/highlightCustomization.test.ts`: `current`
      contains an unrelated rule (e.g. `{ scope: "entity.name.tag.python", settings:
      {...} }`) plus an unrelated top-level key (e.g. a `"[Some Theme]"` override
      object); setting `drut.highlight.controlWords` adds the new rule while both
      pieces of unrelated content remain present, unmodified (spec.md AS1).
- [X] T013 [P] [US2] Add test: continuing from T012, unsetting
      `drut.highlight.controlWords` removes only that rule — the unrelated rule and the
      unrelated top-level key both remain (spec.md AS2).
- [X] T014 [P] [US2] Add test: a rule whose `scope` is an array combining one of drut's
      known scope names with an unrelated scope (e.g. `["keyword.control.drut",
      "keyword.other.foo"]`) is never touched by any `mergeHighlightRules` call — proves
      the exact-scope-set-match ownership test (research.md §4), not a
      substring/overlap match, is what's actually implemented.

**Checkpoint**: User Stories 1 AND 2 both pass independently — the full
`highlightCustomization.test.ts` suite is green.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Full regression re-proof and project bookkeeping.

- [X] T015 Run `npm test` in `editors/vscode/` (grammar + formatOnSave +
      binaryBootstrap + this feature's new suite) and confirm zero regressions on every
      pre-existing check.
- [ ] T016 Manual spot-check per `quickstart.md` §4 in a real Extension Development
      Host (`F5`) — confirm live recoloring with no window reload, and confirm an
      unrelated hand-written `editor.tokenColorCustomizations` rule survives a real
      set/unset cycle end to end (not just in the unit tests).
- [X] T017 [P] Update `ROADMAP.md`'s "Resolved queued items" log and `CHANGELOG.md`'s
      `## [Unreleased]` section with this feature's summary, matching the entry style
      already used for `024`/`025`.

---

## Dependencies & Execution Order

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2, T002-T005)**: Depends on Setup. BLOCKS every test task in
  Phase 3/4 (a test asserting a scope/merge result the code doesn't yet produce fails by
  design).
- **User Story 1 (Phase 3)**: Can start once Phase 2 lands. No dependency on US2.
- **User Story 2 (Phase 4)**: Can start once Phase 2 lands. No dependency on US1
  (independent preservation coverage of the same merge function).
- **Polish (Phase 5)**: Depends on Phase 3 + Phase 4 both being green.

## Parallel Example: Phase 3 + Phase 4 together

```bash
# T002-T005 (Foundational) must land first. After that:
Task: "empty desired against empty current is a no-op"
Task: "one category set upserts exactly one correct rule"
Task: "set then unset removes exactly that rule"
Task: "two categories set independently both apply"
Task: "all 9 categories individually produce correct rules, data-driven"
Task: "grammar: functionCalls/statementWords now scope independently"
Task: "unrelated rule + unrelated top-level key survive a set"
Task: "unrelated content survives the following unset too"
Task: "a scope-array only partially matching ours is never touched"
```

## Implementation Strategy

### MVP First (User Story 1 only)

1. Complete Phase 1 (trivial) + Phase 2 (T002-T005 — the actual mechanism).
2. Complete Phase 3 (T006-T011) — validates the core feature works.
3. **STOP and VALIDATE**: the pure-logic suite plus the grammar-split checks passing is
   already a shippable MVP (every Acceptance Scenario in spec.md's User Story 1 passes).

### Incremental Delivery

1. Phase 1 + Phase 2 → mechanism exists.
2. Phase 3 → User Story 1 proven → could ship here.
3. Phase 4 → User Story 2 proven (safety net for other extensions/manual tweaks) → ship.
4. Phase 5 → manual real-editor proof + bookkeeping → merge.

## Notes

- No `voyager-core`/`drut-config`/`drut-cli`/`drut-mcp`/`drut-lsp` file is touched by any
  task here (spec.md FR-007/FR-008) — everything lives under `editors/vscode/`.
- `ensureVariableColorCustomization`/`ensureFormatOnSaveEnabled` in `extension.ts` are
  not modified by any task — `applyHighlightCustomizations` (T005) is purely additive
  alongside them.
