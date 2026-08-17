# Specification Quality Checklist: Operator Spacing Normalization

**Purpose**: Requirements quality review, standard depth — focused on the two areas of this
spec most exposed to ambiguity: exact operator-recognition/merge scope (multi-char comparisons,
unary-vs-binary, continuation-position interaction), and `auto` alignment-run boundary
precision.
**Created**: 2026-08-17
**Feature**: [spec.md](../spec.md)
**Depth**: Standard | **Audience**: Reviewer (PR-time), default per no explicit audience stated

## Requirement Completeness

- [X] CHK001 - Is the full operator character set in scope (FR-002) closed and exhaustive, or
      could a reader reasonably wonder whether other Voyager operator-like characters (e.g. `^`
      as exponentiation, `&`/`|` as logical operators) are silently included or silently
      excluded? [Completeness, Spec FR-002] — **Fixed**: FR-002 now states the set is closed
      and explicitly excludes `^`/`&`/`|`.
- [X] CHK002 - Are requirements defined for what happens to a string/quoted-literal boundary
      character that happens to look like an in-scope operator immediately adjacent to the
      quote (e.g. `LIST='x'+y`), or is "never altering values inside string/quoted literals"
      (FR-010) specific enough to resolve that case unambiguously? [Clarity, Spec FR-010,
      Edge Cases] — **Real gap found and fixed**: confirmed by direct testing that
      `tokenize("LIST='a+b'\n")` emits an operator-indistinguishable `Punctuation("+")` for the
      in-string `+`. Added FR-010a, an Edge Case, research.md §9, and a `quoted_token_mask`
      requirement in data-model.md/contracts/tasks.md (new task T003).
- [X] CHK003 - Does the spec define what happens when an alignment run (US2) would need to
      align a left-hand side that itself contains a subscript with an embedded expression
      (e.g. `MW[I+1]`), or is "left-hand side" in FR-006 assumed to mean the literal source
      text without qualification? [Gap, Spec FR-006] — **Resolved, no change needed**:
      `lhs_width` (data-model.md §3) is the post-`Fixed`-normalization literal span width,
      regardless of subscript contents — no special-casing required.
- [X] CHK004 - Are requirements defined for the interaction between `; FMT: OFF`/`ON` regions
      and an alignment run that starts before the protected region and would otherwise continue
      into it, beyond the general "every existing formatting guarantee... continue[s] to hold"
      statement in FR-010? [Gap, Spec FR-010, Edge Cases] — **Fixed**: FR-008 now states a
      protected `Assignment` member breaks the run and is excluded entirely, not
      skipped-while-counted; mirrored in data-model.md §3, contracts, and tasks T022/T023/T025.

## Requirement Clarity

- [X] CHK005 - Is "exactly one space on each side" (FR-002) clear about which whitespace
      characters count (space only, or tabs too) when the source already contains non-space
      whitespace around an operator? [Clarity, Spec FR-002] — **Fixed**: added an Edge Case —
      any space/tab run normalizes to a single literal space.
- [X] CHK006 - Is the unary/binary disambiguation rule (FR-003) precise enough that two readers
      would agree on the same classification for every token immediately preceding a `+`/`-`,
      not just the four examples named in the Assumptions section? [Clarity, Spec FR-003,
      Assumptions] — **Real gap found and fixed**: the Assumptions wording omitted "or another
      operator" (present in research.md/contracts all along), which would have misclassified
      `A + -B`. Reworded to match the actual, fuller rule.
- [X] CHK007 - Is "block nesting depth" in FR-006/FR-008 defined with enough precision that a
      reader could determine, without consulting the implementation, whether two `Assignment`
      statements on either side of an `IF`/`ENDIF` pair at the same textual indentation but
      different structural nesting count as "the same depth"? [Clarity, Spec FR-006, FR-008] —
      **Resolved, no change needed**: research.md §6's sibling-Vec<Node>-adjacency framing
      already answers this precisely; the diagnosed-block Edge Case covers the malformed case.

## Requirement Consistency

- [X] CHK008 - Are FR-002's "every occurrence... to exactly one space" and FR-012's
      "leading-side-only" continuation-position exception stated in a way that a reader
      encounters the exception before assuming FR-002 is unconditional, or could the two be read
      as contradictory in isolation? [Consistency, Spec FR-002, FR-012] — **Resolved, no change
      needed**: the Edge Cases section states the continuation exception explicitly and FR-012
      is its own numbered requirement, not a buried caveat.
- [X] CHK009 - Do FR-007's "never a member of... an alignment run, even when adjacent to one"
      and FR-008's "any non-`Assignment` statement" break condition overlap in a way that makes
      one of them redundant, or does each cover a distinct case a reader needs both statements
      to understand? [Consistency, Spec FR-007, FR-008] — **Resolved, no change needed**:
      intentional — FR-007 removes ambiguity about the one case a reader might guess wrong (a
      `Control` statement's `=` looking assignment-like); FR-008 is the fully general rule.

## Scenario & Edge Case Coverage

- [X] CHK010 - Are requirements defined for a `ShellEscape` or `Label` statement sitting between
      two `Assignment` statements — does it break an alignment run the same way a `Control`
      statement does, or is this left to the general "any non-`Assignment` statement" wording
      without an explicit example? [Coverage, Spec FR-008, Edge Cases] — **Resolved, no change
      needed**: FR-008's "any non-`Assignment` statement" is already fully general and covers
      both implicitly; no per-kind enumeration needed.
- [X] CHK011 - Are requirements defined for an `Assignment` statement whose left-hand side spans
      multiple physical lines via continuation — does it participate in an alignment run at all,
      and if so, against which line's column? [Gap, Edge Cases] — **Fixed**: added an Edge Case
      — alignment only cares about the `=` token's own line position, independent of where the
      value continues to.
- [X] CHK012 - Are non-functional expectations (e.g., formatting remaining fast on a large script
      with many alignment runs) addressed as in-scope or explicitly out-of-scope for this
      feature, or left unaddressed entirely? [Coverage, Non-Functional] — **Resolved, no change
      needed**: plan.md's Performance Goals already covers this (linear per-statement pass, no
      new full-file re-scan).

## Dependencies & Assumptions

- [X] CHK013 - Is the dependency on `ROADMAP.md` item 11 (Bill's data-reference role split)
      remaining wholly unrelated to this feature stated clearly enough that a reader wouldn't
      wonder whether operator spacing interacts with it? [Traceability, Spec Assumptions] —
      **Resolved, no change needed**: already explicit in Assumptions.
- [X] CHK014 - Is the assumption that `gofmt`'s alignment model (not Prettier's/Tidyverse's
      refusal to align) is the right precedent cross-referenced against every acceptance
      scenario that could otherwise imply a different, softer alignment behavior (e.g. an
      alignment that only applies on request rather than automatically)? [Consistency,
      Assumptions, Success Criteria] — **Resolved, no change needed**: Assumptions and US2's
      "Why this priority" both already frame this consistently.
- [X] CHK015 - Is the exact configuration surface shape (field/flag/param name) explicitly
      deferred to planning in a way a reader wouldn't mistake for already being fixed by this
      spec? [Clarity, Assumptions] — **Resolved, no change needed**: already explicit in
      Assumptions.

## Notes

- Focus areas selected: operator-recognition/merge/unary-binary correctness and `auto`
  alignment-run boundary precision — the two places this spec's own design conversation
  (captured in `research.md`) shows the most real complexity, and therefore the most risk of an
  underspecified requirement.
- Depth: Standard, audience Reviewer/PR-time — defaults applied (code-related feature, no
  explicit audience stated for this checklist run).
- No user-specified must-have items beyond the standard-depth requirements-quality review — all
  15 items are self-generated from spec.md content, not consolidated from external instruction.
