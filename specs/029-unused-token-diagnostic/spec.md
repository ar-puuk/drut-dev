# Feature Specification: Unused `@token@` Diagnostic

**Feature Branch**: `029-unused-token-diagnostic`

**Created**: 2026-08-19

**Status**: Draft

**Input**: User description: "Let's do the unused-variable lint tier first" — scoped down
through direct conversation to `@token@` substitution assignments specifically (the only kind
with existing, unambiguous definition/reference resolution logic), as the exact inverse of
`020-undefined-token-diagnostic`: that feature flags a `@token@` reference with no resolvable
assignment; this feature flags an assignment whose name is never read via `@token@` at all.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A developer notices a dead token assignment while editing (Priority: P1)

While editing a `.s`/`.block` file, a developer assigns a value to a name (`ScenarioDir = 'X:\...'`)
intending to reference it later via `@ScenarioDir@`, but never actually does — maybe the
reference was deleted during a later edit, renamed inconsistently, or the assignment itself is
leftover from an earlier version of the script. Today nothing signals this; the assignment just
sits there, silently doing nothing. A subtle, non-alarming underline on the assignment gives an
early signal, without the tool claiming more certainty than it actually has about whether the
name might be used from somewhere it can't see.

**Why this priority**: This is the entire feature — there is no secondary priority tier here,
matching `020-undefined-token-diagnostic`'s own shape. It's independently valuable and
independently shippable.

**Independent Test**: Open a `.s`/`.block` file containing a `TargetName = value` assignment
with no `@TargetName@` reference anywhere the tool can see, and confirm a Hint/Information-severity
underline appears at that assignment — while a second assignment in the same file that *is*
later referenced via `@OtherName@` receives no such underline.

**Acceptance Scenarios**:

1. **Given** a document containing `ScenarioDir = 'X:\model'` with no `@ScenarioDir@` reference
   anywhere in the file, **When** the document is opened or edited, **Then** the editor shows a
   Hint/Information-severity underline at that assignment statement — visually and
   programmatically distinct from the six real structural diagnostics (Error severity) and from
   `UndefinedToken` (a different Hint-severity stream, at a different kind of position).
2. **Given** the same document also contains `Prog = MATRIX` followed later by `RUN PGM=@Prog@`,
   **When** the document is edited, **Then** the `Prog = MATRIX` assignment receives no
   underline — a reference on a block-opener line counts as a genuine use, the same as a
   reference anywhere else (see FR-003; this is a correctness fix this feature makes, not an
   inherited blind spot).
3. **Given** a document containing `ScenarioDir = 'X:\old'` reassigned later in the same file as
   `ScenarioDir = 'X:\new'`, with `@ScenarioDir@` referenced once, after both assignments, **When**
   the document is edited, **Then** neither assignment receives an underline — the name has a
   genuine use, so no individual assignment site is second-guessed as a dead store (that's a
   different, out-of-scope analysis — see Assumptions).
4. **Given** a document with no `@token@`-shaped references at all and no `READ FILE` statements,
   **When** it contains an assignment never referenced, **Then** that assignment receives an
   underline — the simple, fully-self-contained case this feature exists for.
5. **Given** a document containing `ScenarioDir = 'X:\old'` reassigned later in the same file as
   `ScenarioDir = 'X:\new'`, with **no** `@ScenarioDir@` reference anywhere, **When** the document
   is edited, **Then** *both* assignments receive an underline — every dead assignment site is
   flagged independently when the name is never used at all (Clarification Q1).
6. **Given** a document containing `ScenarioDir = 'X:\model'` that also has a `READ FILE`
   statement (in either direction — this file including another, or, unknowably from this file
   alone, some other file including this one) and no `@ScenarioDir@` reference findable within
   this feature's scope, **When** the document is edited, **Then** the assignment still receives
   an underline — this feature does not suppress itself for files that participate in inclusion
   (Clarification Q2), accepting the documented risk that a name genuinely consumed only by an
   including file will render as a false positive.

---

### Edge Cases

- What happens when the same never-referenced name is assigned more than once in one file?
  Every such assignment is flagged independently (Clarification Q1, Acceptance Scenario 5) — no
  deduplication down to "one notice per distinct name," matching `020-undefined-token-diagnostic`'s
  own "each occurrence checked and flagged independently" precedent for the inverse check.
- What happens when a file has any `READ FILE` statement (in either direction — this file
  including another, or, unknowably from this file alone, another file including this one)? This
  feature applies its check regardless (Clarification Q2, Acceptance Scenario 6) — a name
  genuinely used only by a file that includes this one is a known, accepted false-positive risk,
  not a reason to suppress the check for every file that happens to touch `READ FILE` (see
  Assumptions).
- What happens to an assignment whose target name is also a recognized data-reference name
  (`MI`/`MW`/`DBA`/etc.) or a control/statement keyword? Out of scope for flagging as unused —
  `all_assignments` already only returns genuine `Assignment`-statement targets, and this
  feature adds no new classification logic beyond what that collector already does.
- What happens while a document has unsaved, syntactically incomplete edits? The underlying
  tokenizer/parser already never panics on any input; an incomplete assignment simply isn't an
  `Assignment` node yet, so it's outside this diagnostic's scan, the same as every other
  diagnostic in this project.
- What happens when a document contains no assignments at all? No notices are published — an
  empty result is not itself a signal of anything.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST identify every `Assignment` statement's target name in an open
  document for which no `@name@` reference (case-insensitive) exists anywhere within scope: the
  same file, plus one level of directly-included, statically-resolvable `READ FILE` files — the
  identical downward scope `020-undefined-token-diagnostic`'s resolver already uses. This
  feature deliberately applies unconditionally, regardless of whether the file itself
  participates in any `READ FILE` relationship (Clarification Q2) — it does not attempt to
  detect or suppress itself for the unbounded "used only by a file that includes this one" case,
  which is accepted as a documented false-positive risk (see Assumptions) rather than a reason
  to narrow coverage.
- **FR-002**: The system MUST publish a Hint/Information-severity notice at each unused
  assignment's exact span — every dead assignment site independently when a name is reassigned
  multiple times with zero references anywhere in scope (Clarification Q1), not deduplicated
  down to one notice per name. Each notice is distinct in severity and source from the six real
  structural diagnostics (Error severity) and from `UndefinedToken` (a different source, at a
  different kind of position).
- **FR-003**: The system MUST count a `@name@` reference that appears on a block-opener
  statement's own line (e.g. `RUN PGM=@Prog@`, `LOOP NUMREC=@Count@`) as a genuine use. This is
  a correctness requirement, not an optional enhancement: the existing `all_variable_refs`
  collector is documented to exclude these positions, which is an acceptable false-negative for
  `020-undefined-token-diagnostic` (a reference silently not flagged as undefined) but would be
  an unacceptable false-positive here (a genuinely-used name flagged as unused) — constitution
  Principle IV forbids exactly this shape of error. `voyager-core` MUST gain whatever reference
  collection this feature needs (extending `all_variable_refs`, or a variant used only here) so
  block-opener-line references are visible to this check specifically.
- **FR-004**: This capability MUST NOT be implemented as a new variant on `voyager-core`'s core
  `Diagnostic`/`DiagnosticKind` type — it MUST follow the same standalone, independently-sourced
  function shape already established three times (the unclosed `; FMT: OFF` marker stream, the
  malformed `drut.toml` warning, `UndefinedToken`), keeping the six real `DiagnosticKind`
  values' existing meaning, and every consumer that depends on that set being closed, completely
  untouched.
- **FR-005**: The system MUST surface this notice only via the language server's
  `textDocument/publishDiagnostics`, with its own distinct source identifier — it MUST NOT reach
  the command-line `check` command or the MCP `diagnose` tool. This matches every existing
  Hint-severity stream's exact reach in this project (`020-undefined-token-diagnostic`'s own
  FR-005 included) — none of them reach CLI/MCP, and this feature does not become the first
  exception.
- **FR-006**: A name with at least one `@name@` reference anywhere within this feature's defined
  scope MUST NOT receive this notice on any of its assignments, regardless of how many times it
  was assigned (Clarification Q1).
- **FR-007**: This notice MUST update live as the document changes, using the same
  publish-on-change cycle every other diagnostic stream in this project already uses.
- **FR-008**: This capability MUST require no new project configuration — it applies
  identically regardless of `drut.toml` content, matching every existing Hint-severity stream.

### Key Entities

- **Unused assignment**: An `Assignment` statement whose target name has no `@name@` reference
  (case-insensitive) anywhere within scope — same file plus one level of directly-included,
  statically-resolvable `READ FILE` files — including block-opener-line positions (FR-003).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An assignment whose name genuinely has no reference anywhere in scope receives
  this notice, verified against real corpus-shaped script content.
- **SC-002**: An assignment whose name is referenced anywhere in scope — including on a
  block-opener line — never receives this notice, verified with a dedicated case for the
  block-opener position specifically (the correctness fix this feature makes over the
  `020-undefined-token-diagnostic`-era collector).
- **SC-003**: A name reassigned multiple times with zero references anywhere in scope produces a
  separate notice at every assignment site, verified with a dedicated multi-assignment case.
- **SC-004**: The six existing structural diagnostics and `UndefinedToken` continue to publish
  unaffected by this feature's addition, verified by confirming their severity/source values are
  unchanged.
- **SC-005**: Neither the command-line `check` command nor the MCP `diagnose` tool ever includes
  this notice in their output, verified directly against both surfaces.

## Assumptions

- Scope is deliberately narrow, matching `020-undefined-token-diagnostic`: only `@token@`
  substitution assignments are covered. A plain assignment whose value is simply never reused in
  any other way (e.g. a Voyager built-in reads the variable by name directly rather than through
  `@name@` substitution, if such a mechanism exists) is out of scope — this feature only knows
  about the one reference mechanism `voyager-core` already resolves.

  **Post-implementation correction**: that "mechanism" does exist, and is the *common* case, not
  an edge case — confirmed directly against real-corpus fixtures (`AssignHwy/02_Assign_AM_MD_PM_EV.s`)
  and a real false-positive report (`nextLINKSEQ`). `@...@` is only the mechanism for injecting a
  Control-Language-level value *into* a `RUN PGM=...` block's body; a variable that never crosses
  that boundary is correctly read as a plain bareword for its entire lifetime, with no `@...@`
  ever required. Originally, this feature flagged *every* such ordinary, correctly-used variable
  as if it were dead — a real, structural false-positive class, unacceptable under constitution
  Principle IV (a false flag on working code, not a missed true positive). Fixed by widening the
  "referenced" check to also count a plain bareword read in a value position (an `Assignment`'s
  right-hand side, a `Control` statement's pair values, an `IF`/`ELSEIF` condition, or a
  `ShellEscape`'s command text) — `voyager_core::all_bareword_reads`, unioned into the same
  `referenced` set `all_variable_refs_including_openers` already populates. Deliberately imprecise
  in two directions, both accepted because they can only ever suppress a diagnostic, never
  fabricate one: a bareword `X` inside a `RUN PGM=...` body may actually be that PGM's own
  internal, unrelated variable (not the outer, same-named assignment); and a name used only as a
  bracketed-subscript index on an assignment's own left-hand side isn't scanned (see
  `all_bareword_reads`'s own doc comment for the full rationale).
- This feature flags every dead assignment **site** independently when its name has zero
  references anywhere in scope (Clarification Q1) — it does not, however, attempt dead-store
  analysis in the shadowing sense (an earlier assignment overwritten by a later one before ever
  being read, despite the name eventually being read from the final assignment) — that remains a
  materially different, higher-risk analysis this feature does not attempt (Acceptance Scenario 3).
- This feature applies unconditionally regardless of whether a file participates in any
  `READ FILE` relationship (Clarification Q2) — a name defined in this file but genuinely used
  only by some other file that includes this one (a plausible, real Cube Voyager
  shared-parameters authoring pattern) will render as a false positive here. This is a
  deliberately accepted, documented risk rather than a reason to narrow coverage down to
  fully self-contained files only; a future iteration could revisit this if real-world use
  shows the false-positive rate against actual corpus content is unacceptable.
- Hint/Information severity, no on/off configuration surface, and LSP-only reach are deliberate
  choices matching every existing Hint-severity precedent in this project — not something this
  feature introduces new judgment about.

## Clarifications

### Session 2026-08-19

- Q1: When a name is assigned more than once in the same file and never referenced at all,
  should the notice appear on every dead assignment site, or once (anchored at the first
  assignment)? → **A: Every dead assignment site**, independently (FR-002, Acceptance Scenario 5).
- Q2: The existing token-resolution logic only searches *downward* (this file, or a file this
  one `READ FILE`s); it has no visibility into whether some *other* file includes this one and
  uses a token defined here — a real risk for shared-parameters files. How should this feature
  handle that? → **A: Apply the check unconditionally and document the risk** (FR-001,
  Acceptance Scenario 6) rather than suppressing the check for any file touching `READ FILE`.
