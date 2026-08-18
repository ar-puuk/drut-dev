---

description: "Task list for Range-Dash Spacing Exemption"
---

# Tasks: Range-Dash Spacing Exemption

**Input**: Design documents from `/specs/023-range-dash-spacing/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md
(all present)

**Tests**: Included — matches this project's established discipline for every prior
`operator_spacing.rs` change (`018`): real unit tests for the new recognition condition, plus a
real-corpus-shaped integration/idempotence proof, before merge (constitution Principle III/V).

**Organization**: One user story (P1) — there is no independently smaller or larger slice
(spec.md's own "Why this priority"). A Foundational phase carries the entire recognition change
(this is a single, atomic condition added to one existing function — there is no way to build
"half" of it); User Story 1's own phase then proves it against spec.md's Acceptance Scenarios end
to end and adds real-corpus fixture coverage, mirroring `016`'s Foundational/US1 split for a
feature of comparable size.

**Everything in this file's scope was measured against the real, current codebase on this
branch — not estimated** (research.md §1–§5):

- `push_gap_edit` already normalizes an arbitrary existing gap to a target width and already
  no-ops when the gap is already correct — the range-dash rule needs no new gap-handling code,
  only a new `want_spaces` decision for one specific occurrence shape (research.md §1).
- `pair_keyword_boundaries` already exists and is already reused by `collect_comma_edits` for
  the identical "which tokens belong to this pair's value" question — no new boundary-detection
  logic (research.md §2).
- `.` is not a lexer delimiter, so a decimal number (`1.5`) or dotted reference (`mi.1.1`)
  already tokenizes as one `Word` token containing a non-digit character — the bare-integer
  check excludes both by construction, confirmed by reading `lexer.rs::is_delimiter` directly,
  not assumed (research.md §3).
- A unary `-` never becomes an `OperatorOccurrence` at all under the existing `018` logic, so it
  never reaches the new range-dash condition — no additional unary-guard needed (research.md §5).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete sibling task)
- **[Story]**: US1 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/src/operator_spacing.rs` — the only source file this feature modifies
  (own `#[cfg(test)]` module included).
- `crates/voyager-core/tests/format_corpus.rs` — new golden fixture variant, if a real corpus
  file exercises this shape (Polish).
- `ROADMAP.md`, `CHANGELOG.md` — Polish-phase updates.
- No changes anywhere else: `drut-config`, `drut-cli`, `drut-lsp`, `drut-mcp`,
  `editors/vscode/`, `docs-site/` (FR-008).

---

## Phase 1: Setup

- [X] T001 Confirm baseline: `cargo build --workspace` and `cargo clippy --workspace
      --all-targets -- -D warnings` both clean, on this branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The entire range-dash recognition change — one condition added to
`operator_spacing.rs`'s existing binary-`-` handling. Not meaningfully testable in pieces.

- [X] T002 In `crates/voyager-core/src/operator_spacing.rs`: add a private helper
      `fn is_bare_integer_literal(tok: &Token) -> bool` — `true` iff `tok.kind ==
      TokenKind::Word` and `tok.text` is non-empty and every character is an ASCII digit
      (data-model.md §1, research.md §3). Pure, no I/O, never panics.
- [X] T003 In `crates/voyager-core/src/operator_spacing.rs`: add a private helper
      `fn is_range_dash(stmt: &Statement, index: usize) -> bool` (data-model.md §1; signature
      trimmed to just `stmt`+`index` — `boundaries` and `tokens` are both fully derivable from
      `stmt` internally, no need to pass them separately): returns `true` iff (a) `stmt.kind` is
      `StatementKind::Control`, (b) `index` falls inside some pair's value range as derived from
      `pair_keyword_boundaries(&stmt.tokens)` (reuse `[eq_idx + 1, next
      kw_start)`/`[eq_idx + 1, tokens.len())` exactly as `collect_comma_edits` already computes
      it, research.md §2), and (c) `stmt.tokens[index - 1]` and `stmt.tokens[index + 1]` both
      satisfy `is_bare_integer_literal` (T002). Depends on T002.
- [X] T004 In `crates/voyager-core/src/operator_spacing.rs`: change `recognize_operators`'s
      signature to accept the enclosing `stmt: &Statement` (in addition to `tokens: &[Token]`,
      which becomes `&stmt.tokens` at call sites) so `is_range_dash` (T003) has what it needs;
      compute `pair_keyword_boundaries(&stmt.tokens)` once per call when `stmt.kind` is
      `StatementKind::Control` (skip the call entirely otherwise — no wasted work for the common
      non-`Control` case). For the existing `"+" | "-" => is_binary_arithmetic(...).then_some(...)`
      arm specifically: when the token is `-`, is binary, and `is_range_dash` returns `true`,
      record that this occurrence wants `0` surrounding spaces instead of the `1` every other
      occurrence wants (data-model.md §1's `DashSpacing` — implement as a field on
      `OperatorOccurrence` or an inline branch in the caller, whichever keeps `push_gap_edit`'s
      existing call shape simplest; `OperatorKind` itself does not need a new variant, since the
      only difference is the numeric width passed to `push_gap_edit`, not how the occurrence is
      otherwise treated). Depends on T003.
- [X] T005 In `crates/voyager-core/src/operator_spacing.rs`'s `collect_operator_edits`: thread
      each occurrence's own `want_spaces` (T004) through to both `push_gap_edit` calls (leading
      and trailing side) instead of the hard-coded `1` literal — every non-range occurrence keeps
      getting `1`, unchanged. Update the one existing call site (inside `collect_fixed_edits`)
      that invokes `recognize_operators`/`collect_operator_edits` to pass `stmt` instead of
      `&stmt.tokens` where the new signature (T004) requires it. Depends on T004.
- [X] T006 [P] Add unit tests to `crates/voyager-core/src/operator_spacing.rs`'s own test module
      (contracts/range-dash-spacing.md's illustrative table, verified directly against real
      `fixed_edits_for`/`apply` helpers already in that module):
      Note: every `FILEO ...=` example's own `=` also gets `018`'s ordinary, unrelated one-space
      treatment (pre-existing `018` behavior) — only the range/comma handling below is new.
      - `FILEO SELECTLINK=1-50,75,90-100` → `FILEO SELECTLINK = 1-50,75,90-100` (both range dashes
        already tight; the three commas are same-pair-internal and were never in `018`'s comma
        rule's scope at all, the same as `LOOP i=1,5,1`'s own internal commas today — confirm no
        edit is queued for any of them specifically, not merely that the rendered output happens
        to look the same).
      - `FILEO NODES=200 - 300`, `FILEO NODES=200- 300`, `FILEO NODES=200 -300` → all three →
        `FILEO NODES = 200-300`.
      - `X = 100-1` (an `Assignment`) → `X = 100 - 1` (unchanged `018` spacing).
      - `IF (COUNT-1 == 0)` → `IF(COUNT - 1 == 0)` (unchanged `018` spacing; also exercises the
        control-word-paren rule already covered elsewhere, confirming no interaction).
      - `FILEO SELECTLINK=@START@-50` → `FILEO SELECTLINK = @START@ - 50` (non-integer operand,
        unchanged spacing).
      - `FILEO OFFSET=-100,50` → `FILEO OFFSET = -100,50` (unary `-`, never a range-dash
        candidate).
      - `FILEO THRESHOLD=1.5-2.5` → `FILEO THRESHOLD = 1.5 - 2.5` (normal binary-arithmetic
        spacing applied to the `-`; decimal token excluded from the bare-integer check by
        construction).
      - **`FILEO NODES=1-50 ,SELECTLINK=75 - 100` → `FILEO NODES = 1-50, SELECTLINK = 75-100`**
        (spec.md Acceptance Scenario 6 / FR-006): the pair-*boundary* comma (a real candidate for
        `018`'s comma rule, unlike the same-pair-internal commas above) gets its space removed
        before and inserted after, while both range dashes independently tighten — proves the two
        rules compose correctly on genuinely adjacent, disjoint gaps in one pass, not just that
        each works in isolation.
      Depends on T005.

**Checkpoint**: The range-dash exemption is real, compiling, and unit-tested in isolation —
`cargo build --workspace` succeeds (no adapter crate touches this signature change, since
`recognize_operators`/`collect_operator_edits` are both `pub(crate)`-or-narrower, called only
from within this same module).

---

## Phase 3: User Story 1 - A range-list value keeps its conventional tight notation (Priority: P1)

**Goal**: Confirm the Foundational-phase recognition change (T002-T006) actually delivers
spec.md's own Acceptance Scenarios end to end, including through the full `format`/`format_bytes`
pipeline (not just the isolated `fixed_edits_for` helper), and is idempotent.

**Independent Test**: With `operator_spacing` configured to `fixed`, format a script containing a
pair-keyword value with a range written tight, one with spaces on both sides, and one with a
space on only one side — confirm all three render tight, and that a comma-separated list mixing
single IDs and ranges renders every range within it tight independently (spec.md's own
Independent Test, verbatim).

### Tests for User Story 1

- [X] T007 [US1] Add an integration test to `crates/voyager-core/src/format.rs`'s test module
      (or a new test in `crates/voyager-core/tests/operator_spacing_integration.rs` if that file
      already exists and is the established home for this shape — check first) exercising
      spec.md's Acceptance Scenarios 1–6 directly through `format(source, options)`, not just
      `fixed_edits_for` (every `FILEO ...=` example's own `=` also gets `018`'s ordinary
      one-space treatment, pre-existing behavior): `FILEO SELECTLINK=1-50,75,90-100` →
      `FILEO SELECTLINK = 1-50,75,90-100`; `FILEO NODES=200 - 300` → `FILEO NODES = 200-300`;
      `X = 100-1` unchanged under `fixed`; `IF (COUNT-1 == 0)` unchanged under `fixed`; the same
      four scripts under `operator_spacing: Preserve` all byte-identical to their input
      (Acceptance Scenario 5); and `FILEO NODES=1-50 ,SELECTLINK=75 - 100` →
      `FILEO NODES = 1-50, SELECTLINK = 75-100` under `fixed` (Acceptance Scenario 6 — the
      pair-boundary comma and both range dashes normalize together in one real `format()` call,
      not just in the lower-level edit-collection helper). Depends on Phase 2.
- [X] T008 [US1] In `crates/voyager-core/src/operator_spacing.rs`'s (or wherever T007 lands)
      test module: an idempotence test — formatting the already-`fixed`-formatted
      `FILEO SELECTLINK = 1-50,75,90-100` a second time produces zero edits and byte-identical
      output (SC-004); and a
      `; FMT: OFF`/`; FMT: ON` regression test — a range-list value sitting inside a protected
      region is left exactly as written even when spaced with extra whitespace, matching every
      other `018` rule's existing protected-region guarantee. Depends on Phase 2.

**Checkpoint**: User Story 1 independently proven against every one of spec.md's Acceptance
Scenarios through the real `format`/`format_bytes` entry point, confirmed idempotent and
`; FMT: OFF`-respecting — not assumed from the Foundational phase's isolated unit tests alone.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: `ROADMAP.md`/`CHANGELOG.md` updates and whole-workspace/full-corpus re-proof.

- [X] T009 [P] In `ROADMAP.md`: add a new dated entry under "Resolved queued items" (or the
      pre-publish sequence, whichever this project's own current convention points to — check
      the most recent entries first) recording this feature, pointing at
      `specs/023-range-dash-spacing/`. Never rewrite an existing historical entry.
- [X] T010 [P] In `CHANGELOG.md`'s `## [Unreleased]` section: add a `### Fixed` (not `###
      Changed` — this corrects an existing `018-operator-spacing` behavior under an
      already-shipped setting, not a new/renamed configuration surface) bullet describing the
      range-dash exemption, in this project's own established changelog voice (see the existing
      entries for tone/format).
- [X] T011 `cargo test --release --workspace` and `cargo clippy --workspace --all-targets --
      -D warnings`, both clean.
- [X] T012 [P] Full real-corpus revalidation across CLI/LSP/MCP with `operator_spacing`
      **unconfigured** — expected zero diagnostic/output change from before this feature
      (SC-003), reported as its own explicit result. Then format the corpus *with*
      `operator_spacing=fixed` and separately `operator_spacing=auto`, and check specifically
      whether any real fixture contains a `SELECTLINK`/`NODES`-shaped (or similarly-named)
      pair-keyword range-list value — if one exists, hand-verify its diff is exactly the
      expected tightened range(s) with nothing else changed, then promote it to a new golden
      fixture (same discipline `018`/`019` established), with an idempotence check. If no real
      fixture happens to exercise this shape, record that explicitly rather than silently
      skipping golden-fixture coverage. Depends on T011.
- [X] T013 Run `quickstart.md` end-to-end as written, confirming every step's expected result
      holds against the actual shipped code.

**Checkpoint**: Feature-complete against spec.md; `ROADMAP.md`/`CHANGELOG.md` consistent with
shipped code; full workspace and full corpus re-proven clean.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS User Story 1.
- **User Story 1 (Phase 3)**: Depends on Foundational only.
- **Polish (Phase 4)**: T009/T010 are independent of the code phases; T011-T013 depend on
  User Story 1 being complete.

### Parallel Opportunities

- T006 can run once T005 lands (same file, but a distinct, additive test-module change — treat
  as sequential in practice since it's the same file as T002-T005, despite the `[P]` marker
  reflecting "no logical dependency on a *different* incomplete task").
- T009 and T010 can run in parallel with each other and with T011-T013 (different files).
- T012 and T013 can run in parallel once T011 lands.

---

## Implementation Strategy

### MVP First (the only story)

1. Setup → baseline confirmed clean.
2. Foundational → the range-dash exemption exists, compiles, is unit-tested in isolation.
3. User Story 1 → proven end to end against every Acceptance Scenario, idempotent,
   `; FMT: OFF`-respecting.
4. **STOP and VALIDATE**: run T012 against the real corpus.

### Incremental Delivery

This feature has no meaningful incremental slices smaller than "the whole thing" (spec.md's own
framing) — Foundational and User Story 1 together are the entire deliverable; Polish is
housekeeping around it.

---

## Notes

- T004's `recognize_operators` signature change (`tokens: &[Token]` → `stmt: &Statement`) is the
  one small ripple this feature causes beyond the new condition itself — confirm at
  implementation time that its only caller is `collect_operator_edits`, itself only called from
  `collect_fixed_edits`, which already has `stmt` in scope (per the current code read during
  planning) — so this is a same-module, non-breaking signature change, not a public API change.
- T006's decimal-number case (`FILEO THRESHOLD=1.5-2.5`) is the one assertion in this file that exists
  purely to prove a research.md claim (§3: decimals are excluded "by construction") rather than
  to cover a scenario spec.md explicitly demands — kept as its own explicit test rather than
  trusted as an obvious consequence, matching this project's own "confirmed by direct testing,
  not merely reasoned about" standard used throughout `018`'s own test suite.
- Commit after each task or logical group.
