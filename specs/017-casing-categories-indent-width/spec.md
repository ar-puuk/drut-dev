# Feature Specification: Per-Category Casing Configuration and Configurable Indentation Width

**Feature Branch**: `017-casing-categories-indent-width`

**Created**: 2026-08-17

**Status**: Draft

**Input**: User description: "for the bundled casing + indent_width" — bundling `ROADMAP.md`
pre-publish items 9 and 10: (1) `casing` reframed as three independently-configurable
categories (control words, pair-keyword names, and a new "data-reference" category covering
Matrix/Line/Node/Zone/Database abbreviations plus the two reserved loop-index identifiers),
each `upper`/`lower`/`preserve`, no built-in opinionated preset; (2) `indent_width` becomes a
configurable `[format]` setting instead of a fixed 4-space value. Full design history —
corpus/vendor-doc research, stakeholder input (`casing-convention-decision.csv`, GitHub issue
#3), and the decisions that shaped this scope — lives in `ROADMAP.md` items 9–12 and the
(superseded) resolved-queued item 4.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A project sets its own casing convention per token category (Priority: P1)

A team maintaining `.s`/`.block` scripts wants their own established convention respected —
which may treat control words (`IF`/`LOOP`/`RUN`), general statement parameters (`FILE=`/
`LIST=`), and abbreviated data-reference tokens (`MI`/`MW`/`ZI`/`I`/`J`) differently from each
other, and differently from any other team's convention. They set each of the three
independently in their project's configuration, and formatting applies exactly that —
nothing forced, nothing assumed correct for everyone else.

**Why this priority**: This is the actual deliverable — flexibility across teams with
different, undocumented, historically-inherited conventions, without the tool imposing one
house style on anyone. It's the reason a single flat setting (or a built-in opinionated
preset) was explicitly rejected during this feature's design.

**Independent Test**: With a project configured to three different values for the three
categories, format a script mixing all three token kinds and confirm each category's casing
changed independently according to its own setting, with the other two left exactly as
configured.

**Acceptance Scenarios**:

1. **Given** a project configuration setting all three categories to different values,
   **When** a script mixing control words, pair-keyword names, and data-reference tokens is
   formatted, **Then** each token's casing matches its own category's configured value, and no
   category's setting leaks into another's tokens.
2. **Given** no project configuration at all, **When** the same script is formatted, **Then**
   no token's casing changes — Preserve remains the default for every category, including the
   newly-introduced one, matching existing behavior for the two categories already reachable
   today.

---

### User Story 2 - Data-reference tokens become reachable by casing at all (Priority: P1)

Today, only control words and pair-keyword names can be recased — Matrix/Line/Node/Zone/
Database abbreviated data references (`MI`/`MO`/`MW`, `LI`/`LW`, `NI`/`NW`, `ZI`/`ZONES`/`Z`,
`DBI`/`DBA`, `RO`, `A`/`B`) and the two reserved loop-index identifiers (`I`/`J`) are invisible
to casing regardless of configuration. A user who wants `mw`/`li`/`ni`/`i`/`j` uppercased — a
real reported request — cannot get it today no matter what they configure. This closes that
gap.

**Why this priority**: This is the literal, reported complaint (GitHub issue #3) motivating
the whole feature. Without it, the other two categories alone don't solve the reported
problem — they were already configurable before this feature existed.

**Independent Test**: Format a script containing `mw[1] = mi.1.1 + mi.2.1` and `li.ft`/
`ni.class` with the data-reference category set to `upper`, and confirm every one of those
tokens is uppercased, regardless of whether it's playing a pair-keyword-shaped role, an
assignment-target role, or an inline dot-notation read role.

**Acceptance Scenarios**:

1. **Given** the data-reference category set to `upper`, **When** a script assigns
   `mw[1] = mi.1.1 + mi.2.1`, **Then** both `mw` and `mi` render uppercase in every position
   they appear — assignment target and inline read alike.
2. **Given** the data-reference category set to `upper`, **When** a script contains `li.FT`
   and a `PATHLOAD`-style pair-keyword-shaped `mw[201]=` usage, **Then** both are uppercased
   consistently — one setting applies uniformly regardless of which structural shape a given
   occurrence takes.
3. **Given** the data-reference category set to `preserve` (the default), **When** the same
   script is formatted, **Then** none of these tokens change.

---

### User Story 3 - A project sets its own indentation width (Priority: P2)

A team wants nested block indentation — today a fixed 4 spaces per level — to match their own
established convention instead, without giving up any other formatter behavior.

**Why this priority**: Independently valuable and independently shippable — a team could want
just this without touching casing at all, or vice versa. Ranked below the casing stories
because it isn't tied to a specific reported complaint the way User Story 2 is.

**Independent Test**: With indentation width configured to 2, format a script with nested
`IF`/`LOOP` blocks and confirm each nesting level advances by exactly 2 spaces instead of 4,
with casing and top-level indent mode unaffected.

**Acceptance Scenarios**:

1. **Given** indentation width configured to 2, **When** a script with three levels of nested
   blocks is formatted, **Then** indentation increases by 2 spaces per level, consistently
   throughout the file.
2. **Given** an invalid indentation width (zero, negative, or unreasonably large), **When** a
   project's configuration is read, **Then** formatting proceeds using the built-in default
   width with a non-blocking notice, not a hard failure — matching how every other malformed
   configuration value in this project already degrades.
3. **Given** no indentation width configured, **When** any script is formatted, **Then**
   indentation continues to use today's 4-space default — byte-identical output to before this
   feature existed.

---

### Edge Cases

- What happens to `NUMREC`/`CNT`/`ITER`/`LP`/`RECNUM`, today offered (incorrectly) as
  completion/spell-check suggestions for a `LOOP` statement's variable-name position? They
  stop being offered — confirmed, via the documented `LOOP <name>=start,end[,increment]`
  syntax, to be user-chosen loop-variable names rather than reserved keywords.
- What happens to `ZONES`, currently entirely absent from the recognized keyword dictionary
  despite being a real, frequently-used reserved parameter? It's added, and participates in
  data-reference casing like its family members, in both of its real usages (a
  `RUN PGM=MATRIX ZONES=...` parameter and a plain assignment).
- What happens when a data-reference token appears inside a `; FMT: OFF`/`; FMT: ON` protected
  region? Left untouched, exactly as every other formatting rule already respects that marker.
- What happens when a project's configuration sets a category (or the indentation width) to an
  unrecognized or invalid value? Falls back to that setting's built-in default with a
  non-blocking notice — the same established pattern already used for every other malformed
  `drut.toml` field in this project.
- What happens to ordinary, arbitrary user-chosen variable names that happen to resemble a
  data-reference token in spelling but aren't actually being used in that documented role?
  Casing is applied only where a token is structurally recognized as one of these specific
  reserved names in its documented role — never a blind text substitution across the file.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST allow a project to independently configure casing (upper/lower/
  preserve) for three categories of Voyager tokens: control words, pair-keyword names, and
  data-reference tokens.
- **FR-002**: Each casing category MUST default to Preserve when not explicitly configured,
  independently of the other two — matching this project's existing default-to-Preserve
  behavior, extended to the newly-introduced category.
- **FR-003**: The system MUST NOT ship any built-in opinionated preset (an "auto" mode or
  otherwise) that applies a house style by default — every casing value comes from the
  project's own explicit configuration or the Preserve default, never from a value the tool
  itself chooses.
- **FR-004**: The data-reference category MUST cover, at minimum: the Matrix family (`MI`,
  `MO`, `MW`), the Line family (`LI`, `LW`), the Node family (`NI`, `NW`), the Zone family
  (`ZI`, `ZONES`, `Z`), the Database family (`DBI`, `DBA`), the output-record token (`RO`),
  the link-endpoint tokens (`A`, `B`), and the two reserved implicit loop-index identifiers
  (`I`, `J`).
- **FR-005**: A data-reference token's configured casing MUST apply uniformly regardless of
  which structural role a given occurrence plays (a pair-keyword-shaped parameter, an
  assignment target, or an inline dot-notation read) — one setting per token name, not one per
  role.
- **FR-006**: The system MUST correctly recognize and rewrite data-reference tokens even
  though today's tokenizer does not expose the boundary before `.` in dot-notation tokens
  (e.g. `mi.1.1`) as a distinct unit — this requires extending the underlying grammar/
  tokenizer, not just the formatter's rewrite logic.
- **FR-007**: `NUMREC`, `CNT`, `ITER`, `LP`, and `RECNUM` MUST be removed from the recognized
  keyword dictionary — confirmed to be user-chosen `LOOP` variable names rather than reserved
  keywords — and therefore MUST NOT be offered as completion/spell-check suggestions, nor
  targeted by any casing category.
- **FR-008**: `ZONES` MUST be added to the recognized keyword dictionary and MUST be covered
  by data-reference casing in both of its real usages.
- **FR-009**: The system MUST allow a project to independently configure the number of spaces
  used per nesting level of block indentation, with a built-in default matching today's fixed
  4-space behavior.
- **FR-010**: An invalid indentation-width value MUST NOT fail formatting — the system MUST
  fall back to the built-in default for that value and surface a non-blocking notice,
  consistent with how every other malformed configuration value in this project already
  degrades.
- **FR-011**: Every existing formatting guarantee (idempotence, behavior preservation, respect
  for `; FMT: OFF`/`; FMT: ON` regions, never touching values/labels/variable references) MUST
  continue to hold for both the newly-reachable data-reference casing category and the
  configurable indentation width.
- **FR-012**: A script formatted with no project configuration for any setting introduced by
  this feature MUST produce byte-identical output to before this feature existed — this is a
  purely additive capability, not a behavior change for any existing user or configuration.
- **FR-013**: Every surface that already exposes casing or formatting configuration today (the
  command-line tool, the language server's format-on-save/format-on-paste, and the MCP format
  tool) MUST expose the same new per-category casing controls and the new indentation-width
  control identically — no surface silently lagging or disagreeing with another.

### Key Entities

- **Casing category**: One of three named groups of Voyager tokens — control words,
  pair-keyword names, or data-reference tokens — independently settable to Preserve, Upper, or
  Lower.
- **Data-reference token**: A short, reserved, domain-specific abbreviation used to read or
  write a value inline in an expression or statement (the Matrix/Line/Node/Zone/Database
  families, plus the output-record, link-endpoint, and implicit loop-index tokens) — distinct
  from control words (block-structural syntax) and pair-keywords (statement parameter names).
- **Indentation width**: The number of spaces the formatter uses to represent one level of
  nested-block indentation, configurable per project, with a built-in default of 4.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can independently set control-word, pair-keyword, and data-reference
  casing to three different values in one project and see all three respected simultaneously
  in formatted output.
- **SC-002**: The specific tokens named in the original user report (`mw`, `li`, `ni`, `i`,
  `j`) can be uppercased via project configuration, verified against real corpus-shaped
  script content — closing that report end-to-end.
- **SC-003**: A script processed with no casing or indentation-width configuration at all is
  byte-identical before and after this feature ships, verified across the full real fixture
  corpus.
- **SC-004**: A project can set its indentation width to a value other than 4 and see every
  nested block in a real multi-level script reflect that width consistently.
- **SC-005**: An invalid indentation-width value never stops formatting from completing — it
  degrades to the built-in default with a notice, every time, on every surface (command-line,
  editor, MCP).
- **SC-006**: The `NUMREC`/`CNT`/`ITER`/`LP`/`RECNUM` removal and `ZONES` addition are both
  reflected in completion/spell-check suggestions, verified against the real corpus.

## Assumptions

- Bill's evidenced preference for a further split within data-references (casing `MW`/`LW`/
  `NW`'s pair-keyword-shaped usage differently from their assignment-target usage) is real and
  captured (`ROADMAP.md` item 11) but explicitly **out of scope** for this feature — deferred
  as a future, purely additive follow-on that doesn't require revisiting anything this feature
  ships.
- `=`/operator spacing normalization (`ROADMAP.md` item 12) is a separate, not-yet-researched
  feature and explicitly **out of scope** here.
- Role-based semantic highlighting (coloring a dual-role token differently depending on which
  role a given occurrence plays, raised alongside this feature's design) is a separate future
  idea, **out of scope** here.
- No opinionated preset/"auto" mode ships with this feature, by explicit decision — every
  casing value is either the project's own explicit configuration or the Preserve default.
- The exact configuration surface shape (configuration-file field names/structure,
  command-line flag names, MCP parameter names) is a design decision for the planning phase,
  not fixed by this spec. The binding requirement is independent per-category configurability
  with a Preserve default, additive to every existing configuration surface — never a breaking
  change to already-shipped casing behavior for control words and pair-keywords.
- The indentation-width valid-range bound (a sane cap, not unlimited) is a planning-phase
  detail; this spec only requires that invalid values degrade non-fatally, consistent with
  `ROADMAP.md` item 9's carried-forward recommendation (a bound around 1–16).
