# Drut

Drut is a tokenizer, structural parser, and (eventually) linter/formatter/LSP+MCP
tooling suite for **Cube Voyager control-statement scripts** (`.s` / `.block`
files), targeting Cube Voyager 6.5 as its grammar baseline.

The project is architected as one authoritative Rust core crate — grammar, parsing,
and lint-rule logic — with thin adapters on top (CLI, LSP server, MCP server,
formatter, editor extension client). See [`.specify/memory/constitution.md`](.specify/memory/constitution.md)
for the full set of governing principles (single source of truth, no verbatim
vendor-documentation redistribution, formatter behavior-preservation, false
negatives preferred over false positives, vertical phase-gated delivery, and more).

## Status

Four features shipped, all with passing fixture-corpus test gates (constitution
Principle V):

- **`voyager-core`** (`crates/voyager-core`) — a dependency-free tokenizer and
  structural parser, plus a whitespace/casing formatter and a keyword-completion/
  spell-check dictionary built on top of the same structure. See
  [`specs/001-voyager-script-parser/`](specs/001-voyager-script-parser/) for the
  parser's spec/plan/data-model/contracts/tasks/checklists,
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
  artifact; everything else unchanged).
- **`drut-cli`** (`crates/drut-cli`, binary `drut`) — a thin CLI adapter exposing
  `check`, `format`, and `server` as subcommands over `voyager-core`/`drut-lsp`, per
  [`specs/002-cli-check-format/`](specs/002-cli-check-format/) and
  [`specs/003-lsp-vscode-extension/`](specs/003-lsp-vscode-extension/). `check` is
  fully wired (plain-text or SARIF 2.1.0 output); `format` supports default/
  `--write`/`--check`/`--diff` disposition modes and opt-in `--casing=upper|lower`;
  `server` speaks the Language Server Protocol over stdio.
- **`drut-lsp`** (`crates/drut-lsp`) — a thin LSP adapter over `voyager-core`:
  diagnostics (six of seven `voyager-core` categories — `InvalidEncoding` is
  unreachable through live editing by construction of the LSP transport itself,
  see `specs/003-lsp-vscode-extension/research.md` §12), hover (block-kind and
  matched-counterpart info, including through `Run`/`Process`'s implicit-close
  quirk), control-word-scoped completion, "did you mean" spell-check (riding on
  hover), and semantic tokens (short-`IF` vs block-`IF`, unreachable-after-`BREAK`).
  See [`specs/003-lsp-vscode-extension/`](specs/003-lsp-vscode-extension/) for the
  full spec/plan/data-model/contracts/tasks.
- **`editors/vscode`** — a VS Code/Open VSX extension: a static TextMate grammar
  (functional with zero dependency on `drut server` running) plus a
  `vscode-languageclient` wrapper spawning `drut server`, with graceful
  degradation (highlighting-only) when the binary is missing and a one-restart
  crash-recovery policy when the server process dies mid-session.
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
  remain fully synchronous. See
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

## Editor behavior: format-on-save and format-on-paste

`005-format-on-save-paste` adds two automatic-reformatting behaviors on top
of the extension's existing "Format Document" command:

- **Format-on-save** is auto-enabled the first time the extension activates
  in a workspace (workspace-scoped, one-time, and never silently
  re-enabled if you turn it back off — see
  [`specs/005-format-on-save-paste/`](specs/005-format-on-save-paste/) for
  the full mechanism). No action needed to use it; saving a `.s`/`.block`
  file reformats it automatically, the same result "Format Document" would
  already produce.
- **Format-on-paste** stays off by default — turn it on yourself by adding
  the following to your workspace's `.vscode/settings.json`:

  ```json
  {
    "[drut-voyager]": {
      "editor.formatOnPaste": true
    }
  }
  ```

  Once enabled, pasting Cube Voyager script text into a `.s`/`.block` file
  reindents it to match its new surrounding structure immediately after
  the paste.

## Configuration

`012-toml-configuration` lets a project set shared defaults for drut's
formatting behavior once, in a `drut.toml` file, instead of every user
having to pass the same flags every time — and, unlike CLI flags, this is
the only way an editor (LSP) user reaches non-default behavior at all.

**Schema** — a `[format]` table, currently the only table:

```toml
[format]
casing = "lower"                 # "upper" | "lower"; omit = leave casing untouched
top_level_indent = "normalize"   # "preserve" | "normalize"; omit = "preserve"
```

Omitting a key means "use the built-in default for that setting" — there is
no separate "off" value to remember.

**Discovery**: for any file being formatted, drut searches upward from that
file's own directory for the nearest `drut.toml`, the same way on every
surface (CLI, editor, MCP). The search stops at the first `drut.toml`
found, at a `.git` boundary (so a config file from an unrelated parent
project is never picked up by accident), or at the filesystem root —
whichever comes first. A project with no `drut.toml` anywhere behaves
identically to every version of drut before this feature existed.

**Precedence**, per setting, independently: an explicit value passed for
one call (a CLI flag, or an MCP tool parameter) always wins; otherwise the
resolved `drut.toml`'s value applies, if it sets that key; otherwise the
built-in default applies.

**Isolation**: pass `--isolated` (CLI) or `isolated: true` (the MCP
`format` tool) to skip `drut.toml` discovery entirely for one run, using
built-in defaults plus any other explicit flags/parameters — useful for CI
reproducibility or a one-off sanity check.

**A malformed `drut.toml` never blocks formatting.** A problem with one
setting (an unrecognized key, or an invalid value) only affects that one
setting — every other valid setting in the same file still applies, and
formatting completes normally using the built-in default for whatever
couldn't be resolved. The problem is always surfaced, never silent: a
notice on stderr (CLI), a `HINT`-severity diagnostic with a distinct
`drut-config` source (editor/LSP), or a `config_warnings` field in the
response (MCP's `format` tool) — never a change to the CLI's exit code, and
never a reason a file fails to format.

See [`specs/012-toml-configuration/`](specs/012-toml-configuration/) for
the full design rationale.

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

Both under Drut's own publisher identity (`drut-project` in `package.json`) —
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
