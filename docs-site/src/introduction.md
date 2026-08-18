# Introduction

Drut is a linter, formatter, and editor tooling suite for **Cube Voyager
control-statement scripts** (`.s` / `.block` files) — the script language
transportation planners use to write travel-demand model logic.

Editing these scripts without Drut usually means no syntax highlighting, no
error checking until the model actually runs, and no consistent formatting
across a team. Drut catches structural mistakes as you type, keeps scripts
formatted consistently, and adds real editor support — syntax highlighting,
hover help, autocomplete — to VS Code and any other editor that speaks the
Language Server Protocol.

## What Drut does

- **Structural diagnostics** — unmatched `IF`/`LOOP`/`RUN`/`PROCESS` blocks,
  unclosed comments, and other real mistakes, flagged before you run the model.
  See the [Editor Guide](editor-guide.md) for the full list.
- **Formatting** — consistent indentation and, optionally, keyword casing,
  operator spacing, and blank-line normalization — on save, on paste, or from
  the CLI. See the [Formatter Guide](formatter-guide.md).
- **Hover, autocomplete, and "did you mean" spell-check** for control words and
  keyword names.
- **Shared project configuration** via a `drut.toml` file, so a team doesn't
  need to agree on CLI flags or editor settings individually. See the
  [Configuration Reference](configuration-reference.md).
- **Three ways in**: a `drut` CLI, a VS Code/Open VSX extension (built on the
  Language Server Protocol, so it works in any LSP-capable editor, not only VS
  Code), and a Model Context Protocol server for AI coding assistants — all
  built on one shared engine, so every surface agrees on what's valid.

## What Drut does not do

Drut is a **structural** and **formatting** tool, not a full semantic validator.
It does not:

- Validate program-box-specific keyword combinations (e.g. whether a particular
  `PATHLOAD` parameter combination is meaningful for the model you're running).
- Check cross-file/repo-wide semantics beyond the direct `READ FILE` inclusion
  hover already reaches.
- Run or simulate your model in any way.

These are explicit, current scope boundaries — not oversights.

## Who this is for

Anyone writing or reviewing Cube Voyager `.s`/`.block` scripts: transportation
model developers, analysts maintaining a shared script library, and teams that
want consistent formatting without hand-enforcing a style guide.

## Where to go next

- New to Drut? Start with [Install](install.md), then
  [Getting Started](getting-started.md).
- Looking for a specific `drut.toml` field? Jump straight to the
  [Configuration Reference](configuration-reference.md).
- Integrating an AI coding assistant? See the [MCP Guide](mcp-guide.md).
