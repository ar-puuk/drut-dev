# Contract: Unused `@token@` Diagnostic (addition)

A conceptual signature contract, not final Rust source, but the shapes and guarantees below are
binding — same convention every prior contract doc in this repo follows. This feature adds no
new public `voyager-core` API surface beyond one function; everything else is `drut-lsp`-internal.

## `voyager-core` additions

```text
pub fn token_resolution::all_variable_refs_including_openers(nodes: &[Node]) -> Vec<VariableRefAt>
```

- Pure, no I/O, never panics on any input, including structurally broken `nodes` — same contract
  shape every other public function in `token_resolution.rs` already has.
- Returns every `@name@` reference `all_variable_refs` already finds, PLUS a `@name@` reference
  sitting on a block-opener statement's own line (`Block::opener_tokens`) — the one behavioral
  difference from `all_variable_refs`, which excludes that position (research.md §1-2).
- `all_variable_refs`, `variable_ref_at`, `Assignment`, `all_assignments`,
  `resolve_token_value`, `read_file_refs`, everything else in this module: **unchanged**.

## `drut-lsp` additions

- `unused_token.rs` (new): `unused_token_assignments(uri, doc) -> Vec<UnusedAssignment>` — every
  `Assignment` in the given open document whose target name has no `@name@` reference anywhere
  in scope.
- `diagnostics.rs::publish`: gains a fifth chained diagnostic stream, `HINT` severity, source
  `"drut-token"` (shared with `UndefinedToken`), code `"UnusedToken"`.

## Guarantees

- **Never flags a referenced name** (FR-006): an assignment whose target has at least one
  `@name@` reference anywhere in scope — same file, one level of static `READ FILE` inclusion,
  or a block-opener position — never receives this notice, regardless of how many times that
  name was assigned.
- **Counts a block-opener reference as a genuine use** (FR-003): unlike `020`'s reuse of
  `all_variable_refs`, this feature's `all_variable_refs_including_openers` closes that specific
  blind spot — verified by a dedicated test asserting a `RUN PGM=@Prog@`-only-used `Prog =
  MATRIX` assignment is never flagged.
- **Flags every dead assignment site independently** (FR-002, Clarification Q1): a name
  reassigned multiple times with zero references anywhere in scope produces one notice per
  assignment, not deduplicated to one-per-name.
- **Applies unconditionally regardless of `READ FILE` participation** (FR-001, Clarification Q2):
  this feature does not suppress itself for a file that includes another file, or that might
  itself be included by some other file it can't see — a documented, accepted false-positive
  risk for the shared-parameters-file authoring pattern, not a bug.
- **Never reaches `check`/`diagnose`** (FR-005): this stream is built and published only inside
  `drut-lsp/src/diagnostics.rs::publish`. `drut-cli`'s `check` command and `drut-mcp`'s
  `diagnose` tool continue to expose exactly the same `DiagnosticKind` values as before this
  feature — `002-cli-check-format` FR-003's "never a narrowed subset" claim remains true of
  exactly the same set it was true of before.
- **Never a `DiagnosticKind` variant** (FR-004): `voyager_core::Diagnostic`/`DiagnosticKind` are
  untouched — every existing exhaustive consumer compiles and behaves identically.
- **No configuration surface** (FR-008): no `drut.toml` field, CLI flag, or MCP param exists for
  this capability — unconditional whenever the LSP publishes diagnostics.
- **Live updates** (FR-007): published on the same `publish()` call every other diagnostic
  stream already goes through.

## What this contract does *not* promise (by design, this phase)

- No dead-store analysis (an earlier assignment shadowed by a later one before ever being read,
  despite the name eventually being read from the final assignment) — only "is this name
  referenced anywhere at all," per assignment site.
- No detection of a name used only by a file that includes this one — the accepted, documented
  Clarification Q2 blind spot.
- No plain, non-`@token@` unused-identifier checking — `@token@` substitution assignments only,
  the same narrow scope `020-undefined-token-diagnostic` established for its own inverse check.
- No CLI/MCP reach — LSP-only, by explicit decision (spec.md Assumptions).
- No configurability — always on, no severity override, no way to suppress a specific name.
