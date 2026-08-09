# Requirements Quality Checklist: 2026-08-08 Documentation Verification Pass Amendments

**Purpose**: Validate the *wording quality* (completeness, clarity, consistency,
measurability, edge-case coverage) of the grammar rules amended or added to spec.md
by the 2026-08-08 documentation verification pass — implicit block closing
(`RUN`/`ENDRUN`, `PROCESS`/`PHASE`/`ENDPHASE`), short-`IF`, the narrowed
`MisplacedBreak` condition, generalized shell-escape, nested block comments,
blank-line-skip and brace-delimited continuation, and the newly-tasked FR-028–030/
FR-033 block kinds (`PROCESS`, `JLOOP`, `LINKLOOP`, `DistributeMULTISTEP`). This
checklist tests whether the *requirements as written* are ready for implementation —
it does not test whether any code correctly implements them.

**Created**: 2026-08-08
**Feature**: [spec.md](../spec.md) (see also [data-model.md](../data-model.md),
[contracts/diagnostics.md](../contracts/diagnostics.md))
**Depth**: Standard — pre-implementation review, ahead of a decision on
`/speckit-implement`
**Audience**: Author/reviewer deciding whether to proceed to implementation

## Requirement Completeness

- [ ] CHK001 - Is the nesting depth at which an implicit closer (the next `RUN`/
  `!RUN` statement, or a shell-escape statement) must appear relative to the open
  `RUN` block specified — e.g. does a shell-escape or `RUN` nested one level deeper
  (inside an `IF` within the open `RUN`) count as closing the outer `RUN`, or only
  one at the same nesting depth? [Gap, Spec §FR-009]
- [ ] CHK002 - Is the same same-depth-vs-any-depth question answered for
  `PROCESS`/`PHASE=`'s implicit closer? [Gap, Spec §FR-028]
- [ ] CHK003 - Does the spec state whether `{...}`-delimited statement bodies
  (FR-006) can nest — a `{` appearing before the matching `}` of an already-open
  brace body — the way block comments are explicitly required to nest (FR-005), or
  whether the first `}` always closes it regardless of any `{` in between? [Gap,
  Consistency, Spec §FR-005 vs §FR-006]
- [ ] CHK004 - Does `JLOOP`'s nesting-restriction rule (FR-029: "may nest inside
  `If`/`Loop` … not inside another `JLoop`") state whether it may also nest directly
  inside `Run`/`Process`, or only inside `If`/`Loop`, given every real `JLOOP` is
  necessarily inside some program box already? [Gap, Spec §FR-029]
- [ ] CHK005 - Does `LINKLOOP`'s nesting-restriction rule (FR-033) address whether it
  may nest inside a `JLoop`, or `JLoop` inside a `LinkLoop` — the one block-kind pair
  neither FR-029 nor FR-033 mentions relative to each other? [Gap, Spec §FR-029,
  §FR-033]
- [ ] CHK006 - Is it specified whether a nested `DistributeMULTISTEP …
  EndDistributeMULTISTEP` pair (one opened before the prior one closes) is an
  accepted-but-unobserved shape, or an actual structural defect the parser should
  flag — FR-030 only states nesting hasn't been *observed*, which reads differently
  from "isn't valid"? [Ambiguity, Spec §FR-030]
- [ ] CHK007 - Does any Success Criterion require the fixture corpus to contain at
  least one valid example of each of the four undiagnosed block kinds (`Process`,
  `JLoop`, `LinkLoop`, `DistributeMultistep`) before this phase is considered done —
  the way SC-005 requires it for the `.block`-vs-`.s` question — or does the phase's
  Definition of Done permit closing with these still resting on hand-written,
  not-fixture-confirmed examples? [Gap, Spec §Success Criteria]
- [ ] CHK008 - Does any Success Criterion require fixture-corpus (not just
  hand-written) coverage of the constructs this pass sourced from documentation
  alone — short-`IF`, nested block comments, blank-line-skip continuation,
  `{...}`-delimited continuation, `RUN`/`PROCESS` implicit closing — before the
  "zero false positives" claim (SC-001) is considered fully validated for them?
  [Gap, Spec §SC-001, §Assumptions]

## Requirement Clarity

- [ ] CHK009 - Is "fully blank line" (FR-006's blank-line-skipping rule) defined
  precisely enough to resolve whether a line containing only spaces/tabs (no visible
  characters, but not zero-length) counts as blank, or whether that's a separate,
  unaddressed case? [Ambiguity, Spec §FR-006]
- [ ] CHK010 - Is "no enclosing block of any kind" (FR-026's narrowed
  `MisplacedBreak` condition) explicit that "enclosing" means any ancestor block at
  any depth, not only the immediately-surrounding one — or could a reader infer only
  the nearest enclosing block counts? [Clarity, Spec §FR-026]
- [ ] CHK011 - Is the shell-escape double-star rule (FR-022: "optionally immediately
  followed by a second `*`") explicit about what a third or later consecutive `*`
  means — part of the command text, or an undefined third form? [Edge Case, Gap,
  Spec §FR-022]
- [ ] CHK012 - Does FR-007's short-`IF` wording ("followed immediately, on the same
  physical line, by exactly one further statement") state how it interacts with a
  trailing statement that itself uses line continuation (FR-006) — is a
  continued/multi-physical-line statement still "one statement" satisfying the
  same-line requirement for the part that appears on the `IF`'s own line, or does
  continuation disqualify it from the short form entirely? [Ambiguity, Spec §FR-007
  vs §FR-006]
- [ ] CHK013 - Does FR-007 or FR-006 clarify whether a `{...}`-delimited `Control`
  statement (necessarily spanning to a `}` that may be on a later line) can serve as
  short-`IF`'s "one further statement," given the short-`IF` rule is phrased in
  terms of "the same physical line"? [Ambiguity, Consistency, Spec §FR-007, §FR-006]

## Requirement Consistency

- [ ] CHK014 - Do FR-009 (`RUN`/`ENDRUN` implicit closing) and FR-028
  (`PROCESS`/`PHASE`/`ENDPHASE` implicit closing) apply the same nesting-depth
  reasoning to their respective implicit closers, or could a reader reasonably
  implement them differently since neither FR states the rule explicitly (see
  CHK001/CHK002)? [Consistency, Spec §FR-009, §FR-028]
- [ ] CHK015 - Is the `Run`/`Process` implicit-closing model (data-model.md § Block)
  consistent with the Block entity's own definition in spec.md's Key Entities
  ("a structural grouping formed by opening and closing statements") now that two of
  the seven block kinds don't strictly require a closing statement at all? [Consistency,
  Spec §Key Entities vs data-model.md § Block]

## Edge Case Coverage

- [ ] CHK016 - Does FR-005's block-comment-nesting rule address whether `/*`
  appearing inside a quoted string literal (elsewhere in the grammar, string values
  are quote-delimited) is treated as opening a nested comment, or whether
  quote-awareness is out of scope for comment recognition entirely? [Gap, Spec
  §FR-005]
- [ ] CHK017 - Is there a documented edge case for a short-`IF` whose trailing
  statement is itself a control statement that opens a multi-statement block (e.g.
  another bare `IF`, a `LOOP`, or a `RUN`) — does the short form's single-statement
  rule permit that, and if so what closes the nested block? [Gap, Edge Case, Spec
  §Edge Cases]
- [ ] CHK018 - Is there a documented edge case for a `RUN` (or `PROCESS`/`PHASE`)
  block that is implicitly closed by a following opener of the *other* implicit-
  close family — e.g. does a `PHASE=` statement count as closing an open `RUN` (it
  shouldn't, since they're different `BlockKind`s), and is that non-interaction
  stated anywhere rather than left to be inferred? [Gap, Spec §FR-009, §FR-028]
- [ ] CHK019 - Is there a documented edge case for `!RUN` nested inside another
  block (e.g. inside an `IF`) — does its "always needs an explicit `ENDRUN`" rule
  hold regardless of nesting position, and is a `!RUN` missing `ENDRUN` at end-of-file
  vs. before a sibling statement distinguished anywhere? [Gap, Spec §FR-009]

## Dependencies & Assumptions

- [ ] CHK020 - Does spec.md's Assumptions section make clear, for every construct
  this pass resolved via documentation rather than fixtures (short-`IF`, nested
  comments, blank-line skipping, `{...}` continuation, `RUN`/`PROCESS` implicit
  closing), that SC-001's "zero false positives" claim is currently backed only by
  hand-written fixtures, not the real corpus — and is this consistently flagged
  every place those constructs are introduced, not just once? [Traceability,
  Assumption, Spec §Assumptions]
- [ ] CHK021 - Is the numbering gap between FR-030 and FR-033 (FR-031/FR-032 never
  adopted) explained close enough to FR-033 itself that a reader encountering FR-033
  in isolation (e.g. via search) wouldn't assume a documentation error? [Clarity,
  Spec §FR-033]

## Notes

- This checklist is scoped to the 2026-08-08 amendments only, per the user's stated
  focus — it does not re-litigate constructs the verification-pass report already
  marked NO ACTION (FR-003's control-word list, the diagnostic taxonomy's general
  shape) or DOCUMENT ONLY items already folded into Assumptions (the "proper
  context" continuation qualifier, the `.block` fixture-only caveat).
- Several items above (CHK001–CHK006, CHK009–CHK013) are genuinely open questions
  the current spec wording doesn't resolve either way — they're not evidence of a
  mistake, but each is a concrete ambiguity an implementer would have to guess at
  today. Recommend resolving at least the nesting-depth question for implicit
  closers (CHK001/CHK002/CHK014) before implementation, since it changes
  `block.rs`'s matching algorithm rather than just its edge-case handling.
- No items in this checklist were marked complete `[x]` — this is a fresh review of
  new wording, not a re-confirmation of already-settled requirements.
