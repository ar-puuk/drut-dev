# Quickstart: Validating the Voyager Script Tokenizer & Structural Parser

This is a validation/run guide, not an implementation guide — it proves the feature
works end-to-end once built. Implementation steps live in `tasks.md`.

## Prerequisites

- Rust stable toolchain (2021 edition) installed (`rustc --version`, `cargo --version`).
- Repo cloned; workspace `Cargo.toml` and `crates/voyager-core` exist (see plan.md
  Project Structure).
- Fixture corpus present under `crates/voyager-core/tests/fixtures/{valid,broken}/`
  (see research.md § 3 for sourcing/licensing status — this must be resolved before
  a real corpus lands, but hand-written structural-shape fixtures are sufficient to
  exercise this guide in the meantime).

## Setup

```powershell
# from repo root
cargo build -p voyager-core
```

Expect a clean build with **zero** dependencies pulled in beyond `std` (per
research.md § 1) — `cargo tree -p voyager-core` should show no external crates.

## Scenario 1 — Parse a valid script into structure (User Story 1, P1)

```powershell
cargo test -p voyager-core --test fixture_corpus -- valid
```

**Expected outcome**: every fixture under `tests/fixtures/valid/` parses via `parse()`
(contracts/public-api.md) with an empty `diagnostics` list (SC-001). Any fixture
producing a diagnostic here is a bug in the parser (a false positive), not a bad
fixture — per constitution Principle IV, this is exactly the failure mode the corpus
gate exists to catch.

## Scenario 2 — Get precise diagnostics for a broken script (User Story 2, P2)

```powershell
cargo test -p voyager-core --test fixture_corpus -- broken
```

**Expected outcome**: every fixture under `tests/fixtures/broken/` produces at least
one `Diagnostic` whose `kind` matches the defect deliberately injected into that
fixture's filename/manifest entry (SC-002, SC-003 — one fixture per row in
contracts/diagnostics.md). No panics, no hangs (FR-018).

Manual spot-check (no test harness needed) for a single file:

```powershell
cargo run -p voyager-core --example parse_file -- path\to\some.s
```

(An `examples/parse_file.rs` that reads a path via `std::fs`, calls `parse()`, and
prints the resulting nodes/diagnostics is a reasonable implementation task — the
library itself still never touches the filesystem; only this example binary does.)

## Scenario 3 — Token-level detail for editor-style features (User Story 3, P3)

```powershell
cargo test -p voyager-core --test fixture_corpus -- token_detail
```

**Expected outcome**: for a fixture containing a line comment after real content, a
multi-line block comment, and an `@variable@` reference split across a continuation,
`tokenize()` returns distinct tokens with correct spans and, for `@variable@`, the
captured variable name — without needing to call `parse()` at all.

## Definition-of-done check

```powershell
cargo test -p voyager-core
```

All of the above scenarios, plus ordinary unit tests, pass. This is the fixture-corpus
gate constitution Principle V requires before any later phase (CLI, LSP, MCP,
formatter) may begin.

## Cleanup

Nothing persistent is created — no files are written, no services started. Nothing to
clean up.
