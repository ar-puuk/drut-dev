# Feature Specification: Blank-Line-Run Normalization

**Feature Branch**: `019-blank-line-normalization`

**Created**: 2026-08-17

**Status**: Draft

**Input**: User description: "Blank-line-run normalization: a new `[format]` setting with two
modes only (`preserve` default/no-op, `auto`) covering runs of consecutive blank lines (a blank
line includes a whitespace-only line, not just a strictly zero-length one). Two independently-
configurable positive-integer caps, not one: a top-level cap (default 2) and a nested cap
covering any line inside any block regardless of depth, uniformly, not scaling further per
nesting level (default 1) — mirroring `top_level_indent`'s existing top-level-vs-everything-else
split. `auto` only contracts a run of consecutive blank lines down to the applicable cap when it
exceeds that cap — never pads a shorter run up. Both caps validate the same way `indent_width`
already does. `; FMT: OFF`/`; FMT: ON` regions are left untouched. Exposed identically via
`drut.toml`, CLI, and MCP `format` tool." Full design history — the mode/cap-shape decisions and
the industry-precedent check (Python's `black`/JS's `prettier` both cap consecutive blank
lines) — lives in `ROADMAP.md` item 13.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A project caps runaway blank-line runs between top-level statements (Priority: P1)

A team's `.s`/`.block` scripts accumulate long stretches of blank lines over years of editing —
five, ten, or more blank lines in a row between top-level `RUN`/`IF`/plain-statement blocks, left
over from deleted content or careless pasting. The team wants these capped to a sane, consistent
maximum without hand-deleting them file by file.

**Why this priority**: This is the directly reported annoyance and the most visible case — long
gaps between top-level blocks are what a reader actually scrolls past. Useful entirely on its
own, independent of whether nested gaps are ever addressed.

**Independent Test**: With the top-level cap left at its default (2) and `auto` enabled, format a
script containing a run of 5 blank lines between two top-level blocks and confirm exactly 2
remain; confirm a run of 1 or 2 blank lines elsewhere in the same file is left untouched.

**Acceptance Scenarios**:

1. **Given** `auto` mode with the default top-level cap, **When** a script has 5 consecutive
   blank lines between two top-level statements, **Then** exactly 2 blank lines remain there.
2. **Given** the same configuration, **When** a script has exactly 2 (or fewer) consecutive
   blank lines anywhere at the top level, **Then** that run is left completely untouched.
3. **Given** no `blank_lines`-family configuration at all, **When** any script is formatted,
   **Then** no blank-line run is touched, however long — `preserve` remains the default,
   matching every other formatting axis already shipped.

---

### User Story 2 - A project independently caps blank-line runs inside a block's body (Priority: P2)

Alongside top-level gaps, a team's scripts also accumulate excessive blank-line runs *inside* a
block's own body — between statements nested inside `RUN`/`IF`/`LOOP`/etc. The team wants a
tighter, independently-configurable cap there, since a nested body reads better with less
vertical breathing room than the gaps between whole top-level blocks.

**Why this priority**: Independently valuable and independently shippable on top of User Story
1 — a team could want just the top-level cap, or set the two caps to different values entirely
(the reported motivation for two separate settings rather than one).

**Independent Test**: With the nested cap left at its default (1) and `auto` enabled, format a
script containing a run of 4 blank lines inside a block's body (at any nesting depth) and confirm
exactly 1 remains, independent of whatever the top-level cap does elsewhere in the same file.

**Acceptance Scenarios**:

1. **Given** `auto` mode with the default nested cap, **When** a block's body has 4 consecutive
   blank lines between two of its child statements, **Then** exactly 1 blank line remains there.
2. **Given** the same configuration, **When** a doubly-nested block (e.g. a `LOOP` inside a
   `RUN`) has an excessive blank-line run in its own body, **Then** the same nested cap applies
   there too — a deeper level does not get its own, further-reduced cap.
3. **Given** the top-level cap and the nested cap set to two different values, **When** a script
   has excessive runs both between top-level blocks and inside one block's body, **Then** each
   run is capped independently, according to whichever cap actually applies to its own position.

---

### Edge Cases

- What happens to a whitespace-only line (spaces/tabs, no visible content) within a run of blank
  lines? It counts as blank for run-length purposes, exactly like a strictly zero-length line —
  a run mixing zero-length and whitespace-only lines is measured as one continuous run, not
  broken by the distinction.
- When a run is contracted, which specific lines survive? The first N lines of the run (N being
  whichever cap applies), left exactly as they originally were (whitespace-only lines are not
  additionally trimmed to zero-length) — only the excess *lines* are removed entirely, not their
  content altered.
- What happens to an excessive blank-line run at the very start or end of a file? The same rule
  applies uniformly — no special-casing for file boundaries. This includes the degenerate case
  of a file containing nothing but blank lines (no statements or blocks at all): the top-level
  cap applies, the same way it would to any run not enclosed by a block.
- What happens inside a `; FMT: OFF`/`; FMT: ON` protected region? Left untouched, exactly as
  every other formatting rule already respects that marker. A run can never be *partially*
  protected — the `; FMT: OFF`/`; FMT: ON` marker lines themselves are never blank, so (like a
  block boundary) they always break a run rather than sitting inside one; a run is therefore
  always either entirely inside a protected region or entirely outside one.
- What happens when a project's configuration sets a cap to an unrecognized or invalid value
  (non-integer, zero, or unreasonably large)? Falls back to that cap's own built-in default with
  a non-blocking notice, matching the established pattern for every other malformed `drut.toml`
  field in this project.
- What determines whether a given blank line is "top-level" or "nested"? Its position relative
  to block structure, not its raw line number or textual indentation — a blank line sitting
  between two top-level statements/blocks is top-level; a blank line sitting anywhere inside a
  block's own body (between its opener and closer, or between its child statements, at any
  nesting depth) is nested. This includes a run sitting immediately after a block's own opener
  line, before its first child statement — still nested, since it's already inside the block's
  span at that point. A run sitting immediately *before* a block's opener line is never
  considered part of that block (the opener line itself is never blank, so it always breaks the
  run) — such a run's own classification is governed by whatever *encloses it*, not by the block
  that's about to start.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST allow a project to configure blank-line-run normalization as one
  of `preserve` or `auto`, defaulting to `preserve` when not explicitly configured.
- **FR-002**: The system MUST allow a project to independently configure two positive-integer
  caps: one governing blank-line runs between top-level statements/blocks (default `2`), and one
  governing blank-line runs inside any block's own body, uniformly regardless of nesting depth
  (default `1`).
- **FR-003**: `auto` MUST contract a run of consecutive blank lines to the applicable cap only
  when that run's length exceeds the cap — a run already at or below the applicable cap MUST be
  left completely untouched.
- **FR-004**: `auto` MUST NOT pad a blank-line run that is shorter than the applicable cap up to
  meet it — this is a maximum, never a minimum.
- **FR-005**: A whitespace-only line (containing only spaces and/or tabs) MUST be treated as
  blank for run-detection purposes, indistinguishable from a strictly zero-length line.
- **FR-006**: When a run is contracted, the surviving lines MUST be the first N lines of the
  original run (N being the applicable cap), left byte-for-byte as originally written — only the
  excess lines are removed; no surviving line's own content is altered.
- **FR-007**: Whether a given blank-line run is subject to the top-level cap or the nested cap
  MUST be determined by its structural position (between top-level nodes, vs. inside any block's
  body at any depth), never by raw line number or textual indentation alone.
- **FR-008**: A deeper level of nesting MUST NOT receive a further-reduced cap of its own — the
  single nested cap applies uniformly to every depth greater than zero.
- **FR-009**: `preserve` (the default) MUST leave every blank-line run exactly as written,
  regardless of length — a project with no blank-line-run configuration MUST produce
  byte-identical output to before this feature existed.
- **FR-010**: Every existing formatting guarantee (idempotence, behavior preservation, respect
  for `; FMT: OFF`/`; FMT: ON` regions, never altering non-blank line content) MUST continue to
  hold under `auto`.
- **FR-011**: An unrecognized or invalid cap value in a project's `drut.toml` (non-integer, out
  of range, etc.) MUST NOT fail formatting — the system MUST fall back to that cap's own built-in
  default and surface a non-blocking notice, consistent with how every other malformed
  `[format]` value in this project already degrades.
- **FR-012**: Every surface that already exposes formatting configuration today (the
  command-line tool, the language server's format-on-save/format-on-paste, and the MCP format
  tool) MUST expose both new caps and the mode setting identically — no surface silently lagging
  or disagreeing with another.

### Key Entities

- **Blank-line-run mode**: A project-wide setting — `preserve` or `auto` — controlling whether
  excessive blank-line runs are contracted at all.
- **Top-level cap**: The maximum number of consecutive blank lines `auto` allows between
  top-level statements/blocks before contracting the run, default `2`.
- **Nested cap**: The maximum number of consecutive blank lines `auto` allows inside any block's
  own body (any nesting depth greater than zero) before contracting the run, default `1`,
  independent of the top-level cap and not further reduced at deeper nesting.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can enable `auto` mode and see an excessive top-level blank-line run in a
  real corpus-shaped script contracted to exactly the configured top-level cap, in a single pass.
- **SC-002**: A user can enable `auto` mode and see an excessive blank-line run inside a block's
  body, at any nesting depth, contracted to exactly the configured nested cap, independent of
  the top-level cap's own value.
- **SC-003**: A script processed with no blank-line-run configuration at all is byte-identical
  before and after this feature ships, verified across the full real fixture corpus.
- **SC-004**: An invalid cap value never stops formatting from completing — it degrades to that
  cap's built-in default with a notice, every time, on every surface where such a degradation is
  possible (the command-line and MCP surfaces reject an invalid value outright at their own
  input point, matching every other closed-set/bounded formatting setting already shipped).
- **SC-005**: `auto` blank-line-run contraction is idempotent — running it twice in a row on the
  same script produces no further change on the second pass — verified across the full real
  fixture corpus.

## Assumptions

- The exact valid range for each cap (a sane upper bound, not unlimited) is a planning-phase
  detail, not fixed by this spec — the binding requirement is that an out-of-range or malformed
  value degrades non-fatally, the same standard `indent_width` already established.
- The exact `drut.toml`/CLI flag/MCP parameter names for the mode and the two caps are a
  planning-phase decision, not fixed by this spec. The binding requirement is a two-value mode
  setting plus two independent positive-integer caps, default `preserve`/`2`/`1`, additive to
  every existing configuration surface — never a breaking change to already-shipped formatting
  behavior.
- No opinionated preset ships with this feature — `auto`'s behavior is fully determined by the
  two caps' own configured (or default) values, never a hidden house-style choice layered on
  top, consistent with every other formatting axis's own no-preset stance in this project.
- This feature is purely about blank-line *count*, never about blank-line *placement* (e.g.
  inserting a blank line before/after a block that doesn't already have one) — that is a
  meaningfully different feature, out of scope here.
