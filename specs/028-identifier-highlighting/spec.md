# Feature Specification: Data-Reference & User-Variable Highlighting

**Feature Branch**: `028-identifier-highlighting`

**Created**: 2026-08-19

**Status**: Draft

**Input**: User description: "Add real syntax highlighting for two identifier classes that
currently have no genuine highlighting mechanism, only accidental coloring from
position-based grammar rules (found via real-world testing against a production Cube
Voyager script): (1) the data-reference family (DBA/DBI/MI/MO/MW/LI/LW/NI/NW/ZI/ZONES/
Z/RO/A/B/I/J) needs a real, position-independent highlighting mechanism plus a
`drut.highlight.dataReferences` setting; (2) generic user-defined variable identifiers
(assignment targets and expression operands alike) need a real, consistent highlighting
category plus a `drut.highlight.userVariables` setting. Both are `editors/vscode`-only,
same scope as `026-highlight-customization`/`027-named-variable-highlight`."

## Clarifications

### Session 2026-08-19

- Q: Should the two new highlighting categories skip Label declarations (`:STEP0`) and
  ShellEscape lines (`*copy file1 file2`), mirroring the existing exclusion
  `voyager-core`'s casing logic already has for those two statement kinds? → A: Yes,
  mirror the exclusion — a ShellEscape line is raw OS shell text, not Voyager syntax at
  all, and a Label name is a jump target, not a value; both new categories skip both
  statement kinds entirely, the same as `data_reference.rs`'s existing casing scope.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A recognized data-reference name is highlighted everywhere it appears (Priority: P1)

A script author writes an expression that uses a Matrix/Line/Node/Zone/Database
abbreviation (`DBA`, `MI`, `MW`, ...) somewhere other than immediately before or after
`=` — for example as a function-call argument (`ROUND(DBA.2.VOL[numrec])`). Today this
renders with no highlighting at all, even though the exact same name renders (by
accident, via an unrelated positional rule) when it happens to sit right after `=`
elsewhere in the same file. The author wants the name to render consistently, in its
own distinct color, everywhere it's used.

**Why this priority**: This is a real correctness gap — the same recognized name (per
`voyager-core`'s own `data_reference.rs`, already used for casing) renders inconsistently
today purely based on incidental sentence position, which reads as broken to anyone who
notices it, as a real user did against a production script.

**Independent Test**: Open a `.s` file containing `DBA`/`MI`/etc. both adjacent to `=`
and inside a function call or bare expression; confirm every occurrence renders in the
same distinct color.

**Acceptance Scenarios**:

1. **Given** a script containing `VOL_COR = ROUND(DBA.2.VOL[numrec]) / 100`, **When** the
   file is open, **Then** `DBA` renders in the data-reference color, the same as it would
   in `X = DBA.2.field`.
2. **Given** a script containing `LOOP NUMREC = counter, DBI.2.NUMRECORDS`, **When** the
   file is open, **Then** `DBI` renders in the data-reference color even though it sits on
   a block-opener line, not after a bare `=`.
3. **Given** `drut.highlight.dataReferences` is set to a color, **When** the setting
   changes, **Then** every data-reference occurrence recolors immediately, without a
   window reload — the same reactivity the 9 existing `drut.highlight.*` categories from
   `026-highlight-customization` already have.
4. **Given** a name that is both a recognized data-reference name and sits in a
   keyword-pair-name position (e.g. `ZONES` in `RUN PGM=MATRIX ZONES=5`), **When** the
   file is open, **Then** it renders in the data-reference color, not the pair-keyword
   color — one name, one owning category, never both at once.

---

### User Story 2 - A user-defined variable is highlighted consistently regardless of position (Priority: P2)

A script author writes an expression combining several of their own variable names, e.g.
`LINKID = _ANode + '_' + _BNode`. Today `_ANode` happens to render in a color (because it
sits immediately after `=`, an unrelated rule meant for something else) while `_BNode`,
two tokens later in the same expression, renders with no color at all. The author wants
their own variable names to read consistently wherever they appear in an expression.

**Why this priority**: Independently valuable and independently shippable from User
Story 1, but a larger, fuzzier design surface (there's no closed vocabulary for "a user
variable" the way there is for the data-reference family) — sequenced second so User
Story 1's narrower, evidence-backed fix can land on its own.

**Independent Test**: Open a `.s` file with an expression combining more than one
user-defined identifier (not a keyword, function name, pair-keyword, or data-reference
name); confirm every identifier not otherwise claimed by a more specific category renders
in the same distinct color.

**Acceptance Scenarios**:

1. **Given** a script containing `LINKID = _ANode + '_' + _BNode`, **When** the file is
   open, **Then** `_BNode` renders in the user-variable color (today it renders with no
   color at all).
2. **Given** `drut.highlight.userVariables` is set to a color, **When** the setting
   changes, **Then** every matching identifier recolors immediately, without a window
   reload.
3. **Given** a bareword that is a recognized control word, statement word, built-in
   function name (in call position), pair-keyword name, pair-value, or data-reference
   name, **When** the file is open, **Then** it keeps rendering in its existing category's
   color, never the new user-variable color — purely additive, never reclassifying an
   already-owned name.

---

### Edge Cases

- A single-letter data-reference name (`A`, `B`, `I`, `J`, `Z`) matches by exact name,
  the same as `voyager-core`'s own casing recognition — an ordinary variable that happens
  to be named exactly `I` or `Z` renders as a data reference, not as a user variable. This
  is an inherited, pre-existing false-positive rate (identical to what casing already
  accepts for the same names), not a new one introduced by this feature.
- A bareword immediately adjacent to `=` keeps whatever category already claims that
  position today (`pairKeywords` before `=`, `pairValues` after `=`, unless it's also a
  data-reference name, User Story 1 Acceptance Scenario 4) — it does **not** switch to the
  new `userVariables` color just because this feature exists. See Assumptions for why.
- Content inside a quoted string or a comment is never matched by either new category —
  inherited automatically from the grammar's existing string/comment scoping, not a new
  rule this feature has to implement.
- A `ShellEscape` line (`*copy A B`) or a `Label` declaration (`:STEP0`) never triggers
  either new category — the shell command's own words and the label's own name render
  unstyled, exactly as they do today (Clarifications, Session 2026-08-19).
- A coincidentally-named user array or matrix literally called `MAX` and indexed as
  `MAX(1)` is already an accepted rare false positive for `functionCalls` (per
  `024-function-call-highlighting`); this feature does not change that call, and does not
  attempt to reclassify it as a user variable either.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The extension MUST recognize the same data-reference family
  `voyager-core`'s `data_reference.rs` already recognizes for casing (`MI`, `MO`, `MW`,
  `LI`, `LW`, `NI`, `NW`, `ZI`, `ZONES`, `Z`, `DBI`, `DBA`, `RO`, `A`, `B`, `I`, `J`) as its
  own highlightable category, matched case-insensitively by exact name or dot-notation
  prefix (`dba.2.field`), regardless of where it appears in a statement or expression.
- **FR-002**: The extension MUST expose `drut.highlight.dataReferences`, following the
  exact same personal-setting shape `026`'s other 9 categories use (optional string,
  Global scope, live reactive on change, no `drut.toml` equivalent).
- **FR-003**: When a bareword is both a recognized data-reference name and would
  otherwise match the `pairKeywords`/`pairValues` categories' purely positional shape
  (e.g. `ZONES` in `RUN PGM=MATRIX ZONES=5`), the data-reference category MUST take
  precedence — mirrors `data_reference.rs`'s own FR-005 "one name, one occurrence"
  ownership rule, carried over from casing to highlighting for consistency between the
  two mechanisms.
- **FR-004**: The extension MUST recognize a bareword identifier that is none of: a
  recognized control word, a recognized statement word, a recognized built-in function
  name in call position, a pair-keyword-shaped name, a pair-value-shaped value, or a
  data-reference name (per FR-001) as its own "user variable" highlightable category,
  regardless of where it appears in a statement or expression.
- **FR-004a**: Both new categories (`dataReferences` and `userVariables`) MUST skip
  Label declarations (`:LabelName`) and ShellEscape lines (`*shell command...`) entirely
  — mirroring `data_reference.rs`'s existing exclusion of `StatementKind::Label`/
  `StatementKind::ShellEscape` from casing scope (Clarifications, Session 2026-08-19). A
  `ShellEscape` line's content is arbitrary OS shell text, not Voyager syntax, so
  matching it as either category would be actively wrong, not merely imprecise; a
  `Label` name is a jump target, not a value.
- **FR-005**: The extension MUST expose `drut.highlight.userVariables`, same
  personal-setting shape as FR-002.
- **FR-006**: Neither new category may change the matching, scope, or precedence of any
  of the 9 existing `drut.highlight.*` categories from `026` or `drut.highlight.
  namedVariables` from `027` — purely additive, the same constraint both prior features
  stated for themselves.
- **FR-007**: Neither new category may change `voyager-core`'s tokenizer, parser, or
  formatter, `drut-lsp`'s semantic-token emission, or any `Diagnostic` category — this is
  an `editors/vscode` client-side coloring concern only, same as `026`/`027`.
- **FR-008**: Content inside a quoted string or a comment MUST never be matched by either
  new category.

### Key Entities

- **`drut.highlight.dataReferences`**: the 11th `drut.highlight.<category>` entry,
  covering the data-reference family — a `TextMate`-scope-based mechanism like `026`'s
  original 9 categories (not the semantic-token mechanism `027` used for `@name@`, since
  this family has no existing semantic-token emission).
- **`drut.highlight.userVariables`**: the 12th `drut.highlight.<category>` entry,
  covering the catch-all "none of the above" bareword-identifier case — same
  `TextMate`-scope-based mechanism.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A recognized data-reference name renders in one consistent, distinct color
  everywhere it appears in a script, verified against the reported
  `ROUND(DBA.2.VOL[numrec])` and `LOOP NUMREC = counter, DBI.2.NUMRECORDS` cases.
- **SC-002**: A user-defined identifier used as an expression operand renders in one
  consistent, distinct color regardless of its position within the expression, verified
  against the reported `_ANode + '_' + _BNode` case (modulo the documented `=`-adjacency
  trade-off in Assumptions).
- **SC-003**: Setting either `drut.highlight.dataReferences` or `drut.highlight.
  userVariables` visibly recolors matching tokens without a window reload, in every
  tested scenario.
- **SC-004**: Zero behavior change in any of the 9 existing `drut.highlight.*` categories
  or `drut.highlight.namedVariables`, verified explicitly (regression scenarios).

## Assumptions

- **Why a bareword adjacent to `=` keeps its existing category rather than switching to
  `userVariables`**: this is a purely `regex`-based, positionally-scoped `TextMate`
  grammar with no access to `voyager-core`'s real `Statement`/`Assignment` parse tree — it
  cannot structurally distinguish a genuine keyword-pair value (`MATRIX` in `PGM=
  MATRIX`, an enum-like program name) from an ordinary assignment's RHS variable
  reference (`_ANode` in `LINKID = _ANode`), since both are simply "a bareword right
  after `=`." Reassigning that position to `userVariables` would silently change the
  already-shipped `drut.highlight.values`/`drut.highlight.pairKeywords` categories'
  behavior for every existing user of those settings — a real regression FR-006
  forbids. The pragmatic, documented trade-off: `userVariables` is strictly the
  catch-all for identifier positions no existing category already reaches (expression
  operands away from `=`, like `_BNode`) — not a full re-homing of every variable
  reference regardless of position. An identical variable name can therefore render in
  two different configured colors depending on where it sits in an expression; this is
  accepted, not silently hidden, the same way `pair-keywords`' own doc comment already
  accepts a comparable shape-over-name trade-off for itself.
- **Why `dataReferences` uses `026`'s `TextMate`-scope mechanism, not `027`'s
  semantic-token mechanism**: `027`'s workspace-scoped, semantic-token approach exists
  specifically because `drut-lsp` already emits an unconditional semantic `variable`
  token for `@name@` that visually layers over (and hides) a `TextMate`-scope color. The
  data-reference family has no such existing semantic-token emission to conflict with, so
  `026`'s simpler, Global-scope, `TextMate`-only mechanism applies cleanly, with no
  scope-layering problem to work around.
- Scope is `editors/vscode` only — same as `026`/`027`. No `drut.toml`, CLI, or MCP
  surface.
- Both categories are additive to the grammar's existing pattern list; neither removes
  nor renames any existing `TextMate` scope name already in use by a shipped
  `drut.highlight.*` setting.
