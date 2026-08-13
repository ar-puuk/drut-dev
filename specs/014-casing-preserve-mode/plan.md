# Implementation Plan: Casing Gains an Explicit `Preserve` Mode

**Branch**: `014-casing-preserve-mode` | **Date**: 2026-08-13 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/014-casing-preserve-mode/spec.md`

## Summary

Add `Preserve` as `voyager_core::CasingConvention`'s third variant and
`#[default]`, mirroring `TopLevelIndentMode`'s already-shipped shape exactly.
`FormatOptions.casing` changes from `Option<CasingConvention>` to a bare
`CasingConvention`. The formatter's one and only read of this field
(`render`'s `if let Some(convention) = options.casing { collect_casing_edits
(...) }`) becomes `if options.casing != CasingConvention::Preserve {
collect_casing_edits(nodes, ..., options.casing, ...) }` — every downstream
casing function already takes a bare `CasingConvention`, not an `Option`, so
nothing past that one check changes. `drut-config`'s `FormatConfig`/
`ExplicitFormatOverride` keep `casing: Option<CasingConvention>` unchanged
(a different Option, meaning "this layer said nothing"); `resolve_format_
options`'s two `casing` lines each gain a trailing `.unwrap_or_default()`,
matching `top_level_indent`'s existing lines exactly. `drut-config`'s TOML
parser, `drut-cli`'s `CasingArg`, and `drut-mcp`'s `casing` string parameter
each gain a third accepted value (`"preserve"`), matching their
`top_level_indent`/`TopLevelIndentArg` siblings' existing three-value shape.
`drut-lsp` is untouched behaviorally — its two call sites never construct an
explicit override, so they're touched only by the type change compiling
through. Every existing golden fixture and `format_corpus.rs` test is
expected to pass unmodified — FR-003/SC-001 establish there is no output
change to review, unlike `008`/`009`'s golden-fixture regeneration.

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition — unchanged.

**Primary Dependencies**: None new.

**Storage**: N/A.

**Testing**:
- `crates/voyager-core/src/format.rs`'s own `#[cfg(test)] mod tests` — the
  two test builder helpers `upper()`/`normalize()` (lines 695–707) construct
  `FormatOptions` struct literals with `casing: Some(...)`/`casing: None`;
  these become `casing: CasingConvention::Upper`/`CasingConvention::Preserve`
  — a compile error today would force this anyway (the field is no longer
  `Option`-wrapped), a genuine safety net, not something that could be
  silently missed. New unit tests: `Preserve` is the enum's own `#[default]`;
  `CasingConvention::Preserve` produces byte-identical output to every
  existing `casing_off_by_default_leaves_everything_alone`-style assertion
  (already covered by that test continuing to pass with no changes, since
  `FormatOptions::default()`'s casing field resolves to `Preserve` either
  way — confirmed as a *result*, not assumed).
- `crates/voyager-core/tests/format_corpus.rs` / `format_sequence.rs` — no
  fixture regeneration; every existing assertion is expected to keep passing
  unmodified (FR-003/SC-001), confirmed by running the suite, not skipped.
- `crates/drut-config/tests/parse.rs` — new case: `casing = "preserve"`
  parses to `Some(CasingConvention::Preserve)`, not a warning (FR-005/SC-005),
  mirroring the existing `top_level_indent = "preserve"` coverage.
- `crates/drut-config/tests/resolve.rs` — new case: with no `drut.toml` and
  no explicit override, `resolve_format_options` yields
  `CasingConvention::Preserve` (the `.unwrap_or_default()` addition, FR-004).
- `crates/drut-cli/tests/format_flags.rs` — new case:
  `--casing=preserve` overrides a `drut.toml`-resolved `upper`/`lower` for
  one run (FR-006/User Story 1), mirroring the existing
  `explicit_casing_flag_overrides_drut_toml_for_one_run_only` test's shape.
  Existing `casing_with_invalid_value_is_a_usage_error` (`--casing=sideways`)
  and `casing_with_no_value_is_a_usage_error_before_touching_any_file`
  (bare `--casing`) are expected to keep passing unmodified — `preserve` is
  a new *valid* value, not a change to what counts as invalid or bare.
- `crates/drut-mcp/src/format.rs`'s own test module — new case:
  `casing: Some("preserve")` overrides a `drut.toml`-resolved value
  (FR-007), mirroring `explicit_casing_param_overrides_a_present_drut_toml`.
- `crates/drut-lsp` — no new tests required (FR-008 is satisfied by the
  compiler forcing every call site to still type-check; `drut-lsp` never
  constructs an explicit casing override today). Existing `cargo test -p
  drut-lsp` passing unmodified is itself the confirmation.
- Full 161-file corpus revalidation across CLI/LSP/MCP, reported as its own
  explicit result (this project's established standard) — expected zero
  diagnostic or output change, since this is a pure representation change.

**Target Platform**: Cross-platform, unchanged.

**Project Type**: `voyager-core` core change (new enum variant + one
non-optional field + one conditional's comparison operator) plus small,
symmetric adapter-layer wiring in `drut-config`/`drut-cli`/`drut-mcp`
(`drut-lsp` untouched behaviorally) plus a `002-cli-check-format/spec.md`
amendment.

**Performance Goals**: No measurable change — same single boolean check,
now against an enum equality instead of `Option::is_some()`.

**Constraints**:
- MUST NOT change formatter output for any existing input (FR-003, User
  Story 2) — confirmed by the full corpus and every existing golden
  fixture/test passing byte-for-byte unmodified, not by inspection alone.
- MUST keep `drut_config::FormatConfig`/`ExplicitFormatOverride`'s `casing`
  field `Option`-wrapped (FR-004) — only the resolved `voyager_core::
  FormatOptions.casing` field loses its `Option`.
- MUST accept `"preserve"` as a valid `casing` value at every surface that
  already accepts `"upper"`/`"lower"` as a string (TOML, CLI, MCP) —
  FR-005/FR-006/FR-007.
- MUST NOT change `--casing`'s existing "no bare flag" rule
  (`002-cli-check-format` FR-015) — `--casing` alone (no `=value`) remains a
  usage error; `preserve` is a new explicit value, not an implicit default
  for a bare flag.
- MUST correct every doc comment/spec sentence that describes the old
  two-variant-plus-`None` shape as still current (FR-010), including
  `002-cli-check-format/spec.md`'s **FR-026**, which currently contrasts
  itself against `--casing` with the sentence "Unlike FR-015's `--casing`
  flag, this setting has no 'off' state" — that contrast becomes inaccurate
  once `--casing` also resolves to a non-optional default (`Preserve`) with
  no distinct "off" state at the resolved-value layer, the same shape
  `--top-level-indent` already has. Found during Phase 0 research, not
  named in the original feature description — see research.md §3.

**Scale/Scope**: Same 161-file corpus, revalidated for zero change (not
zero-diagnostics-with-changes, as `008`/`009` proved — genuinely
byte-identical `format` output throughout).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | The enum, field, and the one conditional it gates all live entirely in `voyager-core::format`. `drut-config`/`drut-cli`/`drut-mcp` gain only thin string/enum-value mapping, mirroring `top_level_indent`'s already-established pattern exactly — no grammar/parsing/formatting logic duplicated anywhere outside the core crate. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | No new text derived from vendor documentation — this is a pure API-shape change, not new casing-target research (that remains ROADMAP's still-deferred `--casing=auto` item). |
| III. Formatter Idempotence & Behavior Preservation | **PASS, re-verified not assumed** | Idempotence is unaffected — this changes *how* "don't touch casing" is represented, not any actual formatting behavior; FR-003/SC-001 require this held, confirmed by the full existing test suite and corpus passing with zero modification, not by inspection. No golden-file diff review needed (unlike `008`/`009`) — nothing to diff. |
| IV. False Negatives Over False Positives | **N/A** | Governs diagnostics; no diagnostic category is added, changed, or suppressed. |
| V. Vertical, Independently-Usable Increments | **PASS** | Single, atomic, independently valuable and testable change. Does not depend on any other pending pre-publish item (ROADMAP.md item 4's own Assumptions) and blocks none of them. |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **N/A** | No new editor-integration surface — `drut-lsp` gains no new capability, only a type change compiling through unchanged call sites. |
| VII. Naming Honesty | **PASS** | `Preserve`/`--casing=preserve` name exactly what they do, mirroring the already-shipped `TopLevelIndentMode::Preserve`/`--top-level-indent=preserve` naming precedent directly. |
| VIII. Public/Private Boundary | **PASS** | All touched crates are already public; no vendor-documentation-corpus content involved. |

No unjustified violations. No Complexity Tracking entries.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/
quickstart.md): `contracts/casing-preserve-mode.md`'s exact call-site
inventory confirms the Principle I/III framing above holds precisely — no
row's status changed. The FR-026 staleness found in Phase 0 (research.md §3)
is folded into FR-011's amendment scope; no new Constitution concern from it.

## Project Structure

### Documentation (this feature)

```text
specs/014-casing-preserve-mode/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output (/speckit-plan command)
├── data-model.md                  # Phase 1 output (/speckit-plan command)
├── quickstart.md                  # Phase 1 output (/speckit-plan command)
├── contracts/                     # Phase 1 output (/speckit-plan command)
│   └── casing-preserve-mode.md      # exact CasingConvention/FormatOptions
│                                    # shape and the full call-site inventory
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/voyager-core/
├── src/format.rs                    # CasingConvention gains Preserve
│                                    #   (#[default]); FormatOptions.casing
│                                    #   becomes bare CasingConvention;
│                                    #   render()'s `if let Some(convention)
│                                    #   = options.casing` becomes `if
│                                    #   options.casing !=
│                                    #   CasingConvention::Preserve`;
│                                    #   edit_for_span's `match convention {
│                                    #   Upper => .., Lower => .. }` gains a
│                                    #   third, practically-unreachable arm
│                                    #   (render's guard means this
│                                    #   function is never actually called
│                                    #   with Preserve) required purely for
│                                    #   match exhaustiveness — research.md
│                                    #   §1 names this as a second real call
│                                    #   site, not just render()'s guard;
│                                    #   doc comments on both corrected (no
│                                    #   longer describe None as "off");
│                                    #   own test module: upper()/normalize()
│                                    #   helpers updated (compiler-forced),
│                                    #   new Preserve-default tests added
└── src/lib.rs                       # re-export unchanged (CasingConvention
                                     #   already re-exported)

crates/drut-config/
├── src/lib.rs                       # resolve_format_options's `let casing
│                                    #   = explicit.casing.or(config.format.
│                                    #   casing);` gains `.unwrap_or_
│                                    #   default()`, matching top_level_
│                                    #   indent's existing line;
│                                    #   default_options's casing line gets
│                                    #   the same
├── src/parse.rs                     # parse_casing gains a `Some("preserve")
│                                    #   => Some(CasingConvention::Preserve)`
│                                    #   arm; error messages updated to
│                                    #   name all three values
├── tests/parse.rs                   # new: "preserve" parses cleanly
└── tests/resolve.rs                 # new: unset casing resolves to
                                     #   Preserve

crates/drut-cli/
├── src/cli.rs                       # CasingArg gains a third Preserve
│                                    #   variant (ValueEnum) — Option<
│                                    #   CasingArg> shape unchanged
├── src/format_cmd.rs                # impl From<CasingArg> for
│                                    #   CasingConvention gains a
│                                    #   CasingArg::Preserve arm
└── tests/format_flags.rs            # new: --casing=preserve overrides a
                                     #   drut.toml-resolved value

crates/drut-mcp/
├── src/format.rs                    # explicit_override's casing match
│                                    #   gains a `Some("preserve") =>
│                                    #   Some(CasingConvention::Preserve)`
│                                    #   arm; doc comment on FormatInput.
│                                    #   casing updated; error message
│                                    #   updated to name all three values
└── (own test module in format.rs)   # new: casing: Some("preserve")
                                     #   overrides a drut.toml-resolved value

crates/drut-lsp/                     # no source changes — untouched
                                     #   call sites compile through the
                                     #   type change unchanged; existing
                                     #   test suite passing unmodified is
                                     #   the confirmation (FR-008)

specs/002-cli-check-format/
└── spec.md                          # FR-015 amended with a new dated entry
                                     #   (the explicit preserve value + the
                                     #   underlying representation change,
                                     #   FR-011); FR-026's "Unlike FR-015...
                                     #   no 'off' state" contrast corrected
                                     #   to reflect the new shared shape
                                     #   (research.md §3)
```

**Structure Decision**: No new crate, no new module. `voyager-core` gains
one enum variant, one field's type (`Option<CasingConvention>` →
`CasingConvention`), and one comparison operator. Every adapter-layer change
is a small, symmetric addition mirroring a pattern `top_level_indent`
already established in the exact same file/function — no new pattern is
being invented anywhere in this feature.

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new
dependencies, no new architectural components.*
