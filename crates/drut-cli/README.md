# drut-cli

> **This package is under active development.** If you find any bugs, please
> [report an issue on GitHub](https://github.com/ar-puuk/drut-dev/issues).

The `drut` command-line tool for Cube Voyager control-statement scripts
(`.s` / `.block` files) — structural linting, formatting, and editor
integration, built on [`voyager-core`](https://crates.io/crates/voyager-core).

## Install

```sh
cargo install drut-cli
```

This installs a binary named `drut`.

Most people won't need to do this directly — the [VS Code/Open VSX
extension](https://marketplace.visualstudio.com/items?itemName=arpuuk.drut)
downloads and manages a matching `drut` binary automatically on first
activation. This crate is for anyone who wants the CLI on its own: CI
pipelines, scripting, or a different editor's LSP client.

## Subcommands

```sh
drut check <path>    # structural diagnostics, human-readable or SARIF
drut format <path>   # format in place or check formatting, honoring drut.toml
drut server           # launch the Language Server (stdio)
drut mcp               # launch the MCP server (stdio)
```

`check`/`format` honor a `drut.toml` project configuration file
(discovered by walking up from the file being processed) unless
overridden by an explicit flag or run with `--isolated`.

## Part of the Drut workspace

See the [main repository](https://github.com/ar-puuk/drut-dev) for the full
toolchain: this CLI, a Language Server, an MCP server, and the VS Code/Open
VSX extension, all sharing the same `voyager-core` parser as their single
source of truth for Voyager grammar.

Licensed under either of [Apache License, Version 2.0](https://github.com/ar-puuk/drut-dev/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/ar-puuk/drut-dev/blob/main/LICENSE-MIT) at your option.
