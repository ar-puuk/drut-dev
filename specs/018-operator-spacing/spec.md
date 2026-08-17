# Feature Specification: Operator Spacing Normalization

**Feature Branch**: `018-operator-spacing`

**Created**: 2026-08-17

**Status**: Draft

**Input**: User description: "Operator spacing normalization: a new `[format]` setting (three
modes: `preserve` default/no-op, `fixed`, `auto`) covering assignment `=`, comparison, and
arithmetic operators inside expressions. `fixed` normalizes every operator occurrence to
exactly one space on each side, plus comma spacing between multiple pairs on one `Control`
statement, plus removes interior padding inside brackets/parens and the space between a control
word and its opening paren. `auto` does everything `fixed` does, plus vertically aligns the `=`
of consecutive `Assignment` statements at the same block nesting depth to the column of the
longest left-hand side in the run, resetting on a blank line, a comment-only line, or an
indentation-depth change. `preserve` leaves everything exactly as written, matching every other
formatting axis already shipped." Full design history — the mode/scope decisions and the
industry-precedent discussion (`gofmt` vs. Prettier/Tidyverse alignment philosophies) that
shaped this — lives in `ROADMAP.md` item 12.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A project normalizes inconsistent operator spacing (Priority: P1)

A team's `.s`/`.block` scripts have accumulated inconsistent spacing around `=` and other
operators over years of hand-editing and multiple authors — the same statement shape appears
as `ZONES   = 1` in one place and `ZONES=1` a few lines later in the same file. The team wants
one canonical, single-space form applied consistently, without hand-editing every occurrence.

**Why this priority**: This is the direct, reported inconsistency (`ROADMAP.md` item 12's
corpus evidence) motivating the whole feature, and is useful entirely on its own — a team could
want just this, with no interest in vertical alignment at all.

**Independent Test**: With operator spacing configured to `fixed`, format a script containing
inconsistent `=` spacing, a comparison inside an `IF` condition, an arithmetic expression, a
multi-pair `Control` statement, and a subscript/parenthesized reference — confirm every one
normalizes to exactly one space around each operator, one space after each comma, and no
interior padding inside brackets/parens or between a control word and its opening paren.

**Acceptance Scenarios**:

1. **Given** operator spacing configured to `fixed`, **When** a script contains `ZONES   = 1`,
   `MATI=a.mat,MATO=b.mat`, `IF ( x==1 )`, and `MW[ 1 ]=mi.1.1+mi.2.1`, **Then** the formatted
   output renders `ZONES = 1`, `MATI = a.mat, MATO = b.mat`, `IF(x == 1)`, and
   `MW[1] = mi.1.1 + mi.2.1`.
2. **Given** no operator-spacing configuration at all, **When** the same script is formatted,
   **Then** none of that spacing changes — `preserve` remains the default, matching every other
   formatting axis already shipped.
3. **Given** operator spacing configured to `fixed`, **When** a script contains a negative
   literal such as `MW[1] = -5`, **Then** the unary sign stays bound to its operand
   (`MW[1] = -5`, not `MW[1] = - 5`) — only binary operators get surrounding space.

---

### User Story 2 - A project vertically aligns consecutive assignments for readability (Priority: P2)

A team wants a block of consecutive `Assignment` statements to visually line up — the same
effect they've seen from formatters like `gofmt` on consecutive `const`/`var` declarations —
without hand-maintaining that alignment through every future edit.

**Why this priority**: Independently valuable and independently shippable on top of User Story
1's normalization — a team could adopt `fixed` without ever wanting `auto`'s alignment
behavior, or skip straight to `auto` once `fixed` exists.

**Independent Test**: With operator spacing configured to `auto`, format a script with several
consecutive `Assignment` statements of varying left-hand-side length, interrupted partway
through by a blank line, then by a comment-only line, then by an indentation-depth change —
confirm each resulting group aligns independently to its own longest line, and that alignment
never bridges across any of the three break conditions.

**Acceptance Scenarios**:

1. **Given** operator spacing configured to `auto`, **When** three consecutive `Assignment`
   statements at the same nesting depth have left-hand sides of differing length, **Then** all
   three `=` signs align to the column immediately after the longest left-hand side, with the
   shorter ones padded to match.
2. **Given** the same configuration, **When** a blank line, a comment-only line, or a change in
   block nesting depth separates two otherwise-consecutive `Assignment` statements, **Then**
   the two sides of the break are aligned independently, each to its own group's longest line.
3. **Given** the same configuration, **When** a pair-keyword-shaped `Control` statement
   (e.g. `PHASE=ILOOP`) sits among consecutive `Assignment` statements, **Then** the `Control`
   statement's `=` is spaced per `fixed`'s single-space rule only — it neither joins nor
   extends an alignment run, and it breaks the run the same as any other non-`Assignment` line.
4. **Given** a single `Assignment` statement with no adjacent `Assignment` statement on either
   side, **When** the script is formatted, **Then** it is spaced exactly as `fixed` would render
   it — a run of one has nothing to align against.

---

### Edge Cases

- What happens inside a `; FMT: OFF`/`; FMT: ON` protected region? Left untouched, exactly as
  every other formatting rule already respects that marker.
- What happens to operator-shaped characters that appear inside a string/quoted literal or a
  comment (e.g. the `+` in `LIST='a+b'`)? Never touched, even though the underlying tokenizer
  represents them the same way it represents a real operator outside any string — recognition
  MUST independently track quote state and exclude anything between an opening and closing
  quote from every rule in this feature, not rely on the token stream alone to already make
  that distinction (research.md §9).
- What happens when a project's configuration sets operator spacing to an unrecognized or
  invalid value? Falls back to the `preserve` default with a non-blocking notice, matching the
  established pattern for every other malformed `drut.toml` field in this project.
- What happens to a unary `+`/`-` (a signed literal or signed expression term, as opposed to a
  binary arithmetic operator between two operands)? It stays bound to its operand with no
  inserted space, distinguishing it from binary `+`/`-`, which always gets one space on each
  side under `fixed`/`auto`.
- What happens when `auto`'s alignment would need to run across a diagnosed/unmatched block's
  child statements? Alignment still applies structurally; it does not depend on the block being
  well-formed, matching how indentation already behaves on diagnosed blocks.
- What happens to an operator character that is also one of the nine recognized trailing
  line-continuation characters (`, + - / * ^ & | =`) when it sits in that end-of-line
  continuation position rather than mid-expression? Only the space *before* it is normalized
  (ensuring exactly one space between it and its preceding operand); no trailing space is ever
  inserted after it, since nothing follows it on that physical line — the value/operand it
  continues onto starts on the next line, untouched by this feature.
- What whitespace counts toward "exactly one space" (FR-002)? Any run of spaces and/or tabs is
  replaced with a single literal space character — the normalized result is always a space,
  never a tab, matching how this project's indentation already always renders in spaces.
- What happens to an `Assignment` statement whose own `=` sits in trailing line-continuation
  position (its value starts on the next physical line)? It still participates in an alignment
  run under `auto` — alignment only cares about the `=` token's own position on its own line,
  never about where its value's tokens physically continue to.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST allow a project to configure operator spacing as one of
  `preserve`, `fixed`, or `auto`, defaulting to `preserve` when not explicitly configured.
- **FR-002**: `fixed` MUST normalize every occurrence of the assignment operator (`=`),
  comparison operators (`==`, `<>`, `>=`, `<=`, `<`, `>`), and binary arithmetic operators
  (`+`, `-`, `*`, `/`) inside expressions to exactly one space on each side. This is the closed,
  exhaustive operator set for this feature — `^`, `&`, and `|` (also lexer delimiter characters,
  but never researched as arithmetic/logical operators for this feature) are explicitly
  out of scope and MUST NOT be touched.
- **FR-003**: `fixed` MUST distinguish a unary `+`/`-` from a binary `+`/`-` and MUST NOT
  insert space between a unary sign and its operand — only binary arithmetic operators receive
  surrounding space.
- **FR-004**: `fixed` MUST normalize comma spacing between multiple pairs on one `Control`
  statement to exactly one space after each comma and none before.
- **FR-005**: `fixed` MUST remove interior padding inside brackets and parentheses (subscript
  references and parenthesized conditions) and remove any space between a control word and its
  opening parenthesis.
- **FR-006**: `auto` MUST perform everything `fixed` does, plus vertically align the `=` of
  consecutive `Assignment` statements at the same block nesting depth to the column of the
  longest left-hand side within that run.
- **FR-007**: `auto`'s alignment MUST consider only literal `Assignment` statements — a
  pair-keyword-shaped `Control` statement's `=` is never a member of, and never extends, an
  alignment run, even when adjacent to one.
- **FR-008**: An alignment run MUST end, and a new run MUST begin independently realigned, at a
  blank line, a comment-only line, a change in block nesting depth, or any non-`Assignment`
  statement — alignment never bridges across any of these. An `Assignment` statement sitting
  inside a `; FMT: OFF`/`; FMT: ON` protected region breaks a run the same way a non-`Assignment`
  statement does — it is excluded from the run entirely (never padded, never counted toward a
  neighboring run's target column), not merely skipped-but-counted.
- **FR-009**: `preserve` (the default) MUST leave all operator, comma, and bracket/paren
  spacing exactly as written — a project with no operator-spacing configuration MUST produce
  byte-identical output to before this feature existed.
- **FR-010**: Every existing formatting guarantee (idempotence, behavior preservation, no
  reordering of statements, respect for `; FMT: OFF`/`; FMT: ON` regions, never altering values
  inside string/quoted literals or comments) MUST continue to hold under both `fixed` and
  `auto`.
- **FR-010a**: Recognition MUST independently determine whether an operator-shaped character
  sits inside an open string/quoted literal and exclude it from every rule in this feature —
  the underlying token stream alone does not already distinguish that case (research.md §9), so
  this cannot be satisfied by construction and MUST be verified directly.
- **FR-011**: An unrecognized or invalid operator-spacing value in a project's `drut.toml` MUST
  NOT fail formatting — the system MUST fall back to the `preserve` default and surface a
  non-blocking notice, consistent with how every other malformed `[format]` field in this
  project already degrades. At the command-line and MCP surfaces, where the accepted values are
  a closed set (the same `preserve`/`fixed`/`auto` set as everywhere else, matching `casing`'s
  existing closed-set shape), an invalid value MUST be rejected with a clear usage/tool error at
  that surface's own input-validation point — the same existing behavior `casing` already has at
  both surfaces today, never a silent fallback there.
- **FR-012**: `fixed`/`auto` MUST normalize only the leading side of an operator character
  that is also a trailing line-continuation character when it appears in that end-of-line
  continuation position — the trailing side MUST NOT receive an inserted space, since no
  operand follows it on the same physical line.
- **FR-013**: Every surface that already exposes formatting configuration today (the
  command-line tool, the language server's format-on-save/format-on-paste, and the MCP format
  tool) MUST expose the operator-spacing control identically — no surface silently lagging or
  disagreeing with another.

### Key Entities

- **Operator spacing mode**: A project-wide setting — `preserve`, `fixed`, or `auto` —
  controlling whether and how spacing around operators, commas, and bracket/paren interiors is
  normalized.
- **Alignment run**: A maximal consecutive sequence of `Assignment` statements at the same
  block nesting depth, with nothing but blank lines, comment-only lines, depth changes, or
  other statement kinds interrupting it. Used only by `auto` mode to determine the shared
  alignment column for that group.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can enable `fixed` mode and see every operator, comma, and bracket/paren
  spacing instance in a real corpus-shaped script normalized to one canonical form in a single
  pass.
- **SC-002**: A user can enable `auto` mode and see consecutive `Assignment` statements in a
  real multi-line block vertically align, with alignment correctly and independently resetting
  across a blank line, a comment-only line, and an indentation-depth change within the same
  script.
- **SC-003**: A script processed with no operator-spacing configuration at all is byte-identical
  before and after this feature ships, verified across the full real fixture corpus.
- **SC-004**: An invalid operator-spacing value never silently produces the wrong formatting
  result: a `drut.toml` value degrades to the `preserve` default with a non-blocking notice,
  every time; a command-line or MCP value outside the closed accepted set is rejected with a
  clear usage/tool error at that surface's own input point, every time — matching `casing`'s
  existing behavior at both surfaces today.
- **SC-005**: `fixed` and `auto` formatting is idempotent — running either twice in a row on
  the same script produces no further change on the second pass — verified across the full real
  fixture corpus.

## Assumptions

- The unary-vs-binary `+`/`-` distinction (FR-003) follows the same convention essentially
  every mainstream formatter already uses (Python/`black`, JS/`prettier`, Go/`gofmt`): a sign is
  unary — and stays tight against its operand — when nothing precedes it (start of the value),
  or when the immediately preceding token is itself `=`, `(`, a comma, or **another recognized
  operator** (so `A + -B` normalizes to one space around `+` and a tight `-B`, not two operators
  both getting binary spacing); every other case is binary.
- `auto`'s alignment behavior is modeled on `gofmt`'s automatic alignment of consecutive
  `const`/`var`/struct-literal lines (breaks on a blank line or a non-matching line) rather
  than on Prettier's or R's `styler`/Tidyverse convention of refusing to align at all — this
  tradeoff was made knowingly (`ROADMAP.md` item 12) because Drut applies and re-applies the
  alignment automatically rather than asking anyone to hand-maintain it.
- Bill's evidenced further split within casing categories (`ROADMAP.md` item 11) and role-based
  semantic highlighting are unrelated, separately-scoped ideas — **out of scope** here.
- The exact configuration surface shape (configuration-file field name, command-line flag name,
  MCP parameter name) is a planning-phase decision, not fixed by this spec. The binding
  requirement is a three-value setting, default `preserve`, additive to every existing
  configuration surface — never a breaking change to already-shipped formatting behavior.
- Item 12's related cases (4) and (5) — control-word/paren spacing and bracket/paren interior
  padding — are folded into `fixed`/`auto` per FR-005, not left as separate configurable axes,
  by explicit owner decision (2026-08-17): `preserve` remains a true no-op either way.
- Invalid-value handling (FR-011/SC-004) is deliberately surface-specific rather than one
  uniform behavior: `drut.toml` accepts free-form text and can genuinely receive a malformed
  string, so it degrades softly; the command-line and MCP surfaces accept only a closed set of
  values (identical in shape to `casing`'s existing closed set), so an invalid value there is
  rejected at that surface's own input-validation point instead — this was corrected during
  `/speckit-analyze` (finding C1, 2026-08-17) after the original single-behavior wording proved
  inconsistent with how a closed-set CLI/MCP value can actually be rejected.
