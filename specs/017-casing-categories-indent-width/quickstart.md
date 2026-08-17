# Quickstart: Validating Per-Category Casing + Configurable Indent Width

A runnable validation guide, not an implementation walkthrough — proves this feature against
spec.md's Success Criteria. See `contracts/casing-categories-indent-width.md` for the exact
API/config shapes and `data-model.md`/`research.md` for the full design rationale.

## Prerequisites

- Rust stable toolchain.

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` unit tests — validates FR-001–FR-008

```powershell
cargo test -p voyager-core casing
cargo test -p voyager-core data_reference
cargo test -p voyager-core keywords
```

Expected: all green, including —
- `CasingSettings::default()` has all three fields `Preserve`.
- Formatting with `control_words: Upper, pair_keywords: Preserve, data_references: Lower` set
  independently changes only each category's own tokens in a mixed script (US1 AS1).
- `data_reference_occurrences` finds `mw`/`mi` in `mw[1] = mi.1.1 + mi.2.1` (assignment target
  + dot-notation read), and `MW[201]=` in a `PATHLOAD` pair-keyword-shaped statement — all
  three occurrences named `"MW"`/`"MI"` and rewritten identically when `data_references` is set
  (US2 AS1/AS2).
- `data_references: Preserve` (the default) leaves `mw`/`li`/`ni`/`i`/`j` untouched (US2 AS3).
- `NUMREC`/`CNT`/`ITER`/`LP`/`RECNUM` no longer appear in `completion_candidates`/`did_you_mean`
  results for the `LOOP=` position; `ZONES` does appear for `RUN`.

## 3. `voyager-core` format tests — validates FR-009–FR-012

```powershell
cargo test -p voyager-core format
```

Expected: all green, including —
- `FormatOptions::default().indent_width == 4`.
- A 3-level-nested script formatted with `indent_width: 2` advances 2 spaces per level (US3
  AS1).
- `FormatOptions::default()` (nothing configured) produces byte-identical output to this
  feature's pre-existing behavior across every existing golden fixture (FR-012, US3 AS3).

## 4. `drut-config` tests — validates the precedence matrix (data-model.md §3)

```powershell
cargo test -p drut-config
```

Expected: all green, including —
- A `drut.toml` with only the legacy `casing = "upper"` field still sets `control_words` and
  `pair_keywords` to `Upper`, `data_references` still `Preserve` — unchanged from before this
  feature (regression case).
- A `drut.toml` setting both the legacy `casing` field and a granular
  `data_references_casing` field: the granular field governs `data_references`, the legacy
  field governs the other two — no conflict, no silent override of one by the other.
- An `indent_width = 0` (or `500`) in `drut.toml` falls back to `4` with a non-blocking notice,
  never a hard failure (US3 AS2).

## 5. `drut-cli`/`drut-mcp` tests — validates FR-013

```powershell
cargo test -p drut-cli format
cargo test -p drut-mcp format
```

Expected: all green, including —
- `--casing=upper` (legacy flag) still works exactly as before.
- `--data-references-casing=upper` overrides a `drut.toml`-resolved `preserve` for one run.
- `--indent-width=2` overrides a `drut.toml`-resolved width for one run.
- The MCP `format` tool's new parameters produce output matching the CLI's equivalent flags,
  same shape every prior casing/indent parity test in this project already asserts.

## 6. Full workspace re-proof + real-corpus revalidation

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Then run the full 161-file real corpus through `drut format` (CLI), the LSP's format-on-save
path, and the MCP `format` tool, with **no new configuration supplied** — expected: zero
diagnostic or output change from before this feature (SC-003), reported as its own explicit
result per this project's established standard, not inferred from the unit-test suite alone.

Separately, format a handful of real corpus files *with* `data_references_casing=upper` and
`indent_width=2` configured, hand-verify the diffs are exactly what's expected (only the
data-reference tokens changed case; only nesting-level spacing changed), then promote those
diffs to new golden fixtures.

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2 | SC-001, SC-002, SC-006 |
| 3 | SC-003, SC-004 |
| 4, 5 | SC-005 |
| 6 | SC-003 (full-corpus proof), all others (integration re-proof) |
