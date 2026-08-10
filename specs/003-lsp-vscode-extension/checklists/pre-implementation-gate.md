# Pre-Implementation Gate Checklist: Drut LSP Server & VS Code/Open VSX Extension

**Purpose**: A formal pre-`/speckit-implement` requirements-quality gate — broad
coverage across all six user stories, with deliberately extra weight on the
cross-cutting technical contracts (position encoding, completion scoping) that
required correction twice already during this feature's `/speckit-plan` pass, plus
this feature's dependency on prior-feature (`001`/`002`) data-model contracts it
reuses rather than redefines.

**Created**: 2026-08-09
**Feature**: [spec.md](../spec.md) (also references [plan.md](../plan.md),
[research.md](../research.md), [data-model.md](../data-model.md),
[contracts/](../contracts/), [tasks.md](../tasks.md))

**Note**: This checklist validates the *requirements' own quality* — completeness,
clarity, consistency, measurability, coverage — not whether any code correctly
implements them (no code exists yet). Items are phrased as questions about the
documents; check one off only once the underlying document has been reviewed and
found (or fixed to be) satisfactory.

## Requirement Completeness

- [x] CHK001 **RESOLVED 2026-08-09**, via user-directed research (mirroring
      the UTF-16/FR-019 research depth). Investigated: is there any mechanism
      by which `drut-lsp` obtains non-UTF-8 raw bytes for a live LSP document?
      **Finding: no, and this is architecturally correct, not a gap.**
      `textDocument/didOpen`/`didChange` payloads are JSON strings — LSP's own
      base protocol mandates UTF-8 message content, and JSON strings cannot
      represent invalid byte sequences at all. The editor has already decoded
      the file (correctly, or via its own U+FFFD-substituting fallback) before
      the server ever sees content. Full investigation and decision in
      research.md §12. [Gap, Spec §FR-005, Story 2 AS4]
- [x] CHK002 **RESOLVED 2026-08-09.** FR-014 now states delivery is on-request,
      riding on hover/completion responses, not proactive-while-typing (also
      stated in `contracts/lsp-capabilities.md`, unchanged from original
      design — the gap was spec.md not saying so explicitly). [Gap, Spec Story
      5 AS1, §FR-014]
- [x] CHK003 **RESOLVED 2026-08-09.** New Edge Case: a rename/move is modeled
      by the editor as `didClose`(old URI) + `didOpen`(new URI) — standard LSP
      document-identity behavior, no special server-side handling needed. [Gap]
- [x] CHK004 **RESOLVED 2026-08-09.** FR-026 tightened: "exactly one automatic
      restart attempt" per crash, explicitly not retried further — a second,
      still-failing attempt is a distinct, separately-notified failure per
      FR-025's now-clarified "single, non-repeating" definition (CHK009). [Gap,
      Spec §FR-026]
- [x] CHK005 **RESOLVED 2026-08-09.** FR-026 now states: last-known
      diagnostics/hover/completion/semantic-token results stay visible during
      a restart attempt, refreshed once the server reconnects — not cleared to
      an empty state in the interim. [Gap, Spec §FR-026, Edge Cases]

## Requirement Clarity

- [x] CHK006 **RESOLVED 2026-08-09.** SC-003 now states explicitly: no numeric
      millisecond threshold is specified — deliberately left qualitative, no
      profiling baseline exists yet for this feature (unlike FR-012's
      corpus-evidenced indentation rules), same "no evidence, no invented
      number" treatment applied elsewhere. [Clarity, Spec §SC-003]
- [x] CHK007 **RESOLVED 2026-08-09.** FR-012 tightened: control-word-scoped
      completion is now stated as achieved as the *primary* mode this phase
      (not best-effort); the remaining fallback is narrowed to exactly two
      named cases (no control word yet resolved; a control word the census
      recorded no keywords against) — the hedge was intentional per-case
      fallback, not a general capability gap, and now reads that way. [Ambiguity,
      Spec §FR-012, research.md §2]
- [x] CHK008 **RESOLVED 2026-08-09.** Edge Cases now states explicitly: no
      numeric size ceiling is specified, deliberately qualitative — no corpus
      survey exists establishing a real "largest realistic file" figure the
      way FR-012's indentation width had one. [Clarity, Spec Edge Cases]
- [x] CHK009 **RESOLVED 2026-08-09.** FR-025 now defines "single,
      non-repeating" precisely: at most one notification per distinct failure
      (missing-binary and a later crash-then-restart-fails are distinct
      occurrences, each separately notified once); the same ongoing cause is
      never re-notified. [Clarity, Spec §FR-025, §FR-026]
- [x] CHK010 **RESOLVED 2026-08-09.** FR-007 tightened: "match" now explicitly
      means identical category, message, *and* location (message text was
      already passed through unchanged per data-model.md §3 — this just makes
      FR-007 say so directly rather than leaving "match" ambiguous). [Clarity,
      Spec §FR-007]

## Requirement Consistency

- [x] CHK011 **RESOLVED 2026-08-09.** FR-007 now explicitly states its
      guarantee is "for the same text," not "the same file on disk" — a dirty
      buffer's expected divergence from a `drut check` run against the saved
      file is now explicitly acknowledged as expected, not a violation.
      [Conflict, Spec §FR-007, Edge Cases]
- [x] CHK012 **RESOLVED 2026-08-09.** Story 4 AS2 reworded to state scoping is
      applied every time an enclosing control word resolves, with the fallback
      list narrowed to the specific no-recorded-keywords case — matching
      FR-012's own tightened wording (CHK007). [Consistency, Spec Story 4 AS2,
      §FR-012]
- [x] CHK013 **RESOLVED 2026-08-09**, as a side effect of CHK001's research.
      Since raw bytes are never a reachable input to `drut-lsp` (CHK001),
      `data-model.md` §2's hedge is gone: `OpenDocument.parse_result` is now
      stated unconditionally as `voyager_core::parse(&text)`, never
      `parse_bytes` — and FR-005 itself was corrected to name `parse()`, not
      `parse_bytes()`, resolving the disagreement this item flagged. [Ambiguity,
      Spec §FR-005, data-model.md §2]
- [x] CHK014 **RESOLVED 2026-08-09** — verified, no change needed. spec.md's
      Story 5 dependency prose and tasks.md's "Dependencies & Execution Order"
      (US5 depends on US4 and US3) were already consistent; re-checked
      directly against both documents side by side during this pass. [Consistency,
      Spec Story 5, tasks.md Dependencies & Execution Order]

## Cross-Cutting Technical Contracts (Position Encoding, Completion Scoping, Block-Closer Semantics)

- [x] CHK015 **RESOLVED 2026-08-09** (independently reconfirmed via
      `/speckit-analyze` finding I1, then fixed). Do FR-009, Story 3
      Acceptance Scenario 3, and SC-004's claim that hovering a `Run`/`Process`
      block "reports the resolved implicit closer's location" conflict with
      `Block.closer`'s already-shipped, documented semantics — explicitly
      `None` "when the block closed implicitly (`Run`/`Process`) *or* is
      genuinely unmatched" per `001-voyager-script-parser/data-model.md`'s
      amended `Block` entry — that this feature's own `data-model.md` §4 said
      `BlockHoverFact.counterpart` was derived directly from `Block.closer`?
      **Fix applied**: `data-model.md` §4 now specifies a five-rule
      derivation (`Run` uses `UnmatchedRun`-absence to detect implicit close,
      falling back to `Block.span.end`; `Process` falls back to
      `Block.span.end` unconditionally, since it has no "unmatched"
      diagnostic category to check — see research.md §10 for the full
      rationale, including why a `voyager-core` change was considered and
      rejected as disproportionate). `contracts/lsp-capabilities.md` and
      tasks.md T019 updated to match. [Conflict, Spec §FR-009, Story 3 AS3,
      §SC-004; data-model.md §4; `001-voyager-script-parser/data-model.md`
      § Block.closer]
- [x] CHK016 **RESOLVED 2026-08-09, alongside CHK015.** The fallback rule
      (`Block.span.end` when `closer` is `None` but not genuinely unmatched)
      is now fully specified in `data-model.md` §4's Derivation list —
      distinguishing `Run` (diagnostic-based detection) from `Process` (no
      diagnostic exists, unconditional fallback, documented as a deliberate
      best-effort choice in spec.md Assumptions and research.md §10). [Gap,
      follows from CHK015]
- [x] CHK017 **RESOLVED 2026-08-09.** `data-model.md` §6's derivation now
      states explicitly: only *direct* children of a loop are walked; a
      `BREAK` nested inside a conditional branch is a child of that `IF`
      block, not the loop, so it never triggers the rule — deliberately, to
      avoid exactly the false-positive risk this item raised (constitution
      Principle IV). [Ambiguity, Spec §FR-017, Story 6 AS2/AS3; data-model.md §6]
- [x] CHK018 **RESOLVED 2026-08-09.** FR-019 now includes a direct pointer:
      "This decision is made in `research.md` §1... with the concrete
      translation contract in `contracts/position-encoding.md`." [Traceability,
      Spec §FR-019]
- [x] CHK019 **RESOLVED 2026-08-09.** New spec.md Assumptions bullet: this is
      recorded as "a monitored external dependency, not a one-time finding" —
      revisit research.md §1 if a future `vscode-languageclient` release
      relaxes the restriction. [Assumption, research.md §1]
- [x] CHK020 **RESOLVED 2026-08-09.** FR-012 now states the boundary directly
      in spec.md's own wording (not only research.md): "this scoping is
      strictly by control word, never by a program name... this FR MUST NOT
      be read as reopening that boundary" — already present, and Story 4
      AS2's CHK012 tightening reinforces it. Testable from spec.md alone.
      [Clarity, Spec §FR-012]

## Acceptance Criteria & Success Criteria Quality

- [x] CHK021 **RESOLVED 2026-08-09.** SC-006 now states a concrete,
      protocol-level verification proxy directly: "every such token's hover
      response has its block-kind field populated (FR-008)," independent of
      subjective judgment. [Measurability, Spec §SC-006]
- [x] CHK022 **RESOLVED 2026-08-09.** SC-007 now states its verification
      method directly: "the packaged extension's manifest: its
      `extensionDependencies` field is empty." [Measurability, Spec §SC-007]
- [x] CHK023 **RESOLVED 2026-08-09** — verified, no change needed. FR-028's
      measurability doesn't depend on the corpus's *continued* availability
      any differently than `001`/`002` already do (same external corpus, same
      `DRUT_CORPUS_PATH` gating) — confirmed directly in the new Assumptions
      bullet added for CHK032 (same underlying question, answered once,
      covers both). [Assumption, Spec §FR-028, Assumptions]

## Scenario & Edge Case Coverage

- [x] CHK024 **RESOLVED 2026-08-09.** New Edge Case: not reachable in
      practice, since the server processes one LSP message at a time in
      receipt order (no async runtime, research.md §3) — a `didChange`'s
      re-parse always completes before the next queued request is handled.
      [Gap, Exception Flow]
- [x] CHK025 **RESOLVED 2026-08-09.** New Edge Case: this is the editor's own
      document-identity model (one document per URI) to decide, not something
      the server invents independently. [Gap, Spec Edge Cases]
- [x] CHK026 **RESOLVED 2026-08-09.** New concrete Edge Case added: two open
      documents that structurally reference each other still each derive
      their own diagnostics/hover/completion purely from their own content —
      the server never reads a second document to answer a request about the
      first. [Coverage, Spec Assumptions "Out of scope"]
- [x] CHK027 **RESOLVED 2026-08-09.** New Edge Case: with zero matching-language
      files, the extension's own activation trigger means it simply never
      activates — no "activated but idle" state exists to define. [Gap, cf.
      `002-cli-check-format/spec.md` Edge Cases]
- [x] CHK028 **RESOLVED 2026-08-09, alongside CHK001.** Story 2 AS4 removed
      (was untestable as written, per CHK001's finding) — replaced with a new
      Edge Case stating the `InvalidEncoding`-through-live-editing boundary
      explicitly, and FR-005/FR-028/SC-002/SC-008 all updated with matching
      carve-outs. [Gap, follows from CHK001; Spec Story 2 AS4]

## Non-Functional Requirements

- [x] CHK029 **RESOLVED 2026-08-09.** New Edge Case states cross-platform
      binary-resolution parity as an explicit requirement (Windows `.exe`/
      `PATH` conventions vs. macOS/Linux), not only an assumption from plan.md's
      Target Platform note. [Gap, plan.md Target Platform]
- [x] CHK030 **RESOLVED 2026-08-09, alongside CHK008** (same underlying
      question — document size). Edge Cases now states explicitly: no numeric
      ceiling, deliberately qualitative. [Clarity, Spec Edge Cases]
- [x] CHK031 **RESOLVED 2026-08-09.** Edge Cases now states explicitly:
      memory/scale for many open documents is left unbounded by design this
      phase, revisit only if real usage shows it matters. [Gap, Spec Key
      Entities]

## Dependencies & Assumptions

- [x] CHK032 **RESOLVED 2026-08-09.** New spec.md Assumptions bullet:
      exercising the corpus through the LSP protocol layer introduces no new
      corpus-availability or environment dependency beyond what `001`/`002`
      already require — same corpus, same gating, different in-process test
      harness only. [Assumption, Spec Assumptions, plan.md Scale/Scope]
- [x] CHK033 **RESOLVED 2026-08-09.** New spec.md Assumptions bullet: this is
      recorded as "a standing item to periodically re-check," not a closed
      one-time observation. [Assumption, research.md §3, §11]

## Traceability

- [x] CHK034 **RESOLVED 2026-08-09.** Audited every table in `data-model.md`;
      added missing FR citations to `ServerState.documents`, `OpenDocument.text`,
      `OpenDocument.version`, `BlockHoverFact.kind`, `SpellCheckHint.token_span`,
      and `SpellCheckHint.suggestion` — all fields now cite the FR they back.
      [Traceability, data-model.md §5]
- [x] CHK035 **RESOLVED 2026-08-09, alongside CHK015.** Where
      `contracts/lsp-capabilities.md` restated a data-model.md derivation rule
      rather than referencing it, was there a risk of independent drift?
      **Fix applied**: `contracts/lsp-capabilities.md`'s hover section now
      explicitly cross-references data-model.md §4's Derivation list rather
      than restating it, with a one-line note explaining why (avoid drift).
      [Consistency, contracts/lsp-capabilities.md, data-model.md §4]

## Notes

- **All 35 items resolved as of 2026-08-09.** This checklist ran through two
  passes: the first (immediately after generation) resolved the single
  highest-value finding — CHK015/CHK016/CHK035, a direct, evidenced conflict
  between this feature's own hover requirements (FR-009, Story 3 AS3, SC-004)
  and the already-shipped, cross-feature `Block.closer` contract
  (`001-voyager-script-parser`, amended by `002-cli-check-format`),
  independently reconfirmed via `/speckit-analyze` finding I1. The second
  pass (at the `/speckit-implement` gate) resolved the remaining 32 items:
  most were wording/cross-reference corrections (spec.md prose that predated
  a research.md resolution, or documented facts missing an explicit pointer);
  five were deliberate "left qualitative, no invented number" defaults
  (perceptibly-immediate/document-size/memory-scale thresholds), consistent
  with this project's established evidence-or-explicit-default convention;
  and one — CHK001/CHK028 — was a genuine architectural finding requiring
  real investigation (resolved via user-directed research, `research.md`
  §12): `InvalidEncoding` cannot be reported through live document editing
  by construction of the LSP transport itself (JSON payloads cannot carry
  invalid byte sequences; the editor has already decoded the file before the
  server ever sees content), so `InvalidEncoding` was scoped out of FR-005/
  Story 2/FR-028/SC-002/SC-008 as CLI-only, rather than left as an
  unsatisfiable requirement.
- Items marked `[Gap]` described genuinely missing requirements, not
  necessarily defects — most were resolved by adding an explicit Edge Case
  or Assumption rather than new acceptance-scenario text, since the correct
  behavior in each case was a reasonable, low-risk default (e.g. LSP's own
  URI-keyed document identity, standard rename-as-close+open semantics)
  rather than a genuinely open product decision.
- Traceability coverage: 35/35 items (100%) carry at least one bracketed
  reference tag, exceeding the ≥80% minimum.
- Every item is checked off with a one-line note recording what was done or
  concluded — no item was closed silently.
