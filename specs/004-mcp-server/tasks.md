---

description: "Task list for Drut MCP Server"
---

# Tasks: Drut MCP Server

**Input**: Design documents from `/specs/004-mcp-server/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md (all present)

**Tests**: Included — constitution Principle IV/V and this feature's own Success
Criteria (SC-005 read-only guarantee, SC-006 full-corpus parity) require a real
test suite before merge, the same standard `003-lsp-vscode-extension` held
itself to. The `voyager-core::block_at` extraction (research.md §5) carries an
explicit, additional reporting requirement beyond "tests pass" — see Phase 5.

**Organization**: Tasks are grouped by user story (US1–US4, P1–P4 per spec.md),
so each can be implemented, tested, and shipped independently. US1 (`diagnose`)
has no dependency on any other story and is the suggested MVP. US3
(`query_structure`) uniquely requires a prerequisite refactor of already-shipped
`drut-lsp` code (the `block_at` extraction) before its own tool logic can be
written — that extraction is sequenced as US3's own first tasks, not
Foundational, since no other story touches it.

**Revision note (2026-08-10, post-`/speckit-analyze`)**: this version applies
fixes for four findings — F1 (per-tool contract-test files, so `[P]` markers
are actually true instead of four stories sharing one file), F2 (corrected
parallel-execution prose — registration is serial across *all four* stories,
not just US3's), C1 (T004 now also checks `drut-mcp` itself has no `drut-lsp`
dependency, the actual direction FR-011 requires), and C2 (a new dedicated
task for `diagnose`'s `InvalidEncoding`-via-`path` edge case). D1 was left
as-is per explicit decision (a defensible, deliberate choice, not a gap).
Task IDs from T011 onward shifted by one versus the pre-analysis version to
make room for C2's new task.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies — verified
  per-task below; a task sharing a file with an earlier sibling is
  deliberately *not* marked `[P]`, per F1's fix)
- **[Story]**: US1–US4 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

Four Rust crates under `crates/` (plan.md Structure Decision):

- `crates/voyager-core/` — existing crate; US3 adds `src/block_resolution.rs` +
  `tests/block_resolution.rs` to it. US1/US2/US4 touch nothing here.
- `crates/drut-cli/` — existing crate; this feature adds one `mcp` subcommand
  (Foundational phase only).
- `crates/drut-lsp/` — existing crate; US3 refactors (not extends) `src/hover.rs`.
- `crates/drut-mcp/` — new library crate this feature creates; the MCP server
  itself. Each tool's contract tests live in their own file
  (`tests/<tool>_contract.rs`, F1's fix) rather than one shared file, so every
  story's test task is genuinely parallel with its siblings.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Stand up the new `drut-mcp` crate so it builds, with its
dependency surface exactly as research.md §1/§2/§4 specify.

- [X] T001 Add `"crates/drut-mcp"` as a fourth workspace member in `Cargo.toml`
      (repo root) (plan.md Structure Decision).
- [X] T002 Create `crates/drut-mcp/Cargo.toml`: package `drut-mcp` (library, no
      `[[bin]]` — `drut mcp` stays a `drut-cli` subcommand per plan.md's
      Structure Decision), a path dependency on `voyager-core`, `rmcp` pinned
      `~3.1` with `default-features = false` and
      `features = ["server", "macros", "transport-io", "schemars"]`
      (research.md §1/§4 — deliberately excluding every HTTP-transport
      feature), `tokio = "1"` with the `rt-multi-thread`/`macros` features
      needed to construct and block on a runtime, `schemars`, `serde`
      (`derive` feature), `serde_json`.
- [X] T003 [P] Create `crates/drut-mcp/src/lib.rs` with a placeholder
      `pub async fn run() -> anyhow::Result<()> { Ok(()) }` (or equivalent
      minimal stdio-transport bring-up with zero tools registered yet) and
      confirm `cargo build -p drut-mcp` and
      `cargo clippy -p drut-mcp --all-targets -- -D warnings` succeed
      zero-warning.
- [X] T004 Confirm dependency isolation, **both directions** (quickstart.md
      step 1; extended per `/speckit-analyze` finding C1 — the original
      version of this task only checked one of the two directions FR-011
      actually requires):
      (a) `cargo tree -p voyager-core` and `cargo tree -p drut-lsp` show
      neither `tokio` nor `rmcp` anywhere in their trees (research.md §2's
      containment claim), **and**
      (b) `cargo tree -p drut-mcp` shows `drut-lsp` does **not** appear
      anywhere in it (FR-011's actual claim: `drut-mcp`'s own behavior never
      depends on `drut-lsp`/a running LSP session — this is the direction
      the original task omitted). Record both confirmations explicitly —
      this is the structural proof both claims rest on, not an assumption.

**Checkpoint**: `cargo build --workspace` passes with the new (near-empty)
`drut-mcp` crate in place.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The `drut mcp` CLI entry point and the shared `ScriptSource`
input-resolution logic every tool except `lookup_keyword` depends on.

**⚠️ CRITICAL**: No user story's tool can be exercised end-to-end through the
real `drut` binary until this phase is complete (individual tool *logic* can
still be unit-tested against `drut-mcp`'s own internal functions before this
phase finishes, per each story's own test tasks below).

- [X] T005 Add `Command::Mcp` to `crates/drut-cli/src/cli.rs` (no flags, same
      shape as `Command::Server`).
- [X] T006 Create `crates/drut-cli/src/mcp_cmd.rs`: thin dispatch — constructs
      a `tokio::runtime::Runtime` locally and blocks on `drut_mcp::run()`
      (research.md §2). Zero MCP protocol logic here (Principle I), mirroring
      exactly how `server_cmd.rs` contains zero LSP protocol logic.
- [X] T007 Wire `Command::Mcp => mcp_cmd::run()` into `crates/drut-cli/src/lib.rs`'s
      existing dispatch match (depends on T005, T006). Confirm
      `cargo build -p drut-cli` succeeds and `drut mcp` launches without
      panicking (a manual smoke check: run it, confirm it doesn't exit
      immediately with an error, then close stdin and confirm clean exit).
- [X] T008 [P] Create `crates/drut-mcp/src/source.rs`: the `ScriptSource`
      type (data-model.md §2) and its resolution function — validates
      exactly one of `text`/`path` is set (structured error otherwise, FR-002),
      reads file content via `voyager_core::decode::decode_bytes` when `path`
      is set (mirroring how `drut-lsp` already decodes bytes, never adding a
      second decode implementation), returns the resolved text plus whether
      it came from a path (needed by `diagnose`/`format` to decide whether
      `parse`/`format` or `parse_bytes`/`format_bytes` applies).
- [X] T009 [P] Unit tests for `source.rs` in
      `crates/drut-mcp/src/source.rs`'s own `#[cfg(test)] mod tests`: both
      `text`/`path` set → structured error; neither set → structured error;
      `text` only → resolves to that text; `path` only, a real fixture file →
      resolves to that file's content; `path` pointing at a nonexistent file →
      structured error, not a panic (Edge Cases).

**Checkpoint**: `drut mcp` runs (currently advertising zero tools);
`ScriptSource` resolution is fully tested and ready for every story to use.

---

## Phase 3: User Story 1 - Validate a script's structural correctness (Priority: P1) 🎯 MVP

**Goal**: The `diagnose` tool returns every reachable diagnostic category for a
given script, matching `drut check`'s own output exactly.

**Independent Test**: Call `diagnose` (directly against `drut-mcp`'s own
function, or through a real MCP client per quickstart.md step 7) with a
deliberately unmatched `IF` and confirm exactly one `UnmatchedIf` diagnostic
with a correct location; call it again with a valid script and confirm zero
diagnostics.

### Tests for User Story 1

- [X] T010 [P] [US1] Contract tests for `diagnose` in
      `crates/drut-mcp/tests/diagnose_contract.rs` (own file, F1's fix — no
      longer shared with the other three stories' contract tests): the three
      spec.md Acceptance Scenarios (unmatched `IF` → one `UnmatchedIf`; valid
      script → zero diagnostics; `path` input → identical result to that
      file's own `text`) per contracts/mcp-tools.md's `diagnose` section.
- [X] T011 [US1] `InvalidEncoding`-via-`path` test for `diagnose`, added in
      `crates/drut-mcp/tests/diagnose_contract.rs` (same file as T010 —
      deliberately **not** marked `[P]` against it, per F1's own fix
      philosophy: don't overclaim parallelism where a real file conflict
      exists) (`/speckit-analyze` finding C2): a fixture file containing a
      deliberately undecodable byte sequence (neither valid UTF-8 nor
      Windows-1252), read via `path` (never reachable via `text` — an MCP
      tool-call argument is JSON, which cannot carry an invalid byte
      sequence, Edge Cases), confirming `diagnose` reports an
      `InvalidEncoding` diagnostic — the one diagnostic category only
      reachable through the `path` input mode, per FR-003's own scoping.
- [X] T012 [P] [US1] Full-corpus diagnostic parity test in
      `crates/drut-mcp/tests/diagnostics_corpus.rs` (own file — parallel with
      T010/T011, which live in `diagnose_contract.rs`), gated behind
      `DRUT_CORPUS_PATH` and `#[ignore]` (same convention every prior phase's
      corpus test uses): for all 161 real files, `diagnose`'s output is
      diagnostic-category-and-location-identical to `drut check`'s own
      output for the same file (SC-006).

### Implementation for User Story 1

- [X] T013 [US1] Create `DiagnosticDto` (data-model.md §3) and the `diagnose`
      tool function in `crates/drut-mcp/src/diagnose.rs`: takes
      `DiagnosticsInput` (`ScriptSource` via T008), calls
      `voyager_core::parse`/`parse_bytes` depending on whether the source
      came from `text` or `path`, converts every `Diagnostic` in the result
      into a `DiagnosticDto` (never a narrowed subset — FR-003). Empty input
      → empty diagnostic list, not an error (Edge Cases).
- [X] T014 [US1] Register `diagnose` as an `rmcp` `#[tool]` on `drut-mcp`'s
      server-handler struct in `crates/drut-mcp/src/lib.rs` (depends on
      T003, T013). Confirm T010/T011's contract tests now pass through the
      real registered tool, not just the bare function.

**Checkpoint**: `diagnose` is fully functional and independently testable —
suggested MVP stopping point.

---

## Phase 4: User Story 2 - Normalize a script's formatting (Priority: P2)

**Goal**: The `format` tool returns fully reformatted text plus whether
anything changed, matching `drut format --diff`'s own semantics.

**Independent Test**: Call `format` with incorrectly-indented script text and
confirm the result's text is correctly indented with `changed: true`; call it
again with the result's own `text` fed back in and confirm `changed: false`
(idempotence).

### Tests for User Story 2

- [X] T015 [P] [US2] Contract tests for `format` in
      `crates/drut-mcp/tests/format_contract.rs` (own file, F1's fix): the
      three spec.md Acceptance Scenarios (incorrect indentation → corrected
      text, `changed: true`; already-correct text → byte-identical output,
      `changed: false`; feeding the tool's own output back in → `changed:
      false`, proving idempotence carries through) per
      contracts/mcp-tools.md's `format` section.

### Implementation for User Story 2

- [X] T016 [US2] Create `FormatResultDto` (data-model.md §4) and the `format`
      tool function in `crates/drut-mcp/src/format.rs`: takes `FormatInput`
      (`ScriptSource` via T008, optional `casing`), calls
      `voyager_core::format`/`format_bytes` with `FormatOptions` built from
      the optional `casing` field (absent → `FormatOptions::default()`,
      untouched casing, FR-005), converts the resulting `FormatResult` into
      a `FormatResultDto` (text, `changed`, `encoding_fidelity` — always all
      three together, FR-004).
- [X] T017 [US2] Register `format` as an `rmcp` `#[tool]` alongside
      `diagnose` in `crates/drut-mcp/src/lib.rs` (depends on T014, T016 —
      this same-file dependency on the prior story's own registration task
      is real and intentional, see "Dependencies & Execution Order" below).
      Confirm T015's contract tests now pass through the real registered
      tool.

**Checkpoint**: `diagnose` and `format` both independently functional.

---

## Phase 5: User Story 3 - Query which block encloses a position (Priority: P3)

**Goal**: The `query_structure` tool reports block kind and matched-counterpart
location for a position, using the exact same derivation `drut-lsp`'s hover
capability already implements — genuinely one implementation, not two.

**Independent Test**: Call `query_structure` on a script with an
implicitly-closed `RUN` block at a position on that block's opener line and
confirm block kind `Run` with a matched-counterpart location at the block's
resolved implicit-close point (not `null`, not the next `RUN`'s own line).

### `block_at` extraction (research.md §5) — prerequisite to this story's own tool, touches already-shipped code

- [X] T018 [US3] Create `BlockInfo`/`BlockKindName` and the `block_at`
      function (data-model.md §1, contracts/block-resolution-api.md) in a new
      `crates/voyager-core/src/block_resolution.rs`, moving (not
      copying — the private helpers this replaces are deleted from
      `hover.rs` in T020) the derivation logic currently in
      `crates/drut-lsp/src/hover.rs`'s `is_short_if`, `run_closed_implicitly`,
      `counterpart_for`, `find_block_at`, `find_hover_fact`, and
      `block_kind_name`. Add `pub mod block_resolution;` and the
      corresponding `pub use` to `crates/voyager-core/src/lib.rs`.
- [X] T019 [P] [US3] Unit tests for `block_at` in
      `crates/voyager-core/tests/block_resolution.rs`: port every case
      `drut-lsp/src/hover.rs`'s own (pre-extraction) test module already
      covered (block-style `IF`, short-`IF`, implicitly-closed `RUN`,
      position with no enclosing block) directly against
      `voyager_core::block_at`, independent of either caller.
- [X] T020 [US3] Refactor `crates/drut-lsp/src/hover.rs` (depends on T018):
      `handle` now calls `voyager_core::block_at` and translates the
      returned `BlockInfo` into `lsp_types::Hover` markdown; the five
      now-redundant private functions are deleted, not left dead. No other
      file in `drut-lsp` changes.
- [X] T021 [US3] **Dedicated extraction-verification step — reported as its
      own explicit, standalone result, never folded into a general "tests
      pass" summary** (quickstart.md step 2, this session's own explicit
      requirement): run and report, individually —
      (a) `cargo test -p voyager-core block_resolution::` (T019's new tests),
      (b) `cargo test -p drut-lsp --lib hover::` and
      `cargo test -p drut-lsp --test hover` (every pre-existing hover test,
      confirming each passes with **zero assertion changes** — any assertion
      that needed to change to stay green must be called out explicitly, not
      silently absorbed as a pass), and
      (c) the full 161-file corpus hover-parity check (T025 below, pulled
      forward and reported alongside (a)/(b) rather than deferred to Polish).
      Only once all three are true and reported as such does this extraction
      count as verified.

### Tests for User Story 3 (tool-level, beyond T019/T021's extraction verification)

- [X] T022 [P] [US3] Contract tests for `query_structure` in
      `crates/drut-mcp/tests/query_structure_contract.rs` (own file, F1's
      fix): the three spec.md Acceptance Scenarios (explicit `IF`/`ENDIF` →
      kind `If` + `ENDIF` location; implicitly-closed `RUN` → kind `Run` +
      resolved body-extent location, not the next `RUN`'s opener; position
      with no enclosing block → `kind` absent, not an error) per
      contracts/mcp-tools.md's `query_structure` section.

### Implementation for User Story 3

- [X] T023 [US3] Create `BlockInfoDto` (data-model.md §5) and the
      `query_structure` tool function in
      `crates/drut-mcp/src/query_structure.rs`: takes `StructuralQueryInput`
      (`ScriptSource` via T008, 1-based `line`/`column`), parses the source,
      clamps an out-of-range position to the nearest valid one (Edge Cases,
      same no-panic discipline as `contracts/block-resolution-api.md`
      requires), calls `voyager_core::block_at` (depends on T018), converts
      the resulting `Option<BlockInfo>` into a `BlockInfoDto` (`kind` absent
      is a normal successful result, FR-007 — never an error).
- [X] T024 [US3] Register `query_structure` as an `rmcp` `#[tool]` in
      `crates/drut-mcp/src/lib.rs` (depends on T017, T023 — same-file
      dependency on US2's own registration task, see "Dependencies &
      Execution Order" below). Confirm T022's contract tests now pass
      through the real registered tool.
- [X] T025 [P] [US3] Structural-query/hover parity test in
      `crates/drut-mcp/tests/structural_query_parity.rs` (own file), gated
      behind `DRUT_CORPUS_PATH` and `#[ignore]`: for the same real corpus
      positions `003-lsp-vscode-extension`'s own hover tests already use,
      confirm `query_structure`'s result matches what `drut-lsp`'s hover
      reports for the identical position (both now reading
      `voyager_core::block_at`, T018 — this is parity on the wiring, the
      derivation itself already proven correct in T019/T021). Pulled forward
      into T021's own reported result per that task's requirement, not a
      separate, later-reported check.

**Checkpoint**: `diagnose`, `format`, and `query_structure` all independently
functional; the extraction is verified and reported on its own terms.

---

## Phase 6: User Story 4 - Look up valid keyword-pair names (Priority: P4)

**Goal**: The `lookup_keyword` tool returns real, corpus-evidenced keyword-pair
candidates for a control word, and/or a spell-check suggestion for a token.

**Independent Test**: Call `lookup_keyword` with `enclosing_control_word: "RUN"`
and confirm `PGM`/`MSG`/`PRNFILE` appear in the result; call it with a token
one edit-distance from a real keyword and confirm a suggestion is returned.

### Tests for User Story 4

- [X] T026 [P] [US4] Contract tests for `lookup_keyword` in
      `crates/drut-mcp/tests/lookup_keyword_contract.rs` (own file, F1's
      fix): the four spec.md Acceptance Scenarios (`RUN` → includes
      `PGM`/`MSG`/`PRNFILE`; no control word → general-syntax fallback list;
      near-miss token → a suggestion naming the real keyword; exact-match
      token → `suggestion: None`) per contracts/mcp-tools.md's
      `lookup_keyword` section.

### Implementation for User Story 4

- [X] T027 [US4] Create `KeywordCandidateDto`/`SpellCheckSuggestionDto`
      (data-model.md §6) and the `lookup_keyword` tool function in
      `crates/drut-mcp/src/lookup_keyword.rs`: takes `KeywordLookupInput`
      (`enclosing_control_word: Option<String>`,
      `spellcheck_token: Option<String>` — no `ScriptSource`, resolved design
      question from spec.md), calls `voyager_core::keywords::
      completion_candidates` for the candidate list (FR-008) and, when
      `spellcheck_token` is present, `voyager_core::keywords::did_you_mean`
      (FR-009) — independently of each other, per data-model.md §6.
- [X] T028 [US4] Register `lookup_keyword` as an `rmcp` `#[tool]` in
      `crates/drut-mcp/src/lib.rs` (depends on T024, T027 — same-file
      dependency on US3's own registration task, see "Dependencies &
      Execution Order" below). Confirm T026's contract tests now pass
      through the real registered tool.

**Checkpoint**: All four tools independently functional — feature-complete
against spec.md.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Guarantees that span every tool, plus the manual client smoke
test and documentation.

- [X] T029 Read-only guarantee test in `crates/drut-mcp/tests/no_disk_writes.rs`
      (FR-010, SC-005): call all four tools (`format` especially — the one
      most tempting to implement as "format and save") against a fixture
      directory made read-only for the test's duration, and confirm every
      call still succeeds, proving no tool attempts a write rather than
      merely documenting that none should.
- [X] T030 [P] No-panic edge-case sweep in a new
      `crates/drut-mcp/tests/no_panic.rs` (own file, matching `drut-lsp`'s own
      convention, and consistent with F1's fix of never sharing a test file
      across otherwise-independent concerns): exercise every tool against
      the same edge-case document shapes `crates/drut-lsp/tests/no_panic.rs`
      already defines (empty document, unterminated block comment, stray
      closer with nothing open, etc.), reused as fixture content per
      quickstart.md step 3.
- [X] T031 `cargo test --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings`, both clean,
      confirming this feature introduces zero regressions anywhere in the
      four-crate workspace.
- [X] T032 Manual MCP-client smoke test per quickstart.md step 7 — point a
      real MCP-capable client at `drut mcp`, confirm all four tools appear
      with client-rendered parameter descriptions (proving `schemars`
      schemas actually reach a real client, not just that they compile),
      and exercise each tool once. Report what was actually observed at
      each step, not just that calls returned *something* — the same
      standard `003`'s own manual VS Code verification held itself to.
- [X] T033 [P] Update `README.md`'s Status section with `drut-mcp` (mirroring
      how `003`'s merge added `drut-lsp`/`editors/vscode` entries) and
      Dependency auditing section with `rmcp`/`tokio`/`schemars`'s RUSTSEC
      status (research.md §4) and the pinned versions actually used.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS every user
  story's end-to-end (through the real `drut` binary) testing, though US1/US2's
  own tool *logic* could technically be written against bare functions before
  T005–T007 land, if staffed in parallel.
- **User Story 1 (Phase 3)**: Depends on Foundational. No dependency on any
  other story — suggested MVP.
- **User Story 2 (Phase 4)**: Depends on Foundational. No dependency on US1's
  own tool *logic*.
- **User Story 3 (Phase 5)**: Depends on Foundational. Its own internal
  extraction (T018–T021) has no dependency on US1/US2 either. **This is the
  one story that refactors already-shipped code (`drut-lsp/src/hover.rs`) as
  part of its own work** — its extraction verification (T021) is a hard gate
  before T022's contract tests are considered meaningful.
- **User Story 4 (Phase 6)**: Depends on Foundational. No dependency on US1/
  US2/US3's own tool *logic* — genuinely the most independent story (no
  `ScriptSource` dependency at all, per its own resolved design question).
- **Polish (Phase 7)**: Depends on all four stories being complete (T029/T030
  need every tool to exist; T031 needs the whole workspace; T032/T033 are
  feature-complete-only concerns).

**Registration is serial across all four stories, corrected per
`/speckit-analyze` finding F2** — the original version of this section stated
only US3's registration task had a same-file dependency on a prior story's
registration; that was incomplete. In fact every story's registration task
edits the same `crates/drut-mcp/src/lib.rs`, forming one strict chain:
**T014 (US1) → T017 (US2) → T024 (US3) → T028 (US4)**. Each story's own
tool-logic/DTO/contract-test work (T010–T013, T015–T016, T018–T023,
T026–T027) is genuinely independent and parallelizable across stories — only
the final "add this tool to the registered list" step is serial, and only
because all four edits land in one shared file, not because of any real
logical dependency between the tools themselves.

### Within Each User Story

- Tests are written alongside (not strictly before, except where a story's
  own task ordering says so) implementation — contract tests (e.g. T010) are
  listed before their tool's implementation (T013) to keep the "write the
  test against the contract first" discipline visible in task order,
  matching `003`'s own convention.
- US1 specifically: T010 and T011 share `diagnose_contract.rs` — T011 is
  deliberately not marked `[P]` against T010 for that reason, even though
  both are `[US1]`.
- US3 specifically: extraction (T018) → extraction tests (T019) → `drut-lsp`
  refactor (T020) → **dedicated verification (T021, hard gate)** → tool
  contract tests (T022, own file) → tool implementation (T023) →
  registration (T024) → corpus parity (T025, folded into T021's own reported
  result).

### Parallel Opportunities

- T003/T004 (Setup) can run in parallel once T002 lands.
- T008/T009 (Foundational `ScriptSource`) can run in parallel with T005–T007
  (CLI wiring) — different files, no shared dependency.
- Once Foundational (Phase 2) completes, **every story's own tool-logic/DTO/
  contract-test work is genuinely parallel with every other story's**,
  per-tool contract test files now living in their own files (F1's fix):
  T010/T011 (`diagnose_contract.rs`) ∥ T015 (`format_contract.rs`) ∥
  T018–T022 (US3's extraction + `query_structure_contract.rs`) ∥ T026
  (`lookup_keyword_contract.rs`) can all proceed at the same time with zero
  file conflicts between stories. **Only each story's final registration
  task cannot** — T014, T017, T024, T028 are a strict serial chain through
  the shared `crates/drut-mcp/src/lib.rs` (corrected per F2, see
  "Dependencies & Execution Order" above) — plan to land registrations in
  story-priority order (US1 → US2 → US3 → US4) once each story's own
  tool-logic work is otherwise ready.
- T012 (US1 corpus parity) and T025 (US3 corpus/hover parity) are each their
  own file, parallel with everything else in their respective stories.

---

## Parallel Example: Foundational Phase

```bash
# Launch CLI wiring and ScriptSource work together (different files):
Task: "Add Command::Mcp to crates/drut-cli/src/cli.rs"
Task: "Create crates/drut-mcp/src/source.rs"
```

## Parallel Example: Once Foundational Completes

```bash
# Launch every story's own tool-logic/test work together — genuinely
# parallel now that each story's contract tests live in their own file:
Task: "US1: diagnose_contract.rs tests + diagnose.rs implementation"
Task: "US2: format_contract.rs tests + format.rs implementation"
Task: "US3: block_at extraction + query_structure_contract.rs tests + query_structure.rs implementation"
Task: "US4: lookup_keyword_contract.rs tests + lookup_keyword.rs implementation"

# Registration itself (T014 -> T017 -> T024 -> T028) is a separate, later,
# strictly serial pass through crates/drut-mcp/src/lib.rs once each story's
# own work above is ready — do not attempt these four in parallel.
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (`diagnose`)
4. **STOP and VALIDATE**: T010/T011/T012 pass independently; `drut mcp`
   advertises exactly one working tool through a real client (a reduced
   version of quickstart.md step 7, steps 1–3 only)
5. Demo/ship if ready — `diagnose` alone is a complete, independently useful
   capability

### Incremental Delivery

1. Setup + Foundational → foundation ready
2. Add US1 (`diagnose`) → test independently → demo (MVP!)
3. Add US2 (`format`) → test independently → demo
4. Add US3 (`query_structure`, including its own extraction-verification gate,
   T021) → test independently → demo
5. Add US4 (`lookup_keyword`) → test independently → demo
6. Polish (Phase 7) → feature-complete, ready for merge

---

## Notes

- [P] tasks = different files, no dependencies — verified per-task this
  revision (F1), not just asserted.
- [Story] label maps task to specific user story for traceability.
- US3's extraction work (T018–T021) is the one place in this feature where
  "done" means more than "new code plus new tests pass" — T021's three-part
  reported result is a hard gate, not a formality, per this session's own
  explicit requirement.
- Registration (T014/T017/T024/T028) is intentionally sequential — this is a
  real constraint from one shared file, not an oversight (F2's correction).
- Commit after each task or logical group.
- Stop at any checkpoint to validate a story independently before continuing.
- Avoid: vague tasks, same-file conflicts within a phase (checked this
  revision per F1), cross-story dependencies that would break independent
  testability (US3's dependency is on `voyager-core`/`drut-lsp`, never on
  US1/US2/US4's own tool logic).
