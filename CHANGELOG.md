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

- Batteries-included install for the VS Code extension: on activation, the
  extension now automatically resolves a working `drut` binary with no
  manual install step required. It checks `PATH` first — never
  second-guessing a binary already on it — then its own persistent
  extension storage from a prior activation, then, if neither is present,
  downloads the correct binary for your platform from the latest GitHub
  Release and verifies it against its published SHA-256 checksum before
  trusting it. If every option is unavailable (offline, an unsupported
  platform/architecture, or a failed/unverifiable download), the extension
  degrades gracefully to syntax-highlighting-only rather than failing
  outright, and says why exactly once. Once installed this way, a
  throttled (at most once per 24 hours), non-blocking background check
  offers a dismissible notification when a newer release is available — it
  never silently replaces a running binary.
- `drut.toml` project configuration file: a `[format]` table (`casing`,
  `top_level_indent`) discovered by walking up from each file being
  processed, respected identically by the CLI, LSP, and MCP surfaces.
  Explicit CLI flags/MCP parameters still win over it; a `--isolated` CLI
  flag skips it entirely; a malformed value warns and falls back to the
  built-in default for just that field rather than failing the whole run.
- `--top-level-indent` option (`preserve`/`normalize`, default `preserve`)
  to control whether top-level (depth-0) statement indentation is left
  exactly as written or normalized to column 0.
- `--casing=preserve` as an explicit third value alongside `upper`/`lower`:
  lets one invocation force "leave casing untouched" even when `drut.toml`
  sets a project-wide casing convention, mirroring `--top-level-indent`'s
  existing `preserve`/`normalize` shape.
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
- The VS Code extension now has a proper Marketplace/Open VSX icon and
  correctly bundles its dual MIT/Apache-2.0 license text in the `.vsix`
  (previously omitted, since packaging includes only `editors/vscode/` in
  isolation from the repo-root `LICENSE-MIT`/`LICENSE-APACHE` files).
