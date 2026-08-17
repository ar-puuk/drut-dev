---

description: "Task list for Per-Category Casing Configuration and Configurable Indentation Width"
---

# Tasks: Per-Category Casing Configuration and Configurable Indentation Width

**Input**: Design documents from `/specs/017-casing-categories-indent-width/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md
(all present)

**Tests**: Included — a public-API/CLI/config-surface shape change touching four crates, plus a
genuinely new recognition capability (`data_reference.rs`) and this project's first numeric
(not closed-enum) configurable value, needs real coverage at every layer, matching `014`'s own
precedent for a feature of this shape.

**Organization**: Foundational carries everything all three stories share — the
`CasingSettings` type, the `data_reference.rs` recognition module and its wiring into the
formatter (without which `data_references` would be a no-op category, undermining US1's own
independent test), and the `keywords.rs` dictionary corrections. US1 (P1) then builds the
config-surface plumbing (drut-config/CLI/MCP) that lets all three categories actually be set
independently. US2 (P1) is mostly a **verification** story, not a construction one (same shape
`014`'s US2 had) — it proves the literal reported gap (`mw`/`li`/`ni`/`i`/`j` unreachable) is
closed, reusing Foundational's + US1's machinery rather than adding new production code. US3
(P2) is genuinely independent — a different `FormatOptions` field, touching the same struct
definition sequentially (a file dependency, not a logical one).

**Everything in this file's scope was measured against the real, current codebase during
planning (research.md §1, plan.md), not estimated**:

- `collect_casing_edits`/`collect_block_casing_edits`/`collect_statement_casing_edits`
  (`format.rs`) currently thread **one** `convention: CasingConvention` uniformly through
  *both* control-word spans and pair-keyword-name spans — confirmed by reading the current
  code, not assumed. Splitting these into two independently-configurable values is a real,
  scoped change to these three functions' call sites, not just a type-signature change.
- `edit_for_span`/`push_if_present` need **no signature change at all** — they already take one
  `CasingConvention` per call and are called once per span; the split happens at the *caller*
  level (which convention gets passed for which span), not here.
- `edit_for_span`'s `Preserve` arm already produces `replacement == original` (a guaranteed
  no-op) — confirmed directly in the current code. `render()`'s outer gate is a performance
  short-circuit, not a correctness requirement; the 3-category version keeps an equivalent
  short-circuit (skip the walk only if all three settings are `Preserve`) rather than needing
  one gate per category.
- `StatementKind::Assignment { target: String, .. }` already stores bracket-inclusive text
  verbatim (confirmed: an existing test asserts `target == "MW[1]"`) — `data_reference.rs`
  needs no new parsing for the assignment-target shape, only a small base-name-before-`[`
  helper (research.md §1).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete sibling task)
- **[Story]**: US1/US2/US3 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/src/format.rs` — `CasingSettings`, the `FormatOptions.casing` type
  change, the three `collect_*_casing_edits` functions' split, `render()`'s gate, the new
  `indent_width`-parameterized indentation math (US3), and this crate's own test module.
- `crates/voyager-core/src/data_reference.rs` (new) — the recognition module.
- `crates/voyager-core/src/keywords.rs` — `NUMREC`-family removal, `ZONES` addition.
- `crates/voyager-core/src/lib.rs` — new re-exports.
- `crates/drut-config/src/lib.rs`, `src/parse.rs` — new fields, precedence resolution,
  `indent_width` bound validation.
- `crates/drut-config/tests/parse.rs`, `tests/resolve.rs` — new coverage.
- `crates/drut-cli/src/cli.rs`, `src/format_cmd.rs`, `tests/format_flags.rs` — new flags.
- `crates/drut-mcp/src/format.rs` (own test module included) — new params.
- `crates/drut-lsp/` — no source changes expected; existing suite passing unmodified after the
  type change compiles through is the confirmation, same as `014`'s US3 finding.
- `specs/002-cli-check-format/spec.md`, `specs/001-voyager-script-parser/contracts/
  public-api.md` — dated amendments (Polish).
- `ROADMAP.md` — items 9/10 marked done (Polish).

---

## Phase 1: Setup

- [X] T001 Confirm baseline: `cargo build --workspace` and `cargo clippy --workspace
      --all-targets -- -D warnings` both clean, on this branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The shared core every story depends on — `CasingSettings` existing as a real
3-field type, `data_references` tokens being genuinely recognizable and rewritable at all, and
the `keywords.rs` dictionary corrections. No user story is meaningfully testable until this
compiles and its own tests pass.

- [X] T002 In `crates/voyager-core/src/format.rs`: add `CasingSettings` struct (`control_words:
      CasingConvention, pair_keywords: CasingConvention, data_references: CasingConvention`,
      `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]` — each field's own
      `CasingConvention::default() == Preserve` makes the derive correct with no manual impl
      needed, per data-model.md §1). Change `FormatOptions.casing` from `CasingConvention` to
      `CasingSettings`; `FormatOptions` keeps `#[derive(Default)]` at this point (no field yet
      needs a non-derived default — that changes in T0xx/US3). Update both types' doc comments
      to describe the new per-category shape.
- [X] T003 In `crates/voyager-core/src/format.rs`: split `collect_casing_edits`/
      `collect_block_casing_edits`/`collect_statement_casing_edits`'s single `convention:
      CasingConvention` parameter into `settings: CasingSettings`. Apply
      `settings.control_words` at every existing control-word call site (block openers'
      closer/branch words, `first_word_span` results, the statement control-word lookup) and
      `settings.pair_keywords` at every existing pair-keyword-name call site (`block.
      opener_pairs`, the `pair_keyword_boundaries` loop) — a direct 1:1 mapping of today's
      single-convention call sites to the correct new per-category field, not new logic.
      `edit_for_span`/`push_if_present` are unchanged (still take one `CasingConvention` per
      call). Update `render()`'s gate from `if options.casing != CasingConvention::Preserve`
      to a three-field check (skip the walk only if `control_words`, `pair_keywords`, *and*
      `data_references` are all `Preserve`) — a performance short-circuit, not a new
      correctness requirement (`edit_for_span`'s `Preserve` arm is already a guaranteed no-op).
      Depends on T002.
- [X] T004 Create `crates/voyager-core/src/data_reference.rs`: `DataReferenceEntry { name:
      &'static str }`, a `const DATA_REFERENCE_ENTRIES` table with every member from
      research.md §6 (`MI`, `MO`, `MW`, `LI`, `LW`, `NI`, `NW`, `ZI`, `ZONES`, `Z`, `DBI`,
      `DBA`, `RO`, `A`, `B`, `I`, `J`), `data_reference_entries() -> &'static
      [DataReferenceEntry]`. `DataReferenceOccurrence { name: String, span: Span }` and
      `data_reference_occurrences(nodes: &[Node]) -> Vec<DataReferenceOccurrence>` (data-model.md
      §1) covering all three shapes: (a) a `TokenKind::Word` token whose text case-insensitively
      starts with `"<entry>."` (dot-notation read) — match span covers only the prefix, not the
      dot or what follows; (b) a pair-keyword name (reuse `pair_keyword_boundaries`) whose text,
      stripped of an optional trailing `[...]`, case-insensitively matches an entry — match span
      covers only the base-name portion; (c) an `Assignment.target` string, same stripping rule
      as (b). Share one base-name-stripping helper between (b) and (c) (research.md §1). No
      panics on any input; pure, no I/O. Depends on T002 (needs `Node`/`Span` already in scope,
      no other dependency).
- [X] T005 In `crates/voyager-core/src/format.rs`'s `render()`: call
      `data_reference::data_reference_occurrences` once per document, and for each occurrence
      call the existing `push_if_present`/`edit_for_span` machinery with
      `settings.data_references` — the third source of casing edits, alongside T003's two.
      Depends on T003, T004.
- [X] T006 [P] In `crates/voyager-core/src/keywords.rs`: remove the `NUMREC`, `CNT`, `ITER`,
      `LP`, `RECNUM` `pair_entry(...)` rows from `PAIR_KEYWORDS` (all previously
      `observed_with: ["LOOP"]`); add `pair_entry("ZONES", &["RUN"])`. Update the module-doc
      entry count/rationale to note this correction and its rationale (research.md §5), the
      same way the module docs already narrate the original census's own corrections. Depends
      on nothing (independent of T002-T005).
- [X] T007 In `crates/voyager-core/src/format.rs`'s test module: update `upper()`'s `casing:
      CasingConvention::Upper` to `casing: CasingSettings { control_words:
      CasingConvention::Upper, pair_keywords: CasingConvention::Upper, data_references:
      CasingConvention::Preserve }` — **exactly** reproducing old `Upper`'s reach (control
      words + pair keywords, never data-references) so every existing assertion that used
      `upper()` keeps passing unmodified. Same translation for `normalize()`'s `casing:
      CasingConvention::Preserve` → `casing: CasingSettings::default()`, and for any other
      compiler-forced `CasingConvention`-typed struct-literal fields the compiler flags in this
      module. Depends on T002, T003.
- [X] T008 [P] Add unit tests to `crates/voyager-core/src/data_reference.rs`'s own test module:
      one test per family in research.md §6's table (dot-notation shape, pair-keyword-name
      shape where applicable, assignment-target shape where applicable); a dedicated test that
      `MW` matched via its pair-keyword-shaped usage and via its assignment-target-shaped usage
      both produce `name == "MW"` (FR-005's structural guarantee, checked at the recognition
      layer); a negative-case test that an ordinary user variable name (e.g. `ScenarioDir`) is
      never matched. Depends on T004.
- [X] T009 [P] Update `crates/voyager-core/src/keywords.rs`'s test module: existing tests
      referencing `NUMREC`/`CNT`/`ITER`/`LP`/`RECNUM` removed/updated; new test confirming
      `completion_candidates` for `LOOP` no longer contains any of the five removed names; new
      test confirming `completion_candidates` for `RUN` contains `ZONES`. Depends on T006.

- [X] T038 Added post-`/speckit-analyze` (finding I1 — `plan.md`'s Constitution Check claimed
      this re-verification without a corresponding task). In
      `crates/voyager-core/src/format.rs`'s test module: an idempotence test —
      `format(format(x, opts).text, opts).text == format(x, opts).text` — for `opts` with
      `data_references: Upper` (or `Lower`) set, run against a fixture containing multiple
      data-reference tokens; and a `; FMT: OFF`/`; FMT: ON` interaction test — a fixture with a
      protected region containing a data-reference token (e.g. `mw`), formatted with
      `data_references: Upper`, confirming the protected occurrence is left exactly as written
      while an unprotected occurrence elsewhere in the same file is uppercased. Depends on T005.

**Checkpoint**: `CasingSettings` and `data_reference.rs` are real, compiling, tested parts of
`voyager-core`; `keywords.rs` reflects both corrections; `data_references` casing is confirmed
idempotent and `; FMT: OFF`-respecting, not assumed. `cargo build --workspace` succeeds
(adapters' own `casing` handling is still single-value at this point — unaffected, since none
of them construct `CasingSettings` directly yet).

---

## Phase 3: User Story 1 - A project sets its own casing convention per token category (Priority: P1)

**Goal**: A project can independently configure `control_words`, `pair_keywords`, and
`data_references` casing via `drut.toml`, CLI flags, or MCP params — with every already-shipped
surface (`casing`, `--casing`, the MCP `casing` param) continuing to mean exactly what it means
today.

**Independent Test**: With a project configured to three different values for the three
categories, format a script mixing all three token kinds and confirm each category's casing
changed independently.

### Implementation for User Story 1

- [X] T010 [US1] In `crates/drut-config/src/lib.rs`: add `control_words_casing`,
      `pair_keywords_casing`, `data_references_casing: Option<CasingConvention>` to both
      `FormatConfig` and `ExplicitFormatOverride` (existing `casing: Option<CasingConvention>`
      field on both, unchanged — data-model.md §3). Implement the full precedence matrix in
      `resolve_format_options`: for `control_words`/`pair_keywords`, `explicit.<category>_casing
      .or(explicit.casing).or(config.format.<category>_casing).or(config.format.casing)
      .unwrap_or_default()`; for `data_references`, `explicit.data_references_casing.or
      (config.format.data_references_casing).unwrap_or_default()` (legacy `casing` never
      inserted into this chain). Depends on T002 (needs `CasingSettings` to construct the
      resolved `FormatOptions.casing`).
- [X] T011 [US1] In `crates/drut-config/src/parse.rs`: add TOML parsing for
      `control_words_casing`/`pair_keywords_casing`/`data_references_casing` under `[format]`,
      same `"upper"`/`"lower"`/`"preserve"` accepted values and the same non-blocking
      malformed-value-warns-and-falls-back pattern every existing `[format]` field already
      uses. Depends on T010.
- [X] T012 [US1] In `crates/drut-cli/src/cli.rs`: add `--control-words-casing`,
      `--pair-keywords-casing`, `--data-references-casing`, each `Option<CasingArg>` with the
      same `ValueEnum` shape (`upper`/`lower`/`preserve`) `--casing` already has, and the same
      "no bare flag" usage-error rule (`002-cli-check-format` FR-015). In
      `crates/drut-cli/src/format_cmd.rs`: wire each into `ExplicitFormatOverride`. Depends on
      T010.
- [X] T013 [US1] In `crates/drut-mcp/src/format.rs`: add `control_words_casing`,
      `pair_keywords_casing`, `data_references_casing` string parameters to the `format` tool's
      input, same accepted-value shape and error-message pattern as the existing `casing`
      parameter. Depends on T010.

### Tests for User Story 1

- [X] T014 [P] [US1] Add tests to `crates/drut-config/tests/parse.rs`: each new field parses
      `"upper"`/`"lower"`/`"preserve"` cleanly; a malformed value warns and falls back, same
      shape as the existing `casing` field's own coverage.
- [X] T015 [P] [US1] Add tests to `crates/drut-config/tests/resolve.rs`: (a) a `drut.toml` with
      only the legacy `casing = "upper"` field resolves `control_words`/`pair_keywords` to
      `Upper`, `data_references` still `Preserve` — the explicit regression case proving legacy
      behavior is unchanged; (b) a `drut.toml` setting both legacy `casing` and a granular
      `data_references_casing` field: the granular field governs `data_references`, legacy
      governs the other two, no cross-contamination; (c) a `drut.toml` setting all three
      granular fields to three different values: `resolve_format_options` returns exactly those
      three values, independently.
- [X] T016 [P] [US1] Add a test to `crates/drut-cli/tests/format_flags.rs`: with a `drut.toml`
      resolving all three categories to `preserve`, `--data-references-casing=upper` overrides
      only that category for one run — `control_words`/`pair_keywords` in the same output stay
      untouched. Confirm the existing `--casing=upper` regression case (unchanged flag,
      unchanged meaning) still passes unmodified.
- [X] T017 [P] [US1] Add the equivalent test to `crates/drut-mcp/src/format.rs`'s own test
      module, mirroring T016's shape at the MCP surface.
- [X] T018 [US1] Add an integration test (new fixture or extend an existing one) exercising
      spec.md US1's own Acceptance Scenario 1 directly: a script mixing control words
      (`if`/`loop`), pair-keyword names (`file=`/`list=`), and data-reference tokens
      (`mi`/`mw`/`zi`) formatted with three different category values, confirming each
      category's tokens changed independently and no category's setting leaked into another's.
      Depends on T010-T013.
- [X] T039 [US1] Added post-`/speckit-analyze` (finding C1 — FR-003's "no built-in preset"
      guarantee had no regression test). Add a rejection test at each new surface: a
      `drut.toml` `control_words_casing = "auto"` (or `data_references_casing = "auto"`,
      either is sufficient to prove the point) is treated exactly like any other unrecognized
      string (non-blocking warn-and-fallback, `crates/drut-config/tests/parse.rs`); a CLI
      `--data-references-casing=auto` is a usage error, same as any other invalid `ValueEnum`
      value (`crates/drut-cli/tests/format_flags.rs`); an MCP `data_references_casing:
      "auto"` param produces the same invalid-value error as an unrecognized `casing` string
      (`crates/drut-mcp/src/format.rs`'s own test module). Depends on T011, T012, T013.

**Checkpoint**: User Story 1 independently proven — three categories, independently
configurable, at every surface, with every legacy surface unchanged, and `"auto"` explicitly
confirmed rejected everywhere, not just absent from the docs.

---

## Phase 4: User Story 2 - Data-reference tokens become reachable by casing at all (Priority: P1)

**Goal**: Confirm the literal reported gap (`mw`/`li`/`ni`/`i`/`j` unreachable by any casing
setting) is closed — this story is verification-focused; the production code it proves already
exists from Phase 2 (T004/T005) and Phase 3 (the config plumbing that lets `data_references` be
set at all).

**Independent Test**: Format a script containing `mw[1] = mi.1.1 + mi.2.1` and `li.ft`/
`ni.class` with `data_references` set to `upper`, and confirm every one of those tokens is
uppercased regardless of structural shape.

### Verification for User Story 2

- [X] T019 [US2] Add an integration-level test (`crates/voyager-core/tests/` — new fixture
      derived from real corpus shapes, not synthetic-only, matching this project's fixture-
      corpus-as-oracle convention) reproducing the literal GitHub issue #3 report: a script
      containing `mw`, `li`, `ni`, `i`, `j` in real-shaped contexts, formatted with
      `data_references: Upper` — confirm all five are uppercased (SC-002). Depends on Phase 2,
      Phase 3.
- [X] T020 [US2] Add a dedicated `format()`-level test (not just `data_reference.rs`'s own
      lower-level unit test from T008) proving FR-005 end-to-end: a script where `MW` appears
      both in a `PATHLOAD`-style pair-keyword-shaped position and as a plain assignment target,
      formatted once with `data_references: Upper` — confirm both occurrences are uppercased
      identically in the actual rendered output. Depends on T005.
- [X] T021 [US2] Add a test confirming `data_references: Preserve` (the default) leaves
      `mw`/`li`/`ni`/`i`/`j`/`zones` untouched in the same fixture T019 uses — the negative-case
      companion proving this is opt-in, not a silent behavior change (spec.md US2 Acceptance
      Scenario 3).
- [X] T022 [US2] Re-run and report `crates/voyager-core/src/keywords.rs`'s T009 tests explicitly
      as part of this story's own proof (SC-006) — `NUMREC`-family absence and `ZONES`
      presence in completion/spell-check are part of the same reported-gap story, even though
      the underlying dictionary edit landed in Phase 2.

**Checkpoint**: User Story 2 independently proven — the specific tokens named in the original
report are reachable, uniformly across structural shape, opt-in only.

---

## Phase 5: User Story 3 - A project sets its own indentation width (Priority: P2)

**Goal**: `indent_width` becomes a configurable `[format]` setting (default 4), independent of
both casing categories — could ship even if US1/US2 didn't exist.

**Independent Test**: With indentation width configured to 2, format a script with nested
`IF`/`LOOP` blocks and confirm each nesting level advances by exactly 2 spaces.

### Implementation for User Story 3

- [X] T023 [US3] In `crates/voyager-core/src/format.rs`: add `indent_width: u8` to
      `FormatOptions`. Because `4` isn't `u8::default()` (`0`), `FormatOptions` can no longer
      rely on a pure `#[derive(Default)]` — drop the derive, add a manual `impl Default for
      FormatOptions` setting `casing: CasingSettings::default(), top_level_indent:
      TopLevelIndentMode::default(), indent_width: 4` (data-model.md §1, research.md §4).
      Sequenced after Phase 2 (edits the same struct T002 touched) but logically independent of
      T002-T009's content. Depends on T002 (file-level sequencing only).
- [X] T024 [US3] In `crates/voyager-core/src/format.rs`'s indentation-planning logic (the
      `plan_indentation`/`plan_block`/`plan_children` family — `010-fmt-region-markers`'s own
      module-docs name these as the 4 real `plan.insert` call sites): replace the hardcoded
      4-space-per-nesting-level literal with `options.indent_width`. No other indentation
      behavior changes (top-level handling via `TopLevelIndentMode`, continuation-line handling,
      `; FMT: OFF` protection) — this touches only the per-level increment. Depends on T023.
- [X] T025 [US3] In `crates/drut-config/src/lib.rs`: add `indent_width: Option<u8>` to
      `FormatConfig`/`ExplicitFormatOverride`. In `resolve_format_options`: validate against
      1–16 (data-model.md §4) — a value outside that range (from either layer) is discarded
      with the same non-blocking-notice pattern every other malformed `[format]` value uses,
      falling through to the next precedence tier (`explicit > drut.toml > 4`). Depends on
      T023.
- [X] T026 [US3] In `crates/drut-cli/src/cli.rs`/`format_cmd.rs`: add `--indent-width=<N>`, same
      "requires an explicit value" rule as every other format flag. Depends on T025.
- [X] T027 [US3] In `crates/drut-mcp/src/format.rs`: add an `indent_width` integer parameter,
      same shape as `--indent-width`. Depends on T025.

### Tests for User Story 3

- [X] T028 [P] [US3] Add a test to `crates/voyager-core/src/format.rs`'s test module:
      `FormatOptions::default().indent_width == 4`; a 3-level-nested-block fixture formatted
      with `indent_width: 2` advances 2 spaces per level throughout; formatting with
      `FormatOptions::default()` (nothing configured) is byte-identical to this feature's
      pre-existing behavior across the existing golden fixture set (FR-012, US3 Acceptance
      Scenario 3).
- [X] T029 [P] [US3] Add tests to `crates/drut-config/tests/parse.rs`/`resolve.rs`:
      `indent_width` parses from TOML; a value of `0` or `500` falls back to `4` with a
      non-blocking notice, never a hard failure (US3 Acceptance Scenario 2).
- [X] T030 [P] [US3] Add tests to `crates/drut-cli/tests/format_flags.rs` and
      `crates/drut-mcp/src/format.rs`'s own test module: `--indent-width=2`/`indent_width: 2`
      overrides a `drut.toml`-resolved width for one run; each surface's invalid-value handling
      degrades non-fatally (SC-005 — verified at CLI and MCP; LSP's own diagnostic-surfacing
      path reuses `drut-config`'s resolution and needs no separate test, same reasoning `014`'s
      US3 used for `drut-lsp`).

- [X] T040 [US3] Added post-`/speckit-analyze` (finding I1, indent-width half). In
      `crates/voyager-core/src/format.rs`'s test module: an idempotence test —
      `format(format(x, opts).text, opts).text == format(x, opts).text` — for `opts` with
      `indent_width: 2` (or any non-default value), run against a multi-level nested-block
      fixture; and a `; FMT: OFF`/`; FMT: ON` interaction test — a fixture with a protected
      region containing nested blocks, formatted with `indent_width: 2`, confirming the
      protected region's indentation is left exactly as written while an unprotected nested
      region elsewhere in the same file reflects the new width. Depends on T024.

**Checkpoint**: User Story 3 independently proven — configurable indentation width, safe
fallback on invalid input, zero change when unconfigured, confirmed idempotent and
`; FMT: OFF`-respecting, not assumed.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Spec-document amendments and whole-workspace/full-corpus re-proof, once all three
stories are done.

- [X] T031 Amend `specs/002-cli-check-format/spec.md`'s **FR-015** with a new dated entry
      (`contracts/casing-categories-indent-width.md`'s exact shape) describing the categorical
      split — `--casing`/`casing = "..."`/the MCP `casing` param now set two of three
      independently-configurable categories, not "casing" as one undifferentiated concept.
      Original FR-015 text preserved, not replaced (same discipline `009`/`014` established).
- [X] T032 Amend `specs/001-voyager-script-parser/contracts/public-api.md`'s
      `formatting-api.md` "casing is the only configurable axis" exclusion statement — no
      longer true once `indent_width` ships (`ROADMAP.md` item 9). Same dated-amendment
      discipline as T031, not a silent rewrite.
- [X] T033 [P] In `ROADMAP.md`: mark pre-publish items 9 and 10 done, dated, pointing at this
      feature's spec directory — same pattern every other completed `ROADMAP.md` item already
      follows. Items 11/12 (Bill's split, `=`-spacing) are left exactly as-is — unaffected,
      still deferred.
- [X] T034 `cargo test --release --workspace` and `cargo clippy --workspace --all-targets --
      -D warnings`, both clean.
- [X] T035 [P] Full 161-file real-corpus revalidation across CLI/LSP/MCP with **no new
      configuration supplied** — expected zero diagnostic/output change from before this
      feature (SC-003), reported as its own explicit result, not inferred from the unit-test
      suite alone.
- [X] T036 Format a handful of real corpus files *with* `data_references_casing=upper` and
      `indent_width=2` configured (quickstart.md step 6), hand-verify the diffs are exactly the
      expected scope (only data-reference tokens changed case; only nesting-level spacing
      changed), then promote those diffs to new golden fixtures under
      `crates/voyager-core/tests/fixtures/golden/`. Depends on T035.
- [X] T037 Run `quickstart.md` end-to-end as written, confirming every step's expected result
      holds against the actual shipped code, not just against the individual task-level tests
      above in isolation.

**Checkpoint**: Feature-complete against spec.md; both amended spec documents and `ROADMAP.md`
consistent with shipped code; full workspace and full corpus re-proven clean; new golden
fixtures added for the two newly-configurable axes.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational only.
- **User Story 2 (Phase 4)**: Depends on Foundational **and** Phase 3 (its own independent test
  needs `data_references` to be settable at all, which is Phase 3's own deliverable) — the one
  place this feature's stories aren't fully parallel-independent, called out explicitly per
  plan.md's Constitution Check note rather than left implicit.
- **User Story 3 (Phase 5)**: Depends on Foundational only (T023 is sequenced after T002 for
  file-edit reasons, not a logical dependency) — genuinely parallel with Phase 3/4.
- **Polish (Phase 6)**: T031-T033 are independent of the code phases; T034-T037 depend on all
  three stories being complete.

### Parallel Opportunities

- T006 (keywords.rs) can run in parallel with T002-T005 (format.rs/data_reference.rs) — no
  shared file.
- T008, T009 can run in parallel once their respective dependencies (T004, T006) land.
- T014-T017 can run in parallel once T010-T013 land (different test files).
- Phase 3 (US1) and Phase 5 (US3) can proceed in parallel once Foundational completes — Phase 4
  (US2) waits on Phase 3 specifically, per the note above.
- T028, T029, T030 can run in parallel (different files).
- T038 depends only on T005 (not on Phase 3/US1) — can run alongside Phase 3 once Foundational
  completes. T039 depends on T011-T013 (Phase 3's own tasks). T040 depends only on T024 (not on
  T028-T030) — can run alongside the rest of Phase 5's tests.

---

## Parallel Example: Once Foundational (T002-T009) Lands

```bash
Task: "T010: drut-config gains three granular casing fields + precedence"
Task: "T023: FormatOptions gains indent_width + manual Default impl"
```

---

## Implementation Strategy

### MVP First (User Story 1 + User Story 2 together — they share the same P1 priority and US2 depends on US1's plumbing)

1. Setup → baseline confirmed clean.
2. Foundational → `CasingSettings` and `data_reference.rs` exist, compile, are tested.
3. User Story 1 → the config-surface plumbing that makes all three categories settable.
4. User Story 2 → verification that the originally-reported gap is actually closed.
5. **STOP and VALIDATE**: run T019-T022 against a real script matching issue #3's own report.

### Incremental Delivery

1. Foundational → foundation ready.
2. US1 + US2 → MVP (the reported bug is fixed, end to end).
3. US3 → indentation width, independently, whenever convenient.
4. Polish → spec amendments, `ROADMAP.md` update, full re-proof.

---

## Notes

- T003's split of `collect_casing_edits` and friends is the one piece of this feature most
  likely to have a subtle mistake (a call site accidentally left on the wrong category) —
  T007's exact `upper()`/`normalize()` translation rule exists specifically to catch this via
  the existing test suite, not to be treated as mechanical busywork.
- T038/T039/T040 were added after an initial `/speckit-analyze` pass found two real gaps: I1
  (plan.md promised axis-specific idempotence/`; FMT: OFF` re-verification with no
  corresponding task) and C1 (FR-003's "no preset" guarantee had no regression test). Numbered
  out of strict phase-block sequence (appended rather than triggering a full renumbering of
  every cross-reference in this file) but placed physically within the phase each belongs to,
  with dependencies pointing at the specific task each actually needs.
- Commit after each task or logical group.
