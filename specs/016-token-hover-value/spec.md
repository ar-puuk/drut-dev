# Feature Specification: Token Hover Shows Assigned Value

**Feature Branch**: `016-token-hover-value`

**Created**: 2026-08-16

**Status**: Draft

**Input**: User description: "Hover-over-@token@ shows its assigned value. When the
user hovers over an @token@ reference in a .s/.block file in the VS Code extension,
the LSP hover response should show the value most recently assigned to that token
name, so the user doesn't have to scroll up or open another file to find where/what
it was set to. Scope: the token's own open document, plus one level of literal
(non-token-built) `READ FILE = '<path>'` statements that document itself contains —
Voyager's real, textually-spliced-in-place cross-file inclusion mechanism (confirmed
against the real WF-TDM-Development corpus: a scenario's orchestrator script reads a
flat list of sibling files, including a 'control center' file that sets shared
tokens like ParentDir). Deeper, token-built READ FILE paths (e.g.
'@ParentDir@sub\path.block') are out of scope for this feature — resolving them
would require evaluating a token before the path pointing at its own definition can
even be found, which is a materially bigger problem left for later. If no matching
assignment is found anywhere in scope, hover falls back to existing behavior (no
crash, no fabricated value)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See a same-file token's value without scrolling (Priority: P1)

An analyst is reading a `.s` script and sees `@ZoneMsgRate@` used partway through a
long file. Rather than scrolling up (or using find) to locate the assignment that
set it, they hover over the token and see its current value right there.

**Why this priority**: This is the core of the original request — the case that
needs zero cross-file machinery and delivers the bulk of the value on its own, since
most tokens used in a script are also assigned somewhere earlier in that same
script.

**Independent Test**: Open a `.s`/`.block` file containing a `TOKEN = value`
assignment followed later by `@TOKEN@`, hover over the `@TOKEN@` reference, and
confirm the hover shows `value` and where it was assigned.

**Acceptance Scenarios**:

1. **Given** a file with `ZoneMsgRate = 50` on an earlier line and `@ZoneMsgRate@`
   used later, **When** the user hovers over `@ZoneMsgRate@`, **Then** the hover
   shows `50` and the line number where it was assigned.
2. **Given** a file where the same token is assigned more than once before the
   hovered reference (e.g. reassigned inside a later block), **When** the user
   hovers over the reference, **Then** the hover shows the value from the
   assignment closest to (but not after) the hovered reference, not the first one
   in the file.
3. **Given** a file where a token is assigned *after* the hovered reference (never
   before it), **When** the user hovers over that reference, **Then** the hover does
   not show that later assignment's value — an assignment that hasn't executed yet
   at that point in the script is not a legitimate answer to "what is this token
   right now."

---

### User Story 2 - See a value set in a directly-read "control center" file (Priority: P2)

An analyst opens one of a scenario's model-step scripts (not the orchestrator
itself) and hovers over `@ParentDir@` or `@UsedZones@` — a token this file uses but
never assigns itself. The value was set in a separate file, `_ControlCenter.block`
or `GeneralParameters.block`, that this same script pulls in near its top via
`READ FILE = '_ControlCenter.block'`. The user expects the hover to still show the
value, without them having to go open that other file themselves.

**Why this priority**: This is the "control center" case the original request
specifically named. It requires real, if narrow, cross-file work (reading a file
that may not currently be open in the editor), so it is scoped as a second,
separable increment on top of User Story 1, not folded into it.

**Independent Test**: Open a `.s` file that contains `READ FILE = 'sibling.block'`
where `sibling.block` assigns `TOKEN = value` and is never reassigned in the open
file itself; hover over an `@TOKEN@` reference in the open file positioned after the
`READ FILE` line, and confirm the hover shows `value` and names `sibling.block` as
the source.

**Acceptance Scenarios**:

1. **Given** an open file with `READ FILE = 'GeneralParameters.block'` on an earlier
   line, where that file assigns `UsedZones = 3629` and the open file never assigns
   `UsedZones` itself, **When** the user hovers over `@UsedZones@` later in the open
   file, **Then** the hover shows `3629` and identifies `GeneralParameters.block` as
   where it was set.
2. **Given** the same setup, **When** the open file *also* assigns `UsedZones` to a
   different value on a line after the `READ FILE` line but before the hovered
   reference, **Then** the hover shows the open file's own later value, not the one
   from `GeneralParameters.block` — matching the same top-to-bottom "most recent
   wins" rule as User Story 1, now applied across the point where the read file's
   content is spliced in.
3. **Given** an open file with a `READ FILE = '@ParentDir@sub\path.block'` line
   (a token-built path, not a literal string), **When** the user hovers over a
   token that is only ever assigned inside that unresolvable target file, **Then**
   the hover does not show a value for it (falls back to existing behavior) — this
   feature does not attempt to evaluate a token to resolve a path that depends on
   it.
4. **Given** an open file with a `READ FILE = 'missing.block'` line pointing at a
   file that does not exist on disk (a stale reference — confirmed to occur in real
   scripts), **When** the user hovers over a token that would only be found in that
   file, **Then** the hover falls back to existing behavior — no crash, no error
   surfaced to the user for this.

---

### User Story 3 - No answer is not a false answer (Priority: P3)

An analyst hovers over an `@token@` reference whose value this feature cannot
determine — it isn't assigned anywhere in the open file or in any file the open
file directly reads via a literal `READ FILE`. They see the same hover experience
that existed before this feature (block-structure info or a spell-check nudge, or
simply no hover), never a wrong or fabricated value.

**Why this priority**: A correctness guardrail rather than the feature's own value —
but essential, since a plausible-looking wrong value is worse than no value at all
for a tool whose entire purpose is to save a trip to go check the real source.

**Independent Test**: Hover over an `@token@` reference with no discoverable
assignment in scope, and confirm no value is fabricated or guessed — either no
token-value hover content appears, or existing hover behavior (e.g. a spell-check
nudge) is unaffected.

**Acceptance Scenarios**:

1. **Given** an `@token@` reference with no matching assignment anywhere in scope
   (same file or a directly, literally read file), **When** the user hovers over
   it, **Then** no value is shown for it, and no other part of the hover response is
   degraded or altered because of the failed lookup.
2. **Given** an `@token@` reference whose name is only one edit away from a
   known-real token name that *does* have a value in scope, **When** the user
   hovers over the misspelled reference, **Then** the system does not guess and show
   the near-match's value — a wrong value with high apparent confidence is
   explicitly worse than no value here.

---

### Edge Cases

- **Token name casing differs between assignment and reference** (e.g.
  `ParentDir = ...` assigned, `@PARENTDIR@` referenced) — matching MUST be
  case-insensitive, consistent with how this project already treats every other
  Voyager identifier/keyword (see `voyager-core`'s existing
  `eq_ignore_ascii_case` convention throughout its keyword and statement matching).
- **A `READ FILE` line appears in the open file, but at or after the hovered
  reference's own position** — its contents are not yet "in scope" for that
  reference (mirrors User Story 1's own "not-yet-executed" rule from Acceptance
  Scenario 3), since Voyager splices a read file's content in at the exact line the
  `READ FILE` statement occupies.
- **The same file is `READ FILE`'d more than once** in the open document (observed
  in the real corpus, e.g. multiple scenario variants reading overlapping setup
  files) — each occurrence is treated independently at its own position, per the
  same top-to-bottom ordering rule; no special-casing needed.
- **A `READ FILE`'d file itself contains further `READ FILE` statements** (a second
  level of nesting) — explicitly out of scope; only the directly-read file's own
  assignments are considered, never anything *it* in turn reads. This bound is a
  deliberate simplicity/complexity tradeoff (see Assumptions), not an oversight.
- **A `READ FILE` statement literally targets the open document itself** (a direct
  self-reference, or a target that happens to resolve to the same file on disk) —
  no special-casing is required: that file's assignments are already part of the
  same-file scope, so including them a second time changes nothing observable
  (the most-recent-wins rule is unaffected by a duplicate candidate).
- **The hovered `@token@` reference's name is empty or otherwise malformed** (e.g.
  adjacent `@@`) — treated the same as any other name with no matching assignment:
  falls back to existing behavior (FR-008), not a special error case.
- **An assignment's target uses a bracketed subscript** (e.g. `MW[1] = ...`) while
  the hovered reference is a bare name (e.g. `@MW@`) — these are never considered a
  match for each other; matching is by the token's full name exactly as written
  between the `@`s, not a prefix or partial match.
- **The `READ FILE`'d file is not a `.s`/`.block` file, or fails to parse
  cleanly** — treated as contributing no assignments, not as an error surfaced to
  the user.
- **Hovering inside the "control center" file itself** (not the orchestrator) over
  a token it assigns to itself, or reads from a file it in turn reads — this is
  ordinary User Story 1 same-file behavior for the first case; the second case is
  the same one-level bound above, now anchored at whichever file is currently open.
- **No workspace folder, or the `READ FILE` target's resolved path escapes any
  workspace boundary** — resolution is purely relative to the open document's own
  location on disk; there is no workspace-root concept this feature depends on.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When the user hovers over an `@token@` reference, the system MUST
  search for the most recent Assignment statement (per FR-004's ordering rule) that
  sets a target matching that token's name, using the search scope defined by
  FR-002 and FR-003.
- **FR-002**: The search scope MUST include every Assignment statement in the
  hovered reference's own open document.
- **FR-003**: The search scope MUST additionally include every Assignment statement
  found in a file referenced by a `READ FILE = '<literal path>'` statement in the
  hovered reference's own open document, where `<literal path>` contains no
  `@token@` substitution of its own. The system MUST NOT recurse into that
  referenced file's own `READ FILE` statements (one level of inclusion only — see
  Assumptions).
- **FR-004**: Ordering across the combined scope (FR-002 + FR-003) MUST follow
  Voyager's real execution order: a `READ FILE` statement's target file's
  assignments are treated as occurring at that statement's exact position in the
  open document, interleaved with the open document's own assignments — not as a
  separate, lower-priority tier searched only after the open document is exhausted.
  The system MUST select the single assignment closest to, but not after, the
  hovered reference's own position under this ordering.
- **FR-005**: Token name matching (both the hovered `@token@` and each candidate
  Assignment's target) MUST be case-insensitive, consistent with every other
  identifier/keyword comparison already established in `voyager-core`.
- **FR-006**: The system MUST resolve a `READ FILE` path relative to the hovered
  document's own location on disk, and MUST read that target file's content
  directly from disk (it is not required to already be open in the editor). A
  literal path that is already absolute (rather than relative) MUST resolve to
  itself, not be joined onto the hovered document's directory — standard path-join
  behavior, not a special case this feature adds.
- **FR-007**: If a `READ FILE` target does not exist on disk, cannot be read, or
  does not parse as a valid `.s`/`.block` document, the system MUST treat it as
  contributing zero assignments to the search scope — never an error surfaced to
  the user, and never a crash (consistent with `voyager-core`'s existing
  never-panic contract).
- **FR-008**: If no assignment is found anywhere in the defined search scope for a
  hovered `@token@` reference, the system MUST fall back to the hover behavior that
  existed before this feature (e.g. a spell-check nudge, or no hover content) —
  never a fabricated, guessed, or near-match value.
- **FR-009**: The hover response for a resolved token value MUST indicate where the
  value came from (the open document itself, or the specific `READ FILE`-referenced
  file, by name) — not only the bare value, so the user can still verify it at the
  source if they choose to.
- **FR-010**: This feature MUST NOT change hover behavior for anything this project
  already handles today (block-opener/closer info, the spell-check nudge) except by
  adding new content specifically for `@token@` references that resolve to a value
  under FR-001–FR-004.

### Key Entities

- **Token Assignment Scope**: The bounded set of Assignment statements considered
  when resolving one `@token@` hover — the open document's own statements, plus
  (per FR-003) the statements of any file it directly, literally `READ FILE`s.
  Computed fresh on each hover request; not cached or persisted across requests.
- **Read-File Reference**: A single `READ FILE = '<literal path>'` statement in an
  open document, resolved to an on-disk file path relative to that document's own
  location. Only literal (non-token-built) paths participate; a token-built path is
  recorded as present but deliberately not resolved.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user hovering over an `@token@` reference assigned earlier in the
  same open file sees its current value without leaving that file, every time such
  an assignment exists.
- **SC-002**: A user hovering over an `@token@` reference whose value comes from a
  file their open document directly reads via a literal `READ FILE` statement sees
  that value without manually opening the other file, every time such an
  assignment exists and the referenced file is reachable on disk.
- **SC-003**: Every automated test and manual check confirms a hover-reported token
  value always matches the real, most-recent-per-execution-order assignment (per
  FR-004's ordering rule) — no test run, fixture, or manual check ever surfaces a
  mismatch.
- **SC-004**: A user hovering over a token this feature cannot resolve (out-of-scope
  cross-file case, missing file, or genuinely unassigned token) sees the same
  experience they had before this feature existed — no crash, no error, no
  degraded hover for anything else on that line.

## Assumptions

- **One level of `READ FILE` inclusion is a deliberate, permanent scope boundary,
  not a stepping stone to eventually recursing deeper.** Verified against the real
  WF-TDM-Development corpus: the vast majority of nested, multi-level `READ FILE`
  usage builds its path from a token (e.g. `@ParentDir@...`) that is itself only
  resolvable after reading the very "control center" file this feature already
  reaches at one level deep — going further would require a general token-in-path
  evaluator, a substantially bigger feature with its own correctness questions
  (what if the token used in the path is *itself* ambiguous or unresolved?), not a
  small extension of this one.
- **No reverse ("who reads me") resolution.** This feature only follows `READ FILE`
  statements the currently open/hovered document itself contains; it does not
  attempt to discover or search files elsewhere in the workspace that might read
  *this* document, even though the real corpus's orchestrator scripts are
  structurally exactly that (many-to-one fan-in, confirmed in research). A user
  hovering inside a deeply-nested model-step script for a token only ever set by
  the orchestrator that reads *it* (rather than the reverse) will not get a value
  from this feature.
- **Voyager token/identifier names are case-insensitive**, matching this project's
  existing, established convention (`voyager-core` already treats every keyword and
  identifier comparison this way) and confirmed by the real corpus containing the
  same token spelled with different casing (`ParentDir` / `PARENTDIR`) referring to
  the same value.
- **Reads are always fresh, never cached across hover requests**, consistent with
  this project's existing posture for configuration resolution (013's own
  "recompute fresh" decision) — a `READ FILE`-referenced file that changes on disk
  between two hovers is reflected on the very next hover, with no stale-cache
  window to reason about.
- **This feature extends the existing `textDocument/hover` handler** in
  `crates/drut-lsp/src/hover.rs`; it does not introduce a new LSP capability or
  request type.
