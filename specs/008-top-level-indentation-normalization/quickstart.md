# Quickstart: Validating Top-Level Indentation Normalization

A runnable validation guide, not an implementation walkthrough — proves
this feature against spec.md's Success Criteria. See
`contracts/top-level-indentation.md` for the exact algorithm and the
`007`-interaction resolution.

## Prerequisites

- Rust stable toolchain.
- A local checkout of the WF-TDM-Official-Releases corpus (`$CORPUS`).

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` unit tests — validates FR-001/FR-002/FR-003, spec.md Acceptance Scenarios 1-3

```powershell
cargo test -p voyager-core --lib format::
```

Expected: all green, including new cases for a bare top-level statement,
a top-level block opener, and a block whose children carry stale
indentation relative to the block's newly-corrected base.

## 3. Residue-resolution regression — validates FR-005/SC-002, US2

```powershell
cargo test -p voyager-core --test format_sequence
```

Expected: all green, including a new/updated test proving the
`PROCESS`/`RUN` residue sequence fully resolves within the second format
pass alone — specifically covering the case where `RUN` was left at
*stale* non-zero indentation (not already-correct), the scenario `007`
alone never corrected.

## 4. Golden-fixture regeneration — validates FR-006, SC-003 (human-in-the-loop, not automatable)

```powershell
$env:UPDATE_GOLDEN = "1"
cargo test -p voyager-core --test format_corpus
Remove-Item Env:\UPDATE_GOLDEN
git diff crates/voyager-core/tests/fixtures/golden/
```

Expected: exactly the 7 files research.md §3 names change (plus none of
the hand-written set) — review each diff individually, confirming only
leading-whitespace lines changed and every changed line's new value is
`0` for what was previously that file's own top-level baseline. Report
this review's outcome explicitly, per-file, before committing — this is
the step this feature's own Definition of Done (FR-006) treats as
non-optional, not a mechanical regenerate-and-forget step.

## 5. Full-corpus revalidation — validates SC-004

```powershell
$env:DRUT_CORPUS_PATH = "$CORPUS"
cargo test --release -p drut-cli --test fixture_corpus_e2e -- --ignored
cargo test --release -p drut-lsp --test diagnostics_corpus -- --ignored
cargo test --release -p drut-mcp --test diagnostics_corpus -- --ignored
```

Expected: still 161/161 clean — this is a whitespace-shifting change, not
a structural one; zero new diagnostics anywhere.

## 6. Full test suite

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2 | SC-001 |
| 3 | SC-002 |
| 4 | SC-003 |
| 5 | SC-004 |
