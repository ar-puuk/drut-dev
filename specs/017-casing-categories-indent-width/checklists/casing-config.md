# Specification Quality Checklist: Casing Categories & Configuration Precedence

**Purpose**: Requirements quality review, standard depth — focused on the two areas of this
spec most exposed to ambiguity: the `data_references` category's scope/role handling, and the
configuration precedence between legacy and new settings.
**Created**: 2026-08-17
**Feature**: [spec.md](../spec.md)
**Depth**: Standard | **Audience**: Reviewer (PR-time), default per no explicit audience stated

## Requirement Completeness

- [ ] CHK001 - Is the data-reference category's membership closed (exactly the FR-004 list) or
      open-ended (FR-004 says "at minimum")? If open-ended, are criteria given for what would
      qualify a future addition to the list? [Completeness, Ambiguity, Spec FR-004]
- [ ] CHK002 - Does the spec define, even at a capability level, how a data-reference token's
      structural role is distinguished from an arbitrary same-named user identifier, or is the
      disambiguation mechanism left entirely unaddressed? [Gap, Spec Edge Cases]
- [ ] CHK003 - Are requirements defined for what happens when a project's configuration
      specifies conflicting values across the legacy setting and a new per-category setting for
      the same category, or is that interaction left entirely to planning? [Gap, Spec
      Assumptions]
- [ ] CHK004 - Does the spec address whether any two data-reference family members could ever
      textually collide (a token matching more than one recognized entry), or is non-collision
      assumed without being stated? [Gap, Edge Cases]

## Requirement Clarity

- [ ] CHK005 - Are the three structural "roles" a data-reference token can appear in (referenced
      by FR-005/US2 AS2) explicitly enumerated anywhere in the requirements, or only implied?
      [Clarity, Spec FR-005]
- [ ] CHK006 - Is "identically" in FR-013 ("expose the same...controls...identically") defined
      as identical naming, identical semantics, or both, across the command-line, editor, and
      MCP surfaces? [Ambiguity, Spec FR-013]
- [ ] CHK007 - Is the exact threshold for "unreasonably large" (US3 Edge Cases / Acceptance
      Scenario 2) quantified anywhere in the requirements, or intentionally deferred to
      planning? If deferred, is that deferral itself stated clearly enough that a reader
      wouldn't mistake the omission for an oversight? [Clarity, Spec US3 AS2, Assumptions]
- [ ] CHK008 - Is "real corpus-shaped script content" in SC-002 objectively defined, or could
      two reviewers reasonably disagree on whether a given test fixture qualifies? [Measurability,
      Spec SC-002]

## Requirement Consistency

- [ ] CHK009 - Are the "no behavior change when unconfigured" guarantees in FR-012 and US3
      Acceptance Scenario 3 stated consistently with each other, or could a reader interpret
      them as covering different scope (e.g., one covering casing only, the other covering
      indentation only, without an explicit statement that both must hold simultaneously)?
      [Consistency, Spec FR-012, US3 AS3]
- [ ] CHK010 - Do FR-004's data-reference family list and the Key Entities section's
      "Data-reference token" definition describe the same scope, or does either one imply a
      broader/narrower set than the other? [Consistency, Spec FR-004, Key Entities]

## Scenario & Edge Case Coverage

- [ ] CHK011 - Are requirements defined for the case where a project's configuration sets a
      category to a syntactically valid but semantically unrecognized value (distinct from
      the "unrecognized value" edge case already covering the field-level fallback) — e.g., is
      case-sensitivity of the accepted values (`upper`/`Upper`/`UPPER`) addressed? [Gap,
      Edge Cases]
- [ ] CHK012 - Are non-functional expectations (e.g., formatting remaining fast on large
      scripts) addressed as in-scope or explicitly out-of-scope for this feature, or left
      unaddressed entirely? [Coverage, Non-Functional]
- [ ] CHK013 - Are requirements defined for how the `NUMREC`/`CNT`/`ITER`/`LP`/`RECNUM` removal
      (FR-007) interacts with a project that has, coincidentally, been relying on one of these
      appearing in completion suggestions — or is behavior change here treated as inherently
      out of scope for user-facing compatibility concerns? [Edge Case, Gap, Spec FR-007]

## Dependencies & Assumptions

- [ ] CHK014 - Is the dependency on `ROADMAP.md` items 11/12 remaining out of scope stated
      with enough specificity that a reader could not mistake either as silently included by
      this feature? [Traceability, Spec Assumptions]
- [ ] CHK015 - Is the assumption that "no opinionated preset ships" (Assumptions, FR-003)
      cross-referenced against every success criterion that could otherwise imply a default
      *behavior* beyond Preserve, so the two don't read as contradictory to a reader who only
      sees the Success Criteria section? [Consistency, Spec FR-003, Success Criteria]

## Notes

- Focus areas selected: configuration-precedence clarity and `data_references` category scope
  — the two places this spec's own design conversation (captured in `research.md`) shows the
  most real complexity, and therefore the most risk of an underspecified requirement.
- Depth: Standard, per explicit user input.
- Audience/timing: Reviewer, PR-time — default applied (code-related feature, no explicit
  audience stated).
- No user-specified must-have items beyond "requirements quality review, standard depth" — all
  15 items are self-generated from spec.md content, not consolidated from external instruction.
