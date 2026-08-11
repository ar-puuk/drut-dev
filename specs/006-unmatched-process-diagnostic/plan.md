# Implementation Plan: UnmatchedProcess Diagnostic

**Branch**: `006-unmatched-process-diagnostic` | **Date**: 2026-08-11 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/006-unmatched-process-diagnostic/spec.md`

## Summary

Add `DiagnosticKind::UnmatchedProcess` to `voyager-core`, firing under the
exact same condition `UnmatchedRun` already uses for `RUN`
(`crates/voyager-core/src/block.rs`'s `parse_run`, lines 502-556) —
`parse_process` gains the identical implicit-closer check and diagnostic
push, mirroring `parse_run`'s structure line for line rather than
inventing a new shape. Motivated and de-risked by a real 161-file corpus
investigation (123 real `Process` blocks, all 123 explicitly closed — zero
false-positive risk, confirmed empirically, not assumed).

**Scope correction made during planning** (spec.md FR-007/Assumptions):
this is *not* the voyager-core-only change it was first assumed to be.
Three adapters — `drut-cli` (`report/sarif.rs`), `drut-lsp`
(`diagnostics.rs`), `drut-mcp` (`diagnose.rs`) — each maintain an
*exhaustive* `match` over `DiagnosticKind` with no wildcard arm, confirmed
by reading each file directly. Adding a new variant without updating all
three is a compile error. The *decision* logic stays 100% in
`voyager-core` (Principle I unaffected) — the adapter work is the same
thin, non-decision-making naming/rendering category each already does for
the other six kinds.

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition — unchanged,
matches the rest of the workspace.

**Primary Dependencies**: None new. `voyager-core` stays at zero runtime
dependencies (constitution Principle I, FR-027) — this feature adds one
enum variant and one `if`/diagnostic-push branch, nothing else.

**Storage**: N/A.

**Testing**:
- `crates/voyager-core/src/block.rs`'s own `#[cfg(test)] mod tests` —
  direct unit tests for `parse_process`'s new branch, mirroring
  `parse_run`'s existing test shape exactly (`unmatched_run_at_eof_is_diagnosed`-
  style tests already there are the template).
- `crates/voyager-core/tests/fixture_corpus.rs` — gains a new broken
  fixture (FR-009) reproducing the real-world shape (a `PROCESS` with no
  closer, followed by real subsequent content — not a one-line synthetic
  case), plus two small, mechanical test-helper updates:
  `parse_diagnostic_kind`'s string-to-kind match, and
  `every_diagnostic_category_has_at_least_one_broken_fixture`'s hardcoded
  kind array (research.md §1).
- Full 161-file real corpus revalidation (`DRUT_CORPUS_PATH`-gated,
  `--ignored`) — FR-008's required re-verification that the corpus stays
  100% clean, run and reported as its own explicit result (this session's
  established standard for core-crate changes), not folded into a general
  "tests pass."
- `crates/drut-cli`, `crates/drut-lsp`, `crates/drut-mcp` — each crate's
  existing diagnostic-rendering tests continue to pass unchanged (none of
  them assert an exhaustive kind *count*, only per-kind behavior — research.md
  §1 confirms this per file) once each adapter's new match arm is added.

**Target Platform**: Cross-platform (Windows/macOS/Linux), unchanged.

**Project Type**: Library change (`voyager-core`) plus three small,
mechanical adapter updates (`drut-cli`, `drut-lsp`, `drut-mcp`) — no new
crate, no new binary, no new workspace member.

**Performance Goals**: No measurable change — one additional `if` branch
and array/match entry per adapter; same order of magnitude as every other
`DiagnosticKind` addition this project has already made.

**Constraints**:
- MUST mirror `UnmatchedRun`'s firing condition exactly — no new
  structural-matching logic invented (Principle I: reuse the established
  pattern, don't create a second way to express the same kind of rule).
- MUST NOT flag the legitimate implicit-close-by-sibling pattern (FR-004).
- MUST NOT change `Block`'s existing structural representation
  (`closer: Option<Span>`, `BlockKind::Process`) — additive diagnostic
  signal only (spec.md Assumptions).
- MUST NOT touch `JLoop`/`LinkLoop`/`DistributeMultistep` in any way
  (FR-010) — explicitly out of scope.
- MUST keep the full real corpus at zero diagnostics (FR-008).

**Scale/Scope**: Same 161-file WF-TDM-Official-Releases corpus every prior
phase validates against; this feature's own motivating investigation
already characterized every real `Process` block in it (123 found, all
explicitly closed).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | The firing-condition *decision* lives entirely in `voyager-core::block::parse_process`, mirroring `parse_run`'s existing logic exactly — no new grammar/parsing rule invented. The three adapter updates (research.md §1) are thin naming/rendering additions of the same kind every adapter already performs for the other six `DiagnosticKind` variants, not new decision logic. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | The new diagnostic's message and SARIF `shortDescription` are original wording, composed the same way every existing kind's text already is — no vendor documentation consulted or copied. |
| III. Formatter Idempotence & Behavior Preservation | **N/A this phase** | No formatter logic touched — `format.rs` is untouched by this feature. |
| IV. False Negatives Over False Positives | **PASS, and the whole reason this feature is low-risk** | The new diagnostic category ships with an *empirically proven* zero-false-positive record against the real corpus (123/123 real `Process` blocks already explicitly closed) — confirmed before writing code, re-confirmed after (FR-008), not assumed either time. |
| V. Vertical, Independently-Usable Increments | **PASS** | Single, atomic, independently valuable and testable increment. Does not start until `005-format-on-save-paste`'s own automated fixture-corpus tests pass cleanly (already true — T011/T012 both clean; this feature is unrelated to and branches independently of 005's still-pending manual verification, per no file overlap). |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **N/A this phase** | No editor-integration surface — `drut-lsp`'s change is a one-line addition to an already-LSP-standard `Diagnostic` rendering path, not a new capability. |
| VII. Naming Honesty | **PASS** | `UnmatchedProcess` names exactly what it detects, matching `UnmatchedRun`/`UnmatchedIf`/`UnmatchedLoop`'s own naming convention precisely — no overclaim. |
| VIII. Public/Private Boundary | **PASS** | All touched components (`voyager-core`, `drut-cli`, `drut-lsp`, `drut-mcp`) are already public; no vendor-documentation-corpus content involved. |

No unjustified violations. No Complexity Tracking entries — zero new
dependencies, zero new architectural components, one new enum variant plus
its mechanical adapter updates.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/
quickstart.md): `contracts/unmatched-process-diagnostic.md`'s exact
match-arm-by-file inventory (research.md §1) confirms the Principle I
framing above holds precisely — every adapter change is additive rendering
of an existing kind through an existing path, not new logic. No row's
status changed from the pre-design check above.

## Project Structure

### Documentation (this feature)

```text
specs/006-unmatched-process-diagnostic/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output (/speckit-plan command)
├── data-model.md                  # Phase 1 output (/speckit-plan command)
├── quickstart.md                  # Phase 1 output (/speckit-plan command)
├── contracts/                     # Phase 1 output (/speckit-plan command)
│   └── unmatched-process-diagnostic.md   # exact firing condition, message
│                                          # wording, and the full file-by-
│                                          # file adapter change inventory
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/
├── voyager-core/                   # existing crate; this feature's core change
│   ├── src/
│   │   ├── diagnostic.rs             # add: DiagnosticKind::UnmatchedProcess
│   │   │                                # variant + doc comment
│   │   └── block.rs                   # parse_process gains the implicit-
│   │                                  # closer check + diagnostic push,
│   │                                  # mirroring parse_run's existing
│   │                                  # structure exactly; own #[cfg(test)]
│   │                                  # tests gain the new cases
│   └── tests/
│       ├── fixture_corpus.rs          # parse_diagnostic_kind match +
│       │                              # every_diagnostic_category_has_at_
│       │                              # least_one_broken_fixture's array,
│       │                              # each gain one entry (research.md §1)
│       └── fixtures/broken/
│           └── unmatched_process_with_trailing_content.s   # NEW: FR-009's
│                                      # required real-world-shaped fixture
│
├── drut-cli/                       # existing crate; mechanical update only
│   └── src/report/sarif.rs            # ALL_KINDS array, rule_id match,
│                                      # short_description match each gain
│                                      # one entry (research.md §1) —
│                                      # report/text.rs needs NO change
│                                      # (Debug-formats diag.kind directly)
│
├── drut-lsp/                       # existing crate; mechanical update only
│   └── src/diagnostics.rs             # kind_name match gains one entry;
│                                      # module doc's "six of seven" count
│                                      # updated to seven of eight
│
└── drut-mcp/                       # existing crate; mechanical update only
    └── src/diagnose.rs                # category_name match gains one
                                       # entry; DiagnosticDto's doc comment
                                       # count updated
```

Also amends the existing `specs/001-voyager-script-parser/contracts/
diagnostics.md` (the authoritative diagnostics contract every adapter's
own docs point back to) — adds the `UnmatchedProcess` table row and
updates its "Note on block kinds without a diagnostic category" to reflect
`Process` as resolved while `JLoop`/`LinkLoop`/`DistributeMultistep`
remain deferred (FR-010), rather than leaving that note describing all
four as equally undecided.

**Structure Decision**: No new crate, no new workspace member. One new
`voyager-core` enum variant plus its firing logic (the actual feature);
four small, mechanical, non-decision-making updates across the three
adapter crates plus the shared test-fixture-corpus's own helper (research.md
§1) so nothing fails to compile or silently under-reports the new kind. The
1-voyager-script-parser diagnostics contract is amended in place rather
than superseded by a new contract file — it remains the single
authoritative diagnostics reference every adapter and consumer already
points to.

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new
dependencies, no new architectural components.*
