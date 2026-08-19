# Quickstart: Validating Function-Call Casing Normalization

A runnable validation guide, not an implementation walkthrough — proves this feature
against `spec.md`'s Success Criteria. See `contracts/function-casing.md` for the exact
behavior contract and `data-model.md`/`research.md` for the full design rationale.

## Prerequisites

- Rust stable toolchain.
- `017-casing-categories-indent-width` and `024-function-call-highlighting` already
  merged (this feature amends the former's `CasingSettings` shape and reuses the
  latter's 138-name list; it is not a standalone module).

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` unit tests — validates FR-001–FR-004

```powershell
cargo test -p voyager-core function_call
```

Expected: all green, including —

- `RouteName = replacestr(RouteName,'-','',0)` under `function_calls: Upper` renders
  `RouteName = REPLACESTR(RouteName,'-','',0)`.
- `if (rightstr(trim(RouteName),1)='-')` under `function_calls: Upper` renders both
  `RIGHTSTR` and `TRIM` uppercase.
- `FILEO format=csv` under `pair_keywords: Upper, function_calls: Lower` renders
  `FILEO FORMAT=csv` — `format` here is the pair-keyword name, untouched by
  `function_calls`.
- `X = FORMAT(volume,8,2,',')` under the same settings renders
  `X = format(volume,8,2,',')` — `FORMAT` here is the function call, untouched by
  `pair_keywords`.
- `PRINT LIST='calling replacestr(x) here'` under `function_calls: Upper` renders
  byte-identical (quoted text untouched).
- `MAX = 100` under `function_calls: Upper` renders byte-identical (`MAX` not followed
  by `(`).
- All 138 recognized names, individually, rewrite correctly under both `Upper` and
  `Lower` — data-driven test over the full list (mirrors `024`'s own SC-001 remediation).

## 3. Idempotence — validates SC-003

```powershell
cargo test -p voyager-core function_call -- idempotent
```

Expected: formatting an already-`Upper`-cased script a second time produces zero edits.

## 4. `preserve` mode — validates SC-002

```powershell
cargo test -p voyager-core format -- preserve
```

Expected: every fixture from Step 2, formatted with `casing_function_calls` unset or
`preserve`, renders byte-identical to its input.

## 5. Full workspace re-proof + golden-fixture corpus

```powershell
cargo test --release --workspace
cargo test -p voyager-core --test format_corpus -- golden_casing_function_calls
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: the new `golden_casing_function_calls/real_corpus` fixture set (§6 in
`research.md` — one `Upper` variant, applied to the same 9 already-reviewed fixtures
`golden_data_references`/`golden_normalize` use) matches committed golden output
exactly, and the existing `check_idempotent` harness (already run for every configured
variant in `format_corpus.rs`) confirms idempotence end-to-end on real scripts, not just
synthetic unit-test input.

## 6. Adapter surface spot-check

```powershell
cargo test -p drut-config
cargo test -p drut-cli
cargo test -p drut-mcp
```

Expected: each crate's existing casing-field test pattern (mirroring
`casing_pair_keywords`'s own tests) now also covers `casing_function_calls` — TOML
parse/merge, `--casing-function-calls` CLI flag, and the MCP `format` tool's
`casing_function_calls` parameter each round-trip correctly.

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2 | SC-001, SC-004 |
| 3 | SC-003 |
| 4 | SC-002 |
| 5 | All (real-corpus re-proof, golden-fixture evidence) |
| 6 | SC-001 (surface reachability) |
