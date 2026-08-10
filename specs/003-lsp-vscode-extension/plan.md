# Implementation Plan: Drut LSP Server & VS Code/Open VSX Extension

**Branch**: `003-lsp-vscode-extension` | **Date**: 2026-08-09 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-lsp-vscode-extension/spec.md`

## Summary

Add a `server` subcommand to the existing `drut` binary that speaks the Language
Server Protocol, implemented in a new library crate (`drut-lsp`) that is a thin
adapter over `voyager-core`: it re-derives diagnostics (six of `voyager-core`'s
seven categories — `InvalidEncoding` is unreachable through live editing and
stays CLI-only, research.md §12), hover (block-kind/matched-counterpart info),
completion and "did you mean" spell-check (both against a new, real-usage-derived
`voyager-core::keywords` dictionary), and semantic tokens (short-IF vs block-IF,
unreachable-after-`BREAK`) purely from `voyager-core`'s existing `parse`/
`Statement`/`Block` output — no grammar logic duplicated in the adapter. Three
engineering questions research.md was explicitly tasked with resolving are all
settled there, not deferred again: position encoding is owned entirely by
`drut-lsp` at its boundary (`voyager-core`'s `Span` is unchanged); keyword
completion is fully control-word-scoped this phase (not the general-list
fallback) — scoped by which control word (`RUN`, `LOOP`, etc.) encloses the
cursor, never by a `PGM=` value, so this stays inside
`001-voyager-script-parser` FR-019's per-program-box-knowledge boundary — by
re-using the same `parse` call diagnostics already require to find the
enclosing statement's control word; and `InvalidEncoding`'s live-editing
unreachability is a structural fact of the LSP transport itself, not a gap
(research.md §12). A
companion `editors/vscode/` TypeScript extension provides a static TextMate grammar
(functional with no server running) plus a `vscode-languageclient` wrapper that
spawns `drut server`, published to both the VS Code Marketplace and Open VSX under
Drut's own publisher identity.

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition (`drut-lsp`, and the new
`voyager-core::keywords` module) — matches the rest of the workspace. TypeScript
(current stable) for the `editors/vscode/` extension, matching the ecosystem
`vscode-languageclient` itself targets.

**Primary Dependencies**:
- `lsp-server` — synchronous JSON-RPC/stdio transport scaffold (research.md §3).
  Keeps this feature on the same all-synchronous architecture as `voyager-core`/
  `drut-cli` rather than introducing an async runtime for the first time.
- `lsp-types` — typed LSP protocol structs, including `PositionEncodingKind`
  (research.md §1/§3).
- `voyager-core` (path dependency, existing workspace member) — supplies `parse`
  (unchanged; `drut-lsp` never calls `parse_bytes`, research.md §12) for
  diagnostics/hover/completion-context, plus the new `keywords` module
  (research.md §4) this feature adds to it for completion/spell-check candidates
  and fuzzy matching.
- `vscode-languageclient` (npm) — the extension's LSP client wrapper (research.md
  §7).
- Dev/build-only: `@vscode/vsce` (VS Code Marketplace packaging) and `ovsx`
  (Open VSX publishing) — research.md §7.

None of the above touches Voyager grammar, parsing, hover-fact, or completion-
candidate *decision* logic — see Constitution Check, Principle I row.

**Storage**: N/A for `drut-lsp` beyond in-memory open-document state (URI → live
text + derived `ParseResult`, held only for the lifetime of the editor session,
discarded on `textDocument/didClose`); no persistence across server restarts.

**Testing**:
- `cargo test -p voyager-core` — extended with unit tests for the new `keywords`
  module (dictionary lookup, completion-candidate scoping, fuzzy-match threshold
  behavior, research.md §4/§5) at the same layer `parse_bytes`'s own correctness is
  proven, per Principle I.
- `cargo test -p drut-lsp` — an LSP-level test suite driving the server through
  real JSON-RPC messages via `lsp_server::Connection::memory()` (research.md §9),
  including a full-corpus run reproducing FR-028's Definition of Done (same
  161/161-clean, all-broken-fixtures-flagged standard as `voyager-core` and
  `drut-cli`).
- `editors/vscode/`: TypeScript unit tests for the static grammar/language-
  configuration (tokenization spot-checks) plus quickstart.md's manual end-to-end
  validation steps for the packaged extension itself — see quickstart.md for why
  deep automated VS Code UI testing is out of scope this phase.

**Target Platform**: Cross-platform (Windows/macOS/Linux) for `drut-lsp`/`drut`,
same as the rest of the workspace. The extension targets VS Code and other Open
VSX-compatible editors (spec.md Assumptions) on the same three OSes.

**Project Type**: Library crate (`drut-lsp`) wired into the existing `drut-cli`
binary as a new subcommand, plus a separate non-Rust editor-extension package.

**Performance Goals**: Diagnostics/hover/completion/semantic-token responses feel
"perceptibly-immediate" after an edit (spec.md SC-003) — no user-visible lag on
ordinary developer hardware. `001-voyager-script-parser/plan.md` already
establishes `voyager-core` parses a large real file (~980 lines) in low tens of
milliseconds; since `drut-lsp` re-parses the whole open document on every change
(research.md §2's resolved decision, not incremental parsing), this budget carries
over directly as this phase's own responsiveness ceiling for a single document.

**Constraints**:
- MUST NOT duplicate any grammar, parsing, hover-fact, or completion-candidate
  decision logic in `drut-lsp` (Principle I, FR-003) — every one of those is
  delegated to `voyager-core::parse`/`keywords`.
- MUST NOT panic on any document content, including malformed text (FR-004;
  raw non-UTF-8 bytes are not a reachable input to `drut-lsp` at all, per
  research.md §12 — this guarantee concerns malformed-but-valid-Unicode text).
- Every position sent over the wire MUST be UTF-16-code-unit-based, 0-based
  (FR-019/FR-020) — enforced by the single translation function research.md §1
  specifies, not scattered ad hoc conversions.
- The extension MUST deliver Story 1 (static highlighting) with zero dependency on
  `drut server` being reachable (FR-021, FR-025).
- Semantic tokens and all other editor-facing capabilities MUST use LSP-standard
  mechanisms over VS Code-proprietary APIs wherever both achieve the same result
  (Principle VI, research.md §6).

**Scale/Scope**: Same 161-file WF-TDM-Official-Releases fixture corpus as
`001-voyager-script-parser`/`002-cli-check-format`, exercised through the LSP
protocol layer (research.md §9) rather than only the library or CLI layer; single
open document per diagnostics/hover/completion/semantic-token request (no
multi-file/workspace-wide analysis, per spec.md's explicit out-of-scope list).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS, with a design decision worth flagging** | `drut-lsp` adds zero grammar/parsing logic of its own — diagnostics and hover come straight from `parse`'s `ParseResult`/`Block`, and completion-context resolution reuses the same `ParseResult.nodes` (a recursive walk, not a flat `.statements` list — see data-model.md §4's Correction note) rather than re-deriving statement boundaries (research.md §2). The keyword dictionary + fuzzy-match logic (completion/spell-check's actual "what counts as a real keyword" decision) is added to `voyager-core` in this same feature, not `drut-lsp`, for the same reason `002-cli-check-format` put formatting-decision logic in `voyager-core` rather than the CLI — see research.md §4. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | The new `keywords` dictionary is hand-written from the FR-012 corpus census, not vendor documentation. The extension's grammar/bracket-config may structurally reference `bhereth.language-citilabscubevoyager` under the constitution's already-granted, binding-conditioned permission (FR-023, research.md §8) — his files are never committed, and anything ported is rewritten in Drut's own wording. |
| III. Formatter Idempotence & Behavior Preservation | **N/A this phase** | No formatter changes; `drut-lsp` never writes document content back to disk. |
| IV. False Negatives Over False Positives | **PASS** | `drut-lsp` introduces no new diagnostic categories — it surfaces six of `voyager-core`'s existing seven unchanged (FR-005/FR-007; `InvalidEncoding` is structurally unreachable through live editing, research.md §12, not a false negative introduced by this feature — it remains fully available via `drut check`). The spell-check "did you mean" nudge (research.md §5) is deliberately conservative (unique-minimum-within-threshold only) so it never asserts a confident wrong suggestion, consistent with this principle's spirit even though it isn't a `Diagnostic`. |
| V. Vertical, Independently-Usable Increments | **PASS** | Each of spec.md's six user stories is independently testable and independently valuable (Story 1 alone — static highlighting — needs no server at all). This phase does not start until `002-cli-check-format`'s fixture-corpus tests pass cleanly (already true). |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **PASS — this is the phase that instantiates it for real** | Diagnostics, hover, completion, and semantic tokens are all delivered via standard LSP capabilities; semantic tokens specifically were evaluated against a VS Code-proprietary decorations-API alternative and rejected in favor of the standard mechanism (research.md §6). |
| VII. Naming Honesty | **PASS** | The feature is named and scoped as an "LSP server + editor extension," not oversold as e.g. a "language intelligence" or "type checker" — hover/completion/semantic tokens are named for exactly the LSP capability they use, and FR-012's completion explicitly documents its own context-awareness scope rather than implying more than it delivers. |
| VIII. Public/Private Boundary | **PASS** | `drut-lsp` and `editors/vscode/` are both public, per the constitution's own Technology & Architecture Constraints list naming "an LSP server" and "an editor extension client" as public adapters. No vendor-documentation-corpus content is read, generated, or linked into either. |

No unjustified violations. Complexity Tracking is empty (see below).

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/quickstart.md): The
design confirms the Principle I split above — `contracts/lsp-capabilities.md` and
`contracts/keyword-dictionary-api.md` show `drut-lsp` itself defines no grammar
terms beyond what it passes through from `voyager-core`'s existing `Diagnostic`/
`Block`/`Statement` types plus the new `keywords` module's own types.
`contracts/position-encoding.md` codifies research.md §1's translation contract as
the single place that logic lives, rather than leaving it implicit across handler
code. No row's status changed from the pre-design check above.

## Project Structure

### Documentation (this feature)

```text
specs/003-lsp-vscode-extension/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output (/speckit-plan command)
├── data-model.md                   # Phase 1 output (/speckit-plan command)
├── quickstart.md                   # Phase 1 output (/speckit-plan command)
├── contracts/                      # Phase 1 output (/speckit-plan command)
│   ├── lsp-capabilities.md          # drut-lsp's ServerCapabilities surface
│   │                                 # (diagnostics/hover/completion/semantic
│   │                                 # tokens) and request/notification handling
│   ├── keyword-dictionary-api.md     # new voyager-core::keywords entry points
│   ├── position-encoding.md          # the research.md §1 translation contract
│   └── extension-manifest.md          # editors/vscode/package.json contribution
│                                     # points (language, grammar, client wiring)
├── checklists/
│   └── requirements.md             # already created by /speckit-specify
└── tasks.md                        # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
Cargo.toml                          # workspace manifest (existing); add
                                     # "crates/drut-lsp" as a third member

crates/
├── voyager-core/                   # existing crate; this feature ADDS to it
│   ├── src/
│   │   ├── lib.rs                    # add: pub mod keywords; re-export
│   │   │                                # KeywordEntry/CompletionContext/
│   │   │                                # completion_candidates/did_you_mean
│   │   └── keywords.rs                # NEW: FR-012 dictionary (hand-written,
│   │                                  # corpus-census-derived — no new runtime
│   │                                  # dependency, research.md §4/§5) +
│   │                                  # Damerau-Levenshtein fuzzy match
│   └── tests/
│       └── keywords.rs                # NEW: dictionary lookup, context
│                                      # scoping, fuzzy-match threshold tests
│
├── drut-cli/                       # existing crate; this feature ADDS a
│   │                                # subcommand, does not restructure it
│   └── src/
│       ├── cli.rs                    # add: Command::Server (no flags —
│       │                             # FR-001)
│       └── server_cmd.rs              # NEW: thin dispatch — calls
│                                      # drut_lsp::run() over real stdio; zero
│                                      # LSP protocol logic here (Principle I)
│
└── drut-lsp/                       # NEW: this feature's main deliverable
    ├── Cargo.toml                    # package "drut-lsp"; library crate;
    │                                # depends on voyager-core (path) +
    │                                # lsp-server + lsp-types
    ├── src/
    │   ├── lib.rs                     # pub fn run(connection: Connection) —
    │   │                                # entry point drut-cli's server_cmd.rs
    │   │                                # and this crate's own tests both call
    │   ├── document_store.rs           # open-document tracking: URI -> live
    │   │                                # text + cached ParseResult, updated on
    │   │                                # didOpen/didChange/didClose (FR-002)
    │   ├── position.rs                 # research.md §1's translation contract:
    │   │                                # voyager-core Position <-> lsp_types
    │   │                                # Position (UTF-16, 0-based)
    │   ├── diagnostics.rs               # FR-005-FR-007: ParseResult
    │   │                                # .diagnostics -> publishDiagnostics
    │   ├── hover.rs                     # FR-008-FR-011: Block lookup by
    │   │                                # cursor position -> hover contents
    │   ├── completion.rs                # FR-012-FR-013: enclosing-Statement
    │   │                                # lookup (research.md §2) ->
    │   │                                # voyager_core::keywords candidates
    │   ├── spellcheck.rs                 # FR-014-FR-015: did_you_mean nudges,
    │   │                                # surfaced as non-Diagnostic hints
    │   │                                # (research.md §5)
    │   └── semantic_tokens.rs            # FR-016-FR-018: short-IF/block-IF +
    │                                    # unreachable-after-BREAK legend and
    │                                    # token encoding (research.md §6)
    └── tests/
        ├── protocol_smoke.rs            # Connection::memory() basic
        │                                # initialize/didOpen round trip
        ├── diagnostics_corpus.rs         # FR-028/DoD: full-corpus diagnostic
        │                                # parity with drut-cli's check
        ├── hover.rs                      # Story 3 acceptance scenarios
        ├── completion.rs                 # Story 4 acceptance scenarios,
        │                                # including context-aware scoping
        ├── spellcheck.rs                  # Story 5 acceptance scenarios
        ├── semantic_tokens.rs             # Story 6 acceptance scenarios
        └── position_encoding.rs           # FR-019/FR-020: supplementary-plane
                                           # character position correctness

editors/
└── vscode/                         # NEW: this feature's other deliverable —
    │                                # not a Cargo workspace member
    ├── package.json                  # extension manifest: language
    │                                # registration, grammar/config paths,
    │                                # semanticTokenTypes/Scopes contributions,
    │                                # publisher identity (FR-027)
    ├── language-configuration.json    # brackets/comment-toggling (FR-022)
    ├── syntaxes/
    │   └── drut.tmLanguage.json        # static TextMate grammar (FR-021) —
    │                                  # structure only, own wording (FR-023)
    ├── src/
    │   └── extension.ts                # activation: vscode-languageclient
    │                                  # setup, spawns `drut server` (FR-024),
    │                                  # missing-binary/crash handling
    │                                  # (FR-025/FR-026)
    └── test/
        └── grammar.test.ts              # tokenization spot-checks for the
                                         # static grammar (Story 1)
```

**Structure Decision**: Add `drut-lsp` as a third Cargo workspace member under
`crates/`, alongside `voyager-core` and `drut-cli` — same rationale
`002-cli-check-format/plan.md` already gave for `drut-cli`'s placement (one place
for every workspace member). `drut-lsp` is a *library* crate, not its own binary:
the user-facing entry point stays the single `drut` executable (`drut server`,
FR-001), with `drut-cli` gaining only a thin dispatch module
(`server_cmd.rs`) that calls into `drut-lsp`, keeping the "LSP server" adapter the
constitution names as its own component architecturally distinct (own crate, own
dependency graph, own test suite) while still shipping as one binary. The VS Code/
Open VSX extension is not a Cargo crate at all and lives in a new top-level
`editors/vscode/` directory (research.md §7), keeping the non-Rust component
clearly out of the Cargo workspace rather than awkwardly nested under `crates/`.

## Complexity Tracking

*No entries — Constitution Check reported no unjustified violations.*
