---

description: "Task list for Code Folding Support"
---

# Tasks: Code Folding Support

**Input**: Design documents from `/specs/011-code-folding/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/,
quickstart.md (all present)

**Tests**: Included — matches this project's established discipline for every
prior `drut-lsp`/`voyager-core` feature (`004`, `005`, `006`, `010`): real unit
tests for the new `voyager-core` function, and a real protocol test over
`Connection::memory()` for the LSP handler, not just manual spot-checks.

**Organization**: Three user stories, matching spec.md exactly — US1 (P1,
collapse a block/comment — the entire mechanism, MVP), US2 (P2, fold ranges
stay correct as the document is edited), US3 (P3, native Fold All/Unfold All
works for free). US1 is the foundation everything else builds on; US2 and US3
both depend on it but require little to no new implementation code of their
own — mostly dedicated proof that properties already true by construction
actually hold for folding specifically.

**Everything in this file's scope was measured against the real, current
codebase during planning (research.md §1-§6), not estimated**:

- `block_resolution.rs`'s only public entry point today is a single-position
  query (`block_at`); its three derivation helpers (`counterpart_for`,
  `is_short_if`, `block_kind_name`) are private. Folding needs a full-document
  enumeration, so **one new function is added to `voyager-core`**
  (`all_blocks`) — reusing those three helpers completely unchanged, not
  redesigning them (research.md §1, direct answer to the owner's
  pre-`/speckit-tasks` question).
- Block-comment folding needs **zero** `voyager-core` change — `tokenize` and
  `TokenKind::BlockComment` are already public with everything needed
  (research.md §2).
- `lsp-types` 0.97.0 (already a dependency) already ships `FoldingRange`,
  `FoldingRangeKind::{Region,Comment}`, `FoldingRangeProviderCapability`, and
  `request::FoldingRangeRequest` — verified directly against the vendored
  crate source, no dependency bump (research.md §3).
- FR-010 ("ranges reflect live-edited content, never a stale parse") is
  **already guaranteed structurally** by `document_store.rs`'s own invariant
  (`OpenDocument::replace` always recomputes `parse_result` on every
  `didChange`) — US2's tasks below are dedicated proof of this holding for
  folding specifically, not new plumbing.
- FR-004 (no short-IF fold) needs **no special case** in `drut-lsp` — a
  short-IF's `counterpart` is already `None` from the unchanged derivation
  rules, so it's filtered out at the exact same point as any unmatched block
  (research.md §5).

**Post-`/speckit-analyze` remediation (2026-08-12)**: two findings closed
before implementation. **I1 (HIGH)**: FR-008's zero-span guard was described
and implemented in every doc (research.md §5, data-model.md, contracts.md,
this file's original T004) as applying only to the block stream — but a
single-line block comment (`/* note */`) is a real, common case where the
guard is load-bearing, not defensive, and nothing else excludes it. Fixed in
spec.md (FR-006/FR-008/SC-002/Edge Cases), research.md §5, data-model.md's
diagram, contracts.md, and T004/T006 below. **E1 (MEDIUM)**: the Polish-phase
corpus proof (T012) originally cross-checked blocks only, leaving SC-003's
"or comment" half unproven at corpus scale; T012 below now has two full-corpus
assertions, not one.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an
  incomplete sibling task)
- **[Story]**: US1/US2/US3 — omitted for Setup/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/src/block_resolution.rs` — the new `BlockFold` struct
  and `all_blocks` function (research.md §1); zero changes to
  `counterpart_for`/`is_short_if`/`block_kind_name`/`BlockInfo`.
- `crates/voyager-core/tests/block_resolution.rs` — new unit tests for
  `all_blocks` (this module's established integration-test-file convention,
  same as `block_at`'s own tests).
- `crates/drut-lsp/src/folding.rs` — new module, the `textDocument/
  foldingRange` handler and its own test module.
- `crates/drut-lsp/src/lib.rs` — `pub mod folding;`, capability registration,
  request dispatch.
- `crates/drut-lsp/tests/protocol_smoke.rs` — real protocol-level tests over
  `Connection::memory()`.
- `crates/drut-lsp/tests/folding_corpus.rs` — new file, full-corpus
  cross-check (Polish), same `DRUT_CORPUS_PATH`/`#[ignore]` gating
  `diagnostics_corpus.rs` already establishes.

---

## Phase 1: Setup

- [x] T001 Confirm baseline: `cargo build --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings` both clean, on
      this fresh branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: User Story 1 - Collapse a block to see program structure at a glance (Priority: P1) 🎯 MVP

**Goal**: Every block (all 7 kinds, explicit or implicit close) and every
well-formed block comment offers a working fold control that hides exactly
the content strictly between its opener and its resolved counterpart.

**Independent Test**: Open a script with one explicitly-closed block of each
of the 7 kinds and a block comment; request `textDocument/foldingRange`;
confirm each produces the correct range and nothing else does.

### Implementation for User Story 1

- [x] T002 [US1] Add `BlockFold` and `pub fn all_blocks(nodes: &[Node],
      diagnostics: &[Diagnostic]) -> Vec<BlockFold>` to
      `crates/voyager-core/src/block_resolution.rs` (research.md §1,
      contracts/folding-range-api.md): `BlockFold { pub opener: Position, pub
      info: BlockInfo }`. A single-pass recursive traversal mirroring
      `find_block_at`'s own recursion (into `block.children`, and for `If`,
      each branch's `children`) — but *collecting* one `BlockFold` per
      `Node::Block` encountered instead of returning on first position match.
      For each block found, push `BlockFold { opener: block.span.start, info:
      BlockInfo { kind: block_kind_name(&block.kind), is_short_if:
      matches!(block.kind, BlockKind::If { .. }) && is_short_if(block,
      diagnostics), counterpart: counterpart_for(block, diagnostics) } }` —
      the exact same three private-helper calls `block_at` already makes,
      unchanged. `BlockInfo` itself is not modified.
- [x] T003 [P] [US1] Add unit tests to
      `crates/voyager-core/tests/block_resolution.rs` for `all_blocks`:
      one explicitly-closed block of each of the 7 kinds (`If`, `Loop`,
      `Run`, `Process`, `JLoop`, `LinkLoop`, `DistributeMultistep`) each
      produces a `BlockFold` with `info.counterpart == Some(closer_span)`;
      an implicitly-closed `Run` and an implicitly-closed `Process` each
      produce `info.counterpart == Some(...)` per the existing rule 4/5
      derivation; a short-`IF` produces `info.is_short_if == true` and
      `info.counterpart == None`; a genuinely unmatched `If`/`Loop`/`Run`
      each produce `info.counterpart == None`; a nested block (e.g. a `LOOP`
      inside an `IF`) produces a `BlockFold` for both the outer and the
      inner block independently. Depends on T002.
- [x] T004 [US1] Create `crates/drut-lsp/src/folding.rs` with `pub fn
      handle(state: &ServerState, params: &lsp_types::FoldingRangeParams) ->
      Option<Vec<lsp_types::FoldingRange>>` (contracts/folding-range-api.md):
      look up the document via `state.get(uri)`, returning `None` if not
      open (matches `hover::handle`'s own "unknown document" behavior — not
      the "nothing to fold" case, which is `Some(vec![])`). Block ranges:
      `voyager_core::block_resolution::all_blocks(&doc.parse_result.nodes,
      &doc.parse_result.diagnostics)`, filtered to `info.counterpart.is_some
      ()`, each mapped via `to_lsp_position` (both `opener` and the
      `counterpart` span's start) into `FoldingRange { start_line, end_line,
      kind: Some(FoldingRangeKind::Region), ..Default::default() }`. Comment
      ranges: `voyager_core::tokenize(&doc.text)`, filtered to `TokenKind::
      BlockComment { unterminated: false }`, each mapped via `to_lsp_position`
      (both endpoints of the token's own `span`) into `FoldingRange { ...,
      kind: Some(FoldingRangeKind::Comment), .. }`. **Apply the FR-008
      zero-span guard (`start_line >= end_line` → drop) to *both* streams
      independently, before concatenating them** — not only to block ranges.
      For blocks this is defensive today (no block kind's current rules
      produce a same-line `counterpart`, research.md §5); for comments it is
      load-bearing (a single-line `/* note */` comment genuinely has
      `span.start.line == span.end.line` and is `unterminated: false` like
      any terminated comment, so nothing upstream already excludes it —
      caught during `/speckit-analyze` review, research.md §5). Concatenate
      both filtered streams and return `Some(...)`. Depends on T002.
- [x] T005 [US1] Wire the capability and dispatch in
      `crates/drut-lsp/src/lib.rs`: add `pub mod folding;`; add
      `folding_range_provider: Some(lsp_types::FoldingRangeProviderCapability
      ::Simple(true))` to `server_capabilities()`; import
      `lsp_types::request::FoldingRangeRequest` and add a `FoldingRangeRequest
      ::METHOD` match arm in `handle_request`, following the exact same
      `serde_json::from_value::<lsp_types::FoldingRangeParams>(req.params)` →
      `send_ok`/`send_err` pattern every existing arm already uses. Depends
      on T004.
- [x] T006 [P] [US1] Add unit tests to `folding.rs`'s own test module
      (`#[cfg(test)] mod tests`, same style as `hover.rs`'s): one test per
      explicitly-closed block kind confirming a `Region`-kind range from
      opener to counterpart line; implicitly-closed `Run` and `Process` each
      produce a correct range; short-`IF` produces no range for itself
      (while a `Region` range is still produced for any block enclosing it,
      if applicable); a genuinely unmatched `If`/`Loop`/`Run` produces no
      range; a well-formed **multi-line** block comment produces a
      `Comment`-kind range; **a single-line block comment (`/* note */`,
      opening `/*` and closing `*/` on the same physical line) produces NO
      range** (FR-008 applied to the comment stream — the specific gap found
      during `/speckit-analyze` review); an
      unclosed block comment produces no range; nested blocks each get
      independent, correct ranges; a document with zero blocks/comments
      returns `Some(vec![])` (FR-011); a request for a URI never opened
      returns `None`. Depends on T005.
- [x] T007 [US1] Add a real `textDocument/foldingRange` protocol test to
      `crates/drut-lsp/tests/protocol_smoke.rs` (same `Connection::memory()`
      pattern the file's existing formatting/hover tests already use),
      covering spec.md's three US1 Acceptance Scenarios directly: (1) an
      `IF`/`ENDIF` block's range spans exactly the lines strictly between
      opener and closer; (2) a `LOOP` nested inside an `IF` produces two
      independent ranges, the inner fully contained within the outer's
      line span; (3) a multi-line block comment produces a `Comment`-kind
      range from its opening to its closing line. Depends on T005, T006.

**Checkpoint**: Folding works end-to-end for every block kind and block
comments, through the real LSP protocol, not just at the unit level.

---

## Phase 3: User Story 2 - Fold ranges stay correct as the document is edited (Priority: P2)

**Goal**: A folding-range request after a `didChange` reflects the
document's current text, never a stale parse.

**Independent Test**: Open a document, add a line inside an existing block's
body via `didChange`, request folding ranges again; confirm the range now
covers the new line.

### Implementation for User Story 2

- [x] T008 [US2] Add a live-edit test (`crates/drut-lsp/src/folding.rs`'s
      test module or `protocol_smoke.rs`, whichever the block-range tests
      from T006/T007 already live in): open a document with a `LOOP`/
      `ENDLOOP` block, request folding ranges, apply a `didChange` that
      inserts a new line inside the loop body, request folding ranges again,
      and assert the block's `end_line` increased by exactly the number of
      inserted lines (US2 Acceptance Scenario 1). This proves, specifically
      for folding, the guarantee `document_store.rs`'s own `OpenDocument::
      replace` already provides structurally (`did_change_replaces_and_
      reparses`) — not new plumbing, dedicated proof. Depends on T005.
- [x] T009 [P] [US2] Add a test confirming US2 Acceptance Scenario 2: open a
      document with an `IF`/`ENDIF` block, apply a `didChange` that deletes
      the `ENDIF` line entirely (leaving the `IF` structurally unmatched),
      request folding ranges, and assert no range is offered for that
      opener (FR-005, proven as a live-edit scenario specifically — distinct
      from T003/T006's static-document unmatched-block coverage). Depends on
      T005.

**Checkpoint**: Folding never shows a stale view of the document — proven
directly through an edit-then-request cycle, not assumed from the parse
pipeline's general behavior.

---

## Phase 4: User Story 3 - Fold everything / unfold everything (Priority: P3)

**Goal**: The editor's native "Fold All"/"Unfold All" commands work
correctly the moment the capability is registered, with no block or comment
silently excluded.

**Independent Test**: Request folding ranges for a document with several
nested/sibling blocks and a block comment; confirm the full returned set
matches every foldable construct with no omissions.

### Implementation for User Story 3

- [x] T010 [US3] Add a "Fold All" coverage test to `protocol_smoke.rs`: a
      document containing at least one of every block kind, a nested block,
      and a block comment; request folding ranges once; assert the returned
      set's size and line-spans exactly match every foldable construct in
      the document, including asserting a single-line block comment in the
      fixture correctly gets no range (SC-002 — "no block or comment
      silently excluded for a reason other than FR-004/FR-005/FR-007/
      FR-008's documented exceptions"). Depends on T006.
- [x] T011 [US3] Manual verification in a real VS Code instance
      (quickstart.md step 5): launch the extension development host against
      a real multi-block `.s`/`.block` corpus file containing an `IF`, a
      `LOOP`, an implicitly-closed `RUN`, and a block comment. Confirm: fold
      controls appear at the correct lines and nowhere else (no control on
      a short-`IF` line); collapsing a nested block and re-expanding its
      parent restores it correctly; "Fold All" collapses everything in one
      action and "Unfold All" restores the original view; editing inside a
      block's body and re-collapsing picks up the new content with no
      reload. Report the outcome of each check explicitly — this is a real
      manual verification step, not a formality. Depends on T007, T010.

**Checkpoint**: Every user-facing folding interaction (individual collapse,
Fold All/Unfold All, live-edit responsiveness) verified both through the
protocol and in a real editor.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Full-corpus cross-check and whole-workspace re-proof, once all
three stories are done.

- [x] T012 Add `crates/drut-lsp/tests/folding_corpus.rs` (same
      `DRUT_CORPUS_PATH`-gated, `#[ignore]`'d pattern
      `diagnostics_corpus.rs` already establishes — reuse its `common`
      helper module), with **two** full-corpus assertions, matching SC-003's
      own "block **or** comment" wording (added during `/speckit-analyze`
      remediation — the original draft of this task covered only the first
      of the two below):
      1. **Blocks**: for every file in the real 161-file corpus, assert
         `voyager_core::block_resolution::all_blocks` and the existing
         `voyager_core::block_at` **agree with each other** — every block
         `all_blocks` reports with `info.counterpart.is_some()` is
         independently confirmed by calling `block_at(nodes, diagnostics,
         block_fold.opener)` and checking its own `counterpart` matches, and
         every block `all_blocks` reports with `info.counterpart.is_none()`
         (short-IF or unmatched) agrees the same way. This is the concrete
         proof that the new enumeration function and the existing
         single-position query never diverge, across every real block shape
         in the corpus, not just the hand-written T003 cases.
      2. **Block comments**: for every file in the same corpus, assert
         `voyager_core::tokenize(&text)` filtered to every `TokenKind::
         BlockComment` token produces exactly one folding range (via the
         same handler logic under test, or the same filter/map/guard
         reproduced directly against the token stream) when `unterminated ==
         false` **and** the token spans more than one line, and exactly zero
         ranges when `unterminated == true` **or** the token is a
         single-line comment (`span.start.line == span.end.line`) — the
         same full-corpus proof standard as (1), extended to close E1 from
         `/speckit-analyze` review (the original draft only cross-checked
         blocks, leaving comment-folding correctness unproven at corpus
         scale despite SC-003 claiming it for both).
- [x] T013 `cargo test --release --workspace` and `cargo clippy --workspace
      --all-targets -- -D warnings`, both clean.
- [x] T014 Full 161-file corpus revalidation on the *existing* diagnostics
      surface (quickstart.md step 6, first command) — still 161/161 clean, a
      pure regression check, since this feature adds no new diagnostic and
      changes no existing one:
      ```powershell
      $env:DRUT_CORPUS_PATH = "path\to\WF-TDM-Official-Releases"
      cargo test --release -p drut-lsp --test diagnostics_corpus -- --ignored
      cargo test --release -p drut-lsp --test folding_corpus -- --ignored
      ```
- [x] T015 Run quickstart.md end-to-end (all 7 steps); confirm each step's
      expected outcome individually before reporting the feature done.

**Checkpoint**: Feature-complete against spec.md; the new enumeration
function proven to agree with the existing single-position query across the
full real corpus, and block-comment folding independently proven correct
across the same full corpus (both halves of SC-003, not just blocks); every
adapter-visible behavior verified both automatically and manually.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **User Story 1 (Phase 2)**: Depends on Setup. This is the foundation — US2
  and US3 both build on T002/T005.
- **User Story 2 (Phase 3)**: Depends on US1's T005 (the handler must exist
  to request ranges against, before and after an edit).
- **User Story 3 (Phase 4)**: Depends on US1's T006/T007 (folding must
  already work correctly per-block before a "does nothing get missed"
  aggregate check makes sense).
- **Polish (Phase 5)**: Depends on all three stories being complete.

### Within User Story 1

- T002 (the new `voyager-core` function) before T003 (its unit tests) and
  before T004 (the handler that calls it). T004 before T005 (dispatch
  wiring calls the handler) before T006 (handler unit tests) and T007
  (protocol test, which also needs T006's coverage as a base).

### Parallel Opportunities

- T003 can proceed in parallel with T004/T005 once T002 lands (different
  crates, no shared file).
- T006 and T008/T009 (once their own dependencies land) touch the same file
  (`folding.rs`'s test module) — sequence them rather than truly
  parallelizing, even though marked independently dependent.
- T009 can proceed in parallel with T008 once T005 lands (independent test
  cases, same file — coordinate order of insertion, not blocked on each
  other's content).

---

## Implementation Strategy

### Single Pass (all three stories are small and share one core change)

1. Setup → baseline confirmed clean.
2. User Story 1 → the one real implementation surface: `all_blocks` in
   `voyager-core` (research.md §1's minimal, additive reach-back) plus
   `folding.rs`'s handler and capability wiring in `drut-lsp`. Everything
   else in this feature builds on this phase alone.
3. User Story 2 → dedicated proof that live-edit correctness, already
   guaranteed structurally by `document_store.rs`, holds for folding
   specifically.
4. User Story 3 → dedicated proof that nothing is silently missed in
   aggregate, plus the real-editor manual check.
5. Polish → the full-corpus cross-check between the new enumeration function
   and the existing single-position query, whole-workspace re-proof, and
   quickstart execution.

---

## Notes

- T002 is this feature's one meaningful `voyager-core` change — everything
  else in Phase 2 is `drut-lsp`-side translation over already-public
  surface (`tokenize`, `TokenKind::BlockComment`, `to_lsp_position`). See
  research.md §1 for why T002 could not be avoided entirely while still
  honoring constitution Principle I.
- T012 is not a duplicate of T003 — T003 proves `all_blocks`'s behavior
  against hand-constructed shapes; T012 proves `all_blocks` never diverges
  from the already-trusted `block_at` across every real block shape in the
  161-file corpus, a cross-check T003 alone cannot provide.
- Commit after each task or logical group.
