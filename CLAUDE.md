# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Drut is a tokenizer, structural parser, and (eventually) linter/formatter/LSP+MCP
tooling suite for Cube Voyager control-statement scripts (`.s` / `.block` files),
targeting Cube Voyager 6.5 as the grammar baseline. See `README.md` for the repo
layout and `.specify/memory/constitution.md` for the full governing principles —
read the constitution before making architectural decisions; it is binding, not
advisory.

**Current state**: pre-implementation. No Rust code exists yet. The repository is
entirely spec-kit planning artifacts for the first feature (`voyager-core`, the core
tokenizer/parser crate) under `specs/001-voyager-script-parser/`.

## Commands

### Spec-kit workflow (how features move from idea to code here)

This project is driven by spec-kit slash commands, not ad hoc coding. A feature gets
a spec, a plan, and a task list — with a passing fixture-corpus test gate — before
the next phase starts (constitution Principle V). In order:
`/speckit-specify` → `/speckit-clarify` → `/speckit-plan` → `/speckit-tasks` →
`/speckit-checklist` / `/speckit-analyze` (pre-implementation quality gates) →
`/speckit-implement`.

### Rust (once `crates/voyager-core` exists — see plan.md Project Structure)

```powershell
cargo build -p voyager-core
cargo test -p voyager-core                              # full suite
cargo test -p voyager-core --test fixture_corpus -- valid       # US1: structure on valid scripts
cargo test -p voyager-core --test fixture_corpus -- broken      # US2: diagnostics on broken scripts
cargo test -p voyager-core --test fixture_corpus -- token_detail # US3: token-level detail
cargo clippy -p voyager-core                             # must be zero-warning before merge
cargo run -p voyager-core --example parse_file -- path\to\some.s  # manual spot-check
```

`voyager-core` has **zero runtime dependencies** — only `std` (FR-027). Don't add a
crate to fix a parsing problem; hand-write it. `cargo tree -p voyager-core` should
never show an external dependency.

## Architecture

### Single source of truth (constitution Principle I)

All Voyager grammar, parsing, and lint-rule logic lives in exactly one place: the
`voyager-core` Rust crate. Every other surface — CLI, LSP server, MCP server,
formatter, editor extension — is a thin adapter that calls into this crate and MUST
NOT re-implement or duplicate any grammar/parsing logic itself. If you're about to
write parsing logic anywhere outside `crates/voyager-core`, stop — it belongs there
instead, with the adapter just consuming its output.

`voyager-core`'s public contract (see `specs/001-voyager-script-parser/contracts/
public-api.md`) is two pure functions operating on in-memory text only —
`tokenize(source: &str) -> Vec<Token>` and `parse(source: &str) -> ParseResult` —
with no file I/O, network access, or protocol dependency inside the crate itself.
Neither function may panic on any input, including malformed input; defects surface
as `Diagnostic` values in the result, never as a panic or `Err`.

### Grammar model (see `data-model.md` and `contracts/diagnostics.md`)

The parser recognizes four statement forms (`Control`, `Assignment`, `Label`,
`ShellEscape`) and seven block kinds (`If` — including a self-closing single-line
short-`IF` form, `Loop`, `Run` — including a `!RUN`-disabled variant, `Process`
a.k.a. `PHASE`/`ENDPHASE`, `JLoop`, `LinkLoop`, `DistributeMultistep`), all matched
structurally with no per-program semantic validation. Two block families (`Run`,
`Process`) close *implicitly* (by the next same-family opener, or for `Run` a
shell-escape statement) as well as explicitly — this is a documented Voyager
grammar quirk, not a parser bug, and the block-matching implementation has to
account for it. Only six diagnostic categories exist (`UnmatchedIf`, `UnmatchedLoop`,
`UnclosedBlockComment`, `InvalidContinuation`, `UnmatchedRun`, `MisplacedBreak`) —
the other four block kinds are matched structurally but intentionally produce no
diagnostic if left unmatched (see `contracts/diagnostics.md`'s note on this).

Every grammar rule's rationale and evidence trail lives in
`specs/001-voyager-script-parser/spec.md` (Functional Requirements + Assumptions) —
read the relevant FR before changing parsing behavior, since many rules encode a
specific real-world finding (either from a fixture corpus or from a vendor
documentation cross-check) that isn't obvious from the code alone.

### No verbatim vendor documentation (constitution Principle II — strict)

Grammar rules, keyword lists, diagnostic messages, and any hover/help text MUST be
written in the project's own words. Never copy phrasing from Bentley/Citilabs Cube
Voyager documentation, even when researching a rule against it. `_archive/` (if
present locally) holds vendor documentation mirrors for research only — it is
gitignored, must never be committed, and its content must never be imported
verbatim into source, comments, or generated fixtures (Principle VIII: it's a
private-repo concern, not a public-repo one).

### Fixture corpus is the correctness oracle

There is no synthetic "expected output" — correctness is measured against a fixture
corpus of real `.s`/`.block` scripts (valid and deliberately broken), per
constitution Principle IV: a parser MUST produce zero false-positive diagnostics on
valid scripts (an unflagged bug is forgivable; a false flag on working code is not)
and MUST correctly flag every deliberately-broken fixture. The real corpus's
sourcing/licensing is an open item (see `research.md` §3) — until resolved,
hand-written fixtures that reproduce structural *shapes* (not verbatim third-party
script content) stand in for it under `crates/voyager-core/tests/fixtures/`.

### Formatter constraints (future phase, but binding now for anyone touching it)

A formatter, when built, MUST be idempotent (`format(format(x)) == format(x)`) and
MUST NOT change which lines are continuations, reorder statements, or alter program
meaning — only whitespace and, optionally, keyword casing. Every formatter change
requires a golden-file diff against the fixture corpus before merge.
