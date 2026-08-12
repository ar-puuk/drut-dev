# Feature Specification: Drut CLI — `check` and `format` Subcommands

**Feature Branch**: `002-cli-check-format`

**Created**: 2026-08-09

**Status**: Draft

**Input**: User description: "Build a CLI binary crate for Drut, exposing `check` and
`format` subcommands, as a thin adapter over the voyager-core library crate
(constitution Principle I — no grammar/parsing logic duplicated here, only I/O,
traversal, and output formatting). `drut check <path>` walks a file or directory
(respecting .gitignore), processes only `.s`/`.block` files, calls `parse_bytes()` on
each, and reports every `Diagnostic` found as SARIF or plain text, with a documented
non-zero exit convention. `drut format <path>` applies whitespace normalization
(with opt-in, user-configurable keyword-case normalization) under the same
traversal/filtering rules, must be idempotent and strictly behavior-preserving per
constitution Principle III, and is verified against the fixture corpus via
golden-file diffs. Definition of done: `check` reproduces voyager-core's proven
161/161-clean result on the full WF-TDM-Official-Releases corpus end-to-end through
the CLI; `format` passes idempotency/behavior-preservation golden-file checks on the
same corpus; SARIF output validates against the SARIF schema."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Catch structural script defects before they reach Voyager (Priority: P1)

A script author or a CI pipeline runs `drut check` against a project's Voyager
scripts — a single file or an entire directory tree — and gets back every structural
defect (unmatched `IF`/`LOOP`/`RUN`, unclosed block comment, invalid continuation,
misplaced `BREAK`, undecodable byte) the underlying parser can find, with the run
exiting cleanly when scripts are structurally sound and failing loudly, with concrete
file/line detail, when they are not.

**Why this priority**: This is the whole reason the CLI exists — without it, the
already-working `voyager-core` parser has no way for a human or a CI job to actually
use it. Nothing else in this feature has value without this working first.

**Independent Test**: Run `drut check` against the full WF-TDM-Official-Releases
fixture corpus (161 files) and confirm it exits 0 with no diagnostics reported —
reproducing, end-to-end through the CLI, the result `voyager-core`'s own test suite
already proves at the library level. Then run it against a directory containing at
least one deliberately-broken fixture and confirm it exits non-zero and reports the
specific defect with a file path and location.

**Acceptance Scenarios**:

1. **Given** a directory containing only structurally valid `.s`/`.block` files,
   **When** the user runs `drut check <dir>`, **Then** the command exits 0 and
   reports zero diagnostics.
2. **Given** a directory containing at least one file with a structural defect (e.g.
   an unmatched `IF`), **When** the user runs `drut check <dir>`, **Then** the
   command exits with the "diagnostics found" exit code and prints that defect's
   file path, location, and description.
3. **Given** a directory that mixes `.s`/`.block` files with non-script files (e.g.
   `.mat`, `.net`, `.prj`) and a `.gitignore` that excludes a subdirectory,
   **When** the user runs `drut check <dir>`, **Then** only `.s`/`.block` files
   outside the ignored subdirectory are read and reported on; every other file is
   silently skipped.
4. **Given** a single `.s` file passed directly (not a directory), **When** the user
   runs `drut check <file>`, **Then** only that file is checked.
5. **Given** a path that does not exist, or a file that cannot be opened (e.g.
   permission denied), **When** the user runs `drut check <path>`, **Then** the
   command exits with a distinct "couldn't run" exit code, separate from the
   "diagnostics found" exit code, and reports which path failed and why.
6. **Given** the same directory as Scenario 1, **When** the user runs
   `drut check <dir> --format=sarif`, **Then** the command emits a single SARIF log
   on stdout that validates against the SARIF 2.1.0 schema and contains zero
   `results`.
7. **Given** the same directory as Scenario 2, **When** the user runs
   `drut check <dir> --format=sarif`, **Then** the SARIF log's `results` array
   contains one entry per diagnostic, each with a `ruleId` matching the
   diagnostic's kind and a `physicalLocation` pointing at the reported file and
   position.

---

### User Story 2 - Normalize script whitespace without changing behavior (Priority: P2)

A script author runs `drut format` against their scripts to clean up whitespace
inconsistencies, trusting that the tool will never reorder statements, change which
lines continue a prior statement, or otherwise alter what the script does — only its
whitespace (and, only if explicitly requested, keyword casing).

**Why this priority**: Formatting is valuable but strictly secondary to `check` —
teams can adopt structural linting alone, and a formatter that behaves unsafely is
worse than none (constitution Principle III), so this depends on `check`'s
traversal/parsing plumbing being solid first.

**Independent Test**: Run `drut format --write` twice in succession on the full
fixture corpus and diff the corpus against itself between the two runs — the second
run must produce zero further changes (idempotency). Separately, re-parse every
formatted file with `voyager-core` and confirm the statement/block structure is
unchanged from the pre-format parse (behavior preservation).

**Acceptance Scenarios**:

1. **Given** a `.s` file with inconsistent indentation and trailing whitespace,
   **When** the user runs `drut format <file>` with no other flags, **Then** the
   normalized content is printed to stdout and the original file on disk is
   unchanged.
2. **Given** the same file, **When** the user runs `drut format <file> --write`,
   **Then** the file on disk is overwritten with the normalized content.
3. **Given** a file already in normalized form, **When** the user runs
   `drut format <file> --check`, **Then** the command exits 0 and makes no changes.
4. **Given** a file that is not yet normalized, **When** the user runs
   `drut format <file> --check`, **Then** the command exits with the
   "would reformat" exit code, makes no changes, and lists the file as needing
   formatting.
5. **Given** a file that is not yet normalized, **When** the user runs
   `drut format <file> --diff`, **Then** the command prints a unified diff of the
   whitespace changes it would make, without writing to disk.
6. **Given** a directory of files, **When** the user runs
   `drut format <dir> --write`, **Then** every `.s`/`.block` file in the directory
   (respecting `.gitignore` and extension filtering, same rules as `check`) is
   normalized in place, and running the same command again makes no further changes.
7. **Given** a file already normalized, **When** the user runs
   `drut format <file> --write --casing=upper`, **Then** only control-word and
   keyword-name casing changes (to uppercase); no whitespace, statement order, or
   continuation structure changes.
8. **Given** any file in the fixture corpus, **When** it is formatted and then
   re-parsed, **Then** its statement/block tree (ignoring only whitespace-only
   `Token` content) is identical to the tree produced by parsing the original file.
9. **Given** a file containing at least one byte that decodes only via FR-034's
   Windows-1252 fallback (no diagnostic results), **When** the user runs
   `drut format <file> --write`, **Then** the file is written with that byte in its
   decoded UTF-8 form, and the run's output includes a visible line reporting that
   this file's encoding was normalized — not only when `--diff` is also passed.
10. **Given** a file containing at least one byte that decodes under neither UTF-8
    nor Windows-1252 (an `InvalidEncoding` diagnostic results), **When** the user
    runs `drut format <file> --write`, **Then** the file on disk is left completely
    unchanged, the command reports that this file was refused for safety, and the
    run's exit code is the same "couldn't safely complete" code a read/write
    failure would produce — this holds the same way (file flagged, same exit code)
    even if `--write` is replaced with `--check`, `--diff`, or no flag at all.

### Edge Cases

- An empty directory, or a directory containing no `.s`/`.block` files at all: `check`
  and `format` both report zero files processed and exit 0 (not an error — there is
  nothing wrong to report).
- A file with a `.s`/`.block` extension whose bytes are not valid UTF-8: for
  `check`, still processed (via `parse_bytes`'s Windows-1252 fallback, FR-034 in
  `001-voyager-script-parser`); any genuinely undecodable byte surfaces as an
  `InvalidEncoding` diagnostic like any other, not a fatal I/O error. For `format`,
  the two FR-034 fallback outcomes are treated differently (FR-013(b), FR-024,
  FR-025): a successfully-recovered byte is written through in decoded form and
  reported; a genuinely-undecodable byte blocks that file from being written at
  all, in every mode, not only `--write`.
- A file with a `.s`/`.block` extension that happens to be a mislabeled binary file
  (arbitrary non-text bytes): `parse_bytes` never panics or errors, so it is still
  processed and will typically produce a large number of diagnostics rather than
  crashing the run.
- `--casing` is passed to `format` without a value, or with a value that isn't a
  supported convention: the command exits with a usage error before touching any
  file.
- The target path is a symlink, or a directory contains symlinks: out of scope for
  this phase — traversal follows the same default behavior as the underlying
  directory-walking mechanism; symlink cycles are not specifically guarded against.
- Two files under the given path resolve to the same on-disk file (e.g. via
  symlink): each is processed independently; duplicate diagnostics/format actions
  are expected and not deduplicated.
- `drut format --write` is interrupted mid-run (e.g. process killed): files already
  written are normalized, files not yet reached are untouched — there is no
  transactional all-or-nothing guarantee across a multi-file run.

## Requirements *(mandatory)*

### Functional Requirements

**Traversal & filtering (shared by `check` and `format`)**

- **FR-001**: The CLI MUST accept a single path argument that is either a file or a
  directory.
- **FR-002**: When the path is a directory, the CLI MUST recurse through all
  subdirectories, honoring `.gitignore` rules (including nested `.gitignore` files)
  the same way `git` itself would decide whether a file is ignored.
- **FR-003**: The CLI MUST only read and process files whose extension is `.s` or
  `.block` (case-insensitive on the extension). Every other file — including known
  binary Cube file types (`.mat`, `.net`, `.dbd`, `.prj`) and any extension not on
  this list — MUST be skipped without being opened, read, or reported as an error.
- **FR-004**: If the given path does not exist, or is neither a file nor a
  directory, the CLI MUST report this as a fatal error and MUST NOT report it as
  "zero diagnostics" success.
- **FR-005**: If a specific matched file cannot be read (e.g. permission denied,
  removed mid-run), the CLI MUST report which file failed and why, MUST continue
  processing the remaining matched files rather than aborting the whole run, and MUST
  ensure the run's final exit code reflects that at least one file could not be read
  (distinct from "diagnostics found", per FR-010).

**`drut check`**

- **FR-006**: For every matched file, `check` MUST read the file's raw bytes and
  call `parse_bytes()` (never `parse()`) so that non-UTF-8 script content is handled
  the same way the underlying parser already guarantees.
- **FR-007**: `check` MUST collect every `Diagnostic` returned across all matched
  files, each tagged with the file it came from, and MUST report all of them — it
  MUST NOT stop at the first file or the first diagnostic.
- **FR-008**: `check` MUST support a plain-text output mode that lists, per
  diagnostic, at minimum: the file path, the diagnostic's location within that file,
  its kind, and its message.
- **FR-009**: `check` MUST support a SARIF output mode (SARIF 2.1.0) via
  `--format=sarif`, emitting one SARIF `run` covering all processed files, with one
  `result` per diagnostic mapping the diagnostic's kind to a stable SARIF `ruleId`
  and its location to a SARIF `physicalLocation`.
- **FR-010**: `check`'s output format MUST default to plain text; SARIF output is
  opt-in only, selected explicitly with `--format=sarif`, regardless of whether the
  command is run interactively or non-interactively (e.g. in CI).
- **FR-011**: `check` MUST exit with a distinct code for each of these three
  outcomes: (a) every matched file processed with zero diagnostics and no read
  failures, (b) at least one file produced at least one diagnostic but every matched
  file was read successfully, (c) at least one matched file could not be read at all,
  or the given path itself was invalid (FR-004/FR-005) — this outcome MUST be
  distinguishable from (b) so CI tooling can tell "your scripts have a bug" apart
  from "the check run itself couldn't complete." Outcome (c) takes precedence when
  it and (b) both occur in the same run.

**`drut format`**

- **FR-012**: For every matched file, `format` MUST normalize whitespace to a single
  canonical form, using the same parse-then-render pipeline for every file so
  behavior is consistent across the whole corpus. This canonical form is derived
  from a 161-file structural survey of the WF-TDM-Official-Releases corpus (the same
  corpus, and the same "let real signal decide, don't invent a style" methodology,
  as the keyword-casing survey referenced in FR-015/Assumptions), not an invented
  style:
  - **Indentation unit**: spaces. A tab-indented line is converted to the space-based
    scheme below — tabs are the minority convention (23/161 files, 14.3%, contain
    any tab-indented line).
  - **Per-nesting-level increment**: 4 spaces, added to the enclosing block's own
    opening-statement column — not re-anchored to column 0, so a block's own
    existing baseline is preserved and only the *increment* per nested level is
    normalized. Confirmed dominant: 82.4% of 30,652 real body-indent occurrences
    across the corpus use exactly this delta; 112/128 files with nested content
    (87.5%) have 4 as their own per-file dominant delta, and 86/128 (67%) are
    internally ≥90% consistent on a single value. This is a materially stronger,
    more uniform signal than the casing survey found for any keyword family, which
    is why — unlike FR-015's casing decision — a single default is adopted here
    rather than left unforced.
  - **Explicit block closers** (`ENDIF`, `ENDLOOP`, `ENDRUN`, `ENDPROCESS`/
    `ENDPHASE`, `ENDJLOOP`, `ENDLINKLOOP`, `EndDistributeMULTISTEP`) align to the
    same column as their own opening statement (delta 0). Confirmed near-unanimous:
    99.2% of 2,461 real closer/opener pairs.
  - **`ELSEIF`/`ELSE`** align to the same column as their `IF` (delta 0). Confirmed
    near-unanimous: 98.7% of 1,250 real occurrences.
  - **Top-level (depth-0) statement indentation defaults to left untouched
    (`preserve`)**, on every format pass, unless `--top-level-indent=normalize`
    is explicitly requested, in which case it is always normalized to column 0,
    unconditionally — regardless of the statement's current indentation or
    formatting history. **Amended 2026-08-11 (`008-top-level-indentation-
    normalization`)**: this reverses the rule's original form, which left
    top-level indentation untouched. The corpus finding behind that original
    rule is historical record, not disproven — only 20.4% of real top-level
    statements sit at column 0 (best single value 26.9%, at column 8) — but the
    project has deliberately traded preserving that real-author diversity for
    predictability, knowing this reformats a majority of real top-level
    statements in the reference corpus on first run. See the dated Assumptions
    entry below for the full reversal writeup. **Amended again 2026-08-12
    (`009-top-level-indent-toggle`)**: this reverts `008`'s *default* back to
    the original `007`-era `preserve` behavior — `008`'s corpus-evidence
    framing was never in question; the project simply decided predictability-
    by-default was the wrong trade for users who never asked for it. `008`'s
    `normalize` behavior is fully retained, unchanged, as an explicit opt-in
    (FR-026). See the second dated Assumptions entry below.
    A block's children are still only ever indent-planned (per the
    per-nesting-level rule above) when that block itself is *not* the subject
    of an `UnmatchedIf`/`UnmatchedLoop`/`UnmatchedRun`/`UnmatchedProcess`
    diagnostic (**`007-formatter-diagnosed-block-indent-fix`**, narrowed by
    `008`'s own dated entry below) — under `normalize`, the block's *own opener
    line* is unconditionally corrected regardless of diagnosis; only its
    *children* remain protected, since their structural relationship to a
    genuinely unmatched block stays uncertain no matter what column the opener
    lands on. Under `preserve` (the default), neither the opener nor the
    children are touched — the same non-overlapping-responsibility split, just
    with nothing forcing the opener in the first place.
  - **Continuation-line indentation is left untouched.** No dominant convention
    exists (best single value only 23.0%, with a long flat tail) — a weaker signal
    than even the casing survey found for `IF`/`LOOP`/`JLOOP`, so it receives the
    same "don't force a convention where none exists" treatment FR-015 already
    applies to casing.
  - **Comment content, and the whitespace immediately before and after `;`, are
    left entirely untouched** — including the 87.7%-dominant "no space after `;`"
    pattern found in the survey (17,202 real comments sampled). This is deliberately
    *not* normalized despite that signal: the "spaces before `;`" side shows clear
    evidence of authors hand-aligning comment columns across a block of statements
    (a long tail including 47- and 23-space gaps), and a rule that touches one side
    of `;` but not the other is fussier for a small gain. Comments are treated as
    opaque past their opening delimiter.
- **FR-013**: `format` MUST NOT change which lines are continuations of a prior
  statement, MUST NOT reorder statements or blocks, and MUST NOT alter any token's
  meaningful (non-whitespace) content — with exactly two named exceptions:
  (a) keyword/control-word casing, only when casing normalization is explicitly
  enabled (FR-015); and (b) a byte that FR-034's decode fallback
  (`001-voyager-script-parser`) successfully recovered under its Windows-1252
  fallback (no diagnostic resulted), which `format` persists in its decoded UTF-8
  form rather than attempting to reproduce the original non-UTF-8 byte sequence —
  see FR-024 for the reporting obligation this carries. This carve-out does **not**
  extend to a byte FR-034 replaced with the Unicode replacement character
  (`InvalidEncoding`): that substitution is lossy, not a faithful re-encoding, and
  `format` MUST NOT silently persist it — see FR-025.
- **FR-014**: `format` MUST be idempotent: formatting an already-formatted file MUST
  produce byte-identical output to its input.
- **FR-015**: `format` MUST support an opt-in keyword-case normalization flag,
  defaulting to OFF. When enabled, the flag MUST require the caller to specify which
  casing convention to apply (e.g. all-uppercase or all-lowercase) — there is no
  built-in default convention, since corpus research found no dominant house style
  to default to. The rewrite targets only recognized control words and
  `keyword=value` pair keyword names (FR-003 in `001-voyager-script-parser`); a
  label statement's `:name` (FR-021 there), an `@variable@` reference, and any
  keyword's *value* are never casing targets regardless of whether they happen to
  textually match a control word or keyword name — casing normalization only ever
  touches a token already structurally recognized as a control word/keyword name by
  parsing, never a token classified as something else.
- **FR-016**: `format`'s default invocation (no write/check/diff flag) MUST print
  the formatted result to stdout and MUST NOT modify any file on disk.
- **FR-017**: `format` MUST support a `--write` flag that overwrites each matched
  file in place with its formatted content.
- **FR-018**: `format` MUST support a `--check` flag that reports, per matched file,
  whether formatting it would change its content, without writing to disk or
  printing the full formatted content, and without requiring `--write`.
- **FR-019**: `format` MUST support a `--diff` flag that prints a unified diff of
  the whitespace (and, if enabled, casing) changes it would make for each matched
  file that would change, without writing to disk.
- **FR-020**: `format` MUST exit with a distinct code for each of: (a) no matched
  file needed a change (or, for the default/`--write` modes, every write succeeded),
  (b) `--check` found at least one file that would change, (c) a matched file could
  not be read, could not be written (for `--write`), **or was refused for writing
  because persisting its formatted content would require silently writing a
  lossy-decoded byte (FR-025)** — this refusal folds into the same outcome (c) as an
  I/O read/write failure rather than becoming a fourth exit-code case, and applies
  regardless of which mode (`--write`, default, `--check`, or `--diff`) encountered
  the file, since even a non-writing mode benefits from knowing `--write` would
  refuse. Mirrors `check`'s three-way exit convention (FR-011) so both subcommands
  are predictable from the same mental model.
- **FR-021**: Every formatter change MUST be verified against the fixture corpus via
  a golden-file diff before merge — the test suite established in this phase MUST
  be able to run `format` over the full corpus and fail if any file's formatted
  output differs from its checked-in golden copy, or if idempotency (FR-014) or
  structural equivalence (FR-013) does not hold for any corpus file.

**Cross-cutting**

- **FR-022**: Neither `check` nor `format` MUST duplicate any grammar, parsing, or
  formatting-decision logic that belongs in `voyager-core` — both subcommands MUST
  express their behavior purely in terms of `voyager-core`'s public entry points
  (`parse_bytes` for `check`; whatever `voyager-core`-exposed formatting primitive
  `format` is built on) plus file I/O, traversal, and output rendering.
- **FR-023**: Neither subcommand MUST panic on any input file content, including
  malformed or non-UTF-8 bytes — a per-file failure MUST be reported as a diagnostic
  or a read/write error, never as an uncaught crash that aborts the whole run.

**Encoding safety for `format` (FR-034 interaction)**

- **FR-024**: `format` MUST report, visibly and in every output mode (default,
  `--write`, `--check`, and `--diff` alike — not only `--diff`), when a matched
  file's formatted content differs from its input because a byte needed FR-013(b)'s
  Windows-1252-recovery carve-out, and MUST report an aggregate count of how many
  matched files this occurred for in a given run. This kind of byte-level content
  change MUST NOT be a side effect a user only discovers by re-diffing the file
  later.
- **FR-025**: `format` MUST refuse to persist formatted content (i.e. MUST NOT
  write, under `--write`) for any matched file where decoding replaced a byte with
  the Unicode replacement character (`InvalidEncoding`, FR-034 in
  `001-voyager-script-parser`) — real data loss MUST NOT be silently written to
  disk. This refusal is reported the same way regardless of mode: even `--check`/
  `--diff`/default (none of which write anyway) MUST flag such a file distinctly
  from an ordinary "would reformat"/"changed" result, and the run's overall exit
  code MUST reflect the same "couldn't safely complete" outcome as a read/write
  failure (FR-020(c)) — this is a specific *reason* within that existing outcome,
  not a new, fourth exit-code case.
- **FR-026**: `format` MUST support a `--top-level-indent` flag accepting `preserve`
  (default) or `normalize`, selecting between FR-012's two top-level indentation
  behaviors. Unlike FR-015's `--casing` flag, this setting has no "off" state —
  omitting the flag resolves to the explicit `preserve` default, not an unset/`None`
  value. Added `2026-08-12` (`009-top-level-indent-toggle`).

### Key Entities

- **Matched File**: One file selected for processing by the traversal/filtering
  rules (FR-001–FR-003) — a path plus its raw bytes once read.
- **Check Report**: The aggregate result of a `check` run — the full list of
  per-file `Diagnostic` values (from `001-voyager-script-parser`'s `Diagnostic`
  entity) plus the list of any files that could not be read, rendered as either
  plain text or a SARIF log.
- **SARIF Log**: A SARIF 2.1.0 document produced by `check --format=sarif`; one
  `run`, with `results` derived one-for-one from the Check Report's diagnostics.
- **Format Report**: The aggregate result of a `format` run — for each matched
  file, whether it needed a change, and (for `--diff`) the unified diff of that
  change; plus, run-wide, the count of files whose content changed only/also
  because of an FR-013(b) encoding recovery (FR-024), and the list of files refused
  for writing under FR-025.
- **Encoding Fidelity**: A per-file classification of how `format`'s decoding of a
  matched file relates to what's safe to write back: unchanged from a pure-UTF-8
  read: fully faithful; at least one byte recovered via FR-034's Windows-1252
  fallback with no diagnostic: recovered (safe to write, but reportable, FR-024);
  at least one byte replaced with the Unicode replacement character
  (`InvalidEncoding`): lossy (never safe to write, FR-025).
- **Golden Fixture**: A checked-in "known-correct formatted output" file paired
  with a fixture corpus input file, used by the golden-file test suite (FR-021) to
  detect any unintended formatting drift.
- **Exit Code**: The single integer the CLI process returns, drawn from each
  subcommand's three-way convention (FR-011, FR-020).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Running `drut check` against the full WF-TDM-Official-Releases corpus
  (161 files) reports zero diagnostics and exits with the "clean" exit code —
  reproducing, through the CLI end-to-end, the same result already proven at the
  `voyager-core` library level.
- **SC-002**: Running `drut check` against a directory containing any deliberately-
  broken fixture reports at least one diagnostic correctly identifying that fixture's
  injected defect and exits with the "diagnostics found" exit code, distinguishable
  from a run where a file simply couldn't be read.
- **SC-003**: A SARIF log produced by `drut check --format=sarif` validates against
  the official SARIF 2.1.0 JSON schema on 100% of corpus runs (clean and broken).
- **SC-004**: Running `drut format --write` twice in a row on the full corpus
  produces no further file changes on the second run (idempotency holds for every
  file, not just a sample).
- **SC-005**: For every file in the corpus, the statement/block structure obtained
  by parsing the file before and after `drut format` is applied is identical
  (behavior preservation holds for every file).
- **SC-006**: A user or CI job can distinguish, purely from the process exit code
  and without parsing output text, among "nothing wrong," "the tool found a problem
  in your scripts," and "the tool itself failed to complete the run" for both
  `check` and `format`.
- **SC-007**: `drut check` completes a full run over the 161-file corpus in under 5
  seconds on typical developer hardware, making it practical to run on every commit
  or as a local pre-commit check.
- **SC-008**: Across every run of `drut format` in any mode, against any corpus
  file, a byte that was genuinely undecodable (`InvalidEncoding`) is never written
  to disk in its lossy-substituted form — 100% of such files are refused rather than
  silently written — and a byte that was successfully recovered under FR-034's
  Windows-1252 fallback is never written without that file also being named in the
  run's visible output, whether or not `--diff` was passed.

## Assumptions

- **CLI crate is not bound by `voyager-core`'s zero-dependency rule.** Constitution
  Principle I and `voyager-core`'s FR-027 scope the "zero runtime dependencies"
  constraint to the core crate specifically; this CLI is a separate crate and may
  depend on ordinary Rust ecosystem crates for concerns like argument parsing,
  `.gitignore`-aware directory walking, and SARIF/JSON serialization, since
  hand-rolling those would be pure duplicated effort with no grammar/parsing
  content.
- **Exit code convention**: `0` = clean run, nothing to report; `1` = the run
  completed but found something to report (diagnostics for `check`, files needing
  formatting for `format --check`); `2` = the run itself could not complete for at
  least one target (bad path, unreadable/unwritable file). This gives CI tooling a
  simple three-way signal without inventing a large code space, and mirrors common
  conventions in comparable tools (e.g. linters that separate "lint failures" from
  "tool crashed").
- **`format`'s default output target**: default (no flag) prints to stdout and
  never writes, matching common Unix formatter conventions (e.g. `gofmt`) and
  erring toward the safer, non-destructive choice; `--write` is required to modify
  files in place. For a directory target, default/no-flag behavior still prints
  each matched file's formatted content to stdout (concatenated, file boundaries
  distinguishable in output) — in practice, users targeting a directory are
  expected to use `--write`, `--check`, or `--diff` rather than the concatenated-
  stdout default.
- **Keyword-casing convention has no hardcoded default.** Per the feature
  description's corpus research (no dominant house style for `IF`/`ELSEIF`/
  `LOOP`/`JLOOP`), enabling casing normalization requires the caller to name the
  convention explicitly (e.g. `--casing=upper` or `--casing=lower`); there is no
  bare "on" state for this flag.
- **SARIF is opt-in everywhere, including CI.** Plain text remains the default
  output for `check` in both interactive and non-interactive contexts; the CLI does
  not sniff for a CI environment or a non-TTY stdout to silently switch formats,
  keeping output format entirely explicit and predictable from the invocation
  alone.
- **SARIF severity mapping**: `001-voyager-script-parser`'s `Diagnostic` contract
  deliberately defines no severity levels (all seven kinds are structural defects
  or decoding fallbacks, not heuristic lint warnings). Every SARIF `result` in this
  phase is emitted at SARIF `level: "error"`, since a structural parse defect is
  never merely stylistic; this may become configurable if/when a future phase adds
  heuristic lint rules with their own severity spectrum (constitution Principle IV).
- **Golden-file fixtures live alongside the existing fixture corpus** used by
  `voyager-core`'s own tests (`crates/voyager-core/tests/fixtures/`), with formatted
  "golden" counterparts added in this phase's own test infrastructure rather than a
  separate corpus.
- **The full WF-TDM-Official-Releases corpus's sourcing/licensing status remains an
  open item** (per `001-voyager-script-parser`'s `research.md` §3) at the time of
  this spec; this feature's definition-of-done validation against that corpus
  depends on it being available locally the same way it already is for
  `voyager-core`'s own test suite.
- **Symlink handling, transactional multi-file writes, and file-locking/concurrent-
  run safety are out of scope** for this phase — traversal and writes use
  straightforward sequential file I/O with no special-cased handling beyond what's
  described in the Edge Cases above.
- **No configuration file support in this phase** — all behavior (casing
  convention, output format, write mode) is controlled purely via CLI flags on each
  invocation; a persisted project-level config (e.g. `drut.toml`) is left to a later
  phase if it proves necessary.
- **Whitespace-canonical-form survey methodology**: FR-012's concrete rules were
  derived by parsing all 161 WF-TDM-Official-Releases files with `voyager-core`'s
  actual `parse_bytes`/`tokenize_bytes` (not a regex approximation) via a throwaway,
  uncommitted example script — the same read-only, nothing-copied-into-the-repo
  methodology `001-voyager-script-parser/research.md` §3 already used for its
  full-corpus validation pass, and the same "let dominant real signal decide, don't
  invent a style" principle the keyword-casing survey applied. Measured: per-level
  body-indent deltas (against each block's own enclosing opener, not an absolute
  column), explicit-closer and `ELSEIF`/`ELSE` alignment deltas, top-level baseline
  indentation, continuation-line indentation, and spacing on both sides of inline
  `;` comments — both as corpus-wide aggregates and, for indentation width
  specifically, a per-file consistency breakdown (mirroring the casing survey's
  per-file mixing count) to confirm the aggregate dominance wasn't an artifact of a
  few outsized files. Where a dimension showed a real dominant convention
  (indentation width, closer/branch alignment), that convention became the rule;
  where it didn't (top-level baseline, continuation lines, comment spacing), FR-012
  leaves that dimension untouched rather than picking a style — the identical
  reasoning FR-015 already applies to keyword casing, just re-run per dimension.
- **2026-08-11 bug fix (`007-formatter-diagnosed-block-indent-fix`): genuinely
  unmatched blocks no longer have their children speculatively indent-planned.**
  Surfaced via real manual testing during `005-format-on-save-paste`'s
  verification: a `PROCESS PHASE=...` left unclosed swallows trailing content
  (e.g. a `RUN PGM=...` block) as its own children — correct, given the broken
  structure at that point. `format` used to still confidently reindent that
  swallowed content one level deeper, matching FR-012's ordinary per-nesting-level
  rule. The bug: once the user added the real closer (`ENDPROCESS`) in a later
  edit, the swallowed content became a genuine top-level sibling — but the
  *indentation the formatter itself had written* while the block was still broken
  survived untouched forever after, because top-level lines are deliberately
  never re-planned (the bullet above) and there is no way, from source text
  alone, to distinguish that formatter-written residue from an author's own
  deliberate top-level indentation (which the same corpus survey — 26.9% at
  column 8, only 20.4% at column 0 — shows is common and real, not rare). Fix:
  `plan_indentation`/`plan_block` now take the parse's own `&[Diagnostic]` and
  skip planning entirely for a block's children when that block's own opener
  matches an `UnmatchedIf`/`UnmatchedLoop`/`UnmatchedRun`/`UnmatchedProcess`
  diagnostic — the legitimate implicit-close pattern (`closer: None`, no
  diagnostic — e.g. back-to-back `RUN`/`PROCESS`) is completely unaffected and
  still fully indent-planned as before. Confirmed general, not `Process`-specific:
  the identical residue reproduces for `RUN`/`IF` and is fixed by the same
  change. New test category, `tests/format_sequence.rs` — every existing test in
  `format_corpus.rs`/`block.rs` is single-shot (one fixture, one `format` call,
  compared to itself repeated or a static golden file); none of them ever apply
  a structural edit *between* two format calls, which is exactly why this bug had
  zero prior coverage: `format(x)` on the buggy output was already a stable
  no-op, so the existing idempotency check (`format(x) == format(format(x))`)
  held trivially — idempotence proves stability of a fixed point, never its
  correctness. All 161 real corpus files' golden output is unchanged by this fix
  (none of them have a top-level diagnosed block) — re-confirmed, not assumed,
  via `cargo test -p voyager-core --test format_corpus` needing zero golden-file
  regeneration.
- **2026-08-11 policy reversal (`008-top-level-indentation-normalization`):
  top-level indentation now always normalizes to column 0.** A deliberate
  reversal, not new evidence contradicting the original survey — the
  26.9%-at-column-8/20.4%-at-column-0 finding above remains an accurate
  historical record of what real authors did; the project decided
  predictability now outweighs preserving that diversity, knowingly accepting
  that this reformats a majority of real top-level statements in the
  reference corpus on first run. Fix: `plan_indentation` force-plans every
  top-level node's own line (statement *or* block opener — a bare top-level
  statement had *no* code path touching it at all before this change, since
  `plan_indentation` only ever iterated `Node::Block` entries) to column 0,
  unconditionally, before `plan_block` computes each block's own children's
  base — `computed_indent`'s existing "prefer a planned value over the
  original" fallback makes this a single-line addition, no other function
  changed. Interaction with `007` above, resolved and tested explicitly (not
  left to be inferred): `007`'s skip never actually protected a diagnosed
  block's *opener* line — this new rule now does that independently, and
  unconditionally even for a still-diagnosed block, proven against the exact
  stale-indentation shape `007` alone never corrected (a `RUN` block left at
  non-zero indentation by a prior pass, revealed as top-level once
  `ENDPROCESS` is added, now resolves fully in the very next format pass).
  `007`'s skip is kept, unchanged in code, because it protects something
  `008` doesn't touch: a diagnosed block's *children*, whose structural
  relationship to that block stays uncertain regardless of what column the
  opener itself lands on — confirmed with a dedicated test asserting a
  diagnosed block's opener is corrected while every child (legitimate body
  content and swallowed trailing content alike) stays byte-for-byte
  untouched. Golden-fixture impact measured, not estimated: 7 of the 9
  `real_corpus/` fixtures drift (their own top-level content was not
  already at column 0); zero hand-written `valid/` fixtures affected. Every
  regenerated golden file was individually diff-reviewed before being
  committed, confirming only top-level indentation shifted and nothing else
  moved, matching the original `T023b` human-in-the-loop discipline.
- **2026-08-12 default reversal (`009-top-level-indent-toggle`): top-level
  indentation reverts to left-untouched by default; `008`'s always-normalize
  behavior becomes opt-in via `--top-level-indent=normalize` (FR-026).** Not
  new evidence against `008`'s own reasoning — the project simply decided
  predictability-by-default was the wrong trade for users who never asked for
  it, while retaining it fully for users who do want it. Fix: `plan_indentation`
  gained a `TopLevelIndentMode` parameter (`Preserve`/`Normalize`,
  `Preserve` the new `#[default]`); the single line `008` added
  (`plan.insert(node.span().start.line, 0)`) became conditional on
  `mode == Normalize` — no other line of `plan_indentation`, and no line at
  all of `plan_block`/`plan_children`/`computed_indent`, changed. `007`'s
  skip needed no rationale change this time (unlike `008`, which had to
  re-derive it): under `Preserve` it behaves exactly as it did pre-`008`
  (protects a diagnosed block's children; the opener is also untouched, but
  because nothing forces it under `Preserve`, not because the skip protects
  it); under `Normalize` it behaves exactly as `008` already verified.
  Default-placement correctness — the exact class of bug
  (`pair_keyword_boundaries`, `structural_query_parity`) that has bitten this
  codebase before, a setting correct at one call site but silently stale at
  another — was verified individually, not transitively, at all four
  integration points: the CLI flag's own `clap` default, `FormatOptions::
  default()`, and both `drut-lsp` `FormatOptions::default()` call sites
  (`formatting.rs`, `range_formatting.rs`, neither compiler-forced, each
  given its own dedicated test since nothing else would catch a regression
  there); `drut-mcp`'s own struct-literal call site was compiler-forced to
  set the field explicitly by Rust's own struct-literal exhaustiveness (no
  `..Default::default()` spread was used there), converting what could have
  been a silent-miss risk into a build error. `008`'s original `normalize`
  behavior was independently re-proven byte-identical against its own
  already-committed, already-human-reviewed golden output (copied verbatim
  into a new `golden_normalize/` fixture set before any `preserve`-mode
  regeneration touched the original `golden/` directory) — no second
  human-review pass needed, since the expected content itself never changed.
  Golden-fixture impact for the `preserve` default: the same 7
  `real_corpus/` fixtures `008` changed revert toward their pre-`008`
  content; zero hand-written `valid/` fixtures affected either direction.
  Every regenerated golden file was individually diff-reviewed before being
  committed, same `T023b` discipline.
- **FR-013(b)/FR-025's encoding-safety split was a deliberate, considered choice
  between two options**, not the only way to resolve the conflict between FR-013's
  "never alter meaningful content" guarantee and FR-034's decode fallback: (a) treat
  every FR-034 fallback occurrence — recovered or lossy — as un-writable, which
  would keep `format --write` fully behavior-preserving with zero exceptions but
  would exclude real legacy files (the exact ones FR-034 exists to handle) from
  `format --write` entirely over a single incidental byte, and would need a fourth
  exit-code case; (b) write through both fallback outcomes uniformly, which keeps
  `format --write` universally available but persists a genuinely lossy
  substitution (not a faithful re-encoding) for the `InvalidEncoding` case
  specifically. The adopted design splits by fallback outcome instead of choosing
  one option for both: recovered bytes (lossless, just re-encoded) follow option
  (b)'s reasoning; genuinely undecodable bytes (lossy by FR-034's own definition)
  follow option (a)'s caution — and the refusal folds into the existing Fatal/
  exit-2 outcome (FR-020(c)) rather than becoming a genuine fourth case, so neither
  option's downside (a permanently-unformattable file class, or a brand-new
  exit-code state) was accepted wholesale.
