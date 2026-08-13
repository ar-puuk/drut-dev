# drut-mcp

Model Context Protocol server for Cube Voyager control-statement scripts,
part of [Drut](https://github.com/ar-puuk/drut-dev). Gives an LLM agent
read-only, structural tools over [`voyager-core`](https://crates.io/crates/voyager-core)
instead of asking it to parse Voyager syntax by eye.

This crate is a library, not a standalone binary — its tools are exposed as
the `drut mcp` subcommand of [`drut-cli`](https://crates.io/crates/drut-cli),
which is what an MCP-capable client actually launches.

## Tools

- **`diagnose`** — run `voyager-core`'s parser over a script and return its
  diagnostics.
- **`format`** — format a script, honoring `drut.toml` the same way the
  CLI's `format` subcommand does. Returns the formatted text; never writes
  to disk itself.
- **`query_structure`** — ask structural questions about a script (block
  boundaries, matching, nesting) without re-deriving them from raw text.
- **`lookup_keyword`** — look up a control/statement word's recognized
  category.

`diagnose`/`query_structure`/`lookup_keyword` are read-only; `format`
returns formatted text rather than modifying anything in place — this
server never edits a file on disk.

## Part of the Drut workspace

See the [main repository](https://github.com/ar-puuk/drut-dev) for the full
toolchain — the `drut` CLI, a Language Server, and a VS Code/Open VSX
extension, all sharing this same `voyager-core` foundation.

Licensed under either of [Apache License, Version 2.0](https://github.com/ar-puuk/drut-dev/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/ar-puuk/drut-dev/blob/main/LICENSE-MIT) at your option.
