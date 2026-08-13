# Drut

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

Not yet published to the VS Code Marketplace, Open VSX, or crates.io — see
[`ROADMAP.md`](ROADMAP.md) for what's left before that's true. For now,
build from source:

```powershell
# Build the `drut` CLI
cargo build --release -p drut-cli
# binary at target/release/drut(.exe) -- put it on PATH
```

```powershell
# Build and package the VS Code extension
cd editors\vscode
npm install
npm run compile
npx @vscode/vsce package
# In VS Code: Extensions view -> "..." menu -> Install from VSIX... -> select the generated .vsix
```

## Features

- **Structural diagnostics** — unmatched `IF`/`LOOP`/`RUN`/`PROCESS` blocks,
  unclosed comments, and other real mistakes, flagged before you run the
  model.
- **Formatting** — consistent indentation and, optionally, keyword casing,
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

- [`CONTRIBUTING.md`](CONTRIBUTING.md) — architecture, per-crate status,
  configuration design, build/test commands, versioning, and credits.
- [`CHANGELOG.md`](CHANGELOG.md) — what's shipped, in user-facing terms.
- [`ROADMAP.md`](ROADMAP.md) — what's left before first publish.

## License

Licensed under either the [MIT License](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
