# Quickstart: Validating the UnmatchedProcess Diagnostic

A runnable validation guide, not an implementation walkthrough — proves
this feature against spec.md's Success Criteria. See
`contracts/unmatched-process-diagnostic.md` for the exact firing condition
and the full adapter-change checklist.

## Prerequisites

- Rust stable toolchain.
- The workspace builds: `cargo build --workspace` from repo root.
- A local checkout of the WF-TDM-Official-Releases corpus (referred to
  below as `$CORPUS`), same as every prior phase's full-corpus validation.

## 1. Build (proves every exhaustive match got its new arm — contracts §"Required adapter changes")

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: builds cleanly, zero warnings. If any of `sarif.rs`/
`diagnostics.rs`/`diagnose.rs`/`fixture_corpus.rs` was missed, this step
fails to compile — the exhaustiveness guarantee (research.md §1) makes
this step itself a completeness proof, not just a build check.

## 2. `voyager-core` unit tests — validates FR-002/FR-003/FR-004, spec.md Acceptance Scenarios 1-4

```powershell
cargo test -p voyager-core block::tests::
```

Expected: all green, including the new `UnmatchedProcess`-firing case, the
explicit-close-suppresses-it case, the implicit-close-by-sibling-suppresses-it
case, and the nested-inside-another-block-closes-first case (Acceptance
Scenario 4).

## 3. Fixture corpus — validates FR-009's real-shaped regression fixture

```powershell
cargo test -p voyager-core --test fixture_corpus
```

Expected: `every_diagnostic_category_has_at_least_one_broken_fixture` now
covers all 8 kinds and passes; the new
`unmatched_process_with_trailing_content.s` fixture is picked up
automatically by `collect_fixtures` and correctly declares
`UnmatchedProcess` via its `; EXPECT:` marker.

## 4. Full real-corpus revalidation — validates FR-008 (the empirical zero-false-positive re-proof)

```powershell
$env:DRUT_CORPUS_PATH = "$CORPUS"
cargo test -p drut-cli --test fixture_corpus_e2e -- --ignored
cargo test -p drut-lsp --test diagnostics_corpus -- --ignored
cargo test -p drut-mcp --test diagnostics_corpus -- --ignored
```

Expected: still 161/161 clean across all three corpus-validation suites —
the same result this feature's own motivating investigation already found,
now re-proven through every adapter's own diagnostic-surfacing path, not
just `voyager-core::parse` in isolation. **Report this step's result
explicitly, as its own item** — this is the empirical claim the entire
feature's low-risk framing rests on, not a routine regression check to
fold into a general "tests pass" summary.

## 5. Adapter-level spot checks — validates FR-007

```powershell
cargo run -p drut-cli --bin drut -- check path\to\a-fixture-with-unmatched-process.s
cargo run -p drut-cli --bin drut -- check path\to\a-fixture-with-unmatched-process.s --format sarif
```

Expected: plain-text output shows `UnmatchedProcess` (via `{:?}` Debug
formatting, unchanged code); SARIF output includes a
`"ruleId": "unmatched-process"` result with the correct `shortDescription`
in the rule catalog.

## 6. Full test suite

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: all green, zero clippy warnings, confirming zero regressions
anywhere in the four-crate workspace.

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2, 5 | SC-001 |
| 3, 4 | SC-002 |
| 2, 3 | SC-003 |
