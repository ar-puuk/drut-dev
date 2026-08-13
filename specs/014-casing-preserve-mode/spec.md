# Feature Specification: Casing Gains an Explicit `Preserve` Mode

**Feature Branch**: `014-casing-preserve-mode`

**Created**: 2026-08-13

**Status**: Draft

**Input**: User description: "Give `voyager_core::CasingConvention` an explicit
third variant, `Preserve`, matching the shape `TopLevelIndentMode` already uses
(`Preserve` as `#[default]`, plus `Upper`, `Lower`) instead of today's design
where 'leave casing untouched' is represented by `FormatOptions.casing:
Option<CasingConvention>` being `None`. Pure representation/API change, not a
formatting-behavior change — `Preserve` must produce byte-identical formatter
output to today's `None` for every existing input. Required design symmetry
with `TopLevelIndentMode`/`009-top-level-indent-toggle` throughout
`voyager-core`/`drut-config`/`drut-cli`/`drut-mcp`."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Force casing untouched for one run, overriding a project's config (Priority: P1)

A project has a `drut.toml` setting `casing = "upper"` project-wide. For one
particular run — a one-off diff-minimizing format, or a script the user
deliberately wants left exactly as authored — the user explicitly requests
`drut format --casing=preserve` (or, via an MCP client, passes
`casing: "preserve"` to the `format` tool). The output leaves every
control-word/keyword casing exactly as written, ignoring the project's
`upper` setting for that one invocation only.

**Why this priority**: This is the actual new user-facing capability this
feature delivers — everything else is the plumbing that makes it possible.
Without it, there is no way to force "don't touch casing" for a single run
once a project has any `drut.toml` casing setting at all; the only way to
"turn casing off" today is to have no config and pass no flag, which stops
working the moment a project adopts a casing convention.

**Independent Test**: With a resolved `drut.toml` specifying
`casing = "upper"`, format a script containing lowercase control words two
ways — once with no flag (picks up `upper` from the config) and once with
`--casing=preserve` — and confirm the second run's casing is byte-identical
to the input while the first run's is not.

**Acceptance Scenarios**:

1. **Given** a resolved `drut.toml` with `casing = "upper"`, **When** `drut
   format --casing=preserve` is run on a script with lowercase control
   words, **Then** every control word's casing is left exactly as written,
   not forced to uppercase.
2. **Given** the same setup, **When** the MCP `format` tool is called with
   `casing: "preserve"` (source: a file path under the config's scope),
   **Then** the result matches the CLI's `--casing=preserve` output exactly.
3. **Given** no `drut.toml` and no flag at all, **When** a script is
   formatted, **Then** casing is left untouched — the same outcome as
   explicit `preserve`, since `preserve` is now the standing default, not a
   behavior change from before this feature existed.

---

### User Story 2 - Nothing about existing formatting output changes (Priority: P1)

Every script that, before this feature, formatted with casing left
untouched (today's `FormatOptions.casing == None` case) continues to
produce byte-identical output after the change, for every existing caller
that doesn't opt into the new explicit value.

**Why this priority**: Equal weight to User Story 1 — this is a type-level
refactor of how "don't touch casing" is represented, not a new formatting
policy. If any existing output changes as a side effect, the feature has a
real defect, not just an incomplete addition.

**Independent Test**: Run the full 161-file real corpus through `drut
format` before and after the change with no casing flag passed either time,
and confirm zero output differs.

**Acceptance Scenarios**:

1. **Given** any script previously formatted with no `--casing` flag and no
   governing `drut.toml` casing setting, **When** formatted again after
   this feature ships, **Then** the output is byte-identical to before.
2. **Given** `voyager_core::FormatOptions::default()` (no fields explicitly
   set), **When** used to format any script, **Then** casing is left
   untouched — the same outcome `None` produced before this feature. (This
   overlaps deliberately with User Story 3's Acceptance Scenario 1 below —
   the same fact matters to both stories: here, as proof nothing regressed;
   there, as one of three independently-confirmed integration points.)

---

### User Story 3 - The default is the same everywhere a format request can originate (Priority: P2)

A user formatting a file through the CLI, through VS Code's format-on-save
(LSP), or through an MCP-connected tool all see the identical default
(casing left untouched) when nothing explicitly overrides it — no
integration point silently disagrees because its own call site wasn't
updated when `FormatOptions.casing`'s type changed.

**Why this priority**: Named explicitly because this exact class of bug — a
setting correct at one call site but silently stale at another — has
already caused real defects in this codebase (`pair_keyword_boundaries`,
`structural_query_parity`) and was the specific reason `009` gave this same
concern its own P1 story for `top_level_indent`. Ranked P2 here rather than
P1 only because, unlike `009`, this change is representation-only — there is
no behavior for a missed call site to get *wrong* the way `008`'s
unconditional-normalize default could have; a missed call site here would
be a compile error (the field is no longer `Option`-wrapped at the
`FormatOptions` layer), not a silent divergence. Still independently
verified, not assumed from the compiler alone.

**Independent Test**: With no explicit override passed by the caller, call
`voyager_core::format` directly, then every LSP handler that formats a
document, then the MCP `format` tool — confirm all three leave casing
untouched using the same underlying default.

**Acceptance Scenarios**:

1. **Given** `voyager_core::FormatOptions::default()`, **When** used to
   format a script with lowercase control words, **Then** casing is left
   untouched.
2. **Given** a document formatted via `textDocument/formatting` or
   `textDocument/rangeFormatting` with no governing `drut.toml` casing
   setting, **When** it contains lowercase control words, **Then** casing
   is left untouched.
3. **Given** the MCP `format` tool invoked with no `casing` parameter and no
   governing `drut.toml` casing setting, **When** the target script has
   lowercase control words, **Then** casing is left untouched.

---

### Edge Cases

- What happens when a resolved `drut.toml` specifies `casing = "upper"` and
  the CLI is invoked with `--casing=preserve`? The explicit CLI value wins —
  same precedence rule (`defaults < drut.toml < explicit`) `top_level_indent`
  already follows; `preserve` is a real, deliberate override here, not
  merely "no flag given" collapsing to the config's value.
- Can `drut.toml` itself write `casing = "preserve"` literally (rather than
  just omitting the key)? Yes — mirrors `top_level_indent` already accepting
  `"preserve"`/`"normalize"` as literal TOML strings, not just `"upper"`/
  `"lower"`. Semantically a no-op versus omitting the key entirely (both
  resolve to `Preserve` absent a higher-precedence override), but it must
  parse as a recognized value, not warn as unrecognized.
- What happens to the CLI's existing "no bare `--casing`" rule
  (`002-cli-check-format` FR-015 — a value is always required when the flag
  is given at all)? Unchanged. `--casing=preserve` is a new explicit value
  alongside `upper`/`lower`, not an implicit no-value default; `--casing`
  with no `=value` remains a CLI usage error exactly as it is today.
- What happens to every existing `if let Some(casing) = options.casing { .. }`
  call site inside `voyager-core`'s formatter once the field is no longer
  `Option`-wrapped? Each becomes a `match` (or equivalent) over the
  three-variant enum with an explicit no-op `Preserve` arm — enforced by the
  compiler (an unhandled variant is a compile error), not a runtime risk,
  but the actual mechanical work this feature requires wherever the field is
  read.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `voyager_core::CasingConvention` MUST gain a third variant,
  `Preserve`, marked as the type's `#[default]` — mirroring
  `TopLevelIndentMode`'s existing shape exactly (`Preserve`/`#[default]`,
  `Upper`, `Lower`).
- **FR-002**: `voyager_core::FormatOptions.casing` MUST change from
  `Option<CasingConvention>` to a bare (non-`Option`) `CasingConvention`,
  defaulting to `Preserve` via `FormatOptions::default()` — the same
  non-optional shape `top_level_indent` already uses on this same struct.
- **FR-003**: For every input that previously formatted under
  `FormatOptions.casing == None`, `CasingConvention::Preserve` MUST produce
  byte-identical output — a pure representation change with zero formatting-
  behavior difference (User Story 2).
- **FR-004**: `drut_config::FormatConfig.casing` and
  `ExplicitFormatOverride.casing` MUST remain `Option<CasingConvention>` —
  representing "this layer (a `drut.toml` file, or an explicit CLI-flag/MCP-
  param) stated no casing preference," a distinct concept from the enum's
  own `Preserve` variant — exactly mirroring `top_level_indent`'s existing
  two-layer shape (`Option`-wrapped at the config/override layer,
  non-optional at the resolved `FormatOptions` layer) on those same two
  structs.
- **FR-005**: `drut-config`'s TOML parser MUST accept the literal string
  `"preserve"` as a valid `casing` value inside a `[format]` table (parsed
  to `Some(CasingConvention::Preserve)`), not flagged as unrecognized —
  mirroring `top_level_indent`'s existing acceptance of `"preserve"`/
  `"normalize"` as literal TOML strings.
- **FR-006**: `drut-cli`'s `--casing` flag MUST gain an explicit third
  value, `preserve`, alongside `upper`/`lower` — mirroring
  `--top-level-indent`'s existing `preserve`/`normalize` CLI-visible
  symmetry — letting a user force casing untouched for one run even when a
  resolved `drut.toml` specifies `upper` or `lower` (User Story 1). The
  existing "no bare `--casing`" rule (`002-cli-check-format` FR-015) is
  unchanged: the flag still requires an explicit value whenever given.
- **FR-007**: `drut-mcp`'s `format` tool's `casing` parameter MUST accept
  `"preserve"` as an explicit third string value alongside `"upper"`/
  `"lower"`/absent, mirroring `top_level_indent`'s existing `"preserve"`/
  `"normalize"`/absent shape on the same tool.
- **FR-008**: Every `FormatOptions` construction site in `drut-lsp` and
  `drut-mcp` that does not explicitly resolve a casing override MUST
  continue to yield `CasingConvention::Preserve` — independently confirmed
  at each site, not assumed to hold transitively from any single shared
  code path (User Story 3).
- **FR-009**: `resolve_format_options`'s existing precedence
  (`defaults < drut.toml < explicit CLI-flag/MCP-param`) MUST hold for
  `casing` exactly as it already does for `top_level_indent`, including
  when the explicit layer specifies `preserve` — an explicit `preserve`
  MUST win over a `drut.toml`-specified `upper`/`lower`, not be treated as
  "no override given" and fall through to the config value.
- **FR-010**: `voyager-core`'s existing doc comment on
  `FormatOptions.casing` (which currently states, of the old shape, "`None`
  is how off is represented, not a third variant here") MUST be corrected
  to describe the new shape, and any other prose elsewhere in the codebase
  making the same now-superseded claim MUST be updated to match — no stale
  documentation left contradicting the shipped type.
- **FR-011**: `002-cli-check-format/spec.md`'s FR-015 (which defines
  `--casing`'s shape) MUST be amended with a new dated entry documenting
  the addition of the explicit `preserve` value and the underlying
  representation change — added alongside, not replacing, the original
  FR-015 text, matching the amendment discipline `008`/`009` established
  for FR-012.
- **FR-012**: `002-cli-check-format/spec.md`'s **FR-026** MUST also be
  corrected, as its own named requirement distinct from FR-011 above —
  not merely implied by FR-010's general "any other prose" clause. FR-026
  currently contrasts itself against `--casing` with "Unlike FR-015's
  `--casing` flag, this setting has no 'off' state," a claim that becomes
  factually inaccurate the moment FR-011 ships: `--casing` then shares
  `--top-level-indent`'s exact non-optional resolved-value shape, with no
  distinct "off"/`None` state remaining at that layer either (research.md
  §3). The corrected text MUST state plainly that both settings now share
  this shape, differing only in how many named values each carries.

### Key Entities

- **Casing convention**: A three-valued setting (`Preserve`/`#[default]`,
  `Upper`, `Lower`) governing whether `format` leaves control-word/
  pair-keyword casing untouched or forces it to a given case. Carried
  through `voyager_core::FormatOptions` as a bare (non-`Option`) field,
  surfaced as an explicit CLI value, MCP string parameter, and TOML string,
  defaulting to `Preserve` everywhere it is read — the same shape
  `top_level_indent` already established.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The full 161-file real corpus produces byte-identical
  `format` output before and after this change, with no `--casing` flag
  passed either time — zero regressions from a pure representation change.
- **SC-002**: A user can force casing to stay untouched for a single run via
  `--casing=preserve` (CLI) or `casing: "preserve"` (MCP) even when a
  resolved `drut.toml` specifies `upper` or `lower`, and the output reflects
  that override, not the config.
- **SC-003**: The `Preserve` default is independently confirmed in effect at
  three distinct points — `CasingConvention`'s own `#[default]`,
  `FormatOptions::default()`, and every `drut-lsp`/`drut-mcp` call site that
  doesn't pass an explicit override — never assumed to hold transitively
  from just one of them.
- **SC-004**: CLI, LSP, and MCP surfaces all resolve an unset casing
  preference to the identical `Preserve` behavior — no surface silently
  disagreeing with the others.
- **SC-005**: `drut.toml` accepts `casing = "preserve"` as a recognized
  value (no warning emitted), verified with a real fixture, matching how
  `top_level_indent = "preserve"` is already accepted.

## Assumptions

- No golden-fixture regeneration is required (unlike `008`/`009`) — FR-003
  and SC-001 establish this is a pure representation/API change with zero
  formatting-behavior difference for any existing input, so
  `format_corpus.rs`'s existing golden fixtures need no review or
  regeneration.
- The CLI/MCP question of "should an explicit third `preserve` value be
  addable at all" is resolved here as yes, mirroring `--top-level-indent`'s
  already-shipped `preserve`/`normalize` CLI-visible symmetry exactly — a
  reasonable default given that symmetry is this feature's whole premise,
  not left open as a clarification.
- `drut-lsp` gains no new user-facing behavior — it never constructs an
  explicit casing override today (always calls `resolve_format_options`
  with `ExplicitFormatOverride::default()`) and continues not to; it is
  touched only by the type-signature change rippling through, per FR-008.
- Out of scope: `--casing=auto` (ROADMAP's resolved queued item 4, still
  deliberately deferred pending the open casing-philosophy question); any
  change to which token categories casing actually reaches (still only
  `CONTROL` and `PAIRKEYWORD` tokens, per that same ROADMAP investigation);
  any change to README/CI/publish work (separate, later items in the
  pre-publish sequence).
- This feature depends on nothing else in the pre-publish sequence and
  blocks nothing else in it — pure crate-surface work with no external
  infrastructure dependency, confirmed during the pre-publish audit that
  produced the current sequencing (ROADMAP.md item 4).
