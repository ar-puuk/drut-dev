# Implementation Plan: Published Documentation Site

**Branch**: `022-docs-site` | **Date**: 2026-08-17 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/022-docs-site/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Publish a browsable, searchable user-guide site (introduction, install, getting
started, CLI/LSP/MCP references, formatter behavior, and a complete field-by-field
`drut.toml` reference) at a stable public URL, built with mdBook and served by
GitHub Pages directly from a committed `docs/` folder on `main` — **no GitHub
Actions-based deploy step**, per the owner's direct instruction to minimize this
feature's Actions footprint (spec.md's 2026-08-17 Clarification). A single,
deploy-free CI job (`mdbook build` + two verification checks) still catches a
broken book or a stale/undocumented config field before merge. `README.md`
becomes a short pitch page linking to the site; `CONTRIBUTING.md` sheds
user-facing content it currently over-carries, keeping only genuinely
contributor-facing material.

## Technical Context

**Language/Version**: N/A for new Rust code — the site is authored content
(Markdown) plus CI plumbing. mdBook itself is a Rust-ecosystem tool but is used only
as a build-time CLI, never added as a workspace/crate dependency (doesn't touch any
`Cargo.toml`).

**Primary Dependencies**: mdBook (pinned version, installed in CI via a pinned
binary/action step — research.md §2a); GitHub's own `actions/checkout` only —
**no** `actions/configure-pages`/`upload-pages-artifact`/`deploy-pages` and no
Pages-specific workflow permissions, since deployment is Actions-free (research.md
§2). No new dependency in any published crate or the VS Code extension's
`package.json`.

**Storage**: Static Markdown source files under a new `docs-site/src/` directory
(never served directly). mdBook's build output IS committed — redirected via
`book.toml`'s `build-dir` to repo-root `docs/`, which GitHub Pages ("Deploy from a
branch" → `main` → `/docs`) serves directly (research.md §2). `docs/` requires a
`.nojekyll` marker file (empty, committed) so GitHub doesn't run Jekyll processing
over mdBook's own output.

**Testing**: `mdbook build` (fails CI on a structural problem — a `SUMMARY.md`
entry pointing at a missing file, etc.); a doc-coverage check (a small script, no
new crate) that fails CI if any `drut-config::FormatConfig` field name is missing
from the configuration-reference chapter (contracts/config-reference-entry.md);
and a freshness check (research.md §6) that fails CI if a fresh `mdbook build`
differs from the committed `docs/`, catching a forgotten rebuild-before-commit.

**Target Platform**: Static site served by GitHub Pages; any modern browser, no
server-side component, no JavaScript framework runtime beyond mdBook's own default
theme assets (search index, theme toggle).

**Project Type**: Documentation site — a new top-level directory
(`docs-site/`), not a Rust crate; no changes to the Cargo workspace's member list.

**Performance Goals**: N/A beyond "loads like any static page" — no request-rate,
latency, or throughput target applies to a static site with no backend.

**Constraints**: Zero new runtime dependency in any published artifact (crate or
extension); zero GitHub Actions-based deployment step and zero new secrets
(deployment is a committed `docs/` folder GitHub Pages serves directly — research.md
§2, per the owner's explicit instruction to minimize this feature's Actions
footprint); the one CI job that does exist (research.md §2a) needs no Pages
permission at all, since it never deploys; content MUST NOT reproduce
vendor-documentation text (Principle II) or anything derived from the private
`_archive/` mirror (Principle VIII) — every page is written in the project's own
words, describing the project's own shipped behavior.

**Scale/Scope**: ~9 chapters (introduction, install, getting started, CLI
reference, editor/LSP guide, MCP guide, formatter guide, configuration reference,
troubleshooting/FAQ), one config-reference entry per each of the 10 current
`[format]` fields, single-version (no versioned-docs infrastructure — matches the
project's current pre-1.0 state, spec.md Assumptions).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Single Source of Truth** — N/A to this feature directly (no grammar/parsing/
  lint-rule logic is added or duplicated; the site describes already-implemented
  `voyager-core`/`drut-config`/`drut-cli`/`drut-lsp`/`drut-mcp` behavior, it doesn't
  reimplement any of it). Real risk this principle's *spirit* still flags: prose
  documentation can drift from actual behavior over time the same way duplicated
  code can. Mitigated three ways: (1) the doc-coverage check (Testing, above;
  contracts/config-reference-entry.md) makes the configuration reference
  mechanically tied to `drut-config`'s real field list, not hand-maintained from
  memory; (2) the freshness check (research.md §6) catches a maintainer who edits
  content but forgets to rebuild/commit `docs/`; (3) spec.md FR-011 makes updating
  the site part of any future feature's own definition of done — made discoverable
  via a stated obligation in `CONTRIBUTING.md`'s own Workflow section (tasks.md),
  not just spec prose nobody but this feature's author reads. **Pass, with
  mitigation.**
- **II. No Verbatim Redistribution of Vendor Documentation** — Directly applicable:
  every page (especially the CLI/formatter/config-reference chapters, which
  describe the same grammar-adjacent territory CONTRIBUTING.md and spec.md already
  cover) MUST be written in the project's own words, never copied from Bentley/
  Citilabs docs or from the local `_archive/` mirror. Since all source content here
  is Drut's own existing CHANGELOG/spec/source-comment prose rewritten for a reader,
  not vendor text, this is achievable by construction — flagged as a task-level
  writing constraint (tasks.md), not a design blocker. **Pass.**
- **III. Formatter Idempotence & Behavior Preservation** — N/A structurally (no
  formatter code changes); the Formatter guide chapter's *content* must accurately
  state the existing idempotence/behavior-preservation guarantee rather than
  overstate or understate it. **Pass.**
- **IV. False Negatives Over False Positives** — N/A (no new diagnostic/lint logic).
  **Pass.**
- **V. Vertical, Independently-Usable Increments** — This feature has no fixture
  corpus of its own (it's not core-crate grammar code) — the equivalent gate here is
  the doc-coverage check plus `mdbook build` succeeding in CI before the site is
  considered done for this feature. No next phase is blocked on this one. **Pass
  (adapted gate, not the literal fixture corpus).**
- **VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs** — N/A (no editor
  integration code). The Editor/LSP guide chapter's content must describe the
  already-standard mechanisms accurately (e.g. `021`'s `workspace/configuration`
  usage) rather than imply VS Code-only reach where none exists. **Pass.**
- **VII. Naming Honesty** — Directly applicable to the site's own wording: chapter
  and feature descriptions must not overclaim (e.g. the undefined-`@token@` Hint
  must be described as a best-effort hint with documented blind spots, not as a
  guaranteed "finds every undefined variable" checker). **Pass, tracked as a
  content-accuracy task.**
- **VIII. Public/Private Boundary** — Directly applicable: the site is part of the
  public repository, so it MUST NOT include any content derived from the private
  `_archive/` vendor-documentation mirror. All new site content in this feature is
  written from the project's own existing public source/spec/CHANGELOG material.
  **Pass.**

No violations requiring justification — Complexity Tracking table is empty/omitted.

**Post-Phase-1 re-check**: data-model.md, contracts/, and quickstart.md introduce
no new component beyond what Technical Context already scoped (mdBook content, one
PowerShell coverage script, one build-check-only GitHub Actions workflow with no
deploy job) — no gate re-evaluation changes. Still **Pass** on all eight
principles.

## Project Structure

### Documentation (this feature)

```text
specs/022-docs-site/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md         # Phase 1 output (/speckit-plan command)
├── quickstart.md         # Phase 1 output (/speckit-plan command)
├── contracts/            # Phase 1 output (/speckit-plan command)
│   ├── site-structure.md
│   └── config-reference-entry.md
├── checklists/
│   └── requirements.md
└── tasks.md              # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
docs-site/                       # New: mdBook book SOURCE (never served directly;
│                                 # not a Cargo workspace member)
├── book.toml                    # mdBook config: title, GitHub Pages site-url, theme,
│                                 # search, build-dir = "../docs" (research.md §2)
└── src/
    ├── SUMMARY.md                # mdBook's table of contents — drives navigation
    ├── introduction.md           # What is Drut, why it exists
    ├── install.md                 # CLI (cargo/binary) + VS Code/Open VSX extension
    ├── getting-started.md         # First check/format walkthrough against a sample script
    ├── cli-reference.md           # check/format/server/mcp subcommands and flags
    ├── editor-guide.md            # diagnostics, hover, completion, folding, format-on-save/
    │                               # paste, editor client settings (021)
    ├── mcp-guide.md                # diagnose/format/query_structure/lookup_keyword
    ├── formatter-guide.md          # casing categories, indentation, operator spacing,
    │                               # blank-line normalization, FMT:OFF/ON regions
    └── configuration-reference.md  # the centerpiece: all 10 [format] fields, precedence
                                    # chain, legacy/granular relationships

docs/                             # New: mdBook's COMMITTED build output — this IS the
├── .nojekyll                     # published site (research.md §2). GitHub Pages
├── index.html                    # ("Deploy from a branch" → main → /docs) serves this
├── ...                           # folder directly; regenerated + committed by whoever
                                  # publishes a content change, never hand-edited.

dev-notes/                        # New: relocated from the old docs/ (research.md §5)
└── known-environment-quirks.md   # unrelated to the published site; moved because docs/
                                  # is now reserved for committed Pages output

.github/workflows/
└── docs.yml                     # New: ONE job — mdbook build + coverage check +
                                  # freshness check, on every push/PR. No deploy job,
                                  # no Pages permissions, no secrets (research.md §2a)

scripts/                          # New home for small cross-cutting automation
├── build-docs.ps1                 # mdbook build (docs-site -> docs/) + recreate
│                                 # docs/.nojekyll — used identically by a local
│                                 # publish and by docs.yml's freshness-check step
└── check-docs-coverage.ps1       # fails if a drut-config FormatConfig field
                                  # name is missing from configuration-reference.md

README.md                         # Edited: trimmed to a short pitch page linking to the
                                  # published site as the documentation home (FR-007/008)
CONTRIBUTING.md                   # Edited: user-facing content (the "Editor behavior" and
                                  # "Configuration" sections) removed/replaced with pointers
                                  # to the site; contributor-facing content untouched (FR-009);
                                  # gains a Workflow-section pointer stating FR-011's
                                  # update-the-site obligation explicitly (analyze finding E2)
```

**Structure Decision**: `docs-site/` (mdBook source) and `docs/` (mdBook's
committed build output, which GitHub Pages serves) are two distinct top-level
directories, not one — required, not just tidy, once Pages is configured to serve
`main`'s `/docs` folder directly (classic "Deploy from a branch" only offers
repo-root `/` or `/docs`; research.md §2, revised 2026-08-17 after direct owner
correction to minimize this feature's GitHub Actions usage). The pre-existing
`docs/known-environment-quirks.md` (a contributor/dev-machine troubleshooting log)
moves to a new `dev-notes/` directory, since `docs/` is now reserved for published
output and GitHub Pages serves every file placed there — an internal engineering
note has no business becoming technically fetchable as part of the public site
(research.md §5). `scripts/` is new because no existing directory is the right
home for a small cross-cutting shell script that isn't part of any one crate; it
can hold future non-crate automation too.

## Complexity Tracking

*No constitution violations — table omitted.*
