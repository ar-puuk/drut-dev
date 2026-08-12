---

description: "Task list for FMT Region Markers"
---

# Tasks: FMT Region Markers

**Input**: Design documents from `/specs/010-fmt-region-markers/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/,
quickstart.md (all present)

**Tests**: Included — a formatter-scope feature touching real rendering
output requires real coverage: `voyager-core`'s own unit tests (including
the load-bearing opener-residue interaction research.md §2 identified, and
its two sibling-mechanism interaction tests added after `/speckit-analyze`
review — T007/T008 below), golden-fixture proof, and dedicated
adapter-level tests at every integration point named in FR-007/User Story
3, matching the discipline `008`/`009` each already established in this
same module.

**Organization**: Three P1/P2 user stories, matching spec.md exactly — US1
(protect a range — the entire mechanism), US2 (an unclosed marker protects
to EOF and is visibly surfaced, FR-010), US3 (the same behavior verified
independently at every adapter surface, not inferred). US1 is the
foundation everything else builds on; US2 and US3 both depend on it but are
otherwise independent of each other.

**Everything in this file's scope was measured against the real, current
codebase during planning (research.md §1-§5), not estimated**:

- Marker recognition reuses the existing `TokenKind::LineComment` — no new
  lexer/parser/grammar shape (research.md §1, §5).
- Protection must be gated at **collection** time (inside
  `plan_indentation`/`plan_block`/`plan_children`/`push_if_present`), not
  at the final render loop — render-time filtering was shown in research.md
  §2 to reproduce the exact "opener residue" failure mode `007`'s
  diagnosed-block-skip mechanism was built to avoid. **This is the single
  most important correctness property in this feature and gets its own
  dedicated regression test (T006), not just incidental coverage.**
- Five gate points total: 4 `plan.insert` call sites (contracts/
  fmt-region-markers.md) + `push_if_present`'s single funnel point for
  every casing edit.
- The unclosed-marker notice (FR-010) is a dedicated, non-`Diagnostic`
  signal (`FormatResult.unclosed_fmt_off_markers` + a standalone
  `unclosed_fmt_off_markers()` function) — **not** a new `DiagnosticKind`
  variant, per the owner's explicit steer during spec review.

**Post-`/speckit-analyze` remediation (2026-08-12)**: five coverage gaps
and one wording inconsistency were found and closed before implementation
— T007, T008, T009, T011 are new tasks; T019 (MCP) was expanded; spec.md's
Assumptions section was reworded (see spec.md's own amendment). Priority
order for review: T008 (007×010 interaction) and T019's protection-survives
addition (MCP) matter most — both close gaps at a MUST-requirement
boundary, not cosmetic gaps.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an
  incomplete sibling task)
- **[Story]**: US1/US2/US3 — omitted for Setup/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/src/format.rs` — marker recognition,
  `protected_regions`, the five gate points, `unclosed_fmt_off_markers`,
  `FormatResult`'s new field, and this crate's own test module.
- `crates/voyager-core/src/lib.rs` — re-export
  `unclosed_fmt_off_markers`.
- `crates/voyager-core/tests/fixtures/golden/` — new hand-written fixtures
  with synthetic marker pairs, plus a new real-corpus-derived set (existing
  `real_corpus/` fixtures themselves untouched — SC-002).
- `crates/drut-cli/src/format_cmd.rs`, `tests/format_flags.rs` — the new
  report field, stderr notice, and its test coverage (US3).
- `crates/drut-mcp/src/format.rs` — the new response field, its test
  coverage, and its protection-survives coverage (US3).
- `crates/drut-lsp/src/diagnostics.rs` — the new independent diagnostics
  stream and its test coverage (US3).
- `crates/drut-lsp/src/formatting.rs`, `range_formatting.rs` — new
  protection-survives tests, no code change (US3).
- `specs/002-cli-check-format/spec.md` +
  `specs/002-cli-check-format/contracts/formatting-api.md` — amended per
  `contracts/fmt-region-markers.md`'s exact replacement text (Polish).

---

## Phase 1: Setup

- [ ] T001 Confirm baseline: `cargo build --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings` both clean,
      on this fresh branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: User Story 1 - Protect a hand-tuned range from reformatting (Priority: P1) 🎯 MVP

**Goal**: A `; FMT: OFF`/`; FMT: ON` region is reproduced byte-for-byte
untouched; everything outside every region continues to format exactly as
it does today.

**Independent Test**: Format a fixture with a `; FMT: OFF`/`; FMT: ON` pair
wrapping deliberately "wrong" indentation and casing; confirm the wrapped
lines are byte-identical before and after, while lines outside the pair are
normalized exactly as they would be without any markers present.

### Implementation for User Story 1

- [ ] T002 [US1] Add marker-recognition logic to
      `crates/voyager-core/src/format.rs` (research.md §1/§4): given a
      `TokenKind::LineComment` token, determine whether it is a whole-line
      `; FMT: OFF`/`; FMT: ON` marker — no other token in the stream
      shares its `span.start.line` (whole-line check), and
      `token.text.trim_start_matches(';').trim()` splits once on `:` with
      both trimmed sides case-insensitively equal to `("FMT", "OFF")` or
      `("FMT", "ON")`.
- [ ] T003 [US1] Add the internal scan function
      `fn protected_regions(tokens: &[Token]) -> (BTreeSet<u32>, Vec<Position>)`
      to `format.rs` per contracts/fmt-region-markers.md's exact
      left-to-right state-machine algorithm: `FMT: OFF` while closed opens
      a region; `FMT: OFF` while already open is a no-op; `FMT: ON` while
      open closes the region inclusive of both marker lines; `FMT: ON`
      while closed is a no-op; any region still open at end-of-scan
      contributes every remaining line through EOF to the protected set
      and its opening marker's position to the second return value.
      Depends on T002.
- [ ] T004 [US1] Wire `protected_regions` into `render`: tokenize `source`,
      compute `(protected, _unclosed)`, and thread `&protected` as a new
      parameter through `plan_indentation`/`plan_block`/`plan_children`
      (guard every `plan.insert(line, value)` call with
      `if !protected.contains(&line)`) and through
      `collect_casing_edits`/`collect_block_casing_edits`/
      `collect_statement_casing_edits` down to `push_if_present` (single
      guard at its top, since every casing edit already funnels through
      it). No change to the final per-line render loop — a line with no
      plan entry and no casing edit is already reproduced untouched.
      Depends on T003.
- [ ] T005 [US1] Add unit tests to `format.rs`'s own test module covering
      every Edge Case spec.md names: a single protected range with wrong
      indentation and casing left untouched; a file with no markers
      produces identical output to before this feature; multiple
      non-overlapping regions each independently protected; a duplicate
      `; FMT: OFF` while already open is a no-op (US1 Acceptance Scenario
      4); a stray `; FMT: ON` with no open region is a no-op (US1
      Acceptance Scenario 5); a region straddling a block boundary
      (opens inside an `IF`, closes outside it); a whole-file-is-one-region
      case (including the no-closing-marker variant); **and (added after
      `/speckit-analyze` review, FR-009's own named example) marker-looking
      text (`; FMT: OFF`) appearing *inside* a real `/* ... */` block
      comment is correctly NOT treated as a marker** — the exact
      false-positive research.md §1 chose tokenizer-based recognition
      specifically to avoid; confirm the surrounding block comment's
      content (and everything after it) formats exactly as if that text
      weren't there. Depends on T004.
- [ ] T006 [US1] **Opener-residue regression test** (research.md §2's
      load-bearing finding — the exact interaction `007`'s diagnosed-block-
      skip mechanism exists to avoid, now re-created by markers instead of
      a diagnostic). Construct a fixture where a block's **opener** line
      sits inside a protected region, at a real on-disk column that
      differs from what normalization would compute for it, and at least
      one **child** statement of that block sits outside the region (after
      the matching `; FMT: ON`). Assert both: (a) the opener line is
      reproduced at its true original on-disk column, unchanged; (b) the
      out-of-region child is indented relative to the opener's *actual*
      on-disk column, not a discarded planned value. Construct the fixture
      so these two values differ — a test where they happen to coincide
      would pass even under the render-time-filtering bug research.md §2
      ruled out, so it must not be built that way. Depends on T004.
- [ ] T007 [US1] **009×010 interaction test** (added after
      `/speckit-analyze` review, G5 — matches this project's own
      precedent of an explicit cross-feature interaction test for the
      analogous 007+008 case, commit `0c8b51b`). Format a fixture
      containing a protected **top-level** (depth-0) line under
      `TopLevelIndentMode::Normalize` (`--top-level-indent=normalize`);
      assert the protected line stays at its original on-disk column,
      **not** forced to column 0 — proving gate point #1
      (`plan_indentation`'s top-level insert, only reached under
      `Normalize` mode) is guarded by `protected` exactly like the other
      three gate points. Build the fixture so the protected line's real
      column is non-zero, so a bug that skipped this specific gate point
      would visibly force it to 0 rather than passing by coincidence.
      Depends on T004.
- [ ] T008 [US1] **007×010 interaction test** (added after
      `/speckit-analyze` review, G5, extended per owner request beyond the
      009 case above). Format a fixture where a protected region
      (`; FMT: OFF`/`; FMT: ON`) contains, or sits immediately adjacent
      to, a diagnosed/unmatched block opener (one of `UnmatchedIf`/
      `UnmatchedLoop`/`UnmatchedRun`/`UnmatchedProcess` — `007`'s
      `diagnosed_block_openers`-gated children-skip, `plan_block`'s
      `if diagnosed_openers.contains(&block.span.start) { return; }`
      early-return). `007`'s skip and this feature's marker-gate are both
      independent "don't touch this range" mechanisms that can apply to
      overlapping or adjacent territory — confirm they compose correctly
      rather than one silently overriding, duplicating, or fighting the
      other: (a) content inside the marked region stays untouched by
      marker-protection regardless of whether it's also diagnosed; (b) a
      diagnosed block's own children that fall *outside* the marked region
      remain independently protected by `007`'s unchanged mechanism,
      unaffected by this feature; (c) the diagnostic itself (`parse`'s
      output) is unaffected by the presence of markers (ties to T009).
      Depends on T004.
- [ ] T009 [US1] **`parse()`-untouched test** (added after
      `/speckit-analyze` review, G4). Add a direct test asserting
      `parse(source)` produces byte-for-byte identical `nodes` and
      `diagnostics` whether or not a structurally-equivalent file contains
      `; FMT: OFF`/`; FMT: ON` markers (FR-006) — confirms markers are a
      pure formatting-only concern with zero effect on tokenize/parse
      output, proven directly rather than left as an architectural
      assumption. No implementation dependency — `parse()` is unaffected
      by this feature by construction — sequenced here for coverage
      tracking alongside US1's other correctness proofs.
- [ ] T010 [P] [US1] Add new hand-written golden fixtures with synthetic
      `; FMT: OFF`/`; FMT: ON` marker pairs under
      `crates/voyager-core/tests/fixtures/golden/` (existing fixtures
      untouched — SC-002), including one exercising T006's opener-residue
      shape end-to-end through the real `format_corpus.rs` golden-diff
      pipeline, not only at the unit-test level. Each new fixture's
      initial expected golden output must be manually verified correct on
      its own merits (not just "whatever the new code currently
      produces") before being locked in — there is no prior committed
      state to diff against, so this is the equivalent rigor to 008/009's
      per-file human review of a regenerated diff. Depends on T004.
- [ ] T011 [P] [US1] **Real-corpus-derived marker fixtures** (added after
      `/speckit-analyze` review, G1 — SC-001 explicitly names "the real
      161-file corpus with synthetic marker pairs inserted" as part of
      its own evidence requirement; T010 alone, being purely hand-written,
      does not satisfy that clause). Create new fixtures under
      `crates/voyager-core/tests/fixtures/golden/` (a new subdirectory,
      e.g. `real_corpus_fmt_markers/`) derived from a small sample (3-5)
      of real-world script shapes already present in the existing
      `real_corpus/` set, with synthetic marker pairs inserted around a
      genuinely would-otherwise-reformat section in each. The existing
      `real_corpus/` fixtures themselves remain completely untouched
      (SC-002) — this is an additive, separate fixture set proving
      protection against real-world script shapes, not a modification of
      the existing corpus proof. Depends on T004.

**Checkpoint**: Protection is real, gated correctly at collection time
(not render time), and independently proven against the specific failure
mode it must avoid, its interactions with both sibling "don't touch this
range" mechanisms in this codebase (`007`, `009`), and against real-world
script shapes.

---

## Phase 3: User Story 2 - An unclosed `; FMT: OFF` protects to end of file, visibly (Priority: P2)

**Goal**: An unmatched `; FMT: OFF` protects through end-of-file, and its
location is surfaced via a dedicated, non-`Diagnostic` signal rather than
left silent.

**Independent Test**: Format a fixture containing a `; FMT: OFF` marker
with no following `; FMT: ON`; confirm every line from the marker to the
end of the file is left untouched, the file formats without error, and a
visible notice identifying the unclosed marker is present in the result.

### Implementation for User Story 2

- [ ] T012 [US2] Add
      `pub fn unclosed_fmt_off_markers(source: &str) -> Vec<Position>` to
      `format.rs` (contracts/fmt-region-markers.md) — tokenizes `source`
      internally and returns `protected_regions`'s second return value
      directly, for callers that want this signal without a full
      format pass. Depends on T003.
- [ ] T013 [US2] Add `unclosed_fmt_off_markers: Vec<Position>` to
      `FormatResult`; populate it in `format`/`format_bytes` from the same
      scan `render` already computes (avoid tokenizing twice — thread the
      unclosed list out of `render`, or have `format`/`format_bytes` call
      `protected_regions` once and pass the protected set into `render`;
      either is correct per contracts' note that this wiring detail has no
      behavioral difference). Depends on T004, T012.
- [ ] T014 [P] [US2] Add unit tests: an unmatched `; FMT: OFF` protects
      every line through end-of-file (protection behavior, not just the
      notice); `unclosed_fmt_off_markers` standalone returns the correct
      position(s) directly, independent of a full `format()` call; the
      common case (no markers, or every marker matched) returns an empty
      `Vec` from both the standalone function and `FormatResult`'s field.
      Depends on T013.
- [ ] T015 [P] [US2] Add an idempotency test: formatting a fixture
      containing protected regions, including at least one unclosed
      `; FMT: OFF`, twice in a row produces byte-identical output on both
      passes (FR-008/SC-003). Depends on T004.

**Checkpoint**: Unclosed markers behave predictably and are never silent —
matches this project's own established stance against silent
unbounded-scope behavior (`UnmatchedProcess`, `007`'s formatter-residue
fix).

---

## Phase 4: User Story 3 - Markers are recognized consistently everywhere formatting happens (Priority: P2)

**Goal**: Protection and the unclosed-marker notice both behave
identically through the CLI, both LSP formatting handlers, and the MCP
`format` tool.

**Independent Test**: Format the same fixture containing a protected range
(and, separately, one containing an unclosed marker) through the CLI, both
LSP handlers, and the MCP tool independently; confirm all four leave the
protected range untouched identically, and that the unclosed-marker notice
appears in each surface's own idiom.

### Implementation for User Story 3

- [ ] T016 [US3] Re-export `unclosed_fmt_off_markers` from
      `crates/voyager-core/src/lib.rs` alongside the existing
      `format`/`format_bytes` re-exports. Depends on T012.
- [ ] T017 [P] [US3] In `crates/drut-cli/src/format_cmd.rs`: add
      `unclosed_fmt_off_files: Vec<(PathBuf, Vec<Position>)>` to
      `FormatReport`, populated in the existing per-file loop (same
      treatment as `recovered_encoding_files`/`unsafe_encoding_files` —
      every mode, not just `--write`); add a third `eprintln!` block to
      `print_report` reporting each file and its unclosed marker line(s);
      no `derive_exit_outcome` change (informational only, matches FR-010's
      "no error occurs"). Depends on T013.
- [ ] T018 [P] [US3] Add coverage to
      `crates/drut-cli/tests/format_flags.rs`: a protected range survives
      `drut format`/`--check`/`--diff`/`--write` identically; the new
      stderr notice's exact text appears for a file with an unclosed
      marker and does not appear for a file with none. Depends on T017.
- [ ] T019 [P] [US3] In `crates/drut-mcp/src/format.rs`: add
      `unclosed_fmt_off_lines: Vec<u32>` to `FormatResultDto`, mapped from
      `result.unclosed_fmt_off_markers.iter().map(|p| p.line)`; add a test
      confirming correct population and the empty common case. **Also add
      a dedicated protection-survives test** (added after
      `/speckit-analyze` review, G2 — FR-007/US3 explicitly requires
      identical protection at every adapter surface including MCP, but
      this task previously only proved the notice field, not that MCP's
      `format()` actually protects a range): a fixture with a protected
      range, formatted through this same `format()` function, leaves the
      protected range byte-for-byte unchanged while normalizing everything
      else — matching the rigor T018/T021/T022 already apply to CLI/LSP.
      Depends on T013.
- [ ] T020 [P] [US3] In `crates/drut-lsp/src/diagnostics.rs`: `publish()`
      gains a second, independently-sourced diagnostics stream built from
      `voyager_core::unclosed_fmt_off_markers(&doc.text)`, each mapped to
      `DiagnosticSeverity::HINT`, `source: "drut-fmt"` (distinct from the
      existing structural diagnostics' `"drut"`), `code:
      "UnclosedFmtOff"`, chained onto (not replacing) the existing
      structural-diagnostics list; add a test confirming exactly one such
      diagnostic for a fixture with one unclosed marker, zero for a clean
      fixture, and confirm no existing structural-diagnostic test's
      assertions change (purely additive stream). Depends on T016.
- [ ] T021 [P] [US3] Add a test to `crates/drut-lsp/src/formatting.rs`'s
      own test module: a document containing a protected range, formatted
      via the existing `handle` function, leaves the protected range
      untouched while normalizing everything else (no code change to
      `formatting.rs` itself — protection is inherited from
      `voyager-core`). Depends on T004.
- [ ] T022 [P] [US3] Add a test to
      `crates/drut-lsp/src/range_formatting.rs`'s own test module: same
      shape as T021, via `textDocument/rangeFormatting`'s `handle`
      function. Depends on T004.

**Checkpoint**: Protection and the unclosed-marker notice are both
independently proven at every integration point named in FR-007/FR-010 —
CLI, both LSP surfaces, and the MCP tool (protection itself, not only the
notice, now proven at all four surfaces).

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Spec-doc amendment and whole-workspace/full-corpus re-proof,
once all three stories are done.

- [ ] T023 Amend `specs/002-cli-check-format/spec.md` (new FR, numbered
      against the live file at implementation time — per `009`'s own "FR
      number collision" precedent, do not assume a number in advance) and
      `contracts/formatting-api.md`, using
      `contracts/fmt-region-markers.md`'s exact replacement text.
- [ ] T024 `cargo test --release --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings`, both
      clean.
- [ ] T025 Full 161-file corpus revalidation across all three adapter
      surfaces (quickstart.md step 7), each reported individually:
      ```powershell
      $env:DRUT_CORPUS_PATH = "path\to\WF-TDM-Official-Releases"
      cargo test --release -p drut-cli --test fixture_corpus_e2e -- --ignored
      cargo test --release -p drut-lsp --test diagnostics_corpus -- --ignored
      cargo test --release -p drut-mcp --test diagnostics_corpus -- --ignored
      ```
      Expected and required: still 161/161 clean — a pure regression
      check, since no real corpus file contains markers today.
- [ ] T026 Run quickstart.md end-to-end (all 8 steps); confirm each step's
      expected outcome individually before reporting the feature done.

**Checkpoint**: Feature-complete against spec.md; opener-residue
interaction and both sibling-mechanism interactions (`007`, `009`)
independently proven; every adapter surface verified for both protection
and the notice; full corpus re-proven clean.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **User Story 1 (Phase 2)**: Depends on Setup. This is the foundation —
  US2 and US3 both build on T003/T004.
- **User Story 2 (Phase 3)**: Depends on US1's T003 (the scan must exist)
  and T004 (the render wiring must exist).
- **User Story 3 (Phase 4)**: Depends on US1's T004 (protection must exist
  to verify it at each adapter) and US2's T012/T013 (the notice must exist
  to surface it at each adapter).
- **Polish (Phase 5)**: Depends on all three stories being complete.

### Within User Story 1

- T002 before T003 (recognition must exist before the scan can use it)
  before T004 (the scan must exist before it can be wired in) before
  T005/T006/T007/T008 (tests need the real behavior to assert against) and
  T010/T011 (golden fixtures need the real behavior too).
- T009 has no real implementation dependency (it tests `parse()`, which
  this feature never touches) but is grouped with US1 for coverage
  tracking.
- T006 is the single highest-priority test in this entire feature — it is
  the direct proof of research.md §2's load-bearing design decision. T007
  and T008 are its two sibling-mechanism interaction tests (009, 007
  respectively), added after `/speckit-analyze` review — treat all three
  as a set, not independently optional.

### Parallel Opportunities

- T010, T011 can proceed in parallel with T005/T006/T007/T008 once T004
  lands.
- T014, T015 can proceed in parallel once T013/T004 land respectively.
- T017, T019, T020, T021, T022 can all proceed in parallel once their
  respective dependencies (T013 or T004 or T016) land — different files,
  non-conflicting.
- T018 depends on T017 (same file's new behavior); otherwise adapter test
  tasks are independent of each other.

---

## Parallel Example: Once T003/T004 Land (US1 core done)

```bash
Task: "T005: Edge Case unit tests in format.rs (incl. block-comment case)"
Task: "T006: opener-residue regression test"
Task: "T007: 009×010 interaction test (Normalize mode)"
Task: "T008: 007×010 interaction test (diagnosed block)"
Task: "T009: parse()-untouched test"
Task: "T010: new golden fixtures with synthetic marker pairs"
Task: "T011: real-corpus-derived marker fixtures"
Task: "T015: idempotency test (once T013 also lands)"
Task: "T021: drut-lsp formatting.rs protection-survives test"
Task: "T022: drut-lsp range_formatting.rs protection-survives test"
```

---

## Implementation Strategy

### Single Pass (all three stories are small and share one core change)

1. Setup → baseline confirmed clean.
2. User Story 1 → marker recognition, the scan, the five gate points, and
   — most importantly — the opener-residue regression test (T006) plus
   its two sibling-mechanism interaction tests against `009`'s
   `TopLevelIndentMode::Normalize` (T007) and `007`'s diagnosed-block skip
   (T008), proving research.md §2's collection-vs-render-time decision
   holds not just in isolation but against every other "don't touch this
   range" mechanism already in this module.
3. User Story 2 → the unclosed-marker notice's core-crate surface
   (`FormatResult` field + standalone function) and its own protection/
   idempotency proof.
4. User Story 3 → the four independent adapter-level verifications
   (CLI report+notice, MCP field **and protection**, LSP diagnostics
   stream, both LSP formatting handlers) named in FR-007/FR-010.
5. Polish → spec-doc amendment and whole-workspace/full-corpus re-proof,
   reported explicitly.

---

## Notes

- T006/T007/T008 (opener-residue, 009-interaction, 007-interaction) are
  this feature's three most important tests — together they are the
  concrete proof of the one design decision research.md flagged as
  load-bearing, checked against every sibling mechanism it can interact
  with. Do not treat any of the three as optional or foldable into T005's
  general Edge Case sweep.
- T013's exact wiring (whether `render` recomputes `protected_regions` or
  receives it as a parameter from `format`/`format_bytes`) is left to
  implementation judgment — contracts/fmt-region-markers.md is explicit
  that both are correct and behaviorally identical.
- Commit after each task or logical group.
