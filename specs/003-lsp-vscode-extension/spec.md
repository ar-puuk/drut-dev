# Feature Specification: Drut LSP Server & VS Code/Open VSX Extension

**Feature Branch**: `003-lsp-vscode-extension`

**Created**: 2026-08-09

**Status**: Draft

**Input**: User description: "Build the LSP server and VS Code/Open VSX extension for
Drut, as a thin adapter over the voyager-core library crate (constitution Principle
I). `drut server` subcommand implements the Language Server Protocol: diagnostics
(all 7 voyager-core categories), hover (block-kind and matched-counterpart info),
keyword completion (grounded in a real-usage census, not a hand-guessed list),
spell-check ('did you mean' against a hand-written dictionary), and semantic tokens
(short-IF vs block-IF, unreachable code after BREAK). Must explicitly resolve the
char-count-vs-UTF-16 position-encoding gap flagged in Phase 1. VS Code/Open VSX
extension: static TextMate grammar for instant baseline highlighting (may reference
bhereth.language-citilabscubevoyager's structure per the granted permission, never
his text/lists verbatim), a vscode-languageclient wrapper spawning `drut server`,
published to both marketplaces under Drut's own publisher identity. Out of scope:
MCP server, per-program-box keyword validation, repo-wide semantic/reference
checking. Definition of done: LSP server passes the same full-corpus validation
already proven for voyager-core and the CLI."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Instant syntax highlighting on install (Priority: P1)

A script author installs the Drut extension from the VS Code Marketplace or Open VSX
and opens a `.s`/`.block` file. Without waiting for any server process to start, the
file is immediately readable: control words, comments (including nested block
comments), strings, and `@variable@` substitutions are visually distinguished, and
brackets/comment delimiters behave correctly for auto-closing and toggling.

**Why this priority**: This is the lowest-risk, zero-dependency layer of the whole
feature — it needs no `drut server` process, can't fail due to a server crash or a
missing binary on `PATH`, and is the first thing every user experiences on install.
Every other story in this feature builds on top of a file that is already legible.

**Independent Test**: Install the extension in a clean VS Code instance with no
`drut` binary anywhere on `PATH`, open a `.s` file from the fixture corpus, and
confirm keywords/comments/strings/`@variable@` are colored distinctly and bracket
matching works — entirely from the static grammar, with the server disabled or
absent.

**Acceptance Scenarios**:

1. **Given** the extension is installed and no `drut server` process is running,
   **When** the user opens a `.s` or `.block` file, **Then** control words,
   comments, strings, and `@variable@` substitutions are each rendered in visually
   distinct colors within the editor's active color theme.
2. **Given** a file containing a nested block comment (`/* outer /* inner */
   still-comment */`), **When** the file is opened, **Then** the entire nested
   region is highlighted as comment, matching the nesting behavior already fixed in
   voyager-core's lexer.
3. **Given** the cursor is positioned just after an opening bracket or comment
   delimiter, **When** the user types the matching close or invokes "toggle
   comment," **Then** the editor's built-in bracket-matching and comment-toggling
   commands behave correctly for Voyager syntax.

---

### User Story 2 - See structural problems as you type, not just at the command line (Priority: P2)

A script author editing a `.s`/`.block` file in VS Code sees the same structural
diagnostics `drut check` would report — unmatched `IF`/`LOOP`/`RUN`, unclosed block
comments, invalid continuations, misplaced `BREAK` — directly in the editor as they
edit, without leaving the editor or running the CLI separately. (`drut check`'s
seventh diagnostic, `InvalidEncoding`, is not part of this live-editing story — see
FR-005 and Assumptions for why.)

**Why this priority**: This is the core value proposition carried over from the CLI
(002) into the editor — it's what turns "catch this defect eventually, in CI" into
"catch this defect the moment you make it." It depends on Story 1's language
registration existing but not on any other editor feature in this spec.

**Independent Test**: Open a fixture file with a deliberately unmatched `IF` and
confirm a diagnostic appears at the correct location within the editor without
running any command manually; fix the defect and confirm the diagnostic disappears
without reopening the file.

**Acceptance Scenarios**:

1. **Given** a `.s` file with a deliberately unmatched `IF`, **When** the file is
   opened in the editor, **Then** a diagnostic appears underlining the offending
   location, with a message describing the unmatched block.
2. **Given** a file with no structural defects, **When** it is opened, **Then** no
   diagnostics are reported for it.
3. **Given** a file currently showing a diagnostic, **When** the user edits the file
   to fix the underlying defect, **Then** the diagnostic is cleared without the user
   having to save or reopen the file.
4. **Given** the full fixture corpus's valid files, **When** each is opened in the
   editor, **Then** none of them shows any diagnostic — reproducing, through the
   editor, the same zero-false-positive result already proven for voyager-core and
   the CLI, for the six diagnostic categories reachable through live editing (see
   FR-005).

---

### User Story 3 - Understand block structure by hovering (Priority: P3)

A script author hovers over an `IF`, `LOOP`, `RUN`, `PHASE`/`ENDPHASE`, `JLOOP`,
`LinkLoop`, or `DistributeMultistep` opener or closer and sees which of the seven
block kinds it belongs to and, when the parser was able to resolve it, the location
of its matched counterpart — useful for orienting inside deeply nested or
implicitly-closed blocks (Run/Process's documented implicit-close quirk) without
manually scrolling to find the matching line.

**Why this priority**: Valuable but strictly secondary to seeing problems (Story 2)
— hover is an orientation aid for correct-but-complex scripts, not a defect-finding
mechanism, so it can ship after diagnostics without reducing this feature's core
value.

**Independent Test**: Hover over an `IF` in a file with several nested blocks and
confirm the hover panel names the block kind and, when resolvable, jumps to or names
the matched `ENDIF`'s location; hover over a `RUN` block that closes implicitly (no
explicit `ENDRUN`, closed by the next `RUN` or a shell-escape statement) and confirm
the hover still reports its resolved implicit closer.

**Acceptance Scenarios**:

1. **Given** the cursor is over an `IF` opener, **When** the user requests hover
   info, **Then** the response names "If" as the block kind and the location of its
   matched `ENDIF`, `ELSEIF`, or `ELSE`, if resolved.
2. **Given** the cursor is over a self-closing short-`IF`, **When** the user
   requests hover info, **Then** the response distinguishes it from a block-style
   `IF` — there is no separate closer to report.
3. **Given** the cursor is over a `RUN` statement whose block closes implicitly
   (per the documented Run/Process quirk), **When** the user requests hover info,
   **Then** the response still reports the resolved implicit closer's location, not
   an "unresolved" state.
4. **Given** the cursor is over a token that is not part of any block opener/closer,
   **When** the user requests hover info, **Then** no block-structure hover is
   shown for that token.

---

### User Story 4 - Get keyword suggestions while typing (Priority: P4)

A script author typing a control statement gets suggestions for control words and
common `keyword=value` pair names, so they don't have to remember exact spelling or
recall which keywords a given control word typically takes.

**Why this priority**: A genuine authoring aid, but the feature remains fully usable
without it (a user can always type keywords by hand) — it depends on Story 1's
grammar and benefits from, but does not require, Story 2's parser plumbing being
wired into the server first.

**Independent Test**: Trigger completion at the start of a new statement and confirm
the suggestion list includes recognized control words; trigger completion after a
recognized control word and confirm keyword-name suggestions relevant to that
context appear (or, if full context-awareness is out of reach this phase per
Assumptions, confirm the documented general-syntax fallback list appears instead).

**Acceptance Scenarios**:

1. **Given** the cursor is at the start of a new statement, **When** the user
   triggers completion, **Then** the suggestion list includes the general-syntax
   control words.
2. **Given** the cursor is positioned to start a new `keyword=value` pair after a
   recognized control word, **When** the user triggers completion, **Then** the
   suggestion list includes keyword names scoped to that control word — every
   time the parser resolves an enclosing control word, scoping is applied; the
   documented general-syntax fallback list is used only for the specific control
   words the census recorded no paired keyword names for, never as a
   capability gap in the scoping mechanism itself.
3. **Given** the cursor is inside a comment or a quoted string, **When** the user
   triggers completion, **Then** no control-word/keyword suggestions are offered.

---

### User Story 5 - Get a nudge on a likely-misspelled keyword (Priority: P5)

A script author who mistypes a control word or keyword name sees a "did you mean"
suggestion pointing at the closest real entry in the keyword dictionary, rather than
silently getting no highlighting/completion support for that token.

**Why this priority**: Smaller, more specialized value than completion itself
(Story 4) — it reuses the same dictionary and only fires on the narrower case of a
near-miss token, so it is ordered after the completion list it depends on.

**Independent Test**: Type a control word with one character changed from a real
entry (e.g. a transposed letter) and confirm the editor surfaces a "did you mean
<real keyword>" suggestion referencing the nearest dictionary entry, not a fixed
generic message.

**Acceptance Scenarios**:

1. **Given** a token that closely matches exactly one dictionary entry (e.g. one
   character off), **When** the file is analyzed, **Then** a "did you mean" nudge
   naming that entry is shown for the token.
2. **Given** a token that does not closely match any dictionary entry, **When** the
   file is analyzed, **Then** no spelling nudge is shown for it (it is left to
   ordinary diagnostics/no diagnostic, not flagged as a likely typo).
3. **Given** a token that exactly matches a dictionary entry, **When** the file is
   analyzed, **Then** no spelling nudge is shown for it.

---

### User Story 6 - See structural nuance through highlighting (Priority: P6)

A script author sees highlighting that reflects parsed structure, not just static
lexical categories — a self-closing short-`IF` is visually distinguishable from a
block-style `IF`, and code that can never execute because it follows a `BREAK`
inside its enclosing loop is visually flagged as unreachable.

**Why this priority**: The most purely cosmetic story in this set — it deepens
comprehension of already-correct code but changes no diagnostic or completion
behavior, so it is ordered last without blocking any other story's value.

**Independent Test**: Open a file containing both a short-`IF` and a block-style
`IF` and confirm they render with distinguishable semantic token types; open a file
with a statement following a `BREAK` inside the same loop body and confirm that
statement is visually flagged as unreachable.

**Acceptance Scenarios**:

1. **Given** a file containing a self-closing short-`IF`, **When** the file is
   opened, **Then** it is rendered with a semantic token type distinguishable from a
   block-style `IF`.
2. **Given** a file containing a statement that follows a `BREAK` within the same
   loop body, before that loop's closer, **When** the file is opened, **Then** that
   statement is visually flagged as unreachable.
3. **Given** a `BREAK` that is itself misplaced (outside any loop, already reported
   as the `MisplacedBreak` diagnostic per Story 2), **When** the file is opened,
   **Then** no statement is additionally flagged as "unreachable after BREAK" on
   account of that misplaced `BREAK` — unreachability highlighting only follows a
   `BREAK` the parser resolved as validly inside a loop.

### Edge Cases

- A workspace has no `drut` binary reachable (not on `PATH`, not bundled): the
  extension must still deliver Story 1's static highlighting; server-dependent
  features (Stories 2–6) degrade to unavailable with a visible, non-blocking
  notification, not a silent failure or a repeating error popup.
- The `drut server` process crashes mid-session: the extension must detect this,
  surface it once, and attempt restart per standard LSP client reconnect behavior,
  without losing Story 1's static highlighting.
- A file exceeds a large-but-realistic size (e.g. a multi-thousand-line generated
  script): diagnostics/hover/completion must still return, even if slower than on a
  typical file — no case where the server hangs indefinitely on a large well-formed
  file. **No numeric size ceiling is specified** — unlike FR-012's indentation
  rules (`002-cli-check-format`), there is no corpus survey establishing a real
  "largest realistic file" figure for this feature to target, so this bar is
  deliberately left qualitative (bounded-but-slower is acceptable; unbounded hang
  is not) rather than an invented precise threshold. The same applies to any
  memory/scale ceiling for many simultaneously-open documents (`ServerState`
  holds one `ParseResult` per open document, Key Entities) — left unbounded by
  design this phase, revisit only if real usage shows it matters.
- The editor buffer is dirty (unsaved changes): diagnostics reflect the current
  in-editor buffer content, not only the last-saved-to-disk version — matching the
  standard LSP "server tracks open document text" model.
- A file contains a multi-UTF-16-code-unit character (e.g. an emoji or other
  supplementary-plane character) inside a comment or string, before a location that
  a diagnostic, hover, or completion response needs to reference: the reported
  position lands on the correct character, not shifted by the char-count-vs-UTF-16
  discrepancy this feature is required to resolve (see FR-019).
- A file is opened whose extension is not `.s`/`.block` but the user manually sets
  its language mode to Drut's: the extension still activates language features for
  it; the CLI's `.gitignore`-aware directory-level filtering (002-cli-check-format)
  does not apply here since there is no directory traversal in an editor session.
- Two workspace folders are open simultaneously, each with its own `.s`/`.block`
  files: diagnostics, hover, and completion are scoped correctly per open document
  regardless of which workspace folder it belongs to.
- The same file is reachable through two different workspace folders (e.g. one
  folder nested inside the other, or two folders both containing the identical
  absolute path): the editor's own document-identity model (one document per URI)
  determines whether this is one shared open document or two — the server does not
  invent its own identity scheme independent of the URIs the editor gives it.
- A file's on-disk bytes are not valid UTF-8: no `InvalidEncoding` diagnostic is
  ever produced through live editing, regardless of how the byte would decode —
  the editor decodes the file to valid Unicode text before the server ever sees
  its content, so the server cannot observe the original bytes at all (see FR-005,
  FR-007, and Assumptions). `InvalidEncoding` remains reachable only via `drut
  check`, which reads raw bytes directly from disk.
- A file is renamed or moved while open: the editor models this as closing the
  document at its old URI and opening it at the new one (standard LSP behavior) —
  the server needs no rename-specific handling beyond its ordinary
  `didClose`/`didOpen` handling.
- A hover, completion, or spell-check request arrives for a document whose most
  recent edit hasn't finished being re-parsed yet: not reachable in practice —
  the server processes one LSP message at a time in receipt order (no async
  runtime, per research.md §3), so a `didChange`'s re-parse always completes
  before the next queued request is handled.
- A workspace contains zero `.s`/`.block` files: the extension's own
  activation trigger (activating only when a matching-language document is
  opened) means it simply never activates — there is no "activated but idle"
  state to define behavior for.
- The `drut` binary is resolvable on one platform's `PATH` convention but not
  another's (e.g. Windows' `.exe` suffix and path-separator handling vs.
  macOS/Linux): binary resolution MUST behave correctly per-platform, not only
  on the platform used during development (see FR-025).
- The keyword dictionary has no entry close enough to a typo'd token to name a
  single confident suggestion: no nudge is shown (per Story 5's Acceptance Scenario
  2) rather than guessing.
- Two open documents structurally reference each other (e.g. one script's
  `RUN`/`PHASE` mentions a file path handled by another open script): each
  document's diagnostics/hover/completion are derived purely from its own
  content — the server never reads or reasons about a second document to answer
  a request about the first, keeping this feature's single-document scope
  unambiguous even when two open documents happen to be topically related
  (repo-wide semantic/reference checking remains out of scope, see Assumptions).

## Requirements *(mandatory)*

### Functional Requirements

**Server bootstrap & protocol conformance**

- **FR-001**: The system MUST provide a `drut server` subcommand that speaks the
  Language Server Protocol over its standard transport, launchable by an LSP client
  without additional configuration beyond pointing it at the binary.
- **FR-002**: The server MUST track each open document's current in-editor content
  (not only its last-saved-to-disk content) and MUST re-derive diagnostics, hover,
  completion, and semantic-token results from that live content as it changes.
- **FR-003**: The server MUST NOT duplicate any grammar, parsing, or lint-rule logic
  that belongs in `voyager-core` (constitution Principle I) — every diagnostic,
  hover fact, completion candidate's structural validity, and semantic-token
  classification MUST be derived from `voyager-core`'s public entry points, never
  reimplemented independently in the server.
- **FR-004**: The server MUST NOT panic on any document content, including
  malformed text — a per-document failure MUST surface as a diagnostic or an
  LSP-level error response, never as a crash that ends the server process. (This
  guarantee no longer names `InvalidEncoding` specifically — see FR-005: that
  category is unreachable through live document content, since the LSP transport
  itself guarantees every document the server receives is already valid Unicode
  text, never raw undecodable bytes.)

**Diagnostics**

- **FR-005**: The server MUST publish a diagnostic for every `Diagnostic` value
  voyager-core's `parse()` returns for an open document's current text content,
  covering six of the seven categories `voyager-core` defines: `UnmatchedIf`,
  `UnmatchedLoop`, `UnclosedBlockComment`, `InvalidContinuation`, `UnmatchedRun`,
  `MisplacedBreak`. The seventh category, `InvalidEncoding`, is **not reachable
  through live document content and is explicitly out of scope for this
  requirement** — see Assumptions for why (in short: LSP's `didOpen`/`didChange`
  payloads are JSON strings, which cannot represent invalid byte sequences at
  all; the editor has already decoded the file to valid Unicode text, correctly
  or via its own replacement-character fallback, before the server ever sees any
  content). `InvalidEncoding` remains fully reachable via `drut check`
  (`002-cli-check-format`), which reads raw file bytes directly from disk and
  calls `parse_bytes()`.
- **FR-006**: The server MUST re-publish a document's diagnostics whenever its
  content changes, and MUST clear all diagnostics for a document when it is closed.
- **FR-007**: Diagnostics published by the server for a given document's current
  text content MUST match, one-for-one in category, message, and location, what
  `drut check` would report if given that same text as its input (accounting
  only for the position-encoding translation required by FR-019) — the editor
  and the CLI MUST never disagree about whether a given script is structurally
  sound. This guarantee is scoped to the six categories FR-005 covers; it
  does not extend to `InvalidEncoding` (FR-005's carve-out) and is stated in
  terms of "the same text," not "the same file on disk" — when the editor
  buffer is dirty (Edge Cases), `drut check`'s own report against the
  saved-to-disk file is expected to differ, and that expected divergence is not
  a violation of this requirement.

**Hover**

- **FR-008**: The server MUST respond to a hover request over a block opener or
  closer token (any of the seven block kinds: If, Loop, Run, Process/Phase, JLoop,
  LinkLoop, DistributeMultistep) with which block kind it belongs to.
- **FR-009**: When the parser has resolved a hovered opener/closer's counterpart,
  the hover response MUST include that counterpart's location, including when the
  match was resolved through a block family's documented implicit-close behavior
  (Run, Process).
- **FR-010**: The hover response for a self-closing short-`IF` MUST distinguish it
  from a block-style `IF` rather than reporting a nonexistent separate closer.
- **FR-011**: A hover request over a token that is not part of any block
  opener/closer MUST NOT return block-structure information (it may return no
  response, or a response scoped to a future non-block-structure hover fact, but
  MUST NOT fabricate a block relationship for an unrelated token).

**Keyword completion**

- **FR-012**: The server MUST offer completion suggestions for control words at the
  start of a statement, and for `keyword=value` pair names following a recognized
  control word, drawn from a hand-written keyword dictionary derived from real-usage
  evidence (structural-position classification against the fixture corpus, per the
  same methodology used for voyager-core's own control-word evidence trail) — never
  a hand-guessed or vendor-doc-copied list (constitution Principle II).
  Control-word-scoped completion (restricting keyword-name suggestions to those
  observed, during the census, paired with the specific control word — `IF`,
  `LOOP`, `RUN`, `PATHLOAD`, etc. — enclosing the cursor) MUST be applied every
  time the cursor resolves inside a recognized control word's statement — this is
  achieved as the primary mode this phase, not a best-effort target (see
  research.md §2). The general-syntax control-word list remains the fallback for
  exactly two narrower cases, neither a capability gap: (a) the cursor sits
  before any control word exists yet on the current statement, and (b) the
  specific enclosing control word has no keyword names recorded against it in
  the census, in which case completion falls back to every recorded keyword
  name rather than an empty list (data-model.md §1). **This scoping is
  strictly by control word, never by a program name** (e.g. the `PGM=` value
  inside a `RUN`/`PHASE` block) — per-program-box keyword knowledge (e.g. that
  `RUN PGM=MATRIX` specifically takes `ZONES=`) is out of scope for this feature,
  exactly as `001-voyager-script-parser` FR-019 already rules out for
  `voyager-core` itself; this FR MUST NOT be read as reopening that boundary.
- **FR-013**: The server MUST NOT offer control-word/keyword completions when the
  cursor is inside a comment or a quoted string literal.

**Spell-check**

- **FR-014**: The server MUST perform fuzzy "did you mean" matching for a token that
  closely — but not exactly — matches a single entry in the same keyword dictionary
  used for completion (FR-012), and MUST surface that suggestion distinctly from
  ordinary diagnostics (FR-005). This is delivered on-request, riding on the
  existing hover and completion responses (a hover over the misspelled token, or
  a distinctly-labeled completion item) rather than as a proactive-while-typing
  notification or a new LSP method — an LSP-standard-mechanisms choice, per
  constitution Principle VI (see `contracts/lsp-capabilities.md`).
- **FR-015**: The server MUST NOT surface a spelling nudge for a token that exactly
  matches a dictionary entry, or that has no sufficiently close match in the
  dictionary.

**Semantic tokens**

- **FR-016**: The server MUST provide semantic tokens that distinguish a
  self-closing short-`IF` from a block-style `IF`.
- **FR-017**: The server MUST provide a semantic token classification (or
  equivalent structural flag) for a statement that is unreachable because it follows
  a `BREAK` validly resolved as inside its enclosing loop, before that loop's
  closer.
- **FR-018**: The server MUST NOT flag a statement as unreachable-after-`BREAK` on
  account of a `BREAK` that itself could not be resolved inside any loop (already
  reported separately as `MisplacedBreak`, FR-005).

**Position encoding (cross-cutting, blocking prerequisite)**

- **FR-019**: Every position the server sends in a diagnostic, hover, completion, or
  semantic-token response MUST be expressed in UTF-16 code units, per the LSP wire
  protocol's requirement — regardless of whether the translation from
  voyager-core's `Span` (Unicode scalar value count) happens inside voyager-core
  itself or at this adapter's boundary, that decision MUST be made explicitly and
  documented (see Assumptions) rather than left as an implicit assumption that the
  two counting schemes coincide. **This decision is made in `research.md` §1**
  (`drut-lsp` owns the translation at its boundary; `voyager-core`'s `Span` is
  unchanged), with the concrete translation contract in
  `contracts/position-encoding.md` — a reader of this spec alone should follow
  this pointer rather than needing to already know where to look.
- **FR-020**: The server MUST report correct UTF-16 positions for content containing
  characters outside the Basic Multilingual Plane (e.g. supplementary-plane
  characters that occupy two UTF-16 code units), not only for content where char
  count and UTF-16 code-unit count happen to coincide.

**Extension: static grammar & language registration**

- **FR-021**: The extension MUST register a language for `.s`/`.block` files with a
  static TextMate grammar providing highlighting for control words, comments
  (including nested block comments), strings, and `@variable@` substitutions,
  functional independently of whether any `drut server` process is running or
  reachable.
- **FR-022**: The extension MUST provide bracket-matching and comment-toggling
  configuration consistent with Voyager script syntax.
- **FR-023**: Where the extension's language registration, bracket-matching, or
  comment-toggling configuration structurally references
  `bhereth.language-citilabscubevoyager` (permission granted 2026-08-08 per the
  constitution), it MUST port only structure/behavior, in Drut's own wording, and
  MUST NOT copy his grammar text or keyword lists verbatim, and his extension's
  files MUST NOT be committed to this repository in any form (constitution
  Principle II, binding conditions).

**Extension: language client**

- **FR-024**: The extension MUST spawn and manage a `drut server` process via a
  standard LSP client wrapper, surfacing the process's diagnostics, hover,
  completion, and semantic-token responses in the editor.
- **FR-025**: If the `drut server` binary cannot be found or fails to start, the
  extension MUST still deliver static highlighting (FR-021) and MUST surface a
  single, non-repeating notification about the missing/failed server rather than
  silently doing nothing or repeatedly erroring. "Single, non-repeating" means:
  at most one notification per distinct failure within a session — a
  missing-binary failure and a later, separately-caused failure (e.g. FR-026's
  crash-then-restart-also-fails case) are distinct occurrences and each gets its
  own single notification, but the same ongoing cause (e.g. the binary still
  being absent) is never re-notified repeatedly. Binary resolution MUST behave
  correctly per-platform (Windows/macOS/Linux `PATH` and executable-extension
  conventions, see Edge Cases), not only on the platform used during
  development.
- **FR-026**: If a running `drut server` process crashes, the extension MUST detect
  this, notify the user once (per FR-025's "single, non-repeating" definition),
  and make exactly one automatic restart attempt, without losing static
  highlighting in the interim. While a restart attempt is in progress, the
  editor MUST continue showing each open document's last-known diagnostics,
  hover, completion, and semantic-token results rather than clearing them —
  they are refreshed once the server reconnects and re-publishes, not held in
  an artificial empty state during the gap. If that one restart attempt also
  fails, this is a distinct, separately-notified failure per FR-025's
  definition — the extension does not attempt a second automatic restart.

**Extension: publishing**

- **FR-027**: The extension MUST be published to both the VS Code Marketplace and
  Open VSX under Drut's own publisher identity (not a fork or rebrand of any
  third-party extension).

**Definition-of-done validation**

- **FR-028**: The server's diagnostic output, exercised through an LSP-level test
  harness (not only voyager-core's own unit tests), MUST reproduce the same
  zero-false-positive result on the full valid-fixture corpus and the same
  correctly-flagged result on every deliberately-broken fixture that voyager-core
  and the CLI (002-cli-check-format) already prove independently — for the six
  diagnostic categories FR-005 covers. The hand-written fixture that exists
  specifically to trigger `InvalidEncoding` (`001-voyager-script-parser`'s
  `tests/fixtures/`) is excluded from this requirement's scope by construction,
  per FR-005's carve-out — it is not expected to (and cannot) reproduce that
  diagnostic through the editor.

### Key Entities

- **Server Session**: The running `drut server` process for a given client
  connection; owns the set of currently-open documents and their live text.
- **Open Document**: A file the editor has opened, tracked by the server as live
  text (not necessarily matching what's on disk) plus its derived diagnostics,
  hover facts, completion context, and semantic tokens.
- **Keyword Dictionary**: The hand-written, real-usage-derived list of control
  words and common `keyword=value` pair names (per FR-012's census methodology),
  shared by completion (Story 4) and spell-check (Story 5) — never sourced from
  vendor documentation (constitution Principle II).
- **Block Hover Fact**: The block kind and (if resolved) matched-counterpart
  location for a hovered opener/closer token (Story 3), reusing voyager-core's
  `Block` entity from `001-voyager-script-parser`.
- **Semantic Token**: A structure-derived highlighting classification layered over
  the static grammar's lexical categories — short-IF vs block-IF, and
  unreachable-after-`BREAK` (Story 6).
- **Position Translation**: The explicit char-count-to-UTF-16-code-unit conversion
  applied to every `Span` the server exposes over the LSP wire protocol (FR-019,
  FR-020).
- **Publisher Identity**: The Drut project's own account/namespace on the VS Code
  Marketplace and Open VSX, distinct from any third-party extension referenced for
  structural guidance (FR-023).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Opening any file from the full fixture corpus in the extension, with
  no server reachable, still shows full static highlighting (Story 1) — 100% of
  corpus files.
- **SC-002**: Opening every valid file in the full fixture corpus shows zero
  diagnostics, and opening every deliberately-broken fixture shows a diagnostic
  correctly identifying its injected defect — reproducing, through the editor, the
  same 100%-clean / 100%-flagged result already proven for voyager-core and the
  CLI, for the six diagnostic categories FR-005 covers (excludes the
  `InvalidEncoding`-triggering fixture, per FR-028's carve-out).
- **SC-003**: A user editing a file sees a newly-introduced structural defect's
  diagnostic appear, and a fixed defect's diagnostic disappear, without saving or
  reopening the file, within a perceptibly-immediate delay (no manual refresh
  action required). No numeric millisecond threshold is specified — deliberately
  left qualitative, the same "no evidence, no invented number" treatment applied
  to file-size/scale limits elsewhere in this spec, since no profiling baseline
  exists yet for this feature the way `voyager-core`'s own per-file parse time
  does (`001-voyager-script-parser/plan.md`); the practical ceiling this bar
  implies is bounded by that existing per-file parse-time budget (plan.md
  Performance Goals), not a new number invented for this criterion.
- **SC-004**: For every block opener/closer in the fixture corpus that voyager-core
  resolves to a matched counterpart (including implicitly-closed Run/Process
  blocks), hovering it in the editor reports that counterpart's correct location —
  100% agreement with voyager-core's own resolution.
- **SC-005**: For every character position in the fixture corpus, including
  supplementary-plane characters, every diagnostic/hover/completion/semantic-token
  response references the correct UTF-16 code-unit position — 100% agreement with
  what the editor itself considers that position to be, with zero drift cases.
- **SC-006**: A user can identify which of the seven block kinds a given
  opener/closer belongs to, for 100% of the fixture corpus's block
  openers/closers — verifiable at the protocol level as: every such token's
  hover response has its block-kind field populated (FR-008), independent of
  any subjective judgment about how legible the hover text itself reads.
- **SC-007**: The extension installs and activates from both the VS Code
  Marketplace and Open VSX under Drut's own publisher identity, with no
  installation step requiring a third-party extension to also be installed —
  verifiable directly from the packaged extension's manifest: its
  `extensionDependencies` field is empty.
- **SC-008**: An LSP-level test run over the full fixture corpus reproduces
  voyager-core's and the CLI's proven diagnostic results end-to-end through the
  server (Definition of Done) — the same 161/161-clean, all-broken-fixtures-flagged
  standard already established for the other two surfaces, scoped to FR-028's
  six-category carve-out.

## Assumptions

- **Position-encoding ownership is an open architectural decision, not a product
  decision** — FR-019 requires it be made and documented explicitly (in this
  phase's `plan.md`/`research.md`), but whether voyager-core's `Span` changes what
  it counts or this adapter crate owns the char-to-UTF-16 translation at its
  boundary does not change any user-facing behavior described in this spec, so it
  is deliberately left as an implementation decision rather than a
  `[NEEDS CLARIFICATION]` item here.
- **Completion's context-awareness depth is likewise a per-plan engineering
  decision**, bounded by FR-012: full control-word-scoped completion (restricting
  keyword suggestions to those a specific enclosing control word — `RUN`, `LOOP`,
  `PATHLOAD`, etc. — was observed taking during the census) is the target where
  feasible; if this phase's implementation cannot reach that depth, the documented
  fallback (general-syntax control words plus locally-inferable context) is an
  explicitly acceptable, spec-conformant outcome — whichever is actually achieved
  MUST be recorded in this phase's `plan.md` so it isn't left implicit. **This is
  deliberately not the same axis as FR-019's per-program-box scope** — nothing in
  this decision involves inspecting or branching on a `PGM=` (or similar)
  keyword's *value*; "context" here means only "which control word structurally
  encloses the cursor," never "which program that control word happens to be
  running." A future phase could add genuine per-program keyword knowledge on top
  of this one, but this feature does not, and this bullet's resolution in
  `research.md` §2 MUST NOT be read as having quietly done so.
- **Diagnostics update on every document change, not only on save** — matching
  standard LSP "live buffer" behavior (`textDocument/didChange`), consistent with
  Story 2's value proposition of catching defects as they're introduced.
- **No configuration file or per-workspace settings are introduced in this phase**
  — the server and extension behave the same way across every workspace; a
  `drut.toml`-style config (mentioned as a possible future item in
  002-cli-check-format's Assumptions) remains out of scope here too.
  **Narrowly qualified 2026-08-10**: this rules out a *Drut-invented*
  configuration surface that changes the server's or extension's own
  functional behavior per workspace (diagnostics, hover, completion,
  formatting — everything above still behaves identically everywhere, no
  exception). It does not cover the extension writing a value into VS
  Code's own pre-existing, generic `editor.semanticTokenColorCustomizations`
  setting purely for color rendering (see `extension.ts`'s
  `ensureVariableColorCustomization`, added the same day, resolving the
  real gap this session's manual testing surfaced: no TextMate scope or
  standard semantic token type is colored by every theme, a structural
  property of VS Code's theming model this extension cannot fix by choosing
  a different scope name). That write changes nothing about how the
  language server parses, diagnoses, or completes anything — the "server
  and extension behave the same way across every workspace" guarantee this
  bullet exists to protect is still fully intact; only the *rendered color*
  of `@variable@` references in a workspace that hasn't already customized
  it for itself differs, and even that is written to the workspace's own
  `.vscode/settings.json` (never the user's global config, never silently
  reapplied if removed), not a new file format or schema this project
  invents and must maintain.
- **Out of scope for this phase** (per the feature description): an MCP server; a
  per-program-box keyword validation (a hypothetical later Phase 5, which would
  need program-specific keyword knowledge this phase's dictionary does not
  attempt); and repo-wide semantic/reference checking across multiple files (a
  hypothetical later Phase 6). This phase is scoped to single-document,
  structurally-derived language features only.
- **The extension targets VS Code and other Open VSX-compatible editors only** — no
  other editor integration (e.g. a Neovim or JetBrains plugin) is in scope for this
  phase.
- **The fixture corpus and its validation methodology are inherited unchanged**
  from `001-voyager-script-parser` and `002-cli-check-format` — this phase adds no
  new fixture-sourcing work beyond what those phases already established (and the
  same open sourcing/licensing item noted there still applies).
- **The keyword dictionary is a new, hand-written artifact for this phase** — it
  reuses Phase 1's census *methodology* (structural-position classification
  against the fixture corpus) but is not assumed to already exist as a committed
  file; producing it is in scope for this phase's planning/implementation work.
- **Bhereth extension reference stays structure-only, as already bound by the
  constitution** — this spec adds no new permission or scope beyond what
  Principle II's binding conditions already state (see FR-023).
- **Hover's implicit-close reporting for `Process` blocks is a documented
  best-effort approximation, not a guaranteed-precise resolution** (added
  2026-08-09, `/speckit-analyze` finding I1; full derivation in research.md
  §10). Unlike `Run` (which has an `UnmatchedRun` diagnostic to confirm a
  block genuinely never closed), `Process` has no "unmatched" diagnostic
  category at all — so FR-009's counterpart location for a `Process` block
  with no explicit closer is `voyager-core`'s own resolved body-extent
  (`Block.span.end`) whether the block closed implicitly or, in the rare
  case it reaches end-of-file with no closer of any kind, never closed at
  all. The fixture corpus shows no real occurrence of the latter case
  (`001-voyager-script-parser`'s full-corpus validation), so this is
  expected to be a theoretical edge case, not a practical accuracy gap.
- **Actually publishing the extension to the VS Code Marketplace and Open
  VSX (FR-027, SC-007) is a release-process action, not a
  `/speckit-implement`-scoped task** (added 2026-08-09, `/speckit-analyze`
  finding E2) — this phase's tasks build, package, and validate the
  extension is ready to publish (a successful `vsce package`/`ovsx` dry-run
  under Drut's own publisher identity); the maintainer executes the actual
  publish afterward, the same way a release/deployment step sits outside an
  implementation task list in general. FR-027/SC-007 should be read as
  "ready to publish," with the real publish tracked as a follow-up action
  once implementation completes.
- **`InvalidEncoding` cannot be reported through live document editing, by
  construction of the LSP transport itself** (added 2026-08-09, resolving
  checklist CHK001/CHK028; full research in `research.md` §12). LSP's
  `textDocument/didOpen`/`didChange` payloads are JSON strings — JSON cannot
  represent an invalid byte sequence at all, so whatever text the server
  receives is already valid Unicode by the time it arrives, regardless of the
  file's original on-disk encoding. The editor (VS Code) has already decoded
  the file before ever sending its content: if the file's actual encoding was
  correctly detected, the server sees a faithful decode; if detection failed
  or a byte was genuinely undecodable, the editor's own decoder (standard
  non-fatal UTF-8 decoding) substitutes the Unicode replacement character
  (U+FFFD) — still ordinary, valid Unicode text from `voyager-core`'s
  perspective, not a trigger for its own `InvalidEncoding` diagnostic (which
  is specifically about `voyager-core`'s *own* byte-level decode fallback in
  `parse_bytes`, never invoked here since the server always calls `parse()`
  on already-decoded text, never `parse_bytes()`). **Considered and
  rejected**: having the server bypass the editor's supplied text and
  re-read the file's raw bytes from disk (via the document's URI) specifically
  to run `parse_bytes()`'s encoding check. Rejected because it would let the
  server's own Windows-1252-fallback guess disagree with what the editor
  actually decoded and displays — a diagnostic that doesn't correspond to
  what's on screen is worse than no diagnostic — and because it only works
  for a document that is actually saved to a real file path, reintroducing
  exactly the "diagnostics reflect stale disk content" problem this spec's
  Edge Cases otherwise explicitly rule out. Given `001-voyager-script-parser`'s
  full-corpus validation found zero real files exercising the genuinely-
  undecodable case at all (only a hand-written fixture does), this is a
  proportionate scope boundary, not a meaningful capability gap.
  `InvalidEncoding` remains fully available via `drut check` (CLI), which
  reads raw bytes directly from disk with no such transport constraint.
- **`vscode-languageclient`'s strict UTF-16-only `positionEncoding`
  validation (research.md §1, point 4) is a monitored external dependency,
  not a one-time finding** — if a future `vscode-languageclient` release
  relaxes this restriction, research.md §1's decision should be revisited,
  though nothing in this spec requires acting on that unless/until it happens.
- **`lsp-types` 0.97.0 being the newest stable release despite being ~2 years
  old (research.md §11) is similarly a standing item to periodically
  re-check**, not a closed one-time observation — no action is required this
  phase, since none of this feature's scope needs the newer LSP 3.18 surface
  `lsp-types` lacks.
- **Exercising the fixture corpus through the LSP protocol layer introduces
  no new corpus-availability or environment dependency** beyond what
  `001-voyager-script-parser`/`002-cli-check-format` already require — it is
  the same external, non-committed corpus, gated the same way
  (`DRUT_CORPUS_PATH`, `#[ignore]`), just exercised through a different
  in-process test harness (`lsp_server::Connection::memory()`, research.md
  §9) rather than a different corpus or a different availability
  requirement.
- **`textDocument/formatting` was added 2026-08-10, outside this feature's
  original scope** (the original feature description named diagnostics,
  hover, completion, spell-check, and semantic tokens only) — surfaced
  during the first-ever hands-on manual VS Code verification of this
  feature: "Format Document"/format-on-save doing nothing at all, with
  `voyager_core::format` already fully built, golden-fixture-tested, and
  shipped via `drut format` (`002-cli-check-format`), was a real, concrete
  editor-usability gap, not a hypothetical one. Implemented as a thin
  wrapper (`drut-lsp/src/formatting.rs`) returning one whole-document
  `TextEdit` — no new whitespace/casing logic, Principle I holds exactly as
  it does for every other capability this feature adds. Casing stays
  untouched (`FormatOptions::default()`) for the reason already stated
  under FR-015/completion's Assumptions: no per-workspace settings surface
  exists yet for an opt-in casing choice. Extension-side wiring required no
  change at all — `vscode-languageclient` registers a document formatting
  provider automatically once the server declares
  `document_formatting_provider: true`; VS Code's own "Format Document"
  command and `editor.formatOnSave` setting are both already generic
  clients of that registration.
- **The same manual verification pass also surfaced a real, separate
  `editors/vscode` bug, unrelated to formatting** (2026-08-10): `extension.
  ts`'s `ServerOptions` set `transport: TransportKind.stdio`, which causes
  `vscode-languageclient` to append a `--stdio` flag to the spawned
  process's args — a convention some language servers (rust-analyzer,
  clangd) opt into but `drut server` (`cli.rs`'s `Server` variant, a bare
  flagless subcommand) never asked for or accepts. Every real launch failed
  immediately: `clap` rejected the unrecognized flag, the process exited
  before the LSP handshake began, and diagnostics/hover/completion never
  worked at all — while automated tests never caught this, since
  `Connection::memory()`-based protocol tests (research.md §9) drive
  `drut_lsp::run` in-process and never actually spawn `drut server` as a
  real child process the way `vscode-languageclient` does, so this class of
  bug was structurally invisible to every test in this feature's own suite.
  Fixed by removing the `transport` field entirely — a plain `command`+
  `args` `ServerOptions` already communicates over stdio by default. No
  automated regression test added for this specific bug, since it lives
  entirely in how `vscode-languageclient` spawns a subprocess, outside what
  any test in this repository (Rust or the grammar's own `vscode-textmate`
  suite) actually exercises; caught only by, and now guarded only by, real
  manual verification — which is exactly why this spec's Definition of Done
  called for a human to watch it work at least once before trusting it.
- **The extension's static TextMate grammar was substantially enriched
  2026-08-10** (numeric literals, operators, pair-keyword names as
  Python-style "parameters," pair values as generic constants, bracket/
  comma punctuation, string escape sequences), again outside this feature's
  original "control words, comments, strings, `@variable@`" grammar scope
  (FR-021) — driven directly by the same real manual verification pass,
  once the `--stdio` bug above stopped masking everything downstream of a
  working connection. All additions are structural/lexical categories
  (number shapes, operator symbols, a generic `word[subscript]=`/`=word`
  positional pattern), not a hand-curated vendor-sourced keyword list, so
  Principle II holds the same way it already did for `statement-words`
  (FR-021's original real-usage-evidenced addition).
- **A single custom TextMate scope is not a reliable way to guarantee a
  token gets *any* distinct color at all, discovered twice via real manual
  testing 2026-08-10** — first for `@variable@`'s original
  `variable.other.substitution` scope (fixed by switching to the more
  conventional `variable.other.readwrite`), then again for that very
  replacement under a different real theme ("Dark 2026"), which also had no
  rule for it. A theme only colors a scope its own author wrote a rule for;
  an extension-defined custom scope (however conventionally named) has no
  guarantee any given theme covers it, and — as this second occurrence
  showed — even a "conventional" choice doesn't reliably fix this in
  general, only for whichever themes happen to already handle it. **The
  general answer, resolved this same day**: standard LSP semantic token
  types (semantic_tokens.rs, see its own dated comment and this file's
  `textDocument/semanticTokens/full` contract entry) get a built-in
  baseline color from VS Code's own editor even when the active theme
  defines nothing for them — the correct mechanism when the goal is a
  reliably-visible color across arbitrary themes, TextMate scopes are the
  right tool for *classification* but not a color guarantee. `@variable@`
  was ported to this mechanism (tagged with the standard `variable`
  semantic type); other newly-added scopes from the entry above were left
  on TextMate-only classification, since they were not reported as
  invisible under the themes tested and porting every scope to semantic
  tokens preemptively, without a demonstrated need, would be scope creep
  beyond what real testing actually surfaced.
