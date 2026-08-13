# Feature Specification: Live Diagnostic Updates on Config File Edits

**Feature Branch**: `013-lsp-config-file-watch`

**Created**: 2026-08-13

**Status**: Draft

**Input**: User description: "A real bug found during 012-toml-configuration's own
manual verification: drut-lsp's diagnostics for an open .s/.block document go stale
when the drut.toml file governing that document is edited directly, without the
.s/.block document itself being closed and reopened. Fix: detect drut.toml changes
via the editor's own standard file-change-notification mechanism and refresh
diagnostics for every affected open document, with a formal graceful-degradation
path for editors that don't support this, and the broad (not narrowly-scoped)
detection approach recorded as a deliberate tradeoff."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A configuration fix or mistake shows up immediately, without reopening anything (Priority: P1)

An analyst has a script open in their editor with a diagnostic pointing at a problem
in the project's `drut.toml` (for example, a mistyped setting value). They edit
`drut.toml` directly to fix it — or to fix one mistake and accidentally introduce a
different one — without closing or reopening the script file they were already
looking at. They expect the diagnostic to reflect what `drut.toml` actually says
right now, not what it said when the script was first opened.

**Why this priority**: This is the bug being fixed, in its exact reported form. It
is the entire reason this feature exists — every other story is a refinement or
safety net around this core behavior.

**Independent Test**: With a script open showing a config-related diagnostic, edit
the governing `drut.toml` to a different value (valid or invalid) without touching
the script file itself; confirm the diagnostic updates (changes, appears, or
disappears as appropriate) to match the new content, with no need to close or
reopen the script.

**Acceptance Scenarios**:

1. **Given** a script file open with a diagnostic naming a specific invalid value in
   its governing `drut.toml`, **When** the user edits that `drut.toml` to a
   *different* invalid value, without closing or reopening the script file,
   **Then** the diagnostic updates to name the new invalid value — this is the
   exact sequence that surfaced the bug, and MUST be the primary regression test
   for this feature, not merely one acceptance criterion among many.
2. **Given** multiple script files open at once, all governed by the same
   `drut.toml` (or by different `drut.toml` files that could each be affected by
   one edit), **When** that `drut.toml` is edited, **Then** every affected open
   document's diagnostics refresh — not only whichever document currently has
   editor focus.
3. **Given** a script file open with no config-related diagnostic (a valid
   `drut.toml`, or none at all), **When** the user edits `drut.toml` to introduce a
   mistake, **Then** a new diagnostic appears on the open script file without any
   action taken on that file itself.
4. **Given** a script file open with a config-related diagnostic, **When** the user
   corrects the underlying `drut.toml` mistake, **Then** the diagnostic disappears
   without closing or reopening the script file.

---

### User Story 2 - Predictable, unbroken behavior on an editor that doesn't support this (Priority: P2)

An analyst uses an LSP-capable editor that doesn't support asking it to watch for
file changes on the tool's behalf. They should never notice anything break because
of that — no crash, no error message, no unexpected behavior change. They simply
keep the same experience they had before this feature existed: a `drut.toml` edit
is picked up the next time the script file itself is opened or edited, not
instantly.

**Why this priority**: A correctness and safety requirement, not the feature's main
value — but essential, since attempting this feature's core mechanism against an
editor that can't support it must never be allowed to degrade or break anything for
that population of users.

**Independent Test**: Using an editor session that does not indicate support for
this capability, confirm the editor session starts normally with no error, and that
editing `drut.toml` without touching an open script file produces no diagnostic
update until that script file is itself reopened or edited — matching behavior from
before this feature existed, exactly.

**Acceptance Scenarios**:

1. **Given** an editor that does not indicate support for being asked to watch
   files on the tool's behalf, **When** the editor session starts, **Then** no
   attempt to activate this feature is made, and the session proceeds normally
   with no error surfaced to the user.
2. **Given** such an editor session, **When** `drut.toml` is edited without the
   open script file being touched, **Then** the open script file's diagnostics do
   not change until that file is itself reopened or edited — the same limitation
   that existed before this feature, not a new defect.
3. **Given** an editor that *does* indicate support and receives the tool's request
   to activate this feature, **When** the editor never confirms that request (no
   response ever arrives, the response arrives but indicates the request failed,
   or the response is delayed), **Then** the system continues handling every other
   request and notification normally throughout — nothing about the rest of the
   session waits on, or is blocked by, that one unconfirmed request. The
   practical effect is the same as User Story 2's own core case: config-only edits
   go undetected until the affected document is itself reopened, but nothing else
   about the session is degraded.

---

### Edge Cases

- **`drut.toml` is deleted** while a script file that depended on it is still open:
  diagnostics for that file refresh to reflect no configuration file being present
  (built-in defaults, matching 012's own "no file anywhere" behavior) — not left
  stuck referencing a file that no longer exists.
- **A new `drut.toml` appears** closer to an open script file than whatever it was
  previously resolving to: diagnostics refresh to reflect the newly-closer file,
  the same as if the file had just been opened fresh.
- **A workspace with no `drut.toml` anywhere**: detection is still active (per the
  broad-scope design — see Assumptions), it simply never has anything to react to;
  zero behavior change, zero errors.
- **Many open documents at once**: covered directly by User Story 1's Acceptance
  Scenario 2 and by the scale question addressed explicitly in Assumptions below.
- **The editor accepts the request to activate this feature but never confirms
  it** (no response, a failure response, or a slow response): this MUST NOT block
  or delay any other part of the session — see US2 Acceptance Scenario 3 and
  FR-010. This is a genuinely new question this feature introduces (the tool has
  never asked an editor to do anything before), not something an earlier feature
  already had to answer.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST detect when a `drut.toml` file anywhere in the active
  workspace is created, changed, or deleted, for the duration of an editor session.
- **FR-002**: Upon detecting such a change, the system MUST re-evaluate and
  re-publish diagnostics for every currently open document whose diagnostics could
  be affected by it — not only whichever document is currently focused or being
  actively edited.
- **FR-003**: This detection MUST rely on the same standard change-notification
  mechanism the editor already uses for reporting file changes to any language
  tool, not a custom or proprietary one.
- **FR-004**: The system MUST NOT attempt to activate this detection against an
  editor session that has not indicated it supports being asked to do so. For such
  a session, the system MUST continue operating exactly as it did before this
  feature existed — no error, no crash, no other behavior change — with the sole,
  known limitation that a `drut.toml`-only edit is not detected until the affected
  document is itself reopened or edited.
- **FR-005**: A single edit to a `drut.toml` file MUST NOT require the user to
  manually close and reopen any already-open document to see diagnostics that
  reflect that edit (on an editor session where detection is active).
- **FR-006**: An updated diagnostic MUST reflect the configuration file's current,
  latest content at the moment of re-evaluation — never a previously-seen value
  from before the most recent edit.
- **FR-007**: Detection MUST NOT be narrowed to only the exact configuration file
  paths already known to govern a currently open document. Detecting any
  `drut.toml` change anywhere in the active workspace is acceptable, even where
  that triggers a re-evaluation for a document that turns out to be unaffected by
  that particular change (see Assumptions for the reasoning behind this choice).
- **FR-008**: A re-evaluation triggered by this mechanism for a document whose
  effective, resolved configuration did not actually change as a result MUST NOT
  produce any visible diagnostic change for that document — re-evaluating an
  unaffected document is invisible to the user, not a source of flicker or
  duplicate/spurious diagnostics.
- **FR-009**: This capability MUST NOT introduce any state that persists beyond the
  current editor session — detection and re-evaluation rely only on the live,
  running session, matching this project's existing "recompute fresh, never cache"
  posture for configuration resolution.
- **FR-010**: The system MUST NOT block, delay, or otherwise make dependent any
  other request or notification handling on receiving confirmation that this
  feature was successfully activated with the editor. A confirmation that never
  arrives, arrives indicating failure, or arrives late MUST have no effect on the
  session beyond this feature's own detection simply not being active — every
  other capability MUST continue operating normally throughout, with no
  observable delay attributable to waiting for that confirmation. This is an
  explicit decision, not an implicit consequence of how the confirmation happens
  to be handled — this project has never before asked an editor to confirm
  anything, so this specific failure mode has no earlier precedent to fall back
  on and needed its own stated answer.

### Key Entities

- **Watched Configuration Change**: An event representing a `drut.toml` file
  somewhere in the active workspace having been created, modified, or removed
  during the current editor session. Not a persistent record — exists only as a
  trigger for immediate re-evaluation, then discarded.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On an editor session where this feature is active, a user editing
  `drut.toml` sees every affected, already-open document's diagnostics reflect that
  edit within the same session, without closing or reopening any document,
  every time.
- **SC-002**: On an editor session where this feature is not active (unsupported
  editor), no new error or crash is ever introduced by this feature — behavior
  matches exactly what existed before it, with only the known, accepted limitation
  of delayed detection.
- **SC-003**: Editing one `drut.toml` correctly refreshes diagnostics for every
  currently open document it could affect in one action — not only the document
  the user happens to be looking at.
- **SC-004**: The delay a user perceives between saving a `drut.toml` edit and
  seeing an affected document's diagnostics update is not noticeably different from
  the delay they already experience today when editing the script file itself —
  the broader-than-strictly-necessary detection scope (Assumptions) does not
  introduce a perceptible slowdown at drut's real target project sizes.
- **SC-005**: An editor session's responsiveness for every other capability
  (opening files, hovering, formatting, and so on) is never affected by whether
  this feature's own activation was confirmed, still pending, or failed — a user
  on an editor where activation silently fails experiences exactly User Story 2's
  limitation and nothing more, never a slowdown or hang anywhere else in the
  session.

## Assumptions

- **Detection scope is deliberately broad, not narrowed to each document's own
  resolved-configuration ancestry — a stated tradeoff, not an oversight.**
  Detecting any `drut.toml` anywhere in the active workspace, rather than only the
  specific file(s) each open document's own upward search actually resolved to, is
  simpler to reason about and test, and structurally guarantees no affected
  document is ever missed (the alternative — precisely tracking which
  configuration file governs which open document, and keeping that mapping correct
  as files are created, moved, or deleted — is meaningfully more complex for a
  benefit that doesn't change user-observable correctness, only how much
  unnecessary re-evaluation work happens on an edit that turns out to be
  irrelevant to a given document). **On scale**: re-evaluating a document's
  diagnostics is already a fast, cache-free operation performed on every normal
  document edit today; doing it for every open document on every `drut.toml`
  change, rather than only affected ones, is judged a non-issue at drut's real
  target scale — Cube Voyager projects with realistic file counts and a practical
  number of simultaneously open editor tabs (tens, not thousands) — not the kind
  of scale (very large monorepos with very many simultaneously open documents)
  where this tradeoff's cost would become perceptible. This should be revisited
  only if real usage ever demonstrates otherwise, not preemptively optimized now.
- Most modern LSP-capable editors, including VS Code, support the standard
  mechanism this feature relies on to ask an editor to report file changes on the
  tool's behalf. The graceful-degradation path (User Story 2) exists as a
  correctness and safety requirement for editors that don't, not because that's
  expected to be the common case.
- Detection covers files literally named `drut.toml` anywhere under the
  workspace root(s) the editor session knows about at startup — consistent with
  012-toml-configuration's own file-naming decision.
- This feature changes only how promptly an already-correct re-evaluation
  mechanism (012's own config resolution, already fresh/uncached on every call) is
  *triggered* — it does not change how configuration is discovered, parsed, or
  resolved in any way.
