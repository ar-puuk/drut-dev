# Specification Quality Checklist: Editor-Settings Exposure for `[format]` Config Fields

**Purpose**: Requirements quality review, standard depth — focused on the two areas of this
spec most exposed to ambiguity: precedence-chain completeness across all 10 fields, and the
graceful-degradation guarantees for a client that doesn't (fully) support the mechanism.
**Created**: 2026-08-17
**Feature**: [spec.md](../spec.md)
**Depth**: Standard | **Audience**: Reviewer (PR-time), default per no explicit audience stated

## Requirement Completeness

- [X] CHK001 - Does FR-003's precedence statement explicitly cover all 10 fields, or could a
      reader wonder whether the two legacy-vs-granular casing fields (`casing` vs.
      `control_words_casing`/`pair_keywords_casing`) interact with a client setting the same
      special way they already interact with `drut.toml`'s own legacy-vs-granular arbitration?
      [Completeness, Spec FR-003] — **Real gap found and fixed**: confirmed directly against
      `drut-config/src/lib.rs` that `control_words`/`pair_keywords` each already get a two-step
      legacy-then-granular fallback at both existing tiers; the client-settings tier needed the
      identical two-step treatment, not the one-step shape every other field gets. Fixed in
      research.md §1, data-model.md §1, contracts, and tasks.md T002/T006.
- [X] CHK002 - Are requirements defined for what happens when the *same* field is set by both a
      legacy client setting (e.g. `drut.format.casing`) and a granular one (e.g.
      `drut.format.controlWordsCasing`) at the client-settings tier specifically, or is this left
      entirely to be inferred from `drut.toml`'s own precedent? [Gap, Spec FR-003] — **Fixed**,
      same correction as CHK001 — granular wins over legacy within the client-settings tier too,
      explicitly stated now rather than left implicit.
- [X] CHK003 - Is there a requirement covering what happens if the client sends a
      `workspace/configuration` response with the requested section entirely absent (as opposed
      to present-but-empty), distinct from a malformed field within it? [Gap, Edge Cases] —
      **Fixed**: added an Edge Case — treated identically to every field being individually
      absent, no special case needed.

## Requirement Clarity

- [X] CHK004 - Is "reflected on the next format request" (FR-006) precise about whether an
      in-flight or already-queued format request at the moment of the settings change is expected
      to use the old or new value, or is that level of race-condition precision explicitly out of
      scope? [Clarity, Spec FR-006] — **Resolved, no change needed**: LSP requests are handled
      one at a time in this server's existing single-threaded main loop (research.md §2) — there
      is no in-flight/queued-request race to specify, by construction of the architecture this
      feature builds on, not a gap in the requirement itself.
- [X] CHK005 - Is "gracefully" in the client-non-support Edge Case defined with enough precision
      (no request ever sent vs. a request sent but its absent response silently ignored) that a
      reader could verify which behavior is intended without consulting the implementation?
      [Clarity, Edge Cases] — **Fixed**: reworded the Edge Case to state explicitly "no request
      is ever sent... not sent-but-ignored."

## Requirement Consistency

- [X] CHK006 - Are FR-004 (graceful degradation for a non-supporting client) and FR-002
      (mandatory use of the standard mechanism) stated consistently, i.e. does a reader
      understand FR-004 as a required fallback *of* FR-002's mechanism rather than a competing,
      alternative mechanism? [Consistency, Spec FR-002, FR-004] — **Resolved, no change needed**:
      FR-004 explicitly says "the connected client does not support `workspace/configuration`,"
      directly naming FR-002's own mechanism, not a separate one.
- [X] CHK007 - Do SC-004 ("next format request... no editor restart") and FR-006 ("next format
      request... no document close/reopen") describe the same guarantee in compatible terms, or
      could a reader read SC-004 as a stronger/different claim than FR-006's? [Consistency, Spec
      FR-006, SC-004] — **Resolved, no change needed**: both state the same "already-open
      document, no reopen/restart needed" guarantee from two adjacent angles (requirement vs.
      measurable outcome), not two different claims.

## Scenario & Edge Case Coverage

- [X] CHK008 - Are requirements defined for a workspace with multiple open documents when a
      client setting changes — does every open document's next format request reflect the new
      value, or only the one currently being edited? [Coverage, Gap] — **Fixed**: added an Edge
      Case — the cache is session-wide, not per-document, so every open document reflects the
      new value on its own next format request, with no separate handling needed.
- [X] CHK009 - Are non-functional expectations (e.g., the pull round trip not delaying server
      startup or the first format request noticeably) addressed as in-scope or explicitly
      out-of-scope? [Coverage, Non-Functional] — **Resolved, no change needed**: plan.md's
      Performance Goals already covers this (one round trip per pull, not per format request;
      every format request itself reads an already-cached value with no new per-request latency).

## Dependencies & Assumptions

- [X] CHK010 - Is the decision not to scope client settings per workspace folder (single global
      pull) justified with enough specificity that a reader wouldn't mistake the omission for an
      oversight, especially given `drut.toml` itself *is* scope-aware? [Traceability, Spec
      Assumptions] — **Resolved, no change needed**: research.md §5 gives the specific reasoning
      (client settings are deliberately the personal, single-global-fallback layer, distinct from
      `drut.toml`'s already-existing scope-aware role) — not an unexplained omission.
- [X] CHK011 - Is the assumption that no `drut-cli`/`drut-mcp` behavior changes (FR-007)
      cross-referenced against every success criterion that could otherwise imply CLI/MCP
      awareness of client settings? [Consistency, Spec FR-007, Success Criteria] — **Resolved, no
      change needed**: every SC-001–SC-006 is scoped to "a document"/"a client setting"/VS
      Code's Settings UI specifically, none imply CLI/MCP involvement, and FR-007/Assumptions
      state the exclusion directly.

## Notes

- Focus areas selected: precedence-chain completeness (especially the legacy/granular casing
  interaction, a real source of complexity in `017`'s own precedence matrix that this feature
  could easily under-specify by analogy) and graceful-degradation precision — the two places
  this spec's own scope is most exposed to an incomplete or ambiguous requirement.
- Depth: Standard, audience Reviewer/PR-time — defaults applied, no explicit audience stated for
  this checklist run.
