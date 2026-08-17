# Requirements Quality Checklist: Token Hover Shows Assigned Value

**Purpose**: Validate the quality (completeness, clarity, consistency,
measurability, edge-case coverage) of `spec.md`'s own requirements — a reviewer's
pre-implementation gate, not a test of the implementation.
**Created**: 2026-08-16
**Feature**: [spec.md](../spec.md)
**Depth**: Standard **Audience**: PR reviewer

## Requirement Completeness

- [ ] CHK001 Are requirements defined for what happens when the same `@token@`
      name is assigned in *both* the open document and a `READ FILE`'d file, with
      no reassignment after the `READ FILE` line? [Completeness, Spec §FR-004] —
      confirm this reduces cleanly to the general ordering rule rather than
      needing its own special case.
- [x] CHK002 Is a requirement stated for what a hover response contains when
      `variable_ref_at` finds a reference but the reference's own name is empty
      or malformed (e.g. adjacent `@@`)? [Gap] — resolved: spec.md Edge Cases now
      states this falls back to existing behavior via FR-008, not a special case.
- [ ] CHK003 Are requirements defined for the maximum/minimum number of
      `READ FILE` statements one open document may contain before this feature's
      guarantees are expected to hold (e.g. the real corpus's 82-in-one-file
      case)? [Completeness, Spec Assumptions]
- [x] CHK004 Is a requirement stated for whether a `READ FILE` target that
      resolves to the *same* open document itself (a literal self-reference) is
      handled, or is this explicitly out of scope? [Gap, Edge Case] — resolved:
      spec.md Edge Cases now states this is a harmless no-op, not a special case.

## Requirement Clarity

- [ ] CHK005 Is "most recently assigned" (spec.md Input, US1) precisely defined
      in terms of source position, not just described narratively? [Clarity,
      Spec §FR-004] — confirm FR-004's interleaved-ordering description is
      sufficient for an implementer to derive a total order without further
      interpretation.
- [ ] CHK006 Is "one level of literal `READ FILE` inclusion" (spec.md
      Assumptions) unambiguous about which file's `READ FILE` statements count
      as "the next level" versus excluded — i.e., is it clear that a
      `READ FILE`'d file's *own* `READ FILE` statements are never followed,
      worded so no reader could interpret "one level" as "one level from each
      newly-reached file, recursively"? [Clarity, Spec §FR-003]
- [ ] CHK007 Is "literal (non-token-built) path" given a precise, checkable
      definition (e.g. contains no `@...@` substitution) rather than left to
      intuition? [Clarity, Spec §FR-003]
- [ ] CHK008 Is "falls back to existing behavior" (FR-008) specific enough to be
      independently verified without cross-referencing a different feature's own
      spec (i.e., does spec.md name *which* existing behavior — block info,
      spell-check nudge, or no hover — rather than leaving it implicit)?
      [Clarity, Spec §FR-008]

## Requirement Consistency

- [ ] CHK009 Do US1 Acceptance Scenario 3 ("not-yet-executed" exclusion) and the
      Edge Cases entry for a `READ FILE` line at/after the hovered position use
      the same cutoff rule (at-or-after vs. strictly-after) without a subtle
      mismatch between the two? [Consistency, Spec §US1/Edge Cases]
- [ ] CHK010 Does FR-005's case-insensitivity requirement apply uniformly to
      *both* the hovered token's own name and every candidate assignment's
      target, or could a reader interpret it as applying to only one side of
      the comparison? [Consistency, Spec §FR-005]
- [ ] CHK011 Is the "no reverse resolution" boundary (Assumptions) consistent
      with FR-003's own wording — could FR-003 be read in isolation as allowing
      some form of reverse lookup, without the Assumptions section's explicit
      exclusion? [Consistency]

## Acceptance Criteria Quality

- [x] CHK012 Can SC-003 ("zero tolerance for a wrong value shown with apparent
      confidence") be objectively verified, or does it rely on a subjective
      judgment of what counts as "apparent confidence"? [Measurability, Spec
      §SC-003] — resolved: reworded to an objective, testable statement (no
      test/fixture/manual check ever surfaces a mismatch).
- [ ] CHK013 Is SC-002's "every time such an assignment exists and the
      referenced file is reachable on disk" independently testable — i.e., is
      "reachable on disk" itself defined elsewhere (FR-006/FR-007) so this
      criterion doesn't introduce an undefined term of its own? [Measurability,
      Spec §SC-002]
- [ ] CHK014 Are the acceptance scenarios under US2 sufficient to derive a test
      for *every* FR-004 ordering branch (same-file-only, read-file-only,
      same-file-overrides-read-file, read-file-after-hover-excluded), or does at
      least one branch lack a corresponding scenario? [Coverage, Spec §US2]

## Scenario Coverage

- [ ] CHK015 Are requirements defined for hovering an `@token@` reference that
      appears *inside* the value of another assignment (e.g. `A = @B@`), as
      opposed to only inside a `PRINT`/pair-value context? [Coverage, Gap]
- [ ] CHK016 Are requirements defined for a token whose only assignment is
      inside a `READ FILE`'d file that is itself a `.s` file (not `.block`) —
      does the spec's scope depend on file extension at all, or is that
      explicitly irrelevant? [Coverage, Clarity]
- [ ] CHK017 Are requirements defined for the case where the hovered document
      and the `READ FILE` target are the same file opened under two different
      URIs (e.g. differing casing on a case-insensitive filesystem)? [Coverage,
      Edge Case, Gap]

## Edge Case Coverage

- [ ] CHK018 Is behavior specified for a `READ FILE` value that is a bare
      (unquoted) word rather than a quoted string, if Voyager's grammar permits
      that shape at all? [Edge Case, Gap]
- [ ] CHK019 Is behavior specified for a `READ FILE` literal path containing
      `..` segments that resolve *outside* any file the editor's workspace
      would otherwise consider in scope? [Edge Case, Gap]
- [x] CHK020 Is behavior specified for an assignment whose target uses a
      bracketed subscript (e.g. `MW[1]`) versus a hovered `@token@` with a bare
      name — does the spec clarify these are never considered a match for each
      other? [Edge Case, Clarity] — resolved: spec.md Edge Cases now states
      matching is by exact full name only, never a prefix/partial match.
- [ ] CHK021 Is behavior specified for a document so large, or a `READ FILE`
      target so large, that resolution could be perceptibly slow — or is
      performance for this feature explicitly deferred/out of scope? [Gap,
      Non-Functional]

## Non-Functional Requirements

- [ ] CHK022 Are freshness/staleness requirements (spec.md Assumptions:
      "reads are always fresh, never cached") precise about *when* a stale read
      could still be observed (e.g. a `READ FILE` target edited by another
      process mid-resolution), or is this left ambiguous? [Clarity, Spec
      §Assumptions]
- [ ] CHK023 Are there any requirements at all governing the response latency of
      a token-hover request, given it may now involve a disk read that a plain
      block-info hover never needed? [Gap, Non-Functional]

## Dependencies & Assumptions

- [ ] CHK024 Is the Assumption that Voyager identifiers are case-insensitive
      explicitly traced to evidence (real corpus, existing codebase convention)
      rather than merely asserted? [Traceability, Spec §Assumptions] — confirm
      it reads as evidenced, not asserted.
- [ ] CHK025 Is the dependency on `crates/drut-lsp/src/hover.rs`'s existing
      fallback chain (block-info, spell-check nudge) stated precisely enough
      that a reader unfamiliar with that file's current behavior could still
      verify FR-010 independently? [Clarity, Spec §FR-010]

## Ambiguities & Conflicts

- [ ] CHK026 Does the spec anywhere imply this feature evaluates or interprets
      a token's *value* (e.g. arithmetic, further substitution) rather than
      only ever displaying the literal assigned text verbatim — and if so, is
      that intentional or an unintended scope expansion? [Ambiguity, Spec
      §FR-009]
- [ ] CHK027 Is there any residual ambiguity between "hover shows a value" and
      "hover shows *the* value" in a case where Voyager's own real runtime
      behavior (e.g. a reassignment inside a conditional block whose branch
      never actually executes) could differ from what this feature reports —
      is the spec explicit that this feature reports positional history, not
      guaranteed runtime truth? [Ambiguity, Spec §Data Model/Key Entities]

## Notes

- Check items off as completed: `[x]`
- Add comments or findings inline
- This checklist tests `spec.md`'s own requirement quality, not
  `plan.md`/`tasks.md`'s implementation approach — an item failing here means
  the spec needs a wording/content change, not that the design is wrong.
