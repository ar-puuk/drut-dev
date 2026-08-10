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

Three features shipped, all with passing fixture-corpus test gates (constitution
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
```

Full-corpus validation (161 real `.s`/`.block` files) is gated behind a
`DRUT_CORPUS_PATH` env var and `#[ignore]`'d by default, since that corpus is
external and not committed (licensing still an open item — see
`001-voyager-script-parser/research.md` §3):

```powershell
$env:DRUT_CORPUS_PATH = "path\to\WF-TDM-Official-Releases"
cargo test -p drut-cli --test fixture_corpus_e2e -- --ignored
cargo test -p drut-lsp --test diagnostics_corpus -- --ignored
```

Build/test the VS Code extension:

```powershell
cd editors\vscode
npm install
npm run compile
npm test           # grammar tokenization spot-checks (vscode-textmate, no VS Code needed)
npx @vscode/vsce package   # produces a .vsix — see Publishing below
```

## Repository layout

```text
.specify/          Spec-kit workflow tooling (templates, scripts, constitution)
specs/             Per-feature spec-kit artifacts (spec/plan/tasks/contracts/...)
crates/
  voyager-core/    Tokenizer, structural parser, formatter, and keyword
                   dictionary — zero runtime dependencies (constitution
                   Principle I, FR-027)
  drut-cli/        `drut` binary: check/format/server subcommands, thin
                   adapter over voyager-core/drut-lsp (traversal, I/O,
                   output rendering, stdio transport only)
  drut-lsp/        LSP server library (diagnostics/hover/completion/
                   spell-check/semantic tokens), thin adapter over
                   voyager-core — wired into `drut server`, not its own binary
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

## Dependency auditing

`voyager-core` has zero runtime dependencies by design (constitution Principle I,
FR-027) — `cargo tree -p voyager-core` should never show an external crate.
`drut-cli` and `drut-lsp` are not bound by that rule (see `002-cli-check-format/
spec.md` Assumptions) and depend on ordinary ecosystem crates — `drut-cli` on
`clap`, `ignore`, `serde`, `serde_json`, `similar`, `lsp-server` for traversal/
argument-parsing/output-rendering/stdio-transport concerns; `drut-lsp` on
`lsp-server`/`lsp-types` for the JSON-RPC protocol layer — with no grammar/parsing
content in either. Their versions were confirmed free of known RUSTSEC advisories
as of 2026-08-09 (`002-cli-check-format/research.md` §6 for the original set;
`003-lsp-vscode-extension/research.md` §11 for `lsp-server`/`lsp-types`), but
that's a point-in-time check, not a standing guarantee — run
[`cargo audit`](https://github.com/rustsec/rustsec) (or `cargo deny check
advisories`) periodically, and wire it into CI once one exists, so an advisory
filed after that date surfaces automatically. `editors/vscode`'s npm dependencies
(`vscode-languageclient` and its own transitive tree, plus the `vscode-textmate`/
`vscode-oniguruma`/`ts-node` devDependencies used by the grammar test) were
confirmed free of known vulnerabilities via `npm audit` as of 2026-08-10 (0
found) — again a point-in-time check, not a standing guarantee; re-run
periodically and wire into CI once one exists.

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
