# Feature Specification: Top-Level Indentation Normalization

**Feature Branch**: `008-top-level-indentation-normalization`

**Created**: 2026-08-11

**Status**: Draft

**Input**: User description: "Amend FR-012 (002-cli-check-format/spec.md): top-level (depth-0) statement indentation now always normalizes to column 0 on every format pass — a deliberate reversal of FR-012's original corpus-evidence-based decision, made knowingly, trading real authors' non-uniform existing top-level styles for consistency/predictability. Confirm whether 007-formatter-diagnosed-block-indent-fix's residue-prevention skip is still needed under the new policy, including whether the PROCESS/RUN residue sequence from this session's own debugging now fully resolves in the second format pass with zero manual intervention. Scope includes regenerating format_corpus.rs's golden fixtures with the same T023b-style human-reviewed-diff discipline. Out of scope: the 'known limitations' README documentation about the old residue-preservation behavior — superseded by this change, not written."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Every top-level statement lands at column 0, always (Priority: P1)

A script author formats any Cube Voyager script — regardless of how it was
originally written, edited, or previously (mis)formatted — and every
top-level (depth-0) statement or block opener is flush at column 0. This
replaces the prior behavior, where a top-level line's existing indentation
(whatever it happened to be) was always left alone.

**Why this priority**: This is the entire policy change — a single,
unconditional rule replacing a previously conditional one. Every other
part of this feature (the residue-scenario resolution, golden-fixture
regeneration) is downstream of this one behavioral change.

**Independent Test**: Format a script with a top-level `RUN`/`PROCESS`/
bare-statement line sitting at some non-zero column (however it got
there), and confirm the result has that line at column 0, with its
children still correctly indented 4 spaces per nesting level relative to
that corrected position.

**Acceptance Scenarios**:

1. **Given** a script where a top-level statement sits at a non-zero
   column, **When** the script is formatted, **Then** that statement's
   line is corrected to column 0.
2. **Given** a script where a top-level block's own opener line has been
   corrected to column 0 but its children still carry indentation relative
   to the block's *old* (non-zero) position, **When** the script is
   formatted, **Then** the children are corrected to align relative to the
   new, column-0 base — not left at their old, now-inconsistent values.
3. **Given** a script where every top-level line is already at column 0,
   **When** the script is formatted, **Then** nothing changes (idempotence
   holds, same as every other formatting rule).

---

### User Story 2 - The known residue scenario fully self-resolves (Priority: P2)

A script author who previously hit the `PROCESS`/`RUN` residue sequence
(an unclosed `PROCESS` swallows a trailing `RUN` as its child; the author
adds the missing `ENDPROCESS`) now sees the `RUN` block correctly land at
column 0 automatically on the very next format pass — no manual
indentation cleanup needed, and no dependency on `007`'s prior
never-touch-a-top-level-line behavior to avoid writing bad indentation in
the first place, since any bad indentation left behind is now
unconditionally corrected regardless of source.

**Why this priority**: Directly resolves the concrete, real bug this whole
session's debugging thread was chasing — the most tangible, already-proven
pain point this policy change fixes, distinct from the general
predictability argument behind User Story 1.

**Independent Test**: Reproduce the exact sequence (unclosed `PROCESS`
swallows a trailing `RUN`, format once, add `ENDPROCESS`, format again)
and confirm `RUN` lands correctly at column 0 after the *second* format
pass alone, with no additional manual edit.

**Acceptance Scenarios**:

1. **Given** a `PROCESS PHASE=...` left unclosed with a trailing `RUN`
   block swallowed as its child, **When** the file is formatted once, then
   `ENDPROCESS` is added, then the file is formatted a second time,
   **Then** `RUN` is correctly positioned at column 0 after that second
   pass alone.

---

### Edge Cases

- What happens to a top-level block that is *still* genuinely unmatched
  (never gets its closer added at all)? Its own opener line is still
  forced to column 0 under the new unconditional rule (User Story 1) —
  but `007`'s existing skip of its *children's* indentation-planning may
  or may not still apply, creating a possible asymmetry (opener corrected,
  children left alone) that needs an explicit resolution during planning,
  not an assumption (see FR-004).
- What happens to existing tests/fixtures whose "already correctly
  formatted, no change expected" baseline relied on a non-zero top-level
  indentation being left alone? These need review individually — this
  isn't limited to the committed golden-fixture corpus (FR-006); any
  hand-written unit test elsewhere in the codebase asserting a
  non-zero-top-level "no change" outcome needs the same scrutiny.
- What happens to the explicit-closer-alignment rule (a closer aligns to
  its own opener, delta 0) once the opener itself is always at column 0?
  No special handling needed — that rule already composes correctly with
  any base value, including 0.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST normalize every top-level (depth-0)
  statement's or block opener's leading indentation to column 0 on every
  format pass, unconditionally — regardless of its current indentation,
  formatting history, or block kind.
- **FR-002**: This normalization MUST apply uniformly to every top-level
  node — bare statements and every block-kind opener (`IF`, `LOOP`, `RUN`,
  `PROCESS`/`PHASE`, `JLoop`, `LinkLoop`, `DistributeMultistep`) — with no
  per-kind exceptions.
- **FR-003**: Nested (non-top-level) indentation MUST continue to use the
  existing per-nesting-level rule (4 spaces per level, FR-012's unchanged
  sub-rule) — now always anchored to the corrected, column-0 top-level
  base rather than to a possibly-untouched one.
- **FR-004**: `007-formatter-diagnosed-block-indent-fix`'s
  skip-indentation-planning-for-a-diagnosed-block's-children behavior MUST
  be explicitly re-evaluated under the new policy — determined during
  planning whether it remains necessary, becomes redundant, or needs
  adjustment — and that determination MUST be recorded, not left as
  unexamined, potentially-vestigial logic.
- **FR-005**: The `PROCESS`/`RUN` residue sequence (unclosed `PROCESS` →
  format → add `ENDPROCESS` → format again) MUST fully resolve — `RUN`
  correctly at column 0 — within that second format pass alone, with no
  further manual or automated intervention needed.
- **FR-006**: Every currently-committed golden fixture affected by this
  policy change (`format_corpus.rs`'s `real_corpus/` set and any
  hand-written `valid/` fixture with non-zero top-level indentation) MUST
  be regenerated and individually diff-reviewed before being committed as
  new expected output — each diff confirmed to change *only* top-level
  indentation, with nothing else moved, reordered, or altered — mirroring
  the original `T023b` human-in-the-loop discipline those golden files
  were first created under.
- **FR-007**: `002-cli-check-format/spec.md`'s FR-012 text and Assumptions
  section MUST be amended to reflect the new policy — the original
  corpus-survey evidence (only 20.4% of real top-level statements at
  column 0, modal value at column 8) reframed as historical context for a
  knowingly-overridden decision, not left reading as an active
  justification for behavior that no longer holds.
- **FR-008**: No "known limitations" documentation describing the old
  residue-preservation behavior MUST be added — explicitly out of scope,
  superseded by this change eliminating the scenario it would have
  described.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user formatting any Cube Voyager script sees every
  top-level statement flush at column 0, with zero exceptions, regardless
  of the file's original or prior-edit indentation.
- **SC-002**: The `PROCESS`/`RUN` residue scenario is fully corrected
  within one additional format pass after the missing closer is added —
  zero manual cleanup required.
- **SC-003**: Every real-corpus golden fixture's regenerated output has
  been individually reviewed and confirmed to change *only* top-level
  indentation — zero unintended content, structure, or diagnostic changes.
- **SC-004**: The full 161-file real corpus remains 100% clean (zero
  diagnostics of any kind) after the change — a purely whitespace-shifting
  change, not a structural or diagnostic one.

## Assumptions

- This remains a whitespace-only change — no structural or program-meaning
  change (constitution Principle III's formatter idempotence/
  behavior-preservation guarantee is unaffected in kind, though it must be
  re-verified against the new rule, not assumed to still hold
  unconditionally).
- No adapter (`drut-cli`, `drut-lsp`, `drut-mcp`) requires its own code
  change — none of them hardcode or reference the top-level-exemption
  policy; every adapter's formatting call path is a generic pass-through
  to `voyager_core::format` (confirmed directly during this session's
  `006`/`007` investigations, not assumed here).
- Golden-fixture regeneration and its human-reviewed-diff discipline
  (FR-006) is part of this feature's own Definition of Done, not a
  follow-up task — matching the constitution's existing "every formatter
  change requires a golden-file diff against the fixture corpus before
  merge" gate (Principle III, Development Workflow & Quality Gates).
- This decision is a deliberate policy reversal, not new evidence
  contradicting the original corpus survey — the original 26.9%-at-
  column-8/20.4%-at-column-0 findings remain accurate as a historical
  record of what real authors did; the project has simply decided
  predictability now outweighs preserving that diversity.
