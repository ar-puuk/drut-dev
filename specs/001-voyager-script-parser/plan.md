# Implementation Plan: Voyager Script Tokenizer & Structural Parser

**Branch**: `001-voyager-script-parser` | **Date**: 2026-08-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-voyager-script-parser/spec.md`

## Summary

Build a dependency-free Rust library crate (`voyager-core`) that tokenizes and
structurally parses Cube Voyager control-statement scripts (`.s`/`.block`). It
recognizes control statements, plain assignments, label statements, shell-escape
statements, comments, line continuations, `@variable@` references, and nested
`IF`/`ELSEIF`/`ELSE`/`ENDIF`, `LOOP`/`ENDLOOP`/`BREAK`, and `RUN PGM=.../ENDRUN`
blocks — case-insensitively — and reports structured, non-panicking diagnostics for
the six required defect categories. The crate takes only in-memory source text and
performs no file I/O, network access, or protocol-specific work, so it can sit
underneath a future CLI, LSP server, MCP server, and formatter (per constitution
Principle I) without any of them re-implementing grammar logic.

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition. No nightly-only features;
nothing in this feature needs them.

**Primary Dependencies**: None at runtime. The core crate uses only `std` — no lexer-
generator, parser-combinator, or error-derive crate. This keeps the single
authoritative grammar implementation (Principle I) minimal, dependency-auditable, and
free of a dependency's own opinions about error types or tokenization strategy leaking
into the one place all adapters rely on. Test-only tooling (see Testing) may add a
dev-dependency if it measurably simplifies the fixture-corpus harness; none is known to
be needed yet.

**Storage**: N/A — the library holds no persistent state; callers own the source text
and the returned parse result.

**Testing**: `cargo test`, plus a fixture-corpus integration test (`tests/
fixture_corpus.rs`) that walks `tests/fixtures/valid/**` and `tests/fixtures/broken/**`
and asserts zero false-positive diagnostics on the former and a correctly-categorized
diagnostic on the latter, per SC-001/SC-002/SC-003.

**Target Platform**: Cross-platform — anywhere Rust's `std` runs (Windows, macOS,
Linux). No OS-specific APIs; the source scripts this parser targets are themselves
authored on Windows but the parser itself has no platform dependency.

**Project Type**: Library — a single Rust crate, laid out inside a new Cargo workspace
so that later phases (CLI, LSP server, MCP server, formatter — per constitution
Technology & Architecture Constraints) can be added as sibling workspace members
without restructuring this one.

**Performance Goals**: Single-pass, O(n) tokenization and structural parsing over
input length, no backtracking over the whole file. Parsing a several-thousand-line
script (the observed size of real fixtures, e.g. `HBW_HBO_calculate_utilities.block`
at ~980 lines) should complete in low tens of milliseconds on ordinary developer
hardware, since a later LSP consumer will need to re-parse on every keystroke-adjacent
edit.

**Constraints**:
- MUST NOT panic on malformed input — every defect surfaces as a `Diagnostic`, never
  an `unwrap`/`panic!`/unhandled `Result::Err` propagated out of the public API.
- MUST NOT perform file I/O, network access, or depend on a specific protocol (FR-001).
- MUST NOT introduce runtime dependencies (FR-027; see Primary Dependencies rationale).
- All diagnostic messages and grammar documentation MUST be original wording, never
  copied from Bentley/Citilabs vendor documentation (constitution Principle II,
  FR-024).

**Scale/Scope**: A fixture corpus on the order of dozens of real `.s`/`.block`
scripts (individual files ranging from a few lines to ~1,000+ lines observed in real
WF-TDM-Official-Releases fixtures), covering: 3 statement forms beyond the basic
control statement (label, shell-escape, assignment), 7 block kinds (`If`-chain incl.
short-`IF`, `Loop`, `Run` incl. `!RUN`, `Process`/`PHASE`, `JLoop`, `LinkLoop`, and
`DistributeMultistep`), and 6 required diagnostic categories (FR-012–FR-016, FR-026).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | This feature *is* the one authoritative core crate for grammar/parsing logic. No adapter (CLI/LSP/MCP/formatter) code is touched in this phase, and none may duplicate this crate's grammar logic later. |
| II. No Verbatim Vendor Doc Redistribution | **PASS, with an open sourcing question** | Diagnostic messages and grammar docs will be original wording (FR-024). The fixture corpus itself is not vendor documentation, but importing real third-party production scripts (e.g. from `WF-TDM-Official-Releases`) into this repository's test fixtures raises a separate licensing question — not a vendor-doc violation, but the same spirit of "don't redistribute what you don't have the right to." Tracked as a research item; resolved before any real script content is committed (see research.md). |
| III. Formatter Idempotence & Behavior Preservation | **N/A this phase** | No formatter is built here (explicitly out of scope, FR-019). Nothing to check. |
| IV. False Negatives Over False Positives | **PASS** | The diagnostics built here (FR-012–FR-016, FR-026) are unambiguous structural/syntax errors (unmatched blocks, unclosed comments, bad continuations, misplaced `BREAK`) rather than heuristic lint rules, so the "ship as a warning until validated" clause doesn't gate them directly — but the zero-false-positive requirement (SC-001) operationalizes the same trust principle at the parser level, and is enforced by the fixture-corpus test gate. |
| V. Vertical, Independently-Usable Increments | **PASS** | This phase is independently testable and usable (a caller can tokenize/parse text and get statements/blocks/diagnostics today, with no dependency on later phases). Per Definition of Done, no later phase (CLI/LSP/MCP/formatter) starts until this phase's fixture-corpus tests pass cleanly. |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **N/A this phase** | No editor integration exists yet. |
| VII. Naming Honesty | **PASS** | This is named and scoped as a "tokenizer and structural parser" — it does not claim semantic, type, or reference checking anywhere in its API or docs. Diagnostic categories are named for exactly what they check (e.g. "unmatched block", not "syntax validator" or "type checker"). |
| VIII. Public/Private Boundary | **PASS** | This crate is public core. It must not embed vendor-documentation-derived text (ties to Principle II) or depend on a private docs corpus; it depends on nothing beyond `std`. |

No unjustified violations. Complexity Tracking is empty (see below).

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/quickstart.md): The
design introduced no new dependency (research.md § 1 confirmed by
contracts/public-api.md's entry-point contract, and now spec-level per FR-027), no
vendor-doc-derived text (every diagnostic message and grammar note is original wording
per contracts/diagnostics.md and data-model.md), and no scope creep into semantic
checking or formatting (contracts/public-api.md's "What this contract does *not*
promise" section restates FR-019 explicitly). The `MisplacedBreak` diagnostic kind
(contracts/diagnostics.md), first surfaced during Phase 1 design, is no longer a
design-time-only addition — it is now FR-026, a full spec-level requirement with the
same binding force as the other five block-matching/comment/continuation diagnostics.
It stays within "structural defect," not a new semantic rule, so it does not require
the Principle IV warnings-first treatment. All eight rows above still hold; no row's
status changed.

## Project Structure

### Documentation (this feature)

```text
specs/001-voyager-script-parser/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   ├── public-api.md
│   └── diagnostics.md
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
Cargo.toml                       # workspace manifest (new); lists crates/voyager-core
                                  # as its first member, leaving room for future
                                  # cli/, lsp/, mcp/, formatter/ crates per the
                                  # constitution's Technology & Architecture Constraints

crates/
└── voyager-core/                # this feature: tokenizer + structural parser
    ├── Cargo.toml                # name = "voyager-core"; no runtime dependencies
    ├── src/
    │   ├── lib.rs                 # public API: tokenize()/parse() and their
    │   │                             # byte-oriented siblings tokenize_bytes()/
    │   │                             # parse_bytes() (FR-034), re-exports
    │   ├── span.rs                 # Span/Position: line/column source locations
    │   ├── token.rs                # Token, TokenKind (incl. @variable@, comments)
    │   ├── lexer.rs                 # char-level scanning, comment recognition,
    │   │                             # continuation-character detection
    │   ├── statement.rs              # groups tokens into Statement (control /
    │   │                             # assignment / label / shell-escape forms),
    │   │                             # joining continued physical lines
    │   ├── block.rs                   # block matching: If-chain (incl. short-IF),
    │   │                             # Loop, Run (incl. !RUN), Process/PHASE,
    │   │                             # JLoop, LinkLoop, DistributeMultistep,
    │   │                             # nesting, BREAK validity
    │   ├── diagnostic.rs               # Diagnostic, DiagnosticKind, rendering
    │   ├── decode.rs                     # byte-oriented decoding: UTF-8 first,
    │   │                             # per-byte Windows-1252 fallback (FR-034)
    │   └── grammar_notes.rs              # per-rule "validated against Voyager 6.5"
    │                                     # notes, in the project's own words
    │                                     # (constitution Principle II, FR-024)
    └── tests/
        ├── fixtures/
        │   ├── valid/                    # real, working .s/.block scripts (or
        │   │                             # representative equivalents — see
        │   │                             # research.md sourcing/licensing note)
        │   └── broken/                    # deliberately-broken variant per
        │                                     # diagnostic category (FR-012–FR-016)
        └── fixture_corpus.rs                # walks fixtures/, asserts SC-001/002/003
```

**Structure Decision**: Single Rust library crate (`voyager-core`), placed inside a new
Cargo workspace from day one. A workspace (rather than a bare single-crate repo) costs
nothing now and avoids a disruptive restructure when the CLI/LSP/MCP/formatter adapter
crates arrive in later phases, all of which must depend on this crate per constitution
Principle I. No `tests/{contract,integration,unit}` split is used — Rust's own
convention (unit tests inline in `src/`, integration tests in `tests/`) already gives
each concern its natural home, and the fixture-corpus test *is* this feature's
integration/contract test.

## Complexity Tracking

*No violations identified — the Constitution Check above passes cleanly. This section
is intentionally left empty; there is nothing to justify.*
