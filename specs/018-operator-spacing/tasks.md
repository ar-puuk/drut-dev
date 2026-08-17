---

description: "Task list for Operator Spacing Normalization"
---

# Tasks: Operator Spacing Normalization

**Input**: Design documents from `/specs/018-operator-spacing/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md
(all present)

**Tests**: Included — a public-API/CLI/config-surface shape change touching four crates, plus a
genuinely new recognition capability (`operator_spacing.rs`) and a new render-pipeline
edit-application mechanism (`SpacingEdit`), needs real coverage at every layer, matching `017`'s
own precedent for a feature of this shape.

**Organization**: Foundational carries the entire `Fixed`-equivalent capability — quote-state
tracking, operator/comma/bracket-paren recognition, the multi-char-comparison merge, unary/
binary disambiguation, continuation-position handling, and the new `SpacingEdit` render-pipeline
capability itself — without which neither story is meaningfully testable (`Auto` is `Fixed`
plus alignment, so it has nothing to build on without this). US1 (P1, `Fixed`) then builds the
config-surface plumbing (drut-config/CLI/MCP) that makes `operator_spacing` actually settable,
and proves `Fixed` end-to-end. US2 (P2, `Auto`) reuses US1's plumbing unchanged (one setting,
three values, already wired) and adds only the alignment-run logic layered on top of
Foundational's `Fixed` edits — genuinely additive, never a divergent code path.

**Everything in this file's scope was measured against the real, current codebase during
planning (research.md), not estimated**:

- Every operator character in scope (`= + - / * ^ & | < >`, plus `,`) is already tokenized as a
  standalone single-character `Punctuation` token (`lexer.rs`'s `is_delimiter`, confirmed by
  reading the current code) — recognition needs no lexer change. Multi-char comparisons (`==`,
  `<>`, `>=`, `<=`) are **not** single tokens today; a zero-gap-adjacency merge step is added in
  the new module, not the shared lexer (research.md §2).
- `StatementKind::Assignment { value: Vec<Token> }` and `Control`'s `pairs: Vec<(String,
  Vec<Token>)>` already store each value as its own ordered token list — unary/binary
  disambiguation is a token-lookback problem, not an expression-parsing one (research.md §5).
  No new parsing capability needed.
- `format.rs::render`'s existing `CasingEdit` application is a same-length in-place column
  splice (confirmed directly in the current code: `repl_chars.len() == end - start` is a hard
  guard) — it silently no-ops any variable-length edit. This is why `SpacingEdit` needs its own
  application path, not a reuse of the existing one (research.md §4).
- Block nesting is already one `Vec<Node>` per level — "same block nesting depth" for alignment
  runs is free sibling-adjacency, not a depth counter that needs building (research.md §6).
- **Confirmed by direct testing, not assumed**: `tokenize("LIST='a+b'\n")` emits a standalone
  `Punctuation("+")` token for the `+` *inside* the quotes — the lexer's quote-tracking only
  gates comment recognition, never operator/delimiter recognition. Operator recognition MUST do
  its own independent quote-tracking (research.md §9) — this is a real, verified correctness
  requirement, not a defensive nicety, caught during pre-implementation checklist review.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete sibling task)
- **[Story]**: US1/US2 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/src/operator_spacing.rs` (new) — the quote-tracking/recognition/merge/
  disambiguation module, its `Fixed`-edit collection, and (Phase 4) alignment-run detection.
- `crates/voyager-core/src/format.rs` — `OperatorSpacing` enum, `FormatOptions.operator_spacing`
  field, the new `SpacingEdit` type and per-line rebuild path in `render()`, the module doc
  comment update.
- `crates/voyager-core/src/lib.rs` — new re-export.
- `crates/drut-config/src/lib.rs`, `src/parse.rs` — new field, precedence resolution, TOML
  parsing.
- `crates/drut-config/tests/parse.rs`, `tests/resolve.rs` — new coverage.
- `crates/drut-cli/src/cli.rs`, `src/format_cmd.rs`, `tests/format_flags.rs` — new flag.
- `crates/drut-mcp/src/format.rs` (own test module included) — new param.
- `crates/drut-lsp/` — no source changes expected; existing suite passing unmodified after the
  type change compiles through is the confirmation, same as `014`/`017`.
- `ROADMAP.md` — item 12 marked done (Polish).

---

## Phase 1: Setup

- [X] T001 Confirm baseline: `cargo build --workspace` and `cargo clippy --workspace
      --all-targets -- -D warnings` both clean, on this branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The entire `Fixed`-equivalent capability — quote-tracking, recognition, merge,
disambiguation, edit collection — plus the render-pipeline's new ability to apply
variable-length edits at all. Neither story is meaningfully testable until this compiles and its
own tests pass.

- [X] T002 In `crates/voyager-core/src/format.rs`: add `OperatorSpacing` enum (`Preserve`
      `#[default]`, `Fixed`, `Auto` — data-model.md §1). Add `operator_spacing: OperatorSpacing`
      to `FormatOptions`; extend the existing manual `impl Default for FormatOptions` (already
      manual since `017`'s `indent_width`) with `operator_spacing: OperatorSpacing::default()`.
      Update `FormatOptions`'s doc comment.
- [X] T003 Create `crates/voyager-core/src/operator_spacing.rs` with `quoted_token_mask(tokens:
      &[Token]) -> Vec<bool>` (data-model.md §1, research.md §9): walks a statement's token
      list maintaining a running odd/even count of `'`/`"` `Punctuation` tokens; a token is
      `true` (inside a string) when that count is currently odd, or once an unmatched trailing
      quote is seen, every token after it (fail toward exclusion, never toward false
      recognition). Add a unit test confirming `tokenize`'s actual, verified behavior:
      `LIST='a+b'` produces a `Punctuation("+")` token that `quoted_token_mask` correctly marks
      `true`. Depends on T002 (module wiring only).
- [X] T004 In `crates/voyager-core/src/operator_spacing.rs`: `OperatorKind` (`Assignment`,
      `Comparison(ComparisonOp)`, `Arithmetic(ArithmeticOp)`, `Comma`), `OperatorOccurrence
      { kind, span, is_continuation }` (data-model.md §1). `recognize_operators(tokens: &[Token])
      -> Vec<OperatorOccurrence>`: consults T003's `quoted_token_mask` first and skips any masked
      token entirely; for everything else, merges two adjacent zero-gap `=`/`<`/`>` `Punctuation`
      tokens (same line, first's `span.end ==` second's `span.start`) into one `Comparison`
      occurrence for `==`/`<>`/`>=`/`<=` (research.md §2), and recognizes lone `=`/`<`/`>`, binary
      `+`/`-`/`*`/`/`, and `,` as their own occurrences. `is_binary_arithmetic(tokens, index)`: a
      `+`/`-` is binary unless the previous token in the same list is absent, or is itself `=`,
      `(`, `,`, or another recognized operator (research.md §5) — unary occurrences are excluded
      from `recognize_operators`'s output entirely (no edit is ever queued for them).
      `is_continuation` is set from the token's `TokenKind::ContinuationMarker` tag (research.md
      §3). Pure, no I/O, never panics. Depends on T003.
- [X] T005 [P] In `crates/voyager-core/src/operator_spacing.rs`: `collect_bracket_paren_edits
      (tokens: &[Token]) -> Vec<SpacingEdit>` (research.md §7) — consults T003's
      `quoted_token_mask` first, same as T004; zero interior padding between `[`/`(` and the
      following token, and between the preceding token and `]`/`)`; zero space between a
      `Control` statement's `word` token and an immediately-following `(` (the short-form
      `IF(x)` case). Same token-pair-adjacency shape as T004's operator recognition, kept
      separate because its "always zero space" rule differs from operators' "always one space"
      rule. Depends on T003.
- [X] T006 In `crates/voyager-core/src/operator_spacing.rs`: `collect_operator_edits(tokens:
      &[Token]) -> Vec<SpacingEdit>` — for each `OperatorOccurrence` from T004, emit an edit
      normalizing the whitespace immediately before it (and, unless `is_continuation`, the
      whitespace immediately after it) to exactly one space; for `Comma`, one space after and
      none before. Depends on T004.
- [X] T007 In `crates/voyager-core/src/format.rs`: add the `SpacingEdit` type alias (data-model.md
      §2, same `(u32, usize, usize, String)` shape as `CasingEdit` but without the same-length
      contract). In `render()`: add a per-line rebuild path used only when a line has queued
      `SpacingEdit`s — merge that line's `CasingEdit`s and `SpacingEdit`s sorted by start column
      (disjoint spans, safe to interleave), walk left-to-right copying untouched gaps verbatim
      and splicing in each edit's replacement (data-model.md §2's algorithm); lines with no
      spacing edits keep using the existing same-length splice unchanged. Wire
      `operator_spacing::collect_operator_edits`/`collect_bracket_paren_edits` into `render()`,
      called once per statement's token list, gated behind `options.operator_spacing !=
      OperatorSpacing::Preserve` (mirrors casing's existing short-circuit). Extend the
      `; FMT: OFF`/`ON` protected-line funnel (`push_if_present`'s pattern) to spacing edits — a
      protected line receives none. Reword the module's "Scope, precisely" doc comment
      (research.md §8, data-model.md §5): still exactly true for `Preserve`, now points to this
      feature for what `Fixed`/`Auto` additionally do. Depends on T005, T006.
- [X] T008 [P] In `crates/voyager-core/src/lib.rs`: re-export `OperatorSpacing`.
- [X] T009 [P] Add unit tests to `crates/voyager-core/src/operator_spacing.rs`'s own test
      module: assignment `=`; each comparison including the two-token-merge regression case
      (`I==1` normalizes to one `==`-shaped gap, never a stray inner space); binary vs. unary
      `+`/`-` (`MW[1] = -5` stays tight, `MW[1] = A - B` gets two-sided spacing, `A + -B`
      normalizes the `+` to two-sided but keeps `-B` tight); comma spacing between `Control`
      pairs; bracket/paren interior padding; control-word-paren adjacency (`IF ( x )` →
      `IF(x)`); a trailing continuation-position operator gets leading-only spacing (no trailing
      space inserted); **a quoted-literal regression test** — `LIST='a+b'` (and a case with an
      unnormalized `=` inside quotes) produces zero edits for anything inside the quotes,
      confirmed against `recognize_operators`'/`collect_bracket_paren_edits`' actual output, not
      just `quoted_token_mask` in isolation (T003 already covers that). Depends on T003, T004,
      T005, T006.
- [X] T010 [P] Add unit tests to `crates/voyager-core/src/format.rs`'s test module: a line with
      multiple spacing edits renders correctly via the new rebuild path with no corrupted
      offsets; a line with both a casing edit and a spacing edit applies both correctly in one
      pass; a `; FMT: OFF`/`ON` protected line receives no spacing edits; `operator_spacing:
      Preserve` (the default) produces byte-identical output to before this feature existed
      across a fixture exercising every operator kind (FR-009 regression case). Depends on T007.

**Checkpoint**: `OperatorSpacing` and `operator_spacing.rs` are real, compiling, tested parts of
`voyager-core`, including verified quoted-literal safety; the `SpacingEdit` render capability
exists and is proven correct in isolation. `cargo build --workspace` succeeds (adapters don't
construct `OperatorSpacing` directly yet — unaffected).

---

## Phase 3: User Story 1 - A project normalizes inconsistent operator spacing (Priority: P1)

**Goal**: A project can set `operator_spacing = "fixed"` via `drut.toml`, CLI, or MCP, and see
every in-scope operator/comma/bracket-paren occurrence in a real script normalized in one pass —
with `preserve` (the default) producing zero change.

**Independent Test**: Format a script containing inconsistent `=` spacing, a comparison inside
an `IF`, an arithmetic expression, a multi-pair `Control` statement, and a subscript/
parenthesized reference, with `operator_spacing = fixed`, and confirm every one normalizes per
spec.md US1's Acceptance Scenarios.

### Implementation for User Story 1

- [X] T011 [US1] In `crates/drut-config/src/lib.rs`: add `operator_spacing:
      Option<OperatorSpacing>` to both `FormatConfig` and `ExplicitFormatOverride`. Implement
      the single-tier precedence in `resolve_format_options` (data-model.md §4):
      `explicit.operator_spacing.or(config.format.operator_spacing).unwrap_or_default()`.
      Depends on T002 (needs `OperatorSpacing` to construct the resolved `FormatOptions`).
- [X] T012 [US1] In `crates/drut-config/src/parse.rs`: add TOML parsing for `operator_spacing`
      under `[format]`, accepting `"preserve"`/`"fixed"`/`"auto"` case-insensitive, same
      non-blocking malformed-value-warns-and-falls-back pattern every existing `[format]` field
      already uses. Depends on T011.
- [X] T013 [US1] In `crates/drut-cli/src/cli.rs`: add `--operator-spacing`, `Option<OperatorSpacingArg>`
      with the same `ValueEnum` shape (`preserve`/`fixed`/`auto`) `--casing`/`--top-level-indent`
      already use, and the same "no bare flag" usage-error rule (`002-cli-check-format`
      FR-015). In `crates/drut-cli/src/format_cmd.rs`: wire it into `ExplicitFormatOverride`.
      Depends on T011.
- [X] T014 [US1] In `crates/drut-mcp/src/format.rs`: add an `operator_spacing` string parameter
      to the `format` tool's input, same accepted-value shape and error-message pattern as the
      existing `casing`/`top_level_indent` parameters. Depends on T011.

### Tests for User Story 1

- [X] T015 [P] [US1] Add tests to `crates/drut-config/tests/parse.rs`: `operator_spacing` parses
      each of `"preserve"`/`"fixed"`/`"auto"` cleanly; a malformed value (e.g. `"tight"`) warns
      and falls back to `preserve`.
- [X] T016 [P] [US1] Add tests to `crates/drut-config/tests/resolve.rs`: an explicit CLI/MCP
      value overrides a `drut.toml`-resolved one; nothing configured anywhere resolves to
      `preserve` (the built-in default).
- [X] T017 [P] [US1] Add a test to `crates/drut-cli/tests/format_flags.rs`: `--operator-spacing=fixed`
      overrides a `drut.toml`-resolved `preserve` for one run.
- [X] T018 [P] [US1] Add the equivalent test to `crates/drut-mcp/src/format.rs`'s own test
      module, mirroring T017's shape at the MCP surface.
- [X] T019 [US1] Add an integration test (`crates/voyager-core/tests/`, real-corpus-shaped
      fixture, not synthetic-only) exercising spec.md US1's own Acceptance Scenarios directly:
      `ZONES   = 1` → `ZONES = 1`; `MATI=a.mat,MATO=b.mat` → `MATI = a.mat, MATO = b.mat`;
      `IF ( x==1 )` → `IF(x == 1)`; `MW[ 1 ]=mi.1.1+mi.2.1` → `MW[1] = mi.1.1 + mi.2.1`; a
      negative literal (`MW[1] = -5`) stays tight; a `LIST='a+b'`-shaped statement is left
      byte-identical inside the quotes; no `operator_spacing` configuration leaves the same
      script byte-identical (US1 AS2). Depends on Phase 2, T011-T014.
- [X] T020 [US1] Added post-`/speckit-analyze` (finding I1 — plan.md's Constitution Check
      claimed axis-specific idempotence/`; FMT: OFF` re-verification with no corresponding fast
      unit-level task; the only prior coverage was a late corpus-based Polish task). In
      `crates/voyager-core/src/format.rs`'s test module: an idempotence test —
      `format(format(x, opts).text, opts).text == format(x, opts).text` — for `opts` with
      `operator_spacing: Fixed`, run against a fixture exercising every operator kind; and a
      `; FMT: OFF`/`; FMT: ON` interaction test — a fixture with a protected region containing
      unnormalized operator spacing, formatted with `operator_spacing: Fixed`, confirming the
      protected region is left exactly as written while an unprotected region elsewhere in the
      same file is normalized. Depends on T007.
- [X] T021 [US1] Added post-`/speckit-analyze` (finding C1 — spec.md SC-004 originally claimed
      CLI/MCP invalid values degrade to `preserve` with a notice, but `operator_spacing` is a
      closed `ValueEnum` like `casing`, so CLI/MCP structurally reject an invalid value at their
      own input point instead; no task tested this). Add a rejection test at each surface: a CLI
      `--operator-spacing=tight` (or any value outside the closed set) is a usage error, same as
      any other invalid `ValueEnum` value (`crates/drut-cli/tests/format_flags.rs`); an MCP
      `operator_spacing: "tight"` param produces the same invalid-value error as an unrecognized
      `casing`/`top_level_indent` string (`crates/drut-mcp/src/format.rs`'s own test module).
      Depends on T013, T014.

**Checkpoint**: User Story 1 independently proven — `Fixed` is configurable end-to-end at every
surface, matches every one of spec.md's US1 Acceptance Scenarios, is confirmed idempotent,
`; FMT: OFF`-respecting, quoted-literal-safe, and safe against invalid input at every surface —
not assumed.

---

## Phase 4: User Story 2 - A project vertically aligns consecutive assignments (Priority: P2)

**Goal**: `operator_spacing = "auto"` (the same setting US1 already wired end-to-end) makes
consecutive `Assignment` statements' `=` vertically align, resetting independently at a blank
line, a comment-only line, a nesting-depth/non-`Assignment`-sibling change, or a `; FMT: OFF`
protected member.

**Independent Test**: Format a script with several consecutive `Assignment` statements of
varying left-hand-side length, interrupted by a blank line, then a comment-only line, then an
indentation-depth change, with `operator_spacing = auto`, and confirm each resulting group
aligns independently per spec.md US2's Acceptance Scenarios.

### Implementation for User Story 2

- [X] T022 [US2] In `crates/voyager-core/src/operator_spacing.rs`: add `AlignmentRun {
      members: Vec<AssignmentMember>, target_column: usize }` and `AssignmentMember {
      equals_span: Span, lhs_width: usize }` (data-model.md §3). Add a function that walks one
      `Vec<Node>` slice (a block's `children`, or the top-level node list) and groups maximal
      runs of consecutive `Node::Statement` entries whose `kind` is `Assignment`, breaking at:
      any non-`Assignment` sibling (including a pair-keyword-shaped `Control` statement, FR-007),
      a blank source line between two siblings' spans, a comment-only source line between them,
      or an `Assignment` statement whose own line falls inside a `; FMT: OFF`/`; FMT: ON`
      protected region (FR-008) — a protected member is excluded from the run entirely, never
      counted toward a neighbor's `target_column`, not merely skipped-while-still-counted.
      Reuses the same line-classification approach `protected_regions` already uses, not a new
      mechanism. Depends on T004 (module already exists).
- [X] T023 [US2] In `crates/voyager-core/src/operator_spacing.rs`: for each `AlignmentRun` with
      more than one member, compute `target_column` from the widest `lhs_width` in the run and
      emit additional padding `SpacingEdit`s inserting extra spaces before each shorter member's
      `=` (never touching members already at the target column, never touching a run of exactly
      one member — spec.md US2 Acceptance Scenario 4). This runs *after* T006's `Fixed`-shaped
      edits are already known for those statements — alignment only ever adds padding on top,
      never recomputes base spacing (contracts/operator-spacing.md). Depends on T022, T006.
- [X] T024 [US2] In `crates/voyager-core/src/format.rs`'s `render()`: gate a call into T022/
      T023's alignment-run detection behind `options.operator_spacing ==
      OperatorSpacing::Auto`, invoked after the `Fixed`-equivalent pass (T007's existing wiring),
      walking `nodes` and each block's `children` recursively. Depends on T007, T023.

### Tests for User Story 2

- [X] T025 [P] [US2] Add unit tests to `crates/voyager-core/src/operator_spacing.rs`'s own test
      module: three consecutive `Assignment` statements with differing left-hand-side lengths
      align to the longest one's column; a blank line, a comment-only line, and an
      indentation-depth change each independently split one run into separately-aligned runs; a
      pair-keyword-shaped `Control` statement among `Assignment` statements is spaced per
      `Fixed` only and splits the run; a lone `Assignment` statement renders identically to what
      `Fixed` alone would produce; a protected (`; FMT: OFF`) `Assignment` statement in the
      middle of an otherwise-alignable run splits it into two independently-aligned runs and
      receives no padding itself. Depends on T022, T023.
- [X] T026 [US2] Add an integration test (`crates/voyager-core/tests/`, real-corpus-shaped
      fixture with several consecutive assignments) exercising spec.md US2's Acceptance
      Scenarios end-to-end via `format()`, not just the module-level unit tests from T025.
      Depends on T024.
- [X] T027 [US2] Added post-`/speckit-analyze` (finding I1, `Auto` half). In
      `crates/voyager-core/src/format.rs`'s test module: an idempotence test —
      `format(format(x, opts).text, opts).text == format(x, opts).text` — for `opts` with
      `operator_spacing: Auto`, run against a multi-assignment fixture (aligned padding must not
      grow further on a second pass); and a `; FMT: OFF`/`; FMT: ON` interaction test — a
      fixture with a protected region containing consecutive `Assignment` statements that would
      otherwise form an alignment run, formatted with `operator_spacing: Auto`, confirming the
      protected region's spacing is left exactly as written while an unprotected run elsewhere
      in the same file aligns normally. Depends on T024.

**Checkpoint**: User Story 2 independently proven — `Auto` aligns correctly, resets at every
documented break condition (including a protected member), never diverges from `Fixed`'s own
base spacing decisions, and is confirmed idempotent and `; FMT: OFF`-respecting — not assumed.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: `ROADMAP.md` update and whole-workspace/full-corpus re-proof, once both stories are
done.

- [X] T028 [P] In `ROADMAP.md`: mark item 12 done, dated, pointing at this feature's spec
      directory — same pattern every other completed `ROADMAP.md` item already follows.
- [X] T029 `cargo test --release --workspace` and `cargo clippy --workspace --all-targets --
      -D warnings`, both clean.
- [X] T030 [P] Full 161-file real-corpus revalidation across CLI/LSP/MCP with **no new
      configuration supplied** — expected zero diagnostic/output change from before this
      feature (SC-003), reported as its own explicit result, not inferred from the unit-test
      suite alone.
- [X] T031 Format a handful of real corpus files *with* `operator_spacing=fixed`, and (on a file
      containing several consecutive assignments) `operator_spacing=auto` (quickstart.md step
      6), hand-verify the diffs are exactly the expected spacing/alignment changes, then promote
      those diffs to new golden fixtures under `crates/voyager-core/tests/fixtures/golden*/`,
      with idempotence checks for both new variants (SC-005) — same discipline `017` already
      established. Depends on T030.
- [X] T032 Run `quickstart.md` end-to-end as written, confirming every step's expected result
      holds against the actual shipped code, not just against the individual task-level tests
      above in isolation.

**Checkpoint**: Feature-complete against spec.md; `ROADMAP.md` consistent with shipped code;
full workspace and full corpus re-proven clean; new golden fixtures added for both new modes.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS both user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational only.
- **User Story 2 (Phase 4)**: Depends on Foundational **and** Phase 3's config-surface plumbing
  (T011-T014) — the same setting/flag/param `auto` is selected through is Phase 3's own
  deliverable, so US2's independent test needs it to exist first. Called out explicitly, same as
  `017`'s US2/US1 dependency.
- **Polish (Phase 5)**: T028 is independent of the code phases; T029-T032 depend on both stories
  being complete.

### Parallel Opportunities

- T005 can run in parallel with T004 (different concerns, same file — sequence by whichever
  lands first in practice; no logical dependency between them beyond both needing T003).
- T008, T009, T010 can run in parallel once their respective dependencies land.
- T015-T018 can run in parallel once T011-T014 land (different test files).
- T025 can run in parallel with T024's own implementation once T022/T023 land (different files:
  module unit tests vs. `render()` wiring).

---

## Parallel Example: Once Foundational (T002-T010) Lands

```bash
Task: "T011: drut-config gains operator_spacing field + precedence"
Task: "T028: ROADMAP.md item 12 marked done"
```

---

## Implementation Strategy

### MVP First (User Story 1 alone)

1. Setup → baseline confirmed clean.
2. Foundational → `Fixed`-equivalent recognition/edit-collection (including quote-tracking) and
   the `SpacingEdit` render capability exist, compile, are tested in isolation.
3. User Story 1 → `operator_spacing = fixed` configurable end-to-end, proven against spec.md's
   own Acceptance Scenarios.
4. **STOP and VALIDATE**: run T019 against a real corpus-shaped script.

### Incremental Delivery

1. Foundational → capability ready.
2. US1 → MVP (`fixed` ships, fully functional on its own).
3. US2 → `auto` alignment, layered on top, whenever convenient.
4. Polish → `ROADMAP.md` update, full re-proof, golden fixtures for both modes.

---

## Notes

- T004/T006's split (recognition vs. edit-emission) mirrors `017`'s own `data_reference.rs`
  shape (recognition module, edit application funneled through `format.rs`) — not a new pattern.
- T003's `quoted_token_mask` was added during pre-implementation checklist review (CHK002):
  the original design assumed the token stream alone already distinguished string-literal
  content from real operators, which direct testing disproved (`tokenize("LIST='a+b'\n")`
  emits an indistinguishable `Punctuation("+")` for the in-string `+`). Every recognition
  function (T004, T005) depends on T003 specifically because of this — not a defensive
  afterthought, a verified correctness requirement.
- T007 is this feature's single highest-risk task: it's the one place a genuinely new capability
  (variable-length edit application) is added to a render path that today only ever does
  same-length splices. T010's tests exist specifically to catch a subtle offset-corruption bug
  here, not as mechanical coverage.
- T020/T021 (US1) and T027 (US2) were added after an initial `/speckit-analyze` pass found two
  real gaps: I1 (plan.md promised axis-specific idempotence/`; FMT: OFF` re-verification for
  both `Fixed` and `Auto`, with the only actual coverage being a late, corpus-based Polish task
  — and the alignment-padding emission wasn't yet routed through the protected-line funnel at
  all, a real correctness gap fixed directly in T022/T023's own description) and C1 (spec.md
  SC-004 originally claimed CLI/MCP invalid values degrade to `preserve` with a notice, but
  `operator_spacing`'s closed `ValueEnum` shape means those two surfaces actually reject an
  invalid value outright, same as `casing` today — spec.md was corrected to match, and the
  rejection behavior itself was previously untested). Full task numbering was cleanly
  renumbered once more (rather than appended out-of-sequence) when T003 was added during
  checklist review, since no task had been started yet.
- Commit after each task or logical group.
