# Phase 1 Data Model: Voyager Script Tokenizer & Structural Parser

Entities below correspond to the Key Entities in spec.md, refined with field-level
detail and validation rules drawn from the Functional Requirements. This is a
conceptual model — exact Rust type/field names are an implementation detail for
`tasks.md`, not fixed here.

## Span

The source-location primitive every other entity carries.

| Field | Type | Notes |
|---|---|---|
| `start_line` | integer (1-based) | FR-002 |
| `start_column` | integer (1-based) | FR-002 |
| `end_line` | integer (1-based) | Needed for multi-line tokens (block comments, continued statements) |
| `end_column` | integer (1-based) | |

**Validation rule**: `end` is never before `start`.

## Token

The smallest recognized lexical unit.

| Field | Type | Notes |
|---|---|---|
| `kind` | `TokenKind` (enum) | See variants below |
| `span` | `Span` | FR-002 |
| `text` | string slice/owned string | Raw source text this token covers |

**`TokenKind` variants** (FR-004, FR-005, FR-006, FR-010):
- `Word` — a control word, keyword, identifier, or value fragment.
- `LineComment` — `; ...` to end of line.
- `BlockComment` — `/* ... */`, possibly spanning lines and possibly nested: a `/*`
  found while a block comment is already open starts its own `BlockComment` token
  inside the outer one (FR-005), rather than being ordinary commented-out text.
- `ContinuationMarker` — the trailing `, + - / * ^ & | =` character that joins a
  statement to its next physical line.
- `VariableRef` — an `@name@` substitution reference; carries the variable name
  separately from its `@...@` delimiters (FR-010).
- `Punctuation` — structural characters not covered above (e.g. `=` when not a
  continuation marker, brackets in `mw[104][j]`-style array indexing, parens, and
  `{`/`}` when they aren't opening/closing a brace-delimited statement body — see
  Statement § validation rules for when they are).

**Validation rules**:
- A `LineComment`/`BlockComment` token never contributes to continuation detection
  except through the "last non-comment character" rule (FR-006) — the lexer looks
  *past* trailing comment tokens on a line, not at the literal last character.
- An unterminated `BlockComment` at end-of-input is still emitted as a token (so
  position info is available for the diagnostic) but is marked/flagged unterminated
  (FR-014).
- Block comments nest: a `/*` opened while one is already open produces its own
  inner `BlockComment` token, and the outer one isn't complete until every comment
  nested inside it has closed with its own `*/` (FR-005). "Unclosed at end-of-input"
  (FR-014) is anchored at whichever `/*` — outer or inner — never found its match.

## Statement

A logical unit of Voyager script, possibly spanning multiple physical lines joined by
continuation.

| Field | Type | Notes |
|---|---|---|
| `kind` | `StatementKind` (enum) | See variants below |
| `span` | `Span` | Covers all joined physical lines (FR-002) |
| `tokens` | `Vec<Token>` | The statement's own tokens, continuation markers excluded or marked (implementation choice) |

**`StatementKind` variants** (FR-003, FR-021, FR-022, FR-023):
- `Control { word: String, pairs: Vec<(String, Vec<Token>)> }` — control word plus
  zero or more `keyword=value` pairs; `word` and each `keyword` are compared
  case-insensitively (FR-011).
- `Assignment { target: String, value: Vec<Token> }` — a plain `identifier = value`
  statement with no control word (FR-023).
- `Label { name: String }` — a `:identifier` line (FR-021).
- `ShellEscape { command_tokens: Vec<Token> }` — a `*` or `**` line; the command text
  that follows (parenthesized or not) is stored opaquely, not parsed as Voyager
  grammar (FR-022). Parentheses, if present, are just part of `command_tokens`, not a
  delimiter the parser looks for.

**Validation rules**:
- A statement continues onto the next physical line iff the last non-comment,
  non-whitespace character on the current line is one of `, + - / * ^ & | =`
  (FR-006). If that condition holds and there is no following line, or the following
  line is not a valid continuation, this is a diagnostic (FR-015), not a silent
  truncation. Any number of fully blank lines between the continuation-ending line
  and the line that actually resumes the statement are skipped and don't themselves
  break the continuation (FR-006).
- A `Control` statement may instead be continued with a second, independent
  mechanism: a `{` immediately after the control word opens a body that runs — across
  any number of physical lines, none needing a trailing continuation character —
  until the next `}` (FR-006). A single statement is built with one continuation
  mechanism or the other, never a mix of both.
- Control words/keywords recognized as `IF`, `ELSEIF`, `ELSE`, `ENDIF`, `LOOP`,
  `ENDLOOP`, `BREAK`, `RUN`, `!RUN`, `ENDRUN`, `PROCESS`, `PHASE`, `ENDPROCESS`,
  `ENDPHASE`, `JLOOP`, `ENDJLOOP`, `LINKLOOP`, `ENDLINKLOOP` are case-insensitive
  (FR-011) and, for block openers/closers, feed directly into Block construction
  below rather than being "just" a `Control` statement with no further structure.
- An `IF (...)` statement followed on the same physical line by exactly one further
  statement is a short-`IF` (FR-007): that trailing statement is consumed as the
  `IF`'s entire body and the resulting `If` block (see Block below) is already
  complete — it does not wait for a later `ENDIF` token.

## Block

A structural grouping formed by opening and (explicit or implicit) closing
statements; may nest.

| Field | Type | Notes |
|---|---|---|
| `kind` | `BlockKind` (enum) | See variants below |
| `span` | `Span` | From the opening statement to the closing statement — explicit or implicit — or to end-of-input if genuinely unmatched |
| `children` | `Vec<Node>` | Nested statements and/or blocks, in source order |

**`BlockKind` variants** (FR-007, FR-008, FR-009, FR-020, FR-028, FR-029, FR-030,
FR-033):
- `If { branches: Vec<IfBranch> }` — one `IfBranch` per `IF`/`ELSEIF`/`ELSE` clause,
  each with its own condition tokens (for `IF`/`ELSEIF`) and its own `children`. A
  short-`IF` (FR-007) is represented the same way, as a single-branch `If` with no
  `ELSEIF`/`ELSE`: that branch's `children` is exactly the one statement trailing the
  `IF` on its line, and the block's `span` ends there — no `ENDIF` token is expected
  or consumed for it.
- `Loop { .. }` — from `LOOP` to matching `ENDLOOP`; `children` may contain a `BREAK`
  statement anywhere within (FR-008).
- `Run { pgm: Option<String>, disabled: bool, .. }` — from `RUN PGM=...` (or, when
  `disabled` is true, `!RUN`) to a closer. A non-`disabled` `Run` closes at an
  explicit `ENDRUN`, *or* implicitly at whichever comes first of the next
  `RUN`/`!RUN` statement or a shell-escape statement — none of which it consumes as
  its own closing token, since they belong to what follows (FR-009). A `disabled`
  (`!RUN`) `Run` does not get implicit closing; it always needs its own explicit
  `ENDRUN`.
- `Process { name: Option<String>, .. }` — the block underlying `PROCESS ...
  ENDPROCESS`; `name` captures the phase name however it was written (`PROCESS
  PHASE=name` or the `PHASE=name` shortcut). Closes at an explicit `ENDPROCESS` or
  `ENDPHASE` (interchangeable), or implicitly at the next `PROCESS`/`PHASE=`
  statement, mirroring `Run`'s implicit-close rule (FR-028).
- `JLoop { .. }` — from `JLOOP` to matching `ENDJLOOP`; may nest inside `If` or
  `Loop` blocks but not inside another `JLoop` (FR-029).
- `LinkLoop { .. }` — from `LINKLOOP` to matching `ENDLINKLOOP`; may nest inside
  `If`, `Loop`, `Run`, or `Process` blocks but not inside another `LinkLoop`
  (FR-033).
- `DistributeMultistep { process_num: Option<String>, .. }` — from
  `DistributeMULTISTEP` to matching `EndDistributeMULTISTEP`; observed always
  sequential and never nested (FR-030).

**Validation rules**:
- Blocks may appear zero or more times at the top level of a file, and nest to
  arbitrary depth; a file's top level is not required to be wrapped in a single
  `Run` block (FR-020) — this applies identically to `.s` and `.block` input.
- `BREAK` with no enclosing block *of any* `BlockKind` (`If`, `Loop`, `Run`,
  `Process`, `JLoop`, or `LinkLoop`) is a defect (see Diagnostic below); nested
  inside any block kind, it's accepted structurally without the parser judging
  whether that particular program actually gives it meaning there (FR-019 scope —
  see spec.md Assumptions).
- An opener with no matching closer before end-of-input produces a diagnostic
  anchored at the *opener's* location, but only for the three diagnosed block
  kinds — `If` → FR-012, `Loop` → FR-013, `Run` → FR-016 — and parsing continues
  past it rather than aborting (FR-018). For a non-`disabled` `Run`, an implicit
  closer (the next `RUN`/`!RUN` or a shell-escape statement) counts the same as an
  explicit `ENDRUN`; the diagnostic only fires when neither is found. `Process`
  blocks close the same way (explicit `ENDPROCESS`/`ENDPHASE`, or implicitly by the
  next `PROCESS`/`PHASE=`) but aren't a diagnosed block kind — like `JLoop`,
  `LinkLoop`, and `DistributeMultistep`, an unmatched `Process` at end-of-input is
  accepted structurally (its `span` simply extends to end-of-input) with no
  diagnostic, since FR-025's required categories cover only `If`/`Loop`/`Run`/
  `BREAK` (see contracts/diagnostics.md's note on block kinds without a diagnostic
  category).
- A closer with no matching opener anywhere earlier (a "dangling close") is also a
  defect for the same three diagnosed kinds — covered by the matching
  `UnmatchedIf`/`UnmatchedLoop`/`UnmatchedRun` diagnostic kind, anchored at the
  closer's location. This includes an `ENDIF` appearing after a short-`IF` has
  already self-closed: that `ENDIF` has no open `IF` to match, the same as any other
  dangling closer.

## Diagnostic

A structured record of a parsing problem.

| Field | Type | Notes |
|---|---|---|
| `kind` | `DiagnosticKind` (enum) | `UnmatchedIf`, `UnmatchedLoop`, `UnclosedBlockComment`, `InvalidContinuation`, `UnmatchedRun`, `MisplacedBreak` (see research.md §2) |
| `span` | `Span` | Anchored at the offending statement/token (FR-012–FR-016, FR-026) |
| `message` | `String` | Original wording, composed once per kind (constitution Principle II, FR-024) |

**Validation rules**:
- Every `DiagnosticKind` maps to exactly one of the six required categories
  (FR-012–FR-016, FR-026) — downstream tools should not assume the kind set is closed
  at six; new structural-defect kinds may be added later within the same
  structural-not-semantic scope.
- Diagnostics never cause the parse to abort early; the parser accumulates them and
  keeps going wherever structurally feasible (FR-018).

## ParseResult (top-level output)

The aggregate value returned to a caller for one input file's text.

| Field | Type | Notes |
|---|---|---|
| `nodes` | `Vec<Node>` where `Node = Statement | Block` | Top-level sequence, source order (FR-020) |
| `diagnostics` | `Vec<Diagnostic>` | Possibly empty (SC-001) |

**Validation rule**: `nodes` and `diagnostics` are always both populated from a single
call over the full input text; there is no partial/streaming API in this phase (spec
Assumptions: API shape beyond "whole text in, full result out" is undecided and left
open, but this phase's own contract commits to whole-document parsing — see
contracts/public-api.md).

## Fixture Corpus (test-only, not a runtime entity)

Not part of the library's data model — an external, supplied collection of real
`.s`/`.block` scripts (valid and deliberately broken) used only by the test suite to
measure false-positive/false-negative rate (FR-025, SC-001–SC-003). See research.md
§3 for its sourcing/licensing status.
