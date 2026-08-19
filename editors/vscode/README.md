# Drut for Cube Voyager

> **This package is under active development.** If you find any bugs, please
> [report an issue on GitHub](https://github.com/ar-puuk/drut-dev/issues).

Syntax highlighting, structural diagnostics, formatting, and full Language
Server support for **Cube Voyager control-statement scripts** (`.s` /
`.block` files) — the script language transportation planners use to write
travel-demand model logic.

Editing these scripts without this extension usually means no syntax
highlighting, no error checking until the model actually runs, and no
consistent formatting across a team. Drut catches structural mistakes as
you type and keeps scripts formatted consistently.

## Features

- **Syntax highlighting** — a static TextMate grammar (works even before the
  language server starts) covering control words, function calls,
  `keyword=value` pairs, data references (`MI`/`MW`/`DBA`/`ZONES`/...), and
  ordinary user variables, each independently recolorable via
  `drut.highlight.*` settings.
- **Structural diagnostics** — unmatched `IF`/`LOOP`/`RUN`/`PROCESS` blocks,
  unclosed comments, and other real mistakes, flagged before you run the
  model.
- **Formatting** — consistent indentation and, optionally, casing (control
  words, pair keywords, data references, and function names, independently),
  on save, on paste, or on demand. `; FMT: OFF` / `; FMT: ON` markers
  exclude a specific range when you need to.
- **Hover, autocomplete, and "did you mean" spell-check** for control words
  and keyword names.
- **Code folding** for every block kind and block comment.
- **Shared project configuration** via a `drut.toml` file at the root of
  your project, so a team doesn't need to agree on editor settings
  individually.

## Nothing to install separately

On first activation, this extension resolves a working `drut` language
server automatically: it checks `PATH` first (never overriding a copy you
already have installed or built from source), then its own storage from a
previous activation, then — if neither is present — downloads the correct
binary for your platform from the project's GitHub Releases and verifies it
against its published checksum before trusting it. If none of that is
possible (offline, or an unsupported platform/architecture), the extension
still works for syntax highlighting; only diagnostics, hover, completion,
and formatting are unavailable until a binary can be resolved.

## More

- [Full documentation](https://ar-puuk.github.io/drut-dev/) — install,
  getting started, the CLI/editor/MCP references, formatter behavior, and a
  complete field-by-field `drut.toml` configuration reference.
- [What's changed release to release](https://github.com/ar-puuk/drut-dev/blob/main/CHANGELOG.md)
- [Report an issue](https://github.com/ar-puuk/drut-dev/issues)

Licensed under either the [MIT License](https://github.com/ar-puuk/drut-dev/blob/main/LICENSE-MIT)
or the [Apache License, Version 2.0](https://github.com/ar-puuk/drut-dev/blob/main/LICENSE-APACHE), at your option.
