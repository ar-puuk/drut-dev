# drut-lsp

Language Server Protocol implementation for Cube Voyager control-statement
scripts, part of [Drut](https://github.com/ar-puuk/drut-dev). Diagnostics,
hover, completion, folding ranges, semantic tokens, and formatting over
[`voyager-core`](https://crates.io/crates/voyager-core).

This crate is a library, not a standalone binary — its handlers are exposed
as the `drut server` subcommand of [`drut-cli`](https://crates.io/crates/drut-cli),
which is what a real editor extension (VS Code, or any other LSP-capable
client) actually launches.

## What it does

- Diagnostics for the six categories `voyager-core` recognizes
  (`UnmatchedIf`, `UnmatchedLoop`, `UnclosedBlockComment`,
  `InvalidContinuation`, `UnmatchedRun`, `MisplacedBreak`), live-updated as
  `drut.toml` changes in editors that support dynamic file-watch
  registration.
- Formatting, honoring `drut.toml`'s `[format]` table and any editor-side
  formatting request options.
- Folding ranges for every block kind and block comments.
- Semantic tokens for constructs a static TextMate grammar can't express on
  its own (a self-closing short-`IF`, an unreachable statement after a
  resolved `BREAK`).

No per-program semantic validation — structural correctness only, matching
`voyager-core`'s own scope.

## Part of the Drut workspace

See the [main repository](https://github.com/ar-puuk/drut-dev) for the full
toolchain, including the VS Code/Open VSX extension this server backs.

Licensed under either of [Apache License, Version 2.0](https://github.com/ar-puuk/drut-dev/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/ar-puuk/drut-dev/blob/main/LICENSE-MIT) at your option.
