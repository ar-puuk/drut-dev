# Quickstart: Validating the Top-Level Indent Default Revert

A runnable validation guide, not an implementation walkthrough — proves
this feature against spec.md's Success Criteria. See
`contracts/top-level-indent-toggle.md` for the exact algorithm and
`research.md` for the full call-site/test-retargeting inventory.

## Prerequisites

- Rust stable toolchain.
- A local checkout of the WF-TDM-Official-Releases corpus (`$CORPUS`).

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` unit tests — validates FR-001/FR-002/FR-003, US1/US2

```powershell
cargo test -p voyager-core --lib format::
```

Expected: all green, including the 3 retargeted `Normalize`-mode tests
(`top_level_baseline_is_always_normalized_to_zero`,
`bare_top_level_statement_is_normalized_to_zero`,
`diagnosed_block_opener_is_normalized_but_children_stay_untouched`) and
their 3 new `Preserve`-mode siblings (research.md §3).

## 3. `008` residue-guarantee regression under explicit `Normalize` — validates FR-006, US2

```powershell
cargo test -p voyager-core --test format_sequence
```

Expected: all green, all 5 tests retargeted to explicit
`TopLevelIndentMode::Normalize` — proving `008`'s own guarantee (the
`PROCESS`/`RUN` residue sequence resolving in one pass) is unchanged now
that it's opt-in rather than default.

## 4. Golden-fixture regeneration (`preserve`) — validates FR-005, SC-001/SC-004

```powershell
$env:UPDATE_GOLDEN = "1"
cargo test -p voyager-core --test format_corpus
Remove-Item Env:\UPDATE_GOLDEN
git diff crates/voyager-core/tests/fixtures/golden/
```

Expected: every file `008` previously changed reverts back toward its
pre-`008` top-level indentation — review each diff individually,
confirming only top-level leading-whitespace lines changed (reverting),
nothing else moved or was corrupted. Report this review's outcome
explicitly, per-file, before committing (same non-optional discipline
`008`'s own quickstart used).

## 5. `Normalize`-mode fixture set — validates FR-006/SC-002

```powershell
cargo test -p voyager-core --test format_corpus -- normalize
```

Expected: all green against `tests/fixtures/golden_normalize/` (a copy of
`008`'s already-committed, already-reviewed golden output, byte-for-byte)
— proving explicit `--top-level-indent=normalize` reproduces `008`'s
shipped behavior exactly, no second human-review pass needed since the
content itself is unchanged from what `008` already had reviewed.

## 6. Default-placement verification — validates FR-004, US3 (the check named explicitly in this feature's own spec)

```powershell
cargo test -p voyager-core --lib
cargo test -p drut-cli --test format_flags
cargo test -p drut-lsp --lib formatting:: range_formatting::
cargo test -p drut-mcp --lib format::
```

Expected: all green, specifically including the new dedicated tests at
each of the four integration points named in FR-004 — `voyager-core`'s
own `FormatOptions::default()` test, `drut-cli`'s flag-omitted-defaults-
to-preserve test, and the two new `drut-lsp` tests plus the new
`drut-mcp` test, each independently confirming `Preserve` is what that
call site actually resolves to (not inferred from any other call site
passing).

## 7. Full-corpus revalidation — validates SC-004 (renumbered from `008`'s SC-004)

```powershell
$env:DRUT_CORPUS_PATH = "$CORPUS"
cargo test --release -p drut-cli --test fixture_corpus_e2e -- --ignored
cargo test --release -p drut-lsp --test diagnostics_corpus -- --ignored
cargo test --release -p drut-mcp --test diagnostics_corpus -- --ignored
```

Expected: still 161/161 clean under both modes — this remains a
whitespace-shifting change only, not a structural one.

## 8. Full test suite

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2, 4 | SC-001 |
| 3, 5 | SC-002 |
| 4 | SC-004 |
| 6 | SC-003 |
| 7 | SC-005 |
