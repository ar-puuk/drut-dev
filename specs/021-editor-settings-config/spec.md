# Feature Specification: Editor-Settings Exposure for `[format]` Config Fields

**Feature Branch**: `021-editor-settings-config`

**Created**: 2026-08-17

**Status**: Draft

**Input**: User description: "all config toml options should also be available as vs code
settings format for extension users" — scoped through direct conversation to all 10 current
`drut-config::FormatConfig` fields (`casing`, `control_words_casing`, `pair_keywords_casing`,
`data_references_casing`, `top_level_indent`, `indent_width`, `operator_spacing`, `blank_lines`,
`top_level_blank_line_cap`, `nested_blank_line_cap`), built as a genuine new `drut-lsp` server
capability on the standard LSP `workspace/configuration`/`workspace/didChangeConfiguration`
mechanism (not something VS Code-proprietary), with `drut.toml` winning over a client setting
whenever both set the same field. Full design history in `ROADMAP.md` item 15.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A developer sets a personal formatting preference without a project `drut.toml` (Priority: P1)

A developer works across several Voyager script projects, some of which have a committed
`drut.toml` and some of which don't. For the ones that don't, they want their own personal
formatting preference (say, uppercase control words) applied automatically in every project,
without creating a `drut.toml` file themselves — especially since adding one would affect every
other person who opens that project too, which isn't what they want for a personal preference.

**Why this priority**: This is the direct scenario motivating the request — today, a developer
with no `drut.toml` gets only the built-in defaults (always `preserve`/untouched), with no way
to express a personal default at all short of remembering CLI flags on every manual invocation.

**Independent Test**: With a client (editor) setting configured for one field and no `drut.toml`
anywhere in the opened project, format a document and confirm that field's client-configured
value is applied.

**Acceptance Scenarios**:

1. **Given** a client setting configuring `control_words_casing` to `upper` and no `drut.toml`
   discoverable for the open document, **When** the document is formatted, **Then** control words
   are uppercased, matching the client setting.
2. **Given** the same client setting, **When** the client setting is changed to a different value
   while the document remains open, **Then** the next format request reflects the new value —
   no editor restart or document close/reopen required.
3. **Given** no client setting configured for a field and no `drut.toml` either, **When** a
   document is formatted, **Then** that field's built-in default applies, unchanged from every
   prior release — this feature introduces no new default behavior for a project with nothing
   configured anywhere.

---

### User Story 2 - A project's `drut.toml` continues to govern team-shared formatting, unaffected by any individual's client settings (Priority: P1)

A project has a committed `drut.toml` establishing the team's agreed formatting conventions. A
team member has their own personal client settings configured differently (perhaps left over
from a different project, or just a personal habit). Opening this project's files must still
format according to the team's `drut.toml`, not that individual's personal client settings —
otherwise the entire reason the project committed a `drut.toml` (consistent, shared formatting
regardless of who's editing) breaks down the moment editor-level settings exist.

**Why this priority**: Equal priority to User Story 1 — without this guarantee, shipping client
settings support would be actively harmful to every project that already relies on `drut.toml`
for team consistency, not just a neutral additional convenience.

**Independent Test**: With a `drut.toml` setting a field to one value and a client setting
configuring the same field to a different value, format a document and confirm the `drut.toml`
value wins.

**Acceptance Scenarios**:

1. **Given** a `drut.toml` setting `indent_width` to `2` and a client setting configuring
   `indent_width` to `8`, **When** a document governed by that `drut.toml` is formatted, **Then**
   indentation uses `2` spaces per level — the `drut.toml` value, not the client setting.
2. **Given** the same `drut.toml`, **When** a *different* field (one `drut.toml` does not set at
   all) has a client setting configured, **Then** that field uses the client-configured value —
   `drut.toml` only wins for the specific fields it actually sets, not as an all-or-nothing
   override of every field at once.

---

### Edge Cases

- What happens when the connected editor doesn't support the LSP `workspace/configuration`
  capability at all? No request for client settings is ever sent to that editor in the first
  place — not sent-but-ignored — the same "capability advertised or the mechanism is never
  attempted" pattern this project's other optional-client-capability features (e.g. the
  `drut.toml` file-change watcher) already follow. Formatting behavior is unaffected, falling
  back to exactly today's behavior (`drut.toml` then built-in default).
- What happens when the requested settings section is entirely absent from the editor's response
  (as opposed to present with one malformed field)? Treated identically to every field within it
  being individually absent — no special case, since an absent section and an all-fields-absent
  section resolve to the same outcome either way.
- What happens with more than one document open when a client setting changes? Every open
  document's *next* format request reflects the new value — the cached value is shared across
  the whole session, not tracked per document, so this needs no separate handling.
- What happens when a client setting has an invalid value (e.g. an unrecognized casing string)?
  Degrades exactly like an invalid `drut.toml` value already does — falls back to the next
  precedence tier with a non-blocking notice, never a hard failure.
- What happens to a CLI invocation or an MCP `format` tool call — do they gain client-setting
  awareness too? No — this capability is specific to the LSP surface, since "client setting" only
  has meaning in the context of a connected editor; CLI/MCP behavior is completely unchanged.
- What happens when a project has both a `drut.toml` and client settings, and `drut.toml` is
  later deleted while the document stays open? The next format request falls through to client
  settings for every field, the same live-update behavior `drut.toml` file-watching already
  provides for the drut.toml-present case.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST allow every one of the 10 current `[format]` configuration fields
  to be set via a client (editor) setting, in addition to `drut.toml`.
- **FR-002**: The system MUST use the standard LSP `workspace/configuration` mechanism to read
  client settings, and MUST refresh its view of them in response to a
  `workspace/didChangeConfiguration` notification — not an editor-proprietary side channel.
- **FR-003**: For each of the 10 fields, resolution precedence MUST be: explicit CLI flag/MCP
  parameter (where applicable) > `drut.toml` > client setting > built-in default. A client
  setting MUST NOT override a value `drut.toml` actually sets for that field.
- **FR-004**: When the connected client does not support `workspace/configuration`, the system
  MUST behave exactly as it does today (no regression) — `drut.toml` then built-in default, with
  no error or degraded experience from the missing capability.
- **FR-005**: An invalid client-setting value for a field MUST degrade to the next precedence
  tier with a non-blocking notice, matching how an invalid `drut.toml` value already degrades —
  never a hard failure.
- **FR-006**: A change to a relevant client setting while a document is open MUST be reflected on
  the next format request against that document, without requiring the document to be closed and
  reopened.
- **FR-007**: This capability MUST NOT change CLI (`drut check`/`drut format`) or MCP `format`
  tool behavior in any way — it is reachable only through the LSP surface.
- **FR-008**: The VS Code extension MUST declare all 10 fields as `contributes.configuration`
  settings in its `package.json`, so they appear in VS Code's standard Settings UI, discoverable
  the same way every other VS Code extension's settings are.

### Key Entities

- **Client setting**: An editor-level (as opposed to project-file-level) value for one `[format]`
  field, read via the standard LSP `workspace/configuration` mechanism — a new, fourth
  precedence tier sitting between `drut.toml` and the built-in default.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can set any of the 10 fields via a client setting and see it take
  effect on a document with no `drut.toml`, verified for at least one field per config category
  (casing, indentation, operator spacing, blank lines).
- **SC-002**: A `drut.toml`-governed project's formatting output is unaffected by any client
  setting for a field `drut.toml` itself sets, verified directly, not inferred.
- **SC-003**: A project with neither a `drut.toml` nor any client settings configured produces
  byte-identical formatting output to before this feature existed, verified across the full real
  fixture corpus — this is a purely additive capability.
- **SC-004**: A client setting change is reflected on the very next format request against an
  already-open document, with no document close/reopen and no editor restart.
- **SC-005**: A client that doesn't support `workspace/configuration` sees no behavior change and
  no error from this feature's presence.
- **SC-006**: All 10 fields are visible and settable through VS Code's built-in Settings UI (not
  only by hand-editing `settings.json`), verified by inspecting the extension's declared
  configuration schema.

## Assumptions

- Scope is exactly the 10 fields that exist in `drut-config::FormatConfig` today — any future
  `[format]` field added by a later feature is expected to follow this same pattern, but isn't
  retroactively in scope for this spec.
- VS Code settings naming (e.g. `drut.format.controlWordsCasing`) and the exact
  `workspace/configuration` section string(s) requested are planning-phase decisions, not fixed
  here — the binding requirement is that all 10 fields are reachable through client settings,
  not the exact identifier scheme.
- Neither `drut-cli` nor `drut-mcp` gain any new capability from this feature (FR-007) — it is
  scoped entirely to `drut-lsp` and the VS Code extension's `package.json`.
- This feature does not change `drut.toml`'s own discovery/precedence relative to CLI flags/MCP
  params — that hierarchy is unchanged; only a new, lower-precedence tier is inserted beneath it.
