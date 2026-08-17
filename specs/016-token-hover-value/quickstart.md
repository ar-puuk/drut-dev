# Quickstart: Validating Token Hover Shows Assigned Value

A runnable validation guide, not an implementation walkthrough — proves this
feature against spec.md's Success Criteria. See
`contracts/token-resolution-api.md` for the exact mechanism and `research.md` for
the full design rationale.

## Prerequisites

- Rust stable toolchain.
- VS Code, for the manual smoke test (step 4).

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` unit tests — validates FR-001 through FR-005

```powershell
cargo test -p voyager-core token_resolution
```

Expected: all green, including — directly against real corpus-shaped fixtures, not
just synthetic minimal cases —
- `variable_ref_at` finding a `@token@` reference at its exact span and returning
  `None` a character outside it.
- `all_assignments` finding a target inside a nested `IF`/`LOOP` block, in source
  order.
- `read_file_refs` correctly classifying `READ FILE = '_ControlCenter.block'` as
  `literal_path: Some("_ControlCenter.block")` and
  `READ FILE = '@ParentDir@sub\path.block'` as `literal_path: None` — the exact
  two real shapes found in `WF-TDM-Development` (spec.md Assumptions).
- `resolve_token_value`'s ordering rule: a same-file reassignment after an included
  file's own assignment wins (US2 Acceptance Scenario 2); an assignment strictly
  after the hover position (same-file, or via a `READ FILE` appearing after the
  hover position) is never selected (US1 Scenario 3, Edge Cases).

## 3. `drut-lsp` tests — validates FR-006, FR-007, FR-009, FR-010, US1–US3

```powershell
cargo test -p drut-lsp hover
```

Expected: all green, including:
- The existing `hover.rs` tests (block info, short-IF, spell-check nudge, unrelated
  token) completely unaffected — byte-for-byte identical results (FR-010).
- A same-file case: `ZoneMsgRate = 50` then `@ZoneMsgRate@` later, hover shows `50`.
- A real-filesystem cross-file case (temp directory, two real files on disk): an
  open document containing `READ FILE = 'sibling.block'` where `sibling.block`
  (written to the same temp directory, not opened in the editor) assigns
  `UsedZones = 3629`; hovering `@UsedZones@` in the open document resolves to
  `3629` and names `sibling.block`.
- A `READ FILE` pointing at a nonexistent file: hover falls back to existing
  behavior, no panic, no error response (FR-007).
- A `READ FILE = '@ParentDir@...'` (dynamic path): no cross-file resolution is
  attempted for it; same-file resolution for other tokens is unaffected.
- Case-insensitive matching: `ParentDir = ...` assigned, `@PARENTDIR@` referenced,
  still resolves (FR-005).

## 4. Manual verification in a real VS Code instance — validates SC-001–SC-004

1. Launch the extension development host (`F5` in `editors/vscode/`).
2. Create (or reuse) a small `.s` file:
   ```
   ZoneMsgRate = 50
   READ FILE = 'params.block'
   PRINT LIST='@ZoneMsgRate@ @UsedZones@ @Nope@'
   ```
   and, next to it, `params.block`:
   ```
   UsedZones = 3629
   ```
3. Hover `@ZoneMsgRate@` — confirm the value `50` and its line number appear
   (SC-001).
4. Hover `@UsedZones@` — confirm the value `3629` appears, naming `params.block`
   as the source, without opening that file yourself (SC-002).
5. Hover `@Nope@` (never assigned anywhere) — confirm no fabricated value appears,
   and that hovering elsewhere in the file (e.g. an `IF`/`ENDIF` pair, if present)
   is completely unaffected (SC-003, SC-004).
6. Rename `params.block`'s own assignment to a different value, save, and hover
   `@UsedZones@` again — confirm the new value shows (proves "always fresh, never
   cached," spec.md Assumptions).

## 5. Full workspace re-proof

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: clean — this feature touches only `voyager-core` and `drut-lsp`, so this
is primarily a regression check that nothing in any earlier feature's suite was
disturbed.

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 3 | SC-001 |
| 4 | SC-002 |
| 3, 4, 5 | SC-003 |
| 5, 6 | SC-004 |
