# Quickstart: Validating FMT Region Markers

A runnable validation guide, not an implementation walkthrough — proves this feature against spec.md's Success Criteria. See `contracts/fmt-region-markers.md` for the exact algorithm and `research.md` for the full design rationale.

## Prerequisites

- Rust stable toolchain.
- A local checkout of the WF-TDM-Official-Releases corpus (`$CORPUS`).

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` unit tests — validates FR-001–FR-006, US1

```powershell
cargo test -p voyager-core --lib format::
```

Expected: all green, including hand-written tests for every Edge Case named in spec.md — a protected range with wrong indentation/casing left untouched (FR-003), lines outside a region normalized as usual (FR-004), duplicate `; FMT: OFF` and stray `; FMT: ON` no-ops (US1 Acceptance Scenarios 4-5), a protected range straddling a block boundary, and a whole-file-is-one-region case.

## 3. Unclosed-marker detection — validates FR-010, US2

```powershell
cargo test -p voyager-core --lib format:: -- unclosed
```

Expected: all green, including a direct assertion on `unclosed_fmt_off_markers`'s standalone return value (not only `FormatResult`'s field) and confirmation that every line from an unmatched `; FMT: OFF` through end-of-file is still left untouched (US2's revised Acceptance Scenario 1 — both the protection *and* the notice hold together).

## 4. Idempotency — validates FR-008, SC-003

```powershell
cargo test -p voyager-core --lib format:: -- idempoten
```

Expected: formatting a fixture containing protected regions twice in a row produces byte-identical output both passes — trivially true given protected lines are never touched, but asserted directly, not assumed.

## 5. Golden-fixture corpus — validates SC-001/SC-002

```powershell
cargo test -p voyager-core --test format_corpus
```

Expected: all green with **zero** golden-fixture diffs — this feature must not change output for any existing fixture, since none of them contain `; FMT: OFF`/`; FMT: ON` markers yet (FR-004/SC-002). New fixtures with synthetic marker pairs inserted (a copy of an existing fixture, markers added around a deliberately "wrong" section) are added under `tests/fixtures/golden/` following the existing naming convention, proving SC-001 without disturbing the existing 161-file-corpus-derived set.

## 6. Adapter surfaces — validates FR-007, US3

```powershell
cargo test -p drut-cli --test format_flags
cargo test -p drut-lsp --lib formatting:: range_formatting:: diagnostics::
cargo test -p drut-mcp --lib format::
```

Expected: all green, including dedicated tests confirming a protected range survives identically through the CLI, both LSP formatting handlers, and the MCP `format` tool — and, separately, that `diagnostics.rs` publishes an `UnclosedFmtOff`-coded, `HINT`-severity diagnostic (distinct `source: "drut-fmt"`, not `"drut"`) for a fixture with an unmatched `; FMT: OFF`, with no change to any existing structural-diagnostic test's assertions (a purely additive diagnostics stream).

## 7. Full-corpus revalidation — validates SC-004

```powershell
$env:DRUT_CORPUS_PATH = "$CORPUS"
cargo test --release -p drut-cli --test fixture_corpus_e2e -- --ignored
cargo test --release -p drut-lsp --test diagnostics_corpus -- --ignored
cargo test --release -p drut-mcp --test diagnostics_corpus -- --ignored
```

Expected: still 161/161 clean — none of the real corpus files contain `; FMT: OFF`/`; FMT: ON` markers today, so this is a pure regression check (zero new diagnostics, zero output changes), not a feature-specific proof.

## 8. Full test suite

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2, 5 | SC-001 |
| 5 | SC-002 |
| 4 | SC-003 |
| 7 | SC-004 |
