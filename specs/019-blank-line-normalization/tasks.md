---

description: "Task list for Blank-Line-Run Normalization"
---

# Tasks: Blank-Line-Run Normalization

**Input**: Design documents from `/specs/019-blank-line-normalization/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md
(all present)

**Tests**: Included — a public-API/CLI/config-surface shape change touching four crates, plus a
genuinely new render-pipeline capability (line deletion), needs real coverage at every layer,
matching `017`/`018`'s own precedent for a feature of this shape.

**Organization**: Foundational carries the entire capability — both `nested_lines`/blank-run
detection *and* the render-pipeline's line-deletion capability, since the underlying algorithm
naturally computes both caps' worth of deletions in one pass (there's no way to build "just the
top-level cap" without the same machinery that also handles the nested cap). US1 (P1, top-level
cap) then builds the config-surface plumbing (drut-config/CLI/MCP) for *all three* new settings
together — the mode and both caps are tightly coupled (a cap is meaningless without the mode
setting existing), so splitting the plumbing itself across stories would be artificial. US2 (P2,
nested cap) is therefore a **verification** story, not a construction one (same shape `017`'s own
US2 had) — it proves the nested cap specifically works, reusing Foundational's + US1's machinery
rather than adding new production code.

**Idempotence and `; FMT: OFF` regression tests are included directly in each story's own phase
below, not deferred to a post-analyze pass** — `018-operator-spacing`'s own `/speckit-analyze`
run found exactly this category of gap (finding I1: a guarantee stated in plan.md's Constitution
Check with no corresponding fast unit-level task, only late corpus-based coverage). Applying that
lesson directly here rather than waiting to rediscover it.

**Everything in this file's scope was measured against the real, current codebase during
planning (research.md), not estimated**:

- `render()`'s per-line emission loop has never needed to delete a line before — every prior
  formatting axis (indentation, casing, operator spacing) preserves a strict 1-input-line-to-
  1-output-line correspondence. This is a real, new capability (research.md §1), not a
  reinterpretation of an existing one — confirmed by reading the current loop structure directly.
- A blank-line run can never straddle a block boundary or a protected-region boundary (both are
  bounded by non-blank lines) — confirmed by construction, not assumed — so a run's
  classification and protection status are uniform across the whole run (research.md §3).
- "Any nesting depth" classification needs no recursion: a nested block's `span` is always
  contained within its parent's (guaranteed by `block.rs`'s own matching logic), so marking only
  *top-level* blocks' span ranges as "nested" already correctly classifies every line at every
  depth (research.md §4) — confirmed against the actual `Block`/`Node` type shapes, not assumed.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete sibling task)
- **[Story]**: US1/US2 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/src/blank_line.rs` (new) — the recognition module (`nested_lines`,
  `find_blank_runs`, `lines_to_delete`).
- `crates/voyager-core/src/format.rs` — `BlankLineMode` enum, `FormatOptions`'s three new fields,
  `render()`'s line-deletion integration.
- `crates/voyager-core/src/lib.rs` — new re-export.
- `crates/drut-config/src/lib.rs`, `src/parse.rs` — new fields, precedence resolution, TOML
  parsing.
- `crates/drut-config/tests/parse.rs`, `tests/resolve.rs` — new coverage.
- `crates/drut-cli/src/cli.rs`, `src/format_cmd.rs`, `src/lib.rs`, `tests/format_flags.rs` — new
  flags.
- `crates/drut-mcp/src/format.rs` (own test module included) — new params.
- `crates/drut-lsp/` — no source changes expected; existing suite passing unmodified after the
  type change compiles through is the confirmation, same as every prior feature.
- `ROADMAP.md` — item 13 marked done (Polish).

---

## Phase 1: Setup

- [X] T001 Confirm baseline: `cargo build --workspace` and `cargo clippy --workspace
      --all-targets -- -D warnings` both clean, on this branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The entire recognition + line-deletion capability. Neither story is meaningfully
testable until this compiles and its own tests pass.

- [X] T002 In `crates/voyager-core/src/format.rs`: add `BlankLineMode` enum (`Preserve`
      `#[default]`, `Auto` — data-model.md §1). Add `blank_lines: BlankLineMode`,
      `top_level_blank_line_cap: u8`, `nested_blank_line_cap: u8` to `FormatOptions`; extend the
      existing manual `impl Default for FormatOptions` with `blank_lines:
      BlankLineMode::default(), top_level_blank_line_cap: 2, nested_blank_line_cap: 1`. Update
      `FormatOptions`'s doc comment.
- [X] T003 Create `crates/voyager-core/src/blank_line.rs` with `nested_lines(nodes: &[Node]) ->
      BTreeSet<u32>` (data-model.md §1, research.md §4): for every `Node::Block` in the
      **top-level** `nodes` slice only (no recursion into `children`/branches — a nested block's
      span is always contained within its parent's, so the parent's own range already covers it),
      insert every line number from `block.span.start.line + 1` through `block.span.end.line`
      into the result set. Pure, no I/O, never panics. Depends on nothing beyond `Node`/`Block`
      already being in scope.
- [X] T004 In `crates/voyager-core/src/blank_line.rs`: `BlankRun { first_line: u32, len: u32,
      is_nested: bool, is_protected: bool }` (data-model.md §1) and `find_blank_runs(lines:
      &[Vec<char>], nested: &BTreeSet<u32>, protected: &BTreeSet<u32>) -> Vec<BlankRun>`: a line
      is blank when every character on it is `' '` or `'\t'` (vacuously true for a zero-length
      line — research.md §2, this project's own existing whitespace-check convention, not Rust's
      general `char::is_whitespace()`). Groups consecutive blank lines into maximal runs;
      `is_nested`/`is_protected` are each computed once per run from its first line (research.md
      §3: a run can never straddle either boundary, so this is safe, not an approximation).
      Depends on T003.
- [X] T005 In `crates/voyager-core/src/blank_line.rs`: `pub(crate) fn lines_to_delete(nodes:
      &[Node], lines: &[Vec<char>], protected: &BTreeSet<u32>, top_level_cap: u8, nested_cap: u8)
      -> BTreeSet<u32>`: calls `nested_lines`/`find_blank_runs`, and for every non-protected run
      whose `len` exceeds its applicable cap (`nested_cap` if `is_nested` else `top_level_cap`),
      inserts the run's own trailing `len - cap` line numbers (i.e. `first_line + cap` through
      `first_line + len - 1`) into the result — the run's first `cap` lines always survive
      untouched (FR-006, research.md §5). A protected run contributes nothing. Depends on T004.
- [X] T006 In `crates/voyager-core/src/format.rs`'s `render()`: compute `lines_to_delete:
      BTreeSet<u32>` by calling `blank_line::lines_to_delete`, gated behind
      `options.blank_lines != BlankLineMode::Preserve` (mirrors every other axis's own
      short-circuit — `Preserve`/unconfigured does exactly the same work as before this feature
      existed). Add one early-exit check at the top of the main per-line emission loop: `if
      lines_to_delete.contains(&line_num) { continue; }` — the line contributes nothing to `out`
      at all, not a blanked line. Every other per-line computation (indentation lookup, casing/
      spacing edit application) is unaffected, since it already only ever runs against a line
      that IS being emitted. Depends on T005.
- [X] T007 [P] In `crates/voyager-core/src/lib.rs`: re-export `BlankLineMode`.
- [X] T008 [P] Add unit tests to `crates/voyager-core/src/blank_line.rs`'s own test module:
      `nested_lines` correctly marks a top-level block's interior (including a doubly-nested
      block inside it, covered "for free" without recursion) and leaves top-level-only lines
      unmarked; `find_blank_runs` treats a whitespace-only line as blank and groups it into the
      same run as a zero-length neighbor; `lines_to_delete` keeps exactly the first `cap` lines
      of an over-cap run and marks the rest, leaves an at-or-under-cap run alone, applies
      `nested_cap` uniformly regardless of depth (not a further-reduced value at deeper
      nesting), and contributes nothing for a run inside a protected region. Depends on T003,
      T004, T005.
- [X] T009 [P] Add unit tests to `crates/voyager-core/src/format.rs`'s test module: a deleted
      line is genuinely absent from `format()`'s output (line count decreases, not just blanked);
      indentation/casing/spacing edits for a *surviving* line in the same file are applied
      correctly, unaffected by deletion elsewhere; `blank_lines: Preserve` (the default) produces
      byte-identical output to before this feature existed across a fixture with several
      over-cap runs (FR-009 regression case). Depends on T006.

**Checkpoint**: `BlankLineMode` and `blank_line.rs` are real, compiling, tested parts of
`voyager-core`; the line-deletion render capability exists and is proven correct in isolation.
`cargo build --workspace` succeeds (adapters don't construct `BlankLineMode` directly yet —
unaffected).

---

## Phase 3: User Story 1 - A project caps runaway blank-line runs between top-level statements (Priority: P1)

**Goal**: A project can set `blank_lines = "auto"` via `drut.toml`, CLI, or MCP, and see an
excessive top-level blank-line run contracted to the configured top-level cap — with `preserve`
(the default) producing zero change.

**Independent Test**: With the top-level cap left at its default (2) and `auto` enabled, format a
script containing a run of 5 blank lines between two top-level blocks and confirm exactly 2
remain; confirm a run of 1 or 2 blank lines elsewhere in the same file is left untouched.

### Implementation for User Story 1

- [X] T010 [US1] In `crates/drut-config/src/lib.rs`: add `blank_lines: Option<BlankLineMode>`,
      `top_level_blank_line_cap: Option<u8>`, `nested_blank_line_cap: Option<u8>` to both
      `FormatConfig` and `ExplicitFormatOverride`. Implement the single-tier precedence in
      `resolve_format_options` for `blank_lines` (`explicit.or(config).unwrap_or_default()`) and
      the range-validated-with-fallback precedence for each cap, mirroring
      `resolve_indent_width`'s existing pattern exactly (data-model.md §3). Depends on T002
      (needs `BlankLineMode` to construct the resolved `FormatOptions`).
- [X] T011 [US1] In `crates/drut-config/src/parse.rs`: add TOML parsing for `blank_lines`
      (`"preserve"`/`"auto"`, case-sensitive matching `top_level_indent`'s existing precedent) and
      for `top_level_blank_line_cap`/`nested_blank_line_cap` (plain integers, same
      malformed-value-warns-and-falls-back pattern `indent_width` already uses). Depends on T010.
- [X] T012 [US1] In `crates/drut-cli/src/cli.rs`: add `--blank-lines` (`Option<BlankLineArg>`,
      same `ValueEnum`/"no bare flag" shape every other mode flag uses),
      `--top-level-blank-line-cap`/`--nested-blank-line-cap` (`Option<u8>`, range-validated at
      the argument-parsing layer like `--indent-width`). In `crates/drut-cli/src/format_cmd.rs`:
      wire all three into `ExplicitFormatOverride`; update `crates/drut-cli/src/lib.rs`'s call
      site. Depends on T010.
- [X] T013 [US1] In `crates/drut-mcp/src/format.rs`: add `blank_lines` (string),
      `top_level_blank_line_cap`/`nested_blank_line_cap` (integer) parameters to the `format`
      tool's input, same accepted-value shape and error-message pattern as the existing
      `top_level_indent`/`indent_width` parameters. Depends on T010.

### Tests for User Story 1

- [X] T014 [P] [US1] Add tests to `crates/drut-config/tests/parse.rs`: `blank_lines` parses
      `"preserve"`/`"auto"` cleanly; each cap parses a plain integer cleanly; a malformed value
      for any of the three warns and falls back.
- [X] T015 [P] [US1] Add tests to `crates/drut-config/tests/resolve.rs`: an explicit CLI/MCP
      value overrides a `drut.toml`-resolved one, per setting; nothing configured anywhere
      resolves to `preserve`/`2`/`1`; an out-of-range cap in `drut.toml` falls back to that cap's
      own default with a warning.
- [X] T016 [P] [US1] Add tests to `crates/drut-cli/tests/format_flags.rs`: `--blank-lines=auto`
      overrides a `drut.toml`-resolved `preserve` for one run; `--top-level-blank-line-cap=N`
      overrides the default; an out-of-range explicit cap value is a usage error, not a silent
      clamp (mirroring `--indent-width`'s own regression case).
- [X] T017 [P] [US1] Add the equivalent tests to `crates/drut-mcp/src/format.rs`'s own test
      module, mirroring T016's shape at the MCP surface.
- [X] T018 [US1] Add an integration test (`crates/voyager-core/tests/`, real-corpus-shaped
      fixture, not synthetic-only) exercising spec.md US1's own Acceptance Scenarios directly: a
      run of 5 blank lines between two top-level blocks contracts to exactly 2 under `auto` with
      default caps; a run of 2 or fewer elsewhere in the same file is untouched; no
      `blank_lines` configuration at all leaves the same script byte-identical (US1 AS3).
      Depends on Phase 2, T010-T013.
- [X] T019 [US1] In `crates/voyager-core/src/format.rs`'s test module: an idempotence test —
      `format(format(x, opts).text, opts).text == format(x, opts).text` — for `opts` with
      `blank_lines: Auto`, run against a fixture with an over-cap top-level run; and a
      `; FMT: OFF`/`; FMT: ON` interaction test — a fixture with a protected region containing an
      over-cap top-level run, formatted with `blank_lines: Auto`, confirming the protected run is
      left exactly as written while an unprotected over-cap run elsewhere in the same file
      contracts normally. Depends on T006.

**Checkpoint**: User Story 1 independently proven — the top-level cap is configurable end-to-end
at every surface, matches every one of spec.md's US1 Acceptance Scenarios, is confirmed
idempotent and `; FMT: OFF`-respecting — not assumed.

---

## Phase 4: User Story 2 - A project independently caps blank-line runs inside a block's body (Priority: P2)

**Goal**: Confirm the nested cap (the same setting/plumbing US1 already wired end-to-end) works
independently of the top-level cap — this story is verification-focused; the production code it
proves already exists from Phase 2 (T003-T006) and Phase 3 (the config plumbing that lets both
caps be set at all).

**Independent Test**: With the nested cap left at its default (1) and `auto` enabled, format a
script containing a run of 4 blank lines inside a block's body (at any nesting depth) and confirm
exactly 1 remains, independent of whatever the top-level cap does elsewhere in the same file.

### Verification for User Story 2

- [X] T020 [US2] Add an integration test (`crates/voyager-core/tests/`, real-corpus-shaped
      fixture) reproducing spec.md US2's own Acceptance Scenarios directly: a run of 4 blank
      lines inside a block's body contracts to exactly 1 under `auto` with default caps; a
      doubly-nested block's own excessive run gets the same nested cap, not a further-reduced
      one (US2 AS2); a single file with both an excessive top-level run and an excessive nested
      run has each contracted independently to its own applicable cap (US2 AS3). Depends on
      Phase 2, Phase 3.
- [X] T021 [US2] In `crates/voyager-core/src/format.rs`'s test module: an idempotence test for
      `blank_lines: Auto` run against a multi-level-nested fixture with an over-cap nested run;
      and a `; FMT: OFF`/`; FMT: ON` interaction test — a fixture with a protected region
      containing an over-cap *nested* run specifically, confirming it's left exactly as written
      while an unprotected nested run elsewhere in the same file contracts normally. Depends on
      T006.

**Checkpoint**: User Story 2 independently proven — the nested cap applies uniformly regardless
of depth, independently of the top-level cap, confirmed idempotent and `; FMT: OFF`-respecting.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: `ROADMAP.md` update and whole-workspace/full-corpus re-proof, once both stories are
done.

- [X] T022 [P] In `ROADMAP.md`: mark item 13 done, dated, pointing at this feature's spec
      directory — same pattern every other completed `ROADMAP.md` item already follows.
- [X] T023 `cargo test --release --workspace` and `cargo clippy --workspace --all-targets --
      -D warnings`, both clean.
- [X] T024 [P] Full 161-file real-corpus revalidation across CLI/LSP/MCP with **no new
      configuration supplied** — expected zero diagnostic/output change from before this
      feature (SC-003), reported as its own explicit result, not inferred from the unit-test
      suite alone.
- [X] T025 Format a handful of real corpus files *with* `blank_lines=auto` (default caps,
      quickstart.md step 5), hand-verify the diffs are exactly the expected line deletions
      (nothing reordered, no surviving line's own content touched, no non-blank content touched),
      then promote those diffs to new golden fixtures under
      `crates/voyager-core/tests/fixtures/golden_blank_lines/`, with idempotence checks (SC-005)
      — same discipline `017`/`018` already established. Depends on T024.
- [X] T026 Run `quickstart.md` end-to-end as written, confirming every step's expected result
      holds against the actual shipped code, not just against the individual task-level tests
      above in isolation.

**Checkpoint**: Feature-complete against spec.md; `ROADMAP.md` consistent with shipped code;
full workspace and full corpus re-proven clean; new golden fixtures added.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS both user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational only.
- **User Story 2 (Phase 4)**: Depends on Foundational **and** Phase 3's config-surface plumbing
  (T010-T013) — the same setting/flags/params the nested cap is set through are Phase 3's own
  deliverable, so US2's independent test needs them to exist first. Called out explicitly, same
  as `017`'s US2/US1 dependency.
- **Polish (Phase 5)**: T022 is independent of the code phases; T023-T026 depend on both stories
  being complete.

### Parallel Opportunities

- T007, T008, T009 can run in parallel once their respective dependencies land.
- T014-T017 can run in parallel once T010-T013 land (different test files).
- T019 can run in parallel with T018 once T006/T010-T013 land (different files: idempotence/
  FMT-off tests vs. the AS1-AS3 integration test).

---

## Parallel Example: Once Foundational (T002-T009) Lands

```bash
Task: "T010: drut-config gains blank_lines + both caps + precedence"
Task: "T022: ROADMAP.md item 13 marked done"
```

---

## Implementation Strategy

### MVP First (User Story 1 alone)

1. Setup → baseline confirmed clean.
2. Foundational → recognition + line-deletion capability exist, compile, are tested in isolation.
3. User Story 1 → the top-level cap configurable end-to-end, proven against spec.md's own
   Acceptance Scenarios, confirmed idempotent and `; FMT: OFF`-respecting.
4. **STOP and VALIDATE**: run T018 against a real corpus-shaped script.

### Incremental Delivery

1. Foundational → capability ready.
2. US1 → MVP (top-level cap ships, fully functional; nested cap already exists underneath but
   unproven).
3. US2 → nested-cap verification, whenever convenient.
4. Polish → `ROADMAP.md` update, full re-proof, golden fixtures.

---

## Notes

- T003's `nested_lines` "top-level blocks only, no recursion" shape is this feature's single
  most load-bearing simplification (research.md §4) — T008's doubly-nested test case exists
  specifically to prove this isn't a false economy, not as mechanical coverage.
- T006 is this feature's one genuinely new render-pipeline capability (line deletion) — T009's
  "line count decreases, not just blanked" assertion exists specifically to catch a
  half-implementation that clears a line's content but still emits an empty output line instead
  of truly removing it.
- T019/T021's idempotence/`; FMT: OFF` tests are written into each story's own phase directly
  from the start, applying `018-operator-spacing`'s own post-`/speckit-analyze` lesson (finding
  I1) up front rather than waiting to rediscover the same gap.
- Commit after each task or logical group.
