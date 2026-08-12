# Feature Specification: Format-On-Save and Format-On-Paste

**Feature Branch**: `005-format-on-save-paste`

**Created**: 2026-08-10

**Status**: Draft

**Input**: User description: "Add format-on-save and format-on-paste support to the VS Code extension, building on the whole-document textDocument/formatting capability drut-lsp already has. Format-on-save needs no new LSP capability — client-side wiring only. Format-on-paste needs a new drut-lsp capability (textDocument/rangeFormatting), backed by voyager_core::format, since VS Code's editor.formatOnPaste is served by DocumentRangeFormattingEditProvider, not a paste-edit provider. Both are adapter-only (drut-lsp + editors/vscode); no changes to voyager-core's public contract."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Automatic reformatting on save (Priority: P1)

A script author edits a `.s`/`.block` file, introduces inconsistent indentation
(by hand, or by editing inside a block without realigning it), and saves the
file. The file is reformatted to Drut's canonical structural formatting
automatically, the same result `drut format --write` or "Format Document"
(Shift+Alt+F) would already produce — without the author having to invoke
either one by hand.

**Why this priority**: This is the highest-leverage, lowest-risk piece —
`textDocument/formatting` already exists and is already proven correct
(002's golden-fixture corpus, 003's manual verification); this story is pure
client-side wiring with no new parsing/formatting logic anywhere.

**Independent Test**: Open a `.s` file with a misindented body statement
inside a block, enable format-on-save for the language, save the file, and
confirm the body statement is reindented relative to its block's opener —
without running "Format Document" separately.

**Acceptance Scenarios**:

1. **Given** an open `.s` file with a body statement indented incorrectly
   relative to its enclosing block, **When** the author saves the file,
   **Then** the file is rewritten with the body statement's indentation
   corrected, matching what "Format Document" would already produce.
2. **Given** an open `.s` file that is already correctly formatted, **When**
   the author saves it, **Then** the file is left byte-for-byte unchanged
   (no phantom edit, no unsaved-changes flicker).

---

### User Story 2 - Automatic reformatting of pasted content (Priority: P2)

A script author pastes a fragment of Cube Voyager script text — copied from
elsewhere in the same file, another file, or an external source — into an
open `.s`/`.block` document. The pasted text is reformatted immediately after
the paste completes, so its indentation matches the structural context
(block nesting) at the point of paste, the same way it would if the author
had typed it correctly by hand and then reformatted the file.

**Why this priority**: Real, requested new capability, but strictly smaller
in blast radius than US1 — it only ever touches the pasted range, is
triggered less often, and depends on US1's underlying `voyager_core::format`
call already being proven correct before this story reuses it for a new
entry point.

**Independent Test**: Copy a block-shaped fragment with wrong indentation,
paste it into a document at a nesting depth different from where it was
copied, enable format-on-paste, and confirm the pasted text is reindented to
match its new surrounding context immediately after the paste.

**Acceptance Scenarios**:

1. **Given** an open `.s` file with an `IF` block at nesting depth 1,
   **When** the author pastes a correctly-relative-formatted two-line
   fragment into that block, **Then** the pasted lines are reindented to
   depth 1's body indentation, matching the block they landed in rather than
   whatever indentation they carried from their source.
2. **Given** a paste whose content is already correctly indented for its new
   location, **When** the paste completes, **Then** no additional edit is
   applied (idempotence — pasting well-formatted content changes nothing
   further).

---

### User Story 3 - Author stays in control of format-on-save (Priority: P3)

A script author who finds the auto-enabled format-on-save behavior unwanted
can turn it off, and the extension respects that choice — it does not
silently turn the setting back on the next time the workspace or VS Code
itself is reopened. (Format-on-paste needs no equivalent story: per
Clarification Q1, the extension never auto-enables it in the first place, so
there is nothing for it to silently re-apply — a user who turns format-on-paste
on or off is exercising VS Code's own standard per-workspace/per-user setting
persistence, with no extension-side involvement at all.)

**Why this priority**: Lower priority than delivering the two behaviors
themselves, but this project has already committed to this exact
non-intrusiveness precedent once before (the semantic-token color
auto-injection added in 003), and reversing it here would be a real,
user-visible inconsistency between two extension features that behave
identically in spirit.

**Independent Test**: With format-on-save auto-enabled (via this feature's
default), turn `editor.formatOnSave` off for `.s`/`.block` files, close and
reopen the workspace, and confirm the setting stays off rather than
reverting.

**Acceptance Scenarios**:

1. **Given** the extension has auto-enabled format-on-save for this language
   on first activation in a workspace, **When** the author explicitly
   disables it, **Then** reopening the workspace or restarting VS Code does
   not silently re-enable it.

---

### Edge Cases

- What happens when a save or paste occurs inside a document that already
  has structural diagnostics (an unmatched `IF`/`LOOP`, an unclosed block
  comment, etc.)? The formatter's existing whole-document behavior already
  has to handle malformed input without panicking (FR-004 of
  `001-voyager-script-parser`'s public-api contract); this feature must not
  introduce a new failure mode on top of that — worst case, no edit is
  applied for the affected content rather than a guessed/incorrect one
  (Principle IV: false negatives over false positives).
- What happens when the pasted range straddles a block boundary only
  partially — e.g., the paste ends mid-block, with the block's closer
  arriving later, outside the pasted range? The requested range alone may
  not carry enough structural context to determine correct indentation in
  isolation; see FR-003/Assumptions for how this is resolved.
- What happens on a very large file? No new performance concern beyond what
  whole-document formatting already accepted when it shipped in 003 — this
  feature does not change the cost of a single `voyager_core::format` call,
  only how often/when one is triggered.
- What happens if no server connection is available (server crashed,
  `drut server` binary missing)? Same graceful degradation already
  established for every other LSP-dependent capability (003's
  highlighting-only fallback) — save/paste simply behave as they would with
  no formatter registered; no new error surfaced to the user.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a range-formatting capability (serving
  VS Code's paste-triggered formatting request) in addition to the existing
  whole-document formatting capability.
- **FR-002**: Both the existing whole-document formatting and this feature's
  new range-formatting MUST derive their result entirely from
  `voyager_core::format` — no independent formatting or grammar/parsing
  logic MUST be implemented in `drut-lsp` or the extension client
  (constitution Principle I).
- **FR-003**: When the requested paste range cannot be reformatted correctly
  in isolation from context outside that range (per `voyager-core`'s
  structural, block-nesting-derived indentation model — see Assumptions),
  the system MUST still produce a correct result by considering the whole
  document's structure, then applying only the portion of the change that
  falls within the requested range — never silently skip the paste, and
  never let context outside the range be misread in a way that produces a
  wrong indentation inside it.
- **FR-004**: The system MUST auto-enable `editor.formatOnSave` for
  `.s`/`.block` documents on first activation per workspace, via the same
  one-time, removal-respecting workspace-setting injection pattern as 003's
  semantic-token color auto-injection (resolved by Clarification Q1, Option
  C, 2026-08-10).
- **FR-005**: The system MUST make `editor.formatOnPaste` available for
  `.s`/`.block` documents as an opt-in setting the user enables themselves —
  the extension MUST NOT auto-enable it (resolved by Clarification Q1,
  Option C, 2026-08-10). The extension's documentation MUST explain how to
  turn it on.
- **FR-006**: The workspace-setting change the extension makes on the user's
  behalf to enable FR-004 (format-on-save only — FR-005 is opt-in and the
  extension never touches that setting) MUST be one-time per workspace and
  MUST NOT be silently reapplied once the user has changed or removed it —
  matching the existing `ensureVariableColorCustomization`/`workspaceState`
  precedent from 003.
- **FR-007**: Neither format-on-save nor format-on-paste MUST ever write to
  disk or apply an edit through any mechanism other than the LSP-standard
  request VS Code itself issues (`textDocument/formatting`,
  `textDocument/rangeFormatting`) — no extension-side reimplementation or
  editor-proprietary formatting API (constitution Principle VI).
- **FR-008**: Neither behavior MUST alter program meaning — only
  whitespace/indentation, the same idempotence and behavior-preservation
  guarantee `voyager-core`'s formatter already holds end to end (constitution
  Principle III); this feature MUST NOT bypass or weaken that guarantee for
  either new entry point.
- **FR-009**: If a save or paste targets a document the server has no open
  record of, the corresponding handler MUST return no edits rather than
  erroring or panicking (matching the existing whole-document handler's
  `unopened_document_returns_none` behavior).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user who pastes Cube Voyager script text into an open
  `.s`/`.block` file sees it reindented to match its new surrounding
  structure immediately after the paste, with no separate manual action.
- **SC-002**: A user who saves a `.s`/`.block` file with inconsistent
  indentation sees it corrected to Drut's canonical formatting automatically,
  with no separate manual action.
- **SC-003**: Saving an already-correctly-formatted file, or pasting
  already-correctly-formatted content, produces no visible change — both
  behaviors are idempotent, not just the underlying formatter call.
- **SC-004**: A user who turns format-on-save off (after its one-time
  auto-enable) keeps it off across workspace reopens and VS Code restarts,
  with no additional action required to keep it off. Format-on-paste, being
  opt-in from the start, is governed entirely by VS Code's own standard
  setting persistence and needs no equivalent guarantee from the extension.

## Assumptions

- Reuses `voyager_core::format` exactly as `002-cli-check-format` and 003's
  whole-document formatting already do — no new formatting/grammar logic,
  no change to `voyager-core`'s public contract (constitution Principle I).
- The precise implementation strategy for range-formatting a pasted fragment
  against `voyager-core`'s structural (block-nesting-derived) indentation
  model — e.g., running a full-document format internally and returning only
  the edits intersecting the requested range, versus a narrower approach —
  is deliberately left open here and resolved during `/speckit-plan`'s
  research phase; FR-003 constrains the *outcome* (correct, context-aware
  indentation within the requested range) without constraining *how* it's
  computed, since both realistic approaches converge on the same
  user-visible result for well-formed input and only differ in edge-case
  handling (see Edge Cases).
- TOML-based user configuration (a separate, later `ROADMAP.md` item) is out
  of scope here; the workspace-settings auto-injection approach (FR-006) is
  this feature's mechanism for now and may later be superseded by a TOML
  setting without changing this feature's user-facing behavior.
- Phase 5 (per-program-box keyword validation) and Phase 6 (repo-wide/
  multi-file semantic checking) remain out of scope, per
  `specs/004-mcp-server/spec.md`'s own Out of Scope framing — unrelated to
  this feature.
- No file I/O, network access, or protocol dependency is introduced into
  `voyager-core` by this feature — both new/reused capabilities stay
  entirely within the existing `drut-lsp`/`editors/vscode` adapter layer.
