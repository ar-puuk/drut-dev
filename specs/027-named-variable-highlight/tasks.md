---

description: "Task list for @name@ Variable Highlight Color Customization (027-named-variable-highlight)"
---

# Tasks: `@name@` Variable Highlight Color Customization

**Input**: Design documents from `/specs/027-named-variable-highlight/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/named-variable-highlight.md, quickstart.md

**Tests**: Included — same standalone-`ts-node` convention `026`/`highlightCustomization.test.ts` already established, and this feature's central risk (not regressing `026`'s existing behavior) specifically needs a dedicated regression test, not just new-feature tests.

## Path Conventions

Single existing project, `editors/vscode/` — no new directory, no other crate touched.

---

## Phase 1: Setup

- [X] T001 Confirm `npm run compile` and `npm test` are clean on `026`'s already-merged
      state before starting.

---

## Phase 2: Foundational (Blocking Prerequisite)

- [X] T002 In `editors/vscode/src/highlightCustomization.ts`, add
      `VariableColorSyncState`, `VariableColorDecision`, `DEFAULT_VARIABLE_COLOR`, and
      `decideVariableColorSync` (data-model.md §1) — pure, zero `vscode` import.
- [X] T003 Add `drut.highlight.namedVariables` to `editors/vscode/package.json`'s
      `contributes.configuration.properties` — same shape as `026`'s 9 settings, plus a
      `markdownDescription` noting the default (`#4EC9B0`) and that it colors `@name@`
      substitution specifically.
- [X] T004 In `editors/vscode/src/extension.ts`, refactor `ensureVariableColorCustomization`
      to call `decideVariableColorSync` (data-model.md §2): add the
      `VARIABLE_COLOR_LIVE_SYNC_KEY` workspaceState key alongside the existing
      `VARIABLE_COLOR_INJECTED_KEY`, read `drut.highlight.namedVariables`'s Global
      value, and act on the returned `VariableColorDecision`. Also call
      `ensureVariableColorCustomization(context)` from the existing
      `onDidChangeConfiguration` handler (already gated on
      `e.affectsConfiguration("drut.highlight")`), alongside the existing
      `applyHighlightCustomizations()` call.

**Checkpoint**: `npm run compile` succeeds.

---

## Phase 3: User Story 1 - Recolor `@name@` (Priority: P1) 🎯 MVP

- [X] T005 [P] [US1] Add test in `test/highlightCustomization.test.ts`:
      `decideVariableColorSync` with `configuredColor` set (any prior state) returns
      `shouldWrite: true` (unless already matching), `value` equal to the configured
      color, `nextState.liveSyncActive: true` (spec.md AS2).
- [X] T006 [P] [US1] Add test: `decideVariableColorSync` with `configuredColor` set to
      the SAME value as `existingRuleValue` returns `shouldWrite: false` (no redundant
      write) but still `nextState.liveSyncActive: true`.
- [X] T007 [P] [US1] Add test: `liveSyncActive: true`, `configuredColor` now `undefined`
      → `shouldWrite: true`, `value: DEFAULT_VARIABLE_COLOR`,
      `nextState.liveSyncActive: false` (spec.md AS3, FR-005).
- [X] T008 [P] [US1] Add test: fresh workspace (`alreadySeeded: false`,
      `liveSyncActive: false`, `existingRuleValue: undefined`, `configuredColor` already
      set) → `shouldWrite: true`, `value` equal to the configured color directly (not
      the default followed by a second write) — spec.md Edge Cases (synced-settings
      first-activation case).

**Checkpoint**: User Story 1 independently testable.

---

## Phase 4: User Story 2 (implicit regression guarantee) - Untouched workspaces are unaffected (Priority: P1)

**Goal**: `026`'s pre-existing behavior is byte-identical for any workspace that never
sets `drut.highlight.namedVariables` (spec.md SC-002) — this is as important as the new
feature itself, so it gets its own explicit test phase rather than being folded in.

- [X] T009 [P] [US2] Add test: `alreadySeeded: false`, `liveSyncActive: false`,
      `existingRuleValue: undefined`, `configuredColor: undefined` → `shouldWrite: true`,
      `value: DEFAULT_VARIABLE_COLOR` — today's exact original one-time-seed behavior,
      unchanged (spec.md AS1, SC-002).
- [X] T010 [P] [US2] Add test: `alreadySeeded: true`, `liveSyncActive: false`,
      `existingRuleValue: undefined` (user manually deleted it), `configuredColor:
      undefined` → `shouldWrite: false` — the rule stays deleted, never re-added
      (spec.md AS4, the specific regression this feature must not introduce).
- [X] T011 [P] [US2] Add test: same as T010 but `existingRuleValue` still present and
      unmodified (never deleted) → `shouldWrite: false` — no redundant write when
      nothing has changed and no override is configured.

**Checkpoint**: Both user stories pass — `npx ts-node test/highlightCustomization.test.ts`
is fully green.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [X] T012 Run `npm test` in `editors/vscode/` (full suite) and confirm zero
      regressions on every pre-existing check, including `026`'s own 26 checks.
- [ ] T013 Manual spot-check per `quickstart.md` §3 in a real Extension Development
      Host — cannot be automated from this environment; flag to the user as unverified
      by anything other than the pure-logic tests above.
- [X] T014 [P] Update `ROADMAP.md`'s "Resolved queued items" log and `CHANGELOG.md`'s
      `## [Unreleased]` section with this feature's summary, matching `026`'s entry
      style.

---

## Dependencies & Execution Order

- **Setup → Foundational → (US1 ‖ US2) → Polish**, same shape as `024`/`025`/`026`.
- US1 and US2 both exercise the same `decideVariableColorSync` function from different
  angles (new-feature correctness vs. no-regression) and have no dependency on each
  other.

## Notes

- No `voyager-core`/`drut-config`/`drut-cli`/`drut-mcp` file touched.
- `applyHighlightCustomizations` (026's function) is not modified — `namedVariables` is
  a separate code path (`ensureVariableColorCustomization`), not a 10th entry in
  `CATEGORY_SCOPES` (research.md §1 explains why it can't be).
