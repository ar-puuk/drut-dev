# Requirements Quality Checklist: Voyager Script Tokenizer & Structural Parser

**Purpose**: Comprehensive self-review of spec.md + plan.md requirement quality
(completeness, clarity, consistency, measurability, coverage) before running
`/speckit-tasks`. Standard depth; author-facing.
**Created**: 2026-08-08
**Feature**: [spec.md](../spec.md) | [plan.md](../plan.md)

**Note**: This checklist tests whether the *requirements are written well* — not
whether the parser works. No item here should be resolved by writing or running code;
each is resolved by editing spec.md, plan.md, or a Phase 1 design doc.

## Requirement Completeness

- [ ] CHK001 - Is there an explicit functional requirement (not just an Assumptions-
  section note) that a `BREAK` outside any `LOOP` must produce a diagnostic, with the
  same binding force as FR-012–FR-016? [Completeness, Gap, Spec §Assumptions]
- [ ] CHK002 - Do the Success Criteria include a measurable outcome for the three
  statement forms added during fixture research (label, shell-escape, assignment) —
  i.e. is corpus coverage of FR-021–FR-023 specified anywhere the way SC-003 specifies
  coverage of the diagnostic categories? [Completeness, Gap, Spec §SC-003]
  — **Deliberately deferred as of 2026-08-08**: SC-003's stale "five" count was
  fixed, but this item's actual substance is fixture-corpus coverage, not spec
  wording — deferred until the real fixture corpus is gathered (see research.md §3);
  no new SC added for it now.
- [ ] CHK003 - Are ordering/priority requirements defined for the case where a single
  defect could plausibly be reported under more than one diagnostic category (e.g. an
  unclosed block comment that also swallows what would otherwise be a bad
  continuation)? [Completeness, Gap]
- [ ] CHK004 - Are requirements defined for non-UTF-8 or otherwise unusually-encoded
  input (real Windows-authored `.s`/`.block` fixtures were observed with stray control
  characters), or is "input is always valid UTF-8 text" an explicit, stated
  assumption rather than a silent one? [Completeness, Gap, Edge Case]
- [ ] CHK005 - Is a bound defined on diagnostic volume for a single defect (e.g. one
  unmatched `IF` should not cascade into dozens of downstream diagnostics), or is
  "continue past a recorded defect" (FR-018) left open-ended? [Completeness, Gap,
  Spec §FR-018]

## Requirement Clarity

- [ ] CHK006 - Is "where feasible" in FR-018 ("the parser MUST continue past a
  recorded defect... where feasible") given any objective criteria, or does it leave
  the continuation guarantee unfalsifiable as written? [Clarity, Ambiguity, Spec §FR-018]
- [ ] CHK007 - Does FR-011's case-insensitivity requirement distinguish between
  matching a keyword *name* (e.g. `PGM`) case-insensitively and matching a keyword's
  *value* (e.g. `PGM=MATRIX` vs `PGM=matrix`) — or is it ambiguous whether values are
  also intended to be case-folded? [Clarity, Ambiguity, Spec §FR-011]
- [ ] CHK008 - Is the boundary between a `Control` statement (FR-003) and an
  `Assignment` statement (FR-023) precisely defined — e.g. is there a fixed/closed set
  of recognized control words, or is the distinguishing rule purely "one keyword=value
  pair with no separate leading word," and if so is that rule stated anywhere?
  [Clarity, Ambiguity, Spec §FR-003, §FR-023]
  — **Partially resolved 2026-08-08**: FR-023 now states the disambiguation rule
  ("`Assignment` whenever the first token is not a recognized control word"). Still
  open: spec.md has no fixed/closed control-word list for FR-003 to point to. A raw
  corpus census (161 real files) was done and reported to the user, but the list was
  explicitly not finalized into spec.md this pass — remains a follow-up.
- [ ] CHK009 - Does FR-009 specify whether `PGM=` is a mandatory part of recognizing a
  `RUN` block opener, or would a bare `RUN` (no `PGM=`) also need to open a block per
  the requirements as written? [Clarity, Ambiguity, Spec §FR-009]
- [ ] CHK010 - Is "independently reviewable as original wording" (SC-006) given any
  concrete review mechanism or measurable check, or is it currently a qualitative
  aspiration with no defined pass/fail procedure? [Clarity, Measurability, Spec §SC-006]

## Requirement Consistency

- [x] CHK011 - Does SC-003 ("each of the five required diagnostic categories...") still
  match the diagnostic category count after `MisplacedBreak` was added in the
  Assumptions section, or is this now a stale/inconsistent "five" that should say
  "six" (or be reworded to not hard-code a count)? [Consistency, Conflict, Spec §SC-003]
  — **Resolved 2026-08-08**: SC-003 no longer hardcodes a count; it now references
  FR-012–FR-016 and FR-026 by ID and explicitly names all six categories.
- [x] CHK012 - Does FR-025 ("correctly flag every deliberately-broken fixture") list
  or reference the same category set as SC-003, so the two requirements can't drift
  out of sync again the next time a category is added? [Consistency, Spec §FR-025]
  — **Resolved 2026-08-08**: FR-025 now explicitly says "covering every diagnostic
  category defined in this specification (FR-012–FR-016 and FR-026)," the same
  reference SC-003 uses.
- [x] CHK013 - Is plan.md's zero-runtime-dependency constraint consistent with, or
  strictly narrower than, spec.md's FR-001 (which only forbids I/O/network/protocol
  dependencies, not dependencies in general) — and if narrower, is that a plan-level
  implementation choice or does it belong back in spec.md as a requirement?
  [Consistency, Spec §FR-001, Plan §Constraints]
  — **Resolved 2026-08-08**: Added FR-027 ("MUST NOT introduce third-party runtime
  dependencies"), elevating the constraint plan.md's research.md §1 had already
  committed to into a real spec-level requirement. FR-001 itself was left unchanged —
  it already covered "no I/O/network/protocol dependency" before this pass; FR-027
  covers the separate, stricter "no third-party crates at all" constraint that CHK013
  was actually about.
- [ ] CHK014 - Does FR-010's blanket "tokenize `@variable@` as its own token type"
  requirement clearly apply (or clearly not apply) inside a shell-escape statement's
  parenthesized contents, given FR-022 says those contents are stored opaquely and not
  parsed as Voyager grammar — or do the two requirements leave this case
  underspecified? [Consistency, Ambiguity, Spec §FR-010, §FR-022]
- [x] CHK015 - Does FR-006's continuation rule apply uniformly to every statement form
  (`Control`, `Assignment`, `Label`, `ShellEscape`), or is it, as written, only clearly
  scoped to control statements — leaving label/shell-escape/assignment continuation
  behavior unstated? [Consistency, Gap, Spec §FR-006, §FR-021–FR-023]
  — **Resolved 2026-08-08**: FR-006 now states the rule applies uniformly to all four
  statement forms, confirmed against real fixtures for `Control` and `Assignment`
  (multi-line arithmetic sums joined by trailing `+`). `Label`/`ShellEscape`
  continuation has no confirmed real-world example either way — this is now stated
  explicitly in spec.md (Assumptions) rather than left silently unstated, which is
  what this item was actually asking for.
- [ ] CHK016 - Does plan.md's performance goal (sub-100ms parse of multi-thousand-line
  scripts) correspond to any measurable outcome in spec.md's Success Criteria, or does
  a non-functional expectation exist in the plan with no matching, testable SC?
  [Consistency, Gap, Plan §Technical Context, Spec §Success Criteria]

## Acceptance Criteria Quality

- [ ] CHK017 - Can SC-001 ("zero reported diagnostics" on every valid fixture) be
  objectively verified without first resolving the open fixture-corpus
  sourcing/licensing question (research.md §3) — i.e. does the spec's Definition of
  Done implicitly depend on an unresolved dependency? [Measurability, Dependency,
  Spec §SC-001]
- [ ] CHK018 - Is SC-004 ("no file access or protocol dependency is required to get a
  result") independently verifiable by a concrete check (e.g. a dependency/import
  audit), or does it read as an architectural intent rather than a testable success
  criterion? [Measurability, Spec §SC-004]
- [ ] CHK019 - Are the acceptance scenarios in User Story 2 sufficient to demonstrate
  FR-018's "continues reporting on the rest of the script" guarantee for a fixture with
  *multiple, independent* defects, or do all listed scenarios cover only a single
  defect per fixture? [Acceptance Criteria, Gap, Spec §User Story 2]
- [ ] CHK020 - Does SC-005 ("at least one fixture of each observed `.block` shape")
  specify who selects/confirms those two fixtures are representative, given the
  broader fixture corpus itself is still an open dependency? [Measurability,
  Dependency, Spec §SC-005]

## Scenario Coverage

- [ ] CHK021 - Are requirements defined for a script that mixes a bare-fragment
  `.block` shape and a fully `RUN`-wrapped shape at different points *within the same
  file* (not just across separate fixtures), given FR-020 allows zero-or-more top-level
  blocks? [Scenario Coverage, Gap, Spec §FR-020]
- [ ] CHK022 - Are requirements defined for a label statement or shell-escape
  statement appearing *inside* an open `IF`/`LOOP`/`RUN` block (not only at top level,
  where the current Edge Cases place them)? [Scenario Coverage, Gap, Spec §Edge Cases]
- [ ] CHK023 - Is there a requirement covering what happens when the *entire* input is
  a single unterminated block comment (no other content before or after it) — does
  this reduce cleanly to FR-014, or does the empty-surrounding-content case need its
  own acceptance scenario? [Scenario Coverage, Edge Case, Spec §FR-014]

## Edge Case Coverage

- [ ] CHK024 - Is the interaction between a dangling closer (e.g. an `ENDIF` with no
  open `IF`) and a *later, otherwise-valid* `IF`/`ENDIF` pair in the same file
  addressed — does the dangling closer's diagnostic prevent correct matching of the
  later, legitimate pair? [Edge Case, Gap, Spec §Edge Cases]
- [ ] CHK025 - Are requirements defined for a `RUN PGM=...` block nested inside another
  open `RUN PGM=.../ENDRUN` block — is it explicitly a non-goal (structural nesting
  allowed, no semantic "already open" check) in spec.md itself, or only in plan.md's
  research notes, leaving the spec silent on a case a reader of spec.md alone would
  reasonably wonder about? [Edge Case, Consistency, Spec §Edge Cases, Plan
  §research.md-referenced-decision]
- [ ] CHK026 - Is behavior specified for a continuation character that is itself
  inside a `@variable@` reference's delimiters (e.g. a line ending in `@partial` with
  the closing `@` on the next line) — does FR-006's continuation rule and FR-010's
  `@variable@` tokenization rule agree on which one governs? [Edge Case, Ambiguity,
  Spec §FR-006, §FR-010]

## Non-Functional Requirements

- [ ] CHK027 - Is a maximum acceptable parse latency stated as a testable requirement
  anywhere a reader of spec.md (not just plan.md) would find it, given downstream LSP
  use is explicitly the motivating rationale for the performance goal? [Non-Functional,
  Gap, Plan §Technical Context]
- [ ] CHK028 - Is "MUST NOT panic on malformed input" — stated in plan.md's
  Constraints — also stated as a functional/non-functional requirement in spec.md, or
  does spec.md rely only on FR-012–FR-016's "structured diagnostic — not a crash"
  phrasing to imply it? [Non-Functional, Consistency, Spec §FR-012, Plan §Constraints]

## Dependencies & Assumptions

- [ ] CHK029 - Does spec.md or plan.md assign an owner and a phase-gate deadline for
  resolving the fixture-corpus sourcing/licensing question, or is it currently an
  open-ended research note with no forcing function before implementation begins?
  [Dependency, Gap, Spec §Assumptions, Plan §Constitution Check]
- [ ] CHK030 - Is the assumption that "diagnostics are structured data... rather than
  opaque free-text strings" validated against how CLI/LSP/MCP adapters (not yet built)
  are actually expected to consume them, or is it asserted without a corresponding
  adapter-side requirement to confirm it holds up? [Assumption, Spec §Assumptions]

## Ambiguities & Conflicts

- [ ] CHK031 - Is there any requirement resolving whether `.s`/`.block` grammar
  version detection (baseline Voyager 6.5, per FR-024) is a per-file, per-project, or
  per-parse-call concept — i.e. once a version flag exists in a later phase, does this
  phase's grammar already anticipate where that flag would plug in, or is that left
  fully undefined? [Ambiguity, Spec §FR-024, Assumptions]

## Notes

- CHK011 and CHK012 shared one root cause: `MisplacedBreak` was added after the
  original five-category framing was written, and SC-003/FR-025 weren't updated to
  match. Both are now resolved by promoting `BREAK`-outside-`LOOP` to a full
  requirement (FR-026) and having SC-003/FR-025 reference the FR list instead of a
  hardcoded count. See each item above for detail.
- CHK013 is resolved: FR-027 now states the "no third-party runtime dependencies"
  constraint at the spec level. Note this is distinct from FR-001, which already
  stated the narrower "no file I/O/network/protocol dependency" constraint before this
  pass — CHK013 was about the stricter, previously plan-only constraint, not FR-001.
- CHK002 remains open — it asks for statement-form (label/shell-escape/assignment)
  corpus-coverage measurability, which is a different gap than the count drift and
  wasn't addressed by this pass.
- CHK016, CHK027, CHK028 still concern requirements (performance target, no-panic
  guarantee) the user has confirmed should stay plan-only for now, absent a concrete
  performance number to treat as a hard acceptance criterion — no further action
  needed on those unless that changes.
- **2026-08-08 corpus investigation (U1/U2/U3)**: CHK015 resolved and CHK008 partially
  resolved (see each item above) by inspecting 161 real `.s`/`.block` files in
  `WF-TDM-Official-Releases`. U3 (`@variable@` in quoted strings) and U2 (continuation
  scope) are now reflected in FR-010/FR-006 and new Assumptions bullets. U1's
  follow-up — a raw control-word census — was reported to the user but not finalized
  into FR-003.
- **New, not-yet-tracked findings from that census** (reported to the user, no
  checklist item yet): three block-pair types not covered by FR-007–FR-009
  (`PHASE`/`ENDPHASE`, `JLOOP`/`ENDJLOOP`, `DistributeMULTISTEP`/
  `EndDistributeMULTISTEP`); a hybrid `WORD=value keyword=value...` statement shape
  (e.g. `COMBINE=EQUI ENHANCE=2,...`) fitting neither FR-003 nor FR-023 cleanly; and a
  brace-delimited `FUNCTION { ... }` block using `{`/`}` rather than paired control
  words, unaddressed by any block-matching rule. These are real gaps in FR-007–FR-009's
  block-matching scope, not just FR-003's word list — worth a dedicated pass.
