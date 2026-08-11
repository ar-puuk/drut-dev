# Specification Checklist: Range-Formatting Boundary Behavior

**Purpose**: Validate that the spec/research/contracts/tasks requirements
for the diff-based range-formatting mechanism actually specify — completely,
clearly, and consistently — what must happen when a pasted range itself
opens or closes a block, changing indentation depth for lines *outside*
the requested range. Not a test run; a review of whether the requirements
are precise enough for an implementer to get this case right without
guessing, and whether tasks.md actually operationalizes it into a required
test case rather than only naming two weaker ones.

**Created**: 2026-08-11
**Feature**: [spec.md](../spec.md) · [research.md](../research.md) §2 ·
[contracts/range-formatting-api.md](../contracts/range-formatting-api.md) ·
[tasks.md](../tasks.md) T006

## Edge Case Coverage

- [ ] CHK001 - Does `contracts/range-formatting-api.md`'s Tests section
      specify a case where the requested range itself contains a block
      opener or closer that changes indentation depth for lines *outside*
      the range — distinct from the currently-listed
      `change_outside_requested_range_is_not_returned`, whose own
      description (research.md §2 / contracts §Tests) is "two separate
      misindented lines," i.e. two *unrelated*, independently-wrong lines,
      not one edit that causally shifts nesting depth for its neighbors?
      [Gap, Coverage; contracts/range-formatting-api.md §Tests]
- [ ] CHK002 - Are both directions of "the paste straddles a block boundary
      only partially" (spec.md Edge Cases) specified separately — a paste
      that *opens* a block (shifting depth for lines after the range) and
      a paste that *closes* a block opened before the range (shifting
      depth for lines that were already after that earlier opener) — or
      does the current wording describe only one shape and leave the other
      to be inferred? [Coverage, Ambiguity; spec.md Edge Cases]
- [ ] CHK003 - Is it stated anywhere (not just inferable from research.md
      §2's general algorithm description) that a line *inside* the
      requested range, whose correct indentation depends on structural
      context established *before* the range (e.g. an already-open block
      from earlier in the document), is expected to already resolve
      correctly via the whole-document format pass — i.e. is this a
      documented guarantee or a silent assumption? [Clarity, Gap;
      research.md §2]

## Acceptance Criteria Quality

- [ ] CHK004 - Is there a concrete, example-level acceptance scenario
      (spec.md US2, or an Edge-Case-derived scenario) stating exactly
      which lines must change and which must not for a paste that both
      reindents its own contents *and* would ripple into surrounding
      context if the range filter weren't applied — or is the "only the
      portion within the range" bound stated solely as general prose in
      FR-003, with no worked example to check an implementation against?
      [Measurability, Spec §FR-003, US2]
- [ ] CHK005 - Does `contracts/range-formatting-api.md`'s algorithm
      description (step 4, `filter_to_range`) state its inclusive
      boundary rule (data-model.md §1) generally enough to explicitly
      cover a *structural* line (a block opener/closer sitting exactly at
      `range.start.line` or `range.end.line`), or does its current
      phrasing/example only address an ordinary body-statement line?
      [Clarity; contracts/range-formatting-api.md, data-model.md §1]

## Requirement Consistency

- [ ] CHK006 - Does `tasks.md`'s T006 description, which names the two
      specific test cases from `contracts/range-formatting-api.md` by
      name, actually require a nesting-depth-changing/block-boundary test
      case by name or description — or does its current wording let T006
      be satisfied while only ever exercising the two currently-named
      cases, neither of which is a block-boundary case (per CHK001)?
      [Consistency, Traceability; tasks.md T006, contracts/range-formatting-api.md §Tests]

## Dependencies & Assumptions

- [ ] CHK007 - Is the assumption underpinning the entire diff-based
      strategy — that `voyager-core`'s formatter never changes line
      count/order — explicitly reconfirmed as holding for the "paste adds
      or removes block nesting" shape specifically, or only asserted in
      general terms (research.md §2) without a stated tie-back to this
      particular risk, which is the shape most likely to stress-test that
      guarantee in practice? [Assumption; research.md §2]

## Notes

- All seven items trace to a real, currently-underspecified or
  under-tested gap: neither test case named in
  `contracts/range-formatting-api.md` (and therefore neither test named in
  `tasks.md` T006) exercises a paste that itself opens or closes a block —
  the scenario class the user's own focus request named specifically.
  CHK001/CHK006 are the two highest-priority items; CHK002–CHK005/CHK007
  are the clarity/consistency gaps that make CHK001's gap possible to miss
  without noticing.
- Items marked incomplete require `spec.md`/`research.md`/
  `contracts/range-formatting-api.md`/`tasks.md` updates before
  `/speckit-implement` — resolving them is a documentation/task-list fix,
  not a code change, since implementation hasn't started on this branch.
