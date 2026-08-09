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

Two features shipped, both with passing fixture-corpus test gates (constitution
Principle V):

- **`voyager-core`** (`crates/voyager-core`) — a dependency-free tokenizer and
  structural parser, plus a whitespace/casing formatter built on top of the same
  structure. See [`specs/001-voyager-script-parser/`](specs/001-voyager-script-parser/)
  for the parser's spec/plan/data-model/contracts/tasks/checklists, and
  [`specs/002-cli-check-format/`](specs/002-cli-check-format/)'s `research.md`/
  `data-model.md` for the formatter additions (`format`/`format_bytes`,
  `Block.closer`/`opener_pairs`) layered onto it afterward.
- **`drut-cli`** (`crates/drut-cli`, binary `drut`) — a thin CLI adapter exposing
  `check` and `format` as subcommands over `voyager-core`, per
  [`specs/002-cli-check-format/`](specs/002-cli-check-format/). `check` is fully
  wired (plain-text or SARIF 2.1.0 output); `format` supports default/`--write`/
  `--check`/`--diff` disposition modes and opt-in `--casing=upper|lower`.

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
```

Full-corpus validation (161 real `.s`/`.block` files) is gated behind a
`DRUT_CORPUS_PATH` env var and `#[ignore]`'d by default, since that corpus is
external and not committed (licensing still an open item — see
`001-voyager-script-parser/research.md` §3):

```powershell
$env:DRUT_CORPUS_PATH = "path\to\WF-TDM-Official-Releases"
cargo test -p drut-cli --test fixture_corpus_e2e -- --ignored
```

## Repository layout

```text
.specify/          Spec-kit workflow tooling (templates, scripts, constitution)
specs/             Per-feature spec-kit artifacts (spec/plan/tasks/contracts/...)
crates/
  voyager-core/    Tokenizer, structural parser, and formatter — zero runtime
                   dependencies (constitution Principle I, FR-027)
  drut-cli/        `drut` binary: check/format subcommands, thin adapter over
                   voyager-core (traversal, I/O, output rendering only)
_archive/          Local-only vendor documentation mirrors, gitignored — never
                   committed; kept for reference during grammar research only
                   (see constitution Principle II / Principle VIII)
```

## Dependency auditing

`voyager-core` has zero runtime dependencies by design (constitution Principle I,
FR-027) — `cargo tree -p voyager-core` should never show an external crate.
`drut-cli` is not bound by that rule (see `002-cli-check-format/spec.md`
Assumptions) and depends on ordinary ecosystem crates (`clap`, `ignore`, `serde`,
`serde_json`, `similar`) for traversal/argument-parsing/output-rendering concerns
with no grammar/parsing content. Their versions were confirmed free of known
RUSTSEC advisories as of 2026-08-09 (`002-cli-check-format/research.md` §6), but
that's a point-in-time check, not a standing guarantee — run
[`cargo audit`](https://github.com/rustsec/rustsec) (or `cargo deny check
advisories`) periodically, and wire it into CI once one exists, so an advisory
filed after that date against `drut-cli`'s dependencies (or their own transitive
trees, which weren't inspectable at pin time since no `Cargo.lock` existed yet)
surfaces automatically.

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
