# Quickstart: Validating Blank-Line-Run Normalization

A runnable validation guide, not an implementation walkthrough — proves this feature against
spec.md's Success Criteria. See `contracts/blank-line-normalization.md` for the exact API/config
shapes and `data-model.md`/`research.md` for the full design rationale.

## Prerequisites

- Rust stable toolchain.

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` `blank_line` unit tests — validates FR-001–FR-008

```powershell
cargo test -p voyager-core blank_line
```

Expected: all green, including —
- A run of 5 blank lines between two top-level statements contracts to exactly 2 (the default
  top-level cap); a run of 2 or fewer is untouched.
- A run of 4 blank lines inside a block's body (any nesting depth) contracts to exactly 1 (the
  default nested cap), independent of the top-level cap.
- A doubly-nested block's own excessive blank-line run gets the same nested cap, not a
  further-reduced one.
- A whitespace-only line counts as blank for run-length purposes.
- Survivor lines are byte-for-byte identical to the original run's own first N lines — a
  whitespace-only survivor is never trimmed to zero-length.
- `Preserve` (the default) leaves every run, however long, byte-for-byte unchanged.

## 3. `; FMT: OFF` interaction — validates FR-010

```powershell
cargo test -p voyager-core blank_lines_auto_respects
```

Expected: a protected region's excessive blank-line run is left exactly as written; an
unprotected run elsewhere in the same file contracts normally.

## 4. `drut-config`/`drut-cli`/`drut-mcp` tests — validates FR-012, data-model.md §3

```powershell
cargo test -p drut-config
cargo test -p drut-cli format
cargo test -p drut-mcp format
```

Expected: all green, including —
- A `drut.toml` with `blank_lines = "auto"` and both caps set resolves correctly; an invalid cap
  value falls back to that cap's own default with a non-blocking notice (FR-011).
- `--blank-lines=auto`/`--top-level-blank-line-cap=N`/`--nested-blank-line-cap=N` each override a
  `drut.toml`-resolved value for one run; an out-of-range explicit value is a clean usage error.
- The MCP `format` tool's new parameters produce output matching the CLI's equivalent flags.

## 5. Full workspace re-proof + real-corpus revalidation

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Then run the full 161-file real corpus through `drut format` (CLI), the LSP's format-on-save
path, and the MCP `format` tool, with **no `blank_lines` configuration supplied** — expected:
zero diagnostic or output change from before this feature (SC-003).

Separately, format a handful of real corpus files *with* `blank_lines=auto` (default caps),
hand-verify the diffs are exactly the expected line deletions (nothing reordered, no surviving
line's own content touched, no non-blank content touched), then promote those diffs to new
golden fixtures, same discipline `017`/`018` already established. Confirm idempotence (SC-005).

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2, 3 | SC-001, SC-002 |
| 4 | SC-004 |
| 5 | SC-003, SC-005, all others (integration re-proof) |
