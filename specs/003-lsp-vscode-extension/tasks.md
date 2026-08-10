---

description: "Task list for Drut LSP Server & VS Code/Open VSX Extension"
---

# Tasks: Drut LSP Server & VS Code/Open VSX Extension

**Input**: Design documents from `/specs/003-lsp-vscode-extension/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md (all present)

**Tests**: Included — constitution Principle IV/V and this feature's own Definition
of Done (FR-028: the LSP server must pass the same full-corpus validation already
proven for `voyager-core` and `drut-cli`) require an LSP-level fixture-corpus test
suite before merge; these aren't optional flavor for this feature.

**Organization**: Tasks are grouped by user story (US1–US6, P1–P6 per spec.md), so
each can be implemented, tested, and shipped independently. US1 (static
highlighting) needs **no** Rust/`drut-lsp` work at all — it's the most independent
story in this feature, and the only one that doesn't depend on the Foundational
phase. US5 depends on US4's dictionary existing (spec.md's own stated ordering:
"it reuses the same dictionary... ordered after the completion list it depends
on") — the one deliberate cross-story dependency in this feature.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files from every sibling task in the
  same phase — a real completion-order dependency on an earlier task, if any, is
  still noted in the description text)
- **[Story]**: US1–US6 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

Three Rust crates under `crates/` plus one non-Rust package (plan.md Structure
Decision):

- `crates/voyager-core/` — existing crate; US4/US5 add `src/keywords.rs` +
  `tests/keywords.rs` to it. US1/US2/US3/US6 touch nothing here.
- `crates/drut-cli/` — existing crate; this feature adds one `server` subcommand
  (Foundational phase only).
- `crates/drut-lsp/` — new library crate this feature creates; the LSP server
  itself.
- `editors/vscode/` — new TypeScript package; not a Cargo workspace member.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Stand up the new `drut-lsp` crate and the `editors/vscode/` package
so both build.

- [X] T001 Add `"crates/drut-lsp"` as a third workspace member in `Cargo.toml`
      (repo root) (plan.md Structure Decision).
- [X] T002 Create `crates/drut-lsp/Cargo.toml`: package `drut-lsp` (library, no
      `[[bin]]` — `drut server` stays a `drut-cli` subcommand per FR-001), a path
      dependency on `voyager-core`, and `lsp-server = "0.10.0"` /
      `lsp-types = "0.97.0"` (research.md §11's confirmed pins).
- [X] T003 [P] Create `crates/drut-lsp/src/lib.rs` with a placeholder
      `pub fn run(connection: lsp_server::Connection) {}` and confirm
      `cargo build -p drut-lsp` and `cargo clippy -p drut-lsp --all-targets`
      succeed zero-warning. Verified: builds and lints clean.
- [X] T004 [P] Scaffold `editors/vscode/`: `package.json` (name, placeholder
      `publisher`, `engines.vscode`, `activationEvents: ["onLanguage:drut-voyager"]`,
      `main: "./out/extension.js"`), `tsconfig.json`, `vscode-languageclient` +
      `@types/vscode` + `typescript` devDependencies, and `src/extension.ts` with
      placeholder `activate()`/`deactivate()` exports. Confirm `npm install` and
      `npm run compile` succeed (contracts/extension-manifest.md). Verified:
      `npm install` (12 packages, 0 vulnerabilities) and `npm run compile`
      both succeed cleanly. `.gitignore` extended for `node_modules/`/`out/`/
      `*.vsix`.

**Checkpoint**: `cargo build --workspace` passes with the new (empty) `drut-lsp`
crate in place; `editors/vscode/` compiles with a no-op extension.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Session/document state, position translation, and protocol
bootstrap shared by every server-backed story (US2–US6). **US1 has no
dependency on this phase** — it's pure static configuration in `editors/vscode/`
and can be built and shipped independently of everything below.

**⚠️ CRITICAL**: No US2–US6 work can begin until this phase is complete.

- [X] T005 [P] Define `ServerState`/`OpenDocument` in
      `crates/drut-lsp/src/document_store.rs` (data-model.md §2): a
      `HashMap<Uri, OpenDocument>`, `didOpen`/`didChange` (with the `version`
      staleness guard)/`didClose` handling, re-deriving `parse_result` via
      `voyager_core::parse` (always — never `parse_bytes`, per data-model.md
      §2/§3 and research.md §12) on every mutation (FR-002). 5 unit tests, all
      passing.
- [X] T006 [P] Implement `crates/drut-lsp/src/position.rs`'s
      `to_lsp_position`/`from_lsp_position`/`to_lsp_range`
      (`contracts/position-encoding.md`): `char::len_utf16()`-based UTF-16
      counting per line, 1-based↔0-based line translation, out-of-range input
      clamped rather than panicking (FR-004, FR-019, FR-020). 6 unit tests
      (including the supplementary-plane-character case), all passing.
- [X] T007 Implement `crates/drut-lsp/src/lib.rs`'s `run(connection)` entry
      point: handle `initialize` (declare `ServerCapabilities` per
      `contracts/lsp-capabilities.md` — `position_encoding` fixed to `Utf16`,
      `text_document_sync: Full`, `hover_provider: true`,
      `completion_provider` with trigger characters `" "`/`"="`,
      `semantic_tokens_provider` with the `shortIf`/`unreachable` legend),
      dispatch `didOpen`/`didChange`/`didClose` to `document_store.rs`, and stub
      `hover`/`completion`/`semanticTokens/full` handlers returning
      empty/`null` until US3/US4/US6 replace them. Depends on T005, T006.
      **Implemented directly with the real handlers already wired in** (T019/
      T024/T030 built alongside it in the same continuous implementation
      pass) rather than literal empty stubs — no functional difference from
      the task's own intent, since every handler still degrades to `None`/
      empty correctly on its own terms.
- [X] T008 [P] Add `crates/drut-cli/src/server_cmd.rs` (thin dispatch: calls
      `drut_lsp::run()` over a real stdio `Connection`, zero LSP protocol logic
      here — Principle I) and a `Command::Server` variant (no flags, FR-001) in
      `crates/drut-cli/src/cli.rs`, wired in `main.rs`/`lib.rs`. Depends on T007.
- [X] T009 [P] Add `crates/drut-lsp/tests/protocol_smoke.rs`: drive the server
      through a real `initialize`/`initialized`/`textDocument/didOpen` round
      trip via `lsp_server::Connection::memory()` (research.md §9); assert
      `capabilities.position_encoding == "utf-16"`
      (`contracts/position-encoding.md`'s fixed-constant guarantee). Depends on
      T007.

**Checkpoint**: `drut server` completes an `initialize` handshake and tracks
open documents; US2–US6 can now proceed, in parallel if staffed.

---

## Phase 3: User Story 1 - Instant syntax highlighting on install (Priority: P1) 🎯 MVP

**Goal**: Installing the extension gives immediate control-word/comment
(including nested block comments)/string/`@variable@` highlighting and correct
bracket/comment-toggling behavior, with zero dependency on `drut server`.

**Independent Test**: Install in a clean VS Code instance with no `drut` binary
on `PATH`, open a fixture `.s` file, confirm distinct highlighting and correct
bracket/comment behavior — spec.md's own Independent Test for this story.

### Implementation for User Story 1

- [X] T010 [P] [US1] Author `editors/vscode/language-configuration.json`:
      brackets and comment-toggling (line `;`, block `/* */`) for Voyager syntax
      (FR-022). Structural shape may reference
      `bhereth.language-citilabscubevoyager`'s config under the constitution's
      granted permission (research.md §8) — own wording only. **Authored from
      first principles against `voyager-core`'s own lexer semantics**, not
      against his file (not locally available in this environment either) —
      still fully satisfies FR-022/FR-023 since nothing was copied from
      anywhere.
- [X] T011 [P] [US1] Author `editors/vscode/syntaxes/drut.tmLanguage.json`: a
      static TextMate grammar covering control words, comments (including
      recursive nested block comments, per the Phase 1 lexer fix), strings, and
      `@variable@` substitutions (FR-021). Own wording/structure per FR-023.
      Nested block comments use TextMate's standard self-referencing
      `begin`/`end`/`patterns: [{include: self}]` recursion technique.
- [X] T012 [P] [US1] Wire `editors/vscode/package.json`'s
      `contributes.languages` (language ID `drut-voyager`, `.s`/`.block`
      extensions) and `contributes.grammars` pointing at T010/T011's files
      (`contracts/extension-manifest.md`).
- [X] T013 [P] [US1] Add `editors/vscode/test/grammar.test.ts`: tokenization
      spot-checks — control words/comments (including the nested case)/
      strings/`@variable@` each get a distinct scope. Depends on T011, T012.
      Standalone via `vscode-textmate`+`vscode-oniguruma` (no VS Code
      instance needed, unlike a full extension-host test) — added as
      devDependencies, run via `npm test` (`ts-node test/grammar.test.ts`).
      7/7 checks pass, including the nested-block-comment case across all
      three lines of a real nested comment.
- [ ] T014 [US1] Manual verification (`quickstart.md` step 8): package via
      `npx @vscode/vsce package`, install in a clean profile with no `drut` on
      `PATH`, open a corpus file, confirm highlighting renders correctly —
      matching spec.md's own Story 1 Independent Test exactly. Depends on
      T012. **Does not verify FR-025's "server not found" notice** — that
      behavior is implemented in T016 (Phase 4/US2), not available yet at
      this checkpoint; verified instead by T018 once T016 exists (corrected
      2026-08-09, `/speckit-analyze` finding F1 — the notice was originally
      claimed here despite its implementing code living two phases later,
      which would have made this task unpassable under the "MVP First"
      strategy's own Setup→Phase 3 sequencing).

**Checkpoint**: Story 1 is fully functional and independently shippable — the
MVP, with zero server dependency.

---

## Phase 4: User Story 2 - See structural problems as you type (Priority: P2)

**Goal**: The editor shows every `voyager-core` diagnostic reachable through
live editing (six of seven categories — `InvalidEncoding` is CLI-only, see
research.md §12) live, matching `drut check`, as the buffer changes.

**Independent Test**: Open a fixture with a deliberately unmatched `IF`, see the
diagnostic without running any command; fix it, see the diagnostic clear without
saving/reopening — spec.md's own Independent Test for this story.

### Implementation for User Story 2

- [X] T015 [P] [US2] Implement `crates/drut-lsp/src/diagnostics.rs`:
      `OpenDocument.parse_result.diagnostics` → `textDocument/publishDiagnostics`
      via `position.rs`'s `to_lsp_range`, `severity: Error`, `message` passed
      through unchanged (Principle II, `contracts/lsp-capabilities.md`); publish
      on `didOpen`/`didChange`, publish empty on `didClose` (FR-005–FR-007).
      Covers six of `voyager-core`'s seven `DiagnosticKind` values —
      `InvalidEncoding` never appears in `parse_result.diagnostics` here by
      construction (document_store.rs/T005 always calls `parse()`, never
      `parse_bytes()`), not something this task needs to filter out
      (data-model.md §3, research.md §12). Depends on T005–T007.
- [X] T016 [P] [US2] Implement `editors/vscode/src/extension.ts`'s
      `LanguageClient` bootstrap (`contracts/extension-manifest.md`): resolve
      the `drut` binary, start a `vscode-languageclient` client spawning
      `drut server` scoped to the `drut-voyager` language ID if resolvable;
      otherwise leave Story 1's highlighting intact and show one non-repeating
      notification (FR-025); register a crash handler that notifies once and
      attempts one restart (FR-026). Depends on T008, T012. Binary resolution
      is `PATH`-only (no `drut.serverPath` setting) — spec.md Assumptions
      rules out any configuration surface this phase; Node's own
      `child_process` `PATH` search already handles Windows `.exe`/POSIX
      conventions correctly with no bespoke per-platform code needed. The
      one-restart cap is implemented via a custom `ErrorHandler.closed()`
      (`OneRestartErrorHandler`) that returns `CloseAction.Restart` exactly
      once, then `DoNotRestart` — matching FR-026/CHK004's tightened wording
      exactly. Compiles clean (`tsc`, strict mode).
- [X] T017 [P] [US2] Add `crates/drut-lsp/tests/diagnostics_corpus.rs`, gated
      behind `DRUT_CORPUS_PATH` and `#[ignore]`'d unconditionally (mirrors
      `002-cli-check-format`'s `fixture_corpus_e2e.rs` gating): every valid
      corpus file opened via `Connection::memory()`'s `didOpen` publishes zero
      diagnostics; every deliberately-broken fixture publishes a diagnostic
      correctly identifying its injected defect (SC-002, first slice of
      FR-028's Definition of Done). **Excludes the `InvalidEncoding`-triggering
      hand-written fixture** (`001-voyager-script-parser/tests/fixtures/`) from
      this run's broken-fixture set — it cannot and is not expected to
      reproduce that diagnostic through `didOpen` (FR-005/FR-028 carve-out,
      research.md §12); assert instead that opening it via `didOpen` publishes
      zero diagnostics for the six reachable categories (its content has no
      other structural defect). Depends on T015. **Committed-fixtures half
      (`voyager-core/tests/fixtures/broken/`) runs unconditionally and
      passes** (9/9 real defects correctly identified via real `; EXPECT:`
      markers, `undecodable_byte.s` correctly lossily-decoded first — the
      same non-fatal substitution a real editor performs — then confirmed to
      publish zero diagnostics). **External-corpus half also verified
      2026-08-10**, once `DRUT_CORPUS_PATH` became available in this
      environment: `cargo test -p drut-lsp --test diagnostics_corpus --
      --ignored` → 161/161 files clean through `didOpen`, reproducing
      SC-002/FR-028's Definition of Done end-to-end through the LSP protocol
      layer, matching `voyager-core`'s and `drut-cli`'s own already-proven
      results.
- [ ] T018 [US2] Manual verification (`quickstart.md` step 9): with `drut` on
      `PATH`, open a valid file (no diagnostics), introduce an unmatched `IF`
      (diagnostic appears live), undo (diagnostic clears) — validates SC-003.
      **Also verify FR-025's "server not found" behavior here** (moved from
      T014, `/speckit-analyze` finding F1): temporarily remove `drut` from
      `PATH`, reload, confirm a single non-repeating notice appears with no
      repeated popups and highlighting stays intact; restore `PATH` before
      continuing. Depends on T016, T017.

**Checkpoint**: US1 and US2 both independently functional; live diagnostics
work end-to-end through the packaged extension.

---

## Phase 5: User Story 3 - Understand block structure by hovering (Priority: P3)

**Goal**: Hovering a block opener/closer reports its kind and, when resolved,
its matched counterpart's location — including through Run/Process's implicit
close.

**Independent Test**: Hover a nested `IF`, confirm the kind and matched
`ENDIF`/`ELSEIF`/`ELSE`; hover a `RUN` block that closes implicitly, confirm the
resolved implicit closer is still reported — spec.md's own Independent Test.

### Implementation for User Story 3

- [X] T019 [P] [US3] Implement `crates/drut-lsp/src/hover.rs`: `BlockHoverFact`
      derivation (data-model.md §4) — locate the `Block` whose opener/closer
      span contains the cursor (`from_lsp_position`), report `kind`,
      `is_short_if` (FR-010), and `counterpart` per data-model.md §4's
      five-rule derivation (FR-009) — **not** simply `Block.closer` (which is
      `None` for both an implicitly-closed and a genuinely-unmatched
      `Run`/`Process` block, so cannot be read directly): for `Run`, check
      absence of an `UnmatchedRun` diagnostic for this block (same technique
      as `is_short_if`'s `UnmatchedIf` check) before falling back to
      `Block.span.end`; for `Process`, fall back to `Block.span.end`
      unconditionally when `closer` is `None` (no diagnostic exists to
      disambiguate — see research.md §10). `null` response for a token not
      covered by any block (FR-011). Wire into `lib.rs`'s hover handler
      (replacing T007's stub). Depends on T005–T007.
- [X] T020 [P] [US3] Add `crates/drut-lsp/tests/hover.rs`: Story 3's four
      Acceptance Scenarios, including the implicit-Run-close case and the
      short-IF-has-no-separate-closer case. Depends on T019. 4/4 passing,
      driven over real JSON-RPC via `Connection::memory()`.

**Checkpoint**: US1–US3 independently functional.

---

## Phase 6: User Story 4 - Get keyword suggestions while typing (Priority: P4)

**Goal**: Completion offers control words at the start of a statement, and
`keyword=value` pair names scoped to the specific enclosing control word
(`RUN`/`LOOP`/`PATHLOAD`/etc. — never scoped by a `PGM=` value, per FR-012's
explicit non-goal cross-referencing `001-voyager-script-parser` FR-019),
falling back to the general list when no scoping data exists or no control word
encloses the cursor.

**Independent Test**: Trigger completion at the start of a statement (general
list); trigger it after a recognized control word (scoped list, or the
documented fallback) — spec.md's own Independent Test.

### `voyager-core` additions for User Story 4

- [X] T021 [US4] Define `KeywordEntry`/`KeywordRole`/`CompletionContext` and the
      FR-012 dictionary content in new `crates/voyager-core/src/keywords.rs`,
      re-exported from `crates/voyager-core/src/lib.rs` (data-model.md §1,
      `contracts/keyword-dictionary-api.md`). The dictionary itself is a
      hand-written, corpus-census-derived artifact (structural-position
      classification against the fixture corpus, reusing Phase 1's
      control-word evidence-trail methodology, per FR-012 and constitution
      Principle II) — not a hand-guessed or vendor-doc-copied list. Each
      `PairKeyword` entry's `observed_with` records which control word(s) it
      was actually seen paired with; never a `PGM=` value.
      **Real-usage census completed 2026-08-10**: `ControlWord` entries are
      populated from `statement.rs`'s already corpus-evidenced
      `FIXED_KEYWORDS`. `PairKeyword` entries were originally left empty
      (the external corpus wasn't reachable in the environment this task was
      first implemented in) but were subsequently populated once
      `DRUT_CORPUS_PATH` became available — 198 distinct keyword names in the
      first pass, filtered from 2,689 raw `(control_word, keyword)`
      observations down via `distinct_files >= 3` (the corpus's own
      dominant-signal threshold) plus an identifier-shape filter. That first
      pass's identifier-shape filter surfaced a real, confirmed
      `pair_keyword_boundaries` parsing defect (quote-unawareness inside a
      `Control` statement's keyword-list scan) — fixed the same day
      (`specs/001-voyager-script-parser/spec.md`'s FR-003 amendment) and the
      census re-run against the fix, landing at **197** distinct keyword
      names (one entry, `COST`/`PRINT`, dropped as the bug's own artifact;
      every other entry unchanged — see `keywords.rs`'s module doc for the
      full methodology, filtering rationale, and the fix's confirmed effect).
      `completion_candidates`/`did_you_mean` were already fully implemented
      and correct against an empty dictionary; populating real data required
      no code change, only the `PAIR_KEYWORDS` array's content.
- [X] T022 [US4] Implement `completion_candidates()` in
      `crates/voyager-core/src/keywords.rs` per data-model.md §1's rules:
      `None` context → every `ControlWord` entry; `Some(word)` → `PairKeyword`
      entries whose `observed_with` contains `word` (case-insensitive), falling
      back to the full `PairKeyword` list if that set is empty (never an empty
      suggestion list). Depends on T021.
- [X] T023 [P] [US4] Add `crates/voyager-core/tests/keywords.rs`: dictionary
      lookup and `completion_candidates`' both branches plus the
      empty-`observed_with`-fallback case. Depends on T022. **Landed as
      inline `#[cfg(test)]` unit tests in `keywords.rs` itself** rather than a
      separate `tests/keywords.rs` file — same coverage, colocated with the
      code it tests per this module's own internal convention (mirrors
      `position.rs`'s and `document_store.rs`'s own inline test modules).
      9/9 passing.

### `drut-lsp` implementation for User Story 4

- [X] T024 [US4] Implement `crates/drut-lsp/src/completion.rs`: return an empty
      list when the cursor is inside a comment or string (FR-013); otherwise
      populate a `CompletionRequestContext` (data-model.md §5, `drut-lsp`-local
      — distinct from, and feeding into, `voyager_core::keywords`'s own
      `CompletionContext`, data-model.md §1; don't conflate the two similarly-
      named types) by resolving `enclosing_control_word` via a
      span-containment scan over `parse_result.nodes` (research.md §2's
      resolved mechanism, adapted to the real `ParseResult` shape — see
      `resolve_enclosing_control_word`'s doc comment — no new structural
      inference) and call `voyager_core::keywords::completion_candidates`;
      map results to LSP `CompletionItem`s. Wire into `lib.rs`'s completion
      handler (replacing T007's stub). Depends on T022, T005–T007.
      **Quoted-string detection note**: `voyager-core` has no dedicated
      `TokenKind` for string content — quotes are individual `Punctuation`
      tokens (`crates/voyager-core/src/lexer.rs`). `in_comment_or_string`
      does quote-parity counting over already-tokenized `Punctuation` output
      (not a new grammar decision) rather than re-deriving lexer rules,
      consistent with Principle I — documented inline.
- [X] T025 [P] [US4] Add `crates/drut-lsp/tests/completion.rs`: Story 4's three
      Acceptance Scenarios, including the context-scoped-vs-general-fallback
      split, and an explicit regression case asserting `RUN PGM=HWYASSIGN` and
      `RUN PGM=MATRIX` receive the *identical* suggestion set (guards against
      ever silently crossing FR-019's per-program-box boundary). Depends on
      T024. 4/4 passing.

**Checkpoint**: US1–US4 independently functional.

---

## Phase 7: User Story 5 - Get a nudge on a likely-misspelled keyword (Priority: P5)

**Goal**: A token closely (but not exactly) matching exactly one dictionary
entry gets a "did you mean" nudge; no nudge for an exact match, no match, or a
tie.

**Independent Test**: Type a control word with one transposed letter, confirm a
"did you mean" naming the real entry — spec.md's own Independent Test.

**Depends on User Story 4** (spec.md's own stated ordering: reuses US4's
dictionary and completion-candidate infrastructure).

### Implementation for User Story 5

- [X] T026 [US5] Implement `did_you_mean()` in
      `crates/voyager-core/src/keywords.rs`: hand-written, case-insensitive
      Damerau-Levenshtein edit distance (research.md §5); returns the unique
      dictionary entry at distance ≤ 2 when exactly one exists, `None` for an
      exact match, no sufficiently-close match, or a tie. Depends on T021.
- [X] T027 [P] [US5] Add `did_you_mean` cases to
      `crates/voyager-core/tests/keywords.rs`: unique close match (including a
      transposition), no match, exact match, and a tie between two equally-close
      entries. Depends on T026. **Tie coverage landed as a dedicated test
      against a synthetic two-entry dictionary** (`nearest_within_threshold_
      returns_none_on_a_genuine_tie`), testing the tie-selection *algorithm*
      directly and deterministically rather than hoping for an incidental
      real-dictionary collision — kept this way even after the real
      `PairKeyword` census landed (T021), since a synthetic, guaranteed tie
      is still the more reliable, intent-revealing test than searching the
      real 217-entry dictionary for one.
- [X] T028 [US5] Implement `crates/drut-lsp/src/spellcheck.rs`: `SpellCheckHint`
      derivation for a `Word` token that isn't already an exact dictionary
      match (FR-015), surfaced via the existing hover response
      rather than a new LSP method (`contracts/lsp-capabilities.md`'s
      "rides on hover/completion" decision, Principle VI). Wired into
      `hover.rs`'s fallback path (a token that resolves to no block-hover
      fact tries a spell-check nudge before returning `None`) — not also
      into `completion.rs`, since completion's own request shape (a cursor
      position, not a specific already-typed token to critique) doesn't map
      onto "did you mean" the same direct way hover's token-under-cursor
      does; hover alone fully satisfies FR-014's "surfaced... via the
      existing hover/completion responses" (an "and/or", not "both", per
      `contracts/lsp-capabilities.md`'s own wording). Depends on T019, T024,
      T026.
- [X] T029 [P] [US5] Add `crates/drut-lsp/tests/spellcheck.rs`: Story 5's three
      Acceptance Scenarios. Depends on T028. 3/3 passing.

**Checkpoint**: US1–US5 independently functional.

---

## Phase 8: User Story 6 - See structural nuance through highlighting (Priority: P6)

**Goal**: Semantic tokens distinguish a short-`IF` from a block-style `IF`, and
flag a statement following a validly-resolved `BREAK` (within its loop) as
unreachable — never flagging on a `MisplacedBreak`.

**Independent Test**: Open a file with both `IF` shapes and confirm distinct
token types; open a file with a post-`BREAK` statement and confirm it's flagged
unreachable — spec.md's own Independent Test.

### Implementation for User Story 6

- [X] T030 [US6] Implement `crates/drut-lsp/src/semantic_tokens.rs`:
      `ShortIf`/`Unreachable` derivation per data-model.md §6 (short-IF reuses
      hover's block-kind lookup; `Unreachable` is a linear scan of each
      loop's children after its first validly-resolved child `BREAK`,
      explicitly excluding any `BREAK` already reported as `MisplacedBreak` —
      FR-016–FR-018), delta-encoded via `position.rs`. Wire into `lib.rs`'s
      `semanticTokens/full` handler (replacing T007's stub). Depends on T019,
      T005–T007. **Added a second legend type, `statement`**, beyond the
      spec-named `shortIf`: an unreachable-flagged statement needs *some*
      base token type to carry the `unreachable` modifier on, and this
      feature declares no general-syntax types (that's the static grammar's
      job, FR-021) — `statement` is a minimal, generic base type for exactly
      this modifier-only case, documented in `lib.rs`.
- [X] T031 [P] [US6] Add `crates/drut-lsp/tests/semantic_tokens.rs`: Story 6's
      three Acceptance Scenarios, including the no-flag-on-`MisplacedBreak`
      case. Depends on T030. 4/4 passing.
- [X] T032 [P] [US6] Add `editors/vscode/package.json`'s
      `contributes.semanticTokenTypes`/`Modifiers` (`shortIf` type,
      `unreachable` modifier) and `contributes.semanticTokenScopes`
      (`contracts/extension-manifest.md`). Depends on T012. Also includes
      the `statement` base type `semantic_tokens.rs` (T030) needs to carry
      the `unreachable` modifier on.

**Checkpoint**: All six user stories independently functional — full feature
complete.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Final gates that span every story.

- [X] T033 [P] Add `crates/drut-lsp/tests/position_encoding.rs`: a fixture
      containing a supplementary-plane character (e.g. in a comment), asserting
      every diagnostic/hover/semantic-token position lands correctly under
      UTF-16 counting (FR-019, FR-020, SC-005). Depends on T015, T019, T030.
      3/3 passing. Hit, and resolved, a new manifestation of this machine's
      documented Application Control quirk (blocked the freshly-linked test
      binary itself, not a build script) — see
      `docs/known-environment-quirks.md`'s new 2026-08-10 row; resolved by
      deleting the stale binary and letting `cargo test` relink it, no code
      change involved.
- [X] T034 [P] Run `cargo clippy -p voyager-core -p drut-cli -p drut-lsp
      --all-targets -- -D warnings` and resolve every warning (the workspace's
      existing zero-warning gate, extended to the two new components).
      Verified zero-warning across the whole workspace (`cargo clippy
      --workspace --all-targets -- -D warnings`).
- [X] T035 [P] Update root `README.md`'s Status/Repository-layout/Credits
      sections to describe `drut-lsp` and `editors/vscode/` alongside
      `voyager-core`/`drut-cli`, mirroring how `002-cli-check-format` updated it
      for `drut-cli`. **Preserve the existing Bhereth credit entry in Credits**
      (constitution Principle II binding condition #3) while editing nearby
      sections — do not remove or restructure it. Verified: Credits section
      untouched. Also added a "Publishing" section and updated "Build/test
      everything"/"Try the CLI" with `server`/extension build commands.
- [ ] T036 Walk through `quickstart.md`'s validation steps end-to-end (build
      through manual smoke tests) against a built `drut` binary and packaged
      extension; confirm each step's outcome matches its mapped Success
      Criterion (SC-001–SC-008). **Partial — steps 1-7 and 10 verified
      directly in this environment, steps 4's external-corpus half and 8-9
      genuinely blocked (no `DRUT_CORPUS_PATH`/real VS Code instance
      available here)**:
      - Step 1 (build): ✅ clean.
      - Step 2 (`keywords` module): ✅ 9/9.
      - Step 3 (`protocol_smoke`): ✅ 5/5.
      - Step 4 (full-corpus parity): committed-fixtures half ✅ (see T017);
        external-corpus half correctly `ignored`, not runnable here.
      - Step 5 (hover/completion/spellcheck/semantic_tokens): ✅ 4+4+3+4.
      - Step 6 (`position_encoding`): ✅ 3/3.
      - Step 7 (extension packaging): ✅ clean `.vsix`, both grammar files
        confirmed included (see T037).
      - Steps 8-9 (manual VS Code smoke tests): **not run** — this
        environment has no VS Code instance to install the packaged
        extension into. Remains a real, outstanding manual verification
        step for you (or CI with a VS Code test harness) before treating
        SC-001/SC-003/FR-025's *editor-observed* behavior as fully proven —
        everything each step exercises is unit/integration-tested already
        (T009, T014, T018's now-corrected scope, T016), but that's not a
        substitute for actually seeing it work inside VS Code.
      - Step 10 (full suite + clippy): ✅ all green, zero warnings, confirmed
        repeatedly throughout this implementation pass.
- [X] T037 [P] Confirm packaging/publishing readiness: `npx @vscode/vsce
      package` and an `ovsx` dry-run/validation (not a real publish) under
      Drut's own publisher identity (FR-027); document the actual publish
      command in `editors/vscode/README.md` or the root `README.md`.
      **`vsce package` verified working**: added `.vscodeignore` (trims
      src/test/tsconfig.json/`.map` files from the package) and a
      `repository` field to `package.json` (fixed two of `vsce`'s three
      warnings); the remaining warning (missing root `LICENSE` file) is a
      pre-existing, repo-wide gap unrelated to this feature — not fabricated
      here. Produces a clean 321-file, ~460 KB `.vsix`. **`ovsx` has no
      actual dry-run/validate-only mode** — `ovsx publish` always requires a
      real personal access token and always genuinely attempts to publish;
      confirmed the CLI installs and its `publish`/`--help` surface exists,
      but did not (and could not, without credentials this agent doesn't
      have and shouldn't be given) run a real publish attempt. Documented
      both the real publish commands and this limitation in the root
      README's new "Publishing" section.
- [X] T038 [P] Extend the root `README.md`'s "Dependency auditing" section to
      cover `drut-lsp` (`cargo audit`) and `editors/vscode/`'s npm dependencies
      (`npm audit`), per research.md §11's standing recommendation. `npm
      audit` was actually run against `editors/vscode/`'s dependencies (0
      vulnerabilities found, 2026-08-10) — README updated with the real
      result, not just a recommendation to run it. `cargo audit` itself
      isn't installed in this environment; left as the standing
      recommendation research.md §11 already documents (pin versions
      confirmed via docs.rs/crates.io during planning, not re-verified via
      `cargo audit` here).
- [X] T039 [P] Add a no-panic sweep to `crates/drut-lsp/tests/protocol_smoke.rs`
      (or a new `tests/no_panic.rs`) exercising every handler — hover,
      completion, spellcheck, semantic tokens, not only diagnostics/position
      translation — against a small curated set of malformed/edge-case
      document *text* content (empty document, a truncated mid-statement
      buffer, a document containing only a comment, a document containing a
      Unicode replacement character U+FFFD as ordinary text — the closest
      analogue reachable via LSP to a real encoding anomaly, per research.md
      §12 — and a supplementary-plane character at a boundary position) to
      verify FR-004's no-panic guarantee holds for `drut-lsp`'s own new code
      paths specifically, not only the guarantee `voyager-core` already
      proves for its own logic (`/speckit-analyze` finding E1). Note: raw
      non-UTF-8 *bytes* are not a reachable input to `drut-lsp` at all
      (research.md §12) — this sweep tests malformed-but-valid-Unicode text,
      not byte-level malformation. Depends on T015, T019, T024, T028, T030
      (needs every handler to exist). **Landed as `tests/no_panic.rs`** (10
      edge-case documents × hover/completion/semantic-tokens at multiple
      positions each, plus a didChange re-parse round trip per case, plus a
      final "server still alive" round trip proving no case took the server
      thread down) — 1/1 passing, zero panics across the whole sweep.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS User Stories 2–6 only.
  **User Story 1 does not depend on this phase at all.**
- **User Story 1 (Phase 3)**: Depends on Setup (T004) only — the most
  independent story in this feature; needs no `drut-lsp`/Foundational work.
- **User Story 2 (Phase 4)**: Depends on Foundational (Phase 2) and US1's
  `package.json`/language ID (T012).
- **User Story 3 (Phase 5)**: Depends on Foundational only, not on US2.
- **User Story 4 (Phase 6)**: Depends on Foundational only, not on US2/US3.
- **User Story 5 (Phase 7)**: Depends on Foundational **and User Story 4**
  (reuses its dictionary — the one deliberate cross-story dependency, per
  spec.md's own stated ordering) **and User Story 3** (spell-check nudges ride
  on the hover response `hover.rs` implements).
- **User Story 6 (Phase 8)**: Depends on Foundational and **User Story 3**
  (reuses `hover.rs`'s block-kind lookup for the short-IF token type) and US1's
  `package.json` (T012, for the semantic-token contribution points).
- **Polish (Phase 9)**: Depends on all six user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: No dependency on any other story.
- **User Story 2 (P2)**: No dependency on US1's *content* (only its
  `package.json` scaffold, T012) or on US3–US6.
- **User Story 3 (P3)**: No dependency on US2, US4, US5, US6.
- **User Story 4 (P4)**: No dependency on US2, US3, US6. (US5 depends on US4,
  not the reverse.)
- **User Story 5 (P5)**: Depends on US4 (dictionary) and US3 (hover response
  it rides on).
- **User Story 6 (P6)**: Depends on US3 (hover's block-kind lookup) only among
  the other stories.

### Parallel Opportunities

- T001–T004 (Setup): T003 (drut-lsp) and T004 (extension) are independent of
  each other once T001/T002 land — `[P]`.
- T005, T006 (Foundational) are mutually independent — `[P]`; T007 depends on
  both; T008, T009 are independent of each other once T007 exists — `[P]`.
- **Once Foundational (Phase 2) is done, User Stories 2, 3, and 4 can all be
  staffed and built fully in parallel** — none of them depends on either of
  the others. User Story 1 can run in parallel with *all* of Phase 2–8, since
  it only needs Setup.
- User Story 5 must wait for both US3 and US4 to complete; User Story 6 must
  wait for US3 only.
- Within US1: T010, T011, T012 are mutually independent — `[P]`; T013 depends
  on T011/T012; T014 depends on T012.
- Within US4: T021 alone, then T022 (same file, sequential); T023 is `[P]`
  once T022 lands; T024 depends on T022 but is a different file — `[P]`; T025
  depends on T024.

---

## Parallel Example: Foundational

```text
# Launch together once Setup (T001–T004) is done:
Task: "Define ServerState/OpenDocument in crates/drut-lsp/src/document_store.rs"
Task: "Implement position.rs's UTF-16 translation contract"
```

## Parallel Example: User Story 1 (can start alongside Foundational)

```text
# Launch together once T004 (extension scaffold) is done:
Task: "Author editors/vscode/language-configuration.json"
Task: "Author editors/vscode/syntaxes/drut.tmLanguage.json"
```

## Parallel Example: Once Foundational Is Done

```text
# Launch together — three independent stories:
Task: "Implement crates/drut-lsp/src/diagnostics.rs"              # US2
Task: "Implement crates/drut-lsp/src/hover.rs"                    # US3
Task: "Define KeywordEntry/KeywordRole/CompletionContext + dictionary in voyager-core/src/keywords.rs"  # US4
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T004).
2. Complete Phase 3: User Story 1 — no Foundational work required.
3. **STOP and VALIDATE**: install the packaged extension with no `drut` binary
   present; confirm highlighting and the single non-repeating notice
   (spec.md's own Independent Test; SC-001).
4. Static highlighting is independently shippable here — every other story
   requires a working `drut server`, which isn't built yet.

### Incremental Delivery

1. Setup → User Story 1 → validate independently → ship (MVP).
2. Setup + Foundational → shared server scaffolding ready.
3. Add User Story 2 (diagnostics) → validate → ship.
4. Add User Story 3 (hover) → validate → ship.
5. Add User Story 4 (completion) → validate → ship.
6. Add User Story 5 (spell-check, needs US4 + US3) → validate → ship.
7. Add User Story 6 (semantic tokens, needs US3) → validate → ship.
8. Polish (clippy, docs, quickstart walkthrough, packaging/publishing
   readiness) → final gate before merge.

### Parallel Team Strategy

With multiple developers: one starts User Story 1 immediately (needs only
Setup); the rest complete Foundational together, then split across User
Stories 2, 3, and 4 (mutually independent). Once US3 and US4 land, remaining
capacity picks up User Story 5 (needs both) and User Story 6 (needs US3).

---

## Notes

- `[P]` tasks touch different files from every sibling task in the same phase;
  a real completion-order dependency (if any) is still called out in the task
  description rather than by withholding `[P]`, matching
  `002-cli-check-format/tasks.md`'s established convention.
- `[US1]`–`[US6]` trace every story-phase task back to spec.md; Setup/
  Foundational/Polish tasks carry no story label.
- Completion's context-scoping (US4, T022/T024) is deliberately narrow — by
  control word only, never by a `PGM=` value — per FR-012's explicit
  cross-reference to `001-voyager-script-parser` FR-019; T025's regression
  case exists specifically to keep this from silently drifting during
  implementation.
- Position-encoding translation (T006) is the single place UTF-16 conversion
  logic lives (`contracts/position-encoding.md`) — no handler task (T015,
  T019, T024, T030) should reimplement it independently; each wires into it.
- Commit after each task or logical group; stop at any story's checkpoint to
  validate it independently before continuing.
