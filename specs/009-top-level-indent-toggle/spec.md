# Feature Specification: Top-Level Indent Default Revert

**Feature Branch**: `009-top-level-indent-toggle`

**Created**: 2026-08-11

**Status**: Draft

**Input**: User description: "Amend FR-012 (002-cli-check-format/spec.md, most recently amended by 008-top-level-indentation-normalization) a second time: revert the DEFAULT top-level (depth-0) indentation behavior back to 007-era leave-untouched/preserve, and make 008's unconditional column-0 normalization an opt-in toggle instead of the default. Mechanism: a CLI flag (--top-level-indent=preserve|normalize), not TOML config — mirrors --casing's shape exactly. The preserve default must be independently verified at the CLI flag's own default, voyager_core::FormatOptions::default(), and every FormatOptions call site in drut-lsp/drut-mcp that doesn't explicitly pass the new field. Regenerate format_corpus.rs's golden fixtures back to preserve-mode, using the same T023b-style human-reviewed-diff discipline. Add coverage proving explicit --top-level-indent=normalize still reproduces 008's original column-0-forcing behavior exactly, including the PROCESS/RUN residue tests. Out of scope: TOML config itself; any change to --casing's own flag shape."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Default formatting leaves top-level indentation exactly as written (Priority: P1)

A script author runs `drut format` with no extra flags — the same command
they've always run. Every top-level (depth-0) statement or block opener
keeps whatever indentation it already had, instead of being forced to
column 0. This restores the original `007`-era default that `008` replaced.

**Why this priority**: This is the entire policy reversal — the single
behavioral change every other part of this feature (the opt-in flag, the
regenerated goldens) exists to support or verify.

**Independent Test**: Format a script with a top-level `RUN`/`PROCESS`/bare
statement line sitting at a non-zero column, using no flags, and confirm
the result is byte-identical to the input for that line (and every other
top-level line) — no column-0 forcing.

**Acceptance Scenarios**:

1. **Given** a script where a top-level statement sits at a non-zero
   column, **When** the script is formatted with no flags, **Then** that
   statement's leading whitespace is left completely untouched.
2. **Given** a script that is already fully `008`-normalized (every
   top-level line at column 0), **When** the script is formatted with no
   flags, **Then** nothing changes (already-column-0 is a valid value for
   "left untouched," so this is a no-op either way).
3. **Given** a script with mixed top-level indentation (some lines at
   column 0, some not), **When** the script is formatted with no flags,
   **Then** every top-level line's existing column is individually
   preserved — no line is forced toward any other line's value.

---

### User Story 2 - `008`'s behavior remains available, opt-in (Priority: P1)

A user who wants Python-style predictability (every top-level line at
column 0, unconditionally) explicitly requests it with
`--top-level-indent=normalize`. The output is identical to what `008`
always produced — nothing about that behavior was removed, only its
default status.

**Why this priority**: Equal weight to User Story 1 — reverting the
default is only a safe change if the alternative it displaces stays fully
available and behaviorally unchanged for the users who want it.

**Independent Test**: Format the same fixture two ways — once with
`--top-level-indent=normalize`, once against a saved pre-`009` (008-era)
golden output for that fixture — and confirm the two are byte-identical.

**Acceptance Scenarios**:

1. **Given** a script with non-zero top-level indentation, **When**
   formatted with `--top-level-indent=normalize`, **Then** every top-level
   line is corrected to column 0, exactly as `008` specified.
2. **Given** the exact `PROCESS`/`RUN` residue sequence `008` was built to
   resolve (unclosed `PROCESS` swallows a trailing `RUN`; format once;
   add `ENDPROCESS`; format again), **When** both format passes use
   `--top-level-indent=normalize`, **Then** `RUN` lands correctly at
   column 0 after the second pass alone, same as `008` guaranteed.

---

### User Story 3 - The default is the same everywhere a format request can originate (Priority: P1)

A user formatting a file through the CLI, through VS Code's format-on-save
(LSP), or through an MCP-connected tool all see the identical default
(`preserve`) — no integration point silently keeps `008`'s old
unconditional-normalize behavior because its own call site was missed
when the default was threaded through.

**Why this priority**: Named explicitly because this exact class of bug —
a setting correct at one call site but silently stale at another — has
already caused two real defects in this codebase (`pair_keyword_boundaries`,
`structural_query_parity`). A reverted default that only takes effect on
the CLI is not actually a completed revert.

**Independent Test**: With no explicit flag/option passed by the caller,
call `voyager_core::format` directly, then every LSP handler that formats
a document, then the MCP `format` tool — confirm all three treat a
non-zero top-level indentation identically (left untouched), using the
same underlying default rather than three independently-set values that
happen to currently agree.

**Acceptance Scenarios**:

1. **Given** `voyager_core::FormatOptions::default()` with no fields
   explicitly set, **When** used to format a script with non-zero
   top-level indentation, **Then** that indentation is left untouched.
2. **Given** a document opened in the LSP server and formatted via
   `textDocument/formatting` or `textDocument/rangeFormatting` with no
   client-side override, **When** it contains non-zero top-level
   indentation, **Then** that indentation is left untouched.
3. **Given** the MCP server's `format` tool invoked with no options,
   **When** the target script has non-zero top-level indentation,
   **Then** that indentation is left untouched.

---

### Edge Cases

- What happens to `007-formatter-diagnosed-block-indent-fix`'s
  skip-indentation-planning-for-a-diagnosed-block's-children behavior
  under the reverted `preserve` default? Under `preserve`, a top-level
  line (diagnosed or not) is left untouched regardless, same as
  `007`-era behavior before `008` existed — `007`'s skip continues to
  matter for a diagnosed block's *children*, unchanged from `007`'s
  original scope. Under explicit `normalize`, `008`'s own resolution
  (opener always corrected, children still protected while diagnosed)
  continues to apply unchanged.
- What happens when a file previously written to disk by `--write
  --top-level-indent=normalize` (or by `008`-era `drut`, before this
  feature shipped) is later formatted again with no flags (the new
  `preserve` default)? A no-op — every top-level line is already at
  whatever column it's at (column 0, in this case), and `preserve` never
  moves a line away from its current column, so re-running with the new
  default cannot undo a prior `normalize` run or otherwise churn the
  file.
- What happens in a shared repository where some contributors run
  `--write` with `--top-level-indent=normalize` and others run plain
  `--write` (the new `preserve` default)? The two modes can disagree on
  a given file's top-level indentation, so alternating between them
  across commits can produce visible whitespace churn — this is an
  inherent, expected consequence of the setting being a per-invocation
  choice rather than project-wide state (project-wide configuration is
  explicitly out of scope, see Assumptions), not a defect this feature
  needs to prevent.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `format`'s DEFAULT top-level (depth-0) indentation behavior
  MUST revert to leaving existing top-level indentation untouched — the
  original `007`-era rule — superseding `008-top-level-indentation-
  normalization`'s unconditional-column-0 default. `008`'s own rule text
  and rationale MUST remain in the spec as historical record, not be
  deleted, per the same amendment discipline `008` itself used when it
  superseded `007`'s original FR-012.
- **FR-002**: `format` MUST support an opt-in flag choosing between
  `preserve` (the FR-001 default) and `normalize` (`008`'s original
  unconditional column-0 behavior), mirroring `FR-015`'s `--casing` flag
  shape: a named CLI option requiring an explicit value when given, no
  bare flag with an implied value.
- **FR-003**: When `normalize` is explicitly selected, top-level
  indentation behavior MUST be identical, byte-for-byte, to `008`'s
  original unconditional rule — including its interaction with `007`'s
  diagnosed-block-children skip (`008`'s FR-004 resolution). `008`'s
  behavior is being made non-default, not removed or altered.
- **FR-004**: The `preserve` default MUST be independently confirmed in
  effect at each of the following, individually — never assumed to hold
  transitively from any one of them: (a) the CLI flag's own default value
  when the flag is omitted; (b) `voyager_core::FormatOptions::default()`'s
  value for the new field; (c) every call site in `drut-lsp` and
  `drut-mcp` that constructs a `FormatOptions` value without explicitly
  setting this field.
- **FR-005**: Every currently-committed golden fixture affected by `008`'s
  column-0 forcing (`format_corpus.rs`'s `real_corpus/` set and any
  hand-written `valid/` fixture) MUST be regenerated to reflect `preserve`
  as the default output, individually diff-reviewed before being
  committed as new expected output — each diff confirmed to change *only*
  top-level indentation (reverting it), nothing else moved, reordered, or
  altered — the same `T023b` human-in-the-loop discipline used for every
  prior golden regeneration.
- **FR-006**: Automated coverage MUST prove explicit
  `--top-level-indent=normalize` reproduces `008`'s original behavior
  exactly, including `008`'s own `format_sequence.rs` regression tests
  (the `PROCESS`/`RUN` residue-resolves-in-one-pass scenarios) — those
  tests MUST keep passing when run under explicit `normalize` mode, not
  be deleted or left only covering the now-default `preserve` path.
- **FR-007**: `002-cli-check-format/spec.md`'s FR-012 MUST be amended a
  second time: a new dated entry documenting this reversal-of-the-
  reversal and the new default/opt-in mechanism, added alongside (not
  replacing) `008`'s own dated entry — preserving the full decision
  history the same way `008`'s entry preserved `007`'s original rationale.

### Key Entities

- **Top-level indent mode**: A two-valued setting (`preserve` /
  `normalize`) governing whether `format` leaves top-level (depth-0)
  indentation untouched or unconditionally corrects it to column 0.
  Carried through `voyager_core::FormatOptions`, surfaced as a CLI flag
  on `drut format`, defaulting to `preserve` everywhere it is read.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user running `drut format` with no extra flags sees every
  top-level statement's indentation exactly as they wrote it — zero
  forced column-0 corrections — matching `007`-era behavior.
- **SC-002**: A user running `drut format --top-level-indent=normalize`
  sees output byte-identical to `008`'s original behavior, with zero
  regressions against `008`'s own test coverage.
- **SC-003**: The `preserve` default is independently verified — via a
  dedicated, individually-attributable check per integration point, not
  inferred from any single shared code path — to be in effect at the CLI
  flag's own default, the core library's own default, and every LSP/MCP
  call site.
- **SC-004**: Every regenerated golden fixture has been individually
  reviewed and confirmed to change *only* top-level indentation
  (reverting to `preserve`'s output), with zero unintended content,
  structure, or diagnostic changes.
- **SC-005**: The full 161-file real corpus remains 100% clean (zero
  diagnostics of any kind) after the change — a purely whitespace-shifting
  reversion, not a structural or diagnostic one.

## Assumptions

- Mechanism is a CLI flag, not project-level configuration. TOML-based
  configuration (pre-publish item 3) was explicitly considered and
  rejected as a blocker for this feature — it remains unscheduled and
  out of scope here. When it eventually lands, it is expected to expose
  this same setting through a config file in addition to (not instead
  of) the flag; that integration work is deferred to that future feature.
- No LSP- or MCP-side user-facing toggle is in scope. `drut-lsp` and
  `drut-mcp` call `voyager_core::format` without explicitly setting the
  new field, so they pick up whatever `FormatOptions::default()` resolves
  to (`preserve`) — same pattern `--casing` already uses for those two
  adapters (neither exposes a casing toggle either, per `002`'s own
  Assumptions).
- `--casing`'s own flag shape and behavior are unchanged — this feature
  only reuses that flag's shape as a structural precedent for
  `--top-level-indent`, per `crates/drut-cli/src/cli.rs`'s existing
  `CasingArg`/`Command::Format` pattern.
- This is a policy reversal, not new evidence contradicting `008`'s own
  reasoning — `008`'s rationale (predictability can be worth trading
  away real-author diversity) remains valid for users who opt into
  `normalize`; the project has simply decided it should not be the
  silent default.
- Golden-fixture regeneration and its human-reviewed-diff discipline
  (FR-005) is part of this feature's own Definition of Done, not a
  follow-up task — the same standing project gate `008` itself applied.
