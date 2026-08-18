# Implementation Plan: Range-Dash Spacing Exemption

**Branch**: `023-range-dash-spacing` | **Date**: 2026-08-18 | **Spec**: [spec.md](spec.md)

## Summary

`operator_spacing`'s `Fixed`/`Auto` modes today treat every binary `-` uniformly as arithmetic
subtraction, spacing it apart (`1 - 50`). Inside a `Control` statement's pair-keyword value, a
`-` directly joining two bare integer literals (e.g. `SELECTLINK=1-50,75,90-100`) is instead Cube
Voyager's conventional inclusive-range list notation and must render tight, regardless of how it
was originally spaced. This amends `018-operator-spacing`'s existing binary-`-` recognition path
in `crates/voyager-core/src/operator_spacing.rs` to special-case that one shape — reusing the
same `pair_keyword_boundaries` value-span data `collect_comma_edits` already derives, and the
same `push_gap_edit` gap-normalization helper every other spacing rule already uses, targeting
zero surrounding whitespace instead of one space. No new `[format]` field, CLI flag, MCP
parameter, or editor setting; no lexer/`TokenKind` change.

## Technical Context

**Language/Version**: Rust (workspace-pinned stable toolchain, matching `018-operator-spacing`
and every other `voyager-core` feature).

**Primary Dependencies**: None new. `voyager-core` remains zero-runtime-dependency
(`cargo tree -p voyager-core` shows nothing beyond `std`), per constitution/CLAUDE.md.

**Storage**: N/A.

**Testing**: `cargo test` — unit tests in `operator_spacing.rs`'s own `#[cfg(test)]` module
(same pattern `018` established), plus the existing `voyager-core` real/golden fixture-corpus
harness (`tests/format_corpus.rs`) for a new configured variant once a real corpus fixture
exercises this shape.

**Target Platform**: Cross-platform Rust library (Windows/Linux/macOS), same as every other
`voyager-core` feature — no platform-specific behavior.

**Project Type**: Library (single Rust workspace crate change: `voyager-core`). No adapter crate
(`drut-cli`/`drut-lsp`/`drut-mcp`/`drut-config`) changes at all — this is invisible at every
adapter boundary, reachable purely through the already-existing `operator_spacing` setting.

**Performance Goals**: No new performance target — this adds one bounded, per-occurrence check
(pair-value-membership + two-neighbor-token digit check) to a recognition pass that already runs
once per statement; no new pass over the token stream, no new allocation beyond what
`pair_keyword_boundaries` already allocates for `collect_comma_edits` on the same statement.

**Constraints**: Must not change `Fixed`/`Auto` behavior for any `-` outside a pair-keyword value
(FR-004), must not affect `Preserve` at all (FR-007), must not introduce a new configuration
surface (FR-008), must remain idempotent (SC-004) and behavior-preserving (constitution
Principle III).

**Scale/Scope**: Single-module amendment (`crates/voyager-core/src/operator_spacing.rs`); no
new module, no new public type.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | Entirely inside `voyager-core`'s `operator_spacing.rs`; zero adapter-crate changes, so nothing to duplicate. |
| II. No Verbatim Vendor Docs | **PASS** | No vendor-documentation text involved — the range-list convention is described here in the project's own words, grounded in the feature description and direct code/lexer inspection (research.md), not copied from any Cube Voyager manual. |
| III. Formatter Idempotence & Behavior Preservation | **PASS** | Reuses `push_gap_edit`'s existing idempotent gap-normalization; quickstart.md Steps 3–5 require an explicit idempotence re-check and a golden-file diff against the fixture corpus before merge, per this principle's own requirement. |
| IV. False Negatives Over False Positives | **PASS** | FR-002 deliberately scopes the exemption to bare integer literals only — a `@token@` reference, decimal, or other non-integer operand falls back to *not* exempting (i.e., keeps normal spacing) rather than guessing, so an ambiguous case degrades toward the existing, already-trusted `018` behavior rather than a new false application. |
| V. Vertical, Independently-Usable Increment | **PASS** | Fully shippable on its own — no dependency on any not-yet-built feature; gated on real-corpus fixture-test evidence before merge (quickstart.md Step 5), same as every prior phase. |
| VI. LSP-Standard Mechanisms | **N/A** | No editor-protocol surface touched — this is invisible above `voyager-core`. |
| VII. Naming Honesty | **PASS** | "Range-Dash Spacing Exemption" states exactly and only what the feature does; no overclaimed capability. |
| VIII. Public/Private Boundary | **PASS** | No vendor documentation corpus content involved anywhere in this feature. |

No violations — Complexity Tracking section is not needed.

## Project Structure

### Documentation (this feature)

```text
specs/023-range-dash-spacing/
├── plan.md              # This file
├── research.md           # Phase 0 output
├── data-model.md          # Phase 1 output
├── quickstart.md          # Phase 1 output
├── contracts/
│   └── range-dash-spacing.md
└── tasks.md              # Phase 2 output (/speckit-tasks — not created by this command)
```

### Source Code (repository root)

```text
crates/
├── voyager-core/
│   └── src/
│       └── operator_spacing.rs   # MODIFIED — the only source file this feature touches
│
# No changes anywhere else: drut-config, drut-cli, drut-lsp, drut-mcp,
# editors/vscode, docs-site/ are all untouched (FR-008 — no configuration
# surface, no user-facing documentation change, since operator_spacing's
# existing behavior description in docs-site/src/formatter-guide.md and
# configuration-reference.md is still accurate at the setting-name level;
# only its worked examples may gain a range-list illustration, a docs-only
# follow-up, not part of this plan's Source Code scope).
```

**Structure Decision**: Single-project Rust workspace, existing structure unchanged — this
feature adds no new crate, module, or directory; it modifies one existing file
(`crates/voyager-core/src/operator_spacing.rs`) and its own `#[cfg(test)]` module in place, the
same shape every prior small `018`/`019`-era amendment already used.

## Complexity Tracking

*No Constitution Check violations — this section is intentionally empty.*
