---

description: "Task list for Function-Call Casing Normalization (025-function-casing)"
---

# Tasks: Function-Call Casing Normalization

**Input**: Design documents from `/specs/025-function-casing/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/function-casing.md, quickstart.md

**Tests**: Not explicitly requested in `spec.md`, but included anyway — Constitution
Principle III makes formatter-behavior test coverage (unit + golden-fixture) mandatory
for this feature, not optional.

**Organization**: Tasks are grouped by user story (spec.md: US1 P1, US2 P2), preceded by
a Foundational phase (the core recognition/casing mechanism both stories test) and
followed by adapter-surface wiring and polish phases (cross-cutting, not story-specific).

## Path Conventions

Existing crates: `crates/voyager-core/`, `crates/drut-config/`, `crates/drut-cli/`,
`crates/drut-mcp/`, `editors/vscode/` (plan.md Project Structure) — no new crate.

---

## Phase 1: Setup

**Purpose**: Confirm the existing dev environment; no new dependencies for this feature.

- [X] T001 Confirm `cargo build --workspace` and `cargo clippy --workspace --all-targets
      -- -D warnings` are clean on the base branch before starting (plan.md: no new
      dependency, zero-runtime-dependency guarantee unaffected).

---

## Phase 2: Foundational (Blocking Prerequisite)

**Purpose**: The core recognition + casing-edit mechanism every user story's tests
validate.

**⚠️ CRITICAL**: No user story task can be meaningfully tested until this is complete.

- [X] T002 Create `crates/voyager-core/src/function_call.rs`: `FunctionCallEntry`
      struct, `FUNCTION_CALL_ENTRIES` table (the 138 names from `024-function-call-
      highlighting/research.md` §2, ported verbatim — data-model.md §1), and
      `function_call_entries()`/`is_function_call_name()` (mirrors
      `data_reference.rs`'s `data_reference_entries()`/`is_data_reference_name()`
      exactly).
- [X] T003 In `function_call.rs`, add `FunctionCallOccurrence` and
      `function_call_occurrences(nodes, lines)`: a quote-aware token scan (mirrors
      `data_reference.rs`'s `collect_tokens`/`collect_statement`/`collect_block`/
      `collect` exactly), with the one additional condition unique to this category —
      the matched `Word` token must be immediately followed by a `(` `Punctuation`
      token with zero intervening whitespace (`tokens[i+1].span.start ==
      tokens[i].span.end`, research.md §4). Covers `Control` and `Assignment`
      statements; excludes `Label`/`ShellEscape` (data-model.md §2).
- [X] T004 Re-export the new module's public items from `crates/voyager-core/src/
      lib.rs` (`pub mod function_call;` + `pub use function_call::{...}`), mirroring
      the existing `data_reference` re-export line exactly (data-model.md §2).
- [X] T005 In `crates/voyager-core/src/format.rs`, add `function_calls:
      CasingConvention` to `CasingSettings` (data-model.md §3), extend the existing
      `!= CasingConvention::Preserve` gate to include it, and wire
      `function_call::function_call_occurrences(nodes, &char_lines)` into `render()`
      the same way `data_reference_occurrences` is already wired (data-model.md §4).

**Checkpoint**: `cargo build -p voyager-core` succeeds; user story test tasks can now be
added.

---

## Phase 3: User Story 1 - A script author normalizes function-name casing (Priority: P1) 🎯 MVP

**Goal**: `casing_function_calls` set to `Upper`/`Lower` rewrites every recognized
function call's casing correctly, regardless of statement position (spec.md Acceptance
Scenarios 1-4).

**Independent Test**: Format each fixture below and assert the resulting text.

- [X] T006 [P] [US1] Add unit test in `function_call.rs` (or `format.rs`'s existing
      casing test module): `RouteName = replacestr(RouteName,'-','',0)` under
      `function_calls: Upper` renders `RouteName = REPLACESTR(RouteName,'-','',0)` —
      string arguments untouched (spec.md AS1).
- [X] T007 [P] [US1] Add unit test: `if (rightstr(trim(RouteName),1)='-')` under
      `function_calls: Lower` renders both `rightstr`/`trim` lowercase — nested
      function calls both rewritten (spec.md AS2).
- [X] T008 [P] [US1] Add unit test: `casing_function_calls` unset (`Preserve`) leaves a
      mixed-case function-call fixture byte-identical (spec.md AS3; contract "strict
      no-op" guarantee).
- [X] T009 [P] [US1] Add idempotence unit test: `format(format(x)) == format(x)` for a
      fixture already in the target casing, under `Upper` and under `Lower` (spec.md
      AS4/SC-003).
- [X] T010 [US1] Add one data-driven unit test iterating all 138 names from
      `FUNCTION_CALL_ENTRIES` (`function_call_entries()`), asserting each rewrites
      correctly under both `Upper` and `Lower` — closes the same coverage gap `024`'s
      own T010 closed for highlighting (spec.md SC-001).

**Checkpoint**: User Story 1 is independently testable — `cargo test -p voyager-core
function_call` passes T006-T010.

---

## Phase 4: User Story 2 - Dual-category names keep their own category (Priority: P2)

**Goal**: `FORMAT`/`LOG` (and any coincidentally-named recognized function) are governed
by whichever category actually recognizes their specific occurrence's structural
position, never both, never neither (spec.md FR-004, SC-004).

**Independent Test**: Format each fixture below with `casing_pair_keywords`/
`casing_control_words`/`casing_function_calls` set to different conventions from each
other, and assert only the expected category's convention applied to each occurrence.

- [X] T011 [P] [US2] Add unit test: `FILEO format=csv` under `casing_pair_keywords:
      Upper, casing_function_calls: Lower` renders `FILEO FORMAT=csv` — `format` (a
      pair-keyword name, followed by `=`) governed by `casing_pair_keywords` only
      (spec.md AS "User Story 2" Scenario 1).
- [X] T012 [P] [US2] Add unit test: `X = FORMAT(volume,8,2,',')` under the same
      settings renders `X = format(volume,8,2,',')` — `FORMAT` (a function call,
      followed by `(`) governed by `casing_function_calls` only (spec.md AS "User Story
      2" Scenario 2).
- [X] T013 [P] [US2] Add unit test: `LOG VAR=x` (control-word occurrence of `LOG`)
      under `casing_control_words: Upper, casing_function_calls: Lower` renders `LOG
      VAR=x` unchanged by `function_calls`; a separate line `Y = log(5)` under the same
      settings renders `Y = log(5)` (already lowercase, confirming `function_calls`
      alone governs it) — the second real dual-category name from research.md §3.
- [X] T014 [P] [US2] Add unit test: `PRINT LIST='calling replacestr(x) here'` under
      `function_calls: Upper` renders byte-identical — quote-safety (spec.md Edge
      Cases).
- [X] T015 [P] [US2] Add unit test: `MAX = 100` under `function_calls: Upper` renders
      byte-identical — `MAX` not followed by `(`, not a function-call occurrence
      (spec.md Edge Cases).

**Checkpoint**: User Stories 1 AND 2 both pass independently — `cargo test -p
voyager-core function_call` is fully green.

---

## Phase 5: Adapter Surface Wiring

**Purpose**: Make `function_calls` reachable through every surface the other three
casing categories already use (spec.md FR-006) — cross-cutting, not story-specific.

- [X] T016 [P] Add `casing_function_calls: Option<voyager_core::CasingConvention>` to
      `crates/drut-config/src/lib.rs`'s config struct(s) and merge logic, and its
      `"casing_function_calls"` TOML-key parse arm in `crates/drut-config/src/
      parse.rs` — same shape as `casing_pair_keywords` (data-model.md §5).
- [X] T017 [P] Add `--casing-function-calls` to `crates/drut-cli/src/cli.rs` and its
      wiring in `crates/drut-cli/src/format_cmd.rs`/`lib.rs` — same shape as
      `--casing-pair-keywords` (data-model.md §5).
- [X] T018 [P] Add `casing_function_calls` to the MCP `format` tool's parameter schema
      and handler in `crates/drut-mcp/src/` — same shape as the existing three
      (data-model.md §5).
- [X] T019 [P] Add `drut.format.casingFunctionCalls` to `editors/vscode/package.json`'s
      configuration contribution and its client-settings passthrough — same generic
      mechanism `829d065` established for every `[format]` field; explicitly NOT a
      change to `editors/vscode/syntaxes/drut.tmLanguage.json` (spec.md FR-009).
- [X] T020 Add/extend existing per-crate tests: `drut-config`'s TOML parse/merge test
      for `casing_function_calls`, `drut-cli`'s `--casing-function-calls` flag test,
      `drut-mcp`'s `format` tool parameter test — mirroring each crate's existing
      `casing_pair_keywords` test exactly (quickstart.md §6).

**Checkpoint**: `casing_function_calls` is configurable through `drut.toml`, the CLI,
the MCP `format` tool, and the VS Code setting — end to end, not just inside
`voyager-core`.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Golden-fixture verification (Constitution Principle III), full regression
re-proof, and project bookkeeping.

- [X] T021 Add `golden_casing_function_calls/real_corpus` fixture directory and its
      `format_corpus.rs` wiring (`function_calls: CasingConvention::Upper`, applied to
      the same 9 already-reviewed `real_corpus` fixtures `golden_data_references`/
      `golden_normalize` use — research.md §6). Regenerate via `UPDATE_GOLDEN=1 cargo
      test -p voyager-core --test format_corpus`, hand-diff every change before
      committing (same discipline `018`/`019`/`023` each followed).
- [X] T022 Add `real_corpus_fixtures_are_idempotent_under_function_calls_upper` (paired
      with T021's golden variant) AND `real_corpus_fixtures_are_idempotent_under_
      function_calls_lower` (`Lower`, no golden directory needed — `check_idempotent`
      only diffs a fixture's format-twice output against itself, research.md §6) —
      together these close spec.md SC-003's full claim ("every non-`preserve` value...
      for every real corpus fixture"), not just the `Upper` half T021 alone would cover.


- [X] T023 Run `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D
      warnings`, `cargo test --release --workspace` — confirm zero regressions on every
      pre-existing test (spec.md SC-002 end-to-end).
- [X] T024 [P] Update `ROADMAP.md`'s "Resolved queued items" log and `CHANGELOG.md`'s
      `## [Unreleased]` section with this feature's summary, matching the entry style
      already used for `024-function-call-highlighting`.

---

## Dependencies & Execution Order

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2, T002-T005)**: Depends on Setup. BLOCKS every test task in
  Phase 3/4 (a test asserting a rewrite the code doesn't yet produce will fail, by
  design — same as `024`'s own T002 relationship to its test phases).
- **User Story 1 (Phase 3)**: Can start once Phase 2 lands. No dependency on US2.
- **User Story 2 (Phase 4)**: Can start once Phase 2 lands. No dependency on US1
  (independent disambiguation/safety coverage of the same mechanism).
- **Adapter Surface Wiring (Phase 5)**: Can start once Phase 2 lands (needs
  `CasingSettings.function_calls` to exist) — independent of Phase 3/4's test content,
  though typically done after the core behavior is trusted.
- **Polish (Phase 6)**: Depends on Phase 3 + Phase 4 + Phase 5 all being green.

## Parallel Example: Phase 3 + Phase 4 + Phase 5 together

```bash
# Phase 2 (T002-T005) must land first. After that:
Task: "replacestr on assignment RHS rewrites under Upper"
Task: "rightstr(trim(...)) nested calls both rewrite under Lower"
Task: "casing_function_calls unset is a strict no-op"
Task: "format(format(x)) idempotent under Upper and Lower"
Task: "all 138 names rewrite correctly, data-driven"
Task: "FILEO format=csv: pair-keyword position, function_calls untouched"
Task: "X = FORMAT(...): function-call position, pair_keywords untouched"
Task: "LOG VAR=x vs Y = log(5): control-word vs function-call position"
Task: "quoted replacestr(x) never rewritten"
Task: "MAX = 100 (no paren) never rewritten"
Task: "drut-config casing_function_calls TOML round-trip"
Task: "drut-cli --casing-function-calls flag"
Task: "drut-mcp format tool casing_function_calls parameter"
Task: "VS Code drut.format.casingFunctionCalls setting passthrough"
```

## Implementation Strategy

### MVP First (User Story 1 only, voyager-core only)

1. Complete Phase 1 (trivial) + Phase 2 (T002-T005 — the actual mechanism).
2. Complete Phase 3 (T006-T010) — validates the core rewrite is correct.
3. **STOP and VALIDATE**: `cargo test -p voyager-core function_call` green for
   T006-T010 proves the feature works at the `voyager-core` level, before any adapter
   surface is touched.

### Incremental Delivery

1. Phase 1 + Phase 2 → core mechanism exists.
2. Phase 3 → User Story 1 proven at the `voyager-core` level.
3. Phase 4 → User Story 2 proven (disambiguation/safety) → `voyager-core` side
   complete.
4. Phase 5 → reachable end-to-end through every adapter.
5. Phase 6 → golden-fixture proof + full regression re-proof + bookkeeping → merge.

## Notes

- No new crate, no new `voyager-core` grammar/parser/diagnostic concept — every task
  edits an existing file or adds one new, narrowly-scoped module
  (`function_call.rs`) mirroring an already-reviewed precedent (`data_reference.rs`)
  almost line-for-line in shape.
- `editors/vscode/syntaxes/drut.tmLanguage.json` (the `024`-shipped highlighting
  pattern) is not touched by any task here (spec.md FR-009) — only T019's client-
  settings schema addition touches `editors/vscode/` at all.
