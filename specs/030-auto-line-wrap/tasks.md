---

description: "Task list for Automatic Line-Width Wrapping"
---

# Tasks: Automatic Line-Width Wrapping

**Input**: Design documents from `/specs/030-auto-line-wrap/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md
(all present)

**Tests**: Included — a public-API/CLI/config-surface shape change touching five crates/the
editor extension, plus a genuinely new recognition capability (`line_wrap.rs`) and a new
render-pipeline capability (a `SpacingEdit` whose replacement embeds a line-terminator, actually
splitting a line for the first time), needs real coverage at every layer, matching
`018-operator-spacing`'s own precedent for a feature of this shape.

**Organization**: Single user story (spec.md has one priority tier — this increment is
deliberately narrower than `018`'s two-story shape). Foundational carries the entire wrapping
capability itself — top-level comma detection with depth tracking, the already-continued check,
`Fill`/`OnePerLine` packing logic, and the new terminator-embedding/independently-indented edit
construction — without which the story is not meaningfully testable. User Story 1 then builds
the config-surface plumbing (drut-config/CLI/MCP/VS Code) that makes the three new settings
actually settable, and proves wrapping end-to-end.

**Everything in this file's scope was measured against the real, current codebase during
planning (research.md), not estimated**:

- `render()`'s per-line rebuild currently maps one original source line to at most one output
  line — confirmed directly by reading the actual loop, not assumed. This feature is the first
  to break that invariant, done by embedding a line-terminator character inside a `SpacingEdit`
  replacement string, reusing `018`'s existing variable-length edit-application mechanism rather
  than adding a second, parallel text-rewriting pass (research.md §1).
- The per-line loop appends that line's own captured terminator *after* every edit's replacement
  — confirmed directly. A wrap edit's embedded terminator must match that specific line's own
  captured CRLF/LF style, never a hardcoded `\n`, or a CRLF file gets one line ending in bare
  `\n` while every other line stays `\r\n` — a real, silent bug this feature must not introduce
  (research.md §1, promoted to FR-level in plan.md's Constraints).
- `indent_plan` is keyed by original source line number — confirmed directly. A wrap-inserted
  continuation line is synthetic and has no entry in it; its indentation must be computed
  independently inside the wrap edit itself (research.md §1).
- `build_statements`'s flat token list (already used by `operator_spacing.rs`, confirmed by
  reading its own call site) is what carries a `Control` statement's full original inter-token
  positions — this feature reuses that exact list rather than walking `nodes`/`Block` (research.md
  §4).
- **Correction made during implementation, not caught at planning time**: research.md §4
  originally claimed a string literal lexes as one atomic token. Direct testing (T005) proved
  this wrong — `'a, b'` lexes as separate `'`/`a`/`,`/`b`/`'` tokens in this grammar, the exact
  same problem `operator_spacing.rs` already solved for its own operator characters.
  `top_level_split_points` reuses that module's own `pub(crate) quoted_token_mask` function
  directly rather than duplicating quote-tracking logic — a masked token is excluded from both
  split-point collection and paren/bracket depth-tracking (research.md §4, corrected).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete sibling task)
- **[Story]**: US1 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/src/line_wrap.rs` (new) — top-level comma detection, already-continued
  check, `Fill`/`OnePerLine` packing, wrap-edit construction.
- `crates/voyager-core/src/format.rs` — `LineWrapMode`/`LineWrapStyle` enums, `FormatOptions`'s
  three new fields, `render()`'s wrap-edit collection call and terminator-aware application.
- `crates/voyager-core/src/lib.rs` — new re-exports.
- `crates/drut-config/src/lib.rs`, `src/parse.rs` — three new fields, precedence resolution,
  TOML parsing, width-range validation.
- `crates/drut-config/tests/parse.rs`, `tests/resolve.rs` — new coverage.
- `crates/drut-cli/src/cli.rs`, `src/format_cmd.rs`, `tests/format_flags.rs` — three new flags.
- `crates/drut-mcp/src/format.rs` (own test module included) — three new params.
- `editors/vscode/package.json` (plus whatever client-side wiring the existing `drut.format.*`
  fields already use, confirmed during implementation) — three new personal settings.
- `crates/drut-lsp/` — no source changes expected; existing suite passing unmodified after the
  type change compiles through is the confirmation, same as prior formatting features.
- `ROADMAP.md` — new item marked done (Polish).

---

## Phase 1: Setup

- [X] T001 Confirm baseline: `cargo build --workspace` and `cargo clippy --workspace
      --all-targets -- -D warnings` both clean, on this branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The entire wrapping capability itself — recognition, packing, edit construction —
plus the render-pipeline's new ability to embed a line-terminator inside an edit's replacement.
The story is not meaningfully testable until this compiles and its own tests pass.

- [X] T002 In `crates/voyager-core/src/format.rs`: add `LineWrapMode` enum (`Preserve`
      `#[default]`, `Auto`) and `LineWrapStyle` enum (`Fill` `#[default]`, `OnePerLine` —
      data-model.md §1). Add `line_wrap: LineWrapMode`, `line_wrap_width: u16`,
      `line_wrap_style: LineWrapStyle` to `FormatOptions`; extend the existing manual `impl
      Default for FormatOptions` with `line_wrap: LineWrapMode::default()`, `line_wrap_width:
      120`, `line_wrap_style: LineWrapStyle::default()`. Update `FormatOptions`'s doc comment.
- [X] T003 Create `crates/voyager-core/src/line_wrap.rs` with `already_continued(tokens:
      &[Token]) -> bool` (data-model.md §1) — `true` if `tokens` contains any
      `TokenKind::ContinuationMarker`. Add a unit test confirming a manually-continued
      `Control` statement's own token list returns `true`, and a plain single-line statement
      returns `false`. Depends on T002 (module wiring only).
- [X] T004 In `crates/voyager-core/src/line_wrap.rs`: `SplitPoint { token_index: usize, span:
      Span }` and `top_level_split_points(tokens: &[Token]) -> Vec<SplitPoint>` (data-model.md
      §1, research.md §4) — walks `tokens` tracking paren `(`/`)` and bracket `[`/`]` depth,
      collecting every `,` `Punctuation` token seen at depth zero. Depends on T002.
- [X] T005 [P] Add a unit test to `crates/voyager-core/src/line_wrap.rs`'s own test module
      confirming the structural claim in research.md §4 directly: a `Control` statement with a
      comma inside a quoted pair-value (e.g. `RUN PGM=MATRIX, MSG='a, b'`) produces exactly the
      split points from the *real* top-level commas, never one from inside the quoted value —
      spot-checked against actual tokenizer output, not assumed. Depends on T004.
- [X] T006 [P] Add unit tests to `crates/voyager-core/src/line_wrap.rs`'s own test module: a
      comma inside a function call's parentheses is never collected; a comma inside a bracketed
      subscript is never collected; a `Control` statement with no comma at all produces an empty
      result; multiple top-level commas on one statement are all collected, in order. Depends on
      T004.
- [X] T007 In `crates/voyager-core/src/line_wrap.rs`: `plan_wrap(statement_text: &str,
      split_points: &[SplitPoint], width: u16, style: LineWrapStyle) -> Option<Vec<SplitPoint>>`
      (data-model.md §1) — returns `None` when `statement_text.len() <= width as usize` or
      `split_points.is_empty()`; otherwise, under `Fill`, walks split points left to right and
      selects a break at the last split point still within budget before the next segment would
      exceed `width`; under `OnePerLine`, selects every split point unconditionally. Depends on
      T004.
- [X] T008 [P] Add unit tests to `crates/voyager-core/src/line_wrap.rs`'s own test module: an
      under-width statement returns `None` regardless of style; a `Control` statement with no
      split points returns `None` regardless of width; `Fill` packs multiple short pairs onto
      one continuation line up to the width budget and only breaks when the next pair would
      exceed it; `OnePerLine` selects every available split point unconditionally, even when
      several consecutive pairs would have fit together under `Fill`. Depends on T007.
- [X] T009 In `crates/voyager-core/src/line_wrap.rs`: `wrap_edit(split: &SplitPoint, terminator:
      &str, continuation_indent: &str) -> SpacingEdit` (data-model.md §1) — a zero-width
      insertion immediately after the comma's own span end, whose replacement is `terminator`
      followed by `continuation_indent`, exactly as given (no further logic — terminator/indent
      resolution is the caller's, i.e. `format.rs::render`'s, responsibility per data-model.md
      §1-2). Depends on T004.
- [X] T010 In `crates/voyager-core/src/format.rs`: extend `render()`'s existing per-line rebuild
      path (`018`'s own addition) with a wrap-edit collection call — for each `Control`
      statement (from the existing `build_statements` call already used for operator-spacing),
      short-circuited entirely behind `options.line_wrap != LineWrapMode::Preserve`, skip
      immediately if T003's `already_continued` is `true`; otherwise call T004's
      `top_level_split_points` and T007's `plan_wrap`, and for each chosen split point construct
      T009's `wrap_edit` using *that specific original line's own already-captured terminator*
      (not a hardcoded `\n`, research.md §1) and an independently-computed continuation indent
      (one level deeper than the statement's own opening line's resolved indent — never read
      from `indent_plan`, which has no entry for the synthetic line, research.md §1). Merge
      these into the same per-line sorted edit list casing/spacing edits already use. Extend the
      `; FMT: OFF`/`ON` protected-line funnel to wrap edits — a protected line receives none.
      Depends on T003, T007, T009.
- [X] T011 [P] In `crates/voyager-core/src/lib.rs`: re-export `LineWrapMode`/`LineWrapStyle`.
- [X] T012 [P] Add unit tests to `crates/voyager-core/src/format.rs`'s test module: a
      CRLF-terminated input file's newly-inserted continuation line ends in CRLF, not a bare
      `\n` (data-model.md §2, the terminator-correctness requirement) — a dedicated test, not
      inferred from an LF-only fixture; a newly-inserted continuation line's indentation is one
      level deeper than the statement's own opening line, correct even though `indent_plan` has
      no entry for that synthetic line; a `; FMT: OFF`/`ON` protected `Control` statement
      receives no wrap edits even when over-width; `line_wrap: Preserve` (the default) produces
      byte-identical output to before this feature existed across a fixture containing a
      genuinely over-width `Control` statement (FR-007 regression case). Depends on T010.

**Checkpoint**: `LineWrapMode`/`LineWrapStyle` and `line_wrap.rs` are real, compiling, tested
parts of `voyager-core`, including verified quoted-value/bracket/paren safety; the
terminator-embedding render capability exists and is proven correct in isolation, including the
CRLF case. `cargo build --workspace` succeeds (adapters don't construct these types directly
yet — unaffected).

---

## Phase 3: User Story 1 - A project keeps long `Control` statements within a readable width (Priority: P1)

**Goal**: A project can set `line_wrap = "auto"` (with an optional width and wrap style) via
`drut.toml`, CLI, MCP, or the VS Code personal-setting mechanism, and see every over-width
`Control` statement in a real script wrapped at valid top-level comma split points — with
`preserve` (the default) producing zero change.

**Independent Test**: Format a script containing one over-width `Control` statement and one
under-width one, with `line_wrap = auto`, and confirm only the over-width one wraps, per
spec.md's own Acceptance Scenarios.

### Implementation for User Story 1

- [X] T013 [US1] In `crates/drut-config/src/lib.rs`: add `line_wrap: Option<LineWrapMode>`,
      `line_wrap_width: Option<u16>`, `line_wrap_style: Option<LineWrapStyle>` to both
      `FormatConfig` and `ExplicitFormatOverride`. Implement the single-tier precedence in
      `resolve_format_options` (data-model.md §3) for all three fields: `explicit.field
      .or(config.format.field).unwrap_or_default()`-shaped, with `line_wrap_width` additionally
      range-validated (mirroring `resolve_blank_line_cap`'s shape) before falling back to `120`
      on an out-of-range value. Depends on T002.
- [X] T014 [US1] In `crates/drut-config/src/parse.rs`: add TOML parsing for `line_wrap`
      (`"preserve"`/`"auto"`, case-insensitive), `line_wrap_width` (positive integer, a
      plan-phase-confirmed valid range), and `line_wrap_style` (`"fill"`/`"one_per_line"`,
      case-insensitive) under `[format]` — same non-blocking malformed-value-warns-and-falls-back
      pattern every existing `[format]` field already uses. Depends on T013.
- [X] T015 [US1] In `crates/drut-cli/src/cli.rs`: add `--line-wrap` (`Option<LineWrapArg>`,
      `ValueEnum`, same shape as `--blank-lines`), `--line-wrap-width` (`Option<u16>`, ranged
      `value_parser`, same shape as `--blank-lines-top-cap`), `--line-wrap-style`
      (`Option<LineWrapStyleArg>`, `ValueEnum`). In `crates/drut-cli/src/format_cmd.rs`: wire all
      three into `ExplicitFormatOverride`. Depends on T013.
- [X] T016 [US1] In `crates/drut-mcp/src/format.rs`: add `line_wrap`, `line_wrap_width`,
      `line_wrap_style` parameters to the `format` tool's input, same accepted-value shape and
      error-message pattern as the existing `blank_lines`/`blank_lines_top_cap` parameters.
      Depends on T013.
- [X] T017 [US1] In `editors/vscode/package.json`: add `drut.format.lineWrap`,
      `drut.format.lineWrapWidth`, `drut.format.lineWrapStyle` personal settings, same shape as
      every existing `drut.format.*` entry. Confirm and mirror whatever client-side threading
      the existing `drut.format.blankLines`/`blankLinesTopCap` entries already use (own task
      since this crosses into the extension's TypeScript, not `voyager-core`/`drut-config`).
      Depends on T013.

### Tests for User Story 1

- [X] T018 [P] [US1] Add tests to `crates/drut-config/tests/parse.rs`: `line_wrap` parses
      `"preserve"`/`"auto"` cleanly; `line_wrap_width` parses a valid positive integer cleanly
      and a malformed/out-of-range value warns and falls back to `120`; `line_wrap_style` parses
      `"fill"`/`"one_per_line"` cleanly; a malformed value for any of the three warns and falls
      back to that field's built-in default.
- [X] T019 [P] [US1] Add tests to `crates/drut-config/tests/resolve.rs`: an explicit CLI/MCP
      value overrides a `drut.toml`-resolved one, for all three fields independently; nothing
      configured anywhere resolves to `preserve`/`120`/`fill` (the built-in defaults).
- [X] T020 [P] [US1] Add tests to `crates/drut-cli/tests/format_flags.rs`: `--line-wrap=auto`
      (with and without `--line-wrap-width`/`--line-wrap-style`) overrides a
      `drut.toml`-resolved `preserve` for one run; an invalid `--line-wrap-style` value is a
      usage error, same as any other invalid `ValueEnum` value.
- [X] T021 [P] [US1] Add the equivalent tests to `crates/drut-mcp/src/format.rs`'s own test
      module, mirroring T020's shape at the MCP surface, including the invalid-value rejection
      case.
- [X] T022 [US1] Add an integration test (`crates/voyager-core/tests/`, real-corpus-shaped
      fixture, not synthetic-only, pulling from an over-width `Control` statement shape already
      seen in this project's own development) exercising spec.md's own Acceptance Scenarios
      directly: an over-width `Control` statement wraps at top-level commas under both `Fill`
      and `OnePerLine`; an under-width statement is untouched; a statement that already contains
      a continuation is untouched regardless of width; no `line_wrap` configuration leaves the
      same script byte-identical; a second `format()` pass on the wrapped output produces no
      further change (idempotence, spec.md Acceptance Scenario 5). Depends on Phase 2,
      T013-T016.
- [X] T023 [US1] Add a rejection test at each closed-set surface: a CLI `--line-wrap-style=tight`
      (or any value outside the closed set) is a usage error (`crates/drut-cli/tests/format_flags.rs`
      — may already be covered by T020, confirm no gap); an MCP `line_wrap_style: "tight"` param
      produces the same invalid-value error shape as an unrecognized `casing`/`blank_lines`
      string (`crates/drut-mcp/src/format.rs`'s own test module — may already be covered by
      T021, confirm no gap). Depends on T015, T016.

**Checkpoint**: User Story 1 independently proven — `Auto` (with configurable width and style)
is settable end-to-end at every surface including the VS Code extension, matches every one of
spec.md's Acceptance Scenarios, is confirmed idempotent by construction (a wrapped statement is
never re-touched), and is safe against invalid input at every surface — not assumed.

---

## Phase 4: Polish & Cross-Cutting Concerns

- [ ] T024 [P] In `ROADMAP.md`: add and mark done a new item for this feature, dated, pointing
      at this feature's spec directory — same pattern every other completed `ROADMAP.md` item
      already follows.
- [ ] T025 `cargo test --release --workspace` and `cargo clippy --workspace --all-targets --
      -D warnings`, both clean.
- [ ] T026 [P] Full real-corpus revalidation across CLI/LSP/MCP with **no new configuration
      supplied** — expected zero diagnostic/output change from before this feature (SC-003),
      reported as its own explicit result, not inferred from the unit-test suite alone.
- [X] T027 Format a handful of real corpus files *with* `line_wrap=auto` (default `Fill`, and
      explicitly `line_wrap_style=one_per_line`), hand-verify the diffs are exactly the expected
      wrapping changes at real over-width `Control` statements, then promote those diffs to new
      golden fixtures under `crates/voyager-core/tests/fixtures/golden_line_wrap_fill/` and
      `golden_line_wrap_one_per_line/` (research.md §6), with idempotence checks for both
      variants including the second-pass "already continued, never re-touched" case specifically
      (SC-004) — same discipline `018` already established. Depends on T026.
- [X] T028 Run `quickstart.md` end-to-end as written, confirming every step's expected result
      holds against the actual shipped code, not just against the individual task-level tests
      above in isolation.

**Checkpoint**: Feature-complete against spec.md; `ROADMAP.md` consistent with shipped code;
full workspace and full corpus re-proven clean; new golden fixtures added for both wrap styles.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS User Story 1.
- **User Story 1 (Phase 3)**: Depends on Foundational only.
- **Polish (Phase 4)**: T024 is independent of the code phases; T025-T028 depend on User Story 1
  being complete.

### Parallel Opportunities

- T005, T006 can run in parallel once T004 lands (different test cases, same test module — no
  logical dependency between them).
- T008 depends on T007 only.
- T011, T012 can run in parallel once their respective dependencies land.
- T018-T021 can run in parallel once T013-T016 land (different test files).

---

## Parallel Example: Once Foundational (T002-T012) Lands

```bash
Task: "T013: drut-config gains line_wrap/line_wrap_width/line_wrap_style fields"
Task: "T024: ROADMAP.md new item marked done"
```

---

## Implementation Strategy

### MVP First (this feature IS the MVP — single story)

1. Setup → baseline confirmed clean.
2. Foundational → wrapping capability (recognition, packing, terminator/indent-aware edit
   construction) exists, compiles, is tested in isolation, including the CRLF case.
3. User Story 1 → `line_wrap = auto` configurable end-to-end (including the VS Code extension),
   proven against spec.md's own Acceptance Scenarios.
4. **STOP and VALIDATE**: run T022 against a real corpus-shaped script.

### Incremental Delivery

1. Foundational → capability ready.
2. US1 → feature complete (there is no second increment for this feature).
3. Polish → `ROADMAP.md` update, full re-proof, golden fixtures for both wrap styles.

---

## Notes

- This feature deliberately narrows scope versus `018-operator-spacing`'s two-story shape
  (spec.md Assumptions: arithmetic-expression/bracket/paren wrapping explicitly deferred to a
  future increment) — single story, matching `029-unused-token-diagnostic`'s own single-story
  precedent for a feature this project scoped conservatively on purpose.
- T010 is this feature's single highest-risk task: it's the one place a genuinely new capability
  (a line-terminator embedded inside an edit's replacement, actually splitting a line for the
  first time) is added to a render path that has never done this before. T012's tests exist
  specifically to catch a subtle terminator/indentation bug here, not as mechanical coverage.
- T009's `wrap_edit` deliberately takes `terminator`/`continuation_indent` as plain caller-supplied
  strings rather than resolving them itself — keeping `line_wrap.rs` a pure, I/O-free,
  `indent_plan`-unaware module (data-model.md §1); `format.rs::render` (T010) is the only place
  that knows both the per-line terminator and the resolved indent width.
- Commit after each task or logical group.
