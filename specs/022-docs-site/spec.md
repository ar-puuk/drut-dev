# Feature Specification: Published Documentation Site

**Feature Branch**: `022-docs-site`

**Created**: 2026-08-17

**Status**: Draft

**Input**: User description: "Documentation website via mdBook, hosted on GitHub Pages: a full user guide replacing scattered README/CONTRIBUTING content with one coherent, navigable site... the centerpiece this was requested for -- a complete, field-by-field drut.toml configuration reference..."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Find out what a config field does (Priority: P1)

A user has a `drut.toml` open (their own, or one inherited from a teammate) and wants
to know what a specific `[format]` field does, what values it accepts, what it
defaults to when omitted, and how it interacts with a value set elsewhere (an editor
setting, a CLI flag). Today this means reading prose scattered across
`CONTRIBUTING.md`'s "Configuration" section (which only documents 2 of the 10 real
fields) or grepping source/spec files that were never meant for this. This is the
specific pain point that prompted this feature.

**Why this priority**: This is the concrete, named frustration ("even I struggle to
find what the options are for each toml item") — the feature exists because of this
gap specifically, everything else is built around it.

**Independent Test**: Can be fully tested by opening the published site and, for
every one of the 10 `[format]` fields, locating a dedicated entry stating its name,
accepted values, default, effect, a short example, and where it sits in the
precedence chain — without reading any source code or spec-kit artifact.

**Acceptance Scenarios**:

1. **Given** the published site, **When** a user looks for `blank_lines`, **Then**
   they find its accepted values (`preserve`/`auto`), its default (`preserve`), a
   plain-language description of what `auto` actually does, and a short example.
2. **Given** the published site, **When** a user wants to know which of `drut.toml`,
   a CLI flag, or an editor setting wins for the same field, **Then** they find one
   clearly-stated precedence order that applies uniformly to every field, not
   scattered per-field caveats.
3. **Given** a field with a legacy/granular relationship (`casing` vs.
   `control_words_casing`/`pair_keywords_casing`/`data_references_casing`), **When**
   a user reads either field's entry, **Then** the relationship between them is
   explicit, not left for the reader to infer.

---

### User Story 2 - Get a new project working end to end (Priority: P2)

A user has never used Drut before. They want to install it (as a CLI, as a VS Code/
Open VSX extension, or both), point it at a real `.s`/`.block` script, and see it
actually do something (a diagnostic, a formatted file, a hover) — without piecing
that path together from a marketing-toned `README.md` and a contributor-facing
`CONTRIBUTING.md` that assumes they're about to read Rust source.

**Why this priority**: Necessary for the site to be a real substitute for the
scattered docs it replaces, not just a config reference bolted onto the existing
structure — but secondary to User Story 1, which is the specific, named complaint.

**Independent Test**: Can be fully tested by following the site's own instructions,
from "nothing installed" to "a diagnostic or formatted result seen from a real
script," using only the site — no existing familiarity with the project's internal
docs required.

**Acceptance Scenarios**:

1. **Given** a user with no Drut install, **When** they follow the site's Install
   page, **Then** they reach a working `drut` CLI or editor extension using only the
   page's own instructions.
2. **Given** a freshly-installed CLI, **When** the user follows the site's Getting
   Started walkthrough, **Then** they successfully run `check` and `format` against
   a sample script and understand what each produced.
3. **Given** a user integrating an AI coding assistant, **When** they read the MCP
   guide, **Then** they can identify all four tools (`diagnose`, `format`,
   `query_structure`, `lookup_keyword`) and what each one is for, without reading
   `drut-mcp`'s source.

---

### User Story 3 - Understand what the formatter will change before running it (Priority: P3)

A user is about to run `drut format --write` (or enable format-on-save) against a
script they didn't write, and wants to know, in plain terms, what the formatter is
and isn't allowed to touch (does it reorder statements? does it fix a broken script?
does it touch a `; FMT: OFF` region?) before trusting it against real work.

**Why this priority**: Lower-volume than the first two stories, but a real trust gap
— a formatter is inherently higher-stakes than a linter, since it rewrites files.

**Independent Test**: Can be fully tested by reading the Formatter guide alone and
correctly predicting the output of at least three representative before/after
examples covering different formatting axes (casing, indentation, operator spacing,
blank-line normalization), without running the tool.

**Acceptance Scenarios**:

1. **Given** the Formatter guide, **When** a user reads it, **Then** they can state
   in their own words that formatting never reorders statements or changes program
   meaning — only whitespace and, optionally, casing.
2. **Given** the Formatter guide, **When** a user reads the operator-spacing and
   blank-line sections, **Then** they can correctly predict the output for a
   `preserve` vs. a `fixed`/`auto` example shown on the page.

---

### Edge Cases

- A field documented on the site is renamed, removed, or gains a new accepted value
  in a future feature — the site must be updated as part of that feature's own
  change, not discovered stale later by a user (see FR-011).
- A visitor lands on the site with only a phone or a narrow window — the site must
  still be readable and navigable (built-in to the chosen doc-site tooling's default
  theme, not a bespoke requirement).
- A visitor searches for a config field by its CLI flag name (`--blank-lines`)
  rather than its `drut.toml` key (`blank_lines`) — both names should lead to the
  same entry (see FR-004).
- A maintainer edits site content but forgets to regenerate/commit the built
  `docs/` output before pushing — the committed output silently goes stale
  relative to its own source. This must be visible (a failed build-check before
  merge, per the 2026-08-17 Clarification) rather than only discoverable later by
  comparing the live site against the source by hand.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A published, browsable documentation site MUST exist at a stable public
  URL, reachable without any authentication or install step.
- **FR-002**: The site MUST include, at minimum, these sections: an introduction
  ("what is Drut and why does it exist"), an install guide covering both the CLI and
  the VS Code/Open VSX extension, a getting-started walkthrough, a CLI reference, an
  editor/LSP guide, an MCP guide, a formatter behavior guide, and a complete
  `drut.toml` configuration reference.
- **FR-003**: The configuration reference MUST document every currently-configurable
  `[format]` field with, at minimum: the field's name, its accepted values, its
  default when omitted, a plain-language description of its effect, and one short
  example.
- **FR-004**: Each configuration field's entry MUST state its equivalent CLI flag
  name and MCP tool parameter name (when either exists), so a user arriving by any
  one name can find the same entry.
- **FR-005**: The configuration reference MUST state the precedence order that
  applies across all configuration surfaces (explicit CLI flag/MCP parameter,
  `drut.toml`, editor client setting, built-in default) once, in a way every field's
  entry can point back to, rather than restating it per field.
- **FR-006**: A field with a legacy/granular relationship to another field (e.g. the
  flat `casing` setting vs. the three per-category casing settings) MUST have that
  relationship explained in both directions — reading either field's entry should
  make the relationship clear without cross-referencing source code.
- **FR-007**: The site MUST be reachable by following a link from the project's
  top-level `README.md`, which becomes the discovery path for anyone landing on the
  source repository first.
- **FR-008**: `README.md` MUST remain a short, visitor-facing pitch page (what Drut
  is, why it exists, quick install, feature list) that links out to the site as the
  documentation home, rather than duplicating the site's content inline.
- **FR-009**: `CONTRIBUTING.md` MUST retain only genuinely contributor-facing
  content (architecture, build/test commands, versioning policy, dependency
  posture, credits, the spec-kit workflow) — content that is about working on Drut's
  own source, not about using a finished build of it. User-facing material already
  covered by the new site (configuration reference, editor behavior walkthroughs)
  MUST be removed from `CONTRIBUTING.md` rather than kept as a duplicate, replaced
  with a pointer to the relevant site page.
- **FR-010** *(revised 2026-08-17 — see Assumptions)*: Publishing an update MUST be
  a direct consequence of an ordinary commit reaching `main` — the regenerated site
  output is committed alongside its source content change, and GitHub Pages serves
  that commit directly with no separate deploy action or GitHub Actions-based
  deploy step involved. An automated check MUST fail, visibly, before merge if a
  content change reaches a branch without its corresponding site-output rebuild —
  so a maintainer forgetting to regenerate the output before committing is caught
  mechanically, not discovered later as a silently stale live site.
- **FR-011**: Any future feature that adds, removes, or changes a `[format]`
  configuration field, a CLI flag, an MCP tool, or LSP-visible behavior documented
  on the site MUST update the corresponding site page as part of that feature's own
  change — the site is a maintained artifact, not a one-time snapshot.
- **FR-012**: The site MUST include a working search capability, so a user can find
  a specific field or topic by keyword without knowing which page it lives on.
- **FR-013**: A failed site build/publish MUST be visible as a failing automated
  check, not a silent no-op that leaves stale content live with no indication of the
  failure.

### Key Entities

- **Documentation site**: the published, navigable collection of guide pages
  described above, at a stable public URL.
- **Configuration field entry**: one documented `[format]` field — name, accepted
  values, default, effect, example, equivalent CLI/MCP names, and a pointer to the
  shared precedence-order explanation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 10 currently-configurable `[format]` fields have a complete,
  accurate, findable entry on the site (name, accepted values, default, effect,
  example) — zero fields undocumented or documented only in source/spec-kit
  artifacts.
- **SC-002**: A user with no prior familiarity with Drut can go from "nothing
  installed" to "sees a real diagnostic or formatted result from a sample script"
  using only the site's own instructions, without outside help.
- **SC-003**: `README.md`'s length and content stay a short pitch-and-links page (no
  net growth into a second copy of the site's content) after this feature ships.
- **SC-004**: Zero content in `CONTRIBUTING.md` post-migration duplicates content
  now on the site — each topic lives in exactly one place, with `CONTRIBUTING.md`
  pointing to the site where a topic moved.
- **SC-005** *(revised 2026-08-17)*: A content change that reaches `main` is live
  as soon as GitHub Pages serves that commit (typically under a minute — no
  separate build/deploy latency, since the served output IS the commit); a
  forgotten site-output rebuild is caught by the automated freshness check before
  merge, never discovered later as a stale live site.

## Clarifications

### Session 2026-08-17

- Q: Classic GitHub Pages ("Deploy from a branch") only serves a source branch's
  repo-root `/` or `/docs` folder — and the owner wants to avoid GitHub Actions as
  much as possible. Does any GitHub Actions workflow remain, or is the goal zero
  new workflows entirely? → A: Keep one lightweight build-check job (`mdbook
  build` + the configuration-reference coverage check, on every push/PR) — no
  deploy job, no `pages: write`/`id-token: write` permissions, no secrets.
  Publishing itself is Actions-free: the maintainer runs `mdbook build` locally
  and commits the regenerated `docs/` folder (mdBook's `book.toml` build-dir
  redirected there) as part of an ordinary content-change commit; GitHub Pages is
  configured to serve `main`'s `/docs` folder directly, with no deploy workflow
  involved. This changed FR-010/SC-005 (above) and displaced the pre-existing
  `docs/known-environment-quirks.md` — `docs/` is now reserved for the published,
  committed build output, so that file moves to a new `dev-notes/` directory
  (unrelated to anything Pages-serves-related) as part of this feature.

## Assumptions

- GitHub Pages (the repository's existing GitHub hosting) is an acceptable, free
  publishing target — no separate hosting account or paid service is expected or
  required.
- The site's initial content is derived from what's already true of the shipped
  product as of this feature (through `021-editor-settings-config`) — it documents
  the current 10-field `[format]` surface, the current CLI subcommands
  (`check`/`format`/`server`/`mcp`), and the current four MCP tools
  (`diagnose`/`format`/`query_structure`/`lookup_keyword`); it does not need to
  anticipate fields or tools that don't exist yet.
- **Superseded by the 2026-08-17 Clarification above**: publishing is a manual-but-
  mechanically-verified two-command sequence (`mdbook build`, then commit) run by
  whoever makes a content change, not a GitHub Actions deploy step — a deliberate
  departure from how `ci.yml`/`release.yml` automate everything else in this
  project, made explicitly to keep this feature's GitHub Actions footprint minimal
  per the owner's direct instruction.
- No versioned/multi-version documentation is required for this feature — the site
  documents the current `main`-branch behavior only, matching the project's current
  pre-1.0, not-yet-tagged-release state (see CONTRIBUTING.md's Versioning section).
- Visual design/branding beyond the chosen doc-site tooling's default theme is out
  of scope — a plain, readable, searchable, navigable default theme satisfies this
  feature; a custom look is not a requirement.
- No analytics, feedback widget, or comments capability is expected on the site.
