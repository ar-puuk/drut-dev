# Implementation Plan: Automatic Line-Width Wrapping

**Branch**: `030-auto-line-wrap` | **Date**: 2026-08-19 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/030-auto-line-wrap/spec.md`

## Summary

One new `voyager-core` formatting axis, three new `FormatOptions` fields
(`line_wrap: LineWrapMode` [`Preserve`/`Auto`, default `Preserve`], `line_wrap_width: u16`
[default `120`], `line_wrap_style: LineWrapStyle` [`Fill`/`OnePerLine`, default `Fill`]),
implemented as a new self-contained module (`src/line_wrap.rs`) mirroring `operator_spacing.rs`'s
established shape: a read-only recognition pass over `build_statements`'s flat token list,
pushing edits into the same `SpacingEdit`-shaped mechanism `018-operator-spacing` already added
to `format.rs::render`.

Two real findings from grounding this in the actual code (research.md) shape the plan:

1. `render()`'s per-line rebuild currently maps one original source line to at most one output
   line — nothing splits a line today. This feature is the first to break that invariant, done
   by having a wrap edit's `SpacingEdit` replacement string carry an embedded line-terminator
   character plus the new continuation line's indentation, reusing the existing variable-length
   edit-application mechanism `018` already built rather than adding a second, parallel
   text-rewriting pass (research.md §1).
2. Two real correctness traps follow directly from finding 1, both promoted to Functional
   Requirements rather than left as implementation notes: the embedded line-terminator MUST
   match that specific line's own already-captured CRLF/LF style (never hardcoded), and the new
   continuation line's indentation MUST be computed independently rather than through
   `indent_plan` (which is keyed by original line number and has no entry for a line that didn't
   exist in the source).

Every existing config surface stays exactly as it is — a purely additive change (FR-007), never
a breaking one.

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition — unchanged.

**Primary Dependencies**: None new — `voyager-core` remains zero-runtime-dependency.

**Storage**: N/A.

**Testing**:
- `crates/voyager-core/src/line_wrap.rs` (new) — unit tests: top-level-comma detection with
  paren/bracket-depth tracking (a comma inside a function call or bracketed subscript never
  eligible), a comma inside a quoted pair-value never eligible (research.md §4's structural
  claim, spot-checked directly rather than trusted from reasoning alone), `Fill` packing
  (multiple pairs per continuation line up to the width budget), `OnePerLine` (exactly one pair
  per line), a statement already containing a `ContinuationMarker` left completely untouched
  regardless of width (FR-005), a `Control` statement with no eligible comma left untouched, a
  non-`Control` statement never touched, an under-width statement never touched.
- `crates/voyager-core/src/format.rs` — new unit tests for the wrap-edit's terminator-matching
  (CRLF-file input produces a CRLF-terminated inserted line, not a bare `\n`) and the
  independently-computed continuation-line indentation (one level deeper than the statement's
  own opening line, correct even though `indent_plan` has no entry for the synthetic line).
- `crates/voyager-core/tests/format_corpus.rs`/`format_sequence.rs` — no golden-fixture
  regeneration expected for the *existing* golden set (FR-007: byte-identical when `line_wrap`
  isn't configured); two new golden fixture sets, `golden_line_wrap_fill/` and
  `golden_line_wrap_one_per_line/` (research.md §6), hand-verified before being trusted as
  golden, plus idempotence checks for both (SC-004) — critically including a *second-pass*
  fixture proving a once-wrapped statement is left alone on reformatting (spec.md Acceptance
  Scenario 5), not just a generic "run twice, diff empty" check.
- `crates/drut-config/tests/parse.rs`/`resolve.rs` — new cases for `line_wrap`,
  `line_wrap_width`, `line_wrap_style`; the invalid-value/out-of-range-width falls back to
  built-in-default case.
- `crates/drut-cli/tests/format_flags.rs` — new cases for `--line-wrap`, `--line-wrap-width`,
  `--line-wrap-style`.
- `crates/drut-mcp/src/format.rs` test module — same shape, MCP-side (confirm exact existing
  param-threading shape directly against source per research.md §5 before mirroring it).
- `editors/vscode` — new `drut.format.lineWrap`/`lineWrapWidth`/`lineWrapStyle` personal
  settings, mirroring every existing `drut.format.*` entry's exact shape (FR-010).
- Full real-corpus revalidation (CLI/LSP/MCP) — expected zero diagnostic/output change with no
  new configuration supplied (SC-003), reported as its own explicit result per this project's
  established standard.

**Target Platform**: Cross-platform, unchanged. CRLF/LF terminator correctness (research.md §1)
is the one platform-sensitive detail this feature must get right that most prior formatting
features didn't need to touch as directly, since this is the first feature to insert a brand
new line rather than only rewrite content on existing lines.

**Project Type**: `voyager-core` core change (one new module, three new `FormatOptions` fields,
one new render-pipeline capability — line-splitting edits) plus symmetric, additive
adapter-layer wiring in `drut-config`/`drut-cli`/`drut-mcp`/`editors/vscode` (`drut-lsp` core
untouched behaviorally beyond the type change rippling through, same as prior formatting
features).

**Performance Goals**: No measurable regression when `line_wrap == Preserve` (the default) — the
entire collection pass is skipped via the same short-circuit pattern `casing`/`operator_spacing`
already use. When `Auto` is configured, cost is one additional linear pass over each `Control`
statement's already-materialized token list (depth-tracked comma scan) — no new full-file
re-scan, no re-tokenization, no re-parsing.

**Constraints**:
- MUST NOT change formatter output for any existing input when `line_wrap` isn't configured
  (FR-007) — confirmed by the full existing golden-fixture set and corpus passing byte-for-byte
  unmodified.
- MUST NOT remove or change the meaning of any already-shipped config surface — this feature
  adds exactly three new fields/flags/params, nothing else changes shape.
- MUST NOT re-flow, re-wrap, or otherwise alter any statement that already contains a
  `ContinuationMarker` token, regardless of width (FR-005) — the mechanism that makes
  idempotence hold by construction rather than by a separately re-derived guarantee
  (research.md §1, spec.md Assumptions).
- MUST NOT insert a line break at any comma that is not at a `Control` statement's own top
  level — never inside a function call's parentheses, a bracketed subscript, or (structurally
  already impossible, per research.md §4, but still spot-checked) a quoted string.
- MUST use the specific original line's own captured line-terminator style for an inserted
  break, never a hardcoded `\n` (research.md §1) — a real, silent correctness bug this feature
  must not introduce into a CRLF-authored file.
- MUST independently compute a new continuation line's indentation rather than relying on
  `indent_plan` (research.md §1) — `indent_plan` has no entry for a line that didn't exist in
  the original source.
- Invalid `line_wrap`/out-of-range `line_wrap_width`/invalid `line_wrap_style` values MUST
  degrade to each field's own built-in default with a non-blocking notice (FR-009), the same
  established pattern every other malformed `[format]` field already uses.

**Scale/Scope**: Same real corpus, revalidated for zero change with no new configuration
supplied, plus new hand-verified golden fixtures for `Fill` and `OnePerLine` specifically
(including at least one real file with a genuinely over-width `Control` statement pulled from
real corpus shapes already seen in this project's development, not only synthetic cases).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | `line_wrap.rs`'s recognition/split-point/packing logic lives entirely in `voyager-core`; every adapter gains only thin field/flag/param mapping, mirroring `operator_spacing`'s already-established pattern. No grammar/parsing/formatting logic duplicated outside the core crate. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | This feature's rules (when/how to wrap) are a formatting-style decision reusing Cube Voyager's own existing, already-documented-in-this-project's-own-words continuation grammar (FR-006 in `001-voyager-script-parser`) — no vendor documentation is newly consulted or paraphrased for this feature. |
| III. Formatter Idempotence & Behavior Preservation | **PASS, only after a constitution amendment** | Caught by `/speckit-analyze` (finding C1, 2026-08-19): the prior version of Principle III separately and explicitly forbade "chang[ing] which lines are continuations of a prior statement," independent of its meaning-preservation clause — this feature's entire purpose is inserting new continuation breaks, a literal conflict, not a nuance. Resolved via an explicit, narrow constitution amendment (v1.1.1 → v1.2.0, 2026-08-19) extending Principle III's existing exception list (previously just "keyword casing") to also permit optionally and configurably inserting/removing a continuation break using the language's own existing, already-valid continuation syntax, without altering program meaning — the same category of presentation-only change casing already was, not a new kind of risk, and not a dilution of the principle's core prohibitions (no reordering, no meaning changes, nothing beyond what real continuation syntax already permits). With the amendment in place: FR-007/SC-003 require zero output change with nothing configured, confirmed by the full existing corpus/golden set passing unmodified, not by inspection. Idempotence (SC-004) is structural, not incidental: FR-005's "never touch an already-continued statement" rule is the actual mechanism, verified with a dedicated second-pass fixture (Testing section above), not a generic re-run-and-diff check alone. |
| IV. False Negatives Over False Positives | **PASS** | Opt-in only (`Preserve` default); a statement this feature is unsure about (any ambiguity in comma eligibility, an already-continued statement) is always left untouched rather than guessed at — the conservative direction is "don't wrap" in every uncertain case, never "wrap incorrectly." |
| V. Vertical, Independently-Usable Increments | **PASS** | Single user story, independently valuable and independently testable — deliberately narrower in scope than `018`'s two-story shape (spec.md Assumptions: arithmetic-expression/bracket/paren wrapping explicitly deferred to a future increment). |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **N/A** | No new editor-integration surface beyond the existing `drut.format.*` personal-setting mechanism (`021-editor-settings-config`'s already-established pattern) — no editor-proprietary API involved. |
| VII. Naming Honesty | **PASS** | `LineWrapMode`/`LineWrapStyle`/`line_wrap.rs` name exactly what they do; the feature is honestly scoped to `Control`-statement pair lists in its own naming/docs, never implying general expression-wrapping it doesn't do. |
| VIII. Public/Private Boundary | **PASS** | All touched crates are already public. No vendor-doc-derived material is introduced. |

No unjustified violations. No Complexity Tracking entries.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/quickstart.md):
`contracts/line-wrap.md`'s exact type/precedence inventory confirms the Principle I/III framing
above holds precisely — no row's status changed. The one genuinely new architectural piece
beyond `018`'s established pattern — a `SpacingEdit` whose replacement embeds a line-terminator
character, actually splitting a line rather than only rewriting its content — stays entirely
inside `format.rs::render`'s existing edit-application loop, not a new public surface, so it
doesn't change this table either.

## Project Structure

### Documentation (this feature)

```text
specs/030-auto-line-wrap/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output
├── data-model.md                  # Phase 1 output
├── quickstart.md                  # Phase 1 output
├── contracts/
│   └── line-wrap.md               # exact type shapes, precedence, edit-application contract
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
crates/voyager-core/
├── src/line_wrap.rs (new)           # LineWrapMode/LineWrapStyle enums; top-level-comma
│                                    #   detection over a Control statement's flat token list
│                                    #   with paren/bracket-depth tracking (research.md §4);
│                                    #   Fill packing / OnePerLine split-point selection given a
│                                    #   width budget; wrap-edit construction (terminator-aware,
│                                    #   independently-indented, research.md §1)
├── src/format.rs                    # FormatOptions gains line_wrap/line_wrap_width/
│                                    #   line_wrap_style. render() gains a wrap-edit collection
│                                    #   call (short-circuited on Preserve, mirroring casing's/
│                                    #   operator_spacing's existing short-circuit), feeding the
│                                    #   same SpacingEdit-shaped mechanism 018 already added —
│                                    #   extended so a replacement string may embed a
│                                    #   line-terminator character (data-model.md §2)
└── src/lib.rs                       # re-exports LineWrapMode/LineWrapStyle

crates/drut-config/
├── src/lib.rs                       # FormatConfig/ExplicitFormatOverride gain line_wrap:
│                                    #   Option<LineWrapMode>, line_wrap_width: Option<u16>,
│                                    #   line_wrap_style: Option<LineWrapStyle>.
│                                    #   resolve_format_options implements the same
│                                    #   explicit-flag > drut.toml > built-in-default precedence
│                                    #   every existing field already has (data-model.md §4),
│                                    #   plus range validation for the width field mirroring
│                                    #   resolve_blank_line_cap's shape
├── src/parse.rs                     # three new TOML fields, same malformed-value-warns-and-
│                                    #   falls-back pattern already used for every existing
│                                    #   [format] field
├── tests/parse.rs                   # new cases for all three fields + their accepted values
└── tests/resolve.rs                 # new cases for precedence + invalid-value fallback

crates/drut-cli/
├── src/cli.rs                       # new --line-wrap/--line-wrap-width/--line-wrap-style
│                                    #   flags (same ValueEnum/ranged-numeric shape as
│                                    #   --blank-lines/--blank-lines-top-cap)
├── src/format_cmd.rs                # wires the new flags into ExplicitFormatOverride
└── tests/format_flags.rs            # new cases

crates/drut-mcp/
└── src/format.rs                    # new line_wrap/line_wrap_width/line_wrap_style params,
                                     #   same shape as existing multi-field options; own test
                                     #   module extended

editors/vscode/
├── package.json                     # new drut.format.lineWrap/lineWrapWidth/lineWrapStyle
│                                    #   personal settings, same shape as every existing
│                                    #   drut.format.* entry
└── (client wiring, if any exists beyond package.json declaration -- confirm against how the
     existing drut.format.* fields are actually threaded, e.g. blank_lines, during
     implementation)

crates/drut-lsp/                     # no source changes beyond the type change rippling
                                     # through existing call sites unchanged, same as prior
                                     # formatting features; existing test suite passing
                                     # unmodified is the confirmation

ROADMAP.md                           # new item marked done on completion
```

**Structure Decision**: No new crate. One new `voyager-core` module (`line_wrap.rs`), three new
`FormatOptions` fields, one new render-pipeline capability (line-splitting edits, extending the
`SpacingEdit` mechanism `018-operator-spacing` already established rather than adding a second
parallel mechanism). Every adapter-layer change is a small, additive, symmetric extension of the
pattern `blank_lines`/`operator_spacing` already established in the exact same files/functions —
no new architectural pattern beyond the one genuinely new piece (a replacement string that
embeds a line-terminator and independently-computed indentation), which is itself scoped to
`format.rs::render` and `line_wrap.rs` alone.

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new dependencies, no new crates.
The one genuinely new architectural piece (a line-splitting edit) is justified directly by
FR-003, which cannot be satisfied any other way (inserting a new physical line is not
expressible through same-length or same-line variable-length edits alone — research.md §1).*
