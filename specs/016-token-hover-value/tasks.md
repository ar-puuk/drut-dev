---

description: "Task list for Token Hover Shows Assigned Value"
---

# Tasks: Token Hover Shows Assigned Value

**Input**: Design documents from `/specs/016-token-hover-value/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/,
quickstart.md (all present)

**Tests**: Included — matches this project's established discipline for every
prior parser/LSP feature (`001`, `003`, `010`–`013`): real unit tests for new
`voyager-core` pure logic, plus real integration-level tests (including actual
on-disk files, since this is the first `drut-lsp` feature that reads a file the
editor never opened) for the `drut-lsp` wiring.

**Organization**: Three user stories, matching spec.md exactly — US1 (P1,
same-file resolution — the core value, needs no cross-file machinery), US2 (P2,
one-level literal `READ FILE` cross-file resolution — the "control center" case),
US3 (P3, the no-fabricated-value guardrail — mostly proof, not new
implementation, since FR-008 falls out of `resolve_token_value` returning `None`
by construction). A Foundational phase carries the new, pure `voyager-core`
resolution logic both later phases build on.

**Everything in this file's scope was measured against the real, current codebase
on this branch and the real `WF-TDM-Development` corpus — not estimated**
(research.md §1-§8; spec.md Assumptions):

- `@name@` is already tokenized as a single `TokenKind::VariableRef { name }` — no
  new lexing needed, only a position-lookup over existing tokens (research.md §1).
- `READ FILE = '<path>'` already parses as `Control { word: "READ", pairs: [("FILE",
  value_tokens)] }` — confirmed by tracing `classify_statement` directly, not
  assumed (research.md §2).
- `voyager-core` stays I/O-free (constitution Principle I); the one on-disk read a
  `READ FILE` target needs happens entirely in `drut-lsp`, which already has
  `workspace::uri_to_path` (built for `012`) for the URI→path step (research.md §4,
  §7).
- No existing token-to-text renderer exists or is needed — a new
  `position::text_for_span` slices the real source substring directly, which is
  simpler and unambiguously correct compared to reconstructing whitespace between
  joined tokens (research.md §3).
- `hover.rs`'s existing `block_at` already returns `None` for a `@token@` position
  today (it's never a block opener/closer) — today's hover over `@token@` always
  falls through to the spell-check-nudge path, which usually finds nothing. The new
  branch is added *before* `block_at`, changing nothing about `block_at`'s or
  `spellcheck`'s behavior for any other position (research.md §6, FR-010).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete
  sibling task)
- **[Story]**: US1/US2/US3 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/src/token_resolution.rs` — new module: all pure resolution
  types/functions (contracts/token-resolution-api.md).
- `crates/voyager-core/src/lib.rs` — `pub mod token_resolution;` + re-exports.
- `crates/drut-lsp/src/position.rs` — new `text_for_span` helper.
- `crates/drut-lsp/src/hover.rs` — new token-value branch, tried before the
  existing `block_at` check; all `drut-lsp`-level tests for this feature live in
  this file's own `#[cfg(test)]` module, matching its existing convention.

---

## Phase 1: Setup

- [X] T001 Confirm baseline: `cargo build --workspace` and `cargo clippy
      --workspace --all-targets -- -D warnings` both clean on this branch before
      any new change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The pure `voyager-core` resolution logic and the one small `drut-lsp`
text-slicing helper every user story's own `hover.rs` wiring depends on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T002 Create `crates/voyager-core/src/token_resolution.rs` with the module
      doc comment and the public types from contracts/token-resolution-api.md:
      `VariableRefAt { name, span }`, `Assignment<'a> { target, value_span,
      statement_span }`, `ReadFileRef<'a> { literal_path, statement_span }`,
      `Source { SameFile, ReadFile { read_file_statement_span } }`,
      `ResolvedTokenValue { value_span, statement_span, source }`. Add `pub mod
      token_resolution;` to `crates/voyager-core/src/lib.rs` and re-export the
      public types alongside the existing `block_resolution` re-exports.
- [X] T003 [P] Implement `pub fn variable_ref_at(nodes: &[Node], pos: Position) ->
      Option<VariableRefAt>` in `token_resolution.rs` (data-model.md): walks every
      `Statement.tokens` at any nesting depth (reusing `block_resolution.rs`'s
      existing node/block traversal shape) for a `TokenKind::VariableRef` token
      whose `span` contains `pos`. Never panics for any input. Depends on T002.
- [X] T004 [P] Implement `pub fn all_assignments(nodes: &[Node]) ->
      Vec<Assignment<'_>>` in `token_resolution.rs` (data-model.md): flattened,
      source-order walk of every `StatementKind::Assignment` at any nesting depth.
      Empty `Vec` (never panic) for a document with none. Depends on T002.
- [X] T005 [P] Implement `pub fn read_file_refs(nodes: &[Node]) ->
      Vec<ReadFileRef>` in `token_resolution.rs` (research.md §2, §3,
      data-model.md): source-order walk for `Control` statements where
      `word.eq_ignore_ascii_case("READ")` and a pair's keyword
      case-insensitively equals `"FILE"`. `literal_value_span` is `Some` (the
      merged span of the value's own tokens, quotes included — never a
      reconstructed `String`, since the lexer splits a quoted value on internal
      whitespace, e.g. a space-bearing directory name, into multiple tokens)
      only when the value contains no `TokenKind::VariableRef` token; otherwise
      `None`. Depends on T002.
- [X] T006 Implement `pub fn resolve_token_value<'a>(nodes: &'a [Node], pos:
      Position, included: &'a [(Span, Vec<Node>)], name: &str) ->
      Option<ResolvedTokenValue>` in `token_resolution.rs` (data-model.md §
      `resolve_token_value`, contracts/token-resolution-api.md): combines
      `all_assignments(nodes)` (each keeping its own real position for ordering)
      with, per `included` entry, `all_assignments(&included_nodes)` (each
      instead compared using that entry's own `Span` for ordering — spec.md
      FR-004's interleaving rule); filters to `name`-matching
      (`eq_ignore_ascii_case`, research.md §8) entries at or before `pos`; returns
      the latest-ordered one, or `None` if none qualify. Depends on T004, T005.
- [X] T007 [P] Add `pub fn text_for_span(text: &str, span: Span) -> String` to
      `crates/drut-lsp/src/position.rs` (data-model.md, contracts/
      token-resolution-api.md): slices `text` for `span` using the same
      line/`char`-walking approach `to_lsp_position` already uses; clamps rather
      than panics for an out-of-range span, matching this module's existing
      guarantee.
- [X] T008 [P] Unit tests for `variable_ref_at` in `token_resolution.rs`'s own
      test module: cursor exactly over a `@name@` reference (including one
      nested inside an `IF`/`LOOP` block) returns the right name/span; a
      position just outside it (one character before the leading `@`, one after
      the trailing `@`) returns `None`. Depends on T003.
- [X] T009 [P] Unit tests for `all_assignments`: a target inside a nested
      `IF`/`LOOP` block is found, in correct source order relative to top-level
      assignments; a document with no assignments returns an empty `Vec`, not a
      panic. Depends on T004.
- [X] T010 [P] Unit tests for `read_file_refs`, using the exact two real shapes
      found in `WF-TDM-Development` (spec.md Assumptions): `READ FILE =
      '_ControlCenter.block'` classifies as `literal_value_span: Some(span)`,
      where slicing the original source at that span and stripping quotes
      yields exactly `_ControlCenter.block`; `READ FILE =
      '@ParentDir@sub\path.block'` classifies as `literal_value_span: None`; a
      quoted value containing internal whitespace (e.g. `READ FILE = 'Network
      Processing Tools\x.block'`) round-trips through
      slice-then-strip-quotes with the space preserved intact, proving the
      span-based design doesn't drop it; a document with no `READ FILE`
      statement at all returns an empty `Vec`. Depends on T005.
- [X] T011 Unit tests for `resolve_token_value` covering every FR-004/FR-005
      ordering rule directly (not only indirectly via `hover.rs` integration
      tests later): a same-file reassignment after an `included`-file's own
      assignment wins (US2 Acceptance Scenario 2's interleaving rule); an
      assignment positioned strictly after `pos` (same-file, or via an
      `included` entry whose own `Span` is after `pos`) is never selected (US1
      Acceptance Scenario 3); case-insensitive name matching (`ParentDir` vs
      `PARENTDIR`, FR-005); an empty `included` slice degrades correctly to
      same-file-only resolution; no matching assignment anywhere returns `None`
      (FR-008's foundation). Depends on T006.
- [X] T012 [P] Unit tests for `text_for_span` in `position.rs`'s own test
      module: a normal in-range span round-trips the exact expected substring;
      an out-of-range span clamps rather than panics (mirroring
      `to_lsp_position`'s own existing out-of-range tests). Depends on T007.

**Checkpoint**: All pure resolution logic exists, is I/O-free, and is proven
correct in isolation before any `hover.rs` wiring begins.

---

## Phase 3: User Story 1 - See a same-file token's value without scrolling (Priority: P1) 🎯 MVP

**Goal**: Hovering an `@token@` reference assigned earlier in the same open
document shows its value and where it was assigned.

**Independent Test**: Open a `.s`/`.block` file with a `TOKEN = value` assignment
followed later by `@TOKEN@`; hover over the reference and confirm the hover shows
`value` and the assigning line.

### Implementation for User Story 1

- [X] T013 [US1] In `crates/drut-lsp/src/hover.rs`'s `handle`: add a new first
      branch, tried before the existing `block_at` call (research.md §6, FR-010)
      — call `token_resolution::variable_ref_at(&doc.parse_result.nodes, pos)`;
      on `Some(var_ref)`, call `token_resolution::resolve_token_value(&doc.
      parse_result.nodes, pos, &[], &var_ref.name)` (empty `included` for this
      story — US2 populates it); on `Some(resolved)`, render `lsp_types::Hover`
      markdown containing the value (via `position::text_for_span(&doc.text,
      resolved.value_span)`) and the assigning line number (via
      `position::to_lsp_range(&doc.text, resolved.statement_span)`, matching
      `block_at`'s own existing "matched counterpart at line N" phrasing style);
      on `None` (from either `variable_ref_at` or `resolve_token_value`), fall
      through unchanged into the existing `block_at` → `spellcheck::hint_for`
      chain. Depends on T003, T006, T007.

### Tests for User Story 1

- [X] T014 [US1] Add a test to `hover.rs`'s own test module: a document with
      `ZoneMsgRate = 50` followed by `@ZoneMsgRate@` later — hovering the
      reference shows `50` and the correct assigning line number (spec.md US1
      Acceptance Scenario 1). Depends on T013.
- [X] T015 [US1] [P] Add a test to `hover.rs`: a token reassigned more than once
      before the hovered reference — the hover shows the value from the
      assignment closest to (not the first one before) the reference (US1
      Acceptance Scenario 2). Depends on T013.
- [X] T016 [US1] [P] Add a test to `hover.rs`: a token assigned only *after* the
      hovered reference — the hover does not show that later value (falls
      through to existing behavior for that reference) (US1 Acceptance Scenario
      3). Depends on T013.
- [X] T017 [US1] [P] Add a regression test to `hover.rs` confirming every
      existing test in this file (block info, short-IF, implicitly-closed-RUN,
      spell-check nudge, unrelated token) still passes unchanged — proves the
      new branch changes nothing for a position that isn't over a `@token@`
      reference (FR-010). Depends on T013.

**Checkpoint**: The core, same-file value-on-hover experience works and is proven
— this alone is a complete, shippable increment.

---

## Phase 4: User Story 2 - See a value set in a directly-read "control center" file (Priority: P2)

**Goal**: Hovering an `@token@` reference whose value comes from a file the open
document directly reads via a literal `READ FILE` statement shows that value,
without the user opening the other file themselves.

**Independent Test**: An open file contains `READ FILE = 'sibling.block'`;
`sibling.block` (on disk, not open in the editor) assigns `TOKEN = value` and the
open file never reassigns it; hovering `@TOKEN@` later in the open file shows
`value` and names `sibling.block`.

### Implementation for User Story 2

- [X] T018 [US2] In `hover.rs`'s new branch (T013): before calling
      `resolve_token_value`, build `included: Vec<(Span, Vec<Node>)>` — call
      `token_resolution::read_file_refs(&doc.parse_result.nodes)`, filter to
      `literal_value_span.is_some()` entries, and for each: slice
      `position::text_for_span(&doc.text, span)`, strip one leading and one
      trailing quote character if both present and matching, then resolve the
      result as a path relative to `workspace::uri_to_path(uri)?.parent()`
      (research.md §7); on any failure at any step (`uri_to_path` returns
      `None` — e.g. an unsaved/untitled buffer; `std::fs::read` fails; the
      bytes don't parse meaningfully) skip that entry silently (spec.md FR-007)
      rather than erroring; otherwise call `voyager_core::parse_bytes` (not
      `parse` — the file's bytes carry no UTF-8 guarantee, research.md §4) and
      push `(ref.statement_span, parsed.nodes)`. Pass the resulting `included`
      to `resolve_token_value` instead of the empty slice from T013. Depends
      on T005, T007, T013.

### Tests for User Story 2

- [X] T019 [US2] Add a real-filesystem test to `hover.rs`'s own test module
      (write real files to a temp directory via `std::fs::write`, no new
      dependency — clean up after): an open document (simulated via `did_open`,
      pointing its URI at the temp file's real path) containing `READ FILE =
      'sibling.block'`, with `sibling.block` written to the same temp directory
      (never opened via `did_open`) assigning `UsedZones = 3629`; hovering
      `@UsedZones@` in the open document resolves to `3629` and the hover text
      names `sibling.block` (spec.md US2 Acceptance Scenario 1, FR-009). Depends
      on T018.
- [X] T020 [US2] [P] Add a test to `hover.rs`: the same setup, but the open
      document *also* reassigns `UsedZones` on a line after the `READ FILE` line
      and before the hovered reference — the hover shows the open document's own
      later value, not `sibling.block`'s (US2 Acceptance Scenario 2, FR-004's
      interleaving rule proven at the full integration level, not only the
      T011 unit-test level). Depends on T018.
- [X] T021 [US2] [P] Add a test to `hover.rs`: `READ FILE =
      '@ParentDir@sub\path.block'` (a dynamic, token-built path) — a token only
      ever assigned inside that unresolvable target does not resolve via this
      path; other, same-file tokens in the same document are unaffected (US2
      Acceptance Scenario 3). Depends on T018.
- [X] T022 [US2] [P] Add a test to `hover.rs`: `READ FILE = 'missing.block'`
      pointing at a file that does not exist on disk — hovering a token that
      would only be found there falls back to existing behavior; no panic, no
      error surfaced (US2 Acceptance Scenario 4, FR-007). Depends on T018.
- [X] T023 [US2] [P] Add a test to `hover.rs`: a document opened with a
      non-`file`-scheme URI (e.g. `untitled:Untitled-1`) that contains a `READ
      FILE` statement — same-file resolution (US1) still works normally; the
      cross-file part silently contributes nothing (`uri_to_path` returns
      `None`), no panic (research.md §7). Depends on T018.

**Checkpoint**: The "control center" case works end-to-end against real on-disk
files, with every documented failure mode (missing file, dynamic path, unsaved
buffer) proven to degrade gracefully.

---

## Phase 5: User Story 3 - No answer is not a false answer (Priority: P3)

**Goal**: A `@token@` reference this feature cannot resolve behaves exactly as it
did before this feature existed — no fabricated or near-match value, ever.

**Independent Test**: Hover a `@token@` reference with no discoverable assignment
in scope; confirm no value is shown and no other part of the hover response is
affected.

**Note**: This story requires no new implementation — FR-008's fallback already
falls out of T013's own control flow (`resolve_token_value` returning `None` →
fall through unchanged) by construction. This phase is dedicated proof only.

### Tests for User Story 3

- [X] T024 [US3] Add a test to `hover.rs`: a `@token@` reference with no matching
      assignment anywhere in scope (same file or a directly, literally read
      file) — hovering it shows no fabricated value, and hovering an unrelated
      position on the same line (e.g. a block keyword) is unaffected (spec.md
      US3 Acceptance Scenario 1). Depends on T013.
- [X] T025 [US3] [P] Add a test to `hover.rs`: a token name one edit away from a
      real, resolvable token (e.g. `@ZoneMsgRat@` vs. an assigned
      `ZoneMsgRate`) — hovering the misspelled reference does not show the
      near-match's value (US3 Acceptance Scenario 2 — a wrong value with
      apparent confidence is explicitly worse than none). Depends on T013.

**Checkpoint**: Every unresolvable case is proven to degrade safely — no
fabricated value under any tested condition.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Whole-workspace re-proof and quickstart execution, once all three
stories are done.

- [X] T026 `cargo test --release --workspace` and `cargo clippy --workspace
      --all-targets -- -D warnings`, both clean.
- [X] T027 Run quickstart.md end-to-end (all steps, including the manual VS Code
      step); confirm each step's expected outcome individually before reporting
      the feature done. Manual VS Code verification is left for the repo owner,
      matching this project's established pattern for every prior LSP-facing
      feature.

**Checkpoint**: Feature-complete against spec.md; every FR and every acceptance
scenario across all three user stories independently proven.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup. Blocks all three user stories —
  every pure resolution function (T002-T007) must exist and be individually
  proven (T008-T012) before any `hover.rs` wiring begins.
- **User Story 1 (Phase 3)**: Depends on Foundational in full.
- **User Story 2 (Phase 4)**: Depends on User Story 1's own T013 (extends the
  same branch, adding the `included` argument T013 leaves empty) plus
  Foundational's T005.
- **User Story 3 (Phase 5)**: Depends on User Story 1's T013 only (no new
  implementation; T024/T025 could in principle be proven before Phase 4 exists,
  but are sequenced after it here since they logically close out the full
  resolve-or-fall-back story).
- **Polish (Phase 6)**: Depends on all three stories being complete.

### Parallel Opportunities

- T003, T004, T005 can proceed in parallel once T002 lands — same file
  (`token_resolution.rs`), coordinate insertion order rather than true
  concurrent editing.
- T008, T009, T010 can proceed in parallel once their respective dependencies
  (T003, T004, T005) land.
- T012 can proceed in parallel with any of T008-T011 once T007 lands — different
  file (`position.rs`).
- T015, T016, T017 can proceed in parallel once T013 lands — same file
  (`hover.rs`), coordinate insertion order.
- T020, T021, T022, T023 can proceed in parallel once T018 lands — same caveat.
- T025 can proceed in parallel with T024 once T013 lands.

---

## Implementation Strategy

### Single Pass (small-to-medium feature, one shared pure-logic layer, three thin story layers)

1. Setup → baseline confirmed clean.
2. Foundational → the entire real analysis logic of this feature, all in
   `voyager-core` and fully I/O-free: finding a hovered `@token@`, finding
   candidate assignments (same-file shape), finding literal `READ FILE`
   statements, and the ordering/selection rule that ties them together — each
   proven in isolation before any adapter wiring exists.
3. User Story 1 → the one new `hover.rs` branch, same-file only (`included: &[]`)
   — already the feature's core value, independently shippable on its own.
4. User Story 2 → extends that same branch with the one genuinely new
   capability (`drut-lsp` reading a file off disk that isn't open in the
   editor), proven against real files with every documented failure mode
   (missing file, dynamic path, unsaved buffer).
5. User Story 3 → no new implementation; dedicated proof that the "found
   nothing" path degrades exactly to pre-existing behavior, never a guess.
6. Polish → whole-workspace re-proof and quickstart execution (manual VS Code
   step left for the owner, matching every prior LSP-facing feature's pattern).

---

## Notes

- T006 (`resolve_token_value`'s ordering rule) and T018 (the actual disk-read
  wiring) are this feature's two most architecturally significant tasks — one
  encodes Voyager's real interleaved execution semantics as a pure comparison,
  the other is the first time `drut-lsp` ever reads a file the editor didn't
  open. Neither should be treated as routine plumbing.
- T017 overlaps with T026's full-workspace `cargo test` by design — T017 is
  expected to require **zero new test logic**, only confirming the existing
  `hover.rs` tests are byte-for-byte undisturbed by T013's new branch (FR-010's
  own explicit traceability). Do not write throwaway duplicate tests for it.
- Commit after each task or logical group.
