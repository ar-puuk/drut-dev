# Feature Specification: Undefined `@token@` Diagnostic

**Feature Branch**: `020-undefined-token-diagnostic`

**Created**: 2026-08-17

**Status**: Draft

**Input**: User description: "if a variable is used without specifying it, a red squiggly
underline should highlight it" — scoped down through direct conversation to `@token@`
substitution references specifically (the only kind with existing resolution logic), a
confidence bar that never treats a resolver blind spot as evidence of non-existence, and
Hint/Information severity rather than the originally-requested Error/"red squiggly" (constitution
Principle IV: false positives are worse than false negatives). Full design history — including
the correction that this needs no `001-voyager-script-parser` spec amendment, once checked
against the two existing same-shape precedents — lives in `ROADMAP.md` item 14.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A developer notices an unresolvable `@token@` reference while editing (Priority: P1)

While editing a `.s`/`.block` file, a developer references `@ScenarioDir@` (or any other
`@token@`) that turns out to have no `TOKEN = value` assignment anywhere the tool can find —
maybe it was renamed elsewhere, never defined in this file's own control-center chain, or simply
mistyped. Today nothing signals this until the script runs and Cube Voyager itself fails or
substitutes an empty value. A subtle, non-alarming underline in the editor gives an earlier
signal, without the tool claiming more certainty than it actually has.

**Why this priority**: This is the entire feature — there is no secondary priority tier here,
unlike prior bundled features. It's independently valuable and independently shippable: a
developer benefits from this signal the moment it exists, with nothing else needing to ship
alongside it.

**Independent Test**: Open a `.s`/`.block` file containing a `@token@` reference with no
resolvable definition anywhere in the file (and no `READ FILE` inclusion that would supply one),
and confirm a Hint/Information-severity underline appears at that reference — while a second
`@token@` reference in the same file that *does* have a same-file `TOKEN = value` assignment
receives no such underline.

**Acceptance Scenarios**:

1. **Given** a document containing `@ScenarioDir@` with no `ScenarioDir = value` assignment
   anywhere in the file and no `READ FILE` statement that could supply one, **When** the
   document is opened or edited, **Then** the editor shows a Hint/Information-severity underline
   at that `@ScenarioDir@` reference — visually and programmatically distinct from the six real
   structural diagnostics (which publish at Error severity).
2. **Given** the same document also contains `@Prog@` with a same-file `Prog = MATRIX`
   assignment earlier in the file, **When** the document is edited, **Then** `@Prog@` receives no
   underline at all.
3. **Given** a document containing `@Prog@` used only on a block-opener line (e.g.
   `RUN PGM=@Prog@`) with no assignment findable through the existing resolution logic, **When**
   the document is edited, **Then** `@Prog@` receives no underline — a resolver blind spot is
   never itself treated as evidence the token is undefined.
4. **Given** a document containing `@ParentDir@` whose only assignment lives two `READ FILE`
   levels away (a file this document includes, which itself includes another file that defines
   it), **When** the document is edited, **Then** `@ParentDir@` receives no underline — the same
   one-level-of-inclusion boundary the existing hover feature already has.
5. **Given** a document containing `@Prog@` whose value is built from another token (e.g. a
   `READ FILE` path like `'@ParentDir@config.s'`, not a literal path), **When** the document is
   edited, **Then** any token that could only be resolved by following that dynamic path
   receives no underline — dynamic/token-built inclusion paths are a known, accepted resolver
   blind spot, not evidence of non-existence.

---

### Edge Cases

- What happens to a `@token@` reference inside a quoted string literal (e.g. a file path built
  with a token)? Same treatment as everywhere else — the existing resolver already finds
  `@token@` references inside quoted strings (confirmed by existing hover behavior), so this
  diagnostic scans the identical set of positions, not a narrower one.
- What happens while a document has unsaved, syntactically incomplete edits (e.g. mid-typing an
  `@`)? The underlying tokenizer already never panics on any input; a reference that doesn't yet
  form a complete `@name@` simply isn't a `VariableRef` token yet, so it's outside this
  diagnostic's scan entirely, the same as it's outside hover's today.
- What happens to a `@token@` used as a `LOOP` variable's own bound name, or in another position
  the tokenizer treats specially? Out of scope for a first read — this diagnostic covers exactly
  the same reference-resolution question hover already answers, nothing broader.
- What happens when a document contains no `@token@` references at all? No notices are
  published for this stream — an empty result is not itself a signal of anything.
- What happens when the same unresolvable `@token@` name is referenced more than once in one
  document? Each occurrence is checked and flagged independently, at its own span — there is no
  deduplication down to "one notice per distinct name."
- What happens when a project has no `drut.toml`, or `drut.toml` doesn't govern this setting at
  all? This diagnostic has no configuration surface (Assumptions) — it behaves identically
  regardless of any project configuration, the same way the two existing Hint-severity streams
  it's modeled on have no on/off toggle either.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST identify every `@token@` reference in an open document for which
  the existing token-resolution logic (same-file `TOKEN = value` assignment, or one level of
  static, non-token-built `READ FILE` inclusion) finds no resolvable definition.
- **FR-002**: The system MUST publish a Hint/Information-severity notice at each such
  reference's exact span, distinct in severity and source from the six real structural
  diagnostics (which publish at Error severity).
- **FR-003**: The system MUST NOT flag a `@token@` reference when the reason nothing was found
  is one of the resolver's own documented blind spots — a reference on a block-opener line
  (`RUN PGM=@Prog@`-shaped), a definition reachable only through more than one level of
  `READ FILE` inclusion, or a definition reachable only through a token-built (dynamic) `READ
  FILE` path. These cases are silently not flagged, identical to how hover already silently
  shows nothing for the same positions today.
- **FR-004**: This capability MUST NOT be implemented as a new variant on `voyager-core`'s core
  `Diagnostic`/`DiagnosticKind` type — it MUST follow the same standalone, independently-sourced
  function shape already established twice (the unclosed `; FMT: OFF` marker stream, the
  malformed `drut.toml` warning stream), keeping the six real `DiagnosticKind` values' existing
  meaning and every consumer that depends on that set being closed (`002-cli-check-format`
  FR-003's "never a narrowed subset") completely untouched.
- **FR-005**: The system MUST surface this notice only via the language server's
  `textDocument/publishDiagnostics`, with its own distinct source identifier (mirroring the
  existing `drut-fmt`/`drut-config` sources) — it MUST NOT reach the command-line `check`
  command or the MCP `diagnose` tool, matching both existing Hint-severity streams' exact reach.
- **FR-006**: A `@token@` reference that resolves successfully (same-file assignment, or a
  static one-level `READ FILE` inclusion) MUST NOT receive this notice.
- **FR-007**: This notice MUST update live as the document changes, using the same
  publish-on-change cycle every other diagnostic stream in this project already uses — no
  separate trigger, no stale state after an edit.
- **FR-008**: This capability MUST require no new project configuration — it applies
  identically regardless of `drut.toml` content, the same as both existing Hint-severity streams
  it's modeled on.

### Key Entities

- **Unresolvable `@token@` reference**: A `VariableRef` token whose name has no definition
  findable by the existing token-resolution logic, and whose non-resolution is not attributable
  to a known resolver blind spot (block-opener position, multi-level inclusion, dynamic
  inclusion path).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A `@token@` reference with a genuinely findable same-file or one-level-`READ
  FILE` definition never receives this notice, verified against real corpus-shaped script
  content.
- **SC-002**: A `@token@` reference with no definition findable anywhere within the resolver's
  existing reach receives this notice, verified against real corpus-shaped script content.
- **SC-003**: None of the three documented resolver blind spots (block-opener position,
  multi-level inclusion, token-built inclusion path) ever produces this notice, verified with a
  dedicated case for each.
- **SC-004**: The six existing structural diagnostics continue to publish at Error severity,
  unaffected by this feature's addition — verified by confirming their severity and source
  values are unchanged.
- **SC-005**: Neither the command-line `check` command nor the MCP `diagnose` tool ever
  includes this notice in their output, verified directly against both surfaces.

## Assumptions

- Scope is deliberately narrow: only `@token@` substitution references are covered. Plain
  assignment identifiers (`X` used with no prior `X = value`) and data-reference tokens
  (`MI`/`MW`/etc., bound by a `FILEI`/`FILEO` pair-keyword statement rather than a plain
  assignment) are explicitly **out of scope** — neither has existing resolution logic, and the
  latter's binding mechanism is structurally different from `@token@`'s and was never
  researched for this feature.
- Hint/Information severity (not Error) is a deliberate, conversation-driven downgrade from the
  original "red squiggly" request, made because this check's confidence is inherently bounded
  by the resolver's own one-level-of-inclusion reach — Error severity would overstate that
  confidence, and constitution Principle IV treats a false positive as strictly worse than a
  false negative.
- This capability has no on/off configuration surface, matching both existing Hint-severity
  streams it's modeled on (the unclosed `; FMT: OFF` marker, the malformed `drut.toml`
  warning) — neither is configurable either.
- No `001-voyager-script-parser` spec amendment is needed — confirmed against the actual
  precedent (neither the `; FMT: OFF` marker stream nor the `drut.toml` warning stream amended
  that spec, since neither is a `DiagnosticKind` value; this feature follows the identical
  shape).
