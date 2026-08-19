# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
once a first version is actually tagged — `voyager-core`, `drut-config`,
`drut-cli`, `drut-lsp`, `drut-mcp`, and the VS Code/Open VSX extension move
together in lockstep at the same version number on every release (see
CONTRIBUTING.md's "Versioning" section).

## [Unreleased]

### Changed

- **Breaking**: every `[format]` field name changed to a flat,
  group-prefixed shape — the "group" word (`casing`, `indent`, `blank_lines`)
  now leads the name instead of trailing it, matching the convention rustfmt
  itself uses (`imports_granularity`, `imports_layout`, ...) for the same
  reason: it clusters related settings alphabetically (in `--help` output,
  editor autocomplete, and this changelog's own docs) instead of scattering
  them. Renamed: `control_words_casing` → `casing_control_words`,
  `pair_keywords_casing` → `casing_pair_keywords`, `data_references_casing`
  → `casing_data_references`, `top_level_indent` → `indent_top_level`,
  `top_level_blank_line_cap` → `blank_lines_top_cap`,
  `nested_blank_line_cap` → `blank_lines_nested_cap`. Applies identically to
  `drut.toml` keys, CLI flags (e.g. `--control-words-casing` →
  `--casing-control-words`), the MCP `format` tool's parameters, and the VS
  Code `drut.format.*` settings (e.g. `drut.format.controlWordsCasing` →
  `drut.format.casingControlWords`). `indent_width`, `operator_spacing`, and
  `blank_lines` are unchanged — each already led with its own group name (or
  needed no grouping at all). An old name in any of these four places now
  degrades to a plain "unrecognized key"/usage-error, the same non-blocking
  fallback every other unrecognized `[format]` key already gets — nothing
  silently keeps working under its old name. `voyager-core`'s own public
  `FormatOptions` struct (published independently to crates.io) renamed its
  matching fields/type the same way — `top_level_indent` →
  `indent_top_level`, `top_level_blank_line_cap` → `blank_lines_top_cap`,
  `nested_blank_line_cap` → `blank_lines_nested_cap`,
  `TopLevelIndentMode` → `IndentTopLevelMode` — so there's one name for each
  setting end to end, not an internal name translated to an external one at
  the `drut-config` boundary.
- **Breaking**: removed the legacy, flat `casing` setting — superseded by the
  three granular fields (`casing_control_words`, `casing_pair_keywords`,
  `casing_data_references`) since `017-casing-categories-indent-width`, which
  together already cover everything `casing` used to (`control_words`+
  `pair_keywords` together; `data_references` was never reachable through it
  at all). Removed everywhere it existed: `drut.toml`'s `casing` key, the
  CLI's `--casing` flag, the MCP `format` tool's `casing` parameter, and the
  VS Code `drut.format.casing` setting. A `drut.toml`/MCP call still using
  `casing` now gets a plain "unrecognized key" warning and each category
  falls back to its own built-in default (`preserve`) instead of the removed
  field's value — the same non-blocking degrade every other unrecognized key
  or invalid value already gets, never a hard failure. `--casing` on the CLI
  is now a usage error (unknown flag) rather than being silently accepted.
  If you relied on one `casing = "upper"` covering both categories, set
  `casing_control_words` and `casing_pair_keywords` explicitly instead.
- **Breaking**: `indent_top_level`'s non-`preserve` value is now `"auto"`,
  not `"normalize"` — matching the `preserve`/`auto` naming
  `operator_spacing`/`blank_lines` already use for the same "leave it
  alone" vs. "actively fix it" shape. Applies identically to `drut.toml`
  (`indent_top_level = "auto"`), the CLI (`--indent-top-level=auto`), the
  MCP `format` tool (`indent_top_level: "auto"`), and the VS Code
  `drut.format.indentTopLevel` setting. A `drut.toml`/CLI/MCP value of
  `"normalize"` is no longer recognized — it now warns and falls back to
  the built-in default (`preserve`), the same as any other invalid value,
  rather than being silently accepted under its old name.

### Fixed

- A single-line, self-closing short-`IF` (e.g. `IF (@MODE@ = 1) PRINT
  LIST=...`, valid without a matching `ENDIF`) rendered its entire
  condition — `@token@` references, operators, numbers — in one flat
  color instead of each element's normal distinct color, unlike the
  equivalent multi-line block-style `IF`. The editor's semantic
  highlighting was tagging the whole header (`IF` through the condition's
  closing paren) as one token to distinguish a short-`IF` from a
  block-style one; narrowed to just the `IF`/`ELSEIF` keyword itself, so
  the condition and body now color exactly like the block-style form.
- `operator_spacing = "fixed"`/`"auto"` spaced apart a `-` joining two bare
  integer literals inside a `Control` statement's pair-keyword value (e.g.
  `SELECTLINK=1-50,75,90-100` → `SELECTLINK=1 - 50,75,90 - 100`), even
  though that's Cube Voyager's own inclusive-range list notation, not
  arithmetic subtraction — confirmed live in the real fixture corpus
  (`mo=31-60`, `EXCLUDEGROUP=1-2,7`, among others), not just a
  hypothetical case. A binary `-` joining two bare integer literals inside
  a pair-keyword value now renders with zero surrounding whitespace
  instead, regardless of how it was originally spaced (`1 - 50`/`1- 50`/
  `1 -50` all become `1-50`); a `-` anywhere else (an `Assignment`'s
  right-hand side, an `IF`/short-`IF` condition, a `LOOP` bound) is
  unaffected and keeps its existing spacing, and `operator_spacing` left
  unset or `"preserve"` is unaffected either way. No new `[format]` field,
  CLI flag, MCP parameter, or editor setting — this is a correction to
  `operator_spacing`'s existing `fixed`/`auto` behavior, not a new one.
- The VS Code syntax highlighting colored a Cube Voyager built-in function
  call (e.g. `REPLACESTR(...)`) only by accident, when the call happened to
  sit immediately after `=` and got caught by an unrelated rule for
  coloring assignment values — the identical function one token deeper
  (nested inside another call's arguments, or inside an `IF` condition,
  e.g. `RIGHTSTR(TRIM(RouteName),1)`) rendered as plain, unstyled text.
  Added a dedicated function-call recognition rule that colors a
  recognized built-in function name every time it's immediately followed
  by `(`, regardless of where in the statement it sits. The recognized-name
  list (138 functions) was built by reading every function-related chapter
  of two vendor documentation editions (Cube Voyager 6.5.1 and OpenPaths
  Cube/CUBE CONNECT Edition) — covering the general-purpose Control
  Language functions (`ABS`, `TRIM`, `REPLACESTR`, `ROUND`, ...), Highway/
  Matrix-program functions (`ROWSUM`, `PATHTRACE`, ...), Public Transport
  skim functions (`TIMEA`, `BRDINGS`, `GCOST`, ...), the CONVERGE-phase
  iteration-statistics family (`GAPCHANGE`, `RGAPMIN`, ...), and CUBE
  Cluster utility functions — not just names this project's own real
  script corpus happens to call. No `voyager-core`/parser change, no new
  `[format]` field, CLI flag, MCP parameter, or editor setting — purely a
  VS Code syntax-highlighting correction.

## [0.2.1] - 2026-08-18

### Fixed

- The VS Code Marketplace/Open VSX "Changelog" tab showed nothing —
  `editors/vscode/` (the directory actually packaged into the `.vsix`) never
  had a `CHANGELOG.md` of its own; only the repo root did, one directory up
  from what `vsce`/`ovsx` bundle. A new `vscode:prepublish` npm script (a
  hook `vsce package`/`vsce publish` already run automatically, the same
  purpose `npm run compile` already used it for) copies the repo-root
  `CHANGELOG.md` into `editors/vscode/CHANGELOG.md` immediately before
  packaging, every time — generated fresh at publish time, not a
  hand-maintained duplicate that could go stale.

## [0.2.0] - 2026-08-17

### Added

- Casing is now three independently-configurable settings instead of one:
  `control_words`, `pair_keywords`, and a new `data_references` category
  covering the Matrix/Line/Node/Zone/Database abbreviations (`MI`/`MO`/`MW`,
  `LI`/`LW`, `NI`/`NW`, `ZI`/`ZONES`/`Z`, `DBI`/`DBA`), `RO`, the link
  endpoint fields `A`/`B`, and the reserved loop-index identifiers `I`/`J`
  — all previously untouched by casing no matter what was configured.
  Each of the three accepts `upper`/`lower`/`preserve` independently (still
  defaulting to `preserve`) via `drut.toml`, the CLI, and the MCP `format`
  tool; the existing flat `casing` setting keeps working exactly as before
  (still covers `control_words`+`pair_keywords` together) alongside the new
  per-category controls. No built-in "auto" or opinionated preset ships —
  every value comes from a project's own configuration.
- `indent_width` is now a configurable `[format]` setting (default `4`,
  matching prior behavior), alongside `casing`/`top_level_indent`.
- A new `operator_spacing` `[format]` setting (`preserve`/`fixed`/`auto`,
  default `preserve`) normalizes whitespace around operators. `fixed`
  brings every occurrence of `=`, the comparison operators (`==`, `<>`,
  `>=`, `<=`, `<`, `>`), and binary arithmetic (`+`, `-`, `*`, `/`) to
  exactly one space on each side; normalizes comma spacing between
  multiple `keyword=value` pairs on one statement; and removes interior
  padding inside `[...]`/`(...)` and the space between a control word and
  its opening `(` (e.g. `IF (x==1)` → `IF(x == 1)`). A unary `+`/`-` (a
  signed literal, or one immediately following another operator) is never
  spaced apart from its operand. `auto` does everything `fixed` does, plus
  vertically aligns the `=` of consecutive `Assignment` statements to the
  column of the longest left-hand side in the run — resetting
  independently at a blank line, a comment-only line, a nesting-depth
  change, or a non-`Assignment` statement (a pair-keyword `Control`
  statement's own `=` is spaced but never joins or extends an alignment
  run). `; FMT: OFF`/`; FMT: ON` regions and string/quoted-literal content
  are never touched by either mode. Exposed identically via `drut.toml`,
  the CLI (`--operator-spacing`), and the MCP `format` tool
  (`operator_spacing`); `preserve` remains the default, so a project with
  nothing configured sees zero behavior change.
- A new `blank_lines` `[format]` setting (`preserve`/`auto`, default
  `preserve`) caps runs of consecutive blank lines. `auto` contracts (never
  pads) a run down to `top_level_blank_line_cap` (default `2`) between
  top-level statements/blocks, or `nested_blank_line_cap` (default `1`)
  anywhere inside any block's own body, regardless of nesting depth — a
  whitespace-only line counts as blank. `; FMT: OFF`/`; FMT: ON` regions
  are never touched. Exposed identically via `drut.toml`, the CLI
  (`--blank-lines`, `--top-level-blank-line-cap`,
  `--nested-blank-line-cap`), and the MCP `format` tool (`blank_lines`,
  `top_level_blank_line_cap`, `nested_blank_line_cap`); `preserve` remains
  the default, so a project with nothing configured sees zero behavior
  change.
- The editor now shows a subtle Hint-level underline on an `@token@`
  reference with no assignment findable in the same file or a directly
  included one — never a hard Error, since a resolver blind spot (a
  `@token@` on a block-opener line, more than one level of `READ FILE`
  inclusion, or a token-built inclusion path) is never itself treated as
  evidence a token is undefined. LSP-only — never reaches the `check`
  command or the MCP `diagnose` tool, matching how the existing unclosed
  `; FMT: OFF` and malformed `drut.toml` hints already behave. No
  configuration surface; always on.
- All 10 `[format]` settings (`casing`, `control_words_casing`,
  `pair_keywords_casing`, `data_references_casing`, `top_level_indent`,
  `indent_width`, `operator_spacing`, `blank_lines`,
  `top_level_blank_line_cap`, `nested_blank_line_cap`) are now settable as
  personal editor (client) settings, not only via a project's committed
  `drut.toml`. A new precedence tier — `client_defaults` — sits between
  `drut.toml` and the built-in default: `explicit CLI flag/MCP parameter >
  drut.toml > client setting > built-in default`. A `drut.toml` value
  always wins over a conflicting client setting for the same field; a
  client setting only ever fills in a field `drut.toml` leaves unset.
  Delivered via the standard LSP `workspace/configuration`/
  `workspace/didChangeConfiguration` mechanism (not a VS Code-proprietary
  side channel) — `drut-lsp` pulls the client's merged `"drut.format"`
  section once at startup (when the client advertises support) and again
  on every `workspace/didChangeConfiguration` notification, so a changed
  setting is reflected on the very next format request against an
  already-open document, with no reopen or editor restart needed. A client
  that doesn't advertise `workspace/configuration` support is never asked
  at all — formatting behaves exactly as before this feature, with no
  error or degraded experience. Exposed in the VS Code extension as 10 new
  `drut.format.*` settings (e.g. `drut.format.controlWordsCasing`,
  `drut.format.indentWidth`), visible and settable through VS Code's
  built-in Settings UI. Scoped entirely to the LSP surface — `drut-cli`
  and the MCP `format` tool gain no new capability and are behaviorally
  unaffected.
- A published, searchable user guide, built with [mdBook](https://rust-lang.github.io/mdBook/):
  an introduction, install instructions (CLI and the VS Code/Open VSX
  extension), a getting-started walkthrough, a CLI reference, an editor (LSP)
  guide, an MCP guide, a formatter behavior guide with real before/after
  examples for every formatting axis, and — the specific gap that prompted
  this — a complete, field-by-field `drut.toml` configuration reference
  covering all 10 `[format]` fields (values, defaults, effect, examples, and
  the shared four-tier precedence chain), replacing `CONTRIBUTING.md`'s old
  two-of-ten-field "Configuration" section. Hosted on GitHub Pages, served
  directly from a committed `docs/` folder — no GitHub Actions deploy step; a
  single build-check CI job (`mdbook build` plus a coverage check tying the
  configuration reference to `drut-config`'s real field list, plus a
  freshness check catching a forgotten rebuild) still gates every push/PR.
  `README.md` now links to the site as the documentation home.

### Fixed

- `NUMREC`, `CNT`, `ITER`, `LP`, and `RECNUM` no longer appear as completion/
  spell-check suggestions for a `LOOP` statement's variable-name position —
  they were never real Voyager keywords, just names a prior census
  mistakenly picked up (the position genuinely accepts any user-chosen
  name). `ZONES` was added in their place, a real, previously-missing
  keyword.

## [0.1.3] - 2026-08-16

### Added

- Hovering an `@token@` reference now shows the value it currently resolves
  to, and where that value was assigned — the most recent same-file
  `TOKEN = value` assignment before the reference, or, if none exists, one
  found in a file the document directly pulls in via a literal
  `READ FILE = '<path>'` statement (e.g. a scenario's "control center"
  file). Token-built `READ FILE` paths (e.g. `@ParentDir@...`) and anything
  beyond that first level of inclusion are not resolved; hovering such a
  token falls back to the previous hover behavior rather than guessing.

### Fixed

- `@token@` syntax highlighting now colors the whole reference — both `@`
  delimiters and the name between them — as a single, uniform color.
  Previously the delimiters were scoped separately from the name, so most
  themes rendered the `@`s in a different (often muted) color than the
  token itself.

## [0.1.2] - 2026-08-13

### Fixed

- The VS Code Marketplace listing's "Overview" tab showed "No overview has
  been entered by publisher" — `editors/vscode/README.md`, the file `vsce`
  bundles as the Marketplace-facing overview, had never existed. Added it,
  distinct from the five crates.io `README.md` files added in 0.1.1 (those
  cover the Rust crates; this one covers the extension itself, scoped for
  a Marketplace visitor rather than a crates.io one).
- The top-level `README.md`'s Install section still said "Not yet
  published to the VS Code Marketplace, Open VSX, or crates.io," stale
  since 0.1.1 actually shipped to all three — replaced with real install
  instructions for each.

## [0.1.1] - 2026-08-13

### Added

- A `README.md` for each of the five published crates (`voyager-core`,
  `drut-config`, `drut-cli`, `drut-lsp`, `drut-mcp`), plus `repository`
  metadata in every `Cargo.toml` — each crate's crates.io page now shows
  real, crate-specific documentation instead of just its one-line
  description.

### Changed

- Renamed the VS Code/Open VSX extension's publisher and identity from
  `drut-project.drut-voyager` to `arpuuk.drut` — including the internal
  language ID and TextMate scope name, not just the Marketplace-facing
  name. The `drut-project.drut-voyager` listing was unpublished from the
  VS Code Marketplace before it had any real installs.

## [0.1.0] - 2026-08-13

First tagged release.

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
