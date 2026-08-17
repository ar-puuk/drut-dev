# Implementation Plan: Token Hover Shows Assigned Value

**Branch**: `016-token-hover-value` | **Date**: 2026-08-16 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/016-token-hover-value/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Extend `crates/drut-lsp/src/hover.rs` so that hovering an `@token@` reference tries
value resolution first, falling back to the existing block-info/spell-check path
(spec.md FR-008/FR-010) only when resolution finds nothing. Resolution logic itself
(finding the `@token@` at a position, finding candidate `Assignment` statements,
finding literal-path `READ FILE` statements, and picking the most-recent value under
Voyager's real interleaved execution order) is new **pure** logic added to
`voyager-core` (constitution Principle I — this is parse-tree analysis, the same
category `block_resolution.rs` already occupies, not adapter glue) in a new
`token_resolution.rs` module. The only genuinely new I/O — reading a `READ FILE`
target off disk, since it may not be an open document — happens in `drut-lsp`,
reusing `workspace::uri_to_path` (already built for `012`'s config discovery) for
the URI→path step and `voyager_core::parse` (already the crate's own single
tokenize/parse entry point) for turning that file's bytes into the same `Vec<Node>`
shape the open document already has. No new external dependency in any crate.

## Technical Context

**Language/Version**: Rust 2021 edition (matches every other crate in this workspace).

**Primary Dependencies**: None new. `voyager-core` stays dependency-free (`std`
only, per `CLAUDE.md`'s FR-027 constraint — confirmed no crate addition is needed:
file reads use `std::fs::read`, path joins use `std::path::Path`, both already used
elsewhere in `drut-lsp`, e.g. `workspace.rs`, `drut-config`).

**Storage**: N/A — no persistent state. Every hover request re-resolves from
scratch: the open document's already-cached `parse_result` (no change to how that's
produced) plus a fresh disk read + fresh `voyager_core::parse` call for any literal
`READ FILE` target, every time (spec.md's own "reads are always fresh, never
cached" Assumption — matches `013`'s established posture for config resolution).

**Testing**: `cargo test -p voyager-core` for the new pure resolution functions
(unit tests directly against `token_resolution.rs`, no LSP/protocol layer involved —
matches `block_resolution.rs`'s own existing test shape) and `cargo test -p
drut-lsp` for `hover.rs`'s new branch, including a real-filesystem test (a temp
directory with two `.s`/`.block` files, one `READ FILE`-referencing the other) since
this is the first `drut-lsp` feature that reads a file the editor never opened.

**Target Platform**: Cross-platform LSP server; VS Code used for manual
verification per constitution Principle VI (this feature adds no editor-proprietary
mechanism — it only enriches the existing standard `textDocument/hover` response).

**Project Type**: Two-crate change — `voyager-core` (new pure resolution module)
and `drut-lsp` (hover integration + the one new disk-read path). No change to
`drut-cli`, `drut-mcp`, `drut-config`, or `editors/vscode/` (this is server-side
hover content, not a client-side/grammar concern — unrelated to `002-fix-token-
highlighting`'s TextMate-grammar work on the same `@token@` syntax, which only
affects *coloring*, not hover).

**Performance Goals**: A hover request already involves a full re-parse-free lookup
against an already-parsed document (`block_at` does this today with no reported
latency issue); this feature adds at most one additional disk read + one additional
`voyager_core::parse` call (bounded by spec.md's one-level-only scope, FR-003 — never
more than one extra file per hover), imperceptible at interactive hover latency.

**Constraints**: `voyager-core` remains I/O-free (constitution Principle I,
`CLAUDE.md`'s "two pure functions operating on in-memory text only" contract) — the
new resolution functions accept already-parsed `&[Node]` for every file involved
(the open document's own, plus, per file, whatever `drut-lsp` already read and
parsed off disk), never a path or file handle. `drut-lsp` is the only place that
touches a filesystem for this feature.

**Scale/Scope**: Small-to-medium — one new `voyager-core` module (a handful of pure
functions, no new public dependency graph edges), one new `hover.rs` branch, one
new `workspace.rs`-adjacent helper reused (not duplicated) from `012`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Single Source of Truth)**: Directly load-bearing. All new
  resolution logic (finding the hovered `@token@`, finding candidate assignments,
  finding literal `READ FILE` statements, picking the most-recent value under
  Voyager's real execution order) lives in `voyager-core`, mirroring
  `block_resolution.rs`'s existing precedent exactly — `hover.rs` stays a thin
  translation into `lsp_types::Hover` markdown, adding no grammar/semantic logic of
  its own. The one piece of genuinely adapter-side work (reading a `READ FILE`
  target off disk) is I/O, not grammar logic, so it correctly belongs in the
  adapter, not the core crate.
- **Principle II (No Verbatim Vendor Docs)**: N/A — no vendor-doc-derived content;
  hover text is newly composed (e.g. "assigned `2` at line N" / "from
  `_ControlCenter.block`, line N").
- **Principle III (Formatter Idempotence)**: N/A — this feature never touches
  `format.rs` or any formatting path.
- **Principle IV (False Negatives Over False Positives)**: Applies by analogy —
  spec.md FR-008/US3 exist specifically so an unresolved or ambiguous token never
  produces a fabricated value; a missing hover is always preferred over a wrong one.
- **Principle V (Vertical Increments)**: PASS — one independently-shippable,
  independently-testable increment; spec.md's own User Story 1/2/3 split already
  separates same-file (P1) from one-level cross-file (P2) from the fallback
  guardrail (P3), each independently testable, matching this project's established
  incremental-story pattern.
- **Principle VI (LSP-Standard Mechanisms)**: PASS — no new LSP capability or
  request type; this enriches the existing standard `textDocument/hover` response
  only (spec.md Assumptions: "does not introduce a new LSP capability or request
  type").
- **Principle VII (Naming Honesty)**: PASS — this feature is named and scoped for
  exactly what it does (resolves and displays an assigned value found via a bounded,
  documented search), not oversold as general token/expression evaluation; the
  token-built-path exclusion (FR-003) is a stated, permanent boundary, not a
  temporary gap the naming glosses over.
- **Principle VIII (Public/Private Boundary)**: N/A — no vendor-documentation-derived
  content involved.

No violations; Complexity Tracking table is not needed.

## Project Structure

### Documentation (this feature)

```text
specs/016-token-hover-value/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md         # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/
├── voyager-core/
│   └── src/
│       ├── lib.rs               # + `pub mod token_resolution;` and its
│       │                        #   public re-exports, alongside the
│       │                        #   existing block_resolution re-exports
│       └── token_resolution.rs  # NEW — variable_ref_at, all_assignments,
│                                 #   read_file_refs, resolve_token_value
│                                 #   (all pure; contracts/token-resolution-api.md)
│
└── drut-lsp/
    └── src/
        ├── hover.rs              # + token-value branch, tried before the
        │                         #   existing block-info/spell-check fallback
        ├── workspace.rs          # unchanged — `uri_to_path` reused as-is
        └── position.rs           # + `text_for_span`, a small new helper
                                   #   (span → concrete source substring),
                                   #   the same "one place, reused everywhere"
                                   #   charter this module already has
```

**Structure Decision**: Split across the two crates the constitution already
assigns this kind of work to — new *analysis* logic in `voyager-core`
(`token_resolution.rs`, mirroring `block_resolution.rs`), new *adapter* logic
(disk I/O, hover-markdown formatting) in `drut-lsp`. No other crate changes.

## Complexity Tracking

Not applicable — no Constitution Check violations (see above).
