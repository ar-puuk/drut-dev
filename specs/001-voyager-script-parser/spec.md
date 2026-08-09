# Feature Specification: Voyager Script Tokenizer & Structural Parser

**Feature Branch**: `001-voyager-script-parser`

**Created**: 2026-08-08

**Status**: Draft

**Input**: User description: "Build a tokenizer and structural parser for Cube Voyager control-statement scripts
(.s and .block files) as a Rust library crate with no I/O or protocol dependencies.

The parser must handle:
- Control statement format: a control word followed by space-separated keyword=value pairs
- Line comments starting with \";\" and block comments delimited by \"/* ... */\"
- Line continuation: a statement continues onto the next line when its last
  non-comment character is one of: , + - / * ^ & | =
- Block matching: IF / ELSEIF / ELSE / ENDIF, LOOP / ENDLOOP (with BREAK as a valid
  early exit), and RUN PGM=... / ENDRUN
- @variable@ token substitution syntax (tokenize and track, no evaluation needed)
- Case-insensitive control words and keywords

Out of scope for this phase: per-program-box keyword validation, formatting, any
protocol layer (LSP/MCP/CLI), semantic/reference checking.

The parser's diagnostic output must include, at minimum, structured errors for:
- Unclosed or unmatched IF/ENDIF or LOOP/ENDLOOP blocks
- Unclosed block comments
- A statement ending in a continuation character with no following line, or a
  following line that isn't a valid continuation
- Unclosed RUN PGM=.../ENDRUN blocks

Definition of done: the parser produces zero false-positive diagnostics against a
fixture corpus of real, working .s/.block scripts (to be supplied), and correctly
flags every deliberately-broken fixture in the corpus.

Open question to resolve during this phase: confirm whether .block files use the same
grammar as .s files or represent script fragments meant to be included into a larger
job. Version scope: target Cube Voyager 6.5 control-statement grammar as the baseline."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
-->

### User Story 1 - Parse a valid script into structure (Priority: P1)

A developer building a Voyager tool (CLI, LSP server, MCP server, or formatter) hands
the parser the raw text of a `.s` script and receives back a structural breakdown of
that script: its statements, keyword=value pairs, and nested IF/LOOP/RUN blocks —
without the parser touching the filesystem, a network, or any editor/protocol layer
itself.

**Why this priority**: Every downstream tool (formatter, linter, LSP, MCP) depends on
this structural breakdown existing and being correct. Nothing else in the project can
be built until this works.

**Independent Test**: Feed a known-good `.s` fixture (multi-line statements, nested
IF/LOOP/RUN blocks, comments, `@variable@` references) directly to the library and
confirm it returns a complete structure with zero diagnostics — no file, editor, or
CLI required to exercise it.

**Acceptance Scenarios**:

1. **Given** a `.s` script containing a control statement with several
   `keyword=value` pairs written in mixed case, **When** the script is parsed,
   **Then** the parser returns a single statement whose control word and keywords are
   recognized regardless of case, with zero diagnostics.
2. **Given** a `.s` script whose statement spans three physical lines using trailing
   continuation characters (e.g. `,`, `+`, `=`), **When** the script is parsed,
   **Then** the parser returns one logical statement (not three), with zero
   diagnostics.
3. **Given** a `.s` script with nested `IF`/`ELSEIF`/`ELSE`/`ENDIF` and `LOOP`/`BREAK`/
   `ENDLOOP` blocks that all correctly close, **When** the script is parsed, **Then**
   the parser returns a correctly nested block structure with zero diagnostics.

---

### User Story 2 - Get precise diagnostics for a broken script (Priority: P2)

A developer building a Voyager tool feeds the parser a script that a real author
mistyped — a missing `ENDIF`, an unclosed `/* comment`, a dangling continuation
character at end of file — and needs back a structured diagnostic that names the
problem and points at its location, instead of a crash or a silent wrong answer.

**Why this priority**: Diagnostics are the entire value proposition of downstream
tools (linter, LSP). Without precise, non-crashing diagnostics here, every consumer of
this library has to re-invent broken-input handling.

**Independent Test**: Feed each deliberately-broken fixture (one per defect category)
directly to the library and confirm each produces at least one diagnostic correctly
naming that category, and that the process does not panic.

**Acceptance Scenarios**:

1. **Given** a script with an `IF` that has no matching `ENDIF` before end of file,
   **When** the script is parsed, **Then** the parser returns a diagnostic identifying
   an unclosed/unmatched `IF`/`ENDIF` block at the `IF` statement's location.
2. **Given** a script with an `ENDLOOP` that has no matching `LOOP`, **When** the
   script is parsed, **Then** the parser returns a diagnostic identifying an unmatched
   `LOOP`/`ENDLOOP` block.
3. **Given** a script containing `/*` with no matching `*/` before end of file,
   **When** the script is parsed, **Then** the parser returns a diagnostic identifying
   an unclosed block comment.
4. **Given** a script whose last line ends in a continuation character (e.g. `,`) with
   no following line, **When** the script is parsed, **Then** the parser returns a
   diagnostic identifying the invalid/missing continuation.
5. **Given** a script with a `RUN PGM=...` that has no matching `ENDRUN` and no
   following `RUN`/`!RUN`/shell-escape statement before end of file, **When** the
   script is parsed, **Then** the parser returns a diagnostic identifying an
   unclosed `RUN`/`ENDRUN` block.
6. **Given** any of the above broken scripts, **When** the script is parsed, **Then**
   parsing completes (returns diagnostics) rather than panicking or hanging, and, where
   feasible, continues to report on the rest of the script rather than stopping at the
   first defect.

---

### User Story 3 - Track token-level detail for editor-style features (Priority: P3)

A developer building an editor-facing tool (LSP hover, syntax highlighting) needs the
tokenizer to expose comments, `@variable@` substitution references, and line
continuations as distinct, position-tracked tokens, so those features can be built
without re-tokenizing the script themselves.

**Why this priority**: This unlocks editor-facing features in later phases but isn't
required for the first structural/diagnostic consumers (a linter or formatter could
technically work from statements/blocks alone).

**Independent Test**: Feed a script containing line comments, a multi-line block
comment, and several `@variable@` references (including one split across a line
continuation) to the library and confirm each is returned as its own token with
correct source position and, for `@variable@`, the variable name.

**Acceptance Scenarios**:

1. **Given** a script with a line comment (`; ...`) following real statement content
   on the same line, **When** the script is parsed, **Then** the comment is tokenized
   separately from the statement content and does not affect continuation detection.
2. **Given** a script with a multi-line `/* ... */` block comment, **When** the script
   is parsed, **Then** the entire block comment is tokenized as a single unit spanning
   its start and end positions.
3. **Given** a script containing `@variable@` inside a keyword's value, **When** the
   script is parsed, **Then** the parser returns a distinct token recording the
   variable's name and position, without attempting to resolve or substitute a value.

---

### Edge Cases

- A `BREAK` statement appears at a file's true top level, with no enclosing block of
  any kind — this is the only shape FR-026 flags. A `BREAK` nested only inside an
  `IF` (with no `LOOP`/`RUN`/`PROCESS` around that `IF`) is *not* flagged, since the
  `IF` itself already counts as an enclosing block.
- Blocks of different kinds are interleaved incorrectly (e.g., a `LOOP` opened inside
  an `IF` closes with `ENDIF` before `ENDLOOP`).
- An `ENDIF`, `ENDLOOP`, or `ENDRUN` appears with no corresponding opening statement
  anywhere earlier in the file (dangling close) — including an `ENDIF` that appears
  after a short-`IF` (FR-007) has already self-closed, which is also a dangling
  close even though an `IF` did appear earlier.
- A `RUN` block has no explicit `ENDRUN` anywhere and is instead closed implicitly by
  the next `RUN`/`!RUN` statement, or by a shell-escape statement (FR-009). A
  `PROCESS`/`PHASE=` block relies on the same kind of implicit closing by the next
  `PROCESS`/`PHASE=` statement (FR-028).
- `!RUN` (a disabled `RUN`) is left open with no explicit `ENDRUN` — unlike a plain
  `RUN`, this is diagnosable, since `!RUN` doesn't get the implicit-close treatment.
- A block comment opens a second `/*` while already inside one — both must be closed
  by their own `*/` before the outer comment is considered closed (FR-005).
- A line comment (`;`) follows a continuation character on the same line — the
  continuation decision must be based on the last non-comment character, not the
  literal last character of the line.
- The file ends mid-block-comment (`/*` never closed) *and* mid-block (`IF` never
  closed) at the same time — both diagnostics are expected, not just one.
- An empty script, or a script containing only comments and whitespace.
- Case is mixed inconsistently for the same control word across a file (e.g. `If` ...
  `ENDIF` ... `endif`).
- A statement's continuation character is itself immediately followed only by
  whitespace and then a line comment, with real content resuming on the next line.
- A statement's continuation character is followed by one or more fully blank lines
  before the line that actually resumes it (FR-006) — this is a valid continuation,
  not a defect.
- A `.block` file's top level is entirely bare statements/blocks with no `RUN PGM=.../
  ENDRUN` wrapper at all (a fragment meant to be read into an already-open box).
- A `.block` (or `.s`) file's top level contains its own complete `RUN PGM=.../
  ENDRUN` box, with ordinary statements, labels, or shell-escapes before or after it.
- A label statement (`:STEP1`) or shell-escape statement (`*...` or `*(...)`) appears
  immediately before or after an `IF`/`LOOP`/`RUN` block, or between `ELSEIF`
  branches.
- An `IF (...)` statement is immediately followed on the same line by exactly one
  further statement, with no `ENDIF` anywhere in the file (short-`IF`, FR-007) — this
  is a complete, valid block on its own.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The parser MUST accept script source text supplied directly by its
  caller and MUST NOT perform file I/O, network access, or depend on any specific
  protocol (CLI, LSP, MCP) to do its work.
- **FR-002**: The parser MUST tokenize source text into positioned tokens (each
  carrying its line/column location in the source) before or as part of building
  higher-level structure.
- **FR-003**: The parser MUST recognize a control statement as a control word followed
  by zero or more space-separated `keyword=value` pairs.
- **FR-004**: The parser MUST recognize line comments beginning with `;` and running
  to the end of the physical line.
- **FR-005**: The parser MUST recognize block comments delimited by `/*` and `*/`,
  including block comments that span multiple physical lines, and MUST match them as
  a nesting construct rather than a flat one: a `/*` encountered while a block
  comment is already open starts its own inner comment, and the enclosing comment
  isn't complete until every comment nested inside it has its own matching `*/`.
  Confirmed against vendor reference documentation; the fixture corpus has not yet
  exercised a nested block comment either way (see Assumptions).
- **FR-006**: The parser MUST treat a statement as continuing onto the next physical
  line whenever the last non-comment, non-whitespace character on the line is one of:
  `,` `+` `-` `/` `*` `^` `&` `|` `=`. Any number of fully blank lines (no content at
  all, not even a comment) between a continuation-ending line and the line that
  actually resumes the statement are themselves skipped without breaking the
  continuation or triggering a diagnostic — the first non-blank line after them is
  the continuation. This applies to every statement form (`Control`, `Assignment`,
  `Label`, `ShellEscape`) uniformly — confirmed for `Control` and `Assignment`
  against real fixtures; `Label` and `ShellEscape` continuation is specified by the
  same uniform rule but has no confirmed real-world example either way. Separately, a
  `Control` statement (FR-003) MAY be continued using a second, independent
  mechanism instead of trailing operators: an opening `{` placed right after the
  control word begins a body that runs across any number of physical lines — none of
  which need a trailing continuation character — until the next `}` closes it. This
  brace form is available after any control word, not a specific one, and a single
  statement uses one continuation mechanism or the other, never both.
- **FR-007**: The parser MUST recognize and structurally match `IF` / `ELSEIF` /
  `ELSE` / `ENDIF` blocks, including nested occurrences. The parser MUST also
  recognize a self-closing short-`IF` form: an `IF (...)` statement followed
  immediately, on the same physical line, by exactly one further statement is a
  complete block on its own — that trailing statement is its entire body, and the
  block closes at the end of the line with no separate `ENDIF` expected or required.
  No equivalent short form exists for `ELSEIF`. A statement that instead trails
  `ELSEIF`, `ELSE`, or `ENDIF` on the same line MUST be parsed as its own ordinary
  statement, not folded into that block's body, and MUST NOT be reported as a
  defect for appearing there. Confirmed against vendor reference documentation; not
  yet exercised by the fixture corpus (see Assumptions).
- **FR-008**: The parser MUST recognize and structurally match `LOOP` / `ENDLOOP`
  blocks, including nested occurrences, and MUST recognize `BREAK` as a statement
  (see FR-026 for where it's diagnosable).
- **FR-009**: The parser MUST recognize and structurally match `RUN PGM=...` /
  `ENDRUN` blocks. `ENDRUN` is optional: a `RUN` block is also considered closed by
  whichever comes first of the next `RUN`/`!RUN` statement or a shell-escape
  statement (FR-022) — a plain `RUN` is well-formed with no explicit `ENDRUN`
  anywhere. The disabled form, `!RUN`, does not get this same treatment: a `!RUN`
  block always requires its own explicit `ENDRUN` and is diagnosable (FR-016) if left
  open, even though the `RUN` it disables would not be. Confirmed against vendor
  reference documentation; the fixture corpus's own zero-unbalanced-pairs finding is
  consistent with, but doesn't by itself confirm, this optional-closer rule (see
  Assumptions).
- **FR-010**: The parser MUST tokenize `@variable@` substitution syntax as its own
  token type, recording the variable name and its position, without evaluating or
  substituting a value for it. This applies whether `@variable@` appears bare in a
  keyword's value or inside a quoted string literal (e.g. `FILEO ... =
  '@ParentDir@@ScenarioDir@file.mtx'`) — multiple `@variable@` references, with or
  without literal text between them, may appear inside a single string.
- **FR-011**: The parser MUST treat control words and keywords case-insensitively
  (e.g. `IF`, `If`, and `if` are equivalent; the same applies to `ENDIF`, `LOOP`,
  `ENDLOOP`, `BREAK`, `RUN`, `PGM`, `ENDRUN`).
- **FR-012**: The parser MUST emit a structured diagnostic — not a crash — for an
  unclosed or unmatched `IF`/`ENDIF` block, identifying the location of the offending
  statement.
- **FR-013**: The parser MUST emit a structured diagnostic for an unclosed or
  unmatched `LOOP`/`ENDLOOP` block.
- **FR-014**: The parser MUST emit a structured diagnostic for an unclosed block
  comment (a `/*` with no matching `*/` before end of input).
- **FR-015**: The parser MUST emit a structured diagnostic when a statement ends in a
  continuation character but is followed by no further line, or by a line that is not
  a valid continuation of that statement.
- **FR-016**: The parser MUST emit a structured diagnostic for an unclosed
  `RUN PGM=...`/`ENDRUN` block.
- **FR-017**: Each diagnostic MUST include, at minimum, a defect category, a message
  describing the problem in the project's own words, and the source location it
  applies to.
- **FR-018**: Where feasible, the parser MUST continue past a recorded defect and keep
  reporting on the remainder of the script, rather than stopping at the first
  diagnostic.
- **FR-019**: The parser MUST NOT perform per-program-box keyword validation,
  formatting, or semantic/reference checking — these remain explicitly out of scope
  for this phase.
- **FR-020**: The parser MUST parse `.block` files using the same grammar as `.s`
  files: a top-level `RUN PGM=.../ENDRUN` wrapper is never required around an entire
  file. `RUN PGM=.../ENDRUN` is one nestable block type among several (alongside
  `IF`/`ENDIF` and `LOOP`/`ENDLOOP`) that may appear zero or more times, anywhere,
  in a file's top-level statement sequence — not a mandatory outer boundary. This
  applies identically to `.s` and `.block` files (resolved by inspecting real
  fixtures; see Assumptions).
- **FR-021**: The parser MUST recognize a label statement — a line whose first
  non-whitespace character is `:` followed by an identifier (e.g. `:STEP0`) — as its
  own statement form, distinct from a control statement, so it is never misread as an
  unrecognized or malformed control word.
- **FR-022**: The parser MUST recognize a shell-command-escape statement — a line
  whose first non-whitespace character is `*`, optionally immediately followed by a
  second `*` (the two forms differ only in how the resulting command window is
  displayed, not in grammar), followed by arbitrary command text — as its own
  statement form, without attempting to parse that text as Voyager keyword=value
  pairs. Parentheses appearing in the command text (e.g. `*(ECHO done)`) are part of
  the command itself, not a delimiter the parser requires — a bare `*DEL file.tmp`
  with no parentheses at all is an equally valid shell-escape statement. The fixture
  corpus so far only shows the parenthesized style; vendor reference documentation
  confirms the bare style is also valid (see Assumptions).
- **FR-023**: The parser MUST recognize a plain assignment statement — an identifier
  followed directly by `=` and a value/expression, with no preceding control word — as
  a valid statement form, both at a file's top level (with no enclosing block) and
  nested inside a `RUN PGM=.../ENDRUN` block. A statement is `Assignment` rather than
  `Control` (FR-003) whenever its first token is not one of the recognized control
  words; no separate keyword=value pairs follow a bare `identifier=value`.
- **FR-024**: The project MUST record, for each grammar rule, which Cube Voyager
  version (baseline: 6.5) it was validated against, written in the project's own
  words rather than copied from vendor documentation.
- **FR-025**: The parser's behavior MUST be validated against a fixture corpus of
  real, working `.s`/`.block` scripts: it MUST produce zero false-positive diagnostics
  on every valid fixture, and MUST correctly flag every deliberately-broken fixture
  with a diagnostic matching its injected defect, covering every diagnostic category
  defined in this specification (FR-012–FR-016 and FR-026).
- **FR-026**: The parser MUST emit a structured diagnostic when a `BREAK` statement
  appears with no enclosing block of any kind at all — that is, at a file's true top
  level, not nested inside an `IF`, `LOOP`, `RUN`, `PROCESS`/`PHASE` (FR-028),
  `JLOOP` (FR-029), or `LINKLOOP` (FR-033) block — identifying the location of the
  offending `BREAK`. This is deliberately narrower than "outside a `LOOP`"
  specifically: vendor reference documentation confirms `BREAK` is legitimate,
  program-dependent syntax inside a `PROCESS`/`PHASE` stack in several Voyager
  programs (with Pilot being the one program that restricts it to `LOOP` only), and a
  structural-only parser has no way to tell those cases apart without knowing which
  program a `RUN PGM=` box is actually running — which is out of scope by design
  (FR-019). Requiring *some* enclosing block, of any kind, is the closest structural
  approximation available. This retains the same binding force, and is covered by
  the same fixture-corpus validation (FR-025), as the other block-matching
  diagnostics (FR-012, FR-013, FR-016).
- **FR-027**: The parser crate MUST be implemented using only Rust's standard
  library — it MUST NOT introduce third-party runtime dependencies. This keeps the
  single authoritative grammar implementation minimal and independently auditable for
  every downstream consumer (CLI, LSP server, MCP server, formatter) that will depend
  on it.
- **FR-028**: The parser MUST recognize and structurally match `PROCESS ...
  ENDPROCESS` blocks — the documented underlying block type — while also accepting
  its two common shortcuts: `PHASE=value` (a `keyword=value` pair optionally
  followed by further space-separated `keyword=value` pairs on the same statement,
  e.g. `PHASE=INPUT, FILEI=li.1`) as a trigger-keyword opener standing in for
  `PROCESS PHASE=value`, and `ENDPHASE` as an interchangeable spelling of
  `ENDPROCESS`. Any opener spelling may be closed by any closer spelling. As with
  `RUN`/`ENDRUN` (FR-009), an explicit closer is optional: a `PROCESS`/`PHASE=`
  block is also considered closed by whichever comes first of the next
  `PROCESS`/`PHASE=` statement. Confirmed against vendor reference documentation;
  the fixture corpus's 35-distinct-file, zero-unbalanced-pairs finding reflects real
  authors consistently writing an explicit closer as a matter of style, not evidence
  that the grammar requires one (see Assumptions).
- **FR-029**: The parser MUST recognize and structurally match `JLOOP ... ENDJLOOP`
  blocks as a loop-block type distinct from `LOOP`/`ENDLOOP` (FR-008), opened by
  `JLOOP` followed by space-separated `keyword=value` pairs (confirmed against real
  fixtures; 30 distinct files).
- **FR-030**: The parser MUST recognize and structurally match `DistributeMULTISTEP
  ... EndDistributeMULTISTEP` blocks — a parallel-processing sub-block construct
  distinct from `RUN`/`ENDRUN`, `LOOP`/`ENDLOOP`, and `PROCESS`/`PHASE`/`ENDPHASE`.
  These are the literal keywords `DistributeMULTISTEP`/`EndDistributeMULTISTEP`
  (case-insensitive per FR-011), not a generic `MULTISTEP` suffix pattern (confirmed
  against real fixtures; 8 distinct files, always sequential, never nested).

  *(FR-031 and FR-032, proposed alongside FR-028–030 for a hybrid `WORD=value
  keyword=value...` statement shape and a `FUNCTION { ... }` brace block, were never
  adopted as numbered requirements — the former remains a deferred, narrow-evidence
  finding [see Assumptions], the latter turned out to be a general mechanism folded
  into FR-006 instead of a `FUNCTION`-specific one. The numbering below picks up at
  FR-033 to keep this history traceable rather than reusing FR-031/032.)*
- **FR-033**: The parser MUST recognize and structurally match `LINKLOOP ...
  ENDLINKLOOP` blocks — a bare, argument-less loop-block type (shorthand for
  looping over a network's link records) distinct from `LOOP`/`ENDLOOP` (FR-008) and
  `JLOOP`/`ENDJLOOP` (FR-029). `LINKLOOP` may nest inside `IF`, `LOOP`, `RUN`, and
  `PROCESS`/`PHASE` blocks, but not inside another `LINKLOOP`. Promoted from a
  deferred, fixture-narrow finding (originally 3 files, one program box) to a full
  requirement on the strength of vendor reference documentation, which describes it
  independently in two unrelated program chapters (Highway and Public Transport) as
  general-purpose syntax, not a one-off — reversing the earlier fixture-only call to
  defer it (see Assumptions).

### Key Entities

- **Token**: The smallest recognized unit of source text (control word, keyword,
  operator/value fragment, comment — including a block comment nested inside another
  one (FR-005) — continuation marker, or `@variable@` reference), each carrying its
  position in the source.
- **Statement**: A control word plus its `keyword=value` pairs, which may span
  multiple physical lines joined by trailing-operator continuation or, for a
  `Control` statement, by a `{...}`-delimited body instead (FR-006).
- **Block**: A structural grouping formed by opening and closing statements —
  `IF…ENDIF` (or chain, for `IF`/`ELSEIF`/`ELSE`/`ENDIF`; including the self-closing
  short-`IF` form, FR-007), `LOOP…ENDLOOP`, `RUN PGM=...…ENDRUN` or `!RUN…ENDRUN`
  (FR-009), `PROCESS…ENDPROCESS` (with `PHASE=`/`ENDPHASE` as accepted opener/closer
  spellings, FR-028), `JLOOP…ENDJLOOP` (FR-029), `LINKLOOP…ENDLINKLOOP` (FR-033), or
  `DistributeMULTISTEP…EndDistributeMULTISTEP` (FR-030) — which may nest inside one
  another and may appear zero or more times at the top level of a file. `RUN` and
  `PROCESS`/`PHASE` blocks may also close implicitly, without any explicit closing
  statement, when the next same-family opener (or, for `RUN`, a shell-escape
  statement) is reached first (FR-009, FR-028); every other block kind requires an
  explicit closer. `BREAK` is a statement valid anywhere, but only diagnosable
  (FR-026) when it has no enclosing block of any kind — nested inside some block, the
  parser accepts it without judging whether that specific program gives it meaning
  there. `.s` and `.block` files share this same structure (see FR-020).
- **Label Statement**: A `:identifier` line (e.g. `:STEP0`) marking a named position
  in the file, distinct from a control statement.
- **Shell-Escape Statement**: A `*` or `**` line followed by arbitrary command text
  (e.g. `*(ECHO done)` or a bare `*DEL file.tmp`) that hands that text to the
  operating system shell rather than to Voyager's own grammar; any parentheses
  present are just part of the command text, not a required delimiter (FR-022).
- **Assignment Statement**: A plain `identifier = value` statement with no preceding
  control word.
- **Diagnostic**: A structured record of a parsing problem: its defect category
  (e.g. unmatched block, unclosed comment, invalid continuation), a description in
  the project's own words, and the source location it applies to.
- **Fixture Corpus**: The external, separately-supplied collection of real `.s`/
  `.block` scripts — both valid and deliberately broken — used to measure the parser's
  false-positive and false-negative rate; not itself part of this library.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every valid script in the fixture corpus parses with zero reported
  diagnostics.
- **SC-002**: Every deliberately-broken script in the fixture corpus produces at least
  one diagnostic that correctly identifies its injected defect.
- **SC-003**: Each diagnostic category defined in this specification (FR-012–FR-016
  and FR-026 — unmatched IF/ENDIF, unmatched LOOP/ENDLOOP, unclosed block comment,
  invalid/missing continuation, unclosed RUN/ENDRUN, and BREAK with no enclosing
  block at all) is exercised by at least one corpus fixture that triggers exactly
  that diagnostic. This criterion is defined by reference to the FR list, not a
  hardcoded count, so it still holds if a diagnostic category is added later.
- **SC-004**: A downstream tool can obtain a script's full statement/block structure
  and its full diagnostic list from a single call, supplying only the script's text —
  no file access or protocol dependency is required to get a result.
- **SC-005**: The `.block`-vs-`.s` grammar question (FR-020) is resolved — confirmed
  against real `.block` and `.s` examples that both file types share one grammar with
  no mandatory top-level `RUN`/`ENDRUN` wrapper — and at least one fixture of each
  observed shape (a bare-fragment `.block` and a self-contained, `RUN`/`ENDRUN`-wrapped
  `.block`) is included in the fixture corpus.
- **SC-006**: No diagnostic message or documented grammar rule reproduces vendor
  documentation text verbatim; each is independently reviewable as original wording.

## Assumptions

- **`.block`-vs-`.s` grammar (resolved)**: Real fixtures inspected in
  `WF-TDM-Official-Releases` confirm `.block` files are always included into a larger
  job via a `READ FILE='...'` statement in a `.s` file, and that `.s` files include
  each other the same way. Some `.block` fixtures
  (e.g. `4_ModeChoice/block/HBW_HBO_calculate_utilities.block`) are bare statement/
  `IF…ENDIF` sequences meant to be read while a `RUN PGM=.../ENDRUN` box is already
  open in the including file. Others
  (e.g. `0_InputProcessing/a_Setup/_TimeStamp_IP.block`) are self-contained, carrying
  their own `RUN PGM=.../ENDRUN` wrapper, and are read at a point where no box is
  currently open. `.s` files exhibit the identical pattern — top-level statements and
  zero or more `RUN PGM=.../ENDRUN` boxes, never a single wrapper around the whole
  file. One unified grammar (FR-020) covers both observed shapes without needing to
  detect which one applies; resolving cross-file `READ FILE` inclusion itself is a
  multi-file concern left to a later phase, not this one. **This conclusion rests
  entirely on fixture evidence, with zero documentary corroboration either way**: a
  verification pass against the vendor reference documentation (both the Voyager 6.5
  manual and the OpenPaths CUBE 7 help) found no mention of `.block` as a file
  convention anywhere — the docs only ever discuss `READ FILE=` generically, for any
  extension. Future readers shouldn't assume the `.block`-specific half of this
  finding is doc-backed just because the `READ FILE=` mechanism itself is.
- **Additional statement forms found in real fixtures**: label statements (`:STEP0`),
  shell-escape statements (`*(ECHO ...)`), and plain assignment statements
  (`ScriptStartTime = currenttime()`) all appear at the top level of real, working
  `.s` files inspected during this phase. These were not in the original feature
  description but must be recognized (FR-021–FR-023) to meet the zero-false-positive
  requirement (FR-025) against real scripts. The fixture corpus so far only shows the
  parenthesized shell-escape style (`*(ECHO ...)`); a later documentation
  cross-check found the bare style (`*DEL file.tmp`, no parentheses) is equally
  valid, which FR-022 now reflects (see below).
- The fixture corpus (real `.s`/`.block` scripts, both valid and deliberately broken)
  is supplied separately by the project, as noted in the feature description; this
  spec assumes it will exist in plain-text form comparable to real Voyager scripts and
  will cover each required diagnostic category. `WF-TDM-Official-Releases` is a
  plausible source for such fixtures given its real, working `.s`/`.block` files.
- The baseline grammar targets Cube Voyager 6.5. Syntax changes introduced by later
  (2024+ OpenPaths CUBE) releases are out of scope for this phase; a version flag or
  second fixture set may be introduced in a later phase if newer scripts diverge.
- **Deferred, narrow-evidence grammar finding (out of scope for this phase)**: A
  hybrid `WORD=value keyword=value...` statement shape (e.g. `COMBINE = EQUI
  ENHANCE=2, SMOOTH=1, MULTITHREAD=@CoresAvailable@, MEMORY=T`) was confirmed in
  real fixtures, but only within the same 3 files (`4pd_mainbody_distribution.block`,
  `4pd_mainbody_managedlanes.block`, `4pd_mainbody_managedlanes_SelectLink.block` —
  all one `RUN PGM=HWYASSIGN` box's `ADJUST` phase), unlike FR-028–FR-030's broad,
  multi-file evidence. It fits neither FR-003 nor FR-023 cleanly on its own, but a
  documentation cross-check traced it to a general, documented mechanism ("trigger
  keywords" — a program-specific keyword that may stand in for its statement's usual
  control word) rather than a novel third statement shape; since that mechanism's
  own keyword list is scattered per-program and not centrally enumerable, this
  remains unspecified by this phase's grammar rather than becoming a new FR — a
  future phase should revisit it if broader real-world usage is found. (Two sibling
  findings originally deferred alongside this one — a brace-delimited `FUNCTION {
  ... }` block, and a bare `LINKLOOP ... ENDLINKLOOP` block — were resolved this
  pass: see the `{...}` continuation note under FR-006 and FR-033 respectively,
  both promoted on documentation evidence rather than fixture breadth.)
- `BREAK` was originally treated as meaningful only nested inside a `LOOP` block,
  making a `LOOP`-less `BREAK` the sole trigger for FR-026. A documentation
  cross-check found this too strict: `BREAK` is legitimate, program-dependent syntax
  inside a `PROCESS`/`PHASE` stack in several Voyager programs (Highway among them),
  with Pilot being the one program that actually restricts it to `LOOP` only — and a
  structural-only parser has no way to tell those cases apart without per-program
  knowledge that's out of scope (FR-019). FR-026 now only fires when `BREAK` has no
  enclosing block *of any kind*, the closest structural approximation available; it
  remains a fully specified diagnostic requirement in its own right, with the same
  binding force as the other five block-matching/comment/continuation diagnostics.
- **`RUN`/`ENDRUN` and `PROCESS`/`PHASE`/`ENDPHASE` implicit closing (resolved via
  documentation, not fixtures)**: the fixture corpus's own "zero unbalanced pairs"
  findings for both block kinds (spec's original FR-009 note, and the 35-file
  finding under FR-028) reflect real authors consistently writing an explicit
  closer — not evidence the grammar requires one. Vendor reference documentation is
  explicit that both block kinds accept an implicit close (by the next same-family
  opener, or, for `RUN`, a shell-escape statement) with no explicit closer at all;
  FR-009 and FR-028 now specify this. The disabled `!RUN` form is documented as the
  one exception — it keeps the strict, explicit-closer-required rule.
- **Short-`IF` (resolved via documentation, not fixtures)**: vendor reference
  documentation describes a self-closing single-line `IF` form (FR-007) that the
  fixture corpus has not been confirmed to contain either way. Until a fixture
  exercises it, its "zero false positives" claim (SC-001) rests on documentation
  alone for this one construct.
- **Block comment nesting (resolved via documentation, not fixtures)**: vendor
  reference documentation is explicit that `/* ... */` blocks nest (FR-005); the
  fixture corpus hasn't been confirmed to contain a nested block comment either way,
  same caveat as short-`IF` above.
- **Continuation character "proper context" (accepted limitation, out of scope)**:
  vendor reference documentation notes that a trailing continuation character (FR-006)
  is only a *real* continuation when it's "in proper context" for the statement being
  written — its own example shows a trailing `&` that *doesn't* actually continue the
  statement because a logical-AND doesn't fit grammatically at that point. This
  parser does not attempt that judgment: every trailing occurrence of the nine
  continuation characters is treated as continuing the statement, unconditionally.
  That's a deliberate simplification — resolving it fully would mean parsing
  expression grammar, which is semantic/expression-level work FR-019 excludes — and
  it's the safer direction per constitution Principle IV (false negatives preferred
  over false positives): being more permissive about what counts as a continuation
  can't itself produce a spurious diagnostic.
- **Blank-line skipping and the `{...}` continuation form (resolved via
  documentation, not fixtures)**: vendor reference documentation confirms that fully
  blank lines between a continuation-ending line and the line that resumes it are
  skipped rather than breaking the continuation, and that a `{...}`-delimited body is
  a second, general continuation mechanism available after any control word
  (FR-006). Neither has a confirmed fixture example yet; same caveat as short-`IF`
  and block-comment nesting above.
- Diagnostics are structured data (category, message, location) rather than opaque
  free-text strings, so that downstream tools (linter, LSP, CLI) can render, filter, or
  translate them without needing to parse a message string.
- **Assignment statement validity contexts (confirmed)**: Real fixtures confirm plain
  assignment statements (FR-023) are valid both bare at a file's top level, with no
  enclosing block at all (e.g. `ScriptStartTime = currenttime()` appearing before any
  `RUN PGM=.../ENDRUN` in dozens of `.s` files), and nested inside a `RUN PGM=.../
  ENDRUN` block (e.g. `EndTime_IP = currenttime()` inside `RUN PGM=MATRIX ...
  ENDRUN`). No "PAR"/"COMP"-style wrapper construct was found in the corpus. A full
  enumeration of the recognized control-word set (to sharpen FR-003's boundary with
  FR-023) was not finalized as part of this pass — see project notes for the raw
  census.
- **`@variable@` inside quoted strings (confirmed)**: Real fixtures inspected in
  `WF-TDM-Official-Releases` confirm `@variable@` substitution inside quoted string
  literals (e.g. `FILEI MATI[01] = '@ParentDir@@ScenarioDir@...\PA_AllPurp.mtx'`) is
  the dominant real-world usage, not an edge case — most `FILEI`/`FILEO`/`PRINT FILE=`
  paths are built this way, often with multiple `@variable@` references (with or
  without literal text between them) inside a single string.
- **Continuation scope for non-Control statement forms (partially confirmed)**: Real
  fixtures confirm `Assignment` statements continue across physical lines using the
  same trailing characters as `Control` statements (e.g. a multi-term arithmetic sum
  spanning several lines, each ending in `+`). No real example of a continued `Label`
  or `ShellEscape` statement was found either way — both are inherently short in
  practice, so this is treated as an open question about real-world occurrence, not
  evidence that the grammar disallows it.
- This library exposes its functionality as callable functions/types only; no
  particular API shape (e.g. streaming vs. whole-document) is mandated by this spec,
  since that is an implementation decision for the planning phase.
