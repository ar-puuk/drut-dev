# Feature Specification: Code Folding Support

**Feature Branch**: `011-code-folding`

**Created**: 2026-08-12

**Status**: Draft

**Input**: User description: "Build code folding support for the Cube Voyager language,
implemented via the LSP-standard textDocument/foldingRange capability (constitution
Principle VI — prefer LSP-standard mechanisms over VS Code-proprietary ones), backed
by voyager-core's real parsed structure — not regex/keyword-based folding like a
Notepad++ UDL would do."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Collapse a block to see program structure at a glance (Priority: P1)

An analyst opens a long `.s`/`.block` script with many nested `IF`/`LOOP`/`RUN`/
`PROCESS` blocks. They want to collapse blocks they aren't currently editing so they
can see the overall shape of the program — which blocks exist, roughly how big each
one is — without scrolling past hundreds of lines of body content.

**Why this priority**: This is the entire value of the feature — without it, folding
support does nothing. Every other story is a refinement of this one.

**Independent Test**: Open a script containing at least one explicitly-closed block of
each of the 7 kinds (`If`, `Loop`, `Run`, `Process`, `JLoop`, `LinkLoop`,
`DistributeMultistep`) in an LSP-capable editor. Confirm each block's opener line
shows a fold control, and collapsing it hides everything from the line after the
opener through the closer, leaving the opener and closer visible.

**Acceptance Scenarios**:

1. **Given** a script with an explicitly-closed `IF`/`ENDIF` block spanning multiple
   lines, **When** the user collapses the fold at the `IF` line, **Then** every line
   between `IF` and `ENDIF` (exclusive of both) is hidden and the `IF` line shows a
   collapsed-content indicator.
2. **Given** a script with nested blocks (e.g., an `IF` containing a `LOOP`),
   **When** the user collapses the outer `IF`, **Then** the inner `LOOP`'s own fold
   control is hidden along with it but remains intact and independently collapsible
   once the outer fold is re-expanded.
3. **Given** a script with a block comment (`/* ... */`) spanning multiple lines,
   **When** the user views the document, **Then** the comment's opening line shows a
   fold control that collapses through the comment's closing `*/` line.

---

### User Story 2 - Fold ranges stay correct as the document is edited (Priority: P2)

An analyst is actively editing a script — adding lines, changing a block's condition,
inserting a new nested block. They expect fold controls to reflect the document's
current, post-edit structure, not a stale snapshot from when the file was opened.

**Why this priority**: A folding feature that only works on first load and then goes
stale is worse than no folding at all — it actively misleads the user about the
document's real structure. This is a correctness requirement on top of User Story 1,
not a separate capability.

**Independent Test**: With a document open and a fold already placed on a block, edit
the document to add a new line inside that block's body, then request fold ranges
again. Confirm the fold range now extends to cover the new line.

**Acceptance Scenarios**:

1. **Given** a document with a collapsed `LOOP`/`ENDLOOP` block, **When** the user
   types a new line inside the loop body (editor auto-expands the fold to show the
   edit, per standard editor behavior), **Then** re-collapsing the fold hides the
   updated body including the new line.
2. **Given** a document where the user deletes a block's closer line entirely
   (leaving the block structurally unmatched), **When** fold ranges are recomputed,
   **Then** no fold range is offered for that now-unmatched opener (folding a range
   with no real closer would hide content arbitrarily, including unrelated
   subsequent lines, which is misleading — see FR-005).

---

### User Story 3 - Fold everything / unfold everything (Priority: P3)

An analyst wants to use their editor's built-in "Fold All" / "Unfold All" commands
(standard in every LSP-capable editor once a folding provider is registered) to
quickly get a table-of-contents view of a large script, or to restore full visibility
before a detailed edit.

**Why this priority**: This is existing editor UI that works automatically once
folding ranges are being reported correctly (User Story 1) — it needs no
Drut-specific work beyond registering the capability, but is called out because it's
a primary way users will actually invoke the feature day-to-day.

**Independent Test**: With User Story 1 implemented, invoke the editor's native
"Fold All" command and confirm every block and block comment in the document
collapses; invoke "Unfold All" and confirm the document returns to its original view.

**Acceptance Scenarios**:

1. **Given** a script with multiple top-level and nested blocks, **When** the user
   runs "Fold All", **Then** every block and block comment with a fold range
   collapses to its opener line.

---

### Edge Cases

- **Implicitly-closed `Run`/`Process` blocks** (no explicit `ENDRUN`/`ENDPROCESS` —
  closed by the next same-family opener or, for `Run`, a shell-escape statement):
  still offered as a fold range, spanning from the opener to the line immediately
  before whatever implicitly closes it. See FR-003 for the reasoning — treating
  these as unfoldable would make folding silently disappear for a large fraction of
  real `Run`/`Process` usage (implicit closing is a normal, common pattern in this
  grammar, not a rare edge case).
- **Short-`IF`** (single-line, self-closing form, e.g. `IF (x=1) y=2`): no fold range
  is offered — there is no body to collapse (opener and closer are the same line).
  See FR-004.
- **A block with a diagnosed/unmatched structure** (e.g., an `IF` with no resolvable
  `ENDIF` anywhere in the document — `UnmatchedIf`): no fold range is offered for
  that opener, since there is no real counterpart line to fold to (per US2 Scenario
  2 and FR-005). This is the same "unresolved structure gets no fold" behavior a
  short-`IF`'s siblings get for a different reason.
- **A block whose opener and would-be closer are on the same line for reasons other
  than short-`IF`** (not currently possible in this grammar, but the underlying rule
  is: a fold range spanning zero collapsible lines is never reported, matching the
  LSP spec's own recommendation that a folding range should span more than one
  line).
- **A single-line block comment** (`/* note */`, opening `/*` and closing `*/` on
  the same physical line — unlike the block case above, this *is* a common,
  currently-possible case): no fold range is offered, by the same same-line rule
  (FR-008) — folding a "range" that collapses zero lines would be a no-op control
  that only adds visual noise. See FR-006/FR-008.
- **Nested block comments** (per the existing lexer's nested-comment support): the
  outermost comment gets one fold range from its opening `/*` to its final closing
  `*/`; nested `/* */` pairs inside it are not separately foldable (block comments
  are a single lexical token, not a nested structural form — folding only exposes
  one collapsible range per comment token, consistent with how every other
  LSP-capable language server treats a single comment token).
- **A document with zero blocks and zero block comments**: folding is registered as
  a capability but returns an empty range list — this is a normal, not an error,
  case.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The LSP server MUST advertise folding-range support in its server
  capabilities so that any LSP-capable editor (not only VS Code) can request and
  render fold controls, per constitution Principle VI.
- **FR-002**: For every block in a document's parsed structure across all 7 block
  kinds (`If`, `Loop`, `Run`, `Process`, `JLoop`, `LinkLoop`,
  `DistributeMultistep`) that has a resolvable counterpart (explicit closer, or an
  implicit close per FR-003), the system MUST report a folding range spanning from
  the opener's line to the resolved counterpart's line.
- **FR-003**: An implicitly-closed `Run` or `Process` block (no explicit
  `ENDRUN`/`ENDPROCESS` present) MUST still be offered a folding range, spanning
  from the opener to the line immediately before whatever construct implicitly
  closes it (the next same-family opener, or for `Run`, a shell-escape statement).
  **Rationale for this default**: implicit closing is a normal, documented,
  frequently-used pattern in this grammar (see `CLAUDE.md`'s grammar-model
  description) — restricting folding to only explicitly-closed blocks would make a
  large fraction of real `Run`/`Process` usage silently unfoldable with no
  indication why, which fails the same "don't be silently confusing" bar this
  project has applied to other features (e.g. `010`'s unclosed-marker notice).
- **FR-004**: A short-`IF` (single-line, self-closing form with no explicit
  `ENDIF`) MUST NOT be offered a folding range — its opener and closer are the same
  line, so there is no body content to collapse.
- **FR-005**: A block with no resolvable counterpart anywhere in the document (i.e.,
  one that would produce a structural diagnostic — `UnmatchedIf`, `UnmatchedLoop`,
  `UnmatchedRun`) MUST NOT be offered a folding range for that opener, since there
  is no real closer line to fold to.
- **FR-006**: Every block comment (`/* ... */`, including nested ones per the
  lexer's existing nested-comment handling) that spans more than one line MUST be
  offered a folding range spanning from its opening line to its closing line,
  matching the LSP's standard comment-kind folding convention. A block comment that
  opens and closes on the same physical line (e.g. `/* note */`) is excluded by
  FR-008, not by this requirement — see FR-008.
- **FR-007**: An unclosed block comment (no matching `*/` before end of file, which
  already produces an `UnclosedBlockComment` diagnostic) MUST NOT be offered a
  folding range — same "no real closer, no fold" rule as FR-005.
- **FR-008**: The system MUST NOT report a folding range that spans only a single
  line (i.e., where the range's start and end resolve to the same line). This
  applies uniformly to both foldable constructs this feature reports: it is what
  makes FR-004's short-`IF` exclusion fall out structurally rather than needing a
  block-kind-specific special case, **and** it is what excludes a single-line block
  comment (`/* note */`, opening and closing `*/` on the same line — a common,
  currently-possible case, unlike the equivalent same-line situation for blocks)
  from FR-006. Both the block stream and the block-comment stream MUST have this
  rule applied before either is reported.
- **FR-009**: Folding-range computation MUST be derived from the same parsed
  structure and the same block-counterpart-resolution logic already used by hover
  and structural-query features (constitution Principle I — no duplicated grammar
  or block-matching logic).
- **FR-010**: Folding ranges MUST be recomputed from the document's current text on
  every request (no caching of a stale parse), so ranges always reflect the
  document's live content as edited in the editor.
- **FR-011**: A document with no foldable blocks and no foldable block comments
  MUST return an empty (not an error) folding-range result.

### Key Entities

- **Folding Range**: A start line and end line (and, per the LSP standard, an
  optional "kind" distinguishing a region fold from a comment fold) identifying a
  span of a document's text that an editor may collapse to a single visual line.
  Derived entirely from existing parsed structure — this feature introduces no new
  persistent data of its own.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In an LSP-capable editor, every explicitly-closed block and every
  well-formed block comment in a real-world script shows a working fold control
  that, when collapsed, hides exactly the lines between (not including) its opener
  and its resolved closer.
- **SC-002**: Running the editor's "Fold All" command on a real-world script with
  multiple nested blocks collapses every foldable block and comment in a single
  action, with no block or comment silently excluded for a reason other than
  FR-004/FR-005/FR-007/FR-008's documented exceptions (FR-008 covers both a
  short-`IF` and a single-line block comment — see Edge Cases).
- **SC-003**: Across a broad sample of real, previously-published Cube Voyager
  scripts, folding ranges are produced with zero false positives (no fold range
  offered where the underlying block/comment isn't actually resolvable to a real
  counterpart) and zero false negatives (no explicitly-closed block or well-formed
  comment silently missing a fold range).
- **SC-004**: A script actively being edited (lines added or removed inside a
  block's body) shows correct, updated fold ranges on the next request — no manual
  editor reload required.

## Assumptions

- The target audience for this feature is the same as every other `drut-lsp`
  capability shipped so far: any LSP-capable editor, with VS Code as the primary
  environment for manual verification, per constitution Principle VI.
- "Resolvable counterpart" reuses this project's existing block-counterpart
  resolution behavior exactly as-is (the same logic hover and structural queries
  already rely on) — this feature defines no new resolution rules of its own, only
  a new consumer of the existing ones.
- Folding ranges are computed per-request from the document's live text (via the
  same document-store mechanism other `drut-lsp` features already use for
  live-edited content), not incrementally maintained — this matches the
  performance characteristics of every other `drut-lsp` feature (diagnostics,
  hover, semantic tokens), all of which already re-derive their output from a full
  re-parse per request on documents of the size this tool targets.
- No new `DiagnosticKind`, grammar rule, or `voyager-core` data shape is required —
  this is a pure adapter-layer (`drut-lsp`) feature consuming existing
  `voyager-core` output, consistent with Principle I.
- VS Code's (and other editors') built-in folding UI (gutter fold icons, Fold
  All/Unfold All commands, fold-level keybindings) is used as-is; no custom
  UI/keybinding work is in scope, per the feature description's explicit
  out-of-scope note.
