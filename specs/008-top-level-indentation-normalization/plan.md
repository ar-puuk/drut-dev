# Implementation Plan: Top-Level Indentation Normalization

**Branch**: `008-top-level-indentation-normalization` | **Date**: 2026-08-11 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/008-top-level-indentation-normalization/spec.md`

## Summary

Reverse FR-012's original "top-level lines are never touched" policy:
`plan_indentation` now force-plans **every** top-level node's own line
(statement or block opener alike) to column 0 unconditionally, before
`plan_block` computes each block's `base` for its children — so the
existing per-nesting-level logic composes correctly with the new,
always-0 top-level anchor with no changes needed to `plan_block` itself
beyond that one pre-seed. `007-formatter-diagnosed-block-indent-fix`'s
skip-a-diagnosed-block's-children logic is kept, unchanged in code, but
its role narrows and its own rationale gets rewritten (research.md §1):
it was never actually protecting the *opener* line (top-level-never-
touched already did that, separately); it only ever protected *children*
whose relationship to a genuinely-unmatched block is structurally
uncertain — a concern this feature doesn't touch or resolve.

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition — unchanged.

**Primary Dependencies**: None new.

**Storage**: N/A.

**Testing**:
- `crates/voyager-core/src/format.rs`'s own `#[cfg(test)] mod tests` —
  new cases for the always-0 top-level rule (bare statement, block
  opener, block-with-stale-children), updated cases where an existing
  test's "already correctly formatted" fixture relied on a non-zero
  top-level baseline (spec.md Edge Cases; each found and reviewed
  individually, not assumed away).
- `crates/voyager-core/tests/format_sequence.rs` — the `007`-era
  `process_run_residue_is_fixed_after_endprocess_is_added`/
  `run_if_residue_is_fixed_after_endrun_is_added` tests' own assertions
  need re-checking against the new policy (pass 1 may no longer be a
  pure no-op, since a still-diagnosed block's *opener* is now
  unconditionally corrected even though its children still aren't) — new
  test confirming FR-005/SC-002 explicitly (the residue fully resolves in
  the second pass, regardless of what pass 1 did to the opener).
- `crates/voyager-core/tests/format_corpus.rs` — golden-fixture
  regeneration (FR-006), human-reviewed diff per file, `UPDATE_GOLDEN=1`
  workflow the suite's own module doc already documents.
- Full 161-file corpus revalidation across `drut-cli`/`drut-lsp`/
  `drut-mcp`, reported as its own explicit result (this session's
  established standard).

**Target Platform**: Cross-platform, unchanged.

**Project Type**: `voyager-core`-only behavioral change (one function's
logic), plus a documentation/spec amendment and a golden-fixture
regeneration pass — no new crate, no adapter code change (confirmed,
not assumed — research.md §2).

**Performance Goals**: No measurable change — one additional `BTreeMap`
insert per top-level node, same order of magnitude as every other
per-line plan entry already computed.

**Constraints**:
- MUST force every top-level node (statement or block) to column 0,
  unconditionally (FR-001/FR-002) — including bare top-level statements,
  which had *no* code path touching them at all before this feature
  (research.md §1 — a real, previously-existing gap this feature also
  closes, not just a policy flip on already-planned lines).
- MUST NOT change nested-indentation logic beyond re-anchoring it to the
  new base (FR-003) — `plan_block`'s per-level/closer-alignment/branch-
  alignment logic is otherwise untouched.
- MUST NOT remove or weaken `007`'s diagnosed-block children skip without
  an explicit, recorded decision (FR-004) — resolved in research.md §1:
  kept, unchanged, rationale narrowed.
- MUST regenerate and individually human-review every affected golden
  fixture before merge (FR-006, constitution Principle III's existing
  gate).

**Scale/Scope**: Same 161-file corpus; `format_corpus.rs`'s 9
`real_corpus/` fixtures plus any hand-written `valid/` fixture with
non-zero top-level indentation are the golden-regeneration surface
(research.md §3 identifies the exact file list).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | Change is entirely within `voyager-core::format::plan_indentation` — no adapter gains or needs new logic. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | No new text derived from vendor documentation. |
| III. Formatter Idempotence & Behavior Preservation | **PASS, re-verified not assumed** | Idempotence is arguably *strengthened* by this change — the fixed point no longer depends on the input's own starting top-level indentation, only one canonical column-0 output exists per structural shape. Behavior preservation (no reordering, no continuation changes, no meaning change) is unaffected — this is still whitespace-only. Golden-file diff review (FR-006) is this principle's own existing gate, applied here explicitly, not skipped. |
| IV. False Negatives Over False Positives | **PASS** | This principle governs *diagnostics*, not formatting style; no diagnostic category is added, changed, or suppressed by this feature. |
| V. Vertical, Independently-Usable Increments | **PASS** | Single, atomic, independently valuable and testable change. Does not start until `007`'s own fixture-corpus tests pass cleanly (already true, merged to `main`). |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **N/A** | No editor-integration surface touched. |
| VII. Naming Honesty | **N/A** | No new named capability. |
| VIII. Public/Private Boundary | **PASS** | `voyager-core` is already public; no vendor-documentation-corpus content involved. |

No unjustified violations. No Complexity Tracking entries.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/
quickstart.md): `contracts/top-level-indentation.md`'s exact algorithm
confirms the Principle I/III framing above holds precisely — no row's
status changed.

## Project Structure

### Documentation (this feature)

```text
specs/008-top-level-indentation-normalization/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output (/speckit-plan command)
├── data-model.md                  # Phase 1 output (/speckit-plan command)
├── quickstart.md                  # Phase 1 output (/speckit-plan command)
├── contracts/                     # Phase 1 output (/speckit-plan command)
│   └── top-level-indentation.md     # exact plan_indentation algorithm +
│                                    # the FR-004 (007-interaction) resolution
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/voyager-core/
├── src/format.rs                    # plan_indentation gains the top-level
│                                    # force-to-0 pre-seed; plan_block
│                                    # unchanged (base already resolves
│                                    # correctly via computed_indent); own
│                                    # test module gains new cases, updates
│                                    # existing ones whose baseline assumed
│                                    # the old policy
├── tests/format_sequence.rs         # 007-era tests' assertions re-checked;
│                                    # new test for FR-005/SC-002 (residue
│                                    # resolves in the second pass, full
│                                    # stop, regardless of pass 1's own
│                                    # opener-line behavior)
└── tests/
    ├── fixtures/golden/               # regenerated, human-reviewed
    │   ├── *.s                          # (hand-written set)
    │   └── real_corpus/**/*.s           # (all 9 real files)
    └── fixtures/valid/                 # any fixture whose own top-level
                                        # indentation needs adjusting so it
                                        # stays a genuine "valid, zero-
                                        # diagnostic" input under the new
                                        # rule (research.md §3)

specs/002-cli-check-format/
├── spec.md                          # FR-012 + Assumptions amended (FR-007)
└── contracts/formatting-api.md       # "top-level baseline... left
                                     # untouched" line amended to match
```

**Structure Decision**: No new crate, no new module. One function
(`plan_indentation`) gains a small, additive pre-seed step;
`plan_block`/`plan_children`/`computed_indent` are unchanged — the
existing "prefer a planned value over the original" fallback in
`computed_indent` is precisely what makes this a minimal, low-risk change
rather than a rewrite.

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new
dependencies, no new architectural components.*
