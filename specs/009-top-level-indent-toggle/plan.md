# Implementation Plan: Top-Level Indent Default Revert

**Branch**: `009-top-level-indent-toggle` | **Date**: 2026-08-11 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/009-top-level-indent-toggle/spec.md`

## Summary

Add a `TopLevelIndentMode` enum (`Preserve` / `Normalize`) to
`voyager_core::FormatOptions`, defaulting to `Preserve` via `#[default]`.
`plan_indentation`'s single 008-added line
(`plan.insert(node.span().start.line, 0)`) becomes conditional on
`mode == Normalize` — everything downstream (`plan_block`/`plan_children`,
closer alignment, per-nesting-level increment, `007`'s diagnosed-block-
children skip) is untouched, because none of it hardcodes top-level
behavior; it all reads through `computed_indent`'s existing "planned value,
else original on-disk value" fallback, which already does the right thing
for both modes with zero changes. Wire a `--top-level-indent
<preserve|normalize>` CLI flag (mirroring `OutputFormat`'s required-with-
default `value_enum` shape, not `--casing`'s optional/off shape, since this
setting always has a value). Explicitly set the new field at every
`FormatOptions` struct-literal call site (`drut-cli`, `drut-mcp` — the
compiler already forces this, a genuine safety net) and add a dedicated
behavioral test at every `FormatOptions::default()` call site in
`drut-lsp` (`formatting.rs`, `range_formatting.rs` — the compiler does
*not* force anything there, so this is the one place a stale default could
still hide). Regenerate `format_corpus.rs`'s golden fixtures back to
`preserve`-mode output; retarget every existing test that asserted
008-era normalize-by-default behavior to explicit `Normalize` mode instead
of deleting it.

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition — unchanged.

**Primary Dependencies**: None new — `clap`'s `ValueEnum` derive
(already a dependency, already used for `CasingArg`/`OutputFormat`) covers
the new CLI enum too.

**Storage**: N/A.

**Testing**:
- `crates/voyager-core/src/format.rs`'s own `#[cfg(test)] mod tests` —
  every existing test that currently relies on `FormatOptions::default()`
  producing 008-era column-0-forced top-level output (a real, non-trivial
  subset — audited individually in research.md §2, not assumed) is
  retargeted to construct `FormatOptions { top_level_indent:
  TopLevelIndentMode::Normalize, .. }` explicitly, so it keeps testing
  what it was written to test. New tests added for the `Preserve` default
  itself (mirroring the shape of `007`-era pre-`008` coverage).
- `crates/voyager-core/tests/format_sequence.rs` — all 5 existing tests
  use `FormatOptions::default()` and assert 008-era normalize behavior
  (the `PROCESS`/`RUN` residue scenarios `008` was built to prove);
  retargeted to explicit `Normalize` (FR-006) so `008`'s own guarantee
  keeps being verified, not silently starting to test (and fail against)
  `preserve`-mode output instead.
- `crates/voyager-core/tests/format_corpus.rs` — golden-fixture
  regeneration (FR-005) back to `preserve`-mode output, same
  `UPDATE_GOLDEN=1` / human-reviewed-diff workflow as every prior
  regeneration. A parallel explicit-`Normalize` run over the same corpus
  MUST still reproduce `008`'s already-committed golden output
  byte-for-byte (FR-006/SC-002) — proven by keeping (not deleting) a
  copy of `008`'s goldens as a second, `Normalize`-mode fixture set, per
  research.md §3.
- `crates/drut-cli/tests/format_flags.rs` — new coverage for
  `--top-level-indent=preserve|normalize`, mirroring the existing
  `--casing` flag test shape.
- `crates/drut-mcp/tests/format_contract.rs` (or equivalent) — confirms
  the MCP `format` tool's output matches `preserve`-mode with no options
  passed (FR-004(c) for this specific call site).
- New `crates/drut-lsp/src/formatting.rs` / `range_formatting.rs` test
  cases (FR-004(b)/User Story 3) — explicit, dedicated assertions that a
  document with non-zero top-level indentation is left untouched by both
  `textDocument/formatting` and `textDocument/rangeFormatting` with no
  client-side override. These are the *only* call sites in the whole
  change with no compiler-enforced forcing function, so they get their
  own named tests rather than relying on an existing test happening to
  cover it incidentally.
- Full 161-file corpus revalidation across `drut-cli`/`drut-lsp`/
  `drut-mcp` under both modes, reported as its own explicit result (this
  session's established standard).

**Target Platform**: Cross-platform, unchanged.

**Project Type**: `voyager-core` core change (new enum + field, one
conditional line) plus small, explicit adapter-layer wiring in all three
adapters (`drut-cli` gains the flag; `drut-lsp`/`drut-mcp` gain no
user-facing surface but need the default explicitly verified) plus a
second FR-012 spec amendment and a golden-fixture regeneration pass.

**Performance Goals**: No measurable change — same single `BTreeMap`
insert per top-level node, now behind one extra branch.

**Constraints**:
- MUST default to `preserve` at every one of: the CLI flag's own default,
  `FormatOptions::default()`, and every `drut-lsp`/`drut-mcp` call site
  (FR-004) — verified individually per research.md §2's call-site
  inventory, not assumed transitively.
- MUST NOT change `plan_block`/`plan_children`/closer-alignment/
  per-nesting-level logic (FR-003) — confirmed in research.md §1 that
  none of it needs to, the same way `008`'s own plan.md confirmed for its
  own change.
- MUST NOT alter `Normalize` mode's actual output versus `008`'s
  already-shipped behavior, byte-for-byte (FR-003/FR-006).
- MUST regenerate and individually human-review every affected golden
  fixture before merge (FR-005, constitution Principle III's existing
  gate).
- MUST retarget (not delete) every existing test whose assertions depend
  on 008-era default behavior (FR-006).

**Scale/Scope**: Same 161-file corpus. `format_corpus.rs`'s golden set
reverts to `preserve` output; a second, `Normalize`-mode-explicit fixture
set (reusing `008`'s already-committed golden files as fixed expected
output, not regenerated) proves `008`'s behavior is unchanged when opted
into.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | The mode lives entirely in `voyager-core::FormatOptions`/`plan_indentation`. `drut-cli` gains a flag that is a thin pass-through (mirrors `--casing`'s own established pattern exactly); `drut-lsp`/`drut-mcp` gain no new logic, only an explicit default-field value (`drut-mcp`) or a verifying test (`drut-lsp`) — no grammar/parsing/formatting logic duplicated anywhere outside the core crate. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | No new text derived from vendor documentation. |
| III. Formatter Idempotence & Behavior Preservation | **PASS, re-verified not assumed** | Idempotence holds independently under each mode (each is its own stable fixed point, same argument `008`'s own plan.md made for its single mode). Behavior preservation (no reordering, no continuation changes, no meaning change) is unaffected — still whitespace-only, under either mode. Golden-file diff review (FR-005) applied explicitly, not skipped. |
| IV. False Negatives Over False Positives | **N/A** | Governs diagnostics; no diagnostic category is added, changed, or suppressed. |
| V. Vertical, Independently-Usable Increments | **PASS** | Single, atomic, independently valuable and testable change. Does not start until `008` (and the unrelated `range_formatting` stale-test fix, and the short-IF semantic-token fix) are already merged and green on `main`. |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **N/A** | No new editor-integration surface — `drut-lsp` gains no new capability, only a verifying test on existing capabilities. |
| VII. Naming Honesty | **PASS** | `--top-level-indent=preserve\|normalize` names exactly what it does; no overclaiming. |
| VIII. Public/Private Boundary | **PASS** | `voyager-core`/`drut-cli`/`drut-lsp`/`drut-mcp` are already public; no vendor-documentation-corpus content involved. |

No unjustified violations. No Complexity Tracking entries.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/
quickstart.md): `contracts/top-level-indent-toggle.md`'s exact algorithm
and call-site inventory confirm the Principle I/III framing above holds
precisely — no row's status changed.

## Project Structure

### Documentation (this feature)

```text
specs/009-top-level-indent-toggle/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output (/speckit-plan command)
├── data-model.md                  # Phase 1 output (/speckit-plan command)
├── quickstart.md                  # Phase 1 output (/speckit-plan command)
├── contracts/                     # Phase 1 output (/speckit-plan command)
│   └── top-level-indent-toggle.md   # exact plan_indentation algorithm,
│                                    # FormatOptions/CLI shape, and the
│                                    # full FormatOptions call-site inventory
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/voyager-core/
├── src/format.rs                    # + TopLevelIndentMode enum (Default =
│                                    #   Preserve); FormatOptions gains
│                                    #   top_level_indent field;
│                                    #   plan_indentation's column-0 insert
│                                    #   becomes conditional on the mode;
│                                    #   plan_block/plan_children/render
│                                    #   otherwise unchanged; own test
│                                    #   module: existing 008-era-assuming
│                                    #   tests retargeted to explicit
│                                    #   Normalize, new Preserve-default
│                                    #   tests added
├── src/lib.rs                       # re-export TopLevelIndentMode
├── tests/format_sequence.rs         # all 5 existing tests retargeted to
│                                    # explicit Normalize (they test 008's
│                                    # guarantee, which must keep holding
│                                    # opt-in); no new Preserve-mode
│                                    # residue test needed (007's skip
│                                    # behavior under Preserve is identical
│                                    # to pre-008, already covered by the
│                                    # still_broken_process_leaves_
│                                    # swallowed_content_untouched test's
│                                    # own pre-008 shape)
└── tests/
    ├── fixtures/golden/               # regenerated back to preserve-mode
    │   ├── *.s                          # output (hand-written set)
    │   └── real_corpus/**/*.s           # (all 9 real files)
    └── fixtures/golden_normalize/       # NEW: 008's already-committed
                                        # golden output, kept verbatim as
                                        # a second fixture set proving
                                        # explicit Normalize mode is
                                        # byte-identical to 008 (FR-006)

crates/drut-cli/
├── src/cli.rs                       # + TopLevelIndentArg (ValueEnum,
│                                    #   default_value_t = Preserve,
│                                    #   mirrors OutputFormat's shape, not
│                                    #   CasingArg's Option shape); new
│                                    #   flag on Command::Format
├── src/format_cmd.rs                # run() gains a top_level_indent
│                                    #   parameter; FormatOptions struct
│                                    #   literal explicitly sets it
│                                    #   (compiler-forced already)
└── tests/format_flags.rs            # new --top-level-indent coverage

crates/drut-mcp/
├── src/format.rs                    # FormatOptions struct literal
│                                    #   explicitly sets top_level_indent:
│                                    #   TopLevelIndentMode::default()
│                                    #   (compiler-forced already; no new
│                                    #   FormatInput field — no MCP-side
│                                    #   toggle in scope, per Assumptions)
└── tests/format_contract.rs         # new: confirms preserve-mode default

crates/drut-lsp/
├── src/formatting.rs                # unchanged code (still
│                                    #   FormatOptions::default()); NEW
│                                    #   dedicated test proving the
│                                    #   resolved default is preserve
└── src/range_formatting.rs          # same: unchanged code, NEW dedicated
                                     #   test

specs/002-cli-check-format/
├── spec.md                          # FR-012 amended a second time (new
│                                    #   dated entry alongside, not
│                                    #   replacing, 008's own entry);
│                                    #   FR-015-style new FR for the
│                                    #   --top-level-indent flag
└── contracts/formatting-api.md       # top-level baseline description
                                     #   amended to describe both modes
```

**Structure Decision**: No new crate, no new module. `voyager-core` gains
one enum, one struct field, and one conditional branch around an
already-existing single line. All adapter-layer changes are equally
small: one new CLI flag definition + one parameter thread-through in
`drut-cli`, one explicit field value in `drut-mcp`'s existing struct
literal, and zero code changes (only new tests) in `drut-lsp`. The
existing "prefer a planned value over the original" fallback in
`computed_indent` is exactly what makes `Preserve` mode work for free
once the seed is skipped — no new fallback logic needed, matching `008`'s
own plan.md observation about this same function in reverse.

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new
dependencies, no new architectural components.*
