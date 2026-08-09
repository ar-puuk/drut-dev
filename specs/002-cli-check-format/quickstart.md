# Quickstart: Validating the Drut CLI

This is a runnable validation guide, not an implementation walkthrough — it proves
the feature works end-to-end against the success criteria in spec.md. See
`contracts/cli-contract.md` for the full flag reference and `data-model.md` for the
types referenced below.

## Prerequisites

- Rust stable toolchain (matches `voyager-core`'s existing requirement).
- The workspace builds: `cargo build` from repo root.
- A local checkout of the WF-TDM-Official-Releases corpus (161 `.s`/`.block`
  files), available the same way it already is for `voyager-core`'s own full-corpus
  validation (`001-voyager-script-parser/research.md` §3) — path referred to below
  as `$CORPUS`.

## 1. Build

```powershell
cargo build -p voyager-core -p drut-cli
```

Expected: builds cleanly, zero `cargo clippy -p drut-cli` warnings (same bar as
`voyager-core`'s existing zero-warning clippy gate).

## 2. `check` on a clean corpus — validates SC-001

```powershell
cargo run -p drut-cli --bin drut -- check $CORPUS
echo $LASTEXITCODE
```

Expected: no diagnostic lines printed, exit code `0`. This is the CLI-level
reproduction of `voyager-core`'s already-proven 161/161-clean result
(`001-voyager-script-parser/research.md` §3) — the Definition of Done requires this
to hold through the CLI itself, not just the library.

## 3. `check` on a directory with a broken fixture — validates SC-002, SC-006

```powershell
cargo run -p drut-cli --bin drut -- check crates\voyager-core\tests\fixtures\broken
echo $LASTEXITCODE
```

Expected: at least one diagnostic line naming a file, location, kind, and message;
exit code `1` (not `0`, not `2` — every fixture file here is readable, just
structurally broken).

## 4. `check --format=sarif` — validates SC-003

```powershell
cargo run -p drut-cli --bin drut -- check $CORPUS --format=sarif > out.sarif
```

Expected: `out.sarif` is a single well-formed SARIF 2.1.0 JSON document (one `run`,
empty `results` for the clean corpus). Validate it against the official SARIF 2.1.0
schema — this is exactly what `tests/sarif_schema.rs` automates with the
`jsonschema` crate (research.md §4); this manual step is the same check by hand.

## 5. Exit-code precedence — validates FR-011/FR-020's `Fatal`-wins rule

```powershell
cargo run -p drut-cli --bin drut -- check .\does-not-exist
echo $LASTEXITCODE
```

Expected: exit code `2`, with a message naming the invalid path — distinct from
both `0` and `1`.

## 6. `format` idempotency and behavior preservation — validates SC-004, SC-005

This is proven exhaustively by `cargo test -p voyager-core --test format_corpus`
(golden-file + idempotency + structural-equivalence checks over the full corpus,
per FR-021) and, through the CLI itself, by
`cargo test -p drut-cli --test fixture_corpus_e2e -- --ignored` (T033 — which,
notably, runs against a **temporary copy** of `$CORPUS`, never `$CORPUS` itself,
for exactly the reason the next paragraph explains). A hand-run spot check:

**⚠️ `--write` overwrites files in place — don't point it at your only copy of
`$CORPUS`.** Copy it first (`Copy-Item -Recurse $CORPUS $SCRATCH`), then:

```powershell
cargo run -p drut-cli --bin drut -- format $SCRATCH --write
cargo run -p drut-cli --bin drut -- format $SCRATCH --check
echo $LASTEXITCODE
```

Expected: the second command's exit code is `0` — after the first `--write` pass,
nothing is left for `--check` to flag (idempotency, observed at the CLI level).

## 7. `format --diff` without writing — validates FR-019

```powershell
cargo run -p drut-cli --bin drut -- format crates\voyager-core\tests\fixtures\valid --diff
```

Expected: a unified diff per file that would change (if any); no files on disk are
modified — confirm with `git status` (or an equivalent check) showing no working-
tree changes under that path.

## 8. Casing normalization is opt-in and explicit — validates FR-015

```powershell
cargo run -p drut-cli --bin drut -- format crates\voyager-core\tests\fixtures\valid --diff --casing=upper
cargo run -p drut-cli --bin drut -- format crates\voyager-core\tests\fixtures\valid --casing
echo $LASTEXITCODE
```

Expected: the first command's diff (if any) touches only control-word/keyword-name
casing, never other whitespace; the second command (bare `--casing`, no value)
exits with a usage error before touching any file.

## 9. Encoding-fallback behavior — validates SC-008 (FR-024, FR-025)

Uses the hand-written encoding-fallback fixtures `tasks.md` (T025) adds
alongside the real corpus — the real 161-file corpus never exercises either
path, so these two files exist specifically for this check:

```powershell
# Recovered: a byte that only decodes via the Windows-1252 fallback (no diagnostic).
cargo run -p drut-cli --bin drut -- format crates\voyager-core\tests\fixtures\encoding_fallback\recovered.s --write
echo $LASTEXITCODE

# Lossy: a byte undecodable under either encoding (InvalidEncoding diagnostic).
cargo run -p drut-cli --bin drut -- format crates\voyager-core\tests\fixtures\encoding_fallback\lossy.s --write
echo $LASTEXITCODE
git status crates\voyager-core\tests\fixtures\encoding_fallback\lossy.s
```

Expected:
- The `recovered.s` run exits `0`, the file on disk is rewritten with that byte in
  decoded UTF-8 form, and the command's output includes a visible line naming this
  file as encoding-normalized — with no other flag (not only under `--diff`).
- The `lossy.s` run exits `2`, the command reports this file was refused for
  safety, and `git status` shows **no working-tree change** to `lossy.s` — the
  refusal must hold even though `--write` was passed.
- Repeating the `lossy.s` run with `--check`, `--diff`, or no flag at all
  (instead of `--write`) still exits `2` and still flags the file — the refusal
  signal doesn't depend on `--write` having been the mode used.

## 10. Full test suite

```powershell
cargo test -p voyager-core
cargo test -p drut-cli
cargo clippy -p voyager-core -p drut-cli
```

Expected: all green, zero clippy warnings — the actual CI gate this quickstart's
manual steps are a human-readable proxy for.

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2 | SC-001 |
| 3 | SC-002 |
| 4 | SC-003 |
| 2, 3, 5 | SC-006 |
| 6 | SC-004, SC-005 |
| (timed run of step 2) | SC-007 |
| 9 | SC-008 |
