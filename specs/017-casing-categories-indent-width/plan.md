# Implementation Plan: Per-Category Casing Configuration and Configurable Indentation Width

**Branch**: `017-casing-categories-indent-width` | **Date**: 2026-08-17 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/017-casing-categories-indent-width/spec.md`

## Summary

Two additive changes to `voyager-core`'s formatter, bundled because both extend the same
`FormatOptions`/`drut-config`/adapter plumbing:

1. `FormatOptions.casing: CasingConvention` (one setting) becomes
   `FormatOptions.casing: CasingSettings` (three independent settings — `control_words`,
   `pair_keywords`, `data_references` — each still a plain `CasingConvention`). A new
   `voyager-core` module recognizes `data_references`-category tokens (Matrix/Line/Node/Zone/
   Database families, `RO`, `A`/`B`, `I`/`J`) across all three shapes they appear in
   (dot-notation read, pair-keyword name, assignment target) **without any lexer/`TokenKind`
   change** — see research.md §1 for why the originally-feared lexer-level rework
   (`ROADMAP.md` resolved-queued item 4, Path (b)) turned out to be unnecessary.
2. `FormatOptions` gains `indent_width: u8` (default 4), mirroring `TopLevelIndentMode`'s
   existing configurable-axis shape.

Every existing config surface (`drut.toml`'s flat `casing` field, `--casing`, the MCP `casing`
param) keeps working exactly as before — a purely additive change, never a breaking one (spec
FR-012). `keywords.rs`'s completion/spell-check dictionary is separately corrected: `NUMREC`/
`CNT`/`ITER`/`LP`/`RECNUM` removed (confirmed non-keywords), `ZONES` added.

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition — unchanged.

**Primary Dependencies**: None new — `voyager-core` remains zero-runtime-dependency (FR-027
in `001-voyager-script-parser`).

**Storage**: N/A.

**Testing**:
- `crates/voyager-core/src/format.rs` — new unit tests for `CasingSettings` (each field
  defaults to `Preserve`; each field's casing applies independently; `indent_width` defaults
  to 4 and changes nesting-level spacing consistently).
- `crates/voyager-core/src/data_reference.rs` (new module) — unit tests per family
  (Matrix/Line/Node/Zone/Database/Record/Endpoint/loop-index), covering all three structural
  shapes for the tokens that have more than one (`MW`/`ZONES`), plus the negative case (an
  ordinary user variable name that isn't a recognized data-reference token is never touched).
- `crates/voyager-core/src/keywords.rs` — existing tests updated for `NUMREC`/`CNT`/`ITER`/
  `LP`/`RECNUM` removal and `ZONES` addition; `completion_candidates`/`did_you_mean` coverage
  extended.
- `crates/voyager-core/tests/format_corpus.rs`/`format_sequence.rs` — no golden-fixture
  regeneration expected for the *existing* golden set (FR-012: byte-identical when nothing new
  is configured); new golden fixtures added specifically exercising `data_references` casing
  and non-default `indent_width`, verified by hand before being trusted as golden.
- `crates/drut-config/tests/parse.rs`/`resolve.rs` — new cases for the three new `drut.toml`
  fields per category, the new `indent_width` field, the legacy-field-still-works case, and the
  full precedence matrix (see data-model.md §3).
- `crates/drut-cli/tests/format_flags.rs` — new cases for the three new casing flags and
  `--indent-width`, plus a legacy-`--casing`-still-works regression case.
- `crates/drut-mcp/src/format.rs` test module — same shape, MCP-side.
- Full real-corpus revalidation (CLI/LSP/MCP) — expected zero diagnostic/output change with no
  new configuration supplied (SC-003), reported as its own explicit result per this project's
  established standard.

**Target Platform**: Cross-platform, unchanged.

**Project Type**: `voyager-core` core change (new module, one struct-shape change, one new
field) plus symmetric, additive adapter-layer wiring in `drut-config`/`drut-cli`/`drut-mcp`
(`drut-lsp` untouched behaviorally, same as `014`) plus `keywords.rs` dictionary corrections.

**Performance Goals**: No measurable regression — `data_references` recognition is a single
additional per-token text check (a small, fixed prefix table lookup) alongside the existing
casing-edit collection pass; no new full-file scan is introduced.

**Constraints**:
- MUST NOT change formatter output for any existing input when no new setting is configured
  (FR-012) — confirmed by the full existing golden-fixture set and corpus passing byte-for-byte
  unmodified.
- MUST NOT remove or change the meaning of any already-shipped config surface (`drut.toml`'s
  flat `casing` field, `--casing`, the MCP `casing` param, `top_level_indent`,
  `--top-level-indent`) — every new field is additive (research.md §2).
- MUST NOT introduce any built-in opinionated preset/"auto" value (FR-003) — the three new
  categories accept only `upper`/`lower`/`preserve`, same as today's single `casing` value.
- MUST apply a `data_references` token's casing uniformly across all structural shapes it can
  appear in (FR-005) — one lookup keyed by base name, not per-shape logic that could disagree.
- MUST NOT change which `voyager-core` grammar rules exist or how any node is structured
  (Principle I/III) — `data_reference.rs` is a read-only recognition pass over already-parsed
  `Node`/`Token` data, the same architectural shape `token_resolution.rs`/`block_resolution.rs`
  already use, not a lexer or parser change.
- Invalid `indent_width` values MUST degrade to the built-in default with a non-blocking notice
  (FR-010), never a hard failure — the first *numeric* (not closed-enum) configurable value in
  this project, so this is a genuinely new validation shape, not a copy of an existing one
  (research.md §4).

**Scale/Scope**: Same 161-file real corpus, revalidated for zero change with no new
configuration supplied, plus new hand-verified golden fixtures for the newly-reachable
`data_references` casing and non-default `indent_width` cases specifically.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | `data_reference.rs`'s recognition logic and the `indent_width` math both live entirely in `voyager-core`; every adapter gains only thin field/flag/param mapping, mirroring `top_level_indent`'s already-established pattern. No grammar/parsing/formatting logic duplicated outside the core crate. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | The `data_references` family list and every rationale in this plan/research are written in this project's own words, derived from (not copied from) the `_archive/` vendor docs already researched earlier in this feature's design — no new verbatim text introduced. |
| III. Formatter Idempotence & Behavior Preservation | **PASS, re-verified not assumed** | `data_references` casing and `indent_width` are both new, opt-in axes — FR-012/SC-003 require zero output change with nothing configured, confirmed by the full existing corpus/golden set passing unmodified, not by inspection. New golden fixtures exercising the new axes get the same idempotence check every existing fixture already gets. `; FMT: OFF`/`ON` and every other existing protection (FR-011) is re-verified against the two new axes specifically, not assumed to hold transitively. |
| IV. False Negatives Over False Positives | **N/A** | Governs diagnostics; no diagnostic category is added, changed, or suppressed by this feature. |
| V. Vertical, Independently-Usable Increments | **PASS** | The three user stories are independently valuable and independently testable (spec.md); `indent_width` (US3) ships fully functional even if `data_references` casing (US2) were somehow reverted, and vice versa. |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **N/A** | No new editor-integration surface — `drut-lsp` gains only the type/field changes rippling through its existing format-on-save/format-on-paste call sites. |
| VII. Naming Honesty | **PASS** | `data_references`/`CasingSettings`/`indent_width` name exactly what they do; no overclaiming. |
| VIII. Public/Private Boundary | **PASS** | All touched crates are already public. The `_archive/` vendor-doc research that grounded the `data_references` family list stays local-only, as it already has throughout this feature's design — nothing from it is imported verbatim. |

No unjustified violations. No Complexity Tracking entries.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/quickstart.md):
`contracts/casing-categories-indent-width.md`'s exact type/precedence inventory confirms the
Principle I/III framing above holds precisely — no row's status changed.

## Project Structure

### Documentation (this feature)

```text
specs/017-casing-categories-indent-width/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output
├── data-model.md                  # Phase 1 output
├── quickstart.md                  # Phase 1 output
├── contracts/
│   └── casing-categories-indent-width.md   # exact type shapes, precedence matrix,
│                                             # data_reference family table
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
crates/voyager-core/
├── src/format.rs                    # CasingConvention: unchanged (still the
│                                    #   per-category value type). New
│                                    #   CasingSettings { control_words,
│                                    #   pair_keywords, data_references },
│                                    #   #[derive(Default)] (each field's own
│                                    #   CasingConvention::default() =
│                                    #   Preserve). FormatOptions.casing
│                                    #   becomes CasingSettings; gains
│                                    #   indent_width: u8. FormatOptions
│                                    #   drops #[derive(Default)] for a
│                                    #   manual impl (indent_width's default
│                                    #   is 4, not u8::default()==0 — the
│                                    #   first field on this struct whose
│                                    #   correct default isn't its type's own
│                                    #   Default::default()). render()'s
│                                    #   casing-edit collection extended to
│                                    #   call into data_reference.rs for the
│                                    #   third category; indentation math
│                                    #   parameterized on options.indent_width
│                                    #   instead of a hardcoded 4.
├── src/data_reference.rs (new)      # Recognizes data_references-category
│                                    #   tokens across all three structural
│                                    #   shapes (dot-notation Word-token text,
│                                    #   pair-keyword name, Assignment
│                                    #   target), keyed by one shared
│                                    #   base-name table (data-model.md §2).
│                                    #   Pure read-only pass over already-
│                                    #   parsed Node/Token data — no lexer/
│                                    #   TokenKind change (research.md §1).
├── src/keywords.rs                  # NUMREC/CNT/ITER/LP/RECNUM pair_entry
│                                    #   rows removed; ZONES pair_entry
│                                    #   added (observed_with: ["RUN"], its
│                                    #   RUN PGM=MATRIX ZONES=... shape only
│                                    #   — its plain-assignment shape is
│                                    #   data_reference.rs's concern, not
│                                    #   this completion dictionary's)
└── src/lib.rs                       # re-exports CasingSettings,
                                     #   data_reference's public items

crates/drut-config/
├── src/lib.rs                       # FormatConfig/ExplicitFormatOverride
│                                    #   keep casing: Option<CasingConvention>
│                                    #   unchanged (legacy, applies to
│                                    #   control_words+pair_keywords only);
│                                    #   gain control_words_casing/
│                                    #   pair_keywords_casing/
│                                    #   data_references_casing:
│                                    #   Option<CasingConvention> (new,
│                                    #   independent); gain indent_width:
│                                    #   Option<u8>. resolve_format_options
│                                    #   implements the per-category
│                                    #   precedence matrix (data-model.md
│                                    #   §3) and the indent_width
│                                    #   1–16-bound-with-fallback validation
├── src/parse.rs                     # new TOML fields parsed with the same
│                                    #   per-field malformed-value-warns-
│                                    #   and-falls-back pattern already used
│                                    #   for every existing [format] field
├── tests/parse.rs                   # new cases per new field + legacy
│                                    #   field still parses unchanged
└── tests/resolve.rs                 # new cases for the full precedence
                                     #   matrix, including legacy-vs-new-
                                     #   field interaction

crates/drut-cli/
├── src/cli.rs                       # new --control-words-casing/
│                                    #   --pair-keywords-casing/
│                                    #   --data-references-casing flags
│                                    #   (same CasingArg ValueEnum shape);
│                                    #   new --indent-width=<N> flag
├── src/format_cmd.rs                # wires new flags into
│                                    #   ExplicitFormatOverride
└── tests/format_flags.rs            # new cases + legacy --casing
                                     #   regression case

crates/drut-mcp/
└── src/format.rs                    # new control_words_casing/
                                     #   pair_keywords_casing/
                                     #   data_references_casing/
                                     #   indent_width params, same shape as
                                     #   existing casing param; own test
                                     #   module extended to match

crates/drut-lsp/                     # no source changes — untouched call
                                     #   sites compile through the type
                                     #   change unchanged, same as `014`;
                                     #   existing test suite passing
                                     #   unmodified is the confirmation

specs/002-cli-check-format/
└── spec.md                          # FR-015/FR-026 amended: a new dated
                                     #   entry documenting the categorical
                                     #   split and indent_width, following
                                     #   the same amendment discipline `009`/
                                     #   `014` already established

specs/001-voyager-script-parser/
└── contracts/public-api.md          # amended: formatting-api.md's "casing
                                     #   is the only configurable axis"
                                     #   exclusion statement corrected —
                                     #   indentation width is now
                                     #   configurable too (ROADMAP.md item 9)

ROADMAP.md                           # items 9/10 marked done on completion;
                                     #   items 11/12 left as-is (still
                                     #   deferred, unaffected by this feature)
```

**Structure Decision**: No new crate. One new `voyager-core` module
(`data_reference.rs`), one struct-shape change (`CasingConvention` →
`CasingSettings` on `FormatOptions.casing`), one new field
(`FormatOptions.indent_width`). Every adapter-layer change is a small, additive,
symmetric extension of a pattern `top_level_indent`/`casing` already established in the
exact same files/functions — no new architectural pattern is being invented, only a
wider version of ones that already exist.

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new dependencies, no new
crates. The one genuinely new architectural piece (`data_reference.rs`) is justified
directly by FR-004/FR-005/FR-006, which cannot be satisfied any other way (the
`data_references` category is, by definition, the set of tokens today's formatter cannot
reach at all).*
