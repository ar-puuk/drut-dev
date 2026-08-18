# Quickstart: Validating the Undefined `@token@` Diagnostic

A runnable validation guide, not an implementation walkthrough — proves this feature against
spec.md's Success Criteria. See `contracts/undefined-token-diagnostic.md` for the exact API/
LSP-stream shape and `data-model.md`/`research.md` for the full design rationale.

## Prerequisites

- Rust stable toolchain.

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` `all_variable_refs` unit tests — validates FR-001, research.md §2/§3

```powershell
cargo test -p voyager-core all_variable_refs
```

Expected: all green, including —
- Every `@name@` reference in a document with several is returned, source order.
- A `@token@` on a block-opener line (`RUN PGM=@Prog@`) is absent from the result — not
  filtered, structurally never collected (same reason `variable_ref_at` can't find it either).
- `IfBranch.condition` references are included, same as `variable_ref_at` already covers them.

## 3. `drut-lsp` `undefined_token` tests — validates FR-002, FR-003, FR-006

```powershell
cargo test -p drut-lsp undefined_token
```

Expected: all green, including —
- A `@token@` with no same-file assignment and no `READ FILE` inclusion is returned by
  `undefined_token_positions` (US1 AS1).
- A `@token@` with a same-file assignment is not returned (US1 AS2).
- A `@token@` resolvable only through a directly-included sibling file (one level) is not
  returned.
- A `@token@` resolvable only through two levels of inclusion is returned (correctly *not*
  resolved, since only one level is followed) — matching hover's own documented boundary (US1
  AS4).
- A `@token@` resolvable only through a token-built `READ FILE` path is returned (correctly not
  resolved) (US1 AS5).

## 4. `diagnostics.rs` stream tests — validates FR-004, FR-005, FR-007, SC-004, SC-005

```powershell
cargo test -p drut-lsp diagnostics
```

Expected: all green, including —
- A published diagnostics list for a document with one unresolvable `@token@` includes exactly
  one `HINT`-severity, `"drut-token"`-sourced entry at that reference's span.
- The six real `DiagnosticKind`-based diagnostics in the same document still publish at `ERROR`
  severity, source `"drut"`, unaffected by this feature's addition (SC-004).
- Editing the document to add the missing assignment removes the notice on the next publish
  cycle — no manual re-trigger needed (FR-007).

## 5. Full workspace re-proof + CLI/MCP non-reach confirmation

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Then confirm directly (not inferred) that neither `drut-cli`'s `check` command nor `drut-mcp`'s
`diagnose` tool ever includes this notice, on a document containing at least one unresolvable
`@token@` — their output categories remain exactly the pre-existing six/seven `DiagnosticKind`
names (SC-005).

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2, 3 | SC-001, SC-002, SC-003 |
| 4 | SC-004 |
| 5 | SC-005, all others (integration re-proof) |
