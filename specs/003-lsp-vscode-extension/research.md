# Phase 0 Research: Drut LSP Server & VS Code/Open VSX Extension

Two items were explicitly left as open engineering decisions in `spec.md`'s
Assumptions — position-encoding ownership (FR-019) and completion's
context-awareness depth (FR-012). §1–§2 below resolve both for real, with
evidence, per this phase's own instruction not to defer them a fourth time.
Everything else in this file is ordinary technology-choice research.

## 1. Position-encoding ownership — RESOLVED: the adapter crate owns the translation; `voyager-core`'s `Span` does not change

This question has been deferred three times on purpose — Phase 1's spec.md
Assumptions flagged it as "Phase 3's problem to solve, not this phase's,"
Phase 3's own feature prompt repeated the flag, and this spec's Assumptions
deferred it once more to this exact document. It stops here.

**Decision**: `voyager-core::Span`/`Position` keep counting Unicode scalar
values (`char`s), 1-based, exactly as they do today
(`crates/voyager-core/src/span.rs`). The new `drut-lsp` crate owns a single
translation function at its boundary that converts a `voyager-core::Position`
into an LSP `Position` (0-based line, UTF-16 code-unit character offset) —
and the reverse, for translating an incoming LSP position (e.g. a hover or
completion request's cursor location) back into the char-column
`voyager-core` needs to query its own data.

**Rationale**:

1. **`voyager-core`'s own contract forbids this.** FR-001 in
   `001-voyager-script-parser/spec.md` states `voyager-core` has "no file I/O,
   network access, or protocol dependency inside the crate itself." UTF-16
   code-unit counting isn't a general-purpose representation — it's
   specifically the Language Server Protocol wire convention (confirmed
   below). Making `Span` count UTF-16 units would mean the core crate's
   foundational position type is shaped around one specific downstream
   protocol's needs, which is exactly the "protocol dependency inside the
   crate" FR-001 rules out — even though no new *runtime dependency* (crate)
   would be added, it's still a semantic coupling the contract forbids.
2. **It would silently degrade the CLI's already-shipped output.**
   `drut-cli`'s plain-text and SARIF output (`002-cli-check-format`) render
   `Position.column` directly for human/CI consumption. For the small set of
   real files containing a supplementary-plane character (outside this
   corpus so far, but not excluded by FR-034's decode path), a UTF-16-based
   column would show `2` for a single visible character's worth of advance —
   correct for an LSP client, actively confusing in a terminal or a SARIF
   viewer that has nothing to do with UTF-16. Phase 1's spec.md already
   states char-count was chosen "so it doesn't introduce a second,
   inconsistent column scheme" — changing `Span` now would introduce exactly
   that inconsistency, just moved to a different pair of consumers (CLI
   readability vs. LSP correctness), and would do it retroactively to a
   contract two already-shipped features depend on.
3. **Changing `Span` wouldn't even remove the adapter's translation layer —
   only shrink it.** LSP positions are 0-based; `voyager-core::Position` is
   1-based (`crates/voyager-core/src/span.rs`). This line-numbering
   translation is unavoidable at the LSP boundary regardless of which way
   the UTF-16 question is decided. Since `drut-lsp` already has to write a
   `Position -> lsp_types::Position` conversion function to handle the
   0-based/1-based difference, folding UTF-16 code-unit counting into that
   same function costs approximately nothing extra. Option A (change `Span`)
   would not eliminate this function, it would only remove one piece of its
   body — a marginal simplification that isn't worth the cost in point 1–2.
4. **The client this feature ships (`vscode-languageclient`) cannot use
   anything but UTF-16 anyway**, which forecloses the one scenario where
   avoiding translation entirely might have been possible. LSP 3.17 added
   negotiable position encoding (`general.positionEncodings` client
   capability; `PositionEncodingKind.Utf8`/`Utf16`/`Utf32`) — if the client
   advertised and the server selected `utf-32` (Unicode scalar values, the
   same thing `char` already counts), `drut-lsp` could skip UTF-16
   conversion entirely and send `voyager-core` positions almost as-is (still
   needing the 0-based/1-based fix from point 3). This was investigated and
   ruled out: `vscode-languageclient`'s `initialize` response handling does
   a strict check on `result.capabilities.positionEncoding` and throws
   `Unsupported position encoding` for anything other than `utf-16` (or
   `undefined`, which defaults to `utf-16`) — confirmed via the
   `microsoft/vscode-languageserver-node` issue tracker and a real-world
   language server hitting exactly this failure mode
   ([`pappasam/jedi-language-server` issue #351](https://github.com/pappasam/jedi-language-server/issues/351)
   shows a server that advertised a non-UTF-16 encoding being rejected by a
   VS Code client). UTF-16 is therefore not a choice `drut-lsp` gets to
   avoid by clever capability negotiation — it is a hard requirement of the
   one client this feature ships, so the translation must exist somewhere;
   points 1–3 settle where.

**Sources**: [LSP 3.17 specification, Position Encoding section](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/) (encoding kinds and the UTF-16-mandatory-for-backward-compatibility rule); [`microsoft/vscode-languageserver-node` issue #748](https://github.com/microsoft/vscode-languageserver-node/issues/748) (UTF-8/UTF-32 support request against the TypeScript client library); [`pappasam/jedi-language-server` issue #351](https://github.com/pappasam/jedi-language-server/issues/351) (real client-side rejection of a non-UTF-16-declaring server).

**Alternatives considered**:

- **Change `Span` to count UTF-16 code units.** Rejected — see rationale
  points 1–3.
- **Change `Span` to store a flat UTF-16 offset instead of line/column.**
  Rejected for the same reason as above, plus it would be a larger, less
  ergonomic change to `voyager-core`'s public API for a benefit that still
  doesn't eliminate the adapter-boundary translation (point 3).
- **Negotiate `utf-32` position encoding to skip translation.** Investigated
  and rejected — not viable with `vscode-languageclient` today (rationale
  point 4). Worth re-checking if a future `vscode-languageclient` release
  relaxes this, but not something this phase can rely on.

**Concrete translation contract** (implemented in `drut-lsp`, not
`voyager-core`): for a given open document's text and a `voyager-core`
`Position { line, column }` (1-based, `char` count), the UTF-16 `character`
offset is computed by taking that line's text, iterating its `char`s up to
(not including) the `column`-th one, and summing each one's
`char::len_utf16()` (1 for any Basic-Multilingual-Plane character, 2 for a
supplementary-plane character) — a direct, dependency-free translation using
only `std`. The reverse direction (LSP position → `voyager-core` position,
needed for e.g. a hover request's cursor location) walks the same line
counting UTF-16 units consumed until reaching the requested `character`
offset, then reports the `char`-index reached. Both directions are `O(line
length)`; per the Edge Cases in spec.md ("a large-but-realistic file"), this
is re-scanned per request rather than cached, since the fixture corpus is
ASCII/Latin-range technical script text (`001-voyager-script-parser/spec.md`
Assumptions) where this is a handful of microseconds even for a long line —
revisit only if real profiling on large files shows otherwise.

## 2. Completion's context-awareness depth — RESOLVED: full control-word-scoped completion is achievable this phase, not the general-list fallback

**Scope note, read this before the rest of this section**: "context-aware"
here means, and only ever means, *which control word* (`RUN`, `LOOP`,
`PATHLOAD`, etc.) structurally encloses the cursor — the same closed,
grammar-recognized vocabulary `voyager-core`'s `FIXED_KEYWORDS`
(`crates/voyager-core/src/statement.rs`) already matches structurally. It
never means *which program* a `RUN`/`PHASE` block happens to invoke (the
`PGM=` keyword's value) — that is per-program-box keyword knowledge, which
`001-voyager-script-parser` FR-019 explicitly puts out of scope for
`voyager-core` ("it does not know that `RUN PGM=MATRIX` takes a `ZONES=`
keyword"), and this resolution does not reopen that boundary. Concretely:
`RUN PGM=HWYASSIGN` and `RUN PGM=MATRIX` receive the *identical* completion
suggestion set from this feature, because both are scoped by the same
control word, `RUN` — the dictionary's `observed_with` field (data-model.md
§1) records control words only, never `PGM=` values, and nothing in
`CompletionContext` or `voyager_core::keywords` ever reads a pair's *value*,
only which control word a `Statement` carries. If a later phase ever wants
real per-program scoping, that is new, additional work on top of this
one — not something this resolution silently already did.

**Decision**: `drut-lsp` scopes `keyword=value` pair-name completion to the
enclosing statement's control word by re-parsing the open document's current
buffer (already required for diagnostics, FR-005/FR-002) via
`voyager-core::parse_bytes`, locating the `Statement` whose `span` contains
the cursor position, and — when that statement's `kind` is
`StatementKind::Control { word, .. }` — restricting keyword-name suggestions
to the subset of the dictionary's entries observed (during the FR-012
corpus census) paired with that control word. The general-syntax fallback
list (Assumptions' documented "acceptable, spec-conformant" degraded mode)
is used only for the genuinely context-free case: the cursor sits before any
control word exists yet on the current statement (Acceptance Scenario 1).

**Rationale**: This isn't new engineering work invented for completion — it
reuses exactly the same `parse_bytes` call and `Statement`/`StatementKind`
data FR-005 (diagnostics) and FR-008 (hover) already require the server to
hold for the open document. `voyager-core` never panics on malformed or
in-progress input (FR-004, and `001-voyager-script-parser` FR-001's
no-panic guarantee), so calling `parse_bytes` on a buffer mid-edit — e.g. a
line reading `PATHLOAD VOL=mw[1] ` with the cursor after the trailing space,
before a new keyword name has been typed — still produces a best-effort
`Statement` whose `kind` is `Control { word: "PATHLOAD", pairs: [("VOL",
..)] }`; the `word` field is populated as soon as the statement's first
token is recognized as being followed by further tokens rather than an `=`
immediately after it (FR-023's own structural rule), independent of how
complete the rest of the line is. Locating "the enclosing statement" for a
cursor position is a matter of a span-containment scan — **recursively over
`ParseResult.nodes`**, not a flat `.statements` list (`nodes: Vec<Node>`,
`Node = Statement | Block`, with `Block.children`/`IfBranch.children` nesting
the same type further; see data-model.md §4's Correction note, added
2026-08-10 after this file's own original wording assumed a flat list that
doesn't exist) — no new structural inference beyond what parsing already
computed either way, so it introduces no grammar-logic duplication
(constitution Principle I): the server asks `voyager-core` what statement is
here and what control word it has, it never re-derives that itself.

**What this resolves in spec.md's Assumptions**: the "or explicitly scoped to
only offer the ~13 general-syntax control words... if full control-word-scoped
completion is out of reach this phase" fallback describes a real, always-kept
code path (Acceptance Scenario 1's case, and any cursor position where
`parse_bytes` can't identify an enclosing `Control` statement at all — e.g.
inside a `ShellEscape` or `Label` statement), but it is not this phase's
*primary* mode: full control-word-scoped completion is the default outcome
whenever the cursor sits inside an already-recognized `Control` statement,
which covers Acceptance Scenario 2 directly — and, per the Scope note above,
"full" here still tops out at the control-word level, never per-`PGM=`-value.

**Alternatives considered**:

- **General-list-only fallback, deferring context-awareness to a later
  phase.** Rejected now that the mechanism above is confirmed to cost no
  extra structural work — choosing the weaker option when the stronger one
  is free would leave real completion value on the table for no reason.
- **A separate, incremental/streaming parse tuned for editor
  responsiveness** (only re-parsing the changed region, rather than the
  whole open document on every keystroke). Rejected for this phase: the
  fixture corpus's file sizes (`001-voyager-script-parser/plan.md` scale
  note) make whole-document re-parse on every change cheap enough that
  incremental parsing would be premature optimization with real
  implementation complexity (and its own risk of drifting from
  `voyager-core`'s canonical parse) — revisit only if real profiling on
  unusually large documents shows the whole-document re-parse costing
  perceptible latency (spec.md SC-003's "perceptibly-immediate" bar).

## 3. LSP transport/protocol crate choice

- **Decision**: `lsp-server` (the synchronous, `crossbeam-channel`-based LSP
  scaffold co-owned by the Rust Programming Language organization and used by
  rust-analyzer itself) for the JSON-RPC/stdio transport loop, paired with
  `lsp-types` for the typed protocol message/capability structs.
- **Rationale**: Hand-rolling JSON-RPC framing and the LSP message catalog
  would be pure duplicated effort with no grammar/parsing content — the same
  "don't reinvent a solved problem" reasoning `002-cli-check-format/
  research.md` §2/§3 already applied to `clap`/`ignore`. `lsp-server` is
  synchronous, matching this project's existing all-synchronous architecture
  (`voyager-core` and `drut-cli` use no async runtime anywhere); adopting an
  async-first crate (`tower-lsp`/`tower-lsp-server`) would introduce `tokio`
  as a new, first-of-its-kind dependency for a workload — fast, in-memory,
  local-process parsing with no network I/O — that has no actual need for
  async concurrency. `lsp-server` also ships `Connection::memory()`, an
  in-process duplex connection constructor built specifically for testing
  (confirmed via docs.rs) — this is the mechanism §9's test strategy uses to
  drive the server through real JSON-RPC messages without spawning a
  subprocess.
- **Alternatives considered**:
  - `tower-lsp` / the community-maintained `tower-lsp-server` fork (the
    original was archived; a `tower-lsp-community` org fork continues it).
    Rejected: async-only design, and open questions in that ecosystem itself
    about whether Tower's request/response model is a natural fit for LSP's
    bidirectional stream shape. No benefit for this project's synchronous,
    single-process, local-parsing workload.
  - `gen-lsp-types` (an automatically-generated alternative to `lsp-types`
    that `rust-analyzer` itself switched to in 2026, per
    [rust-lang/rust-analyzer#22115](https://github.com/rust-lang/rust-analyzer/pull/22115),
    to pick up LSP 3.18 additions `lsp-types` was missing, e.g.
    `SnippetTextEdit`). Rejected for this phase: single-maintainer, young
    (`v0.4.0` as of that PR), not a drop-in replacement (2,000+ line diff in
    rust-analyzer's own migration, different naming conventions throughout).
    None of the specific gaps that motivated rust-analyzer's switch
    (`SnippetTextEdit`, `FoldingRange.kind` limits, a
    `workspace/diagnostics` field typo) are in this phase's scope
    (diagnostics, hover, completion, spell-check, semantic tokens — all
    long-stable LSP 3.16/3.17 surface). `lsp-types` 0.97.0 already exposes
    `PositionEncodingKind` (§1) with no feature flag needed (confirmed via
    docs.rs) and remains, by a wide margin, the most widely depended-on LSP
    types crate in the Rust ecosystem (31M+ downloads). Revisit if a future
    phase needs an LSP 3.18-only capability `lsp-types` genuinely lacks.

## 4. Where the keyword dictionary and fuzzy-match logic live

- **Decision**: Add a new public `keywords` module to `voyager-core`
  (`crates/voyager-core/src/keywords.rs`) exposing the FR-012 dictionary
  (control words + common `keyword=value` pair names, tagged with which
  control word(s) they were observed paired with during the corpus census),
  a completion-candidate query function, and a `did_you_mean`-style
  fuzzy-match function (§5). `drut-lsp` calls these; it does not hold its own
  copy of the dictionary or re-implement the matching algorithm.
- **Rationale**: Directly mirrors `002-cli-check-format/research.md` §1's
  precedent for exactly this kind of question (`format`'s indentation-
  decision logic went into `voyager-core`, not the CLI, so any future
  adapter could reuse it without duplicating structural knowledge). The
  keyword dictionary is a real-usage-derived artifact in the same evidence-
  trail tradition as `voyager-core`'s existing `FIXED_KEYWORDS` list
  (`crates/voyager-core/src/statement.rs`) and Phase 1's control-word census
  work, even though — unlike `FIXED_KEYWORDS` — it plays no role in parsing
  decisions (FR-023's structural rule already fully disambiguates
  `Control`/`Assignment` without any list, per `001-voyager-script-parser`'s
  settled CHK008 finding). Keeping it in `voyager-core` costs nothing this
  phase (only one adapter consumes it) and avoids a real future cost: if a
  later phase adds spell-check to the CLI or an MCP tool, the dictionary and
  matching algorithm are already there to reuse rather than needing to be
  copied or re-derived — consistent with constitution Principle II treating
  keyword lists as a first-class, carefully-sourced artifact, not adapter-
  local convenience data. This module adds no new runtime dependency
  (fuzzy-match is hand-written, §5), so it doesn't touch `voyager-core`'s
  zero-dependency guarantee (FR-027).
- **Alternatives considered**: Keep the dictionary and matching logic inside
  `drut-lsp` as adapter-local data, on the theory that only the editor
  experience needs it this phase. Rejected: it would need to move to
  `voyager-core` the moment any second adapter wanted it, at which point the
  move is strictly harder (two call sites to update, and a real risk the
  `drut-lsp` copy and the moved copy drift during the transition) than
  starting it in the right place now, for a cost of essentially zero this
  phase.

## 5. Fuzzy "did you mean" matching — algorithm and threshold

- **Decision**: Hand-written Damerau-Levenshtein edit distance (insertions,
  deletions, substitutions, and adjacent-character transpositions), case-
  insensitive (matching the case-insensitive keyword comparison
  `001-voyager-script-parser` FR-011 already establishes elsewhere in
  `voyager-core`). A token gets a "did you mean X" nudge only when exactly
  one dictionary entry has the strictly-lowest distance among all entries
  and that distance is ≤ 2 — a tie for lowest distance, or a lowest distance
  > 2, produces no nudge (spec.md Story 5 Acceptance Scenario 2 and the
  Edge Cases' "no entry close enough... no nudge" case).
- **Rationale**: Story 5's own example ("a transposed letter") specifically
  names the one common typo class plain Levenshtein distance doesn't treat
  as a single edit (a transposition costs 2 under plain Levenshtein, 1 under
  Damerau-Levenshtein) — using Damerau-Levenshtein directly serves that
  acceptance scenario. The algorithm is a well-known, small (~30-line)
  dynamic-programming routine; hand-writing it keeps `voyager-core` free of
  a new dependency for a solved-but-tiny problem, consistent with the
  project's general "don't add a crate for something this small and this
  well-understood" posture (contrast with `002-cli-check-format/research.md`
  §3's `ignore` crate decision, adopted specifically because `.gitignore`
  semantics are neither small nor simple). The distance-≤2/unique-minimum
  rule is a reasonable UX default, not a corpus-derived constant (there is
  no equivalent "real signal" survey possible for typo tolerance the way
  `002-cli-check-format/spec.md` FR-012's indentation rules had one) — it
  is documented here explicitly, per the same instruction that produced
  §1/§2, so it isn't invented silently during implementation, and it may be
  tuned later if real dictionary size makes ≤2 too noisy in practice.
- **Alternatives considered**: A dependency such as `strsim` (which itself
  provides Damerau-Levenshtein). Rejected for the same reason as above — the
  algorithm is small enough that hand-writing it costs less than evaluating
  and pinning a new dependency for it.

## 6. Semantic tokens delivery mechanism

- **Decision**: Standard LSP `textDocument/semanticTokens/full` (and its
  legend, declared once in `ServerCapabilities.semantic_tokens_provider`),
  with two Drut-specific token types beyond `lsp-types`'s standard set —
  `shortIf` (Story 6, distinguishing a self-closing short-`IF` from a
  block-style `IF`) and `unreachable` (a token *modifier*, applied to any
  token belonging to a statement that follows a validly-resolved `BREAK`
  within its loop, per FR-017/FR-018). The extension declares matching
  `semanticTokenScopes` in `package.json` so the active color theme can
  style them, and `semanticTokenTypes`/`semanticTokenModifiers`
  contributions so VS Code's generic semantic-token infrastructure
  recognizes the custom names.
- **Rationale**: Semantic tokens are an LSP-standard capability (not a VS
  Code-proprietary API), satisfying constitution Principle VI directly —
  any other LSP-capable editor that implements semantic tokens gets Story
  6's highlighting for free, with no VS Code-specific code in `drut-lsp`
  itself. Declaring custom token types/modifiers through the standard
  legend mechanism (rather than, say, a VS Code-only decorations API) is
  the documented, portable way to extend semantic highlighting beyond the
  built-in type list.
- **Alternatives considered**: VS Code's proprietary `TextEditorDecorationType`
  API (via a custom, non-LSP extension command) for the `unreachable` flag
  specifically. Rejected outright under Principle VI — a standard mechanism
  (semantic token modifiers) already covers this case exactly, so there is
  no justification for reaching for an editor-proprietary one.

## 7. VS Code / Open VSX extension scaffold and publishing

- **Decision**: A TypeScript extension under `editors/vscode/` (new
  top-level directory, sibling to `crates/`) using `vscode-languageclient`
  to spawn `drut server` (FR-024), a hand-written static
  `.tmLanguage.json` grammar plus `language-configuration.json`
  (brackets/comments) for FR-021/FR-022, packaged with `@vscode/vsce`
  (VS Code Marketplace) and published to Open VSX via the `ovsx` CLI
  (FR-027), both under Drut's own publisher identity.
- **Rationale**: `vscode-languageclient` is the standard Microsoft-maintained
  client library for exactly this wrapper role (spawn a server process,
  speak LSP over its stdio) — hand-rolling a JSON-RPC client in TypeScript
  would be the same kind of unjustified reinvention already rejected for
  `.gitignore`/SARIF/diff handling in `002-cli-check-format`. `@vscode/vsce`
  and `ovsx` are each publisher's own standard packaging/publishing CLI —
  there is no reasonable alternative that isn't "hand-build the `.vsix`
  archive and call each marketplace's upload API directly," which both
  tools already do correctly. `editors/vscode/` (not `crates/`, since this
  is not a Rust crate — mirrors the convention used by other
  Cargo-workspace-plus-editor-extension LSP projects such as
  rust-analyzer's own `editors/code/`) keeps the non-Rust component clearly
  separated from the Cargo workspace without inventing a new, unprecedented
  layout for this repository.
- **Alternatives considered**: Folding the extension into `crates/` anyway
  for "one directory to look in." Rejected — it isn't a Cargo workspace
  member and would misleadingly suggest it is one to anyone running
  `cargo build --workspace` there.

## 8. Bhereth extension reference workflow

- **Decision**: Any structural reference to
  `bhereth.language-citilabscubevoyager` (language registration shape,
  bracket/comment config, TextMate scope-naming conventions — constitution
  Principle II, FR-023) happens by reading his extension locally (outside
  this repository, exactly like `_archive/`'s existing vendor-documentation
  policy — never committed, never pulled into this repo's working tree even
  temporarily) and then writing Drut's own `.tmLanguage.json`/
  `language-configuration.json` from scratch in Drut's own structure and
  wording, the same verbatim-copying bar Principle II already holds vendor
  documentation to.
- **Rationale**: This is not a new process — it's the existing `_archive/`
  local-only-reference discipline (README.md, constitution Principle
  VIII) applied to a second kind of external reference material, exactly as
  the constitution's Bhereth-permission addendum already specifies. No new
  tooling or repository structure is needed to support it.

## 9. LSP-level test strategy (FR-028, Definition of Done)

- **Decision**: A new `drut-lsp` test suite drives the server through real
  JSON-RPC messages using `lsp_server::Connection::memory()` (an in-process,
  no-subprocess duplex connection built specifically for this purpose,
  confirmed via docs.rs) — sending `initialize`, `textDocument/didOpen`,
  and (for the full-corpus DoD check) asserting on published
  `textDocument/publishDiagnostics` notifications against every fixture-
  corpus file, mirroring `002-cli-check-format`'s `fixture_corpus_e2e.rs`
  pattern but at the protocol layer instead of spawning the built binary.
  Hover/completion/semantic-tokens requests are exercised the same way for
  their own acceptance scenarios.
- **Rationale**: `Connection::memory()` gives a real, spec-accurate protocol
  round-trip (catching wire-format/serialization bugs a purely in-process
  function-call-based test would miss) without the process-spawn overhead
  `002-cli-check-format/research.md` §7 already reasoned about avoiding for
  its own full-corpus checks — the same "assert correctness once, at the
  cheapest layer that still proves the real thing" principle, applied here
  at the LSP-message layer since that's what FR-028 and the Definition of
  Done actually require proof of (not re-proving `voyager-core`'s own
  parsing correctness, already covered by its own suite).
- **Alternatives considered**: Spawning the real `drut server` subprocess
  and talking to it over actual stdio pipes for every test. Rejected as
  unnecessarily slow for a 161-file corpus run with no meaningful coverage
  gain over `Connection::memory()`'s real-protocol-messages-in-process
  approach — reserved instead for a small number of true end-to-end smoke
  tests (quickstart.md) that also prove the packaged `drut server`
  subcommand itself launches correctly, the one thing `Connection::memory()`
  can't verify since it never invokes the actual subcommand dispatch path.

## 10. Hover's `counterpart` derivation for implicitly-closed blocks (corrected 2026-08-09)

- **Problem found**: `data-model.md` §4's original `BlockHoverFact.counterpart`
  derivation claimed it came from `Block.closer` "when resolved, including
  through Run/Process's implicit-close path." That is inconsistent with
  `Block.closer`'s own documented contract
  (`001-voyager-script-parser/data-model.md`), which is explicitly `None`
  "when the block closed implicitly (`Run`/`Process`) *or* is genuinely
  unmatched" — the exact opposite of what FR-009/Story 3 AS3/SC-004 require.
  As originally written, hover could not satisfy its own requirement for the
  one case (implicit close) that requirement exists to cover. Found via
  `/speckit-checklist` (CHK015/CHK016) and confirmed independently via
  `/speckit-analyze` (finding I1).
- **Decision**: `data-model.md` §4 now specifies a five-rule derivation (see
  that section — not restated here to avoid drift, same reasoning as
  `contracts/lsp-capabilities.md`'s own cross-reference). The two cases that
  needed real resolution:
  - **`Run`**: `Run` has an `UnmatchedRun` diagnostic category
    (`001-voyager-script-parser`'s six-category list). Absence of
    `UnmatchedRun` for a given `Run` block with `closer == None` means it
    closed implicitly — `counterpart` falls back to `Block.span.end`,
    mirroring the exact same diagnostic-absence technique `is_short_if`
    already uses with `UnmatchedIf`. Presence of `UnmatchedRun` means
    genuinely unmatched — `counterpart` stays `None`.
  - **`Process`**: `Process` is one of the four block kinds with *no*
    "unmatched" diagnostic category at all — there is no signal in
    `ParseResult.diagnostics` to distinguish "implicitly closed by the next
    `PROCESS`/`PHASE` opener" from "genuinely unmatched, reached EOF." No
    amount of adapter-side cleverness recovers a distinction `voyager-core`
    itself doesn't expose, and adding a new diagnostic category or `Block`
    field for this single, narrow case was rejected (see Alternatives) as
    disproportionate to the problem. `counterpart` reports `Block.span.end`
    unconditionally for this case — genuinely `voyager-core`'s own resolved
    extent for the block (via the same `end_span_or` fallback
    `002-cli-check-format/research.md` §8 documents), so it's a faithful
    report of what the parser itself determined, not an invented value,
    even on the rare occasions it's reporting a genuinely-unmatched block's
    extent rather than a truly implicit close.
- **Rationale for not changing `voyager-core` instead**: The alternative —
  adding an `UnmatchedProcess`-style diagnostic, or a new `Block` field
  distinguishing implicit-close from unmatched-at-EOF specifically for
  `Process` — would resolve the ambiguity more precisely, but at the cost of
  either introducing a new diagnostic category (a bigger, more consequential
  change than this narrow hover-accuracy question warrants, and one that
  would need its own fixture-corpus evidence per constitution Principle IV)
  or another purely-additive `Block` field in the style of `closer`/
  `opener_pairs` (`002-cli-check-format/research.md` §8) for a distinction
  only one adapter, in one narrow edge case, currently needs. Given the
  fixture corpus's own real-world evidence (`001-voyager-script-parser`'s
  full-corpus validation) shows no genuinely-unmatched `Process` block at
  all in 161 real files, the EOF sub-case this ambiguity concerns is
  expected to be rare-to-nonexistent in practice — the pragmatic,
  data-model-only fix is proportionate; a `voyager-core` change is not
  justified by evidence this phase has.
- **Product-level acknowledgment**: spec.md's Assumptions section now
  records this limitation explicitly (added alongside this research entry)
  rather than leaving it as an implementation detail invisible to a reader
  of spec.md alone.
- **Alternatives considered**:
  - Add a new `UnmatchedProcess` diagnostic category to `voyager-core`,
    giving `Process` the same disambiguation `Run` has. Rejected for this
    phase per the Rationale above — no fixture-corpus evidence motivates it,
    and it's a heavier change than this narrow question needs.
  - Add a new `Block` field (e.g. `closed_implicitly: bool`) purely for this
    disambiguation. Rejected as the same kind of disproportionate response —
    `Block.span.end` already gives a faithful, non-fabricated answer for
    both sub-cases; a new field would only sharpen a distinction with no
    evidenced real-world impact.
  - Report `counterpart = None` for `Process` whenever `closer` is `None`
    (the "safe," conservative option). Rejected: this would silently fail
    FR-009/SC-004 for every real implicitly-closed `Process` block — the
    common case — to guard against a sub-case with no evidenced occurrence,
    the wrong trade-off given `Run`'s own resolution shows the implicit-close
    case is real and worth reporting correctly.

## 11. Dependency versions and currency check (confirmed 2026-08-09)

| Crate | Confirmed version | Role | Notes |
|---|---|---|---|
| `lsp-server` | 0.10.0 (published 2026-07-16) | §3 | Actively maintained, rust-lang-org-affiliated, 1M+ monthly downloads, 213 dependents. |
| `lsp-types` | 0.97.0 (published 2024-06-04) | §3 | Latest stable is genuinely ~2 years old; `PositionEncodingKind` is present and unfeature-gated regardless (confirmed via docs.rs). Missing LSP 3.18-only additions are outside this phase's scope (§3 Alternatives). |

- **Standing recommendation, not a one-time check**: same as
  `002-cli-check-format/research.md` §6 — once `drut-lsp`'s (and the
  extension's `package-lock.json`'s) dependency trees exist, wire
  `cargo audit`/`cargo deny check advisories` and `npm audit` into CI so
  advisories filed after this pass surface automatically, rather than
  requiring another manual pass like this one.
- **Re-confirm exact pins at implementation time** via `cargo add`/
  `cargo update` and `npm install`, since neither `Cargo.lock` (for
  `drut-lsp`) nor `package-lock.json` (for the extension) exists yet.

## 12. `InvalidEncoding` reachability through live document editing — RESOLVED: unreachable, and this is correct, not a gap

**Problem found**: FR-005's original wording promised the server would
publish diagnostics for "all seven categories," including `InvalidEncoding`,
and Story 2 Acceptance Scenario 4 described a live-editing scenario
triggering it. Neither is actually possible given how `drut-lsp` receives
document content. Found via `/speckit-checklist` (CHK001, CHK028).

**Investigation**: Two independent facts, each separately confirmed, combine
to make this unreachable:

1. **LSP's `textDocument/didOpen`/`didChange` transport itself cannot carry
   invalid byte sequences.** The `TextDocumentItem.text` field is a JSON
   string. JSON strings are Unicode text by definition — there is no way to
   place a genuinely undecodable byte sequence inside one; the message
   envelope requires UTF-8 ([LSP specification, base protocol
   section](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/):
   "The content part of messages... defaults to UTF-8, which is the only
   encoding supported right now"). Whatever `drut-lsp` receives via
   `didOpen`/`didChange` is, structurally, always valid Unicode text.
2. **VS Code's own file-reading has already run before the server ever sees
   content.** VS Code decodes a file to text using its own encoding
   detection *before* handing that text to any language server — confirmed
   via VS Code's own encoding-detection issue history (e.g.
   [microsoft/vscode#79675](https://github.com/microsoft/vscode/issues/79675),
   documenting real cases of VS Code's heuristic guessing the wrong encoding
   for a file). Two sub-cases, both still producing valid Unicode text by
   the time `drut-lsp` sees it:
   - VS Code guesses the file's actual encoding correctly → the server
     receives a faithful decode, with no encoding anomaly to report at all.
   - VS Code guesses wrong, or the byte is genuinely undecodable under any
     encoding → VS Code's decoding uses the standard, non-fatal
     `TextDecoder`-style behavior (`fatal: false` is the default per the
     web-platform decoding API VS Code's Electron/Node.js runtime is built
     on), which substitutes the Unicode replacement character (U+FFFD) for
     any byte sequence it can't decode, rather than erroring
     (confirmed via MDN's `TextDecoder` documentation and general
     JavaScript/web-platform decoding behavior). U+FFFD is itself a valid,
     ordinary Unicode code point — `voyager-core`'s `parse()` sees it as any
     other character, not as a trigger for `InvalidEncoding`, since that
     diagnostic is specifically about `voyager-core`'s *own* decode fallback
     inside `parse_bytes()` (per `001-voyager-script-parser/contracts/
     public-api.md`: "A byte undecodable under either encoding becomes an
     `InvalidEncoding` diagnostic (only reachable via `parse_bytes`...)"),
     which `drut-lsp` never calls for live document content (data-model.md
     §2 — always `parse()`, since `text: String` is already guaranteed
     valid UTF-8 by Rust's own type system).

Both facts independently guarantee the same conclusion: by the time
`OpenDocument.text` exists, any encoding ambiguity in the original file has
already been resolved (well or badly) by the editor, and `voyager-core`'s
own `InvalidEncoding`-producing code path is never invoked on it.

**Decision**: `InvalidEncoding` is explicitly out of scope for `drut-lsp`'s
live-editing diagnostics (FR-005's carve-out). `drut-lsp` always calls
`parse()`, never `parse_bytes()`, on `OpenDocument.text` — not as a policy
choice requiring runtime filtering, but because `parse_bytes()`'s raw-byte
input type (`&[u8]` with potentially-invalid content) has no natural source
in this crate at all; there is no raw-byte value anywhere in `drut-lsp`'s
data flow to pass it. `InvalidEncoding` remains fully available through
`drut check` (`002-cli-check-format`), which reads a file's raw bytes
directly from disk with no LSP transport in between.

**Rationale for not working around this**: The alternative — `drut-lsp`
bypassing the editor-supplied `text` and re-reading the document's raw bytes
from disk via its URI, specifically to run `parse_bytes()`'s encoding
check — was considered and rejected:

- **It could report a diagnostic that contradicts what's on screen.** The
  editor's own encoding guess and `voyager-core`'s Windows-1252 fallback
  guess are two independent heuristics with no guarantee of agreeing. If
  they disagree, `drut-lsp` would report an `InvalidEncoding` diagnostic (or
  a "recovered" position) that doesn't correspond to what the user actually
  sees in front of them — actively more confusing than reporting nothing.
- **It only works for a document that is actually saved to a real file
  path.** An untitled/unsaved buffer, or any virtual document, has no raw
  bytes on disk to re-read at all — this would make the capability silently
  inconsistent (works for some open documents, not others) for a single
  diagnostic category.
- **It reintroduces the exact "diagnostics reflect stale disk content, not
  the live buffer" problem this spec's Edge Cases already explicitly rule
  out** for every other diagnostic category (FR-002, FR-007's "for the same
  text" wording) — carving out one diagnostic category to work differently,
  silently, would be a real, confusing inconsistency in the server's own
  model of what "current document state" means.
- **No real evidence motivates the cost.** `001-voyager-script-parser`'s
  full-corpus validation (161 real files) found exactly one non-UTF-8 byte
  across the entire corpus, and it was the *recoverable* Windows-1252 case,
  not the genuinely-undecodable case `InvalidEncoding` reports — meaning
  even the milder case is rare, and the specific case this workaround would
  restore has zero recorded real-world occurrences at all (only a
  hand-written fixture exists to exercise it). Investing in a
  correctness-risking mechanism for a scenario with no evidenced practical
  impact is disproportionate.

**Alternatives considered**:

- **Re-read raw bytes from disk via the URI.** Rejected — see Rationale
  above.
- **Silently filter `InvalidEncoding` out of `parse_bytes()`'s output if
  drut-lsp ever did call it.** Not applicable — moot, since `drut-lsp` never
  has raw bytes to call `parse_bytes()` with in the first place; there would
  be nothing to filter.
- **Leave FR-005/Story 2 AS4 as originally written and treat this as an
  acceptable, silent gap.** Rejected — this is exactly the kind of
  requirement-that-cannot-be-satisfied-as-written this project's spec-kit
  gates (`/speckit-checklist`, `/speckit-analyze`) exist to catch before
  implementation, not after a failing test reveals it.

**Sources**: [LSP 3.17 specification, base protocol](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/) (UTF-8-only message content requirement); [microsoft/vscode#79675](https://github.com/microsoft/vscode/issues/79675) (real-world VS Code encoding-detection failure reports); [MDN `TextDecoder`](https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder) (non-fatal decoding substitutes U+FFFD by default).
