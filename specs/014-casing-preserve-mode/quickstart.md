# Quickstart: Validating Casing's Explicit `Preserve` Mode

A runnable validation guide, not an implementation walkthrough — proves
this feature against spec.md's Success Criteria. See `contracts/
casing-preserve-mode.md` for the exact algorithm/call-site treatment and
`research.md` for the full inventory. Unlike `009`'s quickstart, there is
no golden-fixture regeneration step and no manual VS Code step — this
feature has zero formatter-output change (FR-003) and zero `drut-lsp`
behavior change (research.md §2), so nothing exists for a human to visually
re-review or manually smoke-test beyond the automated suite itself.

## Prerequisites

- Rust stable toolchain.
- A local checkout of the WF-TDM-Official-Releases corpus (`$CORPUS`), for
  step 7.

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` unit tests — validates FR-001/FR-002/FR-003, User Story 2

```powershell
cargo test -p voyager-core --lib format::
```

Expected: all green, with **zero existing test modified beyond the
compiler-forced `upper()`/`normalize()` struct-literal updates** (research.md
§2) — every other existing assertion passes unmodified, which is the actual
proof of FR-003 (byte-identical output), not an inspection substitute for
it. New tests confirm `CasingConvention::default()` is `Preserve` and that
formatting under `Preserve` matches formatting under the old `None`-based
behavior for a representative fixture.

## 3. `format_sequence.rs`/`format_corpus.rs` — validates FR-003, SC-001 (zero output change)

```powershell
cargo test -p voyager-core --test format_sequence
cargo test -p voyager-core --test format_corpus
```

Expected: all green, with **no golden-fixture regeneration** — every
existing golden file, hand-written fixture, and real-corpus fixture is
expected to produce byte-identical output to before this feature, since
`Preserve` is a pure representation change (research.md §4). If any of
these fail, that is itself evidence FR-003 was violated, not a fixture
that needs updating.

## 4. `drut-config` — validates FR-004/FR-005, SC-005

```powershell
cargo test -p drut-config
```

Expected: all green, including new coverage that `casing = "preserve"` in
a `drut.toml` `[format]` table parses cleanly (not a warning, SC-005) and
that an unset `casing` (no file, or a file that doesn't mention it) resolves
to `CasingConvention::Preserve` through `resolve_format_options`.

## 5. `drut-cli` — validates FR-006, User Story 1

```powershell
cargo test -p drut-cli --test format_flags
```

Expected: all green, including a new test that `--casing=preserve`
overrides a `drut.toml`-resolved `upper`/`lower` for one run, and that the
existing bare-`--casing`/invalid-value usage-error tests still pass
unmodified (`preserve` is a new valid value, not a change to what counts
as invalid).

## 6. `drut-mcp` — validates FR-007, User Story 1

```powershell
cargo test -p drut-mcp --lib format::
```

Expected: all green, including a new test that `casing: "preserve"`
overrides a `drut.toml`-resolved value, mirroring step 5's CLI case.

## 7. `drut-lsp` — validates FR-008, User Story 3

```powershell
cargo test -p drut-lsp --lib
```

Expected: all green, with **zero test added or modified** — this is
intentional (research.md §2): `drut-lsp` never constructs an explicit
casing override, so the existing suite passing unmodified after the
`voyager-core`/`drut-config` type change compiles through *is* the
confirmation, not a gap to fill.

## 8. Full-corpus revalidation — validates SC-001 at real-corpus scale

```powershell
$env:DRUT_CORPUS_PATH = "$CORPUS"
cargo test --release -p drut-cli --test fixture_corpus_e2e -- --ignored
cargo test --release -p drut-lsp --test diagnostics_corpus -- --ignored
cargo test --release -p drut-mcp --test diagnostics_corpus -- --ignored
```

Expected: still 161/161 clean, and (unlike `008`/`009`) the CLI's
`fixture_corpus_e2e` idempotent-write check is expected to produce a
byte-identical result to the pre-feature baseline, not just a clean-diagnostics
result — this is a zero-output-change feature at real-corpus scale, not
only at unit-test scale.

## 9. Full test suite

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2, 3, 8 | SC-001 |
| 5, 6 | SC-002 |
| 2, 7 | SC-003 |
| 5, 6, 7 | SC-004 |
| 4 | SC-005 |
