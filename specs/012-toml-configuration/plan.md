# Implementation Plan: TOML-Based Configuration

**Branch**: `012-toml-configuration` | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/012-toml-configuration/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Add a `drut.toml` project configuration file, discovered per file via upward directory
walk-up (stopping at the nearest file, a `.git` boundary, or the filesystem root), that
lets `casing`/`top_level_indent` be set once and reached identically from the CLI, the
LSP server, and the MCP `format` tool — closing the current gap where `drut-lsp` has
zero configuration surface at all. A new `drut-config` crate owns parsing, discovery,
and per-field merge/precedence (explicit override > resolved file value > built-in
default); `voyager-core` is untouched and gains no dependency, preserving its
zero-runtime-dependency constraint (FR-027). A malformed file never blocks the
requested operation — it falls back per-field and surfaces a visible, non-blocking
notice on whichever surface encountered it (spec.md FR-011, resolved directly against
constitution Principle IV during spec review).

## Technical Context

**Language/Version**: Rust 2021 edition (matches every other crate in this workspace).

**Primary Dependencies**: `toml` (pinned major `"1"`, verified current on crates.io as
of this plan — 1.1.2+spec-1.1.0) and `serde` `1.0.229` (already used identically by
`drut-cli`/`drut-mcp`) for the new `drut-config` crate. No new dependency touches
`voyager-core`, whose zero-runtime-dependency constraint (FR-027) is a hard, crate-
scoped rule confirmed directly in `drut-cli`'s and `drut-mcp`'s own `Cargo.toml`
comments ("Not bound by voyager-core's zero-dependency rule — that constraint is
scoped to the core crate specifically").

**Storage**: A project-authored `drut.toml` file on disk, read fresh at resolution
time (per file/request) — never cached across unrelated runs, matching FR-011's
"read fresh" Key Entity note. No new persistent state elsewhere, with one exception:
`drut-lsp`'s `ServerState` gains a new, small piece of session state — the client's
workspace root (captured once at `initialize`) — needed for the untitled-buffer
fallback case (research.md §5).

**Testing**: `cargo test -p drut-config` (new crate: discovery, parsing, per-field
merge, all three malformed-file categories), plus adapter-level tests in each of
`drut-cli`, `drut-lsp`, `drut-mcp` proving identical resolution for the same file
across all three surfaces (spec.md US1's own Independent Test, made concrete).

**Target Platform**: Cross-platform (Windows/macOS/Linux) — matches every existing
crate; `.git`-boundary detection and path walk-up must both behave correctly under
Windows path semantics (drive letters, backslashes), not just POSIX paths, since this
repository's own primary development environment is Windows.

**Project Type**: New crate (`drut-config`) plus adapter-layer changes to the three
existing binaries/libraries (`drut-cli`, `drut-lsp`, `drut-mcp`). No change to
`voyager-core`'s own public surface.

**Performance Goals**: Per-file discovery is a bounded directory walk (at most as many
`stat`-equivalent calls as there are ancestor directories to a repository root) —
matches the same "cheap enough to do per-request" performance posture every other
per-document `drut-lsp` capability already relies on (diagnostics, hover, folding all
re-derive their own state per request with no reported latency issue).

**Constraints**: `voyager-core`'s zero-runtime-dependency rule (FR-027) — resolved by
keeping all TOML parsing and file-discovery logic in the new `drut-config` crate,
never in `voyager-core`, which continues to receive only an already-resolved
`FormatOptions` value exactly as it does today. FR-011's "never blocks, never silent"
requirement — resolved via per-field fallback plus a structured, adapter-surfaced
warning type (research.md §4).

**Scale/Scope**: One new crate (`drut-config`, comparable in size to `drut-cli`'s
existing `traverse.rs` module) plus small, contained changes to 5 existing files
across three crates (`cli.rs`, `format_cmd.rs`, `lib.rs` in `drut-cli`; `lib.rs`,
`formatting.rs`, `range_formatting.rs`, `document_store.rs` in `drut-lsp`; `format.rs`
in `drut-mcp`) — no change to `voyager-core` at all.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Single Source of Truth)**: PASS. `drut-config` owns configuration
  parsing/discovery/merge — a genuinely new concern, not grammar/parsing/lint-rule
  logic, so it does not belong in `voyager-core` by Principle I's own terms, and
  Principle I is precisely why this logic is centralized in one new shared crate
  rather than duplicated three times across CLI/LSP/MCP (the same rationale that
  drove `block_resolution.rs`'s extraction in `004`). `voyager-core` itself performs
  no configuration-aware behavior of its own — it only ever receives an already-
  resolved `FormatOptions`, unchanged from today.
- **Principle II (No Verbatim Vendor Docs)**: N/A — no vendor-doc-derived content;
  Ruff's own public documentation was researched for structural conventions only
  (file discovery, table nesting, precedence), not copied, and Ruff is a comparable
  open-source tool, not a Cube Voyager vendor.
- **Principle III (Formatter Idempotence)**: PASS by construction — this feature
  changes *which* `FormatOptions` value reaches `voyager_core::format`, never
  `format`'s own behavior given a value. Idempotence is unaffected; a config-resolved
  `FormatOptions` is exercised through the exact same golden-fixture corpus tests
  `008`/`009`/`010` already established, no new formatter logic to verify.
- **Principle IV (False Negatives Over False Positives)**: Directly load-bearing here
  — FR-011's "never block, always warn" resolution *is* this principle applied to
  configuration-file handling, confirmed explicitly during spec review (see spec.md's
  Assumptions and FR-011's own citation).
- **Principle V (Vertical Increments)**: PASS — one independently-shippable
  increment; each user story (shared config reachable everywhere → explicit
  override → isolation escape hatch) is independently testable and valuable on its
  own, per spec.md's own priority ordering.
- **Principle VI (LSP-Standard Mechanisms)**: N/A directly — this feature does not add
  a new LSP capability; it changes what `FormatOptions` value flows into the two
  already-registered formatting capabilities (`textDocument/formatting`,
  `textDocument/rangeFormatting`). Malformed-config surfacing on the LSP side will
  reuse LSP-standard diagnostics/notification mechanisms already established by
  `010` (`DiagnosticSeverity`, distinct `source` tag), not any editor-proprietary
  channel — consistent with this principle's spirit even though it isn't the
  primary axis of this feature.
- **Principle VII (Naming Honesty)**: PASS — "configuration file," "discovery,"
  "override" are used in their ordinary, unoverclaimed sense throughout.
- **Principle VIII (Public/Private Boundary)**: N/A — no vendor-documentation-derived
  content involved.

No violations; Complexity Tracking table is not needed.

## Project Structure

### Documentation (this feature)

```text
specs/012-toml-configuration/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
Cargo.toml                          # + "crates/drut-config" workspace member

crates/
├── drut-config/                    # NEW crate
│   ├── Cargo.toml                  #   deps: toml "1", serde "1.0.229", voyager-core (path)
│   └── src/
│       ├── lib.rs                  #   DrutConfig, FormatConfig, ConfigWarning,
│       │                           #   ExplicitFormatOverride, resolve_format_options()
│       ├── discover.rs             #   per-file upward walk-up, .git-boundary stop
│       └── parse.rs                #   toml::Value-level parse for per-field fallback
│   └── tests/
│       └── (discovery, parsing, precedence, all three malformed-file categories)
│
├── voyager-core/                   # UNTOUCHED — no dependency added, no code change
│
├── drut-cli/
│   └── src/
│       ├── cli.rs                  # + --isolated; top_level_indent becomes Option<...>
│       ├── format_cmd.rs           # resolve per-file (inside the traversal loop,
│       │                           #   not once before it), print config warnings
│       └── lib.rs                  # thread the new flag through
│
├── drut-lsp/
│   └── src/
│       ├── lib.rs                  # capture workspace root from InitializeParams
│       ├── document_store.rs       # + ServerState.workspace_root: Option<PathBuf>
│       ├── formatting.rs           # resolve config from the document's own URI
│       └── range_formatting.rs     # same resolution, same handler shape
│
└── drut-mcp/
    └── src/
        └── format.rs               # + top_level_indent, isolated optional params;
                                      #   resolve from ScriptSource's path, if any
```

**Structure Decision**: Matches this repo's existing structure exactly (constitution
Principle I) — the one new shared concern (configuration parsing/discovery/merge)
gets its own new crate, exactly the same shape `block_resolution.rs`'s extraction
into `voyager-core` took for a comparable "two-or-more-adapters-need-this-exact-
logic" problem in `004`, except this logic cannot live in `voyager-core` itself
(FR-027's zero-dependency constraint), so it becomes a new crate the three adapters
each depend on instead. No change to `voyager-core`'s own source at all.

## Complexity Tracking

Not applicable — no Constitution Check violations (see above).
