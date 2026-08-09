# Tasks: Voyager Script Tokenizer & Structural Parser

**Input**: Design documents from `/specs/001-voyager-script-parser/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md (all present)

**Regenerated 2026-08-08** following a documentation verification pass that amended
FR-005, FR-006, FR-007, FR-009, FR-022, FR-026, FR-028 and added FR-033 — enough of
the block-matching and continuation algorithm changed (implicit block closing,
short-`IF`, brace-delimited continuation, nested block comments, a narrowed
`MisplacedBreak` condition) that this file is a full rewrite, not a patch of the
previous version. It also closes a pre-existing gap: FR-028–FR-030 (`PROCESS`/
`PHASE`, `JLOOP`, `DistributeMULTISTEP`) were already in spec.md before this pass but
were never reflected in the previous tasks.md at all — they're fully tasked here for
the first time, alongside the newly-promoted FR-033 (`LINKLOOP`).

**Tests**: This feature's own Definition of Done (spec.md) and constitution Principle V
require a passing fixture-corpus test gate before any later phase (CLI/LSP/MCP/
formatter) may begin. Test tasks are therefore included as core deliverables in every
user-story phase below, not as an optional add-on.

**Organization**: Tasks are grouped by user story (spec.md priorities P1/P2/P3) so each
story is independently implementable and testable, per plan.md's single-crate
structure (`crates/voyager-core`).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, or the same file but an independent
  concern with no dependency on an incomplete task)
- **[Story]**: US1, US2, or US3 — maps to spec.md's three priority-ordered user stories
- Every task lists an exact file path under `crates/voyager-core/`

## Path Conventions

Single Rust library crate inside a new Cargo workspace (plan.md Project Structure):

```text
Cargo.toml                       # workspace manifest
crates/voyager-core/
├── Cargo.toml
├── src/{lib,span,token,lexer,statement,block,diagnostic,grammar_notes}.rs
└── tests/{fixtures/{valid,broken}/, fixture_corpus.rs}
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Workspace and crate scaffolding — nothing grammar-specific yet.

- [X] T001 Create workspace manifest `Cargo.toml` at repo root with
  `members = ["crates/voyager-core"]`, leaving room for future `cli/`, `lsp/`, `mcp/`,
  `formatter/` members per plan.md Project Structure
- [X] T002 Initialize `crates/voyager-core/Cargo.toml` (name = `voyager-core`, edition
  2021, **no runtime dependencies** per FR-027) and an empty `crates/voyager-core/
  src/lib.rs` that compiles
- [X] T003 [P] Add `rustfmt.toml` and a `clippy.toml` (or documented `cargo clippy`
  invocation) at repo root so formatting/lint conventions are fixed before any grammar
  code lands
- [X] T004 [P] Create `crates/voyager-core/tests/fixtures/valid/` and
  `crates/voyager-core/tests/fixtures/broken/` with a `README.md` in each recording the
  sourcing/licensing status from research.md §3 (no real third-party script content
  until rights are confirmed)

**Checkpoint**: Workspace builds (`cargo build`); no grammar logic exists yet.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The shared token/diagnostic/lexer layer every user story (US1: structure,
US2: diagnostics, US3: token detail) builds on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T005 [P] Define `Span`/`Position` (line/column source locations) in
  `crates/voyager-core/src/span.rs` per data-model.md § Span
- [X] T006 [P] Define `Diagnostic` and `DiagnosticKind` (all six categories from
  contracts/diagnostics.md: `UnmatchedIf`, `UnmatchedLoop`, `UnclosedBlockComment`,
  `InvalidContinuation`, `UnmatchedRun`, `MisplacedBreak`) in
  `crates/voyager-core/src/diagnostic.rs` per data-model.md § Diagnostic (FR-017) —
  the *kinds* are unchanged from before this pass; only their triggering conditions
  (implemented in Phase 4) changed
- [X] T007 [P] Define `Token` and `TokenKind` (`Word`, `LineComment`, `BlockComment`,
  `ContinuationMarker`, `VariableRef`, `Punctuation`) in
  `crates/voyager-core/src/token.rs` per data-model.md § Token (FR-002, FR-010) — the
  `BlockComment` variant must be able to represent nesting (FR-005), and `Punctuation`
  must cover `{`/`}` (FR-006)
- [X] T008 Implement the char-level lexer in `crates/voyager-core/src/lexer.rs`:
  line-comment recognition (FR-004), block-comment recognition including
  multi-line **and nested** spans (FR-005 — a `/*` while one is already open starts
  its own inner comment; the outer one isn't done until every inner one closes),
  continuation-character detection based on the last non-comment character
  including skipping fully blank lines before the resuming line (FR-006), and
  `@variable@` tokenization (FR-010) — depends on T005, T007
- [X] T009 Wire the public `tokenize(source: &str) -> Vec<Token>` entry point in
  `crates/voyager-core/src/lib.rs` per contracts/public-api.md — depends on T007, T008
  (not parallelizable with them: it calls into both)
- [X] T010 [P] Create `crates/voyager-core/src/grammar_notes.rs` with the module
  scaffold and initial entries (Voyager 6.5 baseline, original wording) for the
  tokenizer-level rules being established this phase: comments including nesting
  (FR-004, FR-005), continuation including blank-line skipping (FR-006),
  case-insensitivity (FR-011) — FR-024, constitution Principle II. Pure documentation
  with no compile-time dependency on T008, so it stays `[P]` unlike T009.
- [X] T011 Create the fixture-corpus test harness scaffold in
  `crates/voyager-core/tests/fixture_corpus.rs`: walks `tests/fixtures/valid/**` and
  `tests/fixtures/broken/**`, compiles and runs cleanly against the (currently empty)
  fixture directories from T004 — no real assertions yet, those land per-story below

**Checkpoint**: `tokenize()` works end-to-end on comments (including nested block
comments)/continuation (including blank-line skipping)/`@variable@`; no statement/
block structure or diagnostics exist yet. Foundation ready for all three user stories.

---

## Phase 3: User Story 1 - Parse a valid script into structure (Priority: P1) 🎯 MVP

**Goal**: A caller hands `parse()` the text of a valid `.s`/`.block` script and gets
back a full statement/block structure with zero diagnostics — no file I/O needed to
exercise it.

**Independent Test**: Feed a known-good `.s` fixture (multi-line statements, nested
IF/LOOP/RUN blocks, comments, `@variable@` references) directly to `parse()` and
confirm a complete structure with an empty diagnostics list (spec.md User Story 1).

- [X] T012 [P] [US1] Define `Statement` and `StatementKind` (`Control`, `Assignment`,
  `Label`, `ShellEscape`) in `crates/voyager-core/src/statement.rs` per data-model.md §
  Statement (FR-003, FR-021, FR-022, FR-023) — `ShellEscape` stores arbitrary command
  text (parenthesized or not), not specifically a parenthesized command (FR-022
  generalized)
- [X] T013 [P] [US1] Define `Block` and `BlockKind` — `If` (incl. the self-closing
  short-`IF` shape, FR-007), `Loop`, `Run` (with a `disabled` flag for `!RUN`,
  FR-009), `Process` (the `PROCESS`/`PHASE` block, FR-028), `JLoop` (FR-029),
  `LinkLoop` (FR-033), `DistributeMultistep` (FR-030) — in
  `crates/voyager-core/src/block.rs` per data-model.md § Block
- [X] T014 [US1] Implement the statement-building pass in
  `crates/voyager-core/src/statement.rs`: join continuation-joined tokens into one
  `Statement` via either the trailing-operator mechanism or the `{...}`-delimited
  mechanism (FR-006), classify each as `Control`/`Assignment`/`Label`/`ShellEscape`,
  matching control words/keywords case-insensitively (FR-003, FR-006, FR-011,
  FR-021–FR-023) — depends on T012, T008
- [X] T015 [US1] Implement the core block-matching pass (`If`-chain incl. short-`IF`,
  `Loop`, `Run` incl. implicit closing and the `!RUN`/`ENDRUN`-required exception) in
  `crates/voyager-core/src/block.rs`, with zero-or-more top-level blocks (no
  mandatory wrapper) per FR-020 — depends on T013, T014
- [X] T016 [P] [US1] Implement `Process`/`PHASE` block matching in
  `crates/voyager-core/src/block.rs`: recognize `PROCESS PHASE=...` and the bare
  `PHASE=...` trigger-keyword shortcut as equivalent openers, `ENDPROCESS`/`ENDPHASE`
  as equivalent closers, and implicit closing by the next `PROCESS`/`PHASE=`
  statement (FR-028) — depends on T015
- [X] T017 [P] [US1] Implement `JLOOP`/`ENDJLOOP` block matching in
  `crates/voyager-core/src/block.rs`, including the no-nested-`JLOOP` restriction
  (FR-029) — depends on T015
- [X] T018 [P] [US1] Implement `LINKLOOP`/`ENDLINKLOOP` block matching in
  `crates/voyager-core/src/block.rs`, including the no-nested-`LINKLOOP` restriction
  (FR-033) — depends on T015
- [X] T019 [P] [US1] Implement `DistributeMULTISTEP`/`EndDistributeMULTISTEP` block
  matching in `crates/voyager-core/src/block.rs` (FR-030; sequential, non-nesting,
  no special nesting-restriction logic needed beyond ordinary matching) — depends on
  T015
- [X] T020 [US1] Wire the public `parse(source: &str) -> ParseResult` entry point in
  `crates/voyager-core/src/lib.rs`, composing lexer → statement-building → block-
  matching (all seven `BlockKind` variants) per contracts/public-api.md and
  data-model.md § ParseResult — depends on T014, T015, T016, T017, T018, T019
- [X] T021 [P] [US1] Unit test case-insensitive control-word/keyword matching (`IF`,
  `If`, `if`, plus the newer multi-word/synonym pairs — `PROCESS`/`PHASE`,
  `ENDPROCESS`/`ENDPHASE`, `RUN`/`!RUN` — per FR-011) in
  `crates/voyager-core/src/statement.rs`'s test module
- [X] T022 [P] [US1] Unit test that a statement spanning multiple physical lines via
  trailing continuation characters (`,` `+` `-` `/` `*` `^` `&` `|` `=`) produces one
  logical `Statement`, not several (FR-006), including one case where a blank line
  sits between the continuation-ending line and the resuming line, in
  `crates/voyager-core/src/statement.rs`'s test module
- [X] T023 [P] [US1] Unit test that a `Control` statement continued with `{...}`
  (rather than trailing-operator characters) produces one logical `Statement`
  spanning to the closing `}`, with no continuation character required on any
  interior line (FR-006) in `crates/voyager-core/src/statement.rs`'s test module
- [X] T024 [P] [US1] Unit test that an `IF (...)` statement followed on the same
  line by exactly one further statement produces a complete `If` block with no
  `ENDIF` expected (short-`IF`, FR-007), and that a statement trailing `ELSEIF`/
  `ELSE`/`ENDIF` on the same line is parsed as its own separate statement, not
  folded into the block, in `crates/voyager-core/src/block.rs`'s test module
- [X] T025 [P] [US1] Add grammar-note entries for block/statement rules (FR-003,
  FR-007 incl. short-`IF`, FR-008, FR-009 incl. implicit closing and `!RUN`, FR-020,
  FR-021, FR-022 generalized, FR-023, FR-028, FR-029, FR-030, FR-033) to
  `crates/voyager-core/src/grammar_notes.rs` — FR-024
- [X] T026 [US1] Author initial hand-written "valid" fixtures in
  `crates/voyager-core/tests/fixtures/valid/` — structural-shape fixtures per
  research.md §3, not verbatim third-party script content — covering:
  - nested `IF`/`LOOP`/`RUN` blocks, mixed-case control words, and multi-line
    continuation (both the trailing-operator and `{...}` mechanisms);
  - each of the label/shell-escape/assignment statement forms, including at least
    one occurrence placed immediately before or after an `IF`/`LOOP`/`RUN` block or
    between `ELSEIF` branches (spec.md Edge Cases), and a bare (no-parentheses)
    shell-escape alongside a parenthesized one;
  - one **bare-fragment**-shaped `.block` fixture (no top-level `RUN`/`ENDRUN`) and
    one **self-contained**, `RUN PGM=.../ENDRUN`-wrapped `.block` fixture, per SC-005;
  - an empty script and a script containing only comments/whitespace (spec.md Edge
    Cases);
  - a short-`IF` (single trailing statement, no `ENDIF`) alongside an ordinary
    block-form `IF` in the same file;
  - a `RUN` block with no explicit `ENDRUN`, closed implicitly by the next `RUN`
    statement, and a separate `RUN` closed implicitly by a shell-escape statement;
  - a `!RUN` block with its required explicit `ENDRUN` present;
  - a `PROCESS PHASE=...`/`ENDPROCESS` pair and a bare `PHASE=.../ENDPHASE` pair in
    the same file, plus one `PHASE=` block closed implicitly by the next `PHASE=`
    statement with no `ENDPHASE` in between;
  - a `JLOOP...ENDJLOOP` block nested inside `IF`, a `LINKLOOP...ENDLINKLOOP` block
    nested inside `LOOP`, and a `DistributeMULTISTEP...EndDistributeMULTISTEP`
    sequential (non-nested) pair;
  - a nested block comment (`/* ... /* ... */ ... */`);
  - a `BREAK` nested only inside a bare `IF` (no enclosing `LOOP`/`RUN`/`PROCESS`)
    and a `BREAK` nested inside a `PROCESS`/`PHASE` stack — both must produce zero
    diagnostics under the narrowed FR-026
- [X] T027 [US1] Wire "valid" fixture-corpus assertions into
  `crates/voyager-core/tests/fixture_corpus.rs`: every fixture under `tests/fixtures/
  valid/` parses via `parse()` with an empty diagnostics list (SC-001) — depends on
  T020, T026

**Checkpoint**: User Story 1 is fully functional and independently testable — this is
the MVP.

---

## Phase 4: User Story 2 - Get precise diagnostics for a broken script (Priority: P2)

**Goal**: A caller feeds `parse()` a script with a real defect (missing `ENDIF`,
unclosed comment, dangling continuation, etc.) and gets back a structured diagnostic
naming the problem and its location — never a panic, never a silent wrong answer.

**Independent Test**: Feed each deliberately-broken fixture (one per diagnostic
category) to `parse()` and confirm each produces a diagnostic correctly naming that
category, with no panic (spec.md User Story 2).

- [X] T028 [P] [US2] Implement the unclosed-block-comment diagnostic (`/*` with no
  matching `*/` before end of input, correctly anchored at whichever `/*` — outer or
  an inner nested one — never found its match) in
  `crates/voyager-core/src/lexer.rs` (FR-014)
- [X] T029 [P] [US2] Implement the invalid/missing-continuation diagnostic (a
  continuation character with no following line, or an invalid following line — not
  counting fully blank lines in between as invalid) in
  `crates/voyager-core/src/lexer.rs` (FR-015)
- [X] T030 [US2] Implement the unmatched `IF`/`ENDIF` diagnostic, including the
  dangling-closer case (an `ENDIF`/`ELSEIF`/`ELSE` with no open `IF` — including an
  `ENDIF` that follows an already-self-closed short-`IF`), in
  `crates/voyager-core/src/block.rs` (FR-012) — depends on T015
- [X] T031 [US2] Implement the unmatched `LOOP`/`ENDLOOP` diagnostic, including the
  dangling-closer case, in `crates/voyager-core/src/block.rs` (FR-013) — depends on
  T015
- [X] T032 [US2] Implement the unmatched `RUN`/`ENDRUN` diagnostic in
  `crates/voyager-core/src/block.rs` (FR-016): fires only when a non-`disabled` `RUN`
  has neither an explicit `ENDRUN` nor an implicit closer (next `RUN`/`!RUN` or a
  shell-escape statement); a `disabled` (`!RUN`) block is diagnosed on a missing
  `ENDRUN` alone, with no implicit-closer exception; the dangling-closer case (an
  `ENDRUN` with no open `RUN`/`!RUN`) still applies — depends on T015, T016 (needs
  `Process`/`PHASE` matching in place so a `PHASE=` statement isn't mistaken for a
  `RUN`-closing event)
- [X] T033 [US2] Implement the misplaced-`BREAK` diagnostic in
  `crates/voyager-core/src/block.rs` (FR-026): fires only when `BREAK` has no
  enclosing block of *any* `BlockKind` (`If`, `Loop`, `Run`, `Process`, `JLoop`,
  `LinkLoop`) — not "outside `LOOP`" specifically — depends on T015, T016, T017, T018
- [X] T034 [US2] Ensure `parse()` continues past each recorded defect and keeps
  reporting on the remainder of the script rather than aborting, across
  `crates/voyager-core/src/lexer.rs` and `crates/voyager-core/src/block.rs` (FR-018) —
  depends on T028–T033
- [X] T035 [US2] Audit `crates/voyager-core/src/lib.rs`, `lexer.rs`, `statement.rs`,
  and `block.rs` to eliminate any `unwrap`/`panic!`/unhandled `Result::Err` reachable
  from the public API on malformed input (plan.md Constraints) — depends on T028–T033
- [X] T036 [P] [US2] Add grammar-note entries for all six diagnostic rules (FR-012
  incl. short-`IF` dangling closers, FR-013, FR-014 incl. nesting, FR-015 incl.
  blank-line handling, FR-016 incl. implicit closing and `!RUN`, FR-026 narrowed) to
  `crates/voyager-core/src/grammar_notes.rs` — FR-024
- [X] T037 [US2] Author one deliberately-broken fixture per diagnostic category (six
  total) in `crates/voyager-core/tests/fixtures/broken/` (FR-025), including:
  - an `IF` with no `ENDIF` and, separately, a stray `ENDIF` after a short-`IF` has
    already self-closed (both are `UnmatchedIf`);
  - a `RUN` block genuinely left open — no explicit `ENDRUN`, no following `RUN`,
    no following shell-escape, straight to end-of-file — as well as a `!RUN` block
    missing its required explicit `ENDRUN` (both are `UnmatchedRun`);
  - a `BREAK` at true top level with no enclosing block of any kind
    (`MisplacedBreak`) — distinct from the *valid* bare-`IF`-only and
    `PROCESS`/`PHASE`-nested `BREAK` fixtures already covered under US1 (T026),
    which must NOT trigger this diagnostic
- [X] T038 [US2] Wire "broken" fixture-corpus assertions into
  `crates/voyager-core/tests/fixture_corpus.rs`: every fixture under `tests/fixtures/
  broken/` produces a diagnostic matching its injected defect category, with no panic
  (SC-002, SC-003) — depends on T028–T033, T037
- [X] T039 [US2] Author a fixture with at least two independent, simultaneous defects
  (e.g. an unclosed block comment *and* an unmatched `IF` in the same file) in
  `crates/voyager-core/tests/fixtures/broken/`, and wire an assertion into
  `crates/voyager-core/tests/fixture_corpus.rs` that a single `parse()` call returns
  both correctly-categorized diagnostics — proving FR-018's continue-past-defect
  guarantee (spec.md Edge Cases: "both diagnostics are expected, not just one") —
  depends on T034, T038

**Checkpoint**: User Stories 1 and 2 both work independently — structure and
diagnostics are both correct against the fixture corpus.

---

## Phase 5: User Story 3 - Track token-level detail for editor-style features (Priority: P3)

**Goal**: A caller uses `tokenize()` alone (no `parse()` needed) to get comments,
`@variable@` references, and continuation markers as distinct, position-tracked
tokens, for future editor-facing features.

**Independent Test**: Feed a script with a trailing line comment, a multi-line block
comment, and a continuation-split `@variable@` reference to `tokenize()` and confirm
each is its own correctly-positioned token (spec.md User Story 3).

- [X] T040 [P] [US3] Unit test that a line comment following real statement content is
  tokenized separately and does not affect continuation detection (FR-004, FR-006) in
  `crates/voyager-core/src/lexer.rs`'s test module
- [X] T041 [P] [US3] Unit test that a multi-line `/* ... */` block comment tokenizes
  as a single token spanning its start and end positions, and that a nested `/* ...
  /* ... */ ... */` produces an inner `BlockComment` token whose span sits fully
  inside the outer one (FR-005) in `crates/voyager-core/src/lexer.rs`'s test module
- [X] T042 [P] [US3] Unit test that `@variable@` tokenizes with its captured name and
  position, with no evaluation/substitution (FR-010) in
  `crates/voyager-core/src/token.rs`'s test module
- [X] T043 [US3] Author a fixture combining a trailing line comment, a multi-line
  (and, separately, a nested) block comment, and a continuation-split `@variable@`
  reference in `crates/voyager-core/tests/fixtures/valid/` (reused for SC-001
  coverage too)
- [X] T044 [US3] Wire token-detail assertions into `crates/voyager-core/tests/
  fixture_corpus.rs` per quickstart.md Scenario 3 — depends on T009 (foundational
  `tokenize()`), T040–T043

**Checkpoint**: All three user stories are independently functional and covered by the
fixture-corpus test gate.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates and follow-up items that span all three stories.

- [X] T045 [P] Write rustdoc for the public API (`tokenize`, `parse`) in
  `crates/voyager-core/src/lib.rs` restating the no-panic, determinism, and
  case-insensitivity guarantees from contracts/public-api.md
- [X] T046 [P] Add an example binary at `crates/voyager-core/examples/parse_file.rs`
  that reads a path via `std::fs`, calls `parse()`, and prints the resulting nodes/
  diagnostics — the library itself still performs no I/O (quickstart.md manual
  spot-check)
- [X] T047 [P] Run a clean `cargo clippy -p voyager-core` pass (zero warnings) across
  all `src/` files
- [X] T048 Run `cargo test -p voyager-core` end-to-end and record results against every
  quickstart.md scenario and the Definition of Done (constitution Principle V gate
  before any later phase begins)
- [X] T049 Confirm or resolve the fixture-corpus sourcing/licensing open item
  (research.md §3) before treating `tests/fixtures/` as the final corpus — replace any
  hand-written placeholder fixtures with the real, rights-cleared corpus once available
  — **Resolved 2026-08-09**: a 9-file representative subset (~5,200 lines) from
  `WF-TDM-Official-Releases` copied into `tests/fixtures/valid/real_corpus/`, checked
  for sensitive content (none found), and validated against `parse_bytes()` — see
  T051's real-file discovery below for how this subset directly motivated FR-034.
- [X] T050 [P] Audit `crates/voyager-core/src/grammar_notes.rs` for completeness
  against the full current FR list (FR-003 through FR-033, including every rule
  amended or added by the 2026-08-08 documentation verification pass) — confirm every
  entry is original wording, not copied from vendor documentation (constitution
  Principle II, FR-024, SC-006)

---

## Phase 7: FR-034 — Byte-oriented decoding with Windows-1252 fallback

**Purpose**: New work discovered during T049 — one file in the real fixture corpus
(`4pd_mainbody_distribution.block`) contains a single Windows-1252 byte that isn't
valid UTF-8, which `fs::read_to_string`/`parse(&str)` cannot even load. FR-034 (spec.md)
adds byte-oriented sibling entry points that decode UTF-8-first with a per-byte
Windows-1252 fallback, rather than requiring every caller to solve this itself.

- [X] T051 Implement `crates/voyager-core/src/decode.rs`: the Windows-1252 lookup
  table (`0x80`-`0x9F`; `0xA0`-`0xFF` matches Latin-1 identically), and
  `decode_bytes(&[u8]) -> (String, Vec<Diagnostic>)` — surgical per-byte fallback via
  `str::from_utf8`'s `valid_up_to()`, not a whole-file encoding guess, so valid UTF-8
  elsewhere in the same file (including legitimate non-ASCII content) is untouched.
  Position tracking reuses the same running `char`-count `Position::advance` the
  lexer already uses, so `InvalidEncoding`'s span doesn't introduce a second,
  inconsistent position scheme (data-model.md § Span) — depends on T005 (`Span`),
  T006 (`Diagnostic`)
- [X] T052 Add `DiagnosticKind::InvalidEncoding` to
  `crates/voyager-core/src/diagnostic.rs` (FR-034; contracts/diagnostics.md) —
  depends on T006
- [X] T053 Wire `tokenize_bytes`/`parse_bytes` into `crates/voyager-core/src/lib.rs`
  per the amended contracts/public-api.md — depends on T051, T052, T009, T020
- [X] T054 Add a hand-crafted `broken/undecodable_byte.s` fixture (raw byte `0x81`,
  one of the five Windows-1252-undefined code points, inside a comment) to exercise
  the `InvalidEncoding` diagnostic path — the one real non-UTF-8 file found (T049)
  resolves silently under Windows-1252 and so cannot exercise this branch itself —
  depends on T053
- [X] T055 Migrate `crates/voyager-core/tests/fixture_corpus.rs` from
  `fs::read_to_string`/`parse` to `fs::read`/`parse_bytes` uniformly (pure-UTF-8
  fixtures are unaffected; the real corpus's one non-UTF-8 file becomes loadable) and
  add an assertion that it decodes with zero diagnostics — depends on T051, T053, T054
- [X] T056 [P] Add the FR-034 grammar-note entry to
  `crates/voyager-core/src/grammar_notes.rs` and extend T050's completeness audit to
  FR-034 — depends on T051

---

## Phase 8: FR-023 fix — subscripted assignment targets

**Purpose**: A full-corpus validation run (research.md §3) found that
`classify_statement` misclassified any assignment target carrying a bracketed
subscript (`MW[1] = ...`, 6,000+ real occurrences in one file alone) as a `Control`
statement instead of `Assignment` — SC-001-invisible (no diagnostic fires either
way) but a real `StatementKind` defect a future formatter would consume directly.

- [X] T057 Amend FR-023 (spec.md) to state an assignment target MAY carry one or
  more trailing bracketed subscripts, and update data-model.md's `Assignment` entity
  and spec.md's Key Entities/Edge Cases sections to match — depends on nothing new
- [X] T058 Add `assignment_equals_index` to `crates/voyager-core/src/statement.rs`
  and use it in `classify_statement` in place of the old "immediately followed by
  `=`" check — handles zero or more `[...]` subscript groups (including
  double-subscript `SUBAREAID[Seg_Idx][idx_SUBAREAID]`), never panics on unbalanced
  brackets (falls back to `Control`) — depends on T057
- [X] T059 [P] Add `statement.rs` unit tests (single-subscript, double-subscript,
  unsubscripted-still-works, unbalanced-bracket-safety, ordinary-Control-unaffected)
  and a `tests/fixtures/valid/subscripted_assignment_targets.s` fixture using the
  real `MW[1] = ...`/double-subscript shapes, plus a `fixture_corpus.rs` test
  asserting `StatementKind` directly (a zero-diagnostics check alone would not catch
  this class of bug) — depends on T058

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories.
- **User Stories (Phase 3–5)**: All depend on Foundational phase completion.
  - US1 has no dependency on US2 or US3.
  - US2 depends on US1's block-matching structure (T015, and — for `UnmatchedRun`/
    `MisplacedBreak` specifically — T016–T019 too) to attach diagnostics to — it is
    not independent of US1's *code*, but is independently testable once US1 exists
    (US2's tests exercise defect scripts specifically).
  - US3 depends only on the Foundational `tokenize()` (T009), not on US1/US2 at all —
    it can, in principle, be built in parallel with US1/US2 by a different developer.
- **Polish (Phase 6)**: Depends on all three user stories being complete.
- **FR-034 (Phase 7)**: Depends on Foundational (T005 `Span`, T006 `Diagnostic`) and
  US1's `tokenize`/`parse` entry points (T009, T020) to wrap — not on US2/US3.
  Discovered during T049 (real fixture corpus), after Phase 6 had already landed, so
  it sits after Polish in file order despite its actual dependency being much
  earlier; nothing in Phase 6 depends on it.
- **FR-023 fix (Phase 8)**: Depends only on US1's statement-building code (T012,
  T014) already existing to amend — not on Phase 7/FR-034 at all, despite sitting
  after it in file order (discovered later, by a separate full-corpus validation
  pass). Nothing depends on it either.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2). No dependency on
  US2/US3.
- **User Story 2 (P2)**: Can start after Foundational (Phase 2), but its diagnostic
  logic attaches to US1's block-matching code (T015–T019) — implement after or
  alongside US1, not before.
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) alone — independent of
  US1 and US2's code, since it only needs `tokenize()`.

### Within Each User Story

- Types before passes (e.g. T012/T013 before T014/T015).
- The core block-matching pass (T015: `If`/`Loop`/`Run`) before the four
  block-family extensions that build on the same matching machinery (T016
  `Process`, T017 `JLoop`, T018 `LinkLoop`, T019 `DistributeMultistep`) — those four
  are independent of each other and can run in parallel once T015 lands.
- Passes before the public entry-point wiring (e.g. T014–T019 before T020).
- Implementation before its own unit tests and fixture-corpus wiring.
- Grammar-note entries (FR-024) land alongside the rules they document, not deferred
  to Polish.
- Story complete (including its fixture-corpus assertions) before moving to the next
  priority.

### Parallel Opportunities

- All Setup tasks marked [P] (T003, T004) can run in parallel once T001/T002 exist.
- Foundational tasks T005, T006, T007 (independent type definitions) can run in
  parallel; T008 needs T005 and T007 first.
- Once Foundational completes, **US3 can be worked entirely in parallel with US1/US2**
  by a different developer, since it depends only on T009.
- Within US1: T012 and T013 in parallel; once T015 lands, T016/T017/T018/T019 (the
  four extension block-families) in parallel with each other; T021/T022/T023/T024
  (four independent unit tests) in parallel once their subjects exist.
- Within US2: T028 and T029 in parallel; T030/T031/T032/T033 are independent
  diagnostic categories that could be split across developers even though they share
  `block.rs` (coordinate on merge, not blocked on each other's logic) — note T032
  and T033 additionally depend on T016–T019 landing first.
- Within US3: T040, T041, T042 (three independent unit tests) in parallel.

---

## Parallel Example: User Story 1

```bash
# Launch the two independent type-definition tasks together:
Task: "Define Statement and StatementKind in crates/voyager-core/src/statement.rs"
Task: "Define Block and BlockKind in crates/voyager-core/src/block.rs"

# Once the core If/Loop/Run matching pass (T015) exists, launch the four
# independent block-family extensions together:
Task: "Implement Process/PHASE block matching in block.rs"
Task: "Implement JLOOP/ENDJLOOP block matching in block.rs"
Task: "Implement LINKLOOP/ENDLINKLOOP block matching in block.rs"
Task: "Implement DistributeMULTISTEP/EndDistributeMULTISTEP block matching in block.rs"

# Later, once parse() exists, launch the four independent unit tests together:
Task: "Unit test case-insensitive control-word/keyword matching in statement.rs"
Task: "Unit test multi-line continuation (trailing-operator + blank-line-skip) in statement.rs"
Task: "Unit test brace-delimited {...} continuation in statement.rs"
Task: "Unit test short-IF self-closing and ELSEIF/ELSE/ENDIF trailing-statement handling in block.rs"
```

## Parallel Example: User Story 3 (fully parallel with US1/US2)

```bash
# All three token-level unit tests are independent of each other and of US1/US2:
Task: "Unit test trailing line comment tokenized separately in lexer.rs"
Task: "Unit test multi-line and nested block comment tokens in lexer.rs"
Task: "Unit test @variable@ token with name+position in token.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories).
3. Complete Phase 3: User Story 1.
4. **STOP and VALIDATE**: run `cargo test -p voyager-core` and confirm SC-001 holds
   against the fixtures authored in T026.
5. This is the MVP — a caller can already get correct structure from valid scripts,
   across all seven block kinds (`If`, `Loop`, `Run`, `Process`, `JLoop`, `LinkLoop`,
   `DistributeMultistep`), not just the original three.

### Incremental Delivery

1. Setup + Foundational → tokenizer works (incl. nested comments, blank-line-skip
   continuation), no structure/diagnostics yet.
2. Add User Story 1 → structure is correct on valid input, across all seven block
   kinds and both continuation mechanisms → MVP.
3. Add User Story 2 → diagnostics are correct on broken input, including the
   narrowed `MisplacedBreak` condition and the implicit-close-aware `UnmatchedRun`.
4. Add User Story 3 → token-level detail is exposed for future editor features.
5. Polish → docs, example, clippy, full test run, fixture-corpus sourcing follow-up,
   grammar-notes completeness audit.

### Parallel Team Strategy

With multiple developers, after Foundational completes:

- Developer A: User Story 1 (structure — including splitting the four block-family
  extensions T016–T019 across sub-tasks once T015 lands), then User Story 2
  (diagnostics — depends on US1's block-matching code).
- Developer B: User Story 3 (token detail) — fully independent, can proceed in
  parallel with A the whole time.

---

## Notes

- [P] tasks touch different files (or, within `block.rs`, different block families or
  different diagnostic categories) with no dependency on an incomplete task at the
  same tier.
- [Story] labels map every user-story-phase task back to spec.md's P1/P2/P3 stories
  for traceability.
- Grammar-note entries (FR-024, constitution Principle II) are written in the
  project's own words at the same time as the rule they document — never copied from
  vendor documentation. This matters more than usual this round: several rules
  (implicit block closing, short-`IF`, block-comment nesting, the `{...}`
  continuation form) were sourced from a vendor-documentation cross-check rather
  than fixtures alone, so it would be easy to accidentally echo the source's own
  phrasing — don't.
- The real fixture corpus is still an open dependency (research.md §3, T004, T049) —
  T026/T037/T039/T043 use hand-written, structurally-representative fixtures in the
  meantime, not verbatim third-party script content.
- Several constructs added or changed this pass — short-`IF`, block-comment nesting,
  blank-line-skip continuation, `{...}` continuation, `RUN`/`PROCESS` implicit
  closing — are confirmed by vendor documentation but have **no confirmed fixture
  example yet** (spec.md Assumptions). T026/T037's fixtures are hand-written to
  exercise them for now; if/when the real fixture corpus (T049) lands, check it for
  natural examples of each and prefer those.
- Commit after each task or logical group; stop at any checkpoint to validate a story
  independently before continuing.
