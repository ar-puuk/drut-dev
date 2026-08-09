# Formatting API Checklist: Drut CLI — `check` and `format` Subcommands

**Purpose**: Validate the requirements quality of the new `format`/`format_bytes`
public API surface added to `voyager-core` (`contracts/formatting-api.md`, plus the
`FormatOptions`/`CasingConvention`/`FormatResult` types in `data-model.md` §1 and
the format-related requirements FR-012–FR-023 in `spec.md`), held to the same
precision bar `001-voyager-script-parser` applied to its own public-API additions
(FR-034's byte-decoding contract) and grammar-shape requirements (FR-023's
subscript-target rule) — this is new core-crate API, not CLI-adapter plumbing, so
it gets the stricter discipline.

**Created**: 2026-08-09
**Feature**: [spec.md](../spec.md) · [contracts/formatting-api.md](../contracts/formatting-api.md) · [data-model.md](../data-model.md)

**Note**: This checklist tests whether the *requirements* for `format`/`format_bytes`
are complete, unambiguous, and consistent enough to implement and test against —
not whether any implementation is correct.

**Depth/audience** (no clarification needed — request was unambiguous): formal,
pre-`/speckit-tasks` gate, read by whoever implements or reviews the
`voyager-core` formatting module; scope is `formatting-api.md` as primary surface,
cross-checked against `data-model.md` §1 and `spec.md`'s FR-012–FR-023 since those
are the requirements this contract exists to satisfy.

## Requirement Completeness

- [x] CHK001 - Is the canonical whitespace form itself specified anywhere (indentation width per block-nesting level, spacing around `=` in `keyword=value` pairs, trailing-whitespace handling, blank-line collapsing), or only asserted to exist by name? [Completeness, Gap — Spec §FR-012, Contract §formatting-api.md "re-renders it: whitespace is normalized to this feature's canonical form"] — **RESOLVED 2026-08-09**: FR-012 now enumerates seven concrete rules (indentation unit, per-level increment, closer/branch alignment, top-level baseline, continuation lines, comments), each backed by a 161-file corpus survey with cited percentages. Contract §formatting-api.md's `format` bullet now restates the rule set instead of the bare phrase.
- [x] CHK002 - Is the indentation a continuation line receives specified (aligned to its opening line, a fixed increment, or left untouched), given FR-013 only constrains *which* lines are continuations, not how a continuation line is whitespace-formatted? [Completeness, Gap — Spec §FR-013] — **RESOLVED 2026-08-09**: FR-012 explicitly leaves continuation-line indentation untouched, citing the survey's weakest signal (best single value only 23.0%, long flat tail) — a deliberate answer, not a silent gap.
- [x] CHK003 - Is whether comment bodies (`;` line comments, `/* */` block comments) are themselves subject to whitespace normalization specified, or only that comments as tokens are preserved? [Completeness, Gap — Contract §formatting-api.md] — **RESOLVED 2026-08-09**: FR-012 explicitly states comment content and the whitespace on both sides of `;` are left entirely untouched, with the trade-off (hand-aligned comment columns in the survey data) documented in Assumptions.
- [ ] CHK004 - Does a requirement state whether `format`/`format_bytes` output is always UTF-8-encoded text regardless of the input's original byte encoding (plain UTF-8, or containing Windows-1252-fallback bytes per FR-034), and whether that re-encoding is itself considered a "whitespace-only" change under FR-013? [Completeness, Gap — Contract §formatting-api.md, Spec §FR-013, FR-034 in 001-voyager-script-parser] — *partially addressed as a side effect of CHK019/CHK020's resolution (FR-013(b), `EncodingFidelity`), not itself re-run this pass.*
- [ ] CHK005 - Is the set of tokens subject to casing normalization fully enumerated (e.g. do block *closers* like `ENDIF`/`ENDLOOP`/`ENDRUN`/`ENDPROCESS`/`ENDPHASE`/`ENDJLOOP`/`ENDLINKLOOP` normalize the same as their openers), or does "control-word/keyword-name tokens" leave that implicit? [Completeness, Gap — Spec §FR-013, Contract §formatting-api.md] — not in this pass's scope; still open.
- [x] CHK006 - Is label-statement (`:STEP0`) and `@variable@`-reference text explicitly excluded from (or included in) both whitespace and casing normalization? [Completeness, Gap] — **RESOLVED 2026-08-09**: FR-015 now states casing only ever targets a token already structurally recognized as a control word/keyword name — a label's `:name`, an `@variable@` reference, and keyword values are explicitly named as never casing targets. Whitespace/indentation treatment was already covered generically (a label statement is an ordinary body statement under FR-012's per-level rule).
- [ ] CHK007 - Is `FormatOptions` documented as open to future fields (e.g. an eventual indentation-width option) without breaking existing callers, mirroring `public-api.md`'s stability note for `parse`/`parse_bytes`? [Completeness, Gap — Contract §formatting-api.md "Stability expectations for adapters"] — not in this pass's scope; still open.

## Requirement Clarity

- [x] CHK008 - Is "this feature's canonical form" (Contract §formatting-api.md) a forward reference to a definition that exists elsewhere in this feature's artifacts, or a term used without ever being defined? [Clarity, Ambiguity — Contract §formatting-api.md] — **RESOLVED 2026-08-09** (same fix as CHK001 — flagged in this file's own Notes section as the paired highest-priority finding, but omitted from the original explicit closure request; closing it now since it was fixed by the same edit): `contracts/formatting-api.md`'s `format` bullet now cites FR-012's seven concrete rules directly instead of the bare, undefined phrase.
- [ ] CHK009 - Does `FormatOptions.casing`'s documented default — "`None` (default) leaves all keyword/control-word casing untouched, **exactly as originally written**" — unambiguously mean "untouched from the current input's casing" rather than "reverted to the file's pristine, never-before-formatted casing"? [Clarity, Ambiguity — Data-model.md §1 FormatOptions]
- [ ] CHK010 - Is "best-effort" (Contract §formatting-api.md, "formatting proceeds on a best-effort basis over whatever structure was recovered") given any observable definition, or is it left to the reader to infer what output a structurally-broken input (e.g. unmatched `IF`) actually produces? [Clarity, Ambiguity — Contract §formatting-api.md]
- [ ] CHK011 - Is "no reflow of line length / wrapping" (Contract §formatting-api.md, "What this contract does not promise") distinguished clearly enough from the in-scope whitespace normalization that a reader can tell which specific transformations are and are not covered? [Clarity — Contract §formatting-api.md]

## Requirement Consistency

- [ ] CHK012 - Does `format`'s exit-code contract (Spec §FR-020; Data-model.md §5 FormatOutcome/FormatReport) account for `FormatResult.diagnostics` at all, or does `FormatOutcome`'s four variants (`Unchanged`/`Changed`/`Written`/`WriteFailed`) silently have no path to surface a structural diagnostic that `format_bytes` returned for the same file? [Consistency, Conflict — Data-model.md §1 FormatResult vs §5 FormatOutcome, Spec §FR-020]
- [ ] CHK013 - Is `check`'s treatment of a file's diagnostics (always reported, FR-007) consistent with, or silently different from, `format`'s treatment of the *same* diagnostics when the same file is formatted instead of checked — and is that difference (if intended) stated anywhere? [Consistency, Gap — Spec §FR-007 vs FR-012–FR-021]
- [ ] CHK014 - Is the casing-rewrite scope described in Spec §FR-013 ("control-word/keyword-name tokens") worded identically to the scope described in Contract §formatting-api.md ("matched control-word/keyword-name tokens"), or could a reader reasonably conclude these two documents draw the boundary in two different places? [Consistency — Spec §FR-013, Contract §formatting-api.md]
- [ ] CHK015 - Does anything in `formatting-api.md` or `spec.md` state whether keyword synonyms (e.g. `PROCESS`/`PHASE` as accepted opener spellings, per `001-voyager-script-parser`) are canonicalized to one spelling by `format`, given the contract's casing feature only claims to rewrite *case*, not *spelling*? [Consistency, Gap — Contract §formatting-api.md "What this contract does not promise"]

## Acceptance Criteria Quality (Measurability)

- [x] CHK016 - Can SC-004's idempotency claim ("Running `drut format --write` twice... produces no further file changes") be objectively verified without first resolving CHK001's canonical-form gap, given a verifier needs to know the target form to confirm a second pass changes nothing *by construction* rather than *by coincidence*? [Measurability — Spec §SC-004] — **RESOLVED 2026-08-09** (downstream of CHK001): FR-012's concrete rule set makes idempotency checkable by construction rather than by coincidence.
- [x] CHK017 - Is "identical statement/block structure... except for `Span` positions shifting" (Data-model.md §1 FormatResult validation rules; Spec §SC-005) precise enough to be checked automatically (e.g. by an equality rule that ignores `Span` but not other fields), or does it leave room for a reviewer to disagree about which structural differences are "shifting" versus a real regression? [Measurability — Data-model.md §1, Spec §SC-005] — **RESOLVED 2026-08-09**: the validation rule now names exactly which tokens may differ — casing-normalized tokens (FR-015) and `EncodingFidelity::Recovered` re-encoded tokens (FR-013(b)) — closing the "which differences count as shifting" ambiguity.
- [x] CHK018 - Is there a stated, checkable criterion for `FormatResult.changed` (e.g. "true iff `text` is not byte-identical to the decoded input") rather than an informally-described flag? [Measurability, Gap — Data-model.md §1 FormatResult] — **RESOLVED 2026-08-09** (bonus, surfaced while closing CHK019/020): `changed` is now defined precisely as `text.as_bytes() != source` (byte-level comparison against the actual raw input, not a decoded intermediate) — this was also needed to make a pure-encoding-recovery change (no whitespace/casing difference) correctly report `changed: true`.

## Scenario & Edge Case Coverage

- [x] CHK019 - Is the interaction between FR-034's `InvalidEncoding` replacement-character substitution and `format --write` specified — i.e., does a requirement confirm or deny that writing a formatted file back to disk permanently replaces a genuinely-undecodable byte with the UTF-8 encoding of U+FFFD, and whether that counts as "altering meaningful content" under FR-013? [Coverage, Conflict — Spec §FR-013 vs FR-034 (001-voyager-script-parser), Contract §formatting-api.md] — **RESOLVED 2026-08-09**: FR-025 now requires `format` to refuse to persist (under `--write`, and flagged the same way in every other mode) any file whose decoding produced an `InvalidEncoding` diagnostic — confirmed as "MUST NOT," not left ambiguous. `EncodingFidelity::Lossy` (data-model.md §1) is the carrier of this classification; `contracts/formatting-api.md`'s new "Encoding safety" section states the core crate still computes a best-effort result (never refuses to *run*) while the CLI-layer refusal to *write* is documented as the resolving policy.
- [x] CHK020 - Is the (narrower, more common) case of a *recoverable* Windows-1252-fallback byte — one that decodes successfully and produces no diagnostic — covered by a requirement stating whether `format --write` re-encodes it to UTF-8 on disk, changing the file's raw bytes even though the represented character is unchanged? [Coverage, Gap — Spec §FR-013, FR-034 (001-voyager-script-parser)] — **RESOLVED 2026-08-09**: FR-013(b) names this explicitly as the one narrow, named exception to "MUST NOT alter meaningful content" — `format` persists the recovered byte in decoded UTF-8 form — paired with FR-024's new requirement that every such occurrence be reported visibly in every output mode, not discovered only by re-diffing later.
- [ ] CHK021 - Is idempotency (FR-014) scoped explicitly to *identical* `FormatOptions` across both calls, and is the cross-option case addressed — e.g. does formatting with `--casing=upper` and later formatting the *same, already-uppercased* file with casing off (`None`) leave the casing alone, per CHK009's reading, or could "untouched" there be misread as "reverted"? [Coverage, Gap — Data-model.md §1 FormatResult validation rules, FR-014] — not in this pass's scope; still open.
- [ ] CHK022 - Is a corpus fixture (real or hand-written) required to exercise `format_bytes`'s `InvalidEncoding` path specifically, given `001-voyager-script-parser`'s own 161-file corpus validation found zero files triggering it? [Coverage, Gap — Spec §FR-021, Plan.md "voyager-core... golden-file/idempotency/structural-equivalence suite"] — still open; now higher-stakes given FR-025's refusal behavior needs its own test coverage too (not just the recovered-byte path). Recommend `/speckit-tasks` include a hand-written fixture for both `EncodingFidelity::Recovered` and `::Lossy`.
- [ ] CHK023 - Are line-ending requirements (CRLF vs LF) for `format`'s output stated as in-scope or out-of-scope for "whitespace normalization," given Plan.md's Technical Context only says writes "must not corrupt line endings the formatter isn't explicitly asked to normalize" without saying whether it's ever asked? [Coverage, Ambiguity — Plan.md Technical Context, Contract §formatting-api.md]

## Non-Functional Requirements

- [ ] CHK024 - Is there a stated thread-safety/statelessness guarantee for `format`/`format_bytes` comparable to `parse`/`parse_bytes`'s determinism guarantee, given SC-007's 5-second/161-file target may require the CLI to call these concurrently across files? [Gap, NFR — Contract §formatting-api.md, Spec §SC-007]
- [ ] CHK025 - Is a performance expectation stated for `format`/`format_bytes` at all (even a qualitative one, the way `001-voyager-script-parser`'s plan.md states a per-file parse-time expectation), or does SC-007 only bound `check`, leaving `format`'s per-file cost undocumented? [Gap, NFR — Plan.md Performance Goals, Spec §SC-007]

## Dependencies & Assumptions

- [ ] CHK026 - Is the plan.md/research.md claim that a future LSP format-on-save consumer will reuse `format`/`format_bytes` (Contract §formatting-api.md "Stability expectations for adapters") treated as a binding constraint on this contract's shape, or only as forward-looking color that doesn't gate this phase's design? [Assumption — Contract §formatting-api.md, Plan.md Constitution Check Principle I row]
- [x] CHK027 - Is the assumption that "casing decisions require no information beyond what `parse`/`parse_bytes` already recognizes case-insensitively" (Contract §formatting-api.md "Case sensitivity carried through") validated against every FR-013-listed exclusion (e.g. does a value token that happens to textually match a control word, like a string literal `"IF"`, risk being miscategorized as a casing target)? [Assumption, Gap] — **RESOLVED 2026-08-09** (bonus, same edit as CHK006): FR-015 now states casing only ever touches a token already structurally classified as a control word/keyword name by parsing — a value token that happens to textually match one is never miscategorized, since classification comes from parse structure, not text matching.

## Notes

- The highest-priority finding is **CHK001/CHK008**: the contract repeatedly refers
  to "this feature's canonical form" without ever defining what it is — every other
  clarity/measurability item in this checklist (CHK002, CHK003, CHK006, CHK016,
  CHK017) is downstream of that same gap.
- The second-highest-priority finding is **CHK019/CHK020**: a plausible conflict
  between FR-013 ("MUST NOT alter any token's meaningful... content — except...
  casing") and the unavoidable consequence of `format --write` re-encoding
  Windows-1252-fallback or genuinely-undecodable bytes to UTF-8. This should be
  resolved (either by an explicit carve-out in FR-013, or a byte-preserving output
  mode) before `/speckit-tasks` breaks this contract into implementation tasks.
- Check items off as completed; add findings/resolutions inline or link back to an
  updated `contracts/formatting-api.md`/`spec.md` section.

## Re-run confirmation (2026-08-09)

Requested closure set — **CHK001, CHK002, CHK003, CHK006, CHK016, CHK017, CHK019,
CHK020 — all confirmed CLOSED** against the updated `spec.md` (FR-012's seven-rule
canonical form and its corpus-survey citation; FR-013(a)/(b)'s two named
exceptions; new FR-024/FR-025 encoding-safety requirements; SC-008), `data-model.md`
§1 (`EncodingFidelity`, redefined `changed`, refined validation rules) and §5
(`FormatReport.unsafe_encoding_files`/`recovered_encoding_files`, the `Fatal`
derivation rule extended to cover a `Lossy` file in *any* mode), and
`contracts/formatting-api.md`'s new "Encoding safety" section.

Three items were closed as a direct side effect without being separately requested:
**CHK008** (paired with CHK001 in this file's own Notes section as the same
highest-priority finding — missed in the original per-item pass, corrected here),
**CHK018** (`changed`'s definition — needed to make a pure-encoding-recovery change
report correctly), and **CHK027** (casing-target scope — needed to state the
label/`@variable@` exclusion for CHK006). All three are annotated above.

**Still open, out of scope for this pass**: CHK004, CHK005, CHK007, CHK009–CHK015,
CHK021–CHK026 (CHK022's note was updated to flag it now also needs
`EncodingFidelity::Lossy` fixture coverage, not just `Recovered`, but it remains
unresolved). None of these block `/speckit-tasks` on their own — they're narrower
completeness/consistency items, not the two structural gaps this round targeted.
