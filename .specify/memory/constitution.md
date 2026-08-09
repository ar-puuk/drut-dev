<!--
Sync Impact Report
===================
Version change: [TEMPLATE] → 1.0.0 (initial ratification)
Modified principles: n/a (first concrete adoption; all 8 principles are new)
Added sections:
  - Core Principles I–VIII (Single Source of Truth; No Verbatim Vendor Documentation
    Redistribution; Formatter Idempotence & Behavior Preservation; False Negatives Over
    False Positives; Vertical Phase-Gated Delivery; LSP-Standard Mechanisms Over
    Editor-Proprietary APIs; Naming Honesty; Public/Private Boundary)
  - Technology & Architecture Constraints
  - Development Workflow & Quality Gates
  - Governance
Removed sections: none (template placeholders only)
Templates requiring follow-up:
  - .specify/templates/plan-template.md — ⚠ pending manual check that its Constitution
    Check gate references these 8 principles (not yet verified in this run; out of scope
    for this command per the Scope Guard).
  - .specify/templates/spec-template.md, tasks-template.md, checklist-template.md — ⚠ no
    direct constitution references found to update; no action taken (out of scope).
Deferred/TODO items: none — RATIFICATION_DATE set to the date of this adoption since no
  prior constitution existed.
-->

# Drut Constitution

## Core Principles

### I. Single Source of Truth
All Voyager grammar, parsing, and lint-rule logic MUST live in one Rust core crate. The
CLI, LSP server, MCP server, and formatter are thin adapters over that core. Grammar or
rule logic MUST NOT be duplicated in TypeScript or any other adapter layer.
Rationale: A single authoritative implementation is the only way to guarantee that every
surface (CLI, editor, MCP tools) agrees on what is valid Voyager script; duplicated logic
drifts and produces contradictory diagnostics across tools.

### II. No Verbatim Redistribution of Vendor Documentation
Nothing in this repository may reproduce text from Bentley/Citilabs OpenPaths or Cube
documentation (targeting Cube Voyager 6.5 grammar as the initial baseline). Keyword
lists, grammar rules, and any hover/help text MUST be written in the project's own
words, derived from — but never copied from — vendor docs. If a contribution cannot
state its source in original wording, it does not merge. The same rule applies to any
other party's prior-art extension used as a reference: port structure and behavior,
never verbatim text, keyword lists, or grammar files, and only after confirming the
right to reference that source at all.
Rationale: Protects the project legally and ethically from copyright and licensing
exposure while still allowing the tool to be technically accurate.

### III. Formatter Idempotence & Behavior Preservation
The formatter MUST be idempotent (`format(format(x)) == format(x)`) and strictly
behavior-preserving: it MUST NOT change which lines are continuations of a prior
statement, MUST NOT reorder statements, and MUST NOT alter program meaning — only
whitespace and, optionally and configurably, keyword casing. Every formatter change
MUST be verified against the fixture corpus with a golden-file diff before merge.
Rationale: A formatter that silently changes program behavior is worse than no
formatter; idempotence and behavior preservation are the minimum bar for trust.

### IV. False Negatives Over False Positives
The linter MUST prefer false negatives over false positives. An unflagged bug is
forgivable; a false positive on valid, working script erodes trust and gets the tool
uninstalled. New rule categories ship as warnings, not errors, until validated against
the fixture corpus with zero known false positives.
Rationale: Tool adoption depends on user trust; a single confident wrong flag on
correct code costs more credibility than many missed real bugs.

### V. Vertical, Independently-Usable Increments
Work ships in vertical, independently-usable increments. A new project phase MUST NOT
begin until the prior phase's fixture-corpus tests pass cleanly.
Rationale: Keeps the project always in a working, shippable state and prevents
compounding technical debt across phases.

### VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs
Wherever an LSP-standard mechanism and a VS Code-proprietary API are equivalent, the
LSP-standard mechanism MUST be preferred, so the tool works in any LSP-capable editor
and not only the VS Code family.
Rationale: Editor lock-in narrows the tool's audience and contradicts the goal of
broadly usable tooling built on open standards.

### VII. Naming Honesty
Features are named for what they actually do. A feature MUST NOT be called a "type
checker" unless it performs genuine type inference/checking — reference and
symbol-existence validation is called a "semantic checker" or "reference checker."
Cube Voyager control statements are untyped keyword=value pairs, so there is no type
system to check against; if a genuine type-like check emerges later (e.g., matrix
dimension mismatches, numeric vs. string keyword misuse), it MUST be named for that
specific check rather than reaching for "type checker" as an umbrella term.
Rationale: Overclaiming capability in a feature's name misleads users about what
guarantees the tool actually provides.

### VIII. Public/Private Boundary
The core crate, CLI, formatter, LSP server, MCP lint tools, and the extension client
are public. The converted documentation corpus and any tool whose output is derived
from that corpus's specific text (e.g., a docs-search tool) MUST live in a separate
private repository and MUST NOT be imported into the public one.
Rationale: Keeps derivative vendor-documentation content out of the public,
redistributable codebase while still allowing it to power internal or licensed tooling.

## Technology & Architecture Constraints

The system is architected as one authoritative Rust core crate (grammar, parsing, and
lint rules) with thin adapters on top: a CLI, an LSP server, an MCP server, a
formatter, and an editor extension client. The initial grammar baseline targets Cube
Voyager 6.5 (see Phase 1 open question for scope). Editor integration MUST favor
LSP-standard protocol mechanisms over editor-specific extension APIs whenever both
achieve the same result (Principle VI). Any content derived from vendor documentation
corpora is confined to a separate private repository and MUST NOT be linked into the
public core, adapters, or extension client (Principle VIII).

## Development Workflow & Quality Gates

The fixture corpus is the project's source of truth for correctness. Every formatter
change requires a golden-file diff against the fixture corpus before merge (Principle
III). Every new linter rule category ships as a warning, not an error, until it has
zero known false positives against the fixture corpus (Principle IV). A phase's work
is not considered mergeable/complete, and the next phase MUST NOT start, until the
current phase's fixture-corpus tests pass cleanly (Principle V). Any contribution that
introduces keyword lists, grammar rules, or help/hover text MUST be traceable to
original wording — reviewers MUST reject contributions that cannot state their source
in the contributor's own words (Principle II).

## Governance

This constitution supersedes conflicting team practices, code review norms, and ad hoc
decisions. All PRs and reviews MUST verify compliance with the Core Principles above;
any deviation MUST be explicitly justified in the PR description or it MUST be
rejected. Complexity or duplication introduced across the core crate and its adapters
MUST be justified against Principle I before merge.

**Amendment procedure**: Amendments are proposed via a PR that edits this file
directly, including an updated Sync Impact Report (as an HTML comment at the top of
the file) describing the version change, modified/added/removed sections, and any
deferred TODOs. Amendments take effect once merged.

**Versioning policy**: This constitution follows semantic versioning:
- MAJOR: Backward-incompatible governance changes, or removal/redefinition of an
  existing principle.
- MINOR: A new principle or section is added, or existing guidance is materially
  expanded.
- PATCH: Clarifications, wording, or typo fixes with no semantic change.

**Compliance review**: Every feature plan and PR MUST include a constitution
compliance check against the Core Principles before implementation work begins and
again before merge. Downstream templates and commands (plan, tasks, checklist, etc.)
read this constitution at runtime; they are not modified by constitution amendments
and must be checked separately for consistency when principles change materially.

**Version**: 1.0.0 | **Ratified**: 2026-08-08 | **Last Amended**: 2026-08-08
