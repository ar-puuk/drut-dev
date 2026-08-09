# Requirements Quality Checklist: FR-023 Subscripted Assignment Targets

**Purpose**: Validate the *wording quality* (completeness, clarity, consistency,
measurability, edge-case coverage) of FR-023's 2026-08-09 amendment — an assignment
target may carry one or more trailing bracketed subscripts (`MW[1] = value`,
`SUBAREAID[Seg_Idx][idx_SUBAREAID] = value`) — added after a full-corpus validation
run found `classify_statement` misclassifying this shape as `Control` instead of
`Assignment`. This specifically touches the FR-003/FR-023 Control-vs-Assignment
boundary already flagged this session (grammar-amendments.md CHK008, readiness.md)
as genuinely underdetermined without more data. This checklist tests whether the
*requirements as written* are complete and unambiguous — not whether
`classify_statement`'s implementation is correct.

**Created**: 2026-08-09
**Feature**: [spec.md](../spec.md) §FR-023 (see also
[data-model.md](../data-model.md) § Statement, Edge Cases, Assumptions)
**Depth**: Standard
**Audience**: Author/reviewer deciding whether the amendment is implementation-ready

## Requirement Completeness

- [ ] CHK001 - Does FR-023 state what happens for a *malformed* subscript — an
  unbalanced `[` with no matching `]` before end of statement (e.g. `MW[1 = value`)
  — or is the fallback behavior (not promoted to `Assignment`; the statement is
  classified some other way) left entirely to implementation discretion, undocumented
  at the requirement level? [Gap, Spec §FR-023]
- [ ] CHK002 - Does FR-023 (or the Assumptions section) state an upper bound on the
  number of chained subscripts recognized, or explicitly state there is none — given
  real fixtures confirm one and two, but the wording says "one or more" without
  addressing three or more? [Gap, Spec §FR-023]
- [ ] CHK003 - Does any requirement address what token content is permitted *inside*
  a subscript (e.g. a bare identifier/number vs. an arbitrary expression like
  `MW[i+1]`), or is "bracketed subscript" left undefined as to its interior, relying
  entirely on structural bracket-balancing with no content restriction stated
  anywhere? [Gap, Spec §FR-023]

## Requirement Clarity

- [ ] CHK004 - Is it unambiguous from FR-023's wording that the subscript is
  *optional* — i.e. that a bare, unsubscripted `identifier = value` remains valid
  Assignment exactly as before — or could "MAY include... a trailing subscript" be
  misread as narrowing the existing bare-identifier case? [Clarity, Spec §FR-023]
- [ ] CHK005 - Is "the whole subscripted expression, not just the leading name, is
  the assignment target" precise enough to tell a reader whether the target string
  includes exactly the literal source text between the identifier and `=` (brackets,
  contents, and all), or could "expression" be read as implying some normalized/
  evaluated form? [Ambiguity, Spec §FR-023]

## Requirement Consistency

- [x] CHK006 - Does FR-023's amendment clearly scope itself to the top-level
  `Assignment`-vs-`Control` statement-classification boundary only, in a way a reader
  would understand *without* needing the separate Assumptions bullet on subscripted
  `Control.pairs` keywords (e.g. `VOL[1]=`) to learn that the two are different,
  unrelated-in-fix boundaries — or does reading FR-023 alone risk the reader assuming
  it also covers pair-keyword subscripts? [Consistency, Spec §FR-023 vs Assumptions]
  — **Superseded 2026-08-09**: this item's premise (FR-023 and the `Control.pairs`
  gap as "different, unrelated-in-fix boundaries") no longer holds — after confirming
  the two were the identical bug shape with the identical fix, the pair-keyword gap
  was folded into FR-003 under the same pass as FR-023, not left as a separate
  unfixed gap. FR-023 and FR-003 now each state their own scope directly; no reader
  needs to infer a boundary between "fixed" and "not fixed" that no longer exists.
- [ ] CHK007 - Is FR-023's new subscript language consistent in terminology with the
  new Edge Cases bullet and data-model.md's `Assignment` entity description — same
  term ("bracketed subscript"), same scope (single and double), no drift between the
  three restatements of the same rule? [Consistency, Spec §FR-023, §Edge Cases,
  data-model.md § Statement]
- [ ] CHK008 - Does this amendment's fixture-corpus evidence citation (6,000+
  single-subscript occurrences in one file) get echoed consistently between FR-023's
  own text and research.md's addendum, or could a reader encounter two different
  characterizations of the same finding? [Consistency, Spec §FR-023, research.md §3]

## Edge Case Coverage

- [ ] CHK009 - Is there a documented edge case for a subscript containing a literal
  `]` character inside a quoted string (e.g. `MW['a]b'] = value`) — given this
  grammar has no quote-awareness anywhere else either, is that consistent, deliberate
  scope (structural bracket-matching only, no string-literal exception), or an
  unaddressed gap specific to this new rule? [Gap, Edge Case, Spec §FR-023]
- [ ] CHK010 - Is it addressed anywhere that a subscript's content may itself span a
  continuation-joined physical line (FR-006) — e.g. `MW[1,` followed by a
  continuation onto `2] = value` — or is this left to be inferred from FR-006's
  general statement-assembly-happens-before-classification ordering, never stated
  explicitly for this specific interaction? [Gap, Edge Case, Spec §FR-023, §FR-006]

## Dependencies & Assumptions

- [x] CHK011 - Is the newly-added Assumptions bullet on subscripted `Control.pairs`
  keywords (e.g. `VOL[1]=`) explicit that it is a *separate, unfixed* gap rather than
  something FR-023's amendment already resolves — i.e. does a reader who only skims
  FR-023 and this bullet's first sentence come away with the correct boundary, or
  does the distinction require reading the bullet in full? [Clarity, Spec
  §Assumptions]
  — **Superseded 2026-08-09**: same resolution as CHK006 — the gap was confirmed to
  be the same bug shape (verified empirically against the real
  `4pd_mainbody_distribution.block:780-781` line, before and after the fix) and
  folded into FR-003 under the same pass, so the bullet this item asked about now
  reads "resolved," not "separate, unfixed."
- [ ] CHK012 - Does the `DistributeINTRASTEP` deferral bullet added alongside this
  amendment cite a comparable evidentiary standard (one file, narrow occurrence
  count) to the pre-existing `WORD=value keyword=value...` deferral it's explicitly
  compared against, or does the comparison rest on assertion without the reader being
  able to verify the two are actually alike? [Traceability, Spec §Assumptions]

## Notes

- This checklist is scoped to the 2026-08-09 FR-023 amendment and its two sibling
  Assumptions bullets (`DistributeINTRASTEP`, subscripted pair-keywords) only — it
  does not re-litigate FR-003/CHK008 (already tracked in grammar-amendments.md/
  readiness.md) beyond how this amendment specifically touches that boundary.
- CHK001–CHK003 and CHK009–CHK010 are genuinely open wording questions, not evidence
  of a mistake — each is a concrete ambiguity a future implementer or reader could
  resolve differently today.
- No items were marked complete `[x]` — fresh review of new wording.
