# Contract: Undefined `@token@` Diagnostic (addition)

A conceptual signature contract, not final Rust source, but the shapes and guarantees below are
binding — same convention every prior contract doc in this repo follows. This feature adds no
new public `voyager-core` API surface beyond one function; everything else is `drut-lsp`-internal.

## `voyager-core` additions

```text
pub fn token_resolution::all_variable_refs(nodes: &[Node]) -> Vec<VariableRefAt>
```

- Pure, no I/O, never panics on any input, including structurally broken `nodes` — same contract
  shape every other public function in `token_resolution.rs` already has.
- Returns every `@name@` reference `variable_ref_at`'s own traversal can see — a block-opener
  `@token@` is absent from the result by construction, not filtered (research.md §3).
- `VariableRefAt`, `resolve_token_value`, `read_file_refs`, everything else in this module:
  **unchanged**.

## `drut-lsp` additions

- `hover.rs::collect_included_files`/`struct IncludedFile`: visibility widened to `pub(crate)`,
  no behavior change.
- `undefined_token.rs` (new): `undefined_token_positions(uri, doc) -> Vec<VariableRefAt>` —
  every unresolvable reference in the given open document.
- `diagnostics.rs::publish`: gains a fourth chained diagnostic stream, `HINT` severity, source
  `"drut-token"`, code `"UndefinedToken"` — same shape as the two existing non-`DiagnosticKind`
  streams (`"drut-fmt"`'s `UnclosedFmtOff`, `"drut-config"`'s `DrutTomlProblem`).

## Guarantees

- **Never flags a resolvable reference** (FR-006): a `@token@` with a same-file assignment, or
  one reachable through a single level of static `READ FILE` inclusion, at or before its own
  position, never receives this notice.
- **Never flags a resolver blind spot** (FR-003): block-opener position, multi-level inclusion,
  and dynamic (token-built) inclusion path are all structurally excluded by reusing the existing
  resolution functions unmodified (research.md §3) — not a separate suppression rule that could
  drift out of sync with the resolver's own actual reach.
- **Never reaches `check`/`diagnose`** (FR-005): this stream is built and published only inside
  `drut-lsp/src/diagnostics.rs::publish`. `drut-cli`'s `check` command and `drut-mcp`'s
  `diagnose` tool both continue to expose exactly the six/seven real `DiagnosticKind` values,
  unchanged — `002-cli-check-format` FR-003's "never a narrowed subset" claim remains true of
  exactly the same set it was true of before this feature.
- **Never a `DiagnosticKind` variant** (FR-004): `voyager_core::Diagnostic`/`DiagnosticKind` are
  untouched — every existing consumer that pattern-matches or exhaustively handles
  `DiagnosticKind` (e.g. `kind_name` in `diagnostics.rs`, `category_name` in `drut-mcp`) compiles
  and behaves identically, with nothing new to handle.
- **No configuration surface** (FR-008): no `drut.toml` field, CLI flag, or MCP param exists for
  this capability — it is unconditional whenever the LSP publishes diagnostics, the same as the
  two existing Hint-severity streams it's modeled on.
- **Live updates** (FR-007): published on the same `publish()` call every other diagnostic
  stream already goes through — no separate trigger, no stale state after an edit.

## What this contract does *not* promise (by design, this phase)

- No plain-assignment-identifier checking (`X` used with no prior `X = value`) — `@token@`
  substitution references only.
- No data-reference-token checking (`MI`/`MW`/etc., bound via `FILEI`/`FILEO`) — structurally
  different binding mechanism, never researched for this feature.
- No CLI/MCP reach — LSP-only, by explicit decision (spec.md Assumptions, `ROADMAP.md` item 14).
- No configurability — always on, no severity override, no way to suppress a specific reference.
