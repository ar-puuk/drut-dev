# Feature Specification: UnmatchedProcess Diagnostic

**Feature Branch**: `006-unmatched-process-diagnostic`

**Created**: 2026-08-11

**Status**: Draft

**Input**: User description: "Add DiagnosticKind::UnmatchedProcess to voyager-core, mirroring UnmatchedRun's exact firing condition — a PROCESS/PHASE= block with no matching ENDPROCESS/ENDPHASE and no following PROCESS/PHASE= statement before either end-of-input or the enclosing block's own closer forces an early stop. Motivated by a real corpus investigation (161-file WF-TDM-Official-Releases) finding 123 real PROCESS blocks, all 123 explicitly closed — zero false-positive risk, proven empirically. Regression test must include the real-world scenario that surfaced this gap (a PROCESS with no closer, followed by real subsequent content), not just a minimal synthetic case. Full corpus revalidation required. JLoop/LinkLoop/DistributeMultistep explicitly out of scope, flagged for separate future investigation."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Getting warned about a PROCESS block that never closes (Priority: P1)

A script author (or a tool acting on their behalf — `drut check`, the editor's
live diagnostics, or an MCP-connected AI agent's `diagnose` call) has a Cube
Voyager script containing a `PROCESS`/`PHASE=` block that was never actually
closed — no `ENDPROCESS`/`ENDPHASE`, and no following `PROCESS`/`PHASE=`
statement to implicitly close it either. Today this is completely silent:
every statement remaining in the file (or in the block's enclosing scope)
gets silently absorbed as that `PROCESS` block's own nested content, with no
signal anything is wrong — the same class of mistake `RUN` already warns
about, but `PROCESS` doesn't. After this feature, the author sees a clear
diagnostic pointing at the unclosed `PROCESS`/`PHASE=` statement.

**Why this priority**: This is the entire feature — a single, well-scoped
diagnostic addition with no smaller independently-valuable increment
beneath it.

**Independent Test**: Parse a script containing a `PROCESS PHASE=...` with no
`ENDPROCESS`/`ENDPHASE` and no following `PROCESS`/`PHASE=` statement before
the end of the file, and confirm exactly one diagnostic is reported,
pointing at the `PROCESS`/`PHASE=` statement itself.

**Acceptance Scenarios**:

1. **Given** a script with a `PROCESS PHASE=name` block, real content
   after it, and no `ENDPROCESS`/`ENDPHASE`/following `PROCESS`/`PHASE=`
   before end-of-file, **When** the script is parsed, **Then** exactly one
   diagnostic is reported, pointing at the `PROCESS PHASE=name` statement,
   describing that it has no matching closer and no following
   `PROCESS`/`PHASE=` statement.
2. **Given** a script with a `PROCESS`/`PHASE=` block explicitly closed by
   `ENDPROCESS`/`ENDPHASE`, **When** the script is parsed, **Then** no
   diagnostic is reported for that block.
3. **Given** a script with two consecutive `PROCESS`/`PHASE=` blocks where
   the first has no explicit closer but is immediately followed by the
   second (the legitimate implicit-close pattern), **When** the script is
   parsed, **Then** no diagnostic is reported for the first block — this
   remains silent, exactly as it is today.
4. **Given** a script where a `PROCESS`/`PHASE=` block is opened inside
   another block (e.g. an `IF`) and that enclosing block's own closer
   (`ENDIF`) appears before the `PROCESS` gets any closer of its own,
   **When** the script is parsed, **Then** the diagnostic is reported —
   the enclosing block's closer forcing an early stop is treated the same
   as reaching true end-of-file, mirroring how `UnmatchedRun` already
   handles this exact nested case for `RUN`.
5. **Given** the full real-world 161-file reference corpus (all of whose
   `PROCESS`/`PHASE=` blocks are explicitly closed, confirmed by this
   feature's own motivating investigation), **When** every file is parsed,
   **Then** zero new diagnostics appear anywhere in the corpus.

---

### Edge Cases

- What happens to the existing `Block`/`closer` structural representation
  (already used to distinguish an implicit close from a genuine mismatch,
  per `voyager-core`'s own design) once this diagnostic exists? Nothing —
  this feature adds an *additional* signal alongside that existing
  representation; it does not change how `Block.closer`/`BlockKind::Process`
  are computed or represented.
- Does `PROCESS` have a disabled/skip variant analogous to `RUN`'s `!RUN`
  (which gets no implicit-closer exception and is diagnosed on a missing
  `ENDRUN` alone)? To be confirmed during planning against the actual
  grammar rather than assumed — if no such variant exists for `PROCESS`,
  this edge case doesn't apply and that absence should be stated explicitly,
  not silently skipped.
- What happens for the three block kinds this feature deliberately leaves
  untouched (`JLoop`, `LinkLoop`, `DistributeMultistep`)? Nothing — their
  existing silent, no-diagnostic behavior is completely unchanged. This is
  an explicit scope boundary (see Assumptions), not an oversight.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST add a new diagnostic category,
  `DiagnosticKind::UnmatchedProcess`, alongside the six existing categories,
  following the same construction (`kind`/`span`/`message`) and quality bar
  every existing category already meets.
- **FR-002**: The system MUST report `UnmatchedProcess` exactly when a
  `PROCESS`/`PHASE=` block reaches either end-of-input or its enclosing
  block's own closer, with neither an explicit `ENDPROCESS`/`ENDPHASE` nor a
  following `PROCESS`/`PHASE=` statement (the legitimate implicit-close
  pattern) having appeared first — mirroring `UnmatchedRun`'s existing
  firing condition for `RUN` exactly, including the nested-early-stop case
  (Acceptance Scenario 4).
- **FR-003**: The system MUST NOT report `UnmatchedProcess` when a
  `PROCESS`/`PHASE=` block closes explicitly.
- **FR-004**: The system MUST NOT report `UnmatchedProcess` when a
  `PROCESS`/`PHASE=` block closes implicitly via a following sibling
  `PROCESS`/`PHASE=` statement — this remains a legitimate, silent
  structural pattern, unchanged by this feature.
- **FR-005**: The diagnostic's `span` MUST point at the `PROCESS`/`PHASE=`
  opener statement itself, mirroring `UnmatchedRun`'s existing span
  convention (the opener, not a hypothetical missing-closer location).
- **FR-006**: The diagnostic's `message` MUST be original wording
  (constitution Principle II — no verbatim vendor-documentation text),
  describing the specific defect.
- **FR-007**: Every adapter that surfaces `voyager_core::parse`/
  `parse_bytes` diagnostics (`drut-cli`'s `check` — both text and SARIF
  output — `drut-lsp`'s live diagnostics, `drut-mcp`'s `diagnose` tool)
  MUST correctly report the new category. **Corrected 2026-08-11, during
  planning, per this feature's own explicit "confirm, don't assume"
  instruction**: the original draft of this requirement assumed zero
  adapter-specific code changes would be needed. Verified false —
  `drut-cli/src/report/sarif.rs` (`ALL_KINDS`, `rule_id`, `short_description`
  — three exhaustive `match`/array sites), `drut-lsp/src/diagnostics.rs`
  (`kind_name`), and `drut-mcp/src/diagnose.rs` (`category_name`) each
  maintain an **exhaustive** `match` over `DiagnosticKind` with no wildcard
  arm — confirmed by reading each file directly, not inferred. Adding
  `UnmatchedProcess` without updating all three is a compile error, not a
  silent gap. `drut-cli`'s plain-text output (`report/text.rs`) is the one
  surface that needs no change — it Debug-formats `diag.kind` directly.
  This remains a voyager-core-anchored feature (Principle I: the *decision*
  of when to fire lives entirely in `voyager-core`) — the adapter changes
  are the same category of thin, non-decision-making naming/rendering work
  each adapter already does for the other six kinds, not new grammar logic.
- **FR-008**: The full 161-file real reference corpus MUST remain 100%
  clean (zero diagnostics of any kind, including `UnmatchedProcess`) after
  this change — re-verified as part of this feature's own Definition of
  Done, not assumed to still hold from the motivating investigation alone.
- **FR-009**: The committed fixture corpus MUST include a deliberately-broken
  fixture reproducing the real-world shape that motivated this feature — a
  `PROCESS` block with no closer, followed by real subsequent content that
  would otherwise be silently absorbed as the block's own children — not
  merely a minimal single-statement synthetic case.
- **FR-010**: `JLoop`, `LinkLoop`, and `DistributeMultistep` MUST NOT gain a
  diagnostic category as part of this feature. The existing documentation
  of this deferral (`001-voyager-script-parser/contracts/diagnostics.md`)
  MUST be updated to reflect that `Process` has now been resolved while
  these three remain deferred, rather than left describing all four as
  equally undecided.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A script author (through any of `drut check`, an editor's live
  diagnostics, or an MCP-connected agent) sees a clear diagnostic the moment
  a `PROCESS`/`PHASE=` block never closes — the same experience already
  available today for an unclosed `RUN` block.
- **SC-002**: Every script that already parses cleanly today under the
  legitimate implicit-close or explicit-close patterns continues to parse
  with zero new diagnostics — verified against all 161 files of the real
  reference corpus, not a sample.
- **SC-003**: The new diagnostic category is indistinguishable in
  documentation quality, message clarity, and test coverage from the six
  categories that shipped in `001-voyager-script-parser`.

## Assumptions

- The *decision* logic (when `UnmatchedProcess` fires) is entirely a
  `voyager-core` change (constitution Principle I) — no grammar/parsing
  logic is added anywhere else. **The claim that literally zero adapter
  code needed to change was checked and found false during planning
  (FR-007's correction)**: three adapters (`drut-cli`, `drut-lsp`,
  `drut-mcp`) maintain exhaustive `DiagnosticKind` matches that must each
  gain one new arm — mechanical, non-decision-making rendering work, not a
  Principle I violation, but real work nonetheless, not "free." `editors/
  vscode` needs no change — it never lists diagnostic kinds itself, only
  renders whatever `drut-lsp` already sends.
- `JLoop`, `LinkLoop`, and `DistributeMultistep` are explicitly out of
  scope. They were not part of the real-corpus investigation this feature
  is based on, and adding diagnostics for them is deliberately left as a
  separate, future, independently-evidenced decision — not silently
  bundled in and not silently ruled out.
- The real WF-TDM-Official-Releases corpus itself is not committed to this
  repository (licensing still an open item, `001-voyager-script-parser/
  research.md` §3); FR-009's regression fixture is a hand-written
  reproduction of the real shape, not a copy of real corpus content,
  consistent with how every other hand-written fixture in this repo is
  already sourced.
- No changes to `Block`'s existing structural representation
  (`closer: Option<Span>`, `BlockKind::Process`) are needed or in scope —
  this feature adds a diagnostic signal derived from the same parse-time
  information already being computed, not a new data shape.
