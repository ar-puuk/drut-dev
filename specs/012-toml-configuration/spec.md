# Feature Specification: TOML-Based Configuration

**Feature Branch**: `012-toml-configuration`

**Created**: 2026-08-12

**Status**: Draft

**Input**: User description: "TOML-based configuration for drut (ROADMAP.md pre-publish
item 3). Lets users set project-level defaults for drut's configurable behavior via a
`drut.toml` file, instead of only being reachable through CLI flags (and, for casing,
an MCP tool parameter) — today drut-lsp has zero configuration surface at all, meaning
every VS Code user is stuck on hardcoded defaults regardless of what a teammate might
set via the CLI."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A team sets one shared project convention everyone gets automatically (Priority: P1)

A team of analysts works on the same Cube Voyager project in a shared repository. They
want to agree on one formatting convention (e.g. lowercase keyword casing) once, in a
file that lives with the project, so that every team member — whether they run the
command-line tool, use the VS Code extension, or work through any MCP-integrated
tooling — gets that convention automatically, without each person individually
configuring their own environment.

**Why this priority**: This is the entire point of the feature. Today, a project-level
convention can only be enforced by every individual CLI user remembering to pass the
same flags every time — and editor users can't reach it at all. Closing that gap is
the whole value; every other story is a refinement of it.

**Independent Test**: Add a `drut.toml` file to a project directory setting a
non-default casing convention. Run `drut format` on a file in that directory, open the
same file in an LSP-capable editor and use its Format Document action, and invoke the
MCP `format` tool against the same file. Confirm all three produce identically-cased
output, matching the file's setting, with no other configuration supplied by the user
in any of the three cases.

**Acceptance Scenarios**:

1. **Given** a project directory containing a `drut.toml` that sets casing to
   lowercase, **When** a user formats a file in that directory via the CLI with no
   `--casing` flag, **Then** the output uses lowercase casing, matching the file.
2. **Given** the same project directory and `drut.toml`, **When** a user formats the
   same file through an LSP-capable editor's Format Document action, **Then** the
   result is identical to the CLI's output — the editor user reached the same
   behavior with zero editor-specific configuration of their own.
3. **Given** the same project directory and `drut.toml`, **When** the same file is
   formatted through the MCP `format` tool with no casing parameter supplied,
   **Then** the result matches the other two surfaces.
4. **Given** a file that sits in a subdirectory nested below the directory containing
   `drut.toml`, **When** that file is formatted on any surface, **Then** it still
   picks up the same project-level configuration (configuration applies to the whole
   subtree beneath it, not only files in the exact same directory).

---

### User Story 2 - A user overrides the project default for one run without editing the shared file (Priority: P2)

An analyst usually wants the team's shared convention, but occasionally needs a
one-off result that differs from it — for example, checking what the file would look
like under the opposite casing convention — without changing the file everyone else
relies on.

**Why this priority**: This is what makes a shared project default safe to adopt in
the first place — without a working override, any team-wide default would force
every exception into either a temporary file edit (risking an accidental commit) or
abandoning the shared file entirely. It builds directly on User Story 1's mechanism.

**Independent Test**: With a `drut.toml` in place setting one casing convention, run
`drut format` with an explicit `--casing` flag requesting the opposite convention.
Confirm the explicit flag's value wins, and that the shared file's setting is
unaffected for the next invocation that doesn't pass the flag.

**Acceptance Scenarios**:

1. **Given** a `drut.toml` setting casing to lowercase, **When** a user runs
   `drut format` with `--casing upper` on a file governed by that configuration,
   **Then** the output is uppercase — the explicit flag overrides the file.
2. **Given** the same setup, **When** the same file is formatted again with no flag
   supplied, **Then** the output reverts to lowercase — the override was scoped to
   that one invocation only, not a persistent change.
3. **Given** a `drut.toml` that only sets `casing` and leaves top-level-indent
   unset, **When** a file governed by it is formatted with no indent-related
   override supplied, **Then** top-level indentation behaves exactly as it does
   today with no configuration file present at all (an unset field in an existing
   file falls back to the built-in default, the same as no file existing).

---

### User Story 3 - A user bypasses project configuration entirely for a single run (Priority: P3)

A CI pipeline, or an analyst doing a one-off sanity check, needs to run drut against
its plain, unconfigured default behavior — ignoring whatever `drut.toml` files might
exist nearby — without deleting or renaming any file to achieve that.

**Why this priority**: A narrower, less-frequently-needed variant of User Story 2's
override need — useful for CI reproducibility and debugging "is this drut.toml the
cause of what I'm seeing," but not required for the core value of the feature to
land.

**Independent Test**: With a `drut.toml` in place, run the CLI with an explicit
bypass option. Confirm the result matches drut's built-in defaults exactly, ignoring
every setting the nearby file would otherwise have applied.

**Acceptance Scenarios**:

1. **Given** a `drut.toml` setting non-default casing and top-level-indent values,
   **When** a user runs `drut format` with the bypass option enabled, **Then** the
   output matches drut's built-in defaults for both settings, as if no `drut.toml`
   existed anywhere.

---

### Edge Cases

- **No configuration file exists anywhere**: every surface behaves exactly as it
  does today, with zero change in output for any existing project or script. This is
  an explicit backward-compatibility requirement, not an incidental side effect.
- **A configuration file exists but is malformed** (invalid syntax, an unrecognized
  setting name, or an invalid value for a recognized setting): this never blocks the
  requested operation, and it is never silently ignored — both hold at once, they are
  not in tension. The operation being requested (formatting, checking, or any other
  configuration-aware action) always completes, using the built-in default for
  whatever couldn't be resolved, while the affected surface simultaneously reports
  the specific problem visibly, in a form appropriate to that surface. Granularity
  matters here: a single bad setting (an unrecognized name, or an invalid value for a
  recognized one) falls back to the default for *only that setting* — every other
  valid setting in the same file still applies; only a file that fails to parse at
  all (invalid syntax) falls back to the default for every setting it would have
  provided. Refusing to run at all over a configuration-file problem — as opposed to
  a problem with the script actually being processed — was considered and rejected:
  it would make a config-file typo capable of blocking a user's entire workflow over
  a cosmetic formatting setting, the same category of trust-eroding, workflow-
  breaking failure this project's own false-negatives-over-false-positives stance
  (constitution Principle IV) exists to prevent — a hard fail here would be choosing
  the *worse* failure mode to avoid the lesser one. Surfacing loudly without
  blocking matches this project's own established precedent for exactly this shape
  of problem (`006`'s unmatched-process diagnostic, `010`'s unclosed-marker notice —
  both "the operation completes, and a visible, non-blocking notice appears").
- **Multiple configuration files at different levels of the same directory tree**:
  the one closest to the file being processed wins; a more specific, nested
  configuration file is not merged with a more general one further up the tree — it
  replaces it entirely for that file.
- **A file with no real, on-disk location** (e.g. an editor buffer that has never
  been saved): no configuration-file lookup is attempted for it; it falls back to
  built-in defaults (or, if the editor has an open project/workspace with a
  configuration file at that workspace's own root, that root-level file), the same
  as if it were governed by a project with no configuration file for that specific
  case.
- **A configuration file sets only some of the available settings**: settings it
  doesn't mention fall back to the built-in default individually — a partially-filled
  file behaves as "no file" for every field it omits, not as an error and not as
  "unset means apply every other field's default from a different source."

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST support a project configuration file that lets users
  set default values for drut's configurable formatting behavior (today: keyword/
  control-word casing and top-level statement indentation) without needing to supply
  them on every individual command or request.
- **FR-002**: The configuration schema MUST group related settings into named
  sections (starting with one covering formatting behavior), not a single flat list
  of settings, so a future group of settings can be added without restructuring or
  breaking the schema of settings that already exist.
- **FR-003**: Configuration-file discovery MUST search from the location of the file
  being processed upward through its containing directories, applying the nearest
  configuration file found — not only a single fixed location — so a file in a more
  specific subdirectory can be governed by a closer configuration file if one exists,
  while every file without a closer one still inherits from a more general one
  further up the tree.
- **FR-004**: Discovery MUST stop searching upward at the first configuration file
  found, at a version-control repository boundary, or at the top of the filesystem,
  whichever comes first — a configuration file MUST NOT be picked up from an
  unrelated project outside the current one's boundary.
- **FR-005**: Configuration resolution MUST behave identically regardless of which
  surface (command-line tool, editor/language-server integration, or MCP-integrated
  tooling) processes a given file — the same file, in the same location, MUST
  resolve to the same effective settings on every surface.
- **FR-006**: For every configurable setting, an explicit value supplied for a
  single command or request MUST take priority over the value found in the resolved
  configuration file, which in turn MUST take priority over drut's built-in default
  — this MUST hold independently per setting, so overriding one setting for one run
  does not affect any other setting's own resolution.
- **FR-007**: A project or directory tree containing no configuration file anywhere
  MUST behave identically to drut's behavior before this feature existed, on every
  surface — this feature MUST NOT change any default behavior for a user who never
  creates a configuration file.
- **FR-008**: Users MUST be able to explicitly bypass configuration-file discovery
  entirely for a single command or request, running on built-in defaults (plus any
  other explicitly-supplied overrides for that same command/request) even when a
  configuration file would otherwise apply.
- **FR-009**: An editor user MUST be able to obtain the same casing/indentation
  behavior a project's configuration file specifies through their editor's own
  formatting action, without needing to set any editor-specific configuration of
  their own — closing the current gap where editor/language-server users cannot
  reach non-default behavior through any mechanism at all.
- **FR-010**: MCP-integrated formatting MUST support the same set of settings as the
  command-line tool (today: both casing and top-level indentation) — closing the
  current asymmetry where only casing is reachable through MCP.
- **FR-011**: A configuration file that is syntactically invalid, contains an
  unrecognized setting name, or contains an invalid value for a recognized setting
  MUST NOT block the requested operation, and MUST NOT be silently ignored — both
  requirements hold simultaneously, not as alternatives. The system MUST complete
  the requested operation using the built-in default for whatever specific
  setting(s) could not be resolved (or, for a file that fails to parse entirely,
  every setting it would have provided), while the surface that encountered the
  problem MUST also report it to the user, specifically and visibly, in a form
  appropriate to that surface. Refusing to run rather than falling back and warning
  was considered and rejected — see Edge Cases for the full reasoning (constitution
  Principle IV: a configuration-file problem must not be allowed to block a user's
  actual work over a cosmetic formatting setting).
- **FR-012**: The full configuration schema, the discovery rule, and the precedence
  order between an explicit per-command value, a configuration file's value, and the
  built-in default MUST be documented clearly enough that a user can predict which
  settings apply to a given file without needing to read drut's source code.

### Key Entities

- **Configuration File**: A project-level file containing named sections of
  settings. Discovered per file processed, by searching upward from that file's own
  location. Not a new persistent data store — read fresh at resolution time, not
  cached indefinitely across unrelated runs.
- **Resolved Configuration**: The final, effective value for each configurable
  setting for one specific file/request, produced by applying the explicit-override →
  configuration-file → built-in-default precedence independently per setting.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A team can define one shared project configuration and every person
  using the command-line tool, the editor extension, and MCP-integrated tooling
  against that project observes identical formatting behavior, without any of them
  individually configuring their own environment.
- **SC-002**: Editor users gain access to formatting behavior that was previously
  reachable only from the command line (casing normalization, top-level indent
  normalization), through their editor's own standard formatting action, with zero
  additional editor-specific configuration required.
- **SC-003**: Every project or script that has no configuration file continues to
  format identically to how it did before this feature shipped — zero behavior
  change for the common, unconfigured case.
- **SC-004**: A user can override a project's shared configuration for a single run
  without editing the shared file, and can bypass project configuration entirely
  when needed (e.g. for CI reproducibility), in both cases without any risk of
  accidentally changing what the rest of the team sees.
- **SC-005**: A misconfigured configuration file never prevents a user from
  completing the operation they requested — it produces an immediately visible
  indication of the specific problem, on whichever surface encountered it, while the
  operation itself still completes using safe fallback values, rather than either a
  silent fallback that leaves a user unsure why their expected settings aren't
  taking effect, or a hard failure that blocks their work entirely over an
  unrelated configuration-file problem.

## Assumptions

- Only the two settings that exist today (casing, top-level indentation) are in
  scope for the initial schema. The section-based structure (FR-002) is chosen so a
  future settings group (e.g. lint-rule severity, if that's ever built) can be added
  without breaking the schema of settings that already exist — but no such future
  group is designed or implemented as part of this feature.
- No cross-file inheritance (one configuration file explicitly extending another) is
  included in this pass — investigated against a comparable tool's own convention
  and deferred, since nothing about current real-world project usage in this
  ecosystem (single-project directories, not multi-package monorepos) demonstrates a
  need for it yet.
- There is no existing, widely-used project-manifest file in the Cube Voyager
  ecosystem analogous to what other ecosystems use to host a tool's configuration
  inline — confirmed by checking both the vendor-documentation archive and a real,
  representative project's actual root directory layout. The configuration file is
  therefore a standalone, dedicated file, not a section nested inside something else.
- Discovery stops at a version-control repository boundary as a deliberate default,
  borrowed from well-established convention in comparable developer tooling (not
  from anything specific to Cube Voyager, which has no such convention of its own)
  — this prevents a configuration file from an unrelated parent project being picked
  up unintentionally.
- Binary path/location configuration (where a tool or editor extension finds the
  underlying executable) is a distinct category of setting, tied to not-yet-built
  automatic-install/update work, and is explicitly out of scope here.
- A malformed configuration file warns and falls back rather than blocking the
  requested operation (FR-011, SC-005) — a deliberate choice, not a gap left for
  planning to decide. "Never silently ignored" and "never blocks the operation" were
  weighed against each other directly: hard-failing on a configuration-file problem
  (as opposed to a problem with the script actually being processed) would let a
  typo in a cosmetic formatting setting block a user's entire workflow, the exact
  category of trust-eroding failure constitution Principle IV (false negatives over
  false positives) exists to prevent. Falling back per-setting rather than
  per-file — a single bad setting doesn't invalidate the rest of an otherwise-valid
  file — follows from the same reasoning at finer granularity.
- A configuration-file lookup for a file with no real on-disk location (e.g. an
  unsaved editor buffer) is skipped entirely rather than attempted against some
  inferred location — an editor's own open workspace root, if one exists and has a
  configuration file, is used instead; otherwise built-in defaults apply, the same
  as for any file in a project with no configuration file.
