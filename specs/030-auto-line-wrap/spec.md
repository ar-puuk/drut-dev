# Feature Specification: Automatic Line-Width Wrapping

**Feature Branch**: `030-auto-line-wrap`

**Created**: 2026-08-19

**Status**: Draft

**Input**: User description: "auto line width... wrapping codes with syntactically valid
alternative" — scoped down through direct conversation to `Control` statements' comma-separated
`keyword=value` pair lists specifically, using Cube Voyager's existing line-continuation
mechanism (the same trailing-comma/operator character the parser already treats as "this
statement continues onto the next physical line," `001-voyager-script-parser` FR-006) rather
than inventing new syntax. An `Assignment` statement's arithmetic-expression continuation
characters, and splitting inside a function call's parentheses or a bracketed subscript, are
explicitly out of scope for this first increment.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A project keeps long `Control` statements within a readable width (Priority: P1)

A team's `.s`/`.block` scripts accumulate `Control` statements with many `keyword=value` pairs
on one line (`RUN PGM=MATRIX, ZONES=5, PRINT=1, MSG='...', ...`) that run well past a
comfortable editing width — hard to read on a normal monitor, awkward in a side-by-side diff.
The team wants these automatically wrapped across multiple physical lines using valid Cube
Voyager continuation syntax, without hand-editing every occurrence and without the tool
silently changing what the script actually does.

**Why this priority**: This is the entire feature — there is no secondary priority tier for
this first increment, matching how `018-operator-spacing`'s own single most-common-case scoping
worked. It's independently valuable and independently shippable.

**Independent Test**: With line wrapping configured to `auto` and a width, format a script
containing one `Control` statement whose single-line length exceeds the configured width and
one that doesn't — confirm only the over-width statement is wrapped across multiple physical
lines via a trailing comma, each resulting line at or under the configured width, and the
under-width statement is left exactly as written.

**Acceptance Scenarios**:

1. **Given** line wrapping configured to `auto` with a width, **When** a `Control` statement's
   single physical line exceeds that width, **Then** the formatted output wraps it across
   multiple physical lines, splitting only immediately after a comma separating two
   `keyword=value` pairs, with each resulting line's own length at or under the configured
   width wherever a valid split point makes that achievable.
2. **Given** the same configuration, **When** a `Control` statement's single physical line is at
   or under the configured width, **Then** it is left completely untouched — wrapping only ever
   activates for a genuinely over-width statement.
3. **Given** no line-wrapping configuration at all, **When** the same over-width script is
   formatted, **Then** nothing wraps — `preserve` remains the default, matching every other
   formatting axis already shipped.
4. **Given** line wrapping configured to `auto`, **When** a `Control` statement already spans
   multiple physical lines via an existing, author-written continuation (regardless of whether
   any individual resulting line exceeds the configured width), **Then** it is left completely
   untouched — this feature never re-flows a statement that is already continued.
5. **Given** line wrapping configured to `auto`, **When** the same script is formatted twice in
   direct succession, **Then** the second pass produces no further change — the statements this
   feature wrapped on the first pass are now themselves already-continued (Acceptance Scenario
   4), so the feature leaves them alone on the second pass by the same rule, not a separately
   re-derived idempotence check.
6. **Given** line wrapping configured to `auto`, **When** an over-width `Assignment` statement
   (no comma-separated `Control`-statement pair list at all) or an over-width statement whose
   only comma appears inside a quoted string or a function call's parentheses is formatted,
   **Then** it is left completely untouched — this feature never invents a split point outside a
   top-level `Control`-statement pair-list comma.
7. **Given** line wrapping configured to `auto` with the default wrap style, **When** a
   `Control` statement with five short `keyword=value` pairs exceeds the configured width,
   **Then** each continuation line holds as many consecutive pairs as fit within the configured
   width, breaking only when the next pair would exceed it.
8. **Given** line wrapping configured to `auto` with the one-pair-per-line wrap style
   explicitly configured, **When** the same statement is formatted, **Then** each continuation
   line holds exactly one pair.

---

### Edge Cases

- What happens inside a `; FMT: OFF`/`; FMT: ON` protected region? Left untouched, exactly as
  every other formatting rule already respects that marker.
- What happens to a comma that appears inside a quoted string literal, or inside a function
  call's parentheses or a bracketed subscript within one pair's value? Never treated as a split
  point — only a comma at the statement's own top level, directly separating two
  `keyword=value` pairs, is eligible.
- What happens when a single `keyword=value` pair is itself longer than the configured width
  (e.g. one very long quoted value), with no comma available to split before it fits? The
  statement wraps at whatever top-level commas do exist; any individual resulting line that
  still exceeds the width because no further split point is available is left at that length —
  this feature never truncates or otherwise alters a value to force a fit.
- What happens to a `Control` statement with no comma-separated pairs at all (a single
  `keyword=value`, or no pairs)? Never wrapped — there is no eligible split point.
- What happens when a project's configuration sets line wrapping to an unrecognized or invalid
  value, or a non-positive/unreasonable width? Falls back to the `preserve` default (or that
  field's own built-in default) with a non-blocking notice, matching the established pattern
  for every other malformed `[format]` field in this project.
- What happens to indentation on a newly-wrapped continuation line? It receives this project's
  standard continuation-line indentation treatment (one level deeper than the statement's own
  opening line), matching how a hand-written continuation is already expected to read.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST allow a project to configure line wrapping as one of `preserve`
  or `auto`, defaulting to `preserve` when not explicitly configured — wrapping is opt-in only,
  never active by default.
- **FR-002**: The system MUST allow a project to configure a positive-integer maximum line
  width, used only when line wrapping is set to `auto`. When `auto` is enabled but no explicit
  width is configured anywhere, the system MUST fall back to a built-in default width of 120
  characters rather than requiring an explicit value.
- **FR-002a**: The system MUST allow a project to configure how pairs are distributed across
  continuation lines once wrapping activates: as many consecutive pairs as fit within the
  configured width before breaking ("fill", the built-in default), or exactly one
  `keyword=value` pair per continuation line. This setting has no effect unless line wrapping is
  set to `auto`. Fill is the default rather than one-per-line specifically because of FR-005:
  since an already-wrapped statement is never re-flowed by a later format pass, whichever style
  wraps a statement first is effectively permanent for it: further-splitting an already-packed
  fill line by hand later (if a user wants more one-per-line-style breaks for a specific
  statement) is a small, local, always-valid edit, while un-packing many one-per-line
  continuations back into fill form is the more tedious direction — so the default should be the
  one that's cheaper to manually diverge from.
- **FR-003**: `auto` MUST wrap a `Control` statement whose single physical line exceeds the
  configured width by inserting a line break immediately after a top-level comma separating two
  `keyword=value` pairs, distributed per the configured wrap style (FR-002a) — never inside a
  quoted string, never inside a function call's parentheses or a bracketed subscript within a
  pair's value.
- **FR-004**: `auto` MUST NOT wrap a statement that is not a `Control` statement, and MUST NOT
  wrap a `Control` statement with no eligible top-level comma.
- **FR-005**: `auto` MUST NOT wrap, re-flow, or otherwise alter any statement that already
  contains an author-written line continuation, regardless of that statement's width — a
  statement is either left completely alone by this feature (already continued) or is a v1
  wrapping candidate (currently single-line and over-width), never both.
- **FR-006**: A newly-wrapped continuation line MUST receive this project's standard
  continuation-line indentation (one level deeper than the statement's own opening line).
- **FR-007**: `preserve` (the default) MUST leave every statement's line structure exactly as
  written — a project with no line-wrapping configuration MUST produce byte-identical output to
  before this feature existed.
- **FR-008**: Every existing formatting guarantee (idempotence, behavior preservation, no
  reordering of statements, respect for `; FMT: OFF`/`; FMT: ON` regions, never altering values
  inside string/quoted literals) MUST continue to hold under `auto`.
- **FR-009**: An unrecognized or invalid line-wrapping value, or a non-positive/unreasonable
  width, in a project's `drut.toml` MUST NOT fail formatting — the system MUST fall back to that
  field's own built-in default and surface a non-blocking notice, consistent with how every
  other malformed `[format]` field in this project already degrades. At the command-line and MCP
  surfaces, an invalid value outside the accepted shape MUST be rejected with a clear
  usage/tool error at that surface's own input-validation point, matching how every other
  closed-set-or-bounded `[format]` field already behaves at both surfaces.
- **FR-010**: Every surface that already exposes formatting configuration today (the
  command-line tool, the language server's format-on-save/format-on-paste, and the MCP format
  tool) MUST expose the line-wrapping mode, the width control, and the wrap-style control
  identically — no surface silently lagging or disagreeing with another. This includes the
  existing personal-setting mechanism every other `[format]` field already has in the VS Code
  extension (`drut.format.*`, `021-editor-settings-config`'s precedent) — a project's committed
  `drut.toml` still wins over a personal setting for the same field, unchanged from how that
  precedence already works today.

### Key Entities

- **Line-wrapping mode**: A project-wide setting — `preserve` or `auto` — controlling whether an
  over-width `Control` statement's pair list is automatically wrapped across multiple physical
  lines.
- **Maximum line width**: A positive-integer companion setting, used only under `auto`, defining
  the width threshold that triggers wrapping and the target each resulting line is wrapped
  toward wherever a valid split point makes that achievable. Defaults to 120 characters when
  `auto` is enabled with no explicit width configured.
- **Wrap style**: A companion setting, used only under `auto`, controlling how pairs are
  distributed across continuation lines — fill (default: as many consecutive pairs as fit per
  line) or one pair per line.
- **Eligible split point**: A comma token at a `Control` statement's own top level, directly
  separating two `keyword=value` pairs — never one nested inside a quoted string, a function
  call's parentheses, or a bracketed subscript.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can enable `auto` mode (with or without an explicit width — the 120-character
  built-in default applies when none is set) and see every over-width `Control` statement in a
  real corpus-shaped script wrapped across multiple physical lines at valid top-level comma
  split points in a single pass, distributed per the configured (or default) wrap style.
- **SC-002**: A `Control` statement at or under the configured width, any non-`Control`
  statement, and any statement that already contains an author-written continuation are never
  altered by this feature, verified against real corpus-shaped script content covering all
  three cases.
- **SC-003**: A script processed with no line-wrapping configuration at all is byte-identical
  before and after this feature ships, verified across the full real fixture corpus.
- **SC-004**: `auto` line wrapping is idempotent — running it twice in a row on the same script
  produces no further change on the second pass — verified across the full real fixture corpus.
- **SC-005**: An invalid line-wrapping configuration never silently produces the wrong
  formatting result: a `drut.toml` value degrades to the built-in default with a non-blocking
  notice, every time; a command-line or MCP value outside the accepted shape is rejected with a
  clear usage/tool error at that surface's own input point, every time.

## Assumptions

- Scope is deliberately narrow for this first increment: only a `Control` statement's
  comma-separated `keyword=value` pair list is a wrapping candidate — the single most common
  real-world source of over-width lines in this domain, based on real corpus scripts already
  seen in this project's own development. Splitting inside an `Assignment` statement's
  arithmetic/string expression (its own `+ - / * ^ & |` continuation characters), inside a
  function call's parentheses, or inside a bracketed subscript is explicitly **out of scope**
  here — a plausible future increment, not attempted in this pass.
- A statement that already contains any line continuation is left completely untouched by this
  feature, never re-flowed — this is both the safest v1 boundary (never fighting hand-formatted
  content) and the mechanism that makes idempotence hold by construction: once this feature
  wraps a statement, that statement now contains a continuation, so a second pass sees
  "already continued" and leaves it alone (Acceptance Scenario 5).
- Width is measured as the formatted line's own character length (after every other configured
  formatting axis — casing, indentation, operator spacing — has already applied), the same
  plain definition of "line length" mainstream code formatters use.
- The exact configuration surface shape (configuration-file field name(s), command-line flag
  name(s), MCP parameter name(s)) is a planning-phase decision, not fixed by this spec. The
  binding requirement is a mode setting (default `preserve`) plus companion width and wrap-style
  settings, additive to every existing configuration surface — never a breaking change to
  already-shipped formatting behavior.

## Clarifications

### Session 2026-08-19

- Q1: If `auto` mode is enabled but no explicit width value is configured anywhere, should the
  width setting have a sensible built-in default, or should enabling `auto` require an explicit
  width with no built-in fallback at all? → **A: wrapping itself stays opt-in (`preserve`
  default, never active unless configured); once opted in, the width setting defaults to 120
  characters when not explicitly set.** Both the mode and the width are configurable via
  `drut.toml` and via the VS Code extension's personal settings (FR-001, FR-002, FR-010).
- Q2: Once a `Control` statement exceeds the configured width, should wrapping place exactly one
  `keyword=value` pair per continuation line, or greedily pack as many pairs as fit within the
  width budget per line before breaking? → **A: made a third configurable setting** (wrap
  style), rather than picking one — consistent with this project's existing precedent for
  multi-field formatting settings (`blank_lines`' mode-plus-two-numeric-caps shape). Defaults to
  fill (greedy packing); one pair per line is an explicit opt-in alternative (FR-002a). Fill was
  chosen as the default over one-per-line specifically because of the FR-005/never-re-flow
  interaction: whichever style wraps a statement first is effectively permanent for it, and
  further-splitting an already-packed fill line by hand is a smaller, safer, always-valid edit
  than manually un-packing many one-per-line continuations back into fill form — so the default
  should be the direction that's cheaper to manually diverge from later.
