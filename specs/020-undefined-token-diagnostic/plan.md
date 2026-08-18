# Implementation Plan: Undefined `@token@` Diagnostic

**Branch**: `020-undefined-token-diagnostic` | **Date**: 2026-08-17 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/020-undefined-token-diagnostic/spec.md`

## Summary

A fourth Hint-severity, non-`DiagnosticKind` LSP diagnostic stream (`drut-lsp/src/
diagnostics.rs::publish`), following the exact shape two prior features already established (the
unclosed `; FMT: OFF` marker, `010-fmt-region-markers`; the malformed `drut.toml` warning,
`012-toml-configuration`) — a standalone function outside `voyager_core::Diagnostic`/
`DiagnosticKind` entirely, source `"drut-token"`, chained alongside the existing three.

Adds one new pure `voyager-core` function (`token_resolution::all_variable_refs`, an "every
match" counterpart to the existing `variable_ref_at`'s "one match at a position") and reuses
`hover.rs`'s existing `collect_included_files`/`resolve_token_value` machinery, widened from
private to `pub(crate)`, entirely unmodified otherwise. Every one of spec.md FR-003's three
"never flag a resolver blind spot" exclusions (block-opener position, multi-level `READ FILE`
inclusion, token-built inclusion path) turns out to already be satisfied automatically by reusing
these existing functions verbatim — confirmed by reading their actual behavior, not assumed
(research.md §3) — so this feature adds no new suppression logic, only new enumeration and
wiring. LSP-only by explicit decision: never reaches `drut-cli`'s `check` command or `drut-mcp`'s
`diagnose` tool, leaving `002-cli-check-format` FR-003's "never a narrowed subset of
`DiagnosticKind`" contract for those two surfaces completely untouched.

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition — unchanged.

**Primary Dependencies**: None new — `voyager-core` remains zero-runtime-dependency; `drut-lsp`
adds no new crate dependency, only reuses its own existing `lsp-types` usage.

**Storage**: N/A (the one disk read this feature triggers — `READ FILE` target resolution — is
entirely `collect_included_files`'s pre-existing, unmodified responsibility).

**Testing**:
- `crates/voyager-core/src/token_resolution.rs` — new unit tests for `all_variable_refs`:
  multiple references returned in source order; a block-opener reference absent from the result;
  `IfBranch.condition` references included; empty `Vec` (never a panic) for a document with none.
- `crates/drut-lsp/src/undefined_token.rs` (new) — unit tests covering every spec.md US1
  Acceptance Scenario directly: unresolvable reference flagged; same-file-resolved reference not
  flagged; one-level-`READ FILE`-resolved reference not flagged; two-level-inclusion reference
  flagged (correctly, since only one level is followed); token-built-path reference flagged
  (correctly, since dynamic paths are never followed).
- `crates/drut-lsp/src/diagnostics.rs` — new/extended tests: the fourth stream's exact shape
  (`HINT` severity, `"drut-token"` source, `"UndefinedToken"` code) on a fixture with one
  unresolvable reference; the six real `DiagnosticKind`-based diagnostics' severity/source
  unaffected by this feature's addition in the same fixture; live update on edit (add the missing
  assignment, confirm the notice disappears on the next publish).
- `crates/drut-cli`/`crates/drut-mcp` — no new tests needed for this feature's own behavior
  (FR-005 means neither surface is touched at all); existing `check`/`diagnose` test suites
  passing unmodified is itself the confirmation that nothing leaked into either surface.

**Target Platform**: Cross-platform, unchanged.

**Project Type**: `voyager-core` core addition (one new pure function, zero type changes) plus a
`drut-lsp`-only new module and one visibility widening in an existing module — `drut-cli`,
`drut-mcp`, `drut-config`, `drut-lsp`'s formatting/completion/other capabilities: no source
changes.

**Performance Goals**: One `collect_included_files` disk-I/O pass per `publish()` call (already
paid once per hover request today; here paid once per diagnostics publish instead — same shape,
different trigger), then one `resolve_token_value` call per `@token@` reference in the document —
linear in reference count, no new full-file re-scan beyond what `all_variable_refs`'s single
traversal already does.

**Constraints**:
- MUST NOT add a new `DiagnosticKind` variant (FR-004) — confirmed structurally unnecessary
  since the existing non-`DiagnosticKind` stream shape (research.md §1) already fits.
- MUST NOT reach `drut-cli`'s `check` command or `drut-mcp`'s `diagnose` tool (FR-005) — the new
  stream is built and published entirely inside `drut-lsp/src/diagnostics.rs`, with no shared
  code path either CLI/MCP command could accidentally pick up.
- MUST NOT flag a reference the resolver has a documented blind spot for (FR-003) — satisfied by
  reusing `all_variable_refs`/`collect_included_files`/`resolve_token_value` unmodified, not by
  writing new detection logic that could drift out of sync with what those functions actually do
  (research.md §3).
- MUST NOT change `voyager_core::Diagnostic`/`DiagnosticKind`'s existing meaning or any exhaustive
  match over it elsewhere in the workspace (Principle I/VII) — zero changes to that type.
- MUST require no new project configuration (FR-008) — no `drut.toml` field, CLI flag, or MCP
  param exists for this capability at all.

**Scale/Scope**: Single new `voyager-core` function, single new `drut-lsp` module (~50-80 lines
estimated, matching `undefined_token_positions`'s shape in data-model.md §2), one visibility
widening, one new chained stream in `diagnostics.rs`. No real-corpus revalidation gate the way
formatting features need (this changes no formatting/parsing output, only what gets published as
LSP diagnostics) — validated instead via targeted fixtures covering each Acceptance Scenario and
each of FR-003's three exclusions directly, per quickstart.md.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | The one new piece of real logic (`all_variable_refs`) lives in `voyager-core`; `drut-lsp`'s new module is a thin caller reusing existing `voyager-core`/`hover.rs` functions verbatim, not reimplementing resolution logic of its own. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | No vendor documentation is consulted or paraphrased — this is a tooling-behavior decision (which references to flag, at what confidence/severity), not a Cube Voyager grammar fact. The diagnostic message itself is original wording, hedged deliberately (data-model.md §3). |
| III. Formatter Idempotence & Behavior Preservation | **N/A** | This feature touches no formatting logic at all — `format`/`format_bytes` are completely untouched. |
| IV. False Negatives Over False Positives | **PASS, directly load-bearing** | This principle is the entire reason for this feature's scope (Hint not Error severity, the "never flag a blind spot" confidence bar, FR-003). Re-verified, not assumed: research.md §3 confirms each of the three exclusions holds by construction from the existing functions' actual behavior. |
| V. Vertical, Independently-Usable Increments | **PASS** | Single user story, independently valuable and independently testable — no bundling with unrelated work. |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **PASS** | Uses `textDocument/publishDiagnostics` with standard `DiagnosticSeverity::HINT` — no editor-proprietary decoration API, same mechanism the two prior Hint-severity streams already use. |
| VII. Naming Honesty | **PASS** | `"UndefinedToken"` code and the hedged message wording ("may still be defined elsewhere Drut can't see") don't overclaim certainty the check doesn't have — deliberately, given Principle IV's stakes here. |
| VIII. Public/Private Boundary | **PASS** | All touched crates are already public; no vendor-doc-derived material is introduced. |

No unjustified violations. No Complexity Tracking entries.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/quickstart.md):
`contracts/undefined-token-diagnostic.md`'s exact guarantee list confirms the Principle I/IV
framing above holds precisely — no row's status changed. The visibility widening
(`collect_included_files`/`IncludedFile` → `pub(crate)`) is the only change to an existing file
beyond `diagnostics.rs`'s new chained stream, and it changes no behavior, only reach.

## Project Structure

### Documentation (this feature)

```text
specs/020-undefined-token-diagnostic/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output
├── data-model.md                  # Phase 1 output
├── quickstart.md                  # Phase 1 output
├── contracts/
│   └── undefined-token-diagnostic.md   # exact function/stream shapes, guarantee list
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
crates/voyager-core/
└── src/token_resolution.rs          # + all_variable_refs(nodes: &[Node]) ->
                                     #   Vec<VariableRefAt> (research.md §2).
                                     #   Everything else in this file unchanged.

crates/drut-lsp/
├── src/hover.rs                     # collect_included_files/IncludedFile
│                                    #   widened from private to pub(crate)
│                                    #   (research.md §4) — no behavior change
├── src/undefined_token.rs (new)     # undefined_token_positions(uri, doc) ->
│                                    #   Vec<VariableRefAt> (data-model.md §2)
└── src/diagnostics.rs                # + fourth chained stream,
                                     #   undefined_token_diagnostics (HINT,
                                     #   "drut-token" source, "UndefinedToken"
                                     #   code) — data-model.md §3

crates/drut-cli/, crates/drut-mcp/, crates/drut-config/   # no changes (FR-005/FR-008)

ROADMAP.md                           # item 14 marked done on completion
```

**Structure Decision**: No new crate. One new pure function in an existing `voyager-core`
module, one new small `drut-lsp` module, one visibility widening, one new chained stream in an
already-multi-stream function. Every piece of real resolution logic this feature depends on
already exists and is reused unmodified — the only genuinely new code is enumeration
(`all_variable_refs`) and wiring (`undefined_token.rs`, the new stream in `diagnostics.rs`).

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new dependencies, no new crates.
This feature is smaller in scope than any prior feature this session, by design: reusing
existing, already-validated resolution logic verbatim rather than building new semantic analysis
was the explicit point of scoping it down to `@token@` references only (spec.md Assumptions).*
