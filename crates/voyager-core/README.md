# voyager-core

Tokenizer and structural parser for Cube Voyager control-statement scripts
(`.s` / `.block` files), targeting Cube Voyager 6.5 as the grammar baseline.

This is the single source of truth for Drut's Voyager grammar — every other
crate in the workspace (`drut-cli`, `drut-lsp`, `drut-mcp`) builds on it rather
than re-implementing any parsing logic of its own.

## Public API

Two pure functions operating on in-memory text only, with no file I/O,
network access, or protocol dependency:

```rust
pub fn tokenize(source: &str) -> Vec<Token>;
pub fn parse(source: &str) -> ParseResult;
```

Neither function panics on any input, including malformed input — defects
surface as `Diagnostic` values in the result, never as a panic or `Err`.

The parser recognizes four statement forms (`Control`, `Assignment`, `Label`,
`ShellEscape`) and seven block kinds (`If`, `Loop`, `Run`, `Process`, `JLoop`,
`LinkLoop`, `DistributeMultistep`), matched structurally with no
per-program semantic validation.

## Zero runtime dependencies

`voyager-core` depends on nothing but `std`. `cargo tree -p voyager-core`
never shows an external crate — this is a deliberate, enforced constraint,
not an accident of what's been needed so far.

## Part of the Drut workspace

See the [main repository](https://github.com/ar-puuk/drut-dev) for the full
toolchain this crate powers: the `drut` CLI (`check`/`format`/`server`/`mcp`
subcommands), a Language Server, an MCP server, and a VS Code/Open VSX
extension with automatic binary bootstrap.

Licensed under either of [Apache License, Version 2.0](https://github.com/ar-puuk/drut-dev/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/ar-puuk/drut-dev/blob/main/LICENSE-MIT) at your option.
