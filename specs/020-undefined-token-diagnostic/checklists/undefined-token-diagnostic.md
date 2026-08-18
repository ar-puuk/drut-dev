# Specification Quality Checklist: Undefined `@token@` Diagnostic

**Purpose**: Requirements quality review, standard depth — focused on the two areas of this
spec most exposed to ambiguity: whether the "never flag a resolver blind spot" guarantee is
actually closed (exhaustive) or could have an unenumerated fourth case, and whether the
severity/message-wording decisions are internally consistent with each other.
**Created**: 2026-08-17
**Feature**: [spec.md](../spec.md)
**Depth**: Standard | **Audience**: Reviewer (PR-time), default per no explicit audience stated

## Requirement Completeness

- [X] CHK001 - Is the list of resolver "blind spots" (FR-003) stated as closed/exhaustive, or
      could a reader reasonably wonder whether a fourth, unenumerated gap exists in the
      underlying resolver that this spec doesn't account for? [Completeness, Spec FR-003] —
      **Resolved, no change needed**: research.md §3 checked each of the three exclusions
      directly against the actual resolver code (not assumed), and plan.md's Constitution Check
      table re-confirms this at Phase 1. The list is exhaustive *of what the reused functions
      currently do* — if `token_resolution.rs`'s own resolution reach ever changes, this
      feature's exclusions change with it automatically (they're inherited, not hardcoded), so
      there's no separate list to go stale.
- [X] CHK002 - Are requirements defined for what happens when a document has zero `@token@`
      references at all? [Completeness, Edge Cases] — **Fixed**: added as an explicit Edge Case
      (empty result, no notice published, same as any document with nothing to flag).

## Requirement Clarity

- [X] CHK003 - Is "resolvable definition" (FR-001) defined precisely enough that a reader could
      determine, without consulting the implementation, exactly which two mechanisms count
      (same-file assignment; one level of static `READ FILE` inclusion) and which don't? [Clarity,
      Spec FR-001] — **Resolved, no change needed**: FR-001 states both mechanisms explicitly
      and FR-003 states the three exclusions explicitly — no reader is left inferring the
      boundary.
- [X] CHK004 - Is the relationship between Hint and Information severity (FR-002 allows either)
      precise enough, or should the spec commit to exactly one? [Clarity, Spec FR-002] — **Fixed**:
      data-model.md §3 commits to `DiagnosticSeverity::HINT` specifically as the concrete
      implementation choice; spec.md's "Hint or Information" wording is intentionally left as an
      either-is-acceptable outcome at the requirements level (both convey the same "low
      confidence, not asserted as fact" signal to a user), with the plan/data-model layer making
      the binding concrete choice — same layering every other spec in this project uses between
      "what" (spec) and "exactly how" (plan/data-model).

## Requirement Consistency

- [X] CHK005 - Does the hedged diagnostic message wording (data-model.md §3, "may still be
      defined elsewhere Drut can't see") read as consistent with FR-002's Hint/Information
      severity choice, or could a stronger-sounding message undercut the softer severity?
      [Consistency, data-model.md §3, Spec FR-002] — **Resolved, no change needed**: the message
      explicitly avoids asserting non-existence, matching the severity choice's own reasoning
      (Assumptions) word-for-word in spirit.
- [X] CHK006 - Is FR-005 ("LSP-only") stated consistently with FR-004 ("not a `DiagnosticKind`
      variant"), or could a reader think the two are independent, unrelated constraints rather
      than two consequences of the same underlying design shape? [Consistency, Spec FR-004,
      FR-005] — **Resolved, no change needed**: research.md §1 and the contract doc both frame
      these as the same shape (a non-`DiagnosticKind`, LSP-only stream) — not presented as
      coincidentally-adjacent independent facts.

## Scenario & Edge Case Coverage

- [X] CHK007 - Are requirements defined for a `@token@` reference that appears more than once in
      the document, all unresolvable (e.g. the same typo referenced three times)? [Coverage,
      Edge Cases] — **Resolved, no change needed**: `all_variable_refs` returns every occurrence
      independently (data-model.md §1) — each gets its own notice at its own span, no
      deduplication assumed or needed; this falls out of the "every reference, independently
      checked" design without a special case.
- [X] CHK008 - Are non-functional expectations (e.g., publish latency on a document with many
      `@token@` references) addressed as in-scope or explicitly out-of-scope? [Coverage,
      Non-Functional] — **Resolved, no change needed**: plan.md's Performance Goals covers this
      (linear in reference count, one shared disk-I/O pass, no new full-file re-scan).

## Dependencies & Assumptions

- [X] CHK009 - Is the decision to exclude plain-assignment identifiers and data-reference tokens
      from scope justified with enough specificity that a reader wouldn't mistake the omission
      for an oversight? [Traceability, Spec Assumptions] — **Resolved, no change needed**:
      Assumptions states both exclusions with the specific reason each lacks existing resolution
      logic (or, for data-reference tokens, has a structurally different one).
- [X] CHK010 - Is the "no configuration surface" decision (FR-008) cross-referenced against the
      two existing precedents it's modeled on, so a reader can verify the claim rather than take
      it on faith? [Traceability, Spec FR-008, Assumptions] — **Resolved, no change needed**:
      both spec.md's Assumptions and research.md §5 name the two specific existing streams
      (unclosed `; FMT: OFF` marker, malformed `drut.toml` warning) and confirm neither has a
      toggle either.

## Notes

- Focus areas selected: blind-spot-list exhaustiveness and severity/message-wording consistency
  — the two places most exposed to either an incomplete requirement or an internally
  inconsistent one, given this feature's unusually direct dependency on constitution Principle
  IV holding in practice, not just in stated intent.
- Depth: Standard, audience Reviewer/PR-time — defaults applied, no explicit audience stated for
  this checklist run.
- Two real fixes applied during this review (CHK002's empty-document edge case; CHK004's
  Hint-vs-Information layering clarification) — the rest were already adequately addressed and
  are recorded as resolved with reasoning, not left as bare checkmarks.
