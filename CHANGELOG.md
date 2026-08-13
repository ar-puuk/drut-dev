# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
once a first version is actually tagged — `voyager-core`, `drut-config`,
`drut-cli`, `drut-lsp`, `drut-mcp`, and the VS Code/Open VSX extension move
together in lockstep at the same version number on every release (see
CONTRIBUTING.md's "Versioning" section).

## [Unreleased]

Everything below shipped to `main` prior to any tagged release, so it's
grouped here rather than under invented retroactive version numbers.

### Added

- `drut.toml` project configuration file: a `[format]` table (`casing`,
  `top_level_indent`) discovered by walking up from each file being
  processed, respected identically by the CLI, LSP, and MCP surfaces.
  Explicit CLI flags/MCP parameters still win over it; a `--isolated` CLI
  flag skips it entirely; a malformed value warns and falls back to the
  built-in default for just that field rather than failing the whole run.
- `--top-level-indent` option (`preserve`/`normalize`, default `preserve`)
  to control whether top-level (depth-0) statement indentation is left
  exactly as written or normalized to column 0.
- `; FMT: OFF` / `; FMT: ON` inline region markers to exclude a specific
  range of a script from formatting entirely. An unclosed `; FMT: OFF` is
  reported (CLI stderr notice, MCP response field, LSP hint diagnostic)
  rather than silently protecting the rest of the file.
- Format-on-save in the VS Code extension: `.s`/`.block` files get
  `editor.formatOnSave` enabled automatically on first activation
  (one-time; respects the setting being turned back off afterward).
- Format-on-paste in the VS Code extension (opt-in): pasting into a `.s`/
  `.block` file reformats just the pasted range, correctly handling a
  paste that opens or closes a block.
- Folding ranges in the VS Code extension: `IF`/`LOOP`/etc. blocks and
  block comments can be collapsed/expanded like any other language.
- A new diagnostic, `UnmatchedProcess`, for a `PROCESS`/`PHASE` block that
  is never closed — previously left unflagged.
- The language server now logs the exact binary path and build identifier
  it's running as, at startup, to help diagnose "which `drut` is VS Code
  actually using" issues.
- Live config updates: the language server now watches `drut.toml` across
  the workspace (in editors that support dynamic file-watch registration)
  and automatically refreshes every open document's diagnostics when it
  changes — no manual close/reopen needed. Editors without that capability
  fall back to the previous close/reopen behavior; no crash, no broken
  registration attempt.

### Fixed

- The formatter no longer leaves stale indentation on a genuinely
  unmatched/diagnosed block's child statements.
- An open document's `drut.toml`-driven diagnostic no longer goes stale
  when the config file is edited directly while the document stays open.
