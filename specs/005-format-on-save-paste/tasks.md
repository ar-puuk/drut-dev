---

description: "Task list for Format-On-Save and Format-On-Paste"
---

# Tasks: Format-On-Save and Format-On-Paste

**Input**: Design documents from `/specs/005-format-on-save-paste/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/,
quickstart.md (all present)

**Tests**: Included — automated where the underlying behavior is a pure
function (drut-lsp's diff/range logic, the extension's injection-decision
predicate), manual smoke tests where the behavior can only be observed
through real VS Code UI/settings persistence (no automated VS Code
extension-activation harness exists in this repo — `editors/vscode`'s
existing `npm test` only exercises grammar tokenization via
`vscode-textmate`), matching 003/004's own established standard for
anything touching real editor UI.

**Organization**: Tasks are grouped by user story (US1–US3, P1–P3 per
spec.md). US1 (format-on-save) and US2 (format-on-paste) touch fully
disjoint files (`editors/vscode/src/extension.ts` vs.
`crates/drut-lsp/src/range_formatting.rs`) and have no dependency on each
other — either can be built first. **US3 is the one story with no separate
implementation task**: its entire mechanism (respecting an existing
override) is built as an inseparable part of US1's own implementation
(contracts/extension-settings.md's single-function design) — US3 exists as
its own story purely to give that specific guarantee (FR-006) its own
explicit, story-level test coverage, both at the unit level (T002, listed
under US1 since it tests US1's own function) and at the real-VS-Code level
(T010, listed under US3, since that's the level spec.md's User Story 3
actually describes).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an
  incomplete sibling task)
- **[Story]**: US1–US3 — omitted for Setup/Polish tasks
- Every task names its exact file path

## Path Conventions

No new crate, no new npm package (plan.md Structure Decision):

- `crates/drut-lsp/` — existing crate; US2 adds
  `src/range_formatting.rs` (code + its own `#[cfg(test)] mod tests`, one
  file, mirroring `formatting.rs`'s own existing shape exactly — **not**
  `drut-mcp`'s per-tool-separate-contract-file convention, which solved a
  different problem (four independent tools in one crate) that doesn't
  apply here) and touches `src/lib.rs` (capability declaration + dispatch
  arm).
- `editors/vscode/` — existing extension; US1 adds one function to the
  existing single-file `src/extension.ts` (mirroring
  `ensureVariableColorCustomization`'s own placement) plus a new test file
  `test/formatOnSave.test.ts` (mirrors `test/grammar.test.ts`'s existing
  ts-node-run convention). US2 adds a short README instruction, no code.

---

## Phase 1: Setup

**Purpose**: Confirm a clean baseline before any change, so Polish's final
re-run has a true "did this feature regress anything" comparison.

- [ ] T001 Confirm baseline: `cargo build --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings` both clean;
      `cd editors\vscode; npm install; npm run compile; npm test` all
      clean.

**Checkpoint**: Baseline confirmed clean. No Foundational phase follows —
US1 (`editors/vscode/src/extension.ts`) and US2
(`crates/drut-lsp/src/range_formatting.rs` + `src/lib.rs`) share no file
and no code dependency, so there is no shared prerequisite to build before
either can start (unlike 004, where all four stories needed the same
`drut mcp` CLI entry point and `ScriptSource` first).

---

## Phase 2: User Story 1 - Automatic reformatting on save (Priority: P1) 🎯 MVP

**Goal**: The extension auto-enables `editor.formatOnSave` for `.s`/`.block`
files on first activation, one-time, via `drut-lsp`'s already-shipped
`textDocument/formatting` capability (no server-side change needed).

**Independent Test**: Open a `.s` file with a misindented body statement,
save it without running "Format Document" first, and confirm the
misindentation is corrected automatically (spec.md US1 Acceptance
Scenarios).

### Tests for User Story 1

- [ ] T002 [P] [US1] Unit tests for the injection-decision predicate in a
      new `editors/vscode/test/formatOnSave.test.ts` (ts-node-run, mirrors
      `test/grammar.test.ts`'s existing convention): `shouldInjectFormatOnSave`
      returns `true` when not yet injected and no existing language-scoped
      override is present; returns `false` when already injected
      (`workspaceState` gate); returns `false` when an existing
      language-scoped override is already present, regardless of its value
      — this third case is FR-006/US3's core guarantee, covered here as
      real unit coverage even though US3 also gets its own real-VS-Code
      verification (T010).

### Implementation for User Story 1

- [ ] T003 [US1] In `editors/vscode/src/extension.ts`, add the exported
      pure predicate `shouldInjectFormatOnSave(alreadyInjected: boolean,
      existingWorkspaceLanguageValue: unknown): boolean` and
      `ensureFormatOnSaveEnabled(context: vscode.ExtensionContext): Promise<void>`
      (contracts/extension-settings.md) — uses
      `vscode.workspace.getConfiguration(undefined, { languageId: "drut-voyager" })`
      + `WorkspaceConfiguration.update(..., ConfigurationTarget.Workspace, /* overrideInLanguage */ true)`
      (research.md §3), `config.inspect("editor.formatOnSave").workspaceLanguageValue`
      to detect an existing override, and the `drutFormatOnSaveInjected`
      `workspaceState` key, mirroring `ensureVariableColorCustomization`'s
      existing `try`/`catch`-wrapped, never-fails-activation shape.
      Depends on T002 (test written first, expected to fail until this
      task lands).
- [ ] T004 [US1] Wire `void ensureFormatOnSaveEnabled(context);` into
      `activate()` in `editors/vscode/src/extension.ts`, alongside the
      existing `ensureVariableColorCustomization(context)` call. Depends
      on T003.
- [ ] T005 [US1] Manual smoke test — quickstart.md step 5, first half
      (steps 1–3 only: package/install the extension, open a fresh
      workspace with no prior `.vscode/settings.json`, save a misindented
      `.s` file without formatting it manually first). Report what was
      actually observed — the correction happening automatically, and
      `.vscode/settings.json` now containing
      `"[drut-voyager]": { "editor.formatOnSave": true }` — not just that
      the extension packaged successfully. Depends on T004.

**Checkpoint**: Format-on-save fully functional and independently
testable — suggested MVP stopping point.

---

## Phase 3: User Story 2 - Automatic reformatting of pasted content (Priority: P2)

**Goal**: A new `drut-lsp` capability, `textDocument/rangeFormatting`,
backed by the same `voyager_core::format` call as whole-document
formatting, serving VS Code's `editor.formatOnPaste`. Ships opt-in,
documented only — the extension never auto-enables it (Clarification Q1).

**Independent Test**: Copy a block-shaped fragment with wrong indentation,
paste it into a document at a different nesting depth, enable
`editor.formatOnPaste` for `.s`/`.block` files by hand, and confirm the
pasted text is reindented to match its new context immediately after the
paste (spec.md US2 Acceptance Scenarios).

### Tests + Implementation for User Story 2

(Combined per file, matching `drut-lsp`'s own established one-module,
code-plus-`#[cfg(test)]` convention — see `formatting.rs`, `hover.rs`,
`position.rs` — rather than a separate contract-test file.)

- [ ] T006 [US2] Implement `diff_lines`/`filter_to_range`/`handle` in a new
      `crates/drut-lsp/src/range_formatting.rs` (data-model.md §1,
      contracts/range-formatting-api.md), plus its own
      `#[cfg(test)] mod tests` covering all five cases
      contracts/range-formatting-api.md's Tests section specifies:
      `misindented_line_within_range_is_corrected`,
      `already_formatted_document_returns_empty_edit_list`,
      `unopened_document_returns_none`,
      `change_outside_requested_range_is_not_returned`,
      `change_at_exact_range_boundary_is_included`.
- [ ] T007 [US2] In `crates/drut-lsp/src/lib.rs`: add
      `pub mod range_formatting;`, add
      `document_range_formatting_provider: Some(lsp_types::OneOf::Left(true)),`
      to `server_capabilities()` alongside the existing
      `document_formatting_provider` line, and add a
      `RangeFormatting::METHOD` arm to `handle_request`'s `match`,
      structurally identical to the existing `Formatting::METHOD` arm
      (contracts/range-formatting-api.md). Depends on T006.
- [ ] T008 [P] [US2] Add the `editor.formatOnPaste` opt-in instructions to
      `README.md` per contracts/extension-settings.md — the
      `"[drut-voyager]": { "editor.formatOnPaste": true }` snippet plus one
      sentence explaining pasted script text is reformatted to match its
      surrounding indentation once the setting is on. No dependency on
      T006/T007 — documents an already-designed setting, not the code.
- [ ] T009 [US2] Manual smoke test — quickstart.md step 6 (paste with
      `formatOnPaste` off, confirm no change; enable it by hand per T008's
      instruction; paste again, confirm reindentation; paste
      already-correct content a third time, confirm no further edit —
      idempotence). Depends on T007, T008.

**Checkpoint**: Format-on-save and format-on-paste both independently
functional.

---

## Phase 4: User Story 3 - Author stays in control of format-on-save (Priority: P3)

**Goal**: Confirm, at the real-VS-Code level, that an author who disables
the auto-enabled `editor.formatOnSave` setting stays disabled across
workspace reopens — FR-006's guarantee, already built into T003's function
and already unit-tested (T002's third case) at the predicate level.

**Independent Test**: With format-on-save auto-enabled (US1), turn
`editor.formatOnSave` off for `.s`/`.block` files, close and reopen the
workspace, and confirm it stays off (spec.md US3 Acceptance Scenarios).

- [ ] T010 [US3] Manual smoke test — quickstart.md step 5, second half
      (steps 4–5: set
      `"[drut-voyager]": { "editor.formatOnSave": false }` by hand in the
      same workspace T005 used, close and reopen the workspace, introduce
      a new misindentation, save, confirm it is **not** auto-corrected).
      Report what was actually observed. Depends on T005 (needs US1's
      auto-enable already demonstrated working in the "on" direction
      first, in the same workspace, so this step is a genuine "then the
      user turned it off" continuation, not an isolated setup).

**Checkpoint**: All three user stories independently validated — US1 and
US2's own functional behavior, and US3's confirmation that US1's mechanism
doesn't fight a user's explicit choice.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Whole-workspace regression checks and documentation, once all
three stories are complete.

- [ ] T011 `cargo test --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings`, both clean —
      confirms this feature introduces zero regressions anywhere in the
      workspace, including the `lib.rs` dispatch-`match` change (T007).
- [ ] T012 [P] Full-corpus diagnostic regression check (quickstart.md
      step 4): `$env:DRUT_CORPUS_PATH = "..."`;
      `cargo test -p drut-lsp --test diagnostics_corpus -- --ignored` —
      still 161/161 clean, confirming the `lib.rs` `match` addition (T007)
      disturbed nothing already routed through it.
- [ ] T013 [P] `cd editors\vscode; npm run compile; npm test` — confirms
      T002's new predicate tests pass alongside the existing grammar
      tests, and the extension still compiles cleanly with T003/T004's
      additions.
- [ ] T014 [P] Update `ROADMAP.md`'s format-on-save and format-on-paste
      status lines from "not started" to reflect completion, per this
      feature's own shipped state.

**Checkpoint**: Feature-complete against spec.md; `ROADMAP.md` reflects
reality.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **User Story 1 (Phase 2)**: Depends on Setup only. No dependency on US2.
  Suggested MVP.
- **User Story 2 (Phase 3)**: Depends on Setup only. No dependency on US1's
  own implementation — fully disjoint files.
- **User Story 3 (Phase 4)**: **Depends on US1 (T003–T005)** — this is the
  one real cross-story dependency in this feature, stated explicitly
  rather than glossed over: US3 has no implementation of its own to be
  independent *of*; its manual verification (T010) specifically continues
  from the same workspace T005 already set up, to prove the "off stays
  off" half of the same mechanism T005 proved the "on" half of.
- **Polish (Phase 5)**: Depends on all three stories being complete (T011
  needs the whole workspace; T012 needs T007's `lib.rs` change; T013 needs
  T002/T003's extension changes; T014 needs the feature's final state
  known).

### Within Each User Story

- US1: test (T002) before implementation (T003) — predicate unit-tested
  first, expected to fail until T003 lands; T004 (wiring) after T003;
  T005 (manual) last, needs the real built extension.
- US2: implementation + its own tests together in one file (T006, matching
  `drut-lsp`'s established per-module convention) before `lib.rs` wiring
  (T007, same-file dependency on `range_formatting.rs` existing);
  README (T008) independent of both; manual smoke test (T009) last, needs
  both T007 and T008.
- US3: single manual task (T010), depends on US1's T005 specifically (see
  Phase Dependencies above).

### Parallel Opportunities

- Once Setup (T001) completes, **US1's and US2's own work is genuinely
  parallel** — T002/T003/T004 (`editors/vscode/src/extension.ts` +
  `test/formatOnSave.test.ts`) share zero files with T006/T007/T008
  (`crates/drut-lsp/src/range_formatting.rs` + `src/lib.rs` +
  `README.md`). US3 (T010) cannot start until US1's T005 lands (see
  above), so it is not part of this parallel window.
- Within US2, T008 (README) is parallel with T006/T007 (different file,
  no dependency).
- Within Polish, T012/T013/T014 are parallel with each other (different
  files/scopes); T011 is the whole-workspace gate all three implicitly
  depend on having a clean baseline for, so it's listed first though not
  strictly blocking in tooling terms.

---

## Parallel Example: Once Setup Completes

```bash
# Launch US1 and US2's own work together (fully disjoint files):
Task: "US1: formatOnSave.test.ts predicate tests + extension.ts implementation + activate() wiring"
Task: "US2: range_formatting.rs implementation+tests + lib.rs wiring + README opt-in instructions"

# US3 (T010) starts only once US1's manual smoke test (T005) is done —
# not part of this parallel window.
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: User Story 1 (format-on-save)
3. **STOP and VALIDATE**: T002 passes; T005's manual smoke test confirms
   real auto-correction on save in a real VS Code window
4. Demo/ship if ready — format-on-save alone is a complete, independently
   useful capability

### Incremental Delivery

1. Setup → baseline confirmed clean
2. Add US1 (format-on-save) → test independently → demo (MVP!)
3. Add US2 (format-on-paste) → test independently → demo
4. Add US3 (override-respecting confirmation) → test independently → demo
5. Polish (Phase 5) → feature-complete, ready for merge

---

## Notes

- [P] tasks = different files, no dependency on an incomplete sibling.
- [Story] label maps task to specific user story for traceability.
- US3 is deliberately implementation-free — its guarantee (FR-006) is a
  property of US1's own function, not separate code; see this file's
  Organization note and the Phase Dependencies section above for why that
  doesn't collapse US3 into US1 as a *story* (spec.md still frames it as
  its own priority-ordered, independently valuable behavior worth its own
  explicit verification).
- `drut-lsp` additions (US2) follow that crate's own established
  one-module-per-capability convention (code + tests together), not
  `drut-mcp`'s per-tool-separate-file convention — different crates,
  different established shapes; this file matches each to its own
  existing precedent rather than applying one convention uniformly.
- Commit after each task or logical group.
- Stop at any checkpoint to validate a story independently before
  continuing.
