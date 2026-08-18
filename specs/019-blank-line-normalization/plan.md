# Implementation Plan: Blank-Line-Run Normalization

**Branch**: `019-blank-line-normalization` | **Date**: 2026-08-17 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/019-blank-line-normalization/spec.md`

## Summary

One new `voyager-core` formatting axis: `FormatOptions.blank_lines: BlankLineMode` (`Preserve`/
`Auto`, default `Preserve`) plus two new caps (`top_level_blank_line_cap: u8` default `2`,
`nested_blank_line_cap: u8` default `1`), implemented as a new self-contained module
(`src/blank_line.rs`) mirroring `data_reference.rs`/`operator_spacing.rs`'s established shape: a
read-only recognition pass over already-parsed `Node`/line data, no lexer/parser change.

The one genuinely new piece (research.md §1): `render()`'s per-line emission loop has never
needed to *delete* a line before — every prior formatting axis operates on a strict
1-input-line-to-1-output-line correspondence. A small `lines_to_delete: BTreeSet<u32>` computed
before the loop, checked with one early-exit `continue` per iteration, adds this capability
without touching how indentation/casing/operator-spacing edits are computed or applied — they
already work purely in terms of original line numbers, unaffected by which lines end up deleted.

Two other findings materially simplify the implementation versus what the requirement might
suggest at first read (research.md §3, §4): a blank-line run can never straddle a block boundary
or a protected-region boundary (both are bounded by non-blank lines), so a run's top-level/nested
classification and protection status are uniform across the whole run — no per-line
straddling case to handle. And "any nesting depth" classification needs no recursion at all: a
nested block's span is always contained within its parent's, so marking only *top-level*
blocks' own span ranges as "nested" already correctly classifies every line at every depth.

Every existing config surface stays exactly as it is — a purely additive change (spec FR-009),
never a breaking one.

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition — unchanged.

**Primary Dependencies**: None new — `voyager-core` remains zero-runtime-dependency.

**Storage**: N/A.

**Testing**:
- `crates/voyager-core/src/blank_line.rs` (new) — unit tests for `nested_lines` (top-level block
  span marking, no recursion needed per research.md §4), `find_blank_runs` (whitespace-only-counts-
  as-blank, run boundaries), and `lines_to_delete` (cap application, first-N-survive, `; FMT: OFF`
  exclusion, doubly-nested-uses-the-same-cap).
- `crates/voyager-core/src/format.rs` — new unit tests for the `render()` line-deletion
  integration (a deleted line genuinely absent from output, not just blanked; indentation/casing/
  spacing edits for *surviving* lines unaffected; `Preserve`-short-circuit performance test
  mirroring every other axis's own).
- `crates/voyager-core/tests/format_corpus.rs`/`format_sequence.rs` — no golden-fixture
  regeneration expected for the *existing* golden set (FR-009: byte-identical when unconfigured);
  new golden fixtures added specifically exercising `Auto`, verified by hand before being trusted,
  following `017`/`018`'s own precedent.
- `crates/drut-config/tests/parse.rs`/`resolve.rs` — new cases for the mode field and both caps,
  the invalid-value-falls-back-to-default case per field.
- `crates/drut-cli/tests/format_flags.rs` — new cases for the three new flags.
- `crates/drut-mcp/src/format.rs` test module — same shape, MCP-side.
- Full real-corpus revalidation (CLI/LSP/MCP) — expected zero diagnostic/output change with no
  new configuration supplied (SC-003).

**Target Platform**: Cross-platform, unchanged.

**Project Type**: `voyager-core` core change (one new module, three new `FormatOptions` fields,
one new render-pipeline capability — line deletion) plus symmetric, additive adapter-layer wiring
in `drut-config`/`drut-cli`/`drut-mcp` (`drut-lsp` untouched behaviorally, same as every prior
formatting feature in this project).

**Performance Goals**: No measurable regression when `blank_lines == Preserve` (the default) —
the entire `lines_to_delete` computation is skipped via the same short-circuit pattern every
other axis already uses. When `Auto` is configured, cost is one linear pass to build
`nested_lines` (top-level blocks only, no recursion — research.md §4) plus one linear pass over
`char_lines` to find runs — no re-tokenization, no re-parsing.

**Constraints**:
- MUST NOT change formatter output for any existing input when `blank_lines` isn't configured
  (FR-009) — confirmed by the full existing golden-fixture set and corpus passing byte-for-byte
  unmodified.
- MUST NOT remove or change the meaning of any already-shipped config surface — this feature
  adds exactly three new fields/flags/params, nothing else changes shape.
- MUST NOT introduce a lexer/parser change — both `nested_lines` and blank-run detection are
  solvable as read-only scans over already-parsed `Node` data and the existing per-line `char`
  representation `render()` already builds (research.md §4).
- MUST NOT alter a surviving line's own content, even a whitespace-only one (FR-006) — deletion
  only, never a rewrite of what's kept.
- MUST NOT pad a shorter run up to the cap (FR-004) — a maximum only, never a minimum.
- MUST NOT change which `voyager-core` grammar rules exist or how any node is structured
  (Principle I/III) — `blank_line.rs` is a read-only recognition pass over already-parsed data,
  the same architectural shape every prior recognition module in this crate already uses.
- Invalid cap values MUST degrade to that cap's own built-in default with a non-blocking notice
  (FR-011), the same established pattern `indent_width` already uses.

**Scale/Scope**: Same 161-file real corpus, revalidated for zero change with no new configuration
supplied, plus new hand-verified golden fixtures for `Auto` specifically (including at least one
real file with an excessive top-level run and one with an excessive nested run, to exercise both
caps against genuine corpus shapes).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | `blank_line.rs`'s recognition logic lives entirely in `voyager-core`; every adapter gains only thin field/flag/param mapping, mirroring `top_level_indent`/`indent_width`'s already-established pattern. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | This feature is a formatting-style decision (with general programming-tooling precedent, `black`/`prettier`), not a vendor-grammar fact — no vendor documentation consulted. |
| III. Formatter Idempotence & Behavior Preservation | **PASS, re-verified not assumed** | `blank_lines` is a new, opt-in axis — FR-009/SC-003 require zero output change with nothing configured, confirmed by the full existing corpus/golden set passing unmodified. `Auto`'s own idempotence (SC-005) is verified directly. `; FMT: OFF`/`ON` protection (FR-010) is re-verified against this new axis specifically, not assumed to hold transitively. |
| IV. False Negatives Over False Positives | **N/A** | Governs diagnostics; no diagnostic category is added, changed, or suppressed by this feature. |
| V. Vertical, Independently-Usable Increments | **PASS** | US1 (top-level cap) and US2 (nested cap) are independently valuable and independently testable (spec.md) — each configurable and observable without the other. |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **N/A** | No new editor-integration surface — `drut-lsp` gains only the type/field changes rippling through its existing format-on-save/format-on-paste call sites. |
| VII. Naming Honesty | **PASS** | `BlankLineMode`/`top_level_blank_line_cap`/`nested_blank_line_cap` name exactly what they do; no overclaiming. |
| VIII. Public/Private Boundary | **PASS** | All touched crates are already public. No vendor-doc-derived material introduced. |

No unjustified violations. No Complexity Tracking entries.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/quickstart.md):
`contracts/blank-line-normalization.md`'s exact type/precedence inventory confirms the Principle
I/III framing above holds precisely — no row's status changed. The one genuinely new
architectural piece (`render()`'s line-deletion capability) stays entirely inside `render()`
itself, not a new public surface, so it doesn't change this table either.

## Project Structure

### Documentation (this feature)

```text
specs/019-blank-line-normalization/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output
├── data-model.md                  # Phase 1 output
├── quickstart.md                  # Phase 1 output
├── contracts/
│   └── blank-line-normalization.md   # exact type shapes, precedence, deletion contract
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
crates/voyager-core/
├── src/blank_line.rs (new)          # BlankLineMode enum; nested_lines (top-level-block-span
│                                    #   marking, research.md §4); find_blank_runs
│                                    #   (whitespace-only-counts, research.md §2); lines_to_delete
│                                    #   (cap application, FMT:OFF exclusion, research.md §3/§5)
├── src/format.rs                    # FormatOptions gains blank_lines: BlankLineMode,
│                                    #   top_level_blank_line_cap/nested_blank_line_cap: u8
│                                    #   (manual Default impl extended). render() gains a
│                                    #   lines_to_delete computation (short-circuited on
│                                    #   Preserve) and one early-exit `continue` in the main
│                                    #   emission loop (research.md §1) — the one new capability
│                                    #   this feature needs, otherwise untouched.
└── src/lib.rs                       # re-exports BlankLineMode

crates/drut-config/
├── src/lib.rs                       # FormatConfig/ExplicitFormatOverride gain
│                                    #   blank_lines: Option<BlankLineMode>,
│                                    #   top_level_blank_line_cap/nested_blank_line_cap:
│                                    #   Option<u8>. resolve_format_options implements the
│                                    #   single-tier precedence per setting (data-model.md §3)
│                                    #   and the cap-range-with-fallback validation, mirroring
│                                    #   resolve_indent_width
├── src/parse.rs                     # new TOML fields parsed with the same
│                                    #   malformed-value-warns-and-falls-back pattern already
│                                    #   used for every existing [format] field
├── tests/parse.rs                   # new cases per new field
└── tests/resolve.rs                 # new cases for precedence + invalid-value fallback

crates/drut-cli/
├── src/cli.rs                       # new --blank-lines / --top-level-blank-line-cap /
│                                    #   --nested-blank-line-cap flags
├── src/format_cmd.rs                # wires new flags into ExplicitFormatOverride
└── tests/format_flags.rs            # new cases

crates/drut-mcp/
└── src/format.rs                    # new blank_lines/top_level_blank_line_cap/
                                     #   nested_blank_line_cap params; own test module extended

crates/drut-lsp/                     # no source changes — untouched call sites compile through
                                     #   the type change unchanged, same as every prior feature

ROADMAP.md                           # item 13 marked done on completion
```

**Structure Decision**: No new crate. One new `voyager-core` module (`blank_line.rs`), three new
`FormatOptions` fields, one new render-pipeline capability (line deletion, scoped entirely inside
`render()`). Every adapter-layer change is a small, additive, symmetric extension of the pattern
`top_level_indent`/`indent_width` already established in the exact same files/functions — no new
architectural pattern beyond the one genuinely new piece, which is itself minimal (one
`BTreeSet<u32>` and one early-exit check).

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new dependencies, no new crates.
The one genuinely new architectural piece (line-deletion capability) is justified directly by
FR-003/FR-006, which cannot be satisfied any other way (removing excess lines is not expressible
through any existing same-line-count edit mechanism, by construction — research.md §1).*
