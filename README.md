# Drut

Drut is a tokenizer, structural parser, and (eventually) linter/formatter/LSP+MCP
tooling suite for **Cube Voyager control-statement scripts** (`.s` / `.block`
files), targeting Cube Voyager 6.5 as its grammar baseline.

The project is architected as one authoritative Rust core crate — grammar, parsing,
and lint-rule logic — with thin adapters on top (CLI, LSP server, MCP server,
formatter, editor extension client). See [`.specify/memory/constitution.md`](.specify/memory/constitution.md)
for the full set of governing principles (single source of truth, no verbatim
vendor-documentation redistribution, formatter behavior-preservation, false
negatives preferred over false positives, vertical phase-gated delivery, and more).

## Status

Pre-implementation. The first feature — a dependency-free tokenizer and structural
parser (`voyager-core`) — is fully specified and task-planned but not yet built.
See [`specs/001-voyager-script-parser/`](specs/001-voyager-script-parser/) for:

- [`spec.md`](specs/001-voyager-script-parser/spec.md) — functional requirements,
  user stories, success criteria
- [`plan.md`](specs/001-voyager-script-parser/plan.md) — technical approach,
  workspace/crate layout, constitution compliance check
- [`data-model.md`](specs/001-voyager-script-parser/data-model.md) — token/
  statement/block/diagnostic entity definitions
- [`contracts/`](specs/001-voyager-script-parser/contracts/) — public API and
  diagnostic-taxonomy contracts
- [`tasks.md`](specs/001-voyager-script-parser/tasks.md) — dependency-ordered
  implementation tasks, organized by user story
- [`checklists/`](specs/001-voyager-script-parser/checklists/) — requirements-
  quality checklists run against the spec before implementation

Once `crates/voyager-core` exists, build/test it with:

```powershell
cargo build -p voyager-core
cargo test -p voyager-core
```

## Repository layout

```text
.specify/          Spec-kit workflow tooling (templates, scripts, constitution)
specs/             Per-feature spec-kit artifacts (spec/plan/tasks/contracts/...)
_archive/          Local-only vendor documentation mirrors, gitignored — never
                   committed; kept for reference during grammar research only
                   (see constitution Principle II / Principle VIII)
```

## Workflow

This project follows the [spec-kit](https://github.com/github/spec-kit) discipline:
every feature gets a spec, a plan, and a task list before implementation starts, and
a feature's fixture-corpus tests must pass cleanly before the next phase begins
(constitution Principle V). See `.specify/` for the slash-command workflow
(`/speckit-specify`, `/speckit-clarify`, `/speckit-plan`, `/speckit-tasks`,
`/speckit-checklist`, `/speckit-analyze`, `/speckit-implement`).
