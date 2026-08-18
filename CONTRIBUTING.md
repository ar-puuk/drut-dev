# Contributing to Drut

This covers Drut's architecture, per-crate feature status, configuration
design, repository layout, build/test commands, versioning policy,
dependency posture, and credits — everything beyond the short project pitch
in [`README.md`](README.md).

The project is architected as one authoritative Rust core crate — grammar, parsing,
and lint-rule logic — with thin adapters on top (CLI, LSP server, MCP server,
formatter, editor extension client). See [`.specify/memory/constitution.md`](.specify/memory/constitution.md)
for the full set of governing principles (single source of truth, no verbatim
vendor-documentation redistribution, formatter behavior-preservation, false
negatives preferred over false positives, vertical phase-gated delivery, and more).

## Status

Fourteen features shipped (`001`–`014`; see [`CHANGELOG.md`](CHANGELOG.md) for the
user-facing summary), all with passing fixture-corpus test gates (constitution
Principle V). Not yet published anywhere — see [`ROADMAP.md`](ROADMAP.md) for the
remaining pre-publish sequence.

- **`voyager-core`** (`crates/voyager-core`) — a dependency-free tokenizer and
  structural parser, plus a whitespace/casing/indentation formatter and a
  keyword-completion/spell-check dictionary built on top of the same structure.
  See [`specs/001-voyager-script-parser/`](specs/001-voyager-script-parser/) for
  the parser's spec/plan/data-model/contracts/tasks/checklists,
  [`specs/002-cli-check-format/`](specs/002-cli-check-format/)'s `research.md`/
  `data-model.md` for the formatter additions (`format`/`format_bytes`,
  `Block.closer`/`opener_pairs`), and
  [`specs/003-lsp-vscode-extension/`](specs/003-lsp-vscode-extension/) for the
  `keywords` module (`completion_candidates`/`did_you_mean`) layered onto it after
  that. Its `PairKeyword` dictionary (197 keyword names, corpus-census-derived
  per FR-012) was populated 2026-08-10 against the full 161-file corpus — see
  the module's own doc comment for the census methodology and the two filters
  applied (and why). The census's own first pass surfaced a real
  `pair_keyword_boundaries` parsing defect (quote-unawareness inside a
  `Control` statement's keyword-list scan); fixed the same day per
  `specs/001-voyager-script-parser/spec.md`'s FR-003 amendment, and the
  census re-run against the fix — see the module doc for the fix's effect on
  the dictionary (one entry, `COST`/`PRINT`, dropped as the bug's own
  artifact; everything else unchanged). Since then, the formatter has grown: a
  `--top-level-indent` mode (`preserve`, the default, or `auto` — force
  every top-level statement to column 0) and three independent, three-valued
  casing fields (`control_words_casing`/`pair_keywords_casing`/
  `data_references_casing`; each `preserve`, the default, `upper`, or
  `lower`) — see
  [`specs/009-top-level-indent-toggle/`](specs/009-top-level-indent-toggle/),
  [`specs/014-casing-preserve-mode/`](specs/014-casing-preserve-mode/), and
  [`specs/017-casing-categories-indent-width/`](specs/017-casing-categories-indent-width/)
  (a prior flat `casing` field, covering `control_words`+`pair_keywords`
  together, was later removed once these granular fields fully superseded
  it — see `CHANGELOG.md`); a
  `; FMT: OFF`/`; FMT: ON` inline marker pair to exclude a range from
  formatting entirely, with an unclosed marker surfaced (never silently
  extended forever without notice) — see
  [`specs/010-fmt-region-markers/`](specs/010-fmt-region-markers/); a new
  `UnmatchedProcess` diagnostic for an unclosed `PROCESS`/`PHASE` block,
  previously unflagged — see
  [`specs/006-unmatched-process-diagnostic/`](specs/006-unmatched-process-diagnostic/);
  and a fix so a genuinely unmatched/diagnosed block's children no longer pick up
  stale indentation.
- **`drut-config`** (`crates/drut-config`) — `drut.toml` discovery (upward
  directory walk from the file being processed, stopping at a `.git` boundary or
  filesystem root), TOML parsing, and per-field resolution
  (`defaults < drut.toml < explicit CLI-flag/MCP-param`), shared identically by
  `drut-cli`/`drut-lsp`/`drut-mcp` — the only way an LSP/editor user reaches
  non-default formatting behavior without a CLI flag. A malformed value in the
  file never blocks formatting; it warns and falls back to the built-in default
  for just that field. See the ["Configuration"](#configuration) section below
  and [`specs/012-toml-configuration/`](specs/012-toml-configuration/) for the
  full design.
- **`drut-cli`** (`crates/drut-cli`, binary `drut`) — a thin CLI adapter exposing
  `check`, `format`, `server`, and `mcp` as subcommands over
  `voyager-core`/`drut-lsp`/`drut-mcp`, per
  [`specs/002-cli-check-format/`](specs/002-cli-check-format/),
  [`specs/003-lsp-vscode-extension/`](specs/003-lsp-vscode-extension/), and
  [`specs/004-mcp-server/`](specs/004-mcp-server/). `check` is fully wired
  (plain-text or SARIF 2.1.0 output); `format` supports default/`--write`/
  `--check`/`--diff` disposition modes, opt-in
  `--control-words-casing=preserve|upper|lower` (and its
  `--pair-keywords-casing`/`--data-references-casing` siblings),
  `--top-level-indent=preserve|auto`, and `--isolated` (skip `drut.toml`
  discovery for one run); `server` speaks the Language Server Protocol over
  stdio; `mcp` speaks the Model Context Protocol over stdio.
- **`drut-lsp`** (`crates/drut-lsp`) — a thin LSP adapter over `voyager-core`:
  diagnostics (seven categories, including `drut.toml`-problem hints and
  unclosed-`FMT:-OFF` hints — `InvalidEncoding` is unreachable through live
  editing by construction of the LSP transport itself, see
  `specs/003-lsp-vscode-extension/research.md` §12), hover (block-kind and
  matched-counterpart info, including through `Run`/`Process`'s implicit-close
  quirk), control-word-scoped completion, "did you mean" spell-check (riding on
  hover), semantic tokens (short-`IF` vs block-`IF`, unreachable-after-`BREAK`),
  document/range formatting, and folding ranges for every block kind and block
  comment — see [`specs/011-code-folding/`](specs/011-code-folding/). Since
  `013-lsp-config-file-watch`, it also registers a
  `workspace/didChangeWatchedFiles` watcher for `drut.toml` (when the client
  supports dynamic registration) and live-refreshes every open document's
  diagnostics on a config change, without requiring the document to be
  closed/reopened — see
  [`specs/013-lsp-config-file-watch/`](specs/013-lsp-config-file-watch/). See
  [`specs/003-lsp-vscode-extension/`](specs/003-lsp-vscode-extension/) for the
  original spec/plan/data-model/contracts/tasks.
- **`editors/vscode`** — a VS Code/Open VSX extension: a static TextMate grammar
  (functional with zero dependency on `drut server` running) plus a
  `vscode-languageclient` wrapper spawning `drut server`, with graceful
  degradation (highlighting-only) when the binary is missing and a one-restart
  crash-recovery policy when the server process dies mid-session. Since
  `005-format-on-save-paste`: format-on-save auto-enables itself on first
  activation (one-time, removal-respecting), and format-on-paste is available
  opt-in — see ["Editor behavior"](#editor-behavior-format-on-save-and-format-on-paste)
  below.
- **`drut-mcp`** (`crates/drut-mcp`) — a thin Model Context Protocol adapter over
  `voyager-core`, the fourth thin adapter the constitution names: four read-only
  tools (`diagnose`, `format`, `query_structure`, `lookup_keyword`), exposed via
  `drut mcp` over stdio. `query_structure` shares its block-kind/matched-
  counterpart derivation with `drut-lsp`'s hover capability through a single,
  common `voyager-core::block_at` entry point (extracted from `drut-lsp` for
  this feature, `specs/004-mcp-server/research.md` §5) — genuinely one
  implementation behind two adapters, not two independently-maintained copies.
  `tokio`/`rmcp` (this project's first async runtime dependency — every
  actively-maintained Rust MCP SDK is async-only) are scoped entirely to this
  one crate; `voyager-core`, `drut-cli`'s other subcommands, and `drut-lsp`
  remain fully synchronous. `format`'s parameters now mirror the CLI's flags
  (`control_words_casing`, `top_level_indent`, `isolated`), plus response fields for unclosed
  `; FMT: OFF` markers and `drut.toml` warnings. See
  [`specs/004-mcp-server/`](specs/004-mcp-server/) for the full spec/plan/
  data-model/contracts/tasks.

Build/test everything:

```powershell
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
```

Try the CLI:

```powershell
cargo run -p drut-cli --bin drut -- check path\to\some.s
cargo run -p drut-cli --bin drut -- format path\to\some.s --diff
cargo run -p drut-cli --bin drut -- server   # speaks LSP over stdio; launched by an editor, not run interactively
cargo run -p drut-cli --bin drut -- mcp      # speaks MCP over stdio; launched by an MCP client, not run interactively
```

Full-corpus validation (161 real `.s`/`.block` files) is gated behind a
`DRUT_CORPUS_PATH` env var and `#[ignore]`'d by default, since that corpus is
external and not committed (licensing still an open item — see
`001-voyager-script-parser/research.md` §3):

```powershell
$env:DRUT_CORPUS_PATH = "path\to\WF-TDM-Official-Releases"
cargo test -p drut-cli --test fixture_corpus_e2e -- --ignored
cargo test -p drut-lsp --test diagnostics_corpus -- --ignored
cargo test -p drut-mcp --test diagnostics_corpus -- --ignored
cargo test -p drut-cli --test structural_query_parity -- --ignored   # drut-mcp vs. drut-lsp parity; lives here since drut-mcp can't depend on drut-lsp (FR-011)
```

Build/test the VS Code extension:

```powershell
cd editors\vscode
npm install
npm run compile
npm test           # grammar tokenization spot-checks (vscode-textmate, no VS Code needed)
npx @vscode/vsce package   # produces a .vsix — see Publishing below
```

## Editor behavior and configuration (user-facing — moved)

Format-on-save/format-on-paste, editor client settings, and the full
`drut.toml` `[format]` field reference now live in the published user guide,
not here: see [Editor Guide](https://ar-puuk.github.io/drut-dev/editor-guide.html)
and [Configuration Reference](https://ar-puuk.github.io/drut-dev/configuration-reference.html)
(`022-docs-site`). This section used to carry that content directly and, over
several formatting features, fell behind — it documented only 2 of the
eventual 10 real `[format]` fields. Don't let this section's replacement
happen again: **any change that adds/removes a `[format]` field, a CLI flag,
an MCP tool, or LSP-visible behavior updates the corresponding `docs-site/`
page as part of that same change** (spec.md FR-011,
`specs/022-docs-site/`) — not a follow-up, not optional.

## Repository layout

```text
.specify/          Spec-kit workflow tooling (templates, scripts, constitution)
specs/             Per-feature spec-kit artifacts (spec/plan/tasks/contracts/...)
crates/
  voyager-core/    Tokenizer, structural parser, formatter, and keyword
                   dictionary — zero runtime dependencies (constitution
                   Principle I, FR-027)
  drut-config/     drut.toml discovery, parsing, and per-field resolution —
                   shared by drut-cli/drut-lsp/drut-mcp (012-toml-configuration)
  drut-cli/        `drut` binary: check/format/server subcommands, thin
                   adapter over voyager-core/drut-lsp (traversal, I/O,
                   output rendering, stdio transport only)
  drut-lsp/        LSP server library (diagnostics/hover/completion/
                   spell-check/semantic tokens), thin adapter over
                   voyager-core — wired into `drut server`, not its own binary
  drut-mcp/        MCP server library (diagnose/format/query_structure/
                   lookup_keyword — all read-only), thin adapter over
                   voyager-core — wired into `drut mcp`, not its own binary
editors/
  vscode/          VS Code/Open VSX extension: static TextMate grammar +
                   language-configuration.json (no server dependency) plus a
                   vscode-languageclient wrapper spawning `drut server`
_archive/          Local-only vendor documentation mirrors, gitignored — never
                   committed; kept for reference during grammar research only
                   (see constitution Principle II / Principle VIII)
```

## Publishing the VS Code/Open VSX extension

Packaging (`@vscode/vsce package`) and Open VSX validation are part of this
project's own build/test loop (see above); actually publishing to the VS Code
Marketplace and Open VSX is a maintainer-run release action, not something CI or
an agent runs automatically (`specs/003-lsp-vscode-extension/spec.md`
Assumptions):

```powershell
cd editors\vscode
npx @vscode/vsce publish   # requires a Marketplace publisher token
npx ovsx publish           # requires an Open VSX access token
```

Both under Drut's own publisher identity (`arpuuk` in `package.json`) —
never a fork or rebrand of any third-party extension (FR-027).

## Versioning

`voyager-core`, `drut-config`, `drut-cli`, `drut-lsp`, `drut-mcp`, and the
VS Code/Open VSX extension are versioned in **lockstep**: every release bumps
all six to the same version number together, never independently — there is
no scenario where, say, `drut-lsp` is at a different version than `drut-cli`
or the extension. The first publish uses `0.1.0`, not `1.0.0` — this is a
genuine first release, and semver's own convention for a pre-stable public
API fits that honestly. See [`CHANGELOG.md`](CHANGELOG.md) for what's shipped so far (grouped under
"Unreleased" until a version is actually tagged, at which point that section
becomes that version's own dated entry).

## Dependency auditing

`voyager-core` has zero runtime dependencies by design (constitution Principle I,
FR-027) — `cargo tree -p voyager-core` should never show an external crate.
`drut-cli`, `drut-lsp`, and `drut-mcp` are not bound by that rule (see
`002-cli-check-format/spec.md` Assumptions) and depend on ordinary ecosystem
crates — `drut-cli` on `clap`, `ignore`, `serde`, `serde_json`, `similar`,
`lsp-server` for traversal/argument-parsing/output-rendering/stdio-transport
concerns; `drut-lsp` on `lsp-server`/`lsp-types` for the JSON-RPC protocol layer;
`drut-mcp` on `rmcp` (the official Model Context Protocol SDK, pinned `~3.1`),
`tokio` (this project's first async runtime — every actively-maintained Rust MCP
SDK requires one, `specs/004-mcp-server/research.md` §1/§2 — scoped entirely to
this one crate, verified via `cargo tree`), and `schemars` — with no
grammar/parsing content in any of them. Their versions were confirmed free of
known RUSTSEC advisories as of 2026-08-09 (`002-cli-check-format/research.md` §6
for the original set; `003-lsp-vscode-extension/research.md` §11 for
`lsp-server`/`lsp-types`) and 2026-08-10 for `rmcp`/`tokio`/`schemars`
(`004-mcp-server/research.md` §4 — one real advisory found and confirmed
inapplicable, `RUSTSEC-2026-0189`, a DNS-rebinding issue in `rmcp`'s Streamable
HTTP server transport: already patched at the pinned version, and structurally
unreachable regardless since `drut-mcp`'s `Cargo.toml` never enables that
feature — verified at the actual `#[cfg(feature = ...)]` source level, not just
inferred from the dependency graph), but that's a point-in-time check, not a
standing guarantee — run [`cargo audit`](https://github.com/rustsec/rustsec) (or
`cargo deny check advisories`) periodically, and wire it into CI once one
exists, so an advisory filed after that date surfaces automatically.
`editors/vscode`'s npm dependencies (`vscode-languageclient` and its own
transitive tree, plus the `vscode-textmate`/`vscode-oniguruma`/`ts-node`
devDependencies used by the grammar test) were confirmed free of known
vulnerabilities via `npm audit` as of 2026-08-10 (0 found) — again a
point-in-time check, not a standing guarantee; re-run periodically and wire into
CI once one exists.

## Credits

Phase 3 (LSP server + editor extension) references the structure of Bill Hereth's VS
Code extension, [`language-citilabscubevoyager`](https://github.com/WFRCAnalytics/Resources/tree/master/7-Other/VSCode-Extensions/bhereth.language-citilabscubevoyager)
(GitHub: [@bhereth](https://github.com/bhereth)) — permission granted 2026-08-08 4:06
PM MT, for structure/behavior reference with credit, not a license to redistribute
his files or copy his keyword lists/grammar text verbatim (see constitution
[Principle II](.specify/memory/constitution.md#ii-no-verbatim-redistribution-of-vendor-documentation)
for the full binding conditions). His extension's files are never committed to this
repository; his `LICENSE.txt` grants no redistribution rights independent of that
permission, so anything ported from it — language IDs, bracket-matching
configuration, comment delimiters, TextMate scope-naming conventions — is rewritten
in Drut's own structure and wording.

## Workflow

This project follows the [spec-kit](https://github.com/github/spec-kit) discipline:
every feature gets a spec, a plan, and a task list before implementation starts, and
a feature's fixture-corpus tests must pass cleanly before the next phase begins
(constitution Principle V). See `.specify/` for the slash-command workflow
(`/speckit-specify`, `/speckit-clarify`, `/speckit-plan`, `/speckit-tasks`,
`/speckit-checklist`, `/speckit-analyze`, `/speckit-implement`).

A feature that adds, removes, or changes a `[format]` config field, a CLI flag,
an MCP tool, or LSP-visible behavior updates the corresponding
[user-guide page](https://ar-puuk.github.io/drut-dev/) (`docs-site/src/`) as
part of that same change — see the "Editor behavior and configuration" section
above for why this is a hard requirement, not a nice-to-have.
