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

- [X] CHK001 - Does `contracts/range-formatting-api.md`'s Tests section
      specify a case where the requested range itself contains a block
      opener or closer that changes indentation depth for lines *outside*
      the range — distinct from the currently-listed
      `change_outside_requested_range_is_not_returned`, whose own
      description (research.md §2 / contracts §Tests) is "two separate
      misindented lines," i.e. two *unrelated*, independently-wrong lines,
      not one edit that causally shifts nesting depth for its neighbors?
      [Gap, Coverage; contracts/range-formatting-api.md §Tests] —
      **Closed 2026-08-11**: two new named cases added,
      `paste_that_opens_a_block_only_returns_the_in_range_edit` and
      `paste_that_closes_a_block_only_returns_the_in_range_edit`, each
      with a full before/after worked example verified directly against
      `voyager_core::format` (a throwaway `crates/voyager-core/examples/
      diag_check.rs` scratch run, per this project's own convention —
      deleted after use).
- [X] CHK002 - Are both directions of "the paste straddles a block boundary
      only partially" (spec.md Edge Cases) specified separately — a paste
      that *opens* a block (shifting depth for lines after the range) and
      a paste that *closes* a block opened before the range (shifting
      depth for lines that were already after that earlier opener) — or
      does the current wording describe only one shape and leave the other
      to be inferred? [Coverage, Ambiguity; spec.md Edge Cases] —
      **Closed 2026-08-11**: both directions now have their own named
      contract test (CHK001) *and* research.md §2 states explicitly that
      they're the same underlying phenomenon from opposite ends — a single
      unbalanced paste always "steals" a pre-existing opener or closer
      elsewhere to rebalance against. (spec.md's own Edge Cases prose still
      narrates only one direction informally; the binding requirement
      documents — contracts + research — are where this resolution
      actually lives, and both now cover it explicitly.)
- [X] CHK003 - Is it stated anywhere (not just inferable from research.md
      §2's general algorithm description) that a line *inside* the
      requested range, whose correct indentation depends on structural
      context established *before* the range (e.g. an already-open block
      from earlier in the document), is expected to already resolve
      correctly via the whole-document format pass — i.e. is this a
      documented guarantee or a silent assumption? [Clarity, Gap;
      research.md §2] — **Closed 2026-08-11, as a byproduct of CHK001/
      CHK007's edits, not separately targeted**: both new worked examples
      concretely demonstrate this (the pasted line's own correct
      indentation in both fixtures depends on the `a=1`/`b=2` context
      established *before* the requested range, and resolves correctly),
      reinforcing research.md §2's pre-existing general statement
      ("`voyager-core`'s formatter derives a line's correct indentation
      from its position in the *whole* document's block-nesting
      structure") with a concrete, verified instance rather than leaving
      it purely general.

## Acceptance Criteria Quality

- [X] CHK004 - Is there a concrete, example-level acceptance scenario
      (spec.md US2, or an Edge-Case-derived scenario) stating exactly
      which lines must change and which must not for a paste that both
      reindents its own contents *and* would ripple into surrounding
      context if the range filter weren't applied — or is the "only the
      portion within the range" bound stated solely as general prose in
      FR-003, with no worked example to check an implementation against?
      [Measurability, Spec §FR-003, US2] — **Closed 2026-08-11**: both new
      contract test cases include the full original text, the full
      whole-document-reformatted text, and an explicit line-by-line
      accounting of which lines changed (in-range, must be returned) vs.
      which changed but must be filtered out (out-of-range) vs. which
      didn't change at all.
- [X] CHK005 - Does `contracts/range-formatting-api.md`'s algorithm
      description (step 4, `filter_to_range`) state its inclusive
      boundary rule (data-model.md §1) generally enough to explicitly
      cover a *structural* line (a block opener/closer sitting exactly at
      `range.start.line` or `range.end.line`), or does its current
      phrasing/example only address an ordinary body-statement line?
      [Clarity; contracts/range-formatting-api.md, data-model.md §1] —
      **Closed 2026-08-11**: both new worked examples use a single-line
      range (`range.start.line == range.end.line`) sitting exactly on a
      block opener/closer line, not an ordinary body statement — the
      already-general boundary rule (data-model.md §1's wording never
      special-cased content type) is now concretely proven to cover the
      structural-line-at-the-boundary case, not just asserted to.

## Requirement Consistency

- [X] CHK006 - Does `tasks.md`'s T006 description, which names the two
      specific test cases from `contracts/range-formatting-api.md` by
      name, actually require a nesting-depth-changing/block-boundary test
      case by name or description — or does its current wording let T006
      be satisfied while only ever exercising the two currently-named
      cases, neither of which is a block-boundary case (per CHK001)?
      [Consistency, Traceability; tasks.md T006, contracts/range-formatting-api.md §Tests] —
      **Closed 2026-08-11**: T006 now names all seven test cases
      explicitly, calls out the two block-boundary cases by name in bold,
      and states outright that T006 is "not satisfied by only the first
      five tests passing."

## Dependencies & Assumptions

- [X] CHK007 - Is the assumption underpinning the entire diff-based
      strategy — that `voyager-core`'s formatter never changes line
      count/order — explicitly reconfirmed as holding for the "paste adds
      or removes block nesting" shape specifically, or only asserted in
      general terms (research.md §2) without a stated tie-back to this
      particular risk, which is the shape most likely to stress-test that
      guarantee in practice? [Assumption; research.md §2] — **Closed
      2026-08-11**: research.md §2 gained an explicit "Verified against the
      real formatter, both directions" paragraph — both fixtures go in at
      7 lines and come out at 7 lines despite the reindentation ripple and
      a genuine `UnmatchedIf` diagnostic being present, tying the general
      guarantee to this exact scenario rather than leaving the connection
      implicit.

## Notes

- All seven items closed 2026-08-11. Root cause of all seven: neither test
  case named in `contracts/range-formatting-api.md` (and therefore neither
  test named in `tasks.md` T006) exercised a paste that itself opens or
  closes a block — the scenario class the user's own focus request named
  specifically. Fixed by adding two new contract test cases with verified
  (not hand-derived) worked examples, tightening T006's wording to require
  them by name, and adding an explicit research.md paragraph tying the
  line-count-preservation guarantee to this exact case.
- **A real finding surfaced during verification, not anticipated when this
  checklist was written**: a structurally-valid paste that genuinely shifts
  indentation for lines *outside* the requested range always coincides
  with exactly one transient `UnmatchedIf`/`UnmatchedLoop`-shaped
  diagnostic elsewhere in the document — the two are mutually exclusive by
  construction (an unbalanced paste into an otherwise-balanced document
  necessarily either leaves an opener dangling or produces a stray
  closer). This connects spec.md's two separate Edge Cases (the
  already-diagnosed-document case, and the block-boundary case) as
  instances of the same underlying situation, which research.md §2 now
  states explicitly.
- Verification method: a throwaway `crates/voyager-core/examples/
  diag_check.rs` scratch script (this project's own established
  convention — see prior sessions' `diag_check.rs` uses), run via
  `cargo run -p voyager-core --example diag_check`, output inspected
  directly, then deleted — not committed, not left behind.
