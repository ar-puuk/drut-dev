# Implementation Plan: Operator Spacing Normalization

**Branch**: `018-operator-spacing` | **Date**: 2026-08-17 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/018-operator-spacing/spec.md`

## Summary

One new `voyager-core` formatting axis, `FormatOptions.operator_spacing: OperatorSpacing`
(`Preserve`/`Fixed`/`Auto`, default `Preserve`), implemented as a new self-contained module
(`src/operator_spacing.rs`) mirroring the architectural shape `data_reference.rs` established
for `017`: a read-only recognition pass over already-tokenized `Statement`/`Token` data, no
`lexer.rs`/`TokenKind` change. Two real findings from grounding this in the actual code
(research.md) shape the plan:

1. Every operator character this feature cares about (`= + - / * ^ & | < >`, plus `,`) is
   *already* tokenized as a standalone single-character `Punctuation` token — recognition needs
   no lexer change. Multi-character comparisons (`==`, `<>`, `>=`, `<=`) aren't single tokens
   today, so a small zero-gap-adjacency merge step is added inside the new module (research.md
   §2), not the shared lexer.
2. `format.rs::render`'s existing `CasingEdit` application is a same-length in-place column
   splice — it cannot represent whitespace insertion/removal, which this feature fundamentally
   needs (`MW[1]=x` → `MW[1] = x`). A new `SpacingEdit` list and a left-to-right per-line
   rebuild step are added alongside it (research.md §4/data-model.md §2); the existing casing
   path is untouched.

Every existing config surface stays exactly as it is — a purely additive change (spec FR-009),
never a breaking one. `format.rs`'s "Scope, precisely" module doc comment is reworded (still
exactly true for `Preserve`, the default) rather than left stale (research.md §8).

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition — unchanged.

**Primary Dependencies**: None new — `voyager-core` remains zero-runtime-dependency (FR-027 in
`001-voyager-script-parser`).

**Storage**: N/A.

**Testing**:
- `crates/voyager-core/src/operator_spacing.rs` (new) — unit tests per operator kind
  (assignment, each comparison including the two-token-merge case, binary vs. unary arithmetic,
  comma, bracket/paren interior padding, control-word-paren adjacency), the continuation-marker
  leading-only-space case, and the full `Auto` alignment-run behavior (run detection, the three
  break conditions, target-column computation, run-of-one no-op).
- `crates/voyager-core/src/format.rs` — new unit tests for the `SpacingEdit` per-line rebuild
  (multiple edits on one line, a casing edit and a spacing edit coexisting on one line,
  `; FMT: OFF`/`ON` protection extended to spacing edits via the existing `push_if_present`
  funnel point), plus the `Preserve`-short-circuit performance test mirroring casing's existing
  one.
- `crates/voyager-core/tests/format_corpus.rs`/`format_sequence.rs` — no golden-fixture
  regeneration expected for the *existing* golden set (FR-009: byte-identical when
  `operator_spacing` isn't configured); new golden fixtures added specifically exercising
  `Fixed` and `Auto`, verified by hand before being trusted as golden, plus idempotence checks
  for both (SC-005), following the same discipline `017` already established.
- `crates/drut-config/tests/parse.rs`/`resolve.rs` — new cases for the `operator_spacing` field,
  its three accepted values, and the invalid-value-falls-back-to-`preserve` case.
- `crates/drut-cli/tests/format_flags.rs` — new cases for `--operator-spacing`.
- `crates/drut-mcp/src/format.rs` test module — same shape, MCP-side.
- Full real-corpus revalidation (CLI/LSP/MCP) — expected zero diagnostic/output change with no
  new configuration supplied (SC-003), reported as its own explicit result per this project's
  established standard.

**Target Platform**: Cross-platform, unchanged.

**Project Type**: `voyager-core` core change (one new module, one new `FormatOptions` field,
one new render-pipeline capability) plus symmetric, additive adapter-layer wiring in
`drut-config`/`drut-cli`/`drut-mcp` (`drut-lsp` untouched behaviorally, same as `014`/`017`).

**Performance Goals**: No measurable regression when `operator_spacing == Preserve` (the
default) — the entire collection pass and the new per-line rebuild path are skipped via the
same short-circuit pattern `casing` already uses (data-model.md §2). When `Fixed`/`Auto` *are*
configured, cost is one additional linear pass over each statement's already-materialized token
list — no new full-file re-scan, no re-tokenization.

**Constraints**:
- MUST NOT change formatter output for any existing input when `operator_spacing` isn't
  configured (FR-009) — confirmed by the full existing golden-fixture set and corpus passing
  byte-for-byte unmodified.
- MUST NOT remove or change the meaning of any already-shipped config surface (`casing`,
  `top_level_indent`, `indent_width`, and their CLI/MCP equivalents) — this feature adds exactly
  one new field/flag/param, nothing else changes shape.
- MUST NOT introduce a lexer/`TokenKind` change — multi-char operator recognition and unary/
  binary disambiguation are both solvable as read-only scans over the existing token stream
  (research.md §1, §2, §5); adding either directly to `lexer.rs` would change `TokenKind`
  semantics for every other consumer (diagnostics, LSP, hover) that has no reason to care.
- MUST correctly distinguish an operator's mid-expression occurrence from its
  trailing-line-continuation occurrence (research.md §3) — the two positions get different
  spacing treatment (two-sided vs. leading-only) despite sharing the same source character set.
- MUST implement `Auto` as `Fixed` plus alignment, never a second independent spacing
  computation (contracts/operator-spacing.md) — avoids the two paths silently disagreeing on
  base spacing over time.
- MUST NOT change which `voyager-core` grammar rules exist or how any node is structured
  (Principle I/III) — `operator_spacing.rs` is a read-only recognition pass over already-parsed
  `Statement`/`Token` data, the same architectural shape `data_reference.rs`/
  `token_resolution.rs`/`block_resolution.rs` already use, not a lexer or parser change.
- Invalid `operator_spacing` values MUST degrade to `preserve` with a non-blocking notice
  (FR-011), the same established pattern every other malformed `[format]` field already uses.

**Scale/Scope**: Same 161-file real corpus, revalidated for zero change with no new
configuration supplied, plus new hand-verified golden fixtures for `Fixed` and `Auto`
specifically (including at least one real file with several consecutive assignments, to
exercise `Auto`'s alignment behavior against genuine corpus shapes, not just synthetic cases).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | `operator_spacing.rs`'s recognition/merge/alignment logic lives entirely in `voyager-core`; every adapter gains only thin field/flag/param mapping, mirroring `data_references`'s already-established pattern. No grammar/parsing/formatting logic duplicated outside the core crate. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | This feature's rules (spacing normalization, alignment) are a formatting-style decision, not a vendor-grammar fact — no vendor documentation was consulted or paraphrased for this plan; the industry precedent cited (`gofmt`/Prettier/Tidyverse) is general programming-language-tooling knowledge, not Cube Voyager vendor material. |
| III. Formatter Idempotence & Behavior Preservation | **PASS, re-verified not assumed** | `operator_spacing` is a new, opt-in axis — FR-009/SC-003 require zero output change with nothing configured, confirmed by the full existing corpus/golden set passing unmodified, not by inspection. `Fixed`/`Auto`'s own idempotence (SC-005) is verified the same way every existing axis already is — running twice produces no further change. `; FMT: OFF`/`ON` and every other existing protection (FR-010) is re-verified against this new axis specifically via the `push_if_present` funnel point, not assumed to hold transitively. |
| IV. False Negatives Over False Positives | **N/A** | Governs diagnostics; no diagnostic category is added, changed, or suppressed by this feature. |
| V. Vertical, Independently-Usable Increments | **PASS** | US1 (`Fixed`) and US2 (`Auto`) are independently valuable and independently testable (spec.md) — `Fixed` ships fully functional even if `Auto`'s alignment logic were somehow reverted, since `Auto` is implemented as a strict superset, never the reverse dependency. |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **N/A** | No new editor-integration surface — `drut-lsp` gains only the type/field changes rippling through its existing format-on-save/format-on-paste call sites. |
| VII. Naming Honesty | **PASS** | `OperatorSpacing`/`Fixed`/`Auto`/`operator_spacing.rs` name exactly what they do; no overclaiming (`Auto` is explicitly documented as "gofmt-style alignment," not an opinionated house style, matching `017`'s precedent of never shipping a hidden preset). |
| VIII. Public/Private Boundary | **PASS** | All touched crates are already public. No vendor-doc-derived material is introduced by this feature at all (see Principle II row) — nothing to keep local-only. |

No unjustified violations. No Complexity Tracking entries.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/quickstart.md):
`contracts/operator-spacing.md`'s exact type/precedence inventory confirms the Principle I/III
framing above holds precisely — no row's status changed. The one genuinely new architectural
piece beyond `017`'s established pattern — the `SpacingEdit` variable-length edit-application
mechanism (data-model.md §2) — stays entirely inside `format.rs`'s existing `render()` function,
not a new public surface, so it doesn't change this table either.

## Project Structure

### Documentation (this feature)

```text
specs/018-operator-spacing/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output
├── data-model.md                  # Phase 1 output
├── quickstart.md                  # Phase 1 output
├── contracts/
│   └── operator-spacing.md        # exact type shapes, precedence, edit-application contract
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
crates/voyager-core/
├── src/operator_spacing.rs (new)    # OperatorSpacing enum; operator/comma/bracket-paren
│                                    #   recognition over Statement token lists; multi-char
│                                    #   comparison merge (research.md §2); unary/binary +/-
│                                    #   disambiguation (research.md §5); continuation-position
│                                    #   detection (research.md §3); Fixed edit collection;
│                                    #   Auto alignment-run detection + target-column
│                                    #   computation (research.md §6, data-model.md §3)
├── src/format.rs                    # FormatOptions gains operator_spacing: OperatorSpacing
│                                    #   (manual Default impl already exists post-017, gains
│                                    #   one more field). render() gains a SpacingEdit
│                                    #   collection call (short-circuited on Preserve, mirroring
│                                    #   casing's existing short-circuit) and a per-line
│                                    #   left-to-right rebuild path used only when spacing edits
│                                    #   exist for that line (data-model.md §2) — the existing
│                                    #   same-length CasingEdit splice is untouched for lines
│                                    #   with no spacing edits. Module doc comment reworded
│                                    #   (research.md §8, data-model.md §5).
└── src/lib.rs                       # re-exports OperatorSpacing

crates/drut-config/
├── src/lib.rs                       # FormatConfig/ExplicitFormatOverride gain
│                                    #   operator_spacing: Option<OperatorSpacing>.
│                                    #   resolve_format_options implements the single-tier
│                                    #   precedence (data-model.md §4) and the
│                                    #   invalid-value-falls-back-to-preserve validation
├── src/parse.rs                     # new TOML field parsed with the same
│                                    #   malformed-value-warns-and-falls-back pattern already
│                                    #   used for every existing [format] field
├── tests/parse.rs                   # new cases for the field + its three accepted values
└── tests/resolve.rs                 # new cases for precedence + invalid-value fallback

crates/drut-cli/
├── src/cli.rs                       # new --operator-spacing flag (same ValueEnum shape as
│                                    #   --casing/--top-level-indent)
├── src/format_cmd.rs                # wires the new flag into ExplicitFormatOverride
└── tests/format_flags.rs            # new cases

crates/drut-mcp/
└── src/format.rs                    # new operator_spacing param, same shape as existing
                                     #   casing/top_level_indent params; own test module extended

crates/drut-lsp/                     # no source changes — untouched call sites compile through
                                     #   the type change unchanged, same as `014`/`017`;
                                     #   existing test suite passing unmodified is the
                                     #   confirmation

ROADMAP.md                           # item 12 marked done on completion
```

**Structure Decision**: No new crate. One new `voyager-core` module (`operator_spacing.rs`), one
new `FormatOptions` field, one new render-pipeline capability (`SpacingEdit`
insertion/removal-capable application, additive alongside the existing same-length `CasingEdit`
splice). Every adapter-layer change is a small, additive, symmetric extension of the pattern
`top_level_indent`/`casing`/`indent_width` already established in the exact same files/
functions — no new architectural pattern beyond the one genuinely new piece (`SpacingEdit`'s
variable-length application), which is itself scoped to `format.rs::render` alone.

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new dependencies, no new crates.
The one genuinely new architectural piece (`SpacingEdit`'s variable-length edit application) is
justified directly by FR-002/FR-004/FR-005, which cannot be satisfied any other way (inserting
or removing whitespace is not expressible through the existing same-length `CasingEdit`
mechanism, by construction — research.md §4).*
