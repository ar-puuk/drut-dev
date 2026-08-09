# Requirements Quality Checklist: FR-034 Byte-Oriented Decoding

**Purpose**: Validate the *wording quality* (completeness, clarity, consistency,
measurability, edge-case coverage) of FR-034 — the byte-oriented `tokenize_bytes`/
`parse_bytes` entry points, Windows-1252 fallback decoding, and the `InvalidEncoding`
diagnostic — added 2026-08-09 in response to a real non-UTF-8 file found in the T049
fixture corpus. This checklist tests whether the *requirements as written* are
complete and unambiguous — it does not test whether the implementation is correct.

**Created**: 2026-08-09
**Feature**: [spec.md](../spec.md) §FR-034 (see also
[contracts/public-api.md](../contracts/public-api.md),
[contracts/diagnostics.md](../contracts/diagnostics.md),
[data-model.md](../data-model.md) § Span, § Diagnostic)
**Depth**: Standard — smaller scope than the 2026-08-08 documentation-verification
checklist, same discipline
**Audience**: Author/reviewer deciding whether FR-034 is implementation-ready

## Requirement Completeness

- [ ] CHK001 - Does FR-034 (or any linked section) state what happens when the input
  is overwhelmingly or entirely undecodable bytes — e.g. a binary file mistakenly
  passed to `parse_bytes` — or is diagnostic volume in that case left completely
  unbounded, the same open question readiness.md's CHK005 already raised for
  structural diagnostics generally? [Gap, Spec §FR-034]
- [ ] CHK002 - Does FR-034 or contracts/public-api.md state whether a UTF-8
  byte-order-mark (`EF BB BF`) at the start of input — valid UTF-8, but not visible
  Voyager grammar content — is stripped, tokenized as ordinary content, or left
  unaddressed? Real Windows-authored files sometimes carry one. [Gap, Edge Case]
- [ ] CHK003 - Is there a Success Criterion (not just FR-034's own prose) asserting
  the "silent recovery" behavior specifically — that a byte resolving successfully
  under Windows-1252 produces zero diagnostics — or does SC-003's generic per-category
  fixture requirement leave that half of FR-034's contract unmeasured on its own?
  [Gap, Acceptance Criteria, Spec §SC-003]
- [ ] CHK004 - Does `tokenize_bytes`'s silent-discard of decode diagnostics (stated in
  contracts/public-api.md) also appear in spec.md's FR-034 itself, or only in the
  contract — could a reader of spec.md alone reasonably assume `tokenize_bytes`
  surfaces `InvalidEncoding` the way `parse_bytes` does? [Gap, Consistency, Spec
  §FR-034, Contracts §public-api.md]

## Requirement Clarity

- [ ] CHK005 - FR-034 says decoding falls back "wherever an individual byte sequence
  isn't valid UTF-8" but then speaks only of a single "byte" thereafter — is it clear
  from the wording alone that fallback is applied one byte at a time, not one
  multi-byte invalid sequence at a time, or could a reader assume the latter?
  [Ambiguity, Spec §FR-034]
- [ ] CHK006 - Is it explicit anywhere that FR-034's Windows-1252 fallback is a
  general byte-decoding policy, not a Voyager-grammar rule scoped to the 6.5 baseline
  the way FR-003–FR-033 are — or could FR-024's "record which Voyager version each
  rule was validated against" convention be read as applying to FR-034 too, when it
  doesn't (grammar_notes.rs's FR-034 entry uses a different, non-version baseline
  label)? [Clarity, Spec §FR-024, §FR-034]
- [ ] CHK007 - Is "that byte's Windows-1252 interpretation" in FR-034 clear that this
  means the *specific defined mapping table* (with five undefined code points), not
  the more permissive Latin-1/ISO-8859-1 (which has no undefined code points at all)
  that FR-034's own Assumptions-adjacent framing elsewhere invokes loosely? [Ambiguity,
  Spec §FR-034]

## Requirement Consistency

- [ ] CHK008 - Do contracts/diagnostics.md's `InvalidEncoding` row and data-model.md's
  Span note describe the *same* position semantics (running `char` count, not a byte
  offset) in mutually consistent terms, or could a reader of one without the other
  come away with a different understanding? [Consistency, Spec §data-model.md § Span,
  Contracts §diagnostics.md]
- [ ] CHK009 - Do FR-025 and SC-003's diagnostic-category lists both include FR-034
  now, in a way that can't silently drift apart again the next time a category is
  added — the same drift CHK011/CHK012 (readiness.md) already caught once for
  `MisplacedBreak`? [Consistency, Spec §FR-025, §SC-003]

## Edge Case Coverage

- [ ] CHK010 - Is there a documented edge case for a `RUN`/`PROCESS` block whose
  *opener or closer keyword itself* is corrupted by an undecodable byte (e.g. the
  literal text `RUN` with a stray byte fused into it) — does FR-034's per-byte
  substitution risk turning a real control word into an unrecognized one, and if so,
  is that acknowledged anywhere as a known interaction rather than left implicit?
  [Gap, Edge Case, Spec §FR-034 vs §FR-003]
- [ ] CHK011 - Is behavior specified for an undecodable byte landing *inside* an
  `@variable@` reference's delimiters, or inside a block comment — does FR-034's
  decode-before-tokenize ordering guarantee those constructs still recognize
  correctly around a substituted replacement character, or is this untested by the
  requirement text? [Gap, Edge Case, Spec §FR-034, §FR-005, §FR-010]

## Dependencies & Assumptions

- [ ] CHK012 - Does spec.md's Assumptions bullet on `Span`'s `char`-vs-UTF-16 question
  make clear that this is a pre-existing characteristic of *every* diagnostic's
  position, surfaced by FR-034 rather than introduced by it — or could a reader
  mistakenly conclude this is an `InvalidEncoding`-specific limitation? [Clarity,
  Spec §Assumptions]
- [ ] CHK013 - Is the formatter write-back Assumptions bullet (preserve vs. normalize
  non-UTF-8 bytes) explicit that *no* decision is made by this phase — i.e. that
  Phase 2 inherits an open question rather than an implied default — or could
  "flagged for Phase 2" be misread as "Phase 2 should preserve by default"? [Clarity,
  Spec §Assumptions]

## Notes

- This checklist is scoped to FR-034 and its directly linked contract/data-model
  sections only — it does not re-litigate the 2026-08-08 pass's items (already
  tracked in `grammar-amendments.md`/`readiness.md`).
- CHK001–CHK002, CHK005–CHK007, and CHK010–CHK011 are genuinely open wording
  questions, not implementation defects — each is a concrete ambiguity a future
  reader/implementer could reasonably resolve differently today.
- No items were marked complete `[x]` — this is a fresh review of new wording.
