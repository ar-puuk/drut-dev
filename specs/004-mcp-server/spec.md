# Feature Specification: Drut MCP Server

**Feature Branch**: `004-mcp-server`

**Created**: 2026-08-10

**Status**: Draft

**Input**: User description: "Build the MCP server for Drut, as a thin adapter over
the voyager-core library crate (constitution Principle I) — the fourth adapter
named in the constitution alongside the CLI, LSP server, and formatter, none of
which this server may duplicate any grammar/parsing/lint-rule logic from. Expose
Cube Voyager script tooling to MCP-capable AI agents/clients via a set of MCP
tools, read-only (no tool writes to disk — every tool takes script text or a file
path as input and returns a structured result; the calling agent decides what to
do with it, mirroring how drut-cli's --write is the only place in this whole
project that ever touches disk): a diagnostics tool (all six reachable diagnostic
categories), a formatting tool (voyager_core::format, same semantics as `drut
format --diff` returned as data), a keyword-lookup tool (completion_candidates/
did_you_mean), and a structural-query tool (the hover-equivalent — block kind and
matched-counterpart location for a position, reusing the same 5-rule derivation
drut-lsp's hover already implements, never re-derived independently). Aim for the
same completeness/quality bar a genuinely excellent language-tooling MCP server
would have (the user's own reference point: Posit's Ark, for R) — not a minimal
stub. Out of scope: writing to disk from any tool; per-program-box keyword
validation; repo-wide/multi-file semantic checking; anything requiring a running
`drut server`/LSP session — this MCP server is its own standalone process/binary,
depending on voyager-core independently, never on drut-cli or drut-lsp."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - An AI agent validates a script it just wrote or edited (Priority: P1)

An AI coding agent (e.g. inside an MCP-capable client) has just generated or
modified a Cube Voyager `.s`/`.block` script and wants to know, before showing it
to the user or writing it to disk, whether the script is structurally sound. It
calls the diagnostics tool with the script's text and receives back every
structural defect voyager-core can detect — the same six categories `drut check`
and `drut-lsp` already surface — with enough location detail to point at the
exact problem.

**Why this priority**: This is the single highest-value, lowest-complexity tool —
a direct wrapper around `parse`/`parse_bytes` with zero new logic beyond
serialization, and the most immediately useful capability for an agent that's
actively writing or editing scripts, mirroring why `drut check` was the first
capability built in `002-cli-check-format`.

**Independent Test**: Call the diagnostics tool with a script text containing a
deliberately unmatched `IF` (no `ENDIF`) and confirm the result reports exactly
one `UnmatchedIf` diagnostic with a correct location; call it again with a
structurally valid script and confirm zero diagnostics are returned.

**Acceptance Scenarios**:

1. **Given** script text with one deliberately unmatched `IF`, **When** the
   diagnostics tool is called with that text, **Then** the result contains
   exactly one diagnostic of category `UnmatchedIf` with a location pointing at
   the unmatched opener.
2. **Given** a structurally valid script (matching one of the fixture corpus's
   "valid" fixtures), **When** the diagnostics tool is called, **Then** the
   result contains zero diagnostics.
3. **Given** a file path instead of inline text, **When** the diagnostics tool is
   called with that path, **Then** the result is identical to calling it with
   that file's own text content directly.

---

### User Story 2 - An AI agent normalizes a script's whitespace/casing before presenting it (Priority: P2)

An agent wants to offer the user a cleanly formatted version of a script — either
one it generated itself or one the user already has — without guessing at
indentation or keyword casing conventions by hand. It calls the formatting tool
and receives back the fully reformatted text plus whether anything actually
changed, exactly matching what `drut format --diff` would show.

**Why this priority**: Directly reuses `002-cli-check-format`'s already-built,
already-golden-fixture-tested formatting engine with no new formatting logic of
its own — same low-risk, high-value shape as User Story 1, just for the
formatter instead of the checker.

**Independent Test**: Call the formatting tool with a script whose body
indentation doesn't match its block nesting and confirm the returned text has
correct indentation and `changed: true`; call it again with an
already-correctly-formatted script and confirm `changed: false` and the
returned text is byte-identical to the input.

**Acceptance Scenarios**:

1. **Given** script text with incorrect body indentation inside a block, **When**
   the formatting tool is called, **Then** the result's text has the body
   correctly indented relative to its block's opener, and `changed` is `true`.
2. **Given** already-correctly-formatted script text, **When** the formatting
   tool is called, **Then** the returned text is identical to the input and
   `changed` is `false`.
3. **Given** the same script formatted twice in a row (the tool's own output fed
   back in as input), **When** the formatting tool is called the second time,
   **Then** `changed` is `false` (idempotence, matching `voyager_core::format`'s
   own existing guarantee).

---

### User Story 3 - An AI agent asks what block a position in a script belongs to (Priority: P3)

An agent working with a specific location in a script (e.g. a line number a
diagnostic pointed at, or a spot the user is asking about) wants to know which
of the seven block kinds — if any — encloses that position, and where that
block's matched counterpart is, including through `Run`/`Process`'s
implicit-close quirk, without re-deriving that logic itself.

**Why this priority**: Reuses `drut-lsp`'s existing 5-rule counterpart-derivation
logic verbatim rather than re-implementing it, and is meaningfully more useful
once an agent already has diagnostics in hand (User Story 1) and wants to
understand the surrounding structure of a flagged location — a natural
second-order need, not a first one.

**Independent Test**: Call the structural-query tool with a script containing an
implicitly-closed `RUN` block and a position on that block's opener line, and
confirm the result reports block kind `Run` and a matched-counterpart location
at the block's resolved implicit-close point (not `null`, and not the next
`RUN`'s own opener line).

**Acceptance Scenarios**:

1. **Given** a script with an explicitly-closed `IF`/`ENDIF` and a position on
   the `IF` line, **When** the structural-query tool is called, **Then** the
   result reports block kind `If` and a matched-counterpart location at the
   `ENDIF` line.
2. **Given** a script with a `RUN` block closed implicitly by a second `RUN`
   opener, and a position on the first `RUN`'s line, **When** the tool is
   called, **Then** the result reports block kind `Run` and a matched-
   counterpart location at the first `RUN`'s own resolved body extent — the
   same location `drut-lsp`'s hover would report for the identical case.
3. **Given** a position that falls on a line with no enclosing block at all,
   **When** the tool is called, **Then** the result reports no enclosing block,
   not an error.

---

### User Story 4 - An AI agent looks up valid keyword-pair names for a control word it's writing (Priority: P4)

An agent that knows it's constructing (or reviewing) a statement under a specific
control word — e.g. `RUN` — wants to know which `keyword=value` pair names are
actually observed in real usage under that control word, or whether a keyword it
typed is a plausible misspelling of a real one, without needing to construct a
full script and cursor position just to ask a vocabulary question.

**Why this priority**: Lowest-complexity of the four (direct wrapper over
`keywords.rs`'s already-built `completion_candidates`/`did_you_mean`), and the
most exploratory/advisory of the four tools — valuable, but the least essential
to get right first relative to validating (P1), fixing (P2), and understanding
(P3) an actual script.

**Independent Test**: Call the keyword-lookup tool with enclosing control word
`RUN` and confirm the result includes `PGM`, `MSG`, and `PRNFILE` (drut-lsp's
own real, corpus-census-derived data for `RUN`); call it with a misspelled token
close to a real keyword and confirm a "did you mean" suggestion is returned.

**Acceptance Scenarios**:

1. **Given** enclosing control word `RUN`, **When** the keyword-lookup tool is
   called, **Then** the result's candidate list includes `PGM`, `MSG`, and
   `PRNFILE`.
2. **Given** no enclosing control word at all, **When** the keyword-lookup tool
   is called, **Then** the result's candidate list is the general-syntax control
   word list (the same fallback `keywords.rs`'s `completion_candidates`
   already implements).
3. **Given** a token one edit-distance from a real keyword and not itself a
   real keyword, **When** the keyword-lookup tool is called with that token as
   a spell-check query, **Then** the result includes a "did you mean" suggestion
   naming the real keyword.
4. **Given** a token that already exactly matches a real keyword, **When** the
   keyword-lookup tool is called with that token as a spell-check query,
   **Then** the result reports no suggestion (nothing to correct).

### Edge Cases

- What happens when a tool is called with a file path that doesn't exist or
  isn't readable? The tool MUST return a clear, structured error result — never
  a panic, never a silent empty/default result indistinguishable from "no
  problems found."
- What happens when both script text and a file path are supplied to the same
  call? One unambiguous rule MUST apply consistently across every tool (see
  FR-002), not a per-tool guess.
- What happens when a tool is called with empty script text? Every tool MUST
  handle it the same way its underlying `voyager-core`/`drut-lsp` logic already
  does for empty input (e.g. the diagnostics tool returns zero diagnostics, not
  an error) — empty input is valid input, not an edge case requiring special
  handling.
- What happens when the structural-query tool is given a position beyond the
  end of the script (a stale or out-of-range line/column)? It MUST clamp to the
  nearest valid position and answer for that, the same no-panic clamping
  discipline `drut-lsp`'s own position-encoding translation already follows,
  never index out of bounds.
- What happens when a script contains a byte sequence that isn't valid UTF-8
  (only reachable via the file-path input, never via inline text — an MCP
  tool-call argument is JSON, which cannot carry invalid bytes any more than
  an LSP payload can, the same structural reason `InvalidEncoding` is
  unreachable through live LSP editing, `003-lsp-vscode-extension` research.md
  §12)? The diagnostics tool MUST surface an `InvalidEncoding` diagnostic for
  it, the same as `drut check` already does when reading a file directly from
  disk.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The server MUST expose exactly four MCP tools this phase: a
  diagnostics tool, a formatting tool, a structural-query tool, and a
  keyword-lookup tool — no other tools, and no tool that writes to disk.
- **FR-002**: Every tool that accepts script content MUST accept it as either
  inline text or a file path, never both required and never both silently
  accepted at once — if a caller supplies both in the same call, the tool MUST
  return a structured error rather than silently preferring one.
- **FR-003**: The diagnostics tool MUST return every diagnostic category
  `voyager-core`'s `parse`/`parse_bytes` can produce that is reachable through
  this tool's own input path (inline text can never trigger `InvalidEncoding`,
  the same structural reason it's unreachable through live LSP editing; a file
  path can, the same way `drut check` already surfaces it) — never a narrowed
  subset chosen by this server.
- **FR-004**: The formatting tool MUST return both the fully reformatted text
  and an explicit `changed` boolean (byte-level comparison against the
  original input) — never text alone, so a caller can distinguish "nothing to
  change" from "here is different text" without doing its own comparison.
- **FR-005**: The formatting tool MUST NOT apply keyword-casing normalization
  by default (matching `voyager_core::format`'s own `FormatOptions::default()`
  behavior, FR-015 of `001-voyager-script-parser`) — an opt-in casing
  parameter MAY be exposed, mirroring `drut format --casing`, but is not
  required this phase if it isn't reached.
- **FR-006**: The structural-query tool MUST derive block-kind and
  matched-counterpart information using the exact same 5-rule derivation
  `drut-lsp`'s hover capability already implements (data-model.md §4 in
  `003-lsp-vscode-extension`) — never a reimplementation, however similar, of
  that same logic.
- **FR-007**: The structural-query tool MUST report "no enclosing block" as a
  normal, successful result (not an error) when a queried position falls
  outside every block in the script.
- **FR-008**: The keyword-lookup tool MUST accept an enclosing control word as
  a direct string parameter (not derived from a script and a position) and
  return keyword-pair-name candidates scoped to it, falling back to the full
  general-syntax control-word list when no control word is supplied — the same
  scoping/fallback behavior `keywords.rs`'s `completion_candidates` already
  implements.
- **FR-009**: The keyword-lookup tool MUST also accept a token for "did you
  mean" spell-check lookup, independently of the candidate-listing capability
  in FR-008 — a single call MAY request either, both, or neither depending on
  what the caller supplies.
- **FR-010**: No tool MUST write to any file, under any input or parameter
  combination — every tool's only effect is its returned result value.
- **FR-011**: No tool's behavior MUST depend on a running `drut server`/LSP
  session, or on any state shared with `drut-cli` — this server MUST be
  runnable as its own independent process with no dependency on either sibling
  adapter, only on `voyager-core` directly (constitution Principle I).
- **FR-012**: Every tool MUST return a well-formed, structured result for any
  input, including malformed or edge-case input (per the Edge Cases above) —
  never panic, and never crash the server process, matching the same no-panic
  contract `voyager-core` and `drut-lsp` already guarantee for their own
  inputs.
- **FR-013**: Every tool's result schema MUST be self-describing enough
  (named fields, not positional/opaque data) that a calling agent can use it
  correctly without having read this project's source code — matching this
  spec's aim of a genuinely complete tool surface, not a minimal stub with an
  underspecified contract.

### Key Entities

- **Diagnostic** (tool result element): mirrors `voyager-core`'s existing
  `Diagnostic` — category, location, and message, exactly as `drut check`/
  `drut-lsp` already expose it, not a redefined shape.
- **FormatResult** (tool result): reformatted text, a `changed` boolean, and
  whatever `voyager-core`'s `FormatResult` already carries beyond that (e.g.
  encoding fidelity) that's meaningful for a caller receiving the result as
  data instead of a CLI diff.
- **BlockInfo** (structural-query tool result): block kind (one of the seven,
  or "none"), and a matched-counterpart location when one is resolved —
  mirrors `drut-lsp` hover's own resolved fact shape.
- **KeywordCandidate** / **SpellCheckSuggestion** (keyword-lookup tool result
  elements): mirror `keywords.rs`'s existing `KeywordEntry`/`did_you_mean`
  output shapes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An AI agent can validate a script's structural correctness (all
  six reachable diagnostic categories) in a single tool call, with no
  additional round trips needed to interpret the result.
- **SC-002**: An AI agent can obtain a fully reformatted version of a script,
  and know with certainty whether anything changed, in a single tool call.
- **SC-003**: An AI agent can determine which block encloses a given script
  position and where that block's matched counterpart is — including through
  the `Run`/`Process` implicit-close quirk — in a single tool call, with
  results identical to what a human would see hovering the same position in
  the VS Code extension.
- **SC-004**: An AI agent can obtain real-usage-evidenced keyword-pair
  candidates for a control word, or a spelling correction for a suspected
  typo, in a single tool call.
- **SC-005**: The server never writes to disk under any tool call, verified by
  a test suite that exercises every tool against a read-only filesystem
  fixture location.
- **SC-006**: The server passes the same full 161-file corpus validation
  already proven for `voyager-core`, `drut-cli`, and `drut-lsp` — zero panics,
  and diagnostic output identical to `drut check`'s own output for every file.

## Assumptions

- **Transport is stdio, matching this project's own precedent** — `drut-cli`'s
  `server` subcommand and `drut-lsp` both already communicate over stdio with
  no network component; this server follows the same shape (its own
  standalone binary/process, launched by an MCP-capable client, no HTTP/SSE
  transport this phase) unless research surfaces a concrete reason to diverge.
- **Every tool operates on a single document at a time** — no directory
  traversal, no multi-file batch calls this phase, the same single-document
  scope `003-lsp-vscode-extension` held itself to (repo-wide/multi-file
  checking remains a hypothetical later phase, named there and reaffirmed
  out-of-scope here).
- **"Genuinely excellent quality bar, not a minimal stub" (the feature
  description's own framing, referencing Posit's Ark for R) is a completeness
  and result-usability standard, not a literal interface to imitate** — this
  spec does not claim or assume specific knowledge of Ark's actual tool
  surface; FR-013's self-describing-result requirement and the four tools'
  own detailed acceptance scenarios are this spec's concrete interpretation
  of that bar, not a port of any other project's design.
- **Which Rust MCP SDK/crate to depend on is an implementation decision**,
  deferred to `/speckit-plan`'s research phase — the same way `lsp-server`/
  `lsp-types`'s selection was resolved in `003-lsp-vscode-extension/
  research.md` rather than decided at the spec stage.
- **No verbatim vendor documentation applies here the same as everywhere else
  in this project** (constitution Principle II) — nothing in this feature
  depends on or reproduces any Cube Voyager vendor documentation; all four
  tools are thin wrappers over already-real-usage-evidenced `voyager-core`/
  `keywords.rs` logic.
