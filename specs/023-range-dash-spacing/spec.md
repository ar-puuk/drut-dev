# Feature Specification: Range-Dash Spacing Exemption

**Feature Branch**: `023-range-dash-spacing`

**Created**: 2026-08-18

**Status**: Draft

**Input**: User description: "Range-dash spacing exemption in operator_spacing: when
`operator_spacing` is `fixed` or `auto`, a `-` that joins two bare integer literals inside a
pair-keyword's value (e.g. `FILEO SELECTLINK=1-50,75,90-100`, `FILEO NODES=200-300`) is Cube Voyager's
inclusive-range list notation, not arithmetic subtraction, and must never get spaced apart the
way binary `-` currently does uniformly. Fix: scope the existing binary-arithmetic recognition
so that inside a pair-keyword value's position, any `-` directly touching bare integer tokens on
both sides is instead actively normalized to zero surrounding whitespace (stripping any existing
spaces, e.g. `1 - 50` -> `1-50`), never spaced like arithmetic subtraction. Everywhere else
(Assignment RHS, IF/short-IF conditions, LOOP bounds, etc.) `-` keeps today's binary-arithmetic
spacing behavior unchanged. `preserve` mode is unaffected either way." A follow-up/amendment to
`018-operator-spacing` — no new `[format]` config field, no CLI/MCP/editor-setting surface
change; full design discussion (industry-precedent comparison against Black's context-sensitive
slice-colon spacing and CSS `calc()`'s mandatory-spacing disambiguation) lives in the parent
conversation, not restated here.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A range-list value keeps its conventional tight notation (Priority: P1)

A project has `operator_spacing` set to `fixed` (or `auto`) and its scripts contain `Control`
statements whose pair-keyword values are Cube Voyager's inclusive-range list notation — a
comma-separated list of single IDs and/or `N-M` ranges, such as `FILEO SELECTLINK=1-50,75,90-100` or
`FILEO NODES=200-300`. Today, `fixed`/`auto` treats every `-` uniformly as binary arithmetic and spaces
it apart (`1 - 50`), which changes the script's conventional, recognizable range notation into
something that reads like a subtraction expression. The team wants `fixed`/`auto` to instead
render every such range tight (`1-50`), the same as page-range, ID-range, and time-range notation
conventionally renders everywhere else, regardless of how the range's spacing was originally
written.

**Why this priority**: This is the entire scope of the feature — there is no independently
smaller or larger slice. A project cannot adopt `018-operator-spacing`'s `fixed`/`auto` modes on
a script containing real range-list values without this fix producing an incorrect, meaning-
obscuring result today.

**Independent Test**: With `operator_spacing` configured to `fixed`, format a script containing a
pair-keyword value with a range written tight (`1-50`), one written with spaces on both sides
(`1 - 50`), and one written with a space on only one side (`1- 50` or `1 -50`) — confirm all three
render tight (`1-50`) in the output, and that a comma-separated list mixing single IDs and ranges
(`1-50,75,90-100`) renders every range within it tight independently.

**Acceptance Scenarios**:

1. **Given** `operator_spacing` configured to `fixed`, **When** a script contains
   `FILEO SELECTLINK=1-50,75,90-100`, **Then** the formatted output renders
   `FILEO SELECTLINK = 1-50,75,90-100` — `SELECTLINK`'s own `=` gets `018-operator-spacing`'s
   ordinary, unrelated one-space treatment (as it always has), while both ranges stay tight and
   the commas inside this single pair's own value list are untouched (they're outside
   `018`'s comma-spacing rule entirely — that rule only ever touches a comma separating two
   *different* pairs, `018` FR-004's existing scope).
2. **Given** the same configuration, **When** a script contains `FILEO NODES=200 - 300`, **Then** the
   formatted output renders `FILEO NODES = 200-300` — the range's own spacing is actively
   stripped, not merely left alone.
3. **Given** the same configuration, **When** a script contains `X = 100-1` (an `Assignment`
   statement, not a pair-keyword value), **Then** the formatted output renders `X = 100 - 1` —
   today's binary-arithmetic spacing, unchanged, because the `-` does not sit inside a
   pair-keyword's value.
4. **Given** the same configuration, **When** a script contains `IF (COUNT-1 == 0)`, **Then** the
   formatted output renders `IF(COUNT - 1 == 0)` — unchanged, because a condition is not a
   pair-keyword value either.
5. **Given** `operator_spacing` left at its default (`preserve`), **When** any of the scripts
   above is formatted, **Then** nothing changes — this feature alters no behavior under
   `preserve`.
6. **Given** `operator_spacing` configured to `fixed`, **When** a script contains two pairs on
   one statement whose shared boundary comma is misspaced and whose values are each an
   unspaced-inconsistent range — `FILEO NODES=1-50 ,SELECTLINK=75 - 100` — **Then** the formatted
   output renders `FILEO NODES = 1-50, SELECTLINK = 75-100`: both pairs' own `=` get `018`'s
   ordinary spacing, the pair-boundary comma is normalized by `018`'s existing comma rule (space
   removed before it, one space inserted after it), and both ranges render tight, all in the
   same pass — every rule applies to its own disjoint gaps without interfering with another.

---

### Edge Cases

- What happens to a range whose left or right side is not a bare integer literal — a `@token@`
  reference, a decimal number, or another identifier (e.g. `FILEO SELECTLINK=@START@-50`)? It is not
  recognized as a range; the `-` keeps today's binary-arithmetic spacing (spaced, one side or
  both) even though it sits inside a pair-keyword value — the exemption only ever applies when
  both sides are bare integer literals.
- What happens to a leading negative number in a range-list value, such as `FILEO OFFSET=-100,50`? The
  `-` there is unary (nothing precedes it in the value, the same rule `018-operator-spacing`
  already uses to distinguish unary from binary), so it was never spaced apart in the first
  place and this feature does not change it.
- What happens when a pair-keyword's value legitimately intends subtraction between two literal
  integers rather than a range (e.g. a hypothetical `FILEO FACTOR=100-1`)? It renders tight, the
  same as a real range would — pair-keyword values in Cube Voyager are conventionally literal
  values or lists, not computed expressions, so this is treated as an accepted, corpus-informed
  trade-off rather than a case this feature must distinguish.
- What happens inside a `; FMT: OFF`/`; FMT: ON` protected region? Left untouched, exactly as
  every other `018-operator-spacing` rule already respects that marker.
- What happens to a range-shaped `-` inside a string/quoted literal (e.g. `LIST='1-50'`)? Never
  touched, matching how every other operator-spacing rule already excludes quoted content.
- What happens to a range at the very start or end of a pair-keyword's value list, versus one
  in the middle (`1-50,75,90-100`)? Each `-` is evaluated independently against its own
  immediately adjacent tokens — position within the list has no bearing on the result.
- What happens to a pair-keyword value containing a decimal-number range, such as
  `FILEO THRESHOLD=1.5-2.5`? Not recognized as a range (this feature is scoped to bare integer
  literals only, per the input description) — the `-` keeps today's binary-arithmetic spacing.
- What happens to the commas *inside* a single pair's own comma-separated range-list value (e.g.
  the three commas in `1-50,75,90-100`)? They are outside `018-operator-spacing`'s comma-spacing
  rule entirely — that rule only ever touches a comma separating two *different* pairs on one
  statement, never a comma inside one pair's own value (the same existing behavior that already
  leaves `LOOP i=1,5,1`'s internal commas untouched). This feature does not change that; a
  same-pair list's own commas are simply never a candidate for either rule.
- What happens when a pair-boundary comma (the kind `018`'s comma rule *does* touch) sits
  immediately next to a range-dash value, e.g. `FILEO NODES=1-50 ,SELECTLINK=75 - 100`? Both rules
  apply independently to their own disjoint gaps in the same formatting pass — the comma
  normalizes per `018`'s existing rule, both ranges render tight, with neither rule affecting the
  other's own gap (Acceptance Scenario 6).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When `operator_spacing` is `fixed` or `auto`, a binary `-` that sits inside a
  `Control` statement's pair-keyword value and has a bare integer literal directly adjacent on
  both sides MUST be rendered with zero surrounding whitespace, regardless of how it was
  originally spaced (tight, spaced on one side, or spaced on both sides).
- **FR-002**: A bare integer literal, for the purpose of FR-001, means a token consisting only of
  decimal digits — not a `@token@` reference, not a decimal number, not an identifier, and not an
  expression. If either side adjacent to a `-` inside a pair-keyword value is not a bare integer
  literal, FR-001 does not apply and FR-004's unchanged behavior governs instead.
- **FR-003**: FR-001 MUST apply independently to every qualifying `-` within a pair-keyword's
  value, including when the value is a comma-separated list mixing single values and ranges
  (e.g. `1-50,75,90-100`) — each range in the list is evaluated and normalized on its own.
- **FR-004**: A `-` that does not sit inside a pair-keyword's value (an `Assignment` statement's
  right-hand side, an `IF`/short-`IF` condition, a `LOOP` bound, or any other expression context)
  MUST keep the existing binary-arithmetic spacing behavior from `018-operator-spacing`,
  unchanged by this feature.
- **FR-005**: The existing unary-vs-binary distinction for `+`/`-` (`018-operator-spacing`
  FR-003) MUST continue to apply before FR-001 is even considered — a unary `-` (nothing
  precedes it, or the immediately preceding token is itself `=`, `(`, a comma, or another
  recognized operator) is never treated as a range dash, since it was never spaced apart as
  binary arithmetic in the first place.
- **FR-006**: `018-operator-spacing`'s comma-spacing rule MUST continue to apply unchanged,
  exactly to its existing scope (a comma separating two *different* pairs on one statement) — it
  MUST NOT be extended to reach a comma inside a single pair's own comma-separated value list
  (e.g. the three commas in `1-50,75,90-100` stay outside both rules' scope, untouched, the same
  as `018`'s own existing `LOOP i=1,5,1` behavior). Where the two rules' scopes are genuinely
  adjacent — a pair-boundary comma sitting immediately next to a range-dash value, e.g.
  `FILEO NODES=1-50 ,SELECTLINK=75 - 100` — both MUST apply independently to their own disjoint gaps in
  the same formatting pass, with neither rule's edit affecting the other's.
- **FR-007**: `preserve` mode (the default) MUST NOT be affected by this feature in any way — a
  project with `operator_spacing` unset or set to `preserve` sees byte-identical output to before
  this feature existed.
- **FR-008**: This feature MUST NOT introduce a new `[format]` configuration field, CLI flag, MCP
  parameter, or editor setting — it is a correction to `018-operator-spacing`'s existing
  `fixed`/`auto` behavior, reachable through the same `operator_spacing` setting that already
  exists.
- **FR-009**: Every existing formatting guarantee from `018-operator-spacing` (idempotence,
  behavior preservation, no reordering of statements, respect for `; FMT: OFF`/`; FMT: ON`
  regions, never altering content inside string/quoted literals or comments) MUST continue to
  hold with this feature in place.
- **FR-010**: Recognition of "inside a pair-keyword's value" MUST use the same pair-keyword-value
  boundary this project's formatter already derives for `018-operator-spacing`'s own
  comma-spacing rule — not a separately-maintained notion of where a value starts and ends, to
  avoid the two rules ever disagreeing about a value's own boundaries.

### Key Entities

- **Range dash**: A binary `-` occurrence inside a pair-keyword's value with a bare integer
  literal directly adjacent on both sides — the shape this feature exempts from ordinary
  binary-arithmetic spacing and instead actively normalizes to zero surrounding whitespace.
- **Pair-keyword value**: The token span between a `Control` statement's `keyword=` and either
  the next `keyword=` pair on the same statement or the end of the statement — the same
  structural span `018-operator-spacing`'s comma-spacing rule already recognizes (FR-010).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can enable `fixed` or `auto` mode and see every range-shaped `-` in a real
  corpus-shaped script's pair-keyword values (single-item and comma-list forms alike) rendered
  tight in a single formatting pass, regardless of how it was originally spaced.
- **SC-002**: A user can enable `fixed` or `auto` mode and see every `-` outside a pair-keyword's
  value (assignments, conditions, loop bounds) continue to receive `018-operator-spacing`'s
  existing binary-arithmetic spacing, with zero behavior change from before this feature shipped.
- **SC-003**: A script processed with `operator_spacing` unset or set to `preserve` is
  byte-identical before and after this feature ships, verified across the full real fixture
  corpus.
- **SC-004**: `fixed` and `auto` formatting remains idempotent with this feature in place —
  running either twice in a row on the same script produces no further change on the second
  pass — verified across the full real fixture corpus, including any real range-list values it
  contains.

## Assumptions

- "Bare integer literal" (FR-002) is deliberately narrower than "anything that isn't an
  operator" — a `@token@` reference or decimal number adjacent to a `-` inside a pair-keyword
  value falls back to ordinary binary-arithmetic spacing rather than being guessed at, since the
  range convention this feature targets is specifically Cube Voyager's integer-ID list notation,
  not a general "don't touch dashes in values" rule.
- The pair-keyword-value-vs-elsewhere boundary (FR-004) is the disambiguating signal, not any
  property of the numbers themselves (e.g. "both operands are integers") — a bare-integer `-` in
  an `Assignment` RHS such as `X = 100-1` is common, unambiguous subtraction and must keep normal
  spacing; the same shape inside a pair-keyword value is treated as a range (see the Edge Cases
  entry on `FACTOR=100-1`), an accepted trade-off rather than a false negative to eliminate.
- No real fixture corpus evidence of decimal-number ranges (`1.5-2.5`) or non-integer range
  bounds exists at spec time; FR-002's integer-only scope reflects the feature description's own
  framing ("two bare integer literals") and is treated as correct until corpus evidence says
  otherwise, matching this project's fixture-corpus-as-oracle practice (constitution Principle
  IV).
- This is scoped as a direct correction to already-shipped `018-operator-spacing` behavior, not a
  new independently-versioned formatting axis — no new configuration surface, no new precedence
  tier, no new diagnostic category.
