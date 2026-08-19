# Drut

> **This package is under active development.** If you find any bugs, please
> [report an issue on GitHub](https://github.com/ar-puuk/drut-dev/issues).

Drut is a linter, formatter, and editor tooling suite for **Cube Voyager
control-statement scripts** (`.s` / `.block` files) — the script language
transportation planners use to write travel-demand model logic. Editing
these scripts today usually means no syntax highlighting, no error checking
until the model actually runs, and no consistent formatting across a team.
Drut catches structural mistakes as you type, keeps scripts formatted
consistently, and adds real editor support — syntax highlighting, hover
help, autocomplete — to VS Code and any other editor that speaks the
Language Server Protocol.

## Install

**VS Code / any VS Code-compatible editor**: install
[Drut for Cube Voyager](https://marketplace.visualstudio.com/items?itemName=arpuuk.drut)
from the VS Code Marketplace, or [from Open VSX](https://open-vsx.org/extension/arpuuk/drut)
on VS Code-compatible editors that use it instead (Cursor, VSCodium, etc.).
The extension resolves a working `drut` language server automatically on
first activation — nothing else to install.

**Just the CLI**, for scripting or CI:

```sh
cargo install drut-cli
```

Or build from source:

```powershell
cargo build --release -p drut-cli
# binary at target/release/drut(.exe) -- put it on PATH
```

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
  on save, on paste, or from the CLI. `; FMT: OFF` / `; FMT: ON` markers
  exclude a specific range when you need to.
- **Hover, autocomplete, and "did you mean" spell-check** for control words
  and keyword names.
- **Code folding** for every block kind and block comment.
- **Shared project configuration** via a `drut.toml` file, so a team doesn't
  need to agree on CLI flags or editor settings individually.
- **Three ways in** — a `drut` CLI, a VS Code/Open VSX extension (built on
  the Language Server Protocol, so it works in any LSP-capable editor, not
  only VS Code), and a Model Context Protocol server for AI coding
  assistants — all built on one shared engine, so every surface agrees on
  what's valid.

## Documentation

**[User guide](https://ar-puuk.github.io/drut-dev/)** — install, getting
started, the CLI/editor/MCP references, formatter behavior, and a complete
field-by-field `drut.toml` configuration reference. Start here for using Drut.

Contributing to Drut itself:

- [`CONTRIBUTING.md`](CONTRIBUTING.md) — architecture, per-crate status,
  build/test commands, versioning, and credits.
- [`CHANGELOG.md`](CHANGELOG.md) — what's shipped, in user-facing terms.
- [`ROADMAP.md`](ROADMAP.md) — what's left before first publish.

## License

Licensed under either the [MIT License](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
