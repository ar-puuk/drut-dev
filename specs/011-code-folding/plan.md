# Implementation Plan: Code Folding Support

**Branch**: `011-code-folding` | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/011-code-folding/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Add `textDocument/foldingRange` to `drut-lsp` so any LSP-capable editor can collapse
Voyager blocks (all 7 kinds) and block comments. The block side needs one small,
additive `voyager-core` function (`block_resolution.rs` gains a full-document
enumeration entry point alongside its existing single-position `block_at` query — see
research.md §1, which directly answers the owner's pre-`/speckit-tasks` question); the
block-comment side is a pure `drut-lsp`-boundary filter over the already-public
`tokenize` output. No new grammar rule, `DiagnosticKind`, or parsing behavior of any
kind — this is a translation/enumeration feature over existing structure.

## Technical Context

**Language/Version**: Rust 2021 edition (matches every other crate in this workspace).

**Primary Dependencies**: `lsp-types` 0.97.0 (already a `drut-lsp` dependency; confirmed
this version ships `FoldingRange`, `FoldingRangeKind`, `FoldingRangeProviderCapability`,
and `request::FoldingRangeRequest` — no dependency bump needed). `voyager-core`
(workspace crate, zero external runtime dependencies, FR-027 of `001`'s own spec — this
feature adds no new dependency to it either).

**Storage**: N/A — computed per-request from the in-memory document text already held by
`ServerState`/`document_store.rs`, same as every other `drut-lsp` capability.

**Testing**: `cargo test -p voyager-core` (new enumeration function's unit tests, in
`block_resolution.rs`'s existing integration-test file per its own established
convention) and `cargo test -p drut-lsp` (a real `textDocument/foldingRange` protocol
test over `Connection::memory()`, same pattern `hover.rs`/`formatting.rs` already use).

**Target Platform**: Cross-platform LSP server (Windows/macOS/Linux); any LSP-capable
editor client, VS Code used for the manual smoke-test per constitution Principle VI.

**Project Type**: Adapter feature within the existing Cargo workspace — one small
additive `voyager-core` function plus one new `drut-lsp` module
(`crates/drut-lsp/src/folding.rs`) and a capability-registration change in
`crates/drut-lsp/src/lib.rs`. No new crate.

**Performance Goals**: Folding-range computation completes within the same
per-keystroke-interactive latency budget every other `drut-lsp` request already meets
(diagnostics, hover, semantic tokens) — all of which already do a full re-parse per
request at real-corpus document sizes with no reported latency issue. No new
performance goal is introduced by this feature specifically.

**Constraints**: Constitution Principle I (no duplicated grammar/block-matching logic
in the adapter layer — this is the constraint research.md §1 resolves directly) and
Principle VI (LSP-standard mechanism, already satisfied by using
`textDocument/foldingRange` rather than any VS Code-proprietary folding API).

**Scale/Scope**: Comparable to `hover.rs` in size (a thin translation module over an
existing `voyager-core` entry point) plus one small new `voyager-core` function —
materially smaller than `008`/`009`/`010`, none of which touched `drut-lsp` at all.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Single Source of Truth)**: PASS. The five-rule counterpart derivation,
  `is_short_if`, and `block_kind_name` are reused byte-for-byte unchanged from
  `block_resolution.rs` — the only new code there is a full-document traversal wrapper
  (research.md §1) that calls those same private helpers once per block instead of the
  existing single-position `block_at` calling them once per query. `drut-lsp` performs
  no grammar or block-matching decisions of its own; it only enumerates already-public
  tokens (`BlockComment`) and translates already-computed `voyager-core` facts into
  `lsp_types::FoldingRange` shapes.
- **Principle II (No Verbatim Vendor Docs)**: N/A — no vendor-doc-derived text of any
  kind in this feature (no new hover/help strings; folding produces no user-visible
  text beyond the editor's own default collapse indicator).
- **Principle III (Formatter Idempotence)**: N/A — this feature does not touch
  `format.rs` or any formatting behavior.
- **Principle IV (False Negatives Over False Positives)**: Applies by analogy — FR-005/
  FR-007 explicitly choose to omit a fold range rather than offer one anchored to a
  guessed/incorrect counterpart for unmatched blocks or unclosed comments (a "false
  positive" fold would visually hide the wrong span of a user's document, which is a
  worse failure than simply not offering a fold control there).
- **Principle V (Vertical Increments)**: PASS — this is one independently-testable,
  shippable increment (folding), not bundled with unrelated work.
- **Principle VI (LSP-Standard Mechanisms)**: PASS — this is the entire point of the
  feature; `textDocument/foldingRange` is used instead of any editor-proprietary
  folding/region API.
- **Principle VII (Naming Honesty)**: PASS — "folding range" is the LSP's own standard
  term for exactly this capability; no overclaiming.
- **Principle VIII (Public/Private Boundary)**: N/A — no vendor-documentation-derived
  content involved.

No violations; Complexity Tracking table is not needed.

## Project Structure

### Documentation (this feature)

```text
specs/011-code-folding/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/
├── voyager-core/
│   └── src/
│       └── block_resolution.rs   # + one new pub enumeration fn (research.md §1);
│                                  #   zero changes to the existing counterpart_for/
│                                  #   is_short_if/block_kind_name derivation rules
│   └── tests/
│       └── block_resolution.rs   # + unit tests for the new enumeration fn
│
└── drut-lsp/
    └── src/
        ├── lib.rs                # + folding_range_provider capability,
        │                          #   FoldingRangeRequest dispatch
        └── folding.rs             # NEW — translates voyager-core facts into
                                    #   Vec<lsp_types::FoldingRange>
    └── tests/
        └── protocol_smoke.rs      # + textDocument/foldingRange test(s) over
                                    #   Connection::memory()
```

**Structure Decision**: Matches this repo's existing structure exactly (constitution
Principle I) — the one grammar-adjacent addition lives in `voyager-core`, and every
other change is a thin `drut-lsp` adapter file, following the same shape `hover.rs`
already established for consuming `block_resolution.rs`. No new crate, no new
directory beyond the two files named above.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

Not applicable — no Constitution Check violations (see above).

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
