# Specification Quality Checklist: Blank-Line-Run Normalization

**Purpose**: Requirements quality review, standard depth — focused on the two areas of this
spec most exposed to ambiguity: the top-level/nested classification boundary, and the
survivor-selection/protection interaction.
**Created**: 2026-08-17
**Feature**: [spec.md](../spec.md)
**Depth**: Standard | **Audience**: Reviewer (PR-time), default per no explicit audience stated

## Requirement Completeness

- [X] CHK001 - Is the exact valid range for each cap (a sane upper bound) stated in the spec, or
      explicitly deferred to planning in a way a reader wouldn't mistake for an oversight?
      [Completeness, Spec Assumptions] — **Resolved, no change needed**: Assumptions already
      states this explicitly.
- [X] CHK002 - Are requirements defined for what happens to an excessive blank-line run that
      spans from just before a block's opener into the block itself (i.e. a run whose lines are
      partially top-level and partially nested)? Or is the "a run can never straddle a boundary"
      claim (Edge Cases) itself the requirement, leaving no partial-straddle case to define?
      [Completeness, Spec Edge Cases] — **Fixed**: added an explicit Edge Case stating a run
      immediately before an opener is never part of that block (the opener line is never blank,
      so it always breaks the run) — governed by whatever encloses it instead.
- [X] CHK003 - Does the spec define behavior for a run that spans the entire file (a file that is
      only blank lines, or a file starting/ending with an excessive run and containing no other
      content)? [Gap, Edge Cases] — **Fixed**: added the degenerate all-blank-file case to the
      existing file-boundary Edge Case, explicit that the top-level cap applies.

## Requirement Clarity

- [X] CHK004 - Is "any line inside any block's own body" (FR-002, FR-008) precise enough that a
      reader could determine, without consulting the implementation, whether a blank line
      between an `IF`'s own opener line and its first child statement counts as nested? [Clarity,
      Spec FR-002, FR-008] — **Fixed**: added an explicit clause to the top-level/nested Edge
      Case confirming a run immediately after an opener (before the first child) is nested.
- [X] CHK005 - Is "surviving lines... left byte-for-byte as originally written" (FR-006) clear
      about what happens to a whitespace-only survivor specifically, or could a reader
      reasonably expect it to be trimmed to zero-length instead? [Clarity, Spec FR-006, Edge
      Cases] — **Resolved, no change needed**: already explicit in the existing survivor Edge
      Case.

## Requirement Consistency

- [X] CHK006 - Are FR-003's "only when the run's length exceeds the cap" and FR-004's "never pads
      a shorter run up" stated in a way that a reader recognizes these as two restatements of the
      same one-directional guarantee, not two independent, potentially-conflicting rules? [
      Consistency, Spec FR-003, FR-004] — **Resolved, no change needed**: intentional
      complementary framing (positive + negative restatement), not redundant — each forecloses a
      different misreading.
- [X] CHK007 - Do FR-002's "default `2`"/"default `1`" and the Key Entities section's own
      restatement of the same defaults agree exactly, with no drift in which number applies to
      which cap? [Consistency, Spec FR-002, Key Entities] — **Resolved, no change needed**:
      confirmed matching on inspection.

## Scenario & Edge Case Coverage

- [X] CHK008 - Are requirements defined for a run that sits entirely within a `; FMT: OFF`
      region versus one that starts before the region and would, absent the marker, have
      continued into it — or does the "a marker line is non-blank, so it always breaks a run"
      framing (Edge Cases) already foreclose the latter case? [Coverage, Spec Edge Cases] —
      **Fixed**: the FMT:OFF Edge Case now explicitly states the same "marker lines are never
      blank, so a run can never be partially protected" reasoning already used for block
      boundaries.
- [X] CHK009 - Are non-functional expectations (e.g., formatting remaining fast on a file with
      many long blank-line runs) addressed as in-scope or explicitly out-of-scope, or left
      unaddressed entirely? [Coverage, Non-Functional] — **Resolved, no change needed**:
      consistent with `017`/`018`'s own precedent, performance is a plan.md-level concern (see
      its Performance Goals), not spec-level.
- [X] CHK010 - Is the distinction between this feature (blank-line *count* only) and blank-line
      *placement* (inserting a blank line where none exists) stated clearly enough that a reader
      wouldn't expect this feature to also normalize, say, "always one blank line before an
      `IF`"? [Clarity, Spec Assumptions] — **Resolved, no change needed**: already explicit in
      Assumptions.

## Dependencies & Assumptions

- [X] CHK011 - Is the dependency on this feature reusing `top_level_indent`'s own top-level-
      vs-nested framing made explicit enough that a reader understands why there are two caps
      instead of one, without needing `ROADMAP.md` item 13's own history? [Traceability, Spec
      Input] — **Resolved, no change needed**: already explicit in the Input section.
- [X] CHK012 - Is the assumption that no opinionated preset ships (Assumptions) cross-referenced
      against every acceptance scenario that could otherwise imply a specific "house style"
      default beyond the stated `2`/`1` defaults? [Consistency, Assumptions] — **Resolved, no
      change needed**: already explicit in Assumptions.
- [X] CHK013 - Is the exact configuration surface shape (field/flag/param names) explicitly
      deferred to planning in a way a reader wouldn't mistake for already being fixed by this
      spec? [Clarity, Assumptions] — **Resolved, no change needed**: already explicit in
      Assumptions.

## Notes

- Focus areas selected: top-level/nested classification precision and survivor-selection/
  protection interaction — the two places this spec's own design conversation shows the most
  real complexity for a feature this otherwise-simple, and therefore the most risk of an
  underspecified requirement.
- Depth: Standard, audience Reviewer/PR-time — defaults applied (code-related feature, no
  explicit audience stated for this checklist run).
- No user-specified must-have items beyond the standard-depth requirements-quality review — all
  13 items are self-generated from spec.md content, not consolidated from external instruction.
