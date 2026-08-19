# Implementation Plan: Function-Call Casing Normalization

**Branch**: `025-function-casing` | **Date**: 2026-08-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/025-function-casing/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Adds a fourth, independently-configurable casing category, `function_calls`, to
`voyager-core`'s existing `CasingSettings` (`control_words`, `pair_keywords`,
`data_references`, `017-casing-categories-indent-width`). Ports the 138-name function
list from `024-function-call-highlighting/research.md` into `voyager-core` as the
canonical source (Constitution Principle I), recognized by a new `function_call.rs`
module built on the exact architectural shape `data_reference.rs` already establishes: a
quote-aware scan over already-parsed tokens (not a new AST/grammar concept), with one
extra condition unique to this category — the matched name must be immediately followed
by `(` with zero intervening whitespace (mirrors `024`'s own `#function-calls` grammar
pattern, and is required, not just stylistic, to correctly disambiguate the two real
dual-category names found during spec-writing: `FORMAT`/`LOG`). Reachable through the
same four surfaces the other three casing categories already use: `drut.toml`
(`casing_function_calls`), CLI (`--casing-function-calls`), MCP `format` tool
(`casing_function_calls`), VS Code setting (`drut.format.casingFunctionCalls`).

## Technical Context

**Language/Version**: Rust (workspace edition, matching `crates/voyager-core`) +
TOML/CLI/MCP/JSON adapter surfaces in `drut-config`/`drut-cli`/`drut-mcp`/
`editors/vscode`

**Primary Dependencies**: None new — `voyager-core` keeps its zero-runtime-dependency
guarantee (FR-027); this feature reuses existing crate-internal modules
(`data_reference.rs`'s architectural pattern, `format.rs`'s existing casing-edit
pipeline, `statement.rs`'s token/span model)

**Storage**: N/A

**Testing**: `cargo test -p voyager-core` (new unit tests in the new module + `format.rs`
casing tests), `cargo test -p voyager-core --test format_corpus` (golden-fixture
verification, Constitution Principle III — new `golden_casing_function_calls_*`
fixture variants, same discipline `018`/`019`/`023` each established for their own
formatter changes), `cargo clippy --workspace --all-targets -- -D warnings`

**Target Platform**: Same as every other `voyager-core` formatter feature — CLI,
LSP server (format-on-save/paste), MCP server, cross-platform

**Project Type**: Library + thin adapters (Constitution Principle I) — core logic in
`voyager-core`, config/surface wiring in `drut-config`/`drut-cli`/`drut-mcp`, VS Code
client-settings passthrough only (no grammar change, FR-009)

**Performance Goals**: N/A (one more `entry()` table + one more quote-aware token scan,
the same shape/cost `data_reference.rs`'s existing scan already has — no algorithmic
change)

**Constraints**: Constitution Principle III (idempotence, behavior preservation, golden-
fixture verification before merge) applies in full — unlike `024`, this is a real
formatter behavior change, not cosmetic highlighting. Zero-runtime-dependency principle
(FR-027) unaffected — no new dependency.

**Scale/Scope**: One new `voyager-core` module (`function_call.rs`, ~138-entry table +
occurrence scan, mirroring `data_reference.rs`'s existing size/shape), one new
`CasingSettings` field + its `format.rs` wiring, matching field/flag/parameter/setting
additions in `drut-config`/`drut-cli`/`drut-mcp`/`editors/vscode`'s client-settings
schema, golden-fixture additions, no `editors/vscode` grammar change (FR-009)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Single Source of Truth)**: The 138-name list moves from being
  duplicated only in `editors/vscode`'s grammar JSON to living canonically in
  `voyager-core` (`function_call.rs`) — `editors/vscode` becomes the documented,
  manually-synced mirror (the same relationship `#control-words` already has with
  `statement.rs`'s `FIXED_KEYWORDS`), not the other way around. All casing/recognition
  logic lives in `voyager-core` only; every adapter (`drut-config`, `drut-cli`,
  `drut-mcp`, `drut-lsp`) is a thin pass-through of the new `CasingSettings` field,
  duplicating no logic. **PASS** — re-verify in Phase 1 that no adapter reimplements the
  138-name list or the `(`-adjacency check itself.
- **Principle II (No Verbatim Vendor Docs)**: No new vendor-doc research — reuses
  `024-function-call-highlighting/research.md`'s already-compliant 138-name list
  (names only, own wording) as-is. **PASS.**
- **Principle III (Formatter Idempotence & Behavior Preservation)**: Binding in full
  here (unlike `024`). MUST verify: `format(format(x)) == format(x)` for every
  `casing_function_calls` value; MUST NOT change program meaning (only the function-name
  token's casing changes — arguments, spacing, everything else untouched); MUST verify
  against the fixture corpus with a golden-file diff before merge (FR-008). Re-checked
  in Phase 1 design and again before merge.
- **Principle IV (False Negatives Over False Positives)**: A name absent from the
  138-entry table is simply never recognized as a `function_calls` occurrence — no
  casing change, no false flag. The `(`-adjacency requirement itself is a
  false-positive guard (FR-002/FR-004): without it, `FORMAT=CSV` (a pair-keyword
  occurrence) could be wrongly caught by `function_calls` casing too. **PASS.**
- **Principle V (Vertical, Independently-Usable Increments)**: Self-contained addition;
  `casing_function_calls` defaults to `Preserve` (SC-002: zero behavior change for any
  project that doesn't opt in), so no prior phase's fixture-corpus tests are affected
  unless a project explicitly configures the new field. **PASS.**
- **Principle VI (LSP-Standard Mechanisms)**: N/A — no new editor-specific mechanism;
  `drut-lsp`'s format-on-save/paste already routes through `drut-config`/`voyager-core`
  generically, picking up the new field for free (spec.md Assumptions).
- **Principle VII (Naming Honesty)**: `function_calls` is named for exactly what it
  recognizes (a function-call-shaped occurrence) — no overclaiming. **PASS.**
- **Principle VIII (Public/Private Boundary)**: No vendor-documentation-derived prose is
  imported (Principle II re-check above); only identifier names, already-compliant via
  `024`. **PASS.**

No violations — Complexity Tracking table is not needed.

## Project Structure

### Documentation (this feature)

```text
specs/025-function-casing/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/voyager-core/src/
├── function_call.rs         # NEW — 138-entry table + is_function_call_name() +
│                             # function_call_occurrences() (mirrors data_reference.rs)
├── format.rs                 # CasingSettings gains `function_calls`; render() wires
│                             # function_call_occurrences() the same way it already
│                             # wires data_reference_occurrences()
└── lib.rs                    # re-export the new module's public items, if any

crates/voyager-core/tests/
├── format_corpus.rs          # add golden_casing_function_calls_upper/_lower variants
└── fixtures/
    ├── golden_casing_function_calls_upper/   # NEW golden fixture directory
    └── golden_casing_function_calls_lower/   # NEW golden fixture directory

crates/drut-config/src/       # lib.rs + parse.rs: new `casing_function_calls` field,
                               # same three-value parse/merge/default handling the
                               # existing three casing fields already have
crates/drut-cli/src/          # cli.rs, format_cmd.rs, lib.rs: new
                               # `--casing-function-calls` flag, same wiring pattern
crates/drut-mcp/src/          # new `casing_function_calls` parameter on the `format`
                               # tool, same pattern as the existing three
editors/vscode/
├── package.json               # new `drut.format.casingFunctionCalls` client setting
└── src/                       # client-settings passthrough (existing generic
                                # mechanism per `829d065`'s editor-client-settings
                                # support — no new grammar/highlighting change, FR-009)
```

**Structure Decision**: Vertical increment across the existing crate boundaries
(Constitution Principle I) — one new `voyager-core` module plus the already-established
per-adapter wiring pattern every prior casing-category/`[format]`-field addition
followed (`017-casing-categories-indent-width`, `021-editor-settings-config`). No new
crate, no new project type. `editors/vscode`'s TextMate grammar (`024`'s
`#function-calls` pattern) is explicitly NOT touched (FR-009) — only its
client-settings schema gains the new passthrough setting.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations — table not needed.
