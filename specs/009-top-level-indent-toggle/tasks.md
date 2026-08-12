---

description: "Task list for Top-Level Indent Default Revert"
---

# Tasks: Top-Level Indent Default Revert

**Input**: Design documents from `/specs/009-top-level-indent-toggle/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/,
quickstart.md (all present)

**Tests**: Included — a formatter default reversal touching real corpus
output requires real coverage: `voyager-core`'s own unit tests for both
modes, `format_sequence.rs`'s retargeted `008`-guarantee regression,
human-reviewed golden-fixture regeneration, and the explicit
default-placement tests FR-004/User Story 3 name as this feature's own
Definition of Done.

**Organization**: Three P1 user stories, matching spec.md exactly — US1
(the default reverts to `preserve`), US2 (`008`'s behavior stays fully
available, opt-in), US3 (the `preserve` default is independently verified
at every integration point, not just the CLI). All three depend on the
same small core change (T002/T003); US2 and US3 are otherwise independent
of each other.

**Everything in this file's scope was measured against the real,
current codebase during planning (research.md §1-§4), not estimated**:

- **Exactly 3 of ~34 `voyager-core` unit tests need retargeting** to
  explicit `Normalize` (they assert `008`-era top-level forcing); the
  rest are mode-independent by fixture construction (research.md §3).
- **All 5 `format_sequence.rs` tests need retargeting** — every one of
  them exists specifically to prove `008`'s guarantee.
- **Two `FormatOptions` call sites are full struct literals** (`drut-cli`,
  `drut-mcp`) — the compiler already forces explicit handling there;
  **two are `::default()` calls with no compiler forcing**
  (`drut-lsp`'s `formatting.rs`/`range_formatting.rs`) — these get
  dedicated new tests, since nothing else will catch a regression there
  (research.md §2).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an
  incomplete sibling task)
- **[Story]**: US1/US2/US3 — omitted for Setup/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/src/format.rs` — the new enum/field, the
  conditional in `plan_indentation`, and this crate's own test module.
- `crates/voyager-core/src/lib.rs` — re-export `TopLevelIndentMode`.
- `crates/voyager-core/tests/format_sequence.rs` — retargeted `008`
  regression (US2).
- `crates/voyager-core/tests/format_corpus.rs` +
  `tests/fixtures/golden/` + new `tests/fixtures/golden_normalize/` —
  golden regeneration (US1) and the `Normalize`-mode proof (US2).
- `crates/drut-cli/src/cli.rs`, `format_cmd.rs`, `lib.rs` — the new flag
  (US2).
- `crates/drut-mcp/src/format.rs` — explicit default field (US3).
- `crates/drut-lsp/src/formatting.rs`, `range_formatting.rs` — new
  dedicated default-verification tests, no code change (US3).
- `specs/002-cli-check-format/spec.md` +
  `specs/002-cli-check-format/contracts/formatting-api.md` — amended per
  `contracts/top-level-indent-toggle.md`'s exact replacement text (US1).

---

## Phase 1: Setup

- [x] T001 Confirm baseline: `cargo build --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings` both clean,
      on this fresh branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: User Story 1 - Default formatting leaves top-level indentation exactly as written (Priority: P1) 🎯 MVP

**Goal**: `preserve` becomes the default top-level indentation behavior
again; `008`'s golden fixtures revert to reflect it.

**Independent Test**: Format a script with non-zero top-level
indentation, no flags, and confirm it's byte-identical to the input.

### Implementation for User Story 1

- [x] T002 [US1] Add `TopLevelIndentMode` (`Preserve`/`Normalize`,
      `#[default]` on `Preserve`) to `crates/voyager-core/src/format.rs`
      per data-model.md; add the `top_level_indent: TopLevelIndentMode`
      field to `FormatOptions`; re-export `TopLevelIndentMode` from
      `crates/voyager-core/src/lib.rs` alongside `CasingConvention`.
- [x] T003 [US1] Make `plan_indentation`'s column-0 insert conditional on
      `mode == TopLevelIndentMode::Normalize` (research.md §1's exact
      diff — thread the mode through `render`'s call into
      `plan_indentation` from `options.top_level_indent`). No change to
      `plan_block`/`plan_children`/`computed_indent`. Depends on T002.
- [x] T004 [US1] Retarget the 3 tests research.md §3 names in
      `crates/voyager-core/src/format.rs`'s own test module to construct
      `FormatOptions { top_level_indent: TopLevelIndentMode::Normalize,
      ..Default::default() }` explicitly:
      `top_level_baseline_is_always_normalized_to_zero`,
      `bare_top_level_statement_is_normalized_to_zero`,
      `diagnosed_block_opener_is_normalized_but_children_stay_untouched`.
      Depends on T003.
- [x] T005 [P] [US1] Add 3 new `Preserve`-mode sibling tests to the same
      module (default `FormatOptions`, i.e. no explicit mode needed):
      a top-level `RUN` at non-zero indentation stays untouched (revives
      pre-`008`'s `top_level_baseline_is_left_untouched` assertion); a
      bare top-level statement's indentation stays untouched; a diagnosed
      top-level block's opener *and* children both stay untouched
      (unlike the `Normalize`-mode sibling, where only children stay
      protected). Depends on T003.
- [x] T006 [P] [US1] Amend `specs/002-cli-check-format/spec.md`'s FR-012
      bullet (second dated entry, `008`'s own entry preserved) and add the
      new FR-026, plus amend `contracts/formatting-api.md`, using
      `contracts/top-level-indent-toggle.md`'s exact replacement text.
      Independent of T002-T005 — documentation only, different files.
- [x] T007 [US1] Before any regeneration, copy the *current* (`008`-era,
      already-reviewed) `tests/fixtures/golden/` and
      `tests/fixtures/golden/real_corpus/` contents verbatim into a new
      `tests/fixtures/golden_normalize/` directory, mirroring the same
      structure — this captures `008`'s already-shipped, already-
      human-reviewed output as-is, before it gets overwritten by T009's
      `preserve`-mode regeneration. No test changes yet — just the copy.
- [x] T008 [US1] Regenerate and individually human-review every affected
      golden fixture back to `preserve`-mode output (quickstart.md step
      4):
      ```powershell
      $env:UPDATE_GOLDEN = "1"
      cargo test -p voyager-core --test format_corpus
      Remove-Item Env:\UPDATE_GOLDEN
      git diff crates/voyager-core/tests/fixtures/golden/
      ```
      Expect the same file set `008` originally changed (research.md §3
      of `008`'s own spec names 7 `real_corpus/` files) to revert. For
      each file, confirm explicitly: only leading-whitespace lines
      changed (reverting toward the pre-`008` value), nothing else
      moved/added/removed/corrupted. Report each file's review outcome
      individually. Depends on T003, T007 (copy must happen first).

**Checkpoint**: `preserve` is the real, default, tested behavior; goldens
reflect it; `008`'s original output is preserved verbatim in
`golden_normalize/` for T012 to prove against.

---

## Phase 3: User Story 2 - `008`'s behavior remains available, opt-in (Priority: P1)

**Goal**: `--top-level-indent=normalize` reproduces `008`'s original
behavior exactly, verified both at the unit level and against the real
corpus.

**Independent Test**: Format the same fixture with
`--top-level-indent=normalize` and confirm it matches `008`'s originally
shipped output byte-for-byte.

### Implementation for User Story 2

- [x] T009 [US2] Add `TopLevelIndentArg` (`ValueEnum`, mirrors
      `CasingArg`'s shape) to `crates/drut-cli/src/cli.rs`; add
      `#[arg(long, value_enum, default_value_t = TopLevelIndentArg::
      Preserve)] top_level_indent: TopLevelIndentArg` to `Command::
      Format` (research.md §4 — the `OutputFormat` shape, not
      `CasingArg`'s `Option<...>` shape, since this setting is never
      "off").
- [x] T010 [US2] In `crates/drut-cli/src/format_cmd.rs`: add `impl From
      <TopLevelIndentArg> for TopLevelIndentMode` (mirrors the existing
      `CasingArg`→`CasingConvention` impl); add a `top_level_indent:
      TopLevelIndentArg` parameter to `run()`; set the field explicitly
      on the `FormatOptions` struct literal via `.into()`. Update the one
      call site in `crates/drut-cli/src/lib.rs` (`Command::Format { ...,
      top_level_indent } => format_cmd::run(&path, write, check, diff,
      casing, top_level_indent)`) to destructure and pass the new field.
      Depends on T002, T009.
- [x] T011 [P] [US2] Add `--top-level-indent` coverage to
      `crates/drut-cli/tests/format_flags.rs`, mirroring the existing
      `--casing` test shape: omitted flag defaults to `preserve`
      (non-zero top-level indentation untouched); `--top-level-indent=
      normalize` forces column 0; `--top-level-indent=preserve` explicit
      is identical to omitting it. Depends on T010.
- [x] T012 [US2] Retarget all 5 tests in
      `crates/voyager-core/tests/format_sequence.rs` to construct
      `FormatOptions { top_level_indent: TopLevelIndentMode::Normalize,
      ..Default::default() }` explicitly at every `format(...,
      FormatOptions::default())` call site in the file — proving `008`'s
      own guarantee (the `PROCESS`/`RUN` residue sequence resolving in
      one pass) still holds under explicit `Normalize`. Depends on T003.
- [x] T013 [US2] Parameterize `crates/voyager-core/tests/format_corpus.rs`'s
      three shared helpers (`check_golden`, `check_idempotent`,
      `check_structure_and_diagnostics_preserved`) to accept an explicit
      `FormatOptions` argument instead of hardcoding
      `FormatOptions::default()` internally; update every existing
      `#[test]` call site to pass `FormatOptions::default()` explicitly
      (behaviorally unchanged — still `Preserve`). Add new `#[test]`
      functions (hand-written + `real_corpus`) that call the same helpers
      with `FormatOptions { top_level_indent: TopLevelIndentMode::
      Normalize, ..Default::default() }` against the `golden_normalize/`
      fixture set T007 populated — proving `Normalize` mode is
      byte-identical to `008`'s already-reviewed output, no second
      human-review pass needed. Depends on T007, T003.

**Checkpoint**: `008`'s behavior is fully intact and independently
re-proven, both as a CLI flag and against the real corpus, now opt-in
rather than default.

---

## Phase 4: User Story 3 - The default is the same everywhere a format request can originate (Priority: P1)

**Goal**: `preserve` is independently confirmed as the actual, resolved
default at every `FormatOptions` construction site outside `drut-cli`
(FR-004), not inferred from any one of them.

**Independent Test**: Call `voyager_core::FormatOptions::default()`
directly, then the LSP whole-document/range formatting handlers, then the
MCP `format` tool, each with no explicit override, against a non-zero
top-level fixture — confirm all three leave it untouched.

### Implementation for User Story 3

- [x] T014 [P] [US3] Add a direct, minimal test to
      `crates/voyager-core/src/format.rs`'s test module asserting
      `FormatOptions::default().top_level_indent ==
      TopLevelIndentMode::Preserve` — the single most direct
      confirmation of FR-004(b), distinct from (and cheaper than) the
      behavioral tests around it. Depends on T002.
- [x] T015 [P] [US3] In `crates/drut-mcp/src/format.rs`'s
      `casing_option` function (or a renamed equivalent covering both
      fields): explicitly set `top_level_indent:
      voyager_core::TopLevelIndentMode::default()` on the `FormatOptions`
      struct literal (compiler already forces *a* value here — this
      makes the choice visible in the diff, not merely satisfied). No new
      `FormatInput` field — no MCP-side toggle in scope (spec
      Assumptions). Depends on T002.
- [x] T016 [P] [US3] Add a new test to `crates/drut-mcp/src/format.rs`'s
      own test module with a genuinely non-zero top-level fixture (e.g.
      `"    IF (a=b)\n    PRINT LIST=1\n    ENDIF\n"`), confirming the
      `format` tool's default output leaves it untouched. Depends on T015.
- [x] T017 [P] [US3] Add a new test to `crates/drut-lsp/src/formatting.rs`'s
      own test module: a document with non-zero top-level indentation,
      formatted via the existing `handle` function with no client-side
      override, is left untouched at the top level (nested content still
      corrects normally). No code change to `formatting.rs` itself.
      Depends on T002.
- [x] T018 [P] [US3] Add a new test to
      `crates/drut-lsp/src/range_formatting.rs`'s own test module: same
      shape as T017, via `textDocument/rangeFormatting`'s `handle`
      function — a non-zero top-level line inside (or at the edge of) the
      requested range is left untouched. No code change to
      `range_formatting.rs` itself. Depends on T002.

**Checkpoint**: `preserve` is independently proven at every integration
point named in FR-004 — CLI default, core library default, both LSP
handlers, and the MCP tool.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Whole-workspace and full-corpus re-proof, once all three
stories are done.

- [x] T019 `cargo test --release --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings`, both
      clean.
- [x] T020 [P] Full 161-file corpus revalidation across all three adapter
      surfaces (quickstart.md step 7), each reported individually:
      ```powershell
      $env:DRUT_CORPUS_PATH = "path\to\WF-TDM-Official-Releases"
      cargo test --release -p drut-cli --test fixture_corpus_e2e -- --ignored
      cargo test --release -p drut-lsp --test diagnostics_corpus -- --ignored
      cargo test --release -p drut-mcp --test diagnostics_corpus -- --ignored
      ```
      Expected and required: still 161/161 clean (SC-005) — a
      whitespace-shifting reversion only, zero new diagnostics anywhere.

**Checkpoint**: Feature-complete against spec.md; goldens reflect
`preserve`; `008`'s behavior independently re-proven under
`golden_normalize/`; full corpus re-proven clean.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **User Story 1 (Phase 2)**: Depends on Setup.
- **User Story 2 (Phase 3)**: Depends on US1's T002/T003 (the mode must
  exist) and T007 (the `golden_normalize/` copy must happen before
  `preserve`-mode regeneration overwrites the source it's copied from).
- **User Story 3 (Phase 4)**: Depends only on T002 (the field must
  exist) — independent of US2 entirely.
- **Polish (Phase 5)**: Depends on all three stories being complete.

### Within User Story 1

- T002 before T003 (the field must exist before the conditional can read
  it) before T004/T005 (tests need the real behavior to assert against)
  and T007→T008 in that exact order (copy `008`'s golden output before
  overwriting it).
- T006 is independent of the code tasks — different files.

### Parallel Opportunities

- T005 and T006 can proceed in parallel with T004 once T003 lands.
- T011, T014, T015 (started independently), T017, T018 can all proceed in
  parallel once T002/T009/T010 (as applicable) land — different files,
  non-conflicting.
- T020's three corpus-validation commands are independent of each other
  and of T019.

---

## Parallel Example: Once T002/T003 Land

```bash
Task: "T004: retarget the 3 Normalize-assuming format.rs tests"
Task: "T005: add the 3 new Preserve-mode sibling tests"
Task: "T006: amend spec.md FR-012 (second entry) + formatting-api.md"
Task: "T012: retarget format_sequence.rs's 5 tests to explicit Normalize"
Task: "T014: FormatOptions::default() direct assertion"
Task: "T017: drut-lsp formatting.rs default-verification test"
Task: "T018: drut-lsp range_formatting.rs default-verification test"
```

---

## Implementation Strategy

### Single Pass (all three stories are small and share one core change)

1. Setup → baseline confirmed clean.
2. User Story 1 → the default reversion itself, its tests, its doc
   amendment, and the human-reviewed golden regeneration (T007→T008 in
   that order is the one step that cannot be reordered or rushed).
3. User Story 2 → the CLI flag, its tests, and the `Normalize`-mode
   corpus proof against `golden_normalize/`.
4. User Story 3 → the four independent default-placement tests named in
   FR-004/spec.md's own explicit correctness requirement.
5. Polish → whole-workspace and full-corpus re-proof, reported
   explicitly.

---

## Notes

- T007's copy-before-overwrite ordering is the one sequencing detail in
  this feature that actually matters — get it backwards and `008`'s
  original golden output is lost with no easy way to reconstruct it
  except from git history.
- T009/T010's split (flag definition vs. wiring) mirrors how `CasingArg`
  was originally introduced, kept for consistency with the existing
  codebase pattern, not because this feature needs two commits.
- Commit after each task or logical group.
