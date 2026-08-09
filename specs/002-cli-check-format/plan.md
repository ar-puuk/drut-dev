# Implementation Plan: Drut CLI — `check` and `format` Subcommands

**Branch**: `002-cli-check-format` | **Date**: 2026-08-09 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-cli-check-format/spec.md`

## Summary

Build a new binary crate (`drut-cli`, producing the `drut` executable) that is a thin
I/O/traversal/rendering adapter over `voyager-core`. `drut check <path>` walks a file
or `.gitignore`-respecting directory tree, reads every `.s`/`.block` file's raw bytes,
calls `voyager-core::parse_bytes()`, and reports every `Diagnostic` as plain text
(default) or a SARIF 2.1.0 log (`--format=sarif`), exiting with one of three codes
(clean / diagnostics found / run-couldn't-complete). `drut format <path>` applies the
same traversal, then a new `voyager-core::format_bytes()` entry point this feature
also adds to the core crate — whitespace normalization, plus opt-in, explicitly-named
keyword-casing normalization — printing to stdout by default, or writing in place
(`--write`), checking (`--check`), or diffing (`--diff`) instead. The formatting
*decision logic* (indentation from block nesting, casing rewrite) is added to
`voyager-core`, not the CLI, so the CLI never re-implements grammar/parsing/formatting
logic (constitution Principle I) — it only decides *what to do with* the text
`voyager-core` hands back (print/write/diff) and *how to report* what `voyager-core`
found (text/SARIF, exit code).

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition — matches `voyager-core`.

**Primary Dependencies**:
- `clap` (derive) — subcommand/flag parsing (`check`/`format`, `--format`, `--write`,
  `--check`, `--diff`, `--casing`). Hand-rolling argument parsing has no grammar/
  parsing-logic overlap with Principle I's concern and would just reproduce a solved
  problem.
- `ignore` — `.gitignore`-aware recursive directory walking (FR-002). This is the
  same crate `ripgrep` is built on; reimplementing gitignore glob semantics by hand
  is exactly the kind of "duplicated effort with no grammar/parsing content" the
  spec's Assumptions section already calls out as acceptable to depend on.
  (Confirmed current on crates.io during Phase 0 research.)
- `serde` (+ `serde_json`) — typed SARIF 2.1.0 structures for exactly the shape
  this feature emits (FR-009/SC-003), hand-written rather than via `serde-sarif`'s
  code-generated types, after `serde-sarif`'s `schemafy`-based build script proved
  blocked by an Application Control policy on the implementation machine
  (research.md §4). The typed-struct guarantee (no ad-hoc `serde_json::json!`
  assembly) is unaffected; SC-003 is proven by validating emitted output against
  the real schema in tests either way.
- `similar` — unified-diff generation for `format --diff` (FR-019). Hand-writing a
  correct Myers-diff-based unified-diff formatter is a well-solved, non-grammar
  problem; not worth duplicating.
- **Dev-dependency**: `jsonschema` — validates emitted SARIF logs against the
  official SARIF 2.1.0 JSON Schema in tests (SC-003), rather than only trusting
  `serde-sarif`'s types to be correct.
- `voyager-core` (path dependency, existing workspace member) — supplies
  `parse_bytes` (unchanged) and the new `format`/`format_bytes` entry points this
  feature adds to it.

None of the above touches Voyager grammar, parsing, or formatting *decisions* — see
Constitution Check, Principle I row, for why this dependency set doesn't conflict
with `voyager-core`'s own zero-dependency rule (FR-027 in `001-voyager-script-parser`
scopes that rule to the core crate specifically, not this adapter).

**Storage**: N/A — the CLI is stateless between invocations; it only reads/writes the
files reachable under the path the caller gives it, and holds no other persistent
state.

**Testing**:
- `cargo test -p voyager-core` — extended with a new golden-file/idempotency/
  structural-equivalence suite (`tests/format_corpus.rs`) exercising the new
  `format_bytes` entry point directly against the fixture corpus (FR-021, SC-004,
  SC-005) — this is where formatting *correctness* is proven, at the same layer
  `parse_bytes`'s own correctness is proven, per Principle I.
- `cargo test -p drut-cli` — CLI-level tests scoped to what the CLI itself is
  responsible for: traversal/filtering/`.gitignore` behavior, exit-code selection,
  `--format=sarif` schema validity, and `--write`/`--check`/`--diff` file-I/O
  semantics — plus one full-corpus end-to-end smoke test that runs the actual built
  `drut` binary (via `CARGO_BIN_EXE_drut`) against the real corpus to reproduce
  SC-001 (161/161 clean) through the CLI itself, per the spec's Definition of Done.
  This test suite deliberately does *not* re-verify formatting/parsing correctness
  already covered at the `voyager-core` layer — only that the CLI wires it up
  correctly.

**Target Platform**: Cross-platform (Windows/macOS/Linux) — same as `voyager-core`.
The reference corpus is Windows-authored (CRLF line endings); traversal, reads, and
writes must not corrupt line endings the formatter isn't explicitly asked to
normalize (FR-013's "only whitespace... changes" scope covers this).

**Project Type**: CLI — a new binary crate added as a sibling workspace member to
`voyager-core`, both under the existing `crates/` directory.

**Performance Goals**: SC-007 — a full `drut check` run over the 161-file corpus
completes in under 5 seconds on typical developer hardware. `voyager-core`'s own
plan.md already establishes low-tens-of-milliseconds-per-file parsing cost; this
phase's job is to keep traversal, I/O, and rendering overhead from dominating that.

**Constraints**:
- MUST NOT duplicate any grammar, parsing, or formatting-decision logic in this
  crate (Principle I, FR-022) — every structural or whitespace/casing decision is
  delegated to `voyager-core`'s `parse_bytes`/`format`/`format_bytes`.
- MUST NOT panic on any input file's content, including non-UTF-8 bytes or
  arbitrary binary content under a `.s`/`.block` extension (FR-023).
- `format` MUST be idempotent and strictly behavior-preserving (constitution
  Principle III, FR-013, FR-014) — enforced by the new `voyager-core`-level golden-
  file suite, not asserted only at the CLI layer.
- SARIF output MUST validate against the SARIF 2.1.0 schema on every run, clean or
  not (SC-003).
- Exit codes MUST follow the documented three-way convention for both subcommands
  (FR-011, FR-020) and MUST be independently testable from process exit status
  alone (SC-006).

**Scale/Scope**: Same 161-file WF-TDM-Official-Releases corpus as
`001-voyager-script-parser`; single-file and whole-directory-tree invocations;
keyword-casing normalization limited to the two named conventions in the spec
(upper/lower).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS, with a design decision worth flagging** | The CLI itself adds zero grammar/parsing/formatting logic — it calls `voyager-core::parse_bytes`/`format`/`format_bytes` and only handles traversal, I/O, and rendering. The formatting *decision* logic (indentation from block nesting, casing rewrite) is added to `voyager-core` in this same feature rather than the CLI, specifically so it isn't a second, adapter-local implementation of grammar-derived knowledge (block depth) that could drift from the parser. See research.md §1. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | No new vendor-documentation-derived text is introduced. Diagnostic messages are unchanged (already original wording per `001-voyager-script-parser`); this feature adds no new diagnostic categories or hover/help text of its own. |
| III. Formatter Idempotence & Behavior Preservation | **PASS — this is the phase that instantiates it** | `format`/`format_bytes` MUST be idempotent and MUST NOT reorder statements, change continuation structure, or alter meaning (FR-013, FR-014); the golden-file test suite this phase establishes (FR-021) is the enforcement mechanism the constitution requires before any formatter change merges, going forward. |
| IV. False Negatives Over False Positives | **PASS** | `check` introduces no new diagnostic categories — it only surfaces `voyager-core`'s existing ones through the CLI. SC-001/SC-002 require the CLI to reproduce, not regress, the zero-false-positive result already proven at the library level. |
| V. Vertical, Independently-Usable Increments | **PASS** | `check` and `format` are each independently testable and independently valuable per the spec's own Independent Test sections; a team could adopt `check` alone. This phase does not start until `001-voyager-script-parser`'s fixture-corpus tests pass cleanly (already true — see that spec's research.md §3 full-corpus validation), satisfying the phase-gate this principle requires. |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **N/A this phase** | No editor/LSP integration is built here. |
| VII. Naming Honesty | **PASS** | `check` and `format` are named for exactly what they do — `check` reports structural diagnostics (not "lint" or "validate" implying semantic checking out of this phase's scope, FR-019 in `001-voyager-script-parser`); `format` only normalizes whitespace/casing, not a broader "linter" or "fixer." |
| VIII. Public/Private Boundary | **PASS** | This feature touches only the public core crate and a new public CLI crate; no vendor-documentation corpus content is read, generated, or linked in. |

No unjustified violations. Complexity Tracking is empty (see below).

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/quickstart.md): The
design confirms the Principle I split described above — `contracts/formatting-api.md`
shows the new `voyager-core` entry points carry all indentation/casing decision logic,
while `contracts/cli-contract.md` shows the CLI's own contract is limited to
flags/exit-codes/output-rendering, with no grammar terms in it beyond what it passes
through from `Diagnostic`/`FormatResult`. `contracts/sarif-mapping.md`'s
`DiagnosticKind`→`ruleId` table introduces no new diagnostic semantics, only a
presentation mapping. No row's status changed from the pre-design check above.

## Project Structure

### Documentation (this feature)

```text
specs/002-cli-check-format/
├── plan.md                    # This file (/speckit-plan command output)
├── research.md                # Phase 0 output (/speckit-plan command)
├── data-model.md               # Phase 1 output (/speckit-plan command)
├── quickstart.md               # Phase 1 output (/speckit-plan command)
├── contracts/                  # Phase 1 output (/speckit-plan command)
│   ├── cli-contract.md          # drut's command/flag/exit-code surface
│   ├── formatting-api.md         # new voyager-core format/format_bytes entry points
│   └── sarif-mapping.md           # DiagnosticKind -> SARIF ruleId/level mapping
├── checklists/
│   └── requirements.md         # already created by /speckit-specify
└── tasks.md                    # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
Cargo.toml                       # workspace manifest (existing); add "crates/drut-cli"
                                  # as a second member alongside "crates/voyager-core"

crates/
├── voyager-core/                # existing crate (001-voyager-script-parser);
│   │                             # this feature ADDS to it, does not restructure it
│   ├── src/
│   │   ├── lib.rs                 # add: pub mod format; re-export format/format_bytes/
│   │   │                             # FormatOptions/FormatResult/CasingConvention
│   │   └── format.rs               # NEW: whitespace-normalization + opt-in casing
│   │                             # rewrite, built on the existing Token/Statement/
│   │                             # Block/ParseResult data — no new grammar rules,
│   │                             # just a renderer over already-parsed structure
│   └── tests/
│       ├── fixtures/
│       │   └── golden/             # NEW: known-correct formatted counterparts,
│       │                             # one per corpus fixture (FR-021)
│       └── format_corpus.rs        # NEW: idempotency (SC-004), structural-
│                                     # equivalence (SC-005), and golden-diff (FR-021)
│                                     # checks against the fixture corpus
│
└── drut-cli/                     # NEW: this feature's main deliverable
    ├── Cargo.toml                 # package "drut-cli"; [[bin]] name = "drut";
    │                             # depends on voyager-core (path) + clap/ignore/
    │                             # serde/serde_json/similar (no serde-sarif —
    │                             # see research.md §4)
    ├── src/
    │   ├── main.rs                 # entry point: parse args, dispatch, set process
    │   │                             # exit code
    │   ├── cli.rs                   # clap derive: Cli, Command::Check{..}/Format{..},
    │   │                             # --format, --write, --check, --diff, --casing
    │   ├── traverse.rs               # shared file/dir walk + .gitignore + .s/.block
    │   │                             # extension filtering (FR-001-003)
    │   ├── check_cmd.rs               # check orchestration: traverse -> parse_bytes
    │   │                             # per file -> aggregate diagnostics -> report
    │   ├── format_cmd.rs               # format orchestration: traverse -> format_bytes
    │   │                             # per file -> stdout/--write/--check/--diff
    │   ├── report/
    │   │   ├── mod.rs                  # shared report types
    │   │   ├── text.rs                  # plain-text diagnostic/format-status rendering
    │   │   └── sarif.rs                  # Check Report -> serde_sarif::Sarif mapping
    │   └── exit.rs                    # shared three-way ExitOutcome -> process exit
    │                                     # code (FR-011, FR-020)
    └── tests/
        ├── traversal.rs                # .gitignore / extension-filtering behavior
        ├── exit_codes.rs                # all three outcomes, both subcommands
        ├── sarif_schema.rs              # emitted SARIF validates against 2.1.0 schema
        ├── format_flags.rs              # default/--write/--check/--diff behavior
        └── fixture_corpus_e2e.rs        # full-corpus smoke test via the built `drut`
                                          # binary, reproducing SC-001 end-to-end
```

**Structure Decision**: Add `drut-cli` as a second member of the existing Cargo
workspace, under `crates/` alongside `voyager-core` — keeping one place
(`crates/`) for every workspace member rather than splitting core crates and adapter
crates across different top-level directories, since nothing about this project's
layout benefits from that split and it would be one more thing for a newcomer to
learn. (This narrows the root `Cargo.toml`'s existing "future members: cli/, lsp/,
mcp/, formatter/" comment, written before any adapter existed, to the more specific
`crates/drut-cli` path — the comment's own crate *names* still hold, just under
`crates/`.) Formatting-decision logic is added inside `voyager-core` (new
`format.rs` module) rather than a separate crate or the CLI itself, per the
Constitution Check discussion above — this keeps `voyager-core` the single place
that understands Voyager structure well enough to safely re-render it, exactly as
Principle I requires for grammar/parsing logic, and formatting decisions are
grammar-adjacent in the same way (they need block-nesting depth and continuation
boundaries, both of which only `voyager-core` computes).

## Complexity Tracking

*No violations identified — the Constitution Check above passes cleanly. This
section is intentionally left empty; there is nothing to justify.*
