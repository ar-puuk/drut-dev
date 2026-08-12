# Implementation Plan: FMT Region Markers

**Branch**: `010-fmt-region-markers` | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/010-fmt-region-markers/spec.md`

## Summary

Recognize a whole-line `; FMT: OFF` / `; FMT: ON` comment pair (via the existing tokenizer's `TokenKind::LineComment`, not a new lexer/grammar concept — research.md §1) and gate `format.rs`'s existing indent-plan and casing-edit collection so no plan/edit entry is ever produced for a line inside the marked range (research.md §2) — the render loop itself needs no change, since a line with no plan entry and no casing edit is already reproduced untouched. An unmatched `; FMT: OFF` protects through end-of-file (matching Python Black's `# fmt: off`/`# fmt: on` precedent) and is surfaced via a new, dedicated `FormatResult.unclosed_fmt_off_markers` field plus a standalone `unclosed_fmt_off_markers(source)` function — deliberately **not** a new `Diagnostic`/`DiagnosticKind` variant (owner's explicit steer during spec review) — consumed three different ways: `drut-cli` prints a non-fatal stderr notice (mirroring its existing `recovered_encoding_files`/`unsafe_encoding_files` pattern), `drut-mcp` adds a response field, and `drut-lsp` publishes it as a `HINT`-severity, `"drut-fmt"`-sourced diagnostic through its existing independent `diagnostics.rs` publish cycle (not the formatting handler itself, since document text properties should surface on open/edit, not only when Format Document is triggered). No new `TokenKind`, `Node`/`Block`/`Statement` shape, `FormatOptions` field, CLI flag, or LSP capability — protection is driven entirely by in-file markers (research.md §5).

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition — unchanged.

**Primary Dependencies**: None new — `voyager-core` keeps zero runtime dependencies (FR-027 in `001`'s own spec); marker-syntax matching is a 4-line manual split/trim/compare, no regex crate needed (research.md §4).

**Storage**: N/A.

**Testing**:
- `crates/voyager-core/src/format.rs`'s own `#[cfg(test)] mod tests` — new tests for every Edge Case in spec.md (protected range with wrong indentation/casing left untouched; lines outside a region normalized as usual; duplicate `; FMT: OFF`/stray `; FMT: ON` no-ops; a region straddling a block boundary; a whole-file region; a protected block opener whose out-of-region children still anchor correctly — research.md §2's "opener residue" case) plus direct tests of `unclosed_fmt_off_markers`'s standalone return value.
- `crates/voyager-core/tests/format_corpus.rs` — new hand-written fixtures with synthetic marker pairs (existing fixtures untouched, since none contain markers today — SC-002 is a zero-diff regression check on the existing set, not something this feature's own fixtures need to prove).
- `crates/drut-cli/tests/format_flags.rs` — new coverage confirming a protected range survives `drut format`/`--check`/`--diff`/`--write` identically, plus the new stderr notice's exact text for an unclosed marker.
- `crates/drut-lsp/src/formatting.rs`/`range_formatting.rs` — new tests confirming a protected range survives both LSP formatting handlers unchanged.
- `crates/drut-lsp/src/diagnostics.rs` — new test confirming an unclosed `; FMT: OFF` publishes exactly one `HINT`-severity, `"drut-fmt"`-sourced, `UnclosedFmtOff`-coded diagnostic, with **no** change to any existing structural-diagnostic assertion (purely additive stream).
- `crates/drut-mcp/src/format.rs` — new test confirming `unclosed_fmt_off_lines` is populated correctly and empty in the common case.
- Full 161-file corpus revalidation across `drut-cli`/`drut-lsp`/`drut-mcp` (this session's established standard) — a pure regression check, since no real corpus file contains markers today.

**Target Platform**: Cross-platform, unchanged.

**Project Type**: `voyager-core` core change (one new internal scan function, one new public function, one new `FormatResult` field, four/one gated call sites in existing collection functions — research.md §5 confirms zero new lexer/parser/grammar shapes) plus small, additive adapter-layer wiring in all three adapters (new report field in `drut-cli`, new response field in `drut-mcp`, new independent diagnostics-publish source in `drut-lsp`).

**Performance Goals**: No measurable change — one extra `tokenize`-and-scan pass per `format`/`format_bytes` call (source is already tokenized once by `parse` internally; `render` gains its own call per contracts/fmt-region-markers.md's exact diff shape, a single additional linear pass over a source file that's already parsed).

**Constraints**:
- MUST NOT change output for any file containing no `; FMT: OFF`/`; FMT: ON` markers (FR-004) — verified as a zero-diff check against the entire existing golden-fixture corpus, not assumed.
- MUST leave every protected line byte-for-byte identical, with no exception for indentation, casing, or any other rule this renderer currently applies (FR-003).
- MUST NOT introduce a new `DiagnosticKind` variant (FR-010, Assumptions) — the unclosed-marker notice is a dedicated, non-`Diagnostic` signal throughout.
- MUST gate at collection time, not render time (research.md §2's "opener residue" finding) — a child of a protected block opener must anchor to the opener's true on-disk column, not a planned-but-discarded value.
- MUST regenerate no existing golden fixture (SC-002) — new fixtures only, additive.

**Scale/Scope**: Same 161-file corpus. The existing `real_corpus/` set is used only for regression revalidation (zero real corpus file contains markers today, so it must stay byte-identical — SC-002). This feature's own SC-001 proof comes from two additive fixture sets: new hand-written fixtures, and (added after `/speckit-analyze` review — SC-001 explicitly requires this) a small sample of real-world script shapes derived from the existing corpus with synthetic marker pairs inserted, kept as a separate new fixture set rather than modifying `real_corpus/` itself (quickstart.md step 5, tasks.md T010/T011).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | Marker recognition reuses the existing tokenizer's `LineComment` recognition rather than a new adapter-side or ad hoc text scan (research.md §1) — no grammar/parsing logic duplicated anywhere outside `voyager-core`. All three adapters consume `voyager-core`'s new public surface (`FormatResult` field, standalone function) as thin, additive wiring only. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | No text derived from Cube Voyager vendor documentation. The unclosed-marker-protects-to-EOF *semantics* are modeled on Python Black's `# fmt: off`/`# fmt: on` — a general open-source tooling convention, not Bentley/Citilabs documentation, so Principle II's scope (vendor docs, and the separately-resolved Bhereth extension case) doesn't apply. |
| III. Formatter Idempotence & Behavior Preservation | **PASS, re-verified not assumed** | Idempotence holds trivially for protected content (never touched, so a second pass finds nothing new to change) and is unaffected for unprotected content (same collection logic as before, just with some lines excluded) — asserted directly in quickstart.md step 4, not inferred. No statement reordering, no continuation-line change, no meaning change — this feature only ever *removes* candidate edits from what would otherwise be produced, never adds a new kind of edit. Golden-fixture diff review applies (SC-002 requires a *zero*-diff outcome against the existing set, the strictest possible version of this gate). |
| IV. False Negatives Over False Positives | **N/A** | Governs linter diagnostics; the unclosed-marker notice is explicitly not a `Diagnostic`/`DiagnosticKind` (FR-010), and no existing diagnostic category is added, changed, or suppressed. |
| V. Vertical, Independently-Usable Increments | **PASS** | Single, atomic, independently valuable and testable change. Does not start until `009-top-level-indent-toggle` is merged and green on `main` (confirmed prerequisite). |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **PASS** | The unclosed-marker notice uses `textDocument/publishDiagnostics` with a standard `DiagnosticSeverity`/`source`/`code` shape — no VS Code-proprietary API introduced, works in any LSP-capable editor exactly like the existing structural diagnostics. |
| VII. Naming Honesty | **PASS** | `; FMT: OFF`/`; FMT: ON` name exactly what they do (turn formatting off/on for a range); the `UnclosedFmtOff` diagnostic code names exactly the condition it reports. |
| VIII. Public/Private Boundary | **PASS** | `voyager-core`/`drut-cli`/`drut-lsp`/`drut-mcp` are already public; no vendor-documentation-corpus content involved. |

No unjustified violations. No Complexity Tracking entries.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/quickstart.md): `contracts/fmt-region-markers.md`'s exact gate-point diff and the LSP diagnostic-shape decision confirm the Principle I/III/VI framing above holds precisely — no row's status changed. The one design decision most worth re-flagging at this checkpoint: gating happens at *collection* time (inside `plan_indentation`/`plan_block`/`plan_children`/`push_if_present`), not at the final render loop, specifically because render-time filtering was shown in research.md §2 to produce an incorrect "opener residue" case for a protected block opener with unprotected children — re-confirmed correct here, not re-litigated.

## Project Structure

### Documentation (this feature)

```text
specs/010-fmt-region-markers/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output (/speckit-plan command)
├── data-model.md                  # Phase 1 output (/speckit-plan command)
├── quickstart.md                  # Phase 1 output (/speckit-plan command)
├── contracts/                     # Phase 1 output (/speckit-plan command)
│   └── fmt-region-markers.md        # exact marker-recognition rule, gate-point
│                                    # diff shape, and per-adapter FR-010 wiring
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/voyager-core/
├── src/format.rs                    # + protected_regions() (internal scan
│                                    #   over TokenKind::LineComment,
│                                    #   research.md §1-2); + pub fn
│                                    #   unclosed_fmt_off_markers(); +
│                                    #   FormatResult.unclosed_fmt_off_markers
│                                    #   field; plan_indentation/plan_block/
│                                    #   plan_children gain a &BTreeSet<u32>
│                                    #   protected param, each plan.insert
│                                    #   guarded; push_if_present gains the
│                                    #   same guard (single funnel point for
│                                    #   all casing edits); own test module
│                                    #   gains coverage for every spec.md
│                                    #   Edge Case
├── src/lib.rs                       # re-export unclosed_fmt_off_markers
└── tests/
    └── fixtures/golden/               # + new hand-written fixtures with
                                       #   synthetic marker pairs (existing
                                       #   fixtures untouched — SC-002)

crates/drut-cli/
├── src/format_cmd.rs                # FormatReport gains
│                                    #   unclosed_fmt_off_files field,
│                                    #   populated in the existing per-file
│                                    #   loop; print_report gains a third
│                                    #   eprintln! block mirroring the two
│                                    #   existing encoding-notice blocks; no
│                                    #   derive_exit_outcome change
│                                    #   (informational only)
└── tests/format_flags.rs            # new marker-protection + unclosed-
                                     #   notice coverage

crates/drut-mcp/
├── src/format.rs                    # FormatResultDto gains
│                                    #   unclosed_fmt_off_lines: Vec<u32>,
│                                    #   mapped from result.
│                                    #   unclosed_fmt_off_markers
└── (own test module, in format.rs)  # new coverage

crates/drut-lsp/
├── src/diagnostics.rs                # publish() gains a second,
│                                    #   independently-sourced diagnostics
│                                    #   stream from
│                                    #   voyager_core::unclosed_fmt_off_markers
│                                    #   — HINT severity, "drut-fmt" source,
│                                    #   "UnclosedFmtOff" code, additive to
│                                    #   the existing structural-diagnostics
│                                    #   map/collect; own test module gains
│                                    #   coverage
├── src/formatting.rs                # no code change; new test confirming
│                                    #   a protected range survives
│                                    #   textDocument/formatting unchanged
└── src/range_formatting.rs           # same: no code change, new test

specs/002-cli-check-format/
├── spec.md                          # new FR (numbered against the live
│                                    #   file at implementation time, per
│                                    #   009's own "FR number collision"
│                                    #   precedent) for the marker-protection
│                                    #   and unclosed-notice requirements
└── contracts/formatting-api.md       # one new sentence describing the
                                     #   protected-region exception to every
                                     #   other rule in the renderer's scope
```

**Structure Decision**: No new crate, no new module, no new `TokenKind`/`Node`/`DiagnosticKind`. `voyager-core` gains one internal scan function, one public function, one struct field, and a small number of one-line guards inserted at already-existing edit-producing call sites (research.md §2's four `plan.insert` sites plus `push_if_present`'s single funnel point). All adapter-layer changes are equally small and purely additive: one new report field + one new stderr block in `drut-cli`, one new response field in `drut-mcp`, one new independently-sourced diagnostics stream (no new capability registration) in `drut-lsp`. The existing "no plan entry, no casing edit ⇒ line reproduced untouched" property of `render`'s final loop is exactly what makes protection work for free once collection is gated — no new fallback or rendering logic needed, matching the same pattern `007`, `008`, and `009` each independently found and reused in this same module.

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new dependencies, no new architectural components.*
