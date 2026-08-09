---

description: "Task list for Drut CLI — check and format subcommands"
---

# Tasks: Drut CLI — `check` and `format` Subcommands

**Input**: Design documents from `/specs/002-cli-check-format/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md (all present)

**Tests**: Included — the feature's own Definition of Done and constitution
Principle III/IV require a golden-file suite, full-corpus SARIF schema validation,
and exit-code coverage before merge; these aren't optional flavor for this feature.

**Organization**: Tasks are grouped by user story (US1 = `check`, P1; US2 =
`format`, P2) per spec.md, so each can be implemented, tested, and shipped
independently. `check` needs zero changes to `voyager-core` (it already exposes
`parse_bytes`); `format` requires new `voyager-core` entry points first, which is
why US2 has more tasks than US1.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: US1 or US2 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

Two crates, both under `crates/` (plan.md Structure Decision):

- `crates/voyager-core/` — existing crate; US2 adds `src/format.rs` and
  `tests/format_corpus.rs`/`tests/fixtures/golden/` to it. US1 touches nothing here.
- `crates/drut-cli/` — new crate this feature creates; package `drut-cli`,
  binary `drut`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Stand up the new `drut-cli` crate so it builds in the workspace.

- [x] T001 Add `"crates/drut-cli"` as a second workspace member in `Cargo.toml`
      (repo root), and update its "Future members: cli/, lsp/, mcp/, formatter/"
      comment to reflect the actual `crates/drut-cli` path now that it exists
      (plan.md Structure Decision).
- [x] T002 Create `crates/drut-cli/Cargo.toml`: package `drut-cli`, `[[bin]] name =
      "drut"`, a path dependency on `voyager-core`, and the pinned dependencies —
      `clap = "4.6.6"` (derive feature), `ignore = "0.4.33"`,
      `serde = "1.0.229"` (derive feature), `serde_json = "1.0.151"`,
      `similar = "3.1.2"`; dev-dependency `jsonschema = "0.49.8"`. **No
      `serde-sarif`** despite research.md §6's original pin — its build script hit
      an Application Control block on this machine; superseded in research.md §4
      by hand-written SARIF structs over plain `serde_json` (see T012).
- [x] T003 [P] Create `crates/drut-cli/src/main.rs` with a placeholder `fn main()
      {}` and confirm `cargo build -p drut-cli` and `cargo clippy -p drut-cli`
      succeed zero-warning (CLAUDE.md's zero-warning clippy gate, extended to this
      new crate).

**Checkpoint**: `cargo build`/`cargo clippy` pass across the whole workspace with
the new (empty) crate in place.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Traversal, exit-code, and CLI-parsing scaffolding shared by both
`check` and `format` (spec.md FR-001–FR-005 are explicitly scoped as "shared by
`check` and `format`").

**⚠️ CRITICAL**: No user-story work can begin until this phase is complete.

- [x] T004 [P] Define `MatchedFile`, `ReadFailure`, and `TraversalOutcome` in
      `crates/drut-cli/src/traverse.rs` (data-model.md §3).
- [x] T005 Implement the shared traversal function in
      `crates/drut-cli/src/traverse.rs`: walk a file-or-directory path via the
      `ignore` crate honoring `.gitignore` including nested `.gitignore` files
      (FR-002), filter to `.s`/`.block` extensions case-insensitively while
      silently skipping every other extension including `.mat`/`.net`/`.dbd`/`.prj`
      (FR-003), read each matched file's raw bytes, and populate
      `TraversalOutcome` — `invalid_target` when the given path doesn't exist or is
      neither a file nor directory (FR-004), a `ReadFailure` per file that matches
      the extension filter but can't be read, without aborting the rest of the walk
      (FR-005). Depends on T004. **Note**: `ignore`'s `require_git` defaults to
      `true` — `.gitignore` only applies inside an actual repo (a bare `.git` dir
      is enough), matching real `git`'s own behavior; left at default per FR-002's
      "the same way `git` itself would decide" wording (research.md §3).
- [x] T006 [P] Define the shared `ExitOutcome` type and its exit-code mapping
      (`Clean`→0, the code-1 variant→1, `Fatal`→2, with `Fatal` precedence) in
      `crates/drut-cli/src/exit.rs` (data-model.md §6).
- [x] T007 [P] Define the top-level `clap`-derive CLI skeleton — `Cli`,
      `Command::Check { path, .. }` / `Command::Format { path, .. }` sharing the
      single `path` argument (FR-001) — in `crates/drut-cli/src/cli.rs`
      (data-model.md §2). Implemented with all subcommand-specific flags already
      in place (T013, T026's flags), since `clap`'s `Command` enum has to be
      defined coherently in one pass; those two tasks are satisfied by this same
      edit.
- [x] T008 Wire `crates/drut-cli/src/main.rs`/`src/lib.rs` to parse `Cli`,
      dispatch to a (stub, returning `Clean`) `check_cmd::run`/`format_cmd::run`,
      and set the process exit code from the returned `ExitOutcome`. Depends on
      T006, T007. **Restructured from the original plan**: added `src/lib.rs`
      exposing all modules `pub`, with `main.rs` as a thin wrapper — a
      binary-only crate's modules are private to it, so `tests/*.rs` integration
      tests (T009 onward) couldn't otherwise reach them. Not a spec change, just
      the standard testable-binary pattern.
- [x] T009 [P] Add `crates/drut-cli/tests/traversal.rs` covering FR-001–FR-005:
      directory recursion, `.gitignore` respecting (including a nested
      `.gitignore`), extension filtering (`.s`/`.block` case-insensitive vs.
      `.mat`/`.net`/`.dbd`/`.prj`/other), a nonexistent path, an empty directory,
      and an unreadable file (Windows: exclusive-share lock; Unix: `chmod 000`,
      `cfg`-gated per platform). Depends on T005. 6/6 passing,
      `cargo clippy -p drut-cli --all-targets` zero-warning.

**Checkpoint**: Traversal, exit-code, and CLI scaffolding all compile and pass
their own tests. Both user stories can now proceed, in parallel if staffed.

---

## Phase 3: User Story 1 - Catch structural script defects before they reach Voyager (Priority: P1) 🎯 MVP

**Goal**: `drut check <path>` reports every `voyager-core` diagnostic across all
matched files, as plain text (default) or SARIF, with the 3-way exit code.

**Independent Test**: Run `drut check` against the full 161-file corpus (exit 0,
zero diagnostics) and against a directory with a deliberately-broken fixture (exit
1, correct diagnostic reported) — spec.md's own Independent Test for this story.

### Implementation for User Story 1

- [x] T010 [P] [US1] Implement `check_cmd::run` in
      `crates/drut-cli/src/check_cmd.rs`: for each `MatchedFile`, call
      `voyager_core::parse_bytes` (never `parse`, FR-006), tag every returned
      `Diagnostic` with its source file (FR-007) into a `CheckReport`
      (data-model.md §4), and derive its `ExitOutcome` per FR-011's three-way rule
      (`Fatal` takes precedence over "diagnostics found" when both apply).
- [x] T011 [P] [US1] Implement plain-text diagnostic rendering in
      `crates/drut-cli/src/report/text.rs`: per diagnostic, at minimum the file
      path, location, `DiagnosticKind`, and message (FR-008).
- [x] T012 [P] [US1] Implement SARIF rendering in
      `crates/drut-cli/src/report/sarif.rs` using hand-written
      `#[derive(Serialize)]` structs over `serde_json` (research.md §4 — no
      `serde-sarif` dependency): one `run`, `tool.driver.rules` listing every
      `DiagnosticKind` from `contracts/sarif-mapping.md` regardless of whether it
      fired, and one
      `result` per diagnostic with the `ruleId`/`level`/`message`/
      `physicalLocation` mapping that contract defines (FR-009).
- [x] T013 [US1] Add `--format=text|sarif` to `Command::Check` in
      `crates/drut-cli/src/cli.rs`, defaulting to `text` in every context,
      interactive or not (FR-010). Depends on T007. (Landed as part of T007's
      single-pass `cli.rs` edit — see that task's note.)
- [x] T014 [US1] Wire `check_cmd::run`'s report through the selected renderer
      (T011 or T012) and set the process exit code from its `ExitOutcome` in
      `crates/drut-cli/src/main.rs`. Depends on T010, T011, T012, T013. (Wired in
      `src/lib.rs::run()`, per T008's `main.rs`/`lib.rs` split.)
- [x] T015 [P] [US1] Add `crates/drut-cli/tests/exit_codes.rs` (check portion):
      all three FR-011 outcomes (clean / diagnostics-found / fatal) and the
      `Fatal`-takes-precedence rule when both a diagnostic and a read failure occur
      in the same run. 4/4 passing.
- [x] T016 [P] [US1] Add `crates/drut-cli/tests/sarif_schema.rs`: validate
      `check --format=sarif` output against the official SARIF 2.1.0 JSON Schema
      via the `jsonschema` crate, for both a clean fixture set (empty `results`)
      and a broken one (SC-003). Schema vendored at
      `crates/drut-cli/tests/schemas/sarif-2.1.0.json` (fetched from the
      OASIS-published errata01/OS schema — the standards-body source, so this
      runs offline). 2/2 passing, including a bonus check that every emitted
      `ruleId` is declared in `tool.driver.rules`.
- [x] T017 [US1] Add `crates/drut-cli/tests/fixture_corpus_e2e.rs`, gated behind
      a `DRUT_CORPUS_PATH` env var and `#[ignore]`'d unconditionally (not a
      runtime-conditional skip) — the WF-TDM-Official-Releases corpus is external,
      not committed (licensing open item, `001-voyager-script-parser/research.md`
      §3). Depends on T014. **Verified all three gating states directly**: (1)
      plain `cargo test` → reported under "ignored", not mixed into "passed"; (2)
      `cargo test -- --ignored` with the var unset → hard `FAILED` with a clear
      message, not a silent pass; (3) `cargo test -- --ignored` with
      `$env:DRUT_CORPUS_PATH = "D:\GitHub\WF-TDM-Official-Releases"` → **actually
      ran against the real 161-file corpus and passed** (SC-001 reproduced
      end-to-end through the CLI) in ~1.2s, well under SC-007's 5s target.
- [x] T018 [US1] Add a broken-fixture case to the same test file asserting exit
      code 1 and a correctly-identified diagnostic (SC-002), distinguishable from a
      read-failure run. Depends on T014. **Satisfied by T015's
      `check_broken_directory_exits_1` in `exit_codes.rs` instead of a duplicate
      in `fixture_corpus_e2e.rs`** — the external 161-file corpus itself is
      documented as containing zero broken files (001's own full-corpus
      validation), so the only real broken-fixture set to assert against is the
      already-committed `voyager-core/tests/fixtures/broken/`, which T015 already
      covers; see `fixture_corpus_e2e.rs`'s trailing comment for the same
      reasoning inline.

**Checkpoint**: `drut check` is fully functional, independently testable, and
shippable on its own (MVP).

---

## Phase 4: User Story 2 - Normalize script whitespace without changing behavior (Priority: P2)

**Goal**: `drut format <path>` normalizes whitespace (FR-012's concrete canonical
form) and, opt-in, keyword casing (FR-015) — idempotent, behavior-preserving, and
safe with respect to FR-034's decode fallback (FR-013(b)/FR-024 for recovered
bytes, FR-025 for lossy ones).

**Independent Test**: Run `drut format --write` twice on the full corpus (zero
further changes the second time) and re-parse every formatted file to confirm
identical structure — spec.md's own Independent Test for this story.

**✅ Human-in-the-loop dependency — resolved.** All 9 `real_corpus/` files
reviewed and approved (T023b); golden output generated and committed. One file
(`InputProcessing/1_InputSetup.s`) required a real fix first — a pre-existing
lexer bug (quoted strings had no effect on `;`/`/*` comment-start recognition,
silently splitting a `PRINT ... LIST=` statement whenever its string value
contained a literal `;`) — fixed, documented as an FR-004/FR-005 amendment in
`001-voyager-script-parser/spec.md`, regression-tested, and revalidated against
the full 161-file corpus (161/161 clean) before the corrected diff was
re-reviewed and approved. Kept below for the record of what this dependency
required.

**Original note (now resolved)**: `format`'s Definition of Done needs a golden-file corpus with
*known-correct* expected output, not merely "the formatter ran without crashing
and the structure round-trips" (that would only prove internal consistency, not
that the output is actually formatted the way a Cube Voyager script should be).
Concretely:

- **T023a** (golden copies for hand-written, project-authored fixtures) is fully
  agent-completable — no dependency on you. These fixtures have no external
  "correct" to defer to beyond FR-012's own rules, which this project itself
  wrote.
- **T023b** (golden copies for the 9 real, curated WF-TDM files already committed
  under `tests/fixtures/valid/real_corpus/`) is **not** agent-completable
  end-to-end. I can run the (once-implemented) formatter and generate a candidate
  diff, but I have no independent way to confirm that diff is what a Cube Voyager
  script *should* look like — only you (or someone with that domain context) can
  bless it as correct, the same way you directed which real files to curate/redact
  into the fixture corpus in the first place
  (`001-voyager-script-parser/research.md` §3, T049). Expect me to hand you a
  diff to review, not a finished, silently-committed golden file.
- **T024**'s `real_corpus/` assertions (as opposed to its hand-written-fixture
  assertions) inherit this same block — they can be *written* now but can't be
  reported as *passing* until T023b is approved.
- **T017/T033** (the full-161-file-corpus CLI-level end-to-end tests, for both
  `check` and `format`) do **not** have this dependency — they check
  self-referential properties (clean/idempotent/structurally-equivalent), not "is
  this the expected output," so they're safe for me to write and gate behind
  `DRUT_CORPUS_PATH` without your review. You only need to point that env var at
  your local corpus checkout to actually *run* them, which is a much lighter ask
  than reviewing formatted output.

### `voyager-core` additions for User Story 2

- [x] T019 [P] [US2] Define `CasingConvention`, `FormatOptions`,
      `EncodingFidelity`, and `FormatResult` in new
      `crates/voyager-core/src/format.rs`, re-exported from
      `crates/voyager-core/src/lib.rs` (data-model.md §1).
- [x] T020 [US2] Implement the canonical whitespace-normalization renderer in
      `crates/voyager-core/src/format.rs` per FR-012's seven concrete rules:
      spaces-only indentation (converting tabs), 4 spaces added per nesting level
      relative to each block's own opening-statement column (not column 0),
      zero-delta alignment for explicit closers and `ELSEIF`/`ELSE` against their
      `IF`, top-level baseline left untouched, continuation-line indentation left
      untouched, and comment content plus the whitespace on both sides of `;` left
      entirely untouched. Depends on T019.
      **Unplanned prerequisite discovered mid-task**: `Block`/`IfBranch` retained
      no token access for opener/closer statements at all (only `span`s) — needed
      to know whether a block closed *explicitly* (align its closer) versus
      *implicitly*/unmatched (don't touch the last child a second time), which
      isn't reconstructible from `span` alone. Added `Block.closer: Option<Span>`
      and `Block.opener_pairs: Vec<Span>` (the latter needed for T021's casing,
      not T020, but added at the same time/place) to
      `001-voyager-script-parser`'s already-shipped `Block` type — purely
      additive, grep-verified no external code constructs a `Block` literal or
      exhaustively destructures one, `cargo test -p voyager-core` (66 pre-existing
      tests) and `clippy` both stayed green throughout. Documented in that spec's
      own `data-model.md` and in this spec's `research.md` §8. 6 new direct unit
      tests added in `block.rs` for both fields.
      **Bug found and fixed via real-corpus spot check** (not caught by unit
      tests): the first implementation also stripped *trailing* whitespace on
      every re-indented line, which corrupted an inline comment's own trailing
      padding (real example: `;hbc      ` → `;hbc`) — a direct violation of the
      already-approved "comment content left entirely untouched" rule. Fixed by
      never touching trailing whitespace at all (FR-012 has no trailing-
      whitespace rule in the first place); 2 regression tests added.
- [x] T021 [US2] Implement the opt-in casing rewrite in the same renderer
      (FR-015): only tokens already structurally classified as a control word or a
      `keyword=value` pair's keyword name are ever rewritten — never a label's
      `:name`, an `@variable@` reference, or a keyword's value, even if it
      textually matches a control word. Depends on T020. Covers block
      openers/closers/`ELSEIF`/`ELSE` (via the `Block.closer`/`opener_pairs`
      additions above and a source-scanning `first_word_span` helper for the
      opener/closer/branch *words* themselves, since only their pair keywords are
      exact token spans) as well as ordinary (non-block-forming) `Control`
      statements (via `Statement.tokens` directly) — including `RUN PGM=...`'s
      `PGM`, which would otherwise have been silently missed entirely, since the
      opener `Statement` that carries it is discarded once matched into a
      `Block`.
- [x] T022 [US2] Implement the `format`/`format_bytes` entry points in
      `crates/voyager-core/src/format.rs`: `format_bytes` decodes via
      `decode::decode_bytes` (same as `parse_bytes`) and computes
      `EncodingFidelity` from the resulting diagnostics (`Lossy` if any
      `InvalidEncoding` present, else `Recovered` if the Windows-1252 fallback fired
      for any byte, else `Faithful`; `format` is always `Faithful`); both run the
      T020/T021 renderer and compute `changed` as an exact
      `text.as_bytes() != source` byte comparison (data-model.md §1). Neither
      function refuses to run for a `Lossy` input — it still returns a best-effort
      `text`, matching `parse`/`parse_bytes`'s never-refuses-to-run contract.
      Depends on T020, T021. **28 unit tests, all passing** (`cargo test -p
      voyager-core --lib format`), plus a throwaway/uncommitted smoke test
      (mirroring 001's own full-corpus-validation methodology) against the real
      161-file corpus: **0 panics, 0 idempotency failures, 0 structural diffs**
      (before/after `parse` node+diagnostic counts), 80/161 files would change,
      exactly 1 file classified `Recovered` (matching `001`'s own known T049
      finding — the same stray Windows-1252 byte), 0 files classified `Lossy`.
- [x] T023a [P] [US2] Add
      `crates/voyager-core/tests/fixtures/golden/` counterparts for every
      **hand-written, project-authored** fixture in `tests/fixtures/valid/`
      (i.e. everything there except `valid/real_corpus/`) — generate by running
      the T022 formatter and commit directly. **Fully agent-completable, no
      external review needed**: these fixtures' input was authored by this project
      to reproduce structural shapes, not sourced from a third party, so there's no
      independent "correct" to defer to beyond FR-012's own rules. Generated via
      `UPDATE_GOLDEN=1 cargo test -p voyager-core --test format_corpus` (a
      permanent, reusable regeneration mode — not a one-off script — gated behind
      that env var so a normal test run never silently rewrites goldens). All 14
      hand-written fixtures came back byte-identical to their input (they were
      already in canonical form) — spot-checked with a plain `diff` across every
      one, not just trusted blindly.
- [x] T023b [US2] Generate (do not yet commit as golden) candidate formatted
      output for every file in `crates/voyager-core/tests/fixtures/valid/
      real_corpus/` — the 9 curated, redaction-checked real WF-TDM files
      committed during `001-voyager-script-parser` (see that spec's
      `tests/fixtures/valid/real_corpus/README.md` for provenance). Present the
      before/after diff for review. Depends on T022. **Reviewed and approved**:
      8/9 files were clean re-indentation on first generation; 1
      (`InputProcessing/1_InputSetup.s`) exposed a pre-existing `001` lexer bug
      (see the "Human-in-the-loop dependency" note above) that was fixed,
      regression-tested, and revalidated against the full 161-file corpus before
      its corrected diff (1 clean line, down from 3 with 2 bogus) was re-reviewed
      and approved. Golden output committed to
      `tests/fixtures/golden/real_corpus/` (mirroring `real_corpus/`'s own
      subdirectory structure — filenames aren't unique across it), protected in
      `.gitattributes` (`-text`) for the same CRLF-fidelity reason as the source
      files, since `format` preserves each line's original line-ending style
      rather than normalizing it.
- [x] T024 [US2] Add `crates/voyager-core/tests/format_corpus.rs`: for every
      fixture-corpus file with an approved golden copy, assert its formatted
      output matches that golden copy, assert idempotency
      (`format(format(x).text).text == format(x).text`, FR-014/SC-004), and assert
      structural equivalence between the pre- and post-format parse trees modulo
      `Span` shifts and the two named FR-013 exceptions (SC-005). Depends on
      T022, T023a, T023b. **9 tests, all passing**: golden-diff/idempotency/
      structural-shape-signature checks for both the hand-written fixtures and
      (now that T023b is approved) `real_corpus/` — plus both T025
      encoding-fallback fixtures and a fixture-count tripwire
      (`real_corpus_fixture_count_is_the_known_nine`).
- [x] T025 [P] [US2] Add two hand-written fixtures —
      `crates/voyager-core/tests/fixtures/encoding_fallback/recovered.s` (contains
      a byte that decodes only via the Windows-1252 fallback, no diagnostic) and
      `crates/voyager-core/tests/fixtures/encoding_fallback/lossy.s` (contains a
      byte undecodable under either encoding) — plus test cases (in
      `crates/voyager-core/tests/format_corpus.rs` or a dedicated
      `crates/voyager-core/tests/encoding_fidelity.rs`) asserting `format_bytes`
      classifies them `Recovered`/`Lossy` respectively, since the real 161-file
      corpus exercises neither path. `quickstart.md` step 9 and T032 both exercise
      these same two files at the CLI layer, so their content only needs
      authoring/verifying once here. Depends on T022. Both fixtures are raw bytes
      (`recovered.s` has a literal `0x92` Windows-1252 right-single-quote byte;
      `lossy.s` has a literal `0x81`, undefined in Windows-1252), added to
      `.gitattributes` as `-text` so Git never normalizes their exact bytes —
      same discipline `001`'s `real_corpus/` fixtures already established. Both
      classification tests pass.

### CLI implementation for User Story 2

- [ ] T026 [US2] Add mutually-exclusive `--write`/`--check`/`--diff` and
      `--casing=upper|lower` (requiring an explicit value when present — no bare
      `--casing`) to `Command::Format` in `crates/drut-cli/src/cli.rs`. Depends on
      T007.
- [ ] T027 [US2] Implement `format_cmd::run` in `crates/drut-cli/src/format_cmd.rs`:
      for each `MatchedFile`, call `voyager_core::format_bytes`, classify it as
      `Unchanged`/`Changed`/`Written`/`WriteFailed` per data-model.md §5 (a
      `WriteFailed` also covers a pre-write refusal, not only an OS-level I/O
      failure), populate `FormatReport.unsafe_encoding_files`/
      `recovered_encoding_files` from `encoding_fidelity` in **every** mode (not
      only `--write`), and derive the run's `ExitOutcome` per FR-020/data-model.md
      §5 — including `Fatal` whenever `unsafe_encoding_files` is non-empty
      regardless of mode. Depends on T022, T026.
- [ ] T028 [P] [US2] Implement default (stdout print) and `--write`
      (overwrite-in-place) and `--check` (per-file "would reformat" listing, no
      write) output handling in `crates/drut-cli/src/format_cmd.rs` /
      `crates/drut-cli/src/report/text.rs` (FR-016–FR-018). Depends on T027.
- [ ] T029 [P] [US2] Implement `--diff` unified-diff rendering via the `similar`
      crate in `crates/drut-cli/src/report/text.rs` (or a new
      `crates/drut-cli/src/report/diff.rs`), one diff per changed file, writing
      nothing (FR-019). Depends on T027.
- [ ] T030 [US2] Implement the FR-024 visible summary line (e.g. "N file(s) had
      legacy-encoding bytes normalized to UTF-8") whenever
      `recovered_encoding_files` is non-empty, and the FR-025 per-file refusal
      report whenever `unsafe_encoding_files` is non-empty — both printed in every
      mode (default/`--write`/`--check`/`--diff`), not only `--diff`. Depends on
      T027, T028, T029.
- [ ] T031 [P] [US2] Add `crates/drut-cli/tests/format_flags.rs`: default output
      never writes; `--write` writes; `--check` reports without writing;
      `--diff` prints a diff without writing; `--casing` with no value or an
      unsupported value exits with a usage error before touching any file;
      `--write`/`--check`/`--diff` reject being combined. Depends on T026, T027,
      T028, T029.
- [ ] T032 [US2] Add the `format` portion of `crates/drut-cli/tests/exit_codes.rs`:
      clean / would-reformat (`--check`) / fatal, including
      `tests/fixtures/encoding_fallback/lossy.s` (T025) producing `Fatal` in
      **every** mode — default, `--check`, `--diff`, and `--write` alike — not only
      when `--write` is used. Depends on T027, T030, T025.
- [ ] T033 [US2] Extend `crates/drut-cli/tests/fixture_corpus_e2e.rs` with a
      `format --write` run-twice idempotency check and a re-parse
      structural-equivalence check against the full external 161-file corpus
      through the built `drut` binary, gated behind the same `DRUT_CORPUS_PATH`
      env var as T017
      (SC-004/SC-005 reproduced end-to-end via the CLI, matching T017's precedent
      of reproducing a `voyager-core`-level guarantee through the CLI itself).
      Unlike T023b, this needs **no human review** — idempotency and structural
      equivalence are self-referential properties (output-vs-itself,
      tree-shape-vs-tree-shape), not "does this match a human-approved expected
      value," so it's safe to run unattended against the full external corpus the
      same way T017 does. Depends on T027.

**Checkpoint**: `drut format` is fully functional and independently testable; both
user stories are now complete.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final gates that span both stories.

- [ ] T034 [P] Run `cargo clippy -p voyager-core -p drut-cli` and resolve every
      warning (CLAUDE.md's zero-warning gate).
- [ ] T035 [P] Update root `README.md`'s Status section to describe `drut-cli`
      alongside `voyager-core`, mirroring how it currently documents the parser
      crate's spec-kit artifacts.
- [ ] T036 Walk through `quickstart.md`'s validation steps end-to-end against a
      built `drut` binary and confirm each step's outcome matches its mapped
      Success Criterion (SC-001–SC-008).
- [ ] T037 [P] Document `cargo audit`/`cargo deny check advisories` as a
      recommended CI step for `drut-cli` (research.md §6's standing
      recommendation), e.g. in `crates/drut-cli`'s own README or the root one.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS both user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational only. Needs no
  `voyager-core` changes at all (`parse_bytes` already exists) — the most
  independent story in this feature.
- **User Story 2 (Phase 4)**: Depends on Foundational only, not on US1 — the
  `voyager-core` sub-group (T019–T025) has no dependency on anything in Phase 3,
  and the CLI sub-group (T026–T033) only reuses Phase 2's traversal/exit-code/CLI
  scaffolding, not US1's `check_cmd`/`report` code. Both stories can proceed in
  parallel once Phase 2 is done.
- **Polish (Phase 5)**: Depends on both user stories being complete (T034/T036
  exercise both crates; T035/T037 are documentation-only and could technically run
  earlier, but are sequenced last for simplicity).

### User Story Dependencies

- **User Story 1 (P1)**: No dependency on User Story 2.
- **User Story 2 (P2)**: No dependency on User Story 1 — `format` does not call
  `check`'s code, and vice versa. (Within US2 itself, the CLI sub-group T026–T033
  depends on the `voyager-core` sub-group T019–T025 via T022/T027, since
  `format_cmd::run` calls `format_bytes`.)

### Parallel Opportunities

- T001–T003 (Setup) are effectively sequential (each touches/depends on the prior
  file existing), except T003 which can start once T002 exists.
- T004, T006, T007 (Foundational) are mutually independent — `[P]`.
- T009 depends on T005 but is otherwise independent of T006–T008.
- Within US1: T010, T011, T012 are mutually independent — `[P]`; T015, T016 are
  independent of each other and of T017/T018.
- Within US2: T019 alone, then T020→T021→T022 sequential (same file); T023a is
  independent of T019–T022; T023b/T024/T025 all depend on T022 but not each other
  (though T024's `real_corpus/` assertions additionally wait on T023b's approval).
  T028, T029 are independent of each other (different rendering concerns) once
  T027 exists. T031 depends on T026–T029 all being done, since it exercises every
  flag.
- **Once Phase 2 (Foundational) is done, User Story 1 and User Story 2 can be
  staffed and built fully in parallel** — this is the feature's biggest parallel
  opportunity, since neither story's implementation code depends on the other's.

---

## Parallel Example: Foundational

```text
# Launch together once Setup (T001-T003) is done:
Task: "Define MatchedFile, ReadFailure, TraversalOutcome in crates/drut-cli/src/traverse.rs"
Task: "Define ExitOutcome and its exit-code mapping in crates/drut-cli/src/exit.rs"
Task: "Define the clap Cli/Command skeleton in crates/drut-cli/src/cli.rs"
```

## Parallel Example: User Story 1

```text
# Launch together once Foundational (Phase 2) is done:
Task: "Implement check_cmd::run in crates/drut-cli/src/check_cmd.rs"
Task: "Implement plain-text diagnostic rendering in crates/drut-cli/src/report/text.rs"
Task: "Implement SARIF rendering in crates/drut-cli/src/report/sarif.rs"
```

## Parallel Example: User Story 2

```text
# Launch together once Foundational (Phase 2) is done, in parallel with User Story 1:
Task: "Define CasingConvention/FormatOptions/EncodingFidelity/FormatResult in crates/voyager-core/src/format.rs"
Task: "Add crates/voyager-core/tests/fixtures/golden/ golden counterparts"

# Later, once format_bytes (T022) exists:
Task: "Implement default/--write/--check output handling in format_cmd.rs"
Task: "Implement --diff unified-diff rendering via similar"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational (blocks both stories).
3. Complete Phase 3: User Story 1 (`check`).
4. **STOP and VALIDATE**: run `drut check` against the full external 161-file
   corpus (`$DRUT_CORPUS_PATH`, T017) and a broken fixture per spec.md's own
   Independent Test; confirm SC-001/SC-002/SC-003/SC-006 (the `check`-relevant
   slice of SC-006) hold.
5. `drut check` is independently shippable here — `format` is not required for it
   to deliver value, per spec.md's own priority ordering.

### Incremental Delivery

1. Setup + Foundational → shared scaffolding ready.
2. User Story 1 → validate independently → ship (MVP).
3. User Story 2 → validate independently (idempotency + structural equivalence +
   encoding-safety scenarios) → ship.
4. Polish (clippy, docs, quickstart walkthrough) → final gate before merge.

### Parallel Team Strategy

With two developers: both complete Setup + Foundational together; then one takes
User Story 1 (Phase 3) while the other takes User Story 2's `voyager-core`
sub-group first (T019–T025, since the CLI sub-group T026–T033 depends on it) —
the two stories never touch the same files, so this can run fully in parallel
until Polish.

---

## Notes

- `[P]` tasks touch different files and have no incomplete dependency.
- `[US1]`/`[US2]` trace every story-phase task back to spec.md; Setup/Foundational/
  Polish tasks carry no story label by convention.
- Golden-file, idempotency, and structural-equivalence testing (constitution
  Principle III) happens at the `voyager-core` layer (T024) where the actual
  formatting decision logic lives — the CLI-layer tests (T031–T033) verify only
  traversal/flag-wiring/exit-code behavior, deliberately not re-proving what T024
  already proves (research.md §7's documented test-split rationale).
- Commit after each task or logical group; stop at either story's checkpoint to
  validate it independently before continuing.
