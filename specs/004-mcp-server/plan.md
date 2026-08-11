# Implementation Plan: Drut MCP Server

**Branch**: `004-mcp-server` | **Date**: 2026-08-10 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/004-mcp-server/spec.md`

## Summary

Add a `mcp` subcommand to the existing `drut` binary that speaks the Model
Context Protocol over stdio, implemented in a new library crate (`drut-mcp`)
built on the official `rmcp` SDK (research.md §1). Exposes four read-only
tools — `diagnose`, `format`, `query_structure`, `lookup_keyword` — each a
thin wrapper over an existing `voyager-core` entry point
(`parse`/`parse_bytes`, `format`/`format_bytes`, `completion_candidates`/
`did_you_mean`) with zero new grammar/parsing/lint-rule logic of its own. One
piece of existing logic moves rather than gets duplicated: `drut-lsp`'s hover
capability's 5-rule block/counterpart derivation, currently private to that
one adapter, is extracted into `voyager-core` itself as `block_at`
(research.md §5) so both `drut-lsp`'s hover and this feature's
`query_structure` tool call the identical implementation. `tokio` (required
by every viable Rust MCP SDK, research.md §1/§2) is accepted as a dependency
scoped entirely to the new `drut-mcp` crate — `voyager-core` stays at zero
runtime dependencies, and `drut-cli`'s other subcommands remain fully
synchronous.

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition — matches the rest
of the workspace.

**Primary Dependencies**:
- `rmcp` (official SDK, `modelcontextprotocol/rust-sdk`, pinned ~3.1, research.md
  §1) with `default-features = false`, features
  `["server", "macros", "transport-io", "schemars"]` only — deliberately
  excluding every HTTP-transport feature (research.md §4's RUSTSEC-2026-0189
  finding: not applicable to the pinned version or to stdio transport, but
  excluded anyway as defense in depth against ever compiling that code path
  in at all).
- `tokio` (transitive via `rmcp`, research.md §2) — scoped to `drut-mcp` only;
  `drut-cli` constructs a runtime locally inside its new `mcp` subcommand's
  dispatch arm, the same "thin dispatch, zero protocol logic" shape
  `server_cmd.rs` already established for `drut server`.
- `schemars` (research.md §3) — derives JSON Schema for each tool's input
  struct from the struct definition itself, never hand-written separately.
- `serde`/`serde_json` — `drut-mcp`'s own result DTOs (data-model.md §2–§6)
  serialize independently of `voyager-core`'s native types, which gain no new
  dependency (research.md §6).
- `voyager-core` (path dependency, existing workspace member) — supplies
  `parse`/`parse_bytes`, `format`/`format_bytes`, `completion_candidates`/
  `did_you_mean` unchanged, plus the new `block_at` entry point this feature
  adds to it (research.md §5, contracts/block-resolution-api.md).

None of the above touches Voyager grammar, parsing, hover-fact, or
completion-candidate *decision* logic — see Constitution Check, Principle I
row.

**Storage**: N/A — every tool is a pure function of its input for the
duration of one call; no state persists across calls, no document/session
tracking (unlike `drut-lsp`'s `ServerState`, which exists specifically
because LSP is a stateful, open-document protocol — MCP tool calls here are
each self-contained, per spec.md's single-document-per-call scope).

**Testing**:
- `cargo test -p voyager-core` — extended with unit tests for the new
  `block_at` entry point (contracts/block-resolution-api.md) at the same
  layer every other `voyager-core` entry point is proven, per Principle I.
- `cargo test -p drut-lsp` — every pre-existing hover test re-run unchanged
  after the extraction (quickstart.md step 2), proving the refactor altered
  nothing observable.
- `cargo test -p drut-mcp` — per-tool unit/contract tests
  (contracts/mcp-tools.md), a dedicated read-only-filesystem test proving
  FR-010/SC-005, a full-corpus diagnostic-parity run against `drut check`
  (SC-006), and a structural-query parity check against `drut-lsp`'s own
  hover output for real corpus positions (quickstart.md steps 3–6).

**Target Platform**: Cross-platform (Windows/macOS/Linux), same as the rest
of the workspace.

**Project Type**: Library crate (`drut-mcp`) wired into the existing
`drut-cli` binary as a new subcommand, plus one new `voyager-core` public
entry point.

**Performance Goals**: Each tool call completes with no user-visible lag on
ordinary developer hardware — the same "perceptibly-immediate" bar
`003-lsp-vscode-extension/plan.md` set for LSP responses, applicable here
too since every tool does the same order-of-magnitude work (one parse, one
format pass, or one dictionary lookup) `voyager-core` already proved fast at
`001-voyager-script-parser/plan.md`'s scale (a ~980-line real file in low
tens of milliseconds).

**Constraints**:
- MUST NOT duplicate any grammar, parsing, formatting-decision, or
  block/counterpart-derivation logic in `drut-mcp` (Principle I) — every one
  of those is delegated to `voyager-core`, including the newly-extracted
  `block_at`.
- MUST NOT write to disk under any tool call, any input combination (FR-010).
- MUST NOT panic on any input, including malformed script content or a
  missing/unreadable file path (FR-012) — matching `voyager-core`'s own
  crate-wide no-panic guarantee.
- `voyager-core` MUST remain at zero runtime dependencies (FR-027,
  constitution Principle I) — `tokio`/`rmcp`/`schemars`/`serde` all stop at
  `drut-mcp`'s own boundary.
- MUST NOT require a running `drut server`/LSP session for any tool to
  function (FR-011) — `drut-mcp` depends on `voyager-core` directly, never on
  `drut-lsp` or `drut-cli`'s other subcommands.

**Scale/Scope**: Same 161-file WF-TDM-Official-Releases fixture corpus as
every prior phase, exercised through the `diagnose` tool for parity
validation (SC-006); single document per tool call (no multi-file/workspace
batch calls this phase, spec.md Assumptions).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS, with a design decision worth flagging** | `drut-mcp` adds zero grammar/parsing/formatting logic of its own — every tool is a thin wrapper over an existing `voyager-core` entry point. The one non-trivial move: `drut-lsp`'s hover 5-rule derivation, previously private to that adapter, is extracted into `voyager-core` as `block_at` (research.md §5) specifically so it isn't duplicated a second time for `query_structure` — the same reasoning `003-lsp-vscode-extension/plan.md` already applied when it put the `keywords` dictionary in `voyager-core` rather than `drut-lsp`. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | This feature introduces no new keyword lists, grammar rules, or help/hover text of its own — every string a tool returns (diagnostic messages, keyword names, block-kind names) already exists in `voyager-core`/`drut-lsp`'s own hand-written, real-usage-evidenced output. |
| III. Formatter Idempotence & Behavior Preservation | **PASS, unchanged** | No formatter logic changes; the `format` tool calls `voyager_core::format` exactly as `drut format`/`drut-lsp`'s own formatting capability already do, inheriting the same idempotence guarantee rather than re-proving it. |
| IV. False Negatives Over False Positives | **PASS** | `drut-mcp` introduces no new diagnostic categories — `diagnose` surfaces exactly what `parse`/`parse_bytes` already produce, unchanged. |
| V. Vertical, Independently-Usable Increments | **PASS** | Each of spec.md's four user stories is independently testable and independently valuable (P1 `diagnose` alone is a complete, useful capability with no dependency on the other three). This phase does not start until `003-lsp-vscode-extension`'s fixture-corpus tests pass cleanly (already true, merged to `main`). |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **N/A this phase** | This principle governs editor-integration choices specifically; this feature has no editor-integration surface at all (an MCP server, not an LSP capability or editor extension). |
| VII. Naming Honesty | **PASS** | Tools are named for exactly what they do — `diagnose` surfaces diagnostics (not "lint" or "validate," which would imply broader semantic checking this feature doesn't do), `query_structure` reports structural facts only (not "understand" or "analyze," which would overclaim), matching this constitution's own worked example for why "type checker" would be dishonest naming here. |
| VIII. Public/Private Boundary | **PASS** | `drut-mcp` is public, per the constitution's own Technology & Architecture Constraints list naming "an MCP server" as one of the public thin adapters. No vendor-documentation-corpus content is read, generated, or linked into it. |

No unjustified violations. Complexity Tracking has one entry (the `tokio`
dependency), justified below rather than silently accepted.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/quickstart.md):
`contracts/block-resolution-api.md` and `contracts/mcp-tools.md` confirm the
Principle I split above holds precisely — `drut-mcp` itself defines no
grammar terms beyond DTOs that convert `voyager-core`'s existing types
(research.md §6), and the block-resolution extraction is a relocation with an
explicit "byte-for-byte unchanged derivation" contract clause, not a
redesign. No row's status changed from the pre-design check above.

## Project Structure

### Documentation (this feature)

```text
specs/004-mcp-server/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output (/speckit-plan command)
├── data-model.md                  # Phase 1 output (/speckit-plan command)
├── quickstart.md                  # Phase 1 output (/speckit-plan command)
├── contracts/                     # Phase 1 output (/speckit-plan command)
│   ├── block-resolution-api.md      # new voyager-core::block_at entry point
│   └── mcp-tools.md                 # drut-mcp's four-tool MCP surface
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
Cargo.toml                          # workspace manifest (existing); add
                                     # "crates/drut-mcp" as a fourth member

crates/
├── voyager-core/                   # existing crate; this feature ADDS to it
│   ├── src/
│   │   ├── lib.rs                    # add: pub use for block_at/BlockInfo/
│   │   │                                # BlockKindName
│   │   └── block_resolution.rs        # NEW: block_at + BlockInfo/
│   │                                  # BlockKindName — the logic extracted
│   │                                  # from drut-lsp/src/hover.rs
│   │                                  # (research.md §5), unaltered
│   └── tests/
│       └── block_resolution.rs        # NEW: unit tests for block_at,
│                                      # covering the same cases
│                                      # drut-lsp/src/hover.rs's own
│                                      # (removed) private tests did
│
├── drut-lsp/                       # existing crate; this feature REFACTORS
│   │                                # it, doesn't grow its scope
│   └── src/
│       └── hover.rs                   # thinned: calls voyager_core::
│                                      # block_at, translates BlockInfo into
│                                      # lsp_types::Hover markup — private
│                                      # is_short_if/run_closed_implicitly/
│                                      # counterpart_for/find_block_at/
│                                      # find_hover_fact all removed (moved,
│                                      # not duplicated)
│
├── drut-cli/                       # existing crate; this feature ADDS a
│   │                                # subcommand, does not restructure it
│   └── src/
│       ├── cli.rs                    # add: Command::Mcp (no flags)
│       └── mcp_cmd.rs                 # NEW: thin dispatch — constructs a
│                                      # tokio::runtime::Runtime locally,
│                                      # blocks on drut_mcp::run(); zero MCP
│                                      # protocol logic here (Principle I)
│
└── drut-mcp/                       # NEW: this feature's main deliverable
    ├── Cargo.toml                    # package "drut-mcp"; library crate;
    │                                # depends on voyager-core (path) + rmcp
    │                                # (default-features = false, features =
    │                                # ["server","macros","transport-io",
    │                                # "schemars"]) + tokio + serde
    ├── src/
    │   ├── lib.rs                     # pub async fn run() -- entry point
    │   │                                # drut-cli's mcp_cmd.rs and this
    │   │                                # crate's own tests both call
    │   ├── source.rs                   # ScriptSource resolution (FR-002):
    │   │                                # text-xor-path validation, file
    │   │                                # reading
    │   ├── diagnose.rs                  # FR-003: DiagnosticsInput ->
    │   │                                # voyager_core::parse/parse_bytes ->
    │   │                                # Vec<DiagnosticDto>
    │   ├── format.rs                    # FR-004/FR-005: FormatInput ->
    │   │                                # voyager_core::format/format_bytes
    │   │                                # -> FormatResultDto
    │   ├── query_structure.rs            # FR-006/FR-007: StructuralQueryInput
    │   │                                # -> voyager_core::block_at ->
    │   │                                # BlockInfoDto
    │   └── lookup_keyword.rs             # FR-008/FR-009: KeywordLookupInput
    │                                    # -> voyager_core::keywords::
    │                                    # completion_candidates/did_you_mean
    │                                    # -> (Vec<KeywordCandidateDto>,
    │                                    # Option<SpellCheckSuggestionDto>)
    └── tests/
        ├── diagnose_contract.rs           # per-tool contract tests, one file
        │                                 # each (post-/speckit-analyze F1 fix
        │                                 # -- a single shared tool_contracts.rs
        │                                 # would make every story's [P] test
        │                                 # task falsely claim parallelism
        │                                 # against a real same-file conflict);
        │                                 # also carries the InvalidEncoding-
        │                                 # via-path case (C2 fix)
        ├── format_contract.rs             # contracts/mcp-tools.md's `format`
        │                                 # section
        ├── query_structure_contract.rs     # contracts/mcp-tools.md's
        │                                 # `query_structure` section
        ├── lookup_keyword_contract.rs      # contracts/mcp-tools.md's
        │                                 # `lookup_keyword` section
        ├── no_disk_writes.rs              # FR-010/SC-005: every tool against
        │                                # a read-only fixture directory
        ├── no_panic.rs                    # edge-case sweep across every
        │                                 # tool, mirroring drut-lsp's own
        │                                 # tests/no_panic.rs convention
        ├── diagnostics_corpus.rs          # SC-006: full-corpus parity with
        │                                # drut check
        └── structural_query_parity.rs      # query_structure vs. drut-lsp
                                            # hover, same real positions
```

**Structure Decision**: Add `drut-mcp` as a fourth Cargo workspace member
under `crates/`, alongside `voyager-core`, `drut-cli`, and `drut-lsp` — same
rationale `003-lsp-vscode-extension/plan.md` gave for `drut-lsp`'s own
placement (one place for every workspace member, one dependency-graph story).
`drut-mcp` is a *library* crate, not its own binary, for the identical reason
`drut-lsp` is one: the user-facing entry point stays the single `drut`
executable (`drut mcp`, no flags), with `drut-cli` gaining only a thin
dispatch module (`mcp_cmd.rs`) that constructs a `tokio` runtime locally and
calls into `drut-mcp` — keeping the "MCP server" adapter the constitution
names as its own architecturally distinct component (own crate, own
dependency graph including `tokio`/`rmcp`, own test suite) while still
shipping as one binary and touching zero other subcommands' startup cost.
`voyager-core` gains one new module (`block_resolution.rs`) rather than a new
crate, matching how the `keywords` module was added directly to
`voyager-core` in `003` rather than living anywhere else.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| `tokio` as a new workspace dependency (via `drut-mcp`), the project's first async runtime | Every actively-maintained Rust MCP SDK found (research.md §1) is async/tokio-based — this is a consistent, ecosystem-wide characteristic of Rust MCP tooling, not a choice avoidable by picking a different crate | Hand-rolling a synchronous JSON-RPC-over-stdio MCP transport to stay dependency-uniform with the rest of the project was considered and rejected (research.md §1) — MCP's tool-schema advertisement and content-block response shapes are meaningfully more involved to reimplement correctly from scratch than LSP's own `lsp-server` scaffold already is, for real engineering-time cost and no corresponding correctness benefit over using the protocol's own official SDK. The dependency is fully contained to `drut-mcp` alone (research.md §2) — `voyager-core` stays dependency-free and every other `drut-cli` subcommand's startup cost is unaffected. |
