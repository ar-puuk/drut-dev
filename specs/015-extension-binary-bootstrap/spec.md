# Feature Specification: Extension Binary Bootstrap ("Batteries Included")

**Feature Branch**: `015-extension-binary-bootstrap`

**Created**: 2026-08-13

**Status**: Draft

**Input**: User description: "Extension 'batteries included' binary bootstrap —
the last blocker before publish (ROADMAP.md item 7). On activation, the VS Code
client must resolve a working `drut` binary without requiring the user to
install anything manually first, using D2's actual shipped release.yml output
(4 platform binaries + .sha256 sidecars, drut-<target-triple>.<ext> naming) as
the concrete input. Resolution priority: PATH (unchanged) → extension storage
→ download-verify-install from the latest GitHub Release. Graceful degradation
to highlighting-only on any failure or unsupported platform. Version-staleness
handling decided explicitly: throttled background check, storage-only scope,
non-blocking, explicit dismissible update offer — never silent auto-replace."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The extension works immediately after installing from the Marketplace (Priority: P1)

A user with no `drut` binary anywhere on their machine installs the Drut
extension from the VS Code Marketplace or Open VSX and opens a `.s`/`.block`
file. Without running any command, editing any setting, or downloading
anything themselves, they get full diagnostics, hover, completion, and
formatting — not just static syntax highlighting.

**Why this priority**: This is the entire point of the feature and the
literal last blocker before this project can be published at all — a v1
that isn't "batteries included" fails the project's own stated bar the
moment someone installs it (ROADMAP.md item 7).

**Independent Test**: On a supported platform, with no `drut` on PATH and no
prior activation of this extension, open a `.s` file and confirm a
diagnostic appears for a deliberately broken script within a few seconds of
activation, with no user action taken beyond opening the file.

**Acceptance Scenarios**:

1. **Given** a supported platform (Windows x64, macOS x64/arm64, or Linux
   x64) with no `drut` on PATH and no prior extension activation, **When**
   the extension activates, **Then** it downloads, verifies, and installs
   the matching binary from the latest GitHub Release, and the language
   server starts using it — with no user action beyond opening a file.
2. **Given** the extension has already downloaded and installed a binary on
   a previous activation, **When** the extension activates again, **Then**
   it reuses the already-installed binary directly — no re-download, no
   repeated network call for basic startup.

---

### User Story 2 - An existing PATH-based install is never second-guessed (Priority: P1)

A developer who built `drut` from source (or installed it some other way)
already has it on their PATH. They install or already have the extension.
Nothing about this feature changes their experience — their own binary is
used, exactly as it is today.

**Why this priority**: Equal weight to User Story 1 — this feature adding a
new resolution path must not regress the one that already works today for
every developer and contributor on this project. A downloaded copy silently
overriding a deliberately-installed PATH binary would be a real regression,
not an improvement.

**Independent Test**: With a real, working `drut` on PATH, activate the
extension and confirm (e.g. via the existing startup log line reporting the
running binary's path) that the PATH-resolved binary is the one actually
running — never a downloaded copy, even if one exists in extension storage
from an earlier session.

**Acceptance Scenarios**:

1. **Given** `drut` is present and resolvable on PATH, **When** the
   extension activates, **Then** that PATH-resolved binary is used, and no
   download is attempted at all.
2. **Given** `drut` is present on PATH *and* a previously-downloaded copy
   already exists in extension storage from an earlier session, **When**
   the extension activates, **Then** the PATH binary still wins — the
   stored copy is never preferred over it.

---

### User Story 3 - Unsupported platforms and failures degrade gracefully, exactly as today (Priority: P2)

A user is offline, on a network that can't reach GitHub, or on a
platform/architecture this project doesn't publish a binary for (e.g. Linux
arm64). The extension still activates cleanly: static syntax highlighting
works, and the user gets a single, clear, one-time explanation of why the
richer features aren't available — never a crash, never a wall of repeated
notifications, never a broken half-installed state.

**Why this priority**: This is the existing safety net (`notifyOnce`'s
single-non-repeating-notification pattern, already proven in production for
"missing binary"/"crash" cases) — this feature must extend it, not weaken
it. A user with no working install today must never end up *worse off*
after this feature ships.

**Independent Test**: Simulate an unreachable GitHub API (or an unsupported
`process.platform`/`process.arch` combination) and confirm the extension
activates with highlighting fully functional, exactly one notification
appears explaining the specific reason, and no further notification repeats
on subsequent activations for the same ongoing cause.

**Acceptance Scenarios**:

1. **Given** an unsupported platform/architecture combination, **When** the
   extension activates, **Then** no download is attempted, highlighting
   works, and a single notification names the platform/architecture as
   unsupported — not the generic "could not start" message.
2. **Given** a supported platform but no network access (or GitHub
   unreachable/rate-limited), **When** the extension activates, **Then**
   highlighting works, and a single notification explains the download
   couldn't complete — distinct from the unsupported-platform case.
3. **Given** a download completes but its SHA-256 doesn't match the
   published checksum, **When** verification runs, **Then** the binary is
   discarded, never used to start the language server, and the same
   download-failure notification path applies as a network failure would.
4. **Given** any of the above failure notifications has already been shown
   once, **When** the extension activates again with the same ongoing
   cause, **Then** the notification does not repeat.

---

### User Story 4 - A newer release is offered, never silently installed (Priority: P2)

Time passes and a newer version of Drut is published. A user whose binary
came from this feature's own download (not PATH) eventually sees a routine,
dismissible notice that an update is available, with the choice to update
now or later — never a binary that changes out from under them without
their knowledge.

**Why this priority**: Named explicitly per the owner's own instruction that
this decision not be left implicit. Ranked P2, not P1, because the "batteries
included" promise (User Story 1) already delivers real value the moment a
user first installs — staying current afterward matters, but not as much as
working at all on day one.

**Independent Test**: With an extension-storage-installed binary older than
the latest available release, trigger the (throttled) background check and
confirm a single dismissible notification appears offering to update, that
declining it doesn't block normal operation, and that accepting it results
in the newer binary running without any further manual step.

**Acceptance Scenarios**:

1. **Given** the currently-running binary came from extension storage (not
   PATH) and is older than the latest GitHub Release, **When** the
   throttled background check runs, **Then** a single, dismissible
   notification offers to update, without blocking or delaying anything
   already running.
2. **Given** the user declines ("Later"), **When** the extension activates
   again with no newer release published since, **Then** the same version
   is not re-offered.
3. **Given** the user declines, and **then** an even newer release is later
   published, **When** the next throttled check runs, **Then** the newer
   version gets its own fresh, one-time offer — declining once is not
   permanent silence.
4. **Given** the user accepts ("Update"), **When** the update completes,
   **Then** the running language server uses the new binary with no further
   manual action (e.g. no requirement to reinstall the extension or
   manually reload unless the mechanism used genuinely requires it).
5. **Given** the currently-running binary came from PATH, **When** the
   background check would otherwise run, **Then** it does not run at all —
   this feature never second-guesses a PATH install's version either.

---

### Edge Cases

- **GitHub API rate-limiting**: the unauthenticated public API allows 60
  requests/hour per IP. The throttled (at most once per 24h) background
  check keeps steady-state usage far under this; the one unavoidable
  first-ever-activation check is a single request.
- **Interrupted download or extraction** (network drops mid-transfer, VS
  Code closes mid-install): must never leave a partial file mistaken for a
  valid, ready-to-use binary on a later activation — the binary is only
  considered "installed" once fully downloaded, checksum-verified, and
  extracted.
- **Stale/mismatched stored binary** (e.g. extension storage somehow
  carries a binary for a different platform/architecture than the current
  machine): treated as absent, falls through to a fresh download — never
  used as-is.
- **No release published yet**: the GitHub API's "latest release" lookup
  simply finds nothing; treated identically to any other download failure
  (graceful degradation, one notification).
- **User manually deletes extension storage**: next activation behaves
  exactly like a first-ever activation — re-downloads from scratch.
- **D2's own platform matrix expands later** (e.g. Linux arm64 gets a
  binary): this feature's platform/architecture-to-target-triple mapping is
  the only place that would need a new entry — no other design change.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: On activation, the extension MUST attempt binary resolution in
  this exact priority order, stopping at the first success: (a) PATH, (b)
  a previously-installed copy in the extension's own persistent storage,
  (c) download-verify-install from the latest GitHub Release.
- **FR-002**: PATH resolution MUST be a genuine pre-flight check completed
  before deciding whether to fall back further (not a bare attempt-and-catch
  during server startup), and MUST remain behaviorally identical to today's
  PATH-based resolution whenever it succeeds — never overridden by a stored
  or freshly-downloaded copy when a PATH binary is present (User Story 2).
- **FR-003**: When checking extension storage, the extension MUST confirm
  the stored binary corresponds to the current platform/architecture before
  trusting it; a mismatch or absence MUST be treated as "not present,"
  falling through to the download step.
- **FR-004**: When downloading, the extension MUST derive the current
  platform/architecture and map it to exactly one of the target triples D2
  actually publishes; any other combination MUST be treated as unsupported
  and MUST NOT attempt a download (User Story 3, Scenario 1).
- **FR-005**: The extension MUST fetch the latest release's real asset list
  from the public GitHub REST API and locate the matching binary and its
  `.sha256` sidecar by exact name match against that list — never by
  constructing an assumed filename independent of what the API actually
  returns.
- **FR-006**: The extension MUST verify the downloaded binary's SHA-256
  digest against the sidecar's published value before the binary is trusted
  for any use; a mismatch MUST be treated identically to a failed download
  — discarded, never used (User Story 3, Scenario 3).
- **FR-007**: The extension MUST decompress the verified binary using the
  mechanism matching its own platform's actual archive format, without
  introducing a new third-party package dependency for either platform's
  format.
- **FR-008**: On macOS/Linux, the extension MUST mark the extracted binary
  executable before use.
- **FR-009**: The extension MUST install the binary using a pattern that
  makes an interrupted download or extraction impossible to mistake for a
  valid, ready-to-use binary on a later activation.
- **FR-010**: The extension MUST record which release version is installed
  in its own storage, without needing to execute the binary to learn it.
- **FR-011**: If every resolution step fails or is inapplicable, the
  extension MUST degrade to exactly today's highlighting-only behavior —
  the language server simply does not start; static highlighting is
  entirely unaffected (User Story 3).
- **FR-012**: Every distinct failure kind (unsupported platform/
  architecture; download/verification failure; a resolved binary that still
  fails to launch) MUST produce its own single, non-repeating, kind-scoped
  notification, using the existing notification mechanism/pattern already
  in place for "missing binary"/crash cases — no failure kind goes
  unreported, and none repeats on every subsequent activation for the same
  ongoing cause (User Story 3, Scenario 4).
- **FR-013**: The extension MUST periodically check, in the background,
  whether a newer GitHub Release exists than the one currently installed in
  its own storage — only when the active binary came from that storage,
  never when it came from PATH (User Story 4, Scenario 5).
- **FR-014**: This background check MUST be throttled to at most once per a
  bounded time window, and MUST NOT delay or block the language server from
  starting with whatever binary was already resolved.
- **FR-015**: When a newer version is found, the extension MUST present an
  explicit, dismissible choice to update or defer — never silently
  replacing the in-use binary without the user's awareness and action.
- **FR-016**: Declining an update offer MUST NOT suppress a future offer for
  a subsequently newer version — only that same already-declined version
  (User Story 4, Scenarios 2-3).
- **FR-017**: Accepting an update MUST re-run the same download/verify/
  extract/install steps for the newer release and result in the running
  language server using the new binary with no further manual step from the
  user beyond that acceptance.
- **FR-018**: The GitHub repository identity used for API/download URLs
  MUST be derived from the extension's own published package metadata, not
  maintained as a second, independently-editable copy.

### Key Entities

- **Resolution source**: which of the three priority-ordered mechanisms
  (PATH / extension storage / freshly downloaded) actually supplied the
  binary for a given activation.
- **Stored binary record**: the installed binary's location, the
  platform/architecture it was built for, and its release version — kept in
  the extension's own persistent storage, not tied to any single workspace.
- **Release asset match**: the specific binary asset and checksum sidecar
  identified, by exact name, from a real GitHub Release's asset list for a
  given platform/architecture.
- **Update-check state**: when the background update check last ran, and
  which version (if any) the user has already been offered and declined.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user on a supported platform with no prior `drut` install
  anywhere gets working diagnostics/hover/completion/formatting after
  installing the extension and opening a file — zero manual install steps.
- **SC-002**: A user with `drut` already on PATH sees identical behavior to
  before this feature existed — their own binary is used, never replaced or
  second-guessed.
- **SC-003**: A user on an unsupported platform, or without network access,
  still gets full static highlighting and exactly one clear, correctly-
  attributed notification — never a crash, never repeated nagging for the
  same ongoing cause.
- **SC-004**: A corrupted or tampered download is never used to start the
  language server — confirmed via checksum verification before any use, on
  every single download.
- **SC-005**: No user ever has their running language server binary change
  without an explicit action they took.
- **SC-006**: The background update check makes no more than one network
  request per 24-hour window per installation, and adds no perceptible
  delay to extension activation.

## Assumptions

- The public, unauthenticated GitHub REST API rate limit (60 requests/hour
  per IP) is sufficient given the throttled, at-most-once-per-24h check
  design — no authentication token is introduced by this feature.
- Only the four platform/architecture combinations D2 currently publishes
  (Windows x64, macOS x64, macOS arm64, Linux x64) are supported. D2's
  matrix expanding later (e.g. Linux arm64) is expected to extend this
  feature's own platform-to-target-triple mapping with a new entry, not
  require a redesign — but until D2 does, anything else stays unsupported.
- No new settings/configuration surface is introduced (no way to point at a
  custom binary path, disable auto-download, or change the update-check
  interval) — out of scope for this cycle, consistent with this file's
  existing no-settings-surface design (`resolveDrutCommand`'s own prior
  doc comment: "the server and extension behave the same way across every
  workspace").
- The background update-check throttle window is 24 hours — a deliberate,
  fixed default for this cycle, not user-configurable.
- This feature is entirely extension-side (TypeScript, `editors/vscode/`).
  No corresponding change is needed in `drut-cli`, `drut-lsp`,
  `drut-config`, or any other Rust crate — it consumes what D2 (already
  shipped) publishes as-is.
- The version-staleness handling described in User Story 4 was an explicit,
  owner-confirmed design decision made before this spec was written (not
  left as an open question during specification) — a throttled background
  check, scoped only to extension-storage-managed installs, non-blocking,
  resolved via an explicit dismissible per-version offer rather than a
  silent automatic replacement — chosen specifically because it's the
  option most consistent with this file's own existing design language
  (the crash-recovery policy's one-restart-then-stop-and-notify behavior;
  `ensureFormatOnSaveEnabled`/`ensureVariableColorCustomization`'s one-time,
  visibly-inspectable side effects) — never a pattern of invisible,
  repeating automatic action anywhere else in this file.
- **The background update check's own failure is silent by design** —
  distinct from FR-012's three named failure kinds, which all belong to the
  *initial* resolution flow (User Stories 1/3). If the periodic check
  itself can't reach GitHub, nothing about the user's current, already-
  working setup changes — the existing binary (from PATH or storage) keeps
  running exactly as it was; there is no degraded state to explain, unlike
  an initial-resolution failure where the user genuinely loses a
  capability they'd otherwise have. A notification here would be reporting
  a non-event. Recorded explicitly here (found missing during this
  feature's own `/speckit-analyze` pass, added before implementation
  began) rather than left to live only as an inline implementation-detail
  comment.
