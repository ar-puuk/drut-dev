# Implementation Plan: Unused `@token@` Diagnostic

**Branch**: `029-unused-token-diagnostic` | **Date**: 2026-08-19 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/029-unused-token-diagnostic/spec.md`

## Summary

A fifth Hint-severity, non-`DiagnosticKind` LSP diagnostic stream (`drut-lsp/src/
diagnostics.rs::publish`), following the exact shape four prior features already established
(the unclosed `; FMT: OFF` marker, `010-fmt-region-markers`; the malformed `drut.toml` warning,
`012-toml-configuration`; `UndefinedToken`, `020-undefined-token-diagnostic`) — a standalone
function outside `voyager_core::Diagnostic`/`DiagnosticKind` entirely, source `"drut-token"`
(shared with `UndefinedToken` — same conceptual domain, different diagnostic code),
`"UnusedToken"` code, chained alongside the existing four.

The core computation is a set difference: `voyager-core::all_assignments(nodes)` minus every
name reachable from a new `all_variable_refs_including_openers(nodes)` function (a new pure
`voyager-core` addition, *not* a modification of the existing `all_variable_refs` — see
research.md §1 for why). `all_variable_refs_including_openers` closes the block-opener blind
spot `all_variable_refs`/`variable_ref_at` are documented to have, by additionally scanning each
`Block::opener_tokens` (the full opener-statement token stream already added this session for
the `028-identifier-highlighting`-adjacent casing fix) — infrastructure that already exists and
just wasn't being read by anything in `token_resolution.rs` yet.

Per the resolved clarifications: every dead assignment site is flagged independently (not
deduplicated to one-per-name), and the check applies unconditionally regardless of whether the
file has any `READ FILE` relationship — a documented, accepted false-positive risk for the
"used only by a file that includes this one" case that no existing resolution logic can see in
either direction.

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition — unchanged.

**Primary Dependencies**: None new — `voyager-core` remains zero-runtime-dependency; `drut-lsp`
adds no new crate dependency.

**Storage**: N/A. This feature reuses `resolve_token_value`'s `included: &[(Span, Vec<Node>)]`
shape for symmetry with `UndefinedToken`'s one-level `READ FILE` reach, but unlike `UndefinedToken`
it does not actually need per-reference resolution — it only needs the *set of names referenced*
across same-file plus one level of included files, which is a cheaper aggregate than a per-position
resolve. `hover::collect_included_files`'s existing disk-I/O is reused unmodified, same as
`020-undefined-token-diagnostic`.

**Testing**:
- `crates/voyager-core/src/token_resolution.rs` — new unit tests for
  `all_variable_refs_including_openers`: everything `all_variable_refs` already covers, plus a
  block-opener reference (`RUN PGM=@Prog@`) now included; confirms `all_variable_refs` itself is
  completely unchanged (its own existing `all_variable_refs_excludes_a_block_opener_reference`
  test keeps passing unmodified).
- `crates/drut-lsp/src/unused_token.rs` (new) — unit tests covering every spec.md Acceptance
  Scenario directly: unreferenced assignment flagged; referenced assignment (same-file, and via
  block-opener) not flagged; every dead assignment site flagged independently on reassignment
  with zero reads (Clarification Q1); a file with a `READ FILE` statement still flagged
  (Clarification Q2 — the check applies unconditionally); one-level `READ FILE`-resolved
  reference correctly suppresses the notice.
- `crates/drut-lsp/src/diagnostics.rs` — new/extended tests: the fifth stream's exact shape
  (`HINT` severity, `"drut-token"` source, `"UnusedToken"` code) on a fixture with one unused
  assignment; the six real `DiagnosticKind`-based diagnostics and `UndefinedToken` unaffected by
  this feature's addition in the same fixture.
- `crates/drut-cli`/`crates/drut-mcp` — no new tests needed (FR-005 means neither surface is
  touched); existing `check`/`diagnose` test suites passing unmodified is itself the
  confirmation nothing leaked into either surface.

**Target Platform**: Cross-platform, unchanged.

**Project Type**: `voyager-core` core addition (one new pure function, zero type changes) plus a
`drut-lsp`-only new module and one new chained stream in `diagnostics.rs` — `drut-cli`,
`drut-mcp`, `drut-config`: no source changes.

**Performance Goals**: One `collect_included_files` disk-I/O pass per `publish()` call (already
paid for `UndefinedToken`'s own stream in the same call; a second call here, same shape,
different purpose), then one linear scan each over assignments and references per document —
no new full-file re-scan beyond what `all_assignments`/`all_variable_refs_including_openers`'s
own single traversals already do.

**Constraints**:
- MUST NOT add a new `DiagnosticKind` variant (FR-004).
- MUST NOT reach `drut-cli`'s `check` command or `drut-mcp`'s `diagnose` tool (FR-005).
- MUST count a block-opener-line `@name@` reference as a genuine use (FR-003) — the one genuine
  new piece of resolution logic this feature adds, since reusing `all_variable_refs` unmodified
  (the way `020` did) would misclassify a block-opener-only-used name as unused, which is an
  unacceptable false positive under Principle IV (unlike `020`'s use of the same gap, which only
  produces an acceptable false negative).
- MUST NOT change `voyager_core::Diagnostic`/`DiagnosticKind`'s existing meaning, `all_variable_refs`'s
  existing contract/tests, or any exhaustive match over either elsewhere in the workspace
  (Principle I/VII).
- MUST require no new project configuration (FR-008).

**Scale/Scope**: Single new `voyager-core` function (~15-20 lines, thin wrapper composing
existing internals), single new `drut-lsp` module (~40-60 lines, simpler than
`undefined_token.rs` since it's a set difference, not a per-reference resolve), one new chained
stream in `diagnostics.rs`. Validated via targeted fixtures covering each Acceptance Scenario,
per quickstart.md — no real-corpus golden-file gate (this changes no formatting/parsing output,
only what gets published as LSP diagnostics).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | Both new pieces of real logic (`all_variable_refs_including_openers`, the set-difference computation) live in `voyager-core`/a thin `drut-lsp` module that composes `voyager-core` calls — no grammar/resolution logic reimplemented in the adapter layer. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | No vendor documentation consulted — this is a tooling-behavior decision, not a grammar fact. Diagnostic message wording is original. |
| III. Formatter Idempotence & Behavior Preservation | **N/A** | Touches no formatting logic. |
| IV. False Negatives Over False Positives | **PASS, directly load-bearing, honestly qualified** | This principle is *why* the notice ships at Hint (never Error) severity and why FR-003's block-opener fix is mandatory rather than deferred (a false positive there would be exactly the shape this principle forbids). The Clarification Q2 decision — applying the check unconditionally even for files touching `READ FILE` — is a **conscious, documented exception** to this principle's spirit (a real, accepted false-positive risk for the shared-parameters-file pattern), explicitly chosen by the user over the safer alternative (skip such files) and mitigated by keeping this permanently at Hint severity per this same principle's own "ship as warnings, not errors, until validated with zero known false positives" clause — this rule is not expected to ever graduate past Hint without further work. |
| V. Vertical, Independently-Usable Increments | **PASS** | Single user story, independently valuable and independently testable. |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **PASS** | Uses `textDocument/publishDiagnostics` with standard `DiagnosticSeverity::HINT`, same mechanism every prior Hint-severity stream already uses. |
| VII. Naming Honesty | **PASS** | `"UnusedToken"` and its message wording claim exactly what's checked (no reference found *within this tool's reach*) without overclaiming whole-project unused-ness, given the documented Q2 blind spot. |
| VIII. Public/Private Boundary | **PASS** | All touched crates already public; no vendor-doc-derived material introduced. |

No unjustified violations. No Complexity Tracking entries — the Principle IV qualification above
is a deliberate, user-approved scope decision (spec.md Clarifications), not an oversight.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/quickstart.md):
`contracts/unused-token-diagnostic.md`'s exact guarantee list confirms the framing above holds
precisely — no row's status changed. `all_variable_refs` (the pre-existing 020-era function) is
untouched; only a new function is added alongside it.

## Project Structure

### Documentation (this feature)

```text
specs/029-unused-token-diagnostic/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output
├── data-model.md                  # Phase 1 output
├── quickstart.md                  # Phase 1 output
├── contracts/
│   └── unused-token-diagnostic.md      # exact function/stream shapes, guarantee list
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
crates/voyager-core/
└── src/token_resolution.rs          # + all_variable_refs_including_openers(nodes: &[Node]) ->
                                     #   Vec<VariableRefAt> (research.md §1-2). all_variable_refs
                                     #   itself, and every other existing function in this file,
                                     #   unchanged.

crates/drut-lsp/
├── src/unused_token.rs (new)        # unused_token_assignments(uri, doc) ->
│                                    #   Vec<Assignment-shaped span info> (data-model.md §2)
└── src/diagnostics.rs                # + fifth chained stream, unused_token_diagnostics
                                     #   (HINT, "drut-token" source, "UnusedToken" code)
                                     #   — data-model.md §3

crates/drut-cli/, crates/drut-mcp/, crates/drut-config/   # no changes (FR-005/FR-008)

ROADMAP.md                           # new item marked done on completion
```

**Structure Decision**: No new crate. One new pure function in an existing `voyager-core`
module (additive — the existing `all_variable_refs` is not modified), one new small `drut-lsp`
module, one new chained stream in an already-multi-stream function. The genuinely new logic is
small: the opener-token scan (mirroring `collect_if_condition_token_slices`'s existing shape)
and the set-difference wiring.

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new dependencies, no new crates.*
