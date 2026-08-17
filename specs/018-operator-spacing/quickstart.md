# Quickstart: Validating Operator Spacing Normalization

A runnable validation guide, not an implementation walkthrough — proves this feature against
spec.md's Success Criteria. See `contracts/operator-spacing.md` for the exact API/config shapes
and `data-model.md`/`research.md` for the full design rationale.

## Prerequisites

- Rust stable toolchain.

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` `operator_spacing` unit tests — validates FR-001–FR-005, FR-012

```powershell
cargo test -p voyager-core operator_spacing
```

Expected: all green, including —
- `ZONES   = 1` → `ZONES = 1`; `MATI=a.mat,MATO=b.mat` → `MATI = a.mat, MATO = b.mat`;
  `IF ( x==1 )` → `IF(x == 1)`; `MW[ 1 ]=mi.1.1+mi.2.1` → `MW[1] = mi.1.1 + mi.2.1` — all under
  `Fixed`.
- `I==1`, `A<>B`, `A>=B`, `A<=B` each normalize to exactly one space around the whole
  two-character operator, never a stray space *inside* `==`/`<>`/`>=`/`<=` itself
  (research.md §2's merge-recognition regression case).
- `MW[1] = -5` stays `MW[1] = -5` (unary sign, no inserted space); `MW[1] = A - B` normalizes to
  one space each side of the binary `-`.
- A trailing continuation operator (e.g. `FILEI NETI=x,` continuing to the next line) gets
  exactly one space before the `,` and nothing inserted after it (FR-012).
- `Preserve` (the default) leaves every one of the above byte-for-byte unchanged.

## 3. `voyager-core` `Auto` alignment tests — validates FR-006–FR-008

```powershell
cargo test -p voyager-core align
```

Expected: all green, including —
- Three consecutive `Assignment` statements with differing left-hand-side lengths align their
  `=` to the longest one's column (US2 AS1).
- A blank line, then a comment-only line, then an indentation-depth change, each independently
  splits one run into separate, independently-aligned runs (US2 AS2).
- A pair-keyword-shaped `Control` statement sitting among `Assignment` statements is spaced per
  `Fixed` only — it neither joins nor extends the surrounding run, and splits it (US2 AS3).
- A lone `Assignment` statement (no adjacent `Assignment` sibling) renders identically to what
  `Fixed` alone would produce (US2 AS4).

## 4. Render-pipeline edit-application tests — validates research.md §4's new capability

```powershell
cargo test -p voyager-core spacing
```

Expected: all green, including —
- A line with multiple spacing edits (e.g. two operators normalized on the same line) renders
  correctly with no corrupted offsets — the left-to-right rebuild, not the old same-length
  splice, handles this line.
- A line with both a casing edit and a spacing edit (e.g. an uppercased control word followed
  later on the same line by a normalized `=`) applies both correctly in one pass.
- `; FMT: OFF`/`; FMT: ON` protected lines receive no spacing edits at all, same funnel-point
  guarantee `push_if_present` already gives casing edits.

## 5. `drut-config`/`drut-cli`/`drut-mcp` tests — validates FR-013, data-model.md §4

```powershell
cargo test -p drut-config
cargo test -p drut-cli format
cargo test -p drut-mcp format
```

Expected: all green, including —
- A `drut.toml` with `operator_spacing = "fixed"` resolves correctly; an invalid value (e.g.
  `"tight"`) falls back to `preserve` with a non-blocking notice (FR-011).
- `--operator-spacing=auto` overrides a `drut.toml`-resolved value for one run.
- The MCP `format` tool's new `operator_spacing` parameter produces output matching the CLI's
  equivalent flag, same parity shape every prior formatting parameter test in this project
  already asserts.

## 6. Full workspace re-proof + real-corpus revalidation

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Then run the full 161-file real corpus through `drut format` (CLI), the LSP's format-on-save
path, and the MCP `format` tool, with **no `operator_spacing` configuration supplied** —
expected: zero diagnostic or output change from before this feature (SC-003), reported as its
own explicit result, not inferred from the unit-test suite alone.

Separately, format a handful of real corpus files *with* `operator_spacing=fixed` and, on a file
containing several consecutive assignments, `operator_spacing=auto` — hand-verify the diffs are
exactly the expected spacing/alignment changes (nothing reordered, no value/comment content
touched), then promote those diffs to new golden fixtures, same discipline `017` already
established for its own new golden variants. Confirm idempotence on both new variants (formatting
the already-formatted output produces no further change, SC-005) — the same
`check_idempotent` harness `format_corpus.rs` already runs for every other configured variant.

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2, 4 | SC-001 |
| 3 | SC-002 |
| 5 | SC-004 |
| 6 | SC-003, SC-005, all others (integration re-proof) |
