# Quickstart: Validating the Unused `@token@` Diagnostic

A runnable validation guide, not an implementation walkthrough — proves this feature against
spec.md's Success Criteria. See `contracts/unused-token-diagnostic.md` for the exact API/
LSP-stream shape and `data-model.md`/`research.md` for the full design rationale.

## Prerequisites

- Rust stable toolchain.

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` `all_variable_refs_including_openers` unit tests — validates FR-003, research.md §1-2

```powershell
cargo test -p voyager-core all_variable_refs
```

Expected: all green, including —
- Everything `all_variable_refs` already returns is also returned by
  `all_variable_refs_including_openers`.
- A `@token@` on a block-opener line (`RUN PGM=@Prog@`) IS returned by
  `all_variable_refs_including_openers` — the one behavioral difference from `all_variable_refs`.
- `all_variable_refs`'s own existing tests (including
  `all_variable_refs_excludes_a_block_opener_reference`) still pass unmodified — proof the
  pre-existing function and its `020` consumer are untouched.

## 3. `drut-lsp` `unused_token` tests — validates FR-001, FR-002, FR-003, FR-006

```powershell
cargo test -p drut-lsp unused_token
```

Expected: all green, including —
- An assignment with no `@name@` reference anywhere is returned by `unused_token_assignments`
  (US1 AS1).
- An assignment referenced later in the same file is not returned (US1 AS2).
- An assignment referenced only on a block-opener line (`RUN PGM=@Prog@`) is not returned — the
  correctness fix this feature makes (US1 AS2, FR-003).
- A name reassigned twice with one reference after both: neither assignment is returned (US1 AS3).
- A name reassigned twice with zero references anywhere: BOTH assignments are returned
  independently (US1 AS5, Clarification Q1).
- An assignment with no reference in a file that also has a `READ FILE` statement is still
  returned — the check is not suppressed for files touching inclusion (US1 AS6, Clarification Q2).
- An assignment referenced only through one level of `READ FILE` inclusion is not returned.

## 4. `diagnostics.rs` stream tests — validates FR-004, FR-005, FR-007, SC-004, SC-005

```powershell
cargo test -p drut-lsp diagnostics
```

Expected: all green, including —
- A published diagnostics list for a document with one unused assignment includes exactly one
  `HINT`-severity, `"drut-token"`-sourced, `"UnusedToken"`-coded entry spanning that assignment
  statement.
- The six real `DiagnosticKind`-based diagnostics and `UndefinedToken` in the same document still
  publish unaffected by this feature's addition (SC-004).
- Editing the document to add a reference removes the notice on the next publish cycle (FR-007).

## 5. Full workspace re-proof + CLI/MCP non-reach confirmation

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Then confirm directly (not inferred) that neither `drut-cli`'s `check` command nor `drut-mcp`'s
`diagnose` tool ever includes this notice, on a document containing at least one unused
assignment — their output categories remain exactly the pre-existing `DiagnosticKind` names
(SC-005).

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2, 3 | SC-001, SC-002, SC-003 |
| 4 | SC-004 |
| 5 | SC-005, all others (integration re-proof) |
