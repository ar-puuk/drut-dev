# Phase 1 Data Model: Drut CLI — `check` and `format` Subcommands

This feature's entities split across the two crates it touches. `voyager-core`
gains a small set of formatting types (§1); `drut-cli` owns everything about
invocation, traversal, aggregation, and reporting (§2–§5). Types already defined by
`001-voyager-script-parser` (`Token`, `Statement`, `Block`, `ParseResult`,
`Diagnostic`, `DiagnosticKind`, `Span`, `Position`) are referenced, not redefined.

## 1. `voyager-core` additions

### CasingConvention

The two supported keyword-casing targets (spec Assumptions — no hardcoded default,
FR-015).

| Value | Meaning |
|---|---|
| `Upper` | Rewrite matched control words and keyword names to `ALLCAPS` |
| `Lower` | Rewrite matched control words and keyword names to lowercase |

**Validation rule**: There is no `None`/default variant on this enum itself —
"casing normalization off" is represented one level up, by `FormatOptions.casing`
being `Option<CasingConvention>` with the `None` state, not by a variant here. This
keeps "off" from ever being confused with a third normalization target.

### FormatOptions

Caller-supplied configuration for one `format`/`format_bytes` call.

| Field | Type | Notes |
|---|---|---|
| `casing` | `Option<CasingConvention>` | `None` (default) leaves all keyword/control-word casing untouched, exactly as originally written (FR-015) |

### EncodingFidelity

How `format`/`format_bytes`'s decoding of the input relates to what's safe to
persist back to disk (FR-013(b), FR-024, FR-025). Always `Faithful` for `format`
(the `&str` entry point), since a `&str` is already valid UTF-8 by construction —
only `format_bytes` can produce the other two variants.

| Value | Meaning |
|---|---|
| `Faithful` | Decoding needed no fallback at all — `text`'s bytes reconstruct the input exactly, modulo whatever whitespace/casing normalization `format` itself performs |
| `Recovered` | At least one byte needed (and succeeded under) FR-034's Windows-1252 fallback, producing no diagnostic — `text` is a faithful *character* representation, but persisting it re-encodes those specific bytes as UTF-8, changing the file's raw bytes at those positions (FR-013(b)'s narrow carve-out) |
| `Lossy` | At least one byte was undecodable under either encoding and was replaced with the Unicode replacement character (`InvalidEncoding` diagnostic present) — `text` has lost information at that position; MUST NOT be persisted over the original file (FR-025) |

**Validation rule**: `Lossy` iff `diagnostics` contains at least one
`InvalidEncoding`; else `Recovered` iff decoding needed the Windows-1252 fallback
for at least one byte (whether or not that byte also happened to be one already
counted toward `Lossy` — the two aren't mutually exclusive at the byte level, but
`Lossy` takes precedence for classification purposes since it's the stricter,
write-blocking condition); else `Faithful`.

### FormatResult

The aggregate value returned by `format`/`format_bytes` for one input file's text —
deliberately parallel in shape to `ParseResult`.

| Field | Type | Notes |
|---|---|---|
| `text` | `String` | The fully re-rendered source text — whitespace-normalized per FR-012's canonical form, and casing-normalized only if `FormatOptions.casing` was `Some` |
| `changed` | `bool` | `true` iff `text.as_bytes()` differs from the original input's raw bytes at all — a byte-level comparison against the actual input, not an intermediate decoded form, so a file whose *only* difference is an `EncodingFidelity::Recovered` re-encoding (no whitespace/casing change) still reports `changed: true` |
| `diagnostics` | `Vec<Diagnostic>` | Whatever `parse`/`parse_bytes` would have reported for this input (FR-034's `InvalidEncoding`, plus any structural diagnostics) — formatting proceeds on a best-effort basis over whatever structure was recovered, the same way `parse` itself keeps going past a diagnosed defect (FR-018 in `001-voyager-script-parser`) |
| `encoding_fidelity` | `EncodingFidelity` | See above. `format_bytes` still computes and returns a best-effort `text` even when this is `Lossy` — consistent with the crate's never-refuses-to-run contract — but a `Lossy` result MUST NOT be treated as safe to persist; that policy decision (refuse under `--write`) belongs to the CLI (spec FR-025), not to this function refusing to run |

**Validation rules**:
- `format(format(x).text).text == format(x).text` for any `x` (idempotency, FR-014)
  — formatting `FormatResult.text` again produces the same `text`, `changed: false`,
  and `encoding_fidelity: Faithful` (re-formatting already-UTF-8 `text` never
  re-triggers a decode fallback, since `text` is a `String`).
- The statement/block structure obtained by parsing `text` is identical to the
  structure obtained by parsing the original input, except for `Span` positions
  shifting to match any whitespace-width changes and, for the tokens actually
  affected, casing (if enabled, FR-015) or an `EncodingFidelity::Recovered`
  re-encoding (FR-013(b)) — see spec.md FR-013, SC-005.
- `format_bytes` decodes its input the same way `parse_bytes` does (UTF-8 first,
  per-byte Windows-1252 fallback, FR-034) before formatting, and its
  `InvalidEncoding` diagnostics (if any) come first in `diagnostics`, same ordering
  guarantee `parse_bytes` already makes.

## 2. CLI invocation types (`drut-cli`)

### Invocation

The parsed command line for one `drut` run.

| Field | Type | Notes |
|---|---|---|
| `command` | `Command::Check { path, format }` \| `Command::Format { path, write, check, diff, casing }` | Mutually exclusive per-subcommand options; `clap` enforces `--write`/`--check`/`--diff` are not combined with each other (FR-016–FR-019) |
| `path` | `PathBuf` | The single file-or-directory argument (FR-001) |

### OutputFormat (check only)

| Value | Meaning |
|---|---|
| `Text` (default) | Plain-text diagnostic listing (FR-008, FR-010) |
| `Sarif` | SARIF 2.1.0 log on stdout (FR-009) |

## 3. Traversal types

### TraversalOutcome

The result of resolving `Invocation.path` into concrete files to process (FR-001–
FR-005).

| Field | Type | Notes |
|---|---|---|
| `matched_files` | `Vec<MatchedFile>` | Every `.s`/`.block` file found, outside `.gitignore`-excluded paths |
| `read_failures` | `Vec<ReadFailure>` | Files that matched the extension filter but couldn't be read |
| `invalid_target` | `Option<String>` | Set instead of the above two when `path` itself doesn't exist or is neither a file nor directory (FR-004) — mutually exclusive with a non-empty `matched_files`/`read_failures` from the *same* traversal |

### MatchedFile

| Field | Type | Notes |
|---|---|---|
| `path` | `PathBuf` | Absolute or invocation-relative path as walked |
| `bytes` | `Vec<u8>` | Raw file content, read once and reused for `parse_bytes`/`format_bytes` |

### ReadFailure

| Field | Type | Notes |
|---|---|---|
| `path` | `PathBuf` | The file that matched the extension filter but couldn't be opened/read |
| `message` | `String` | The underlying I/O error, for display (FR-005) |

**Validation rule**: Traversal never opens, reads, or reports on a file whose
extension isn't `.s`/`.block` (case-insensitive) or that `.gitignore` excludes
(FR-003) — such files simply never become a `MatchedFile` or a `ReadFailure`.

## 4. `check` report types

### CheckReport

| Field | Type | Notes |
|---|---|---|
| `diagnostics` | `Vec<(PathBuf, Diagnostic)>` | Every diagnostic from every matched file's `parse_bytes()` call, tagged with its source file (FR-006, FR-007) |
| `read_failures` | `Vec<ReadFailure>` | Carried through from `TraversalOutcome` |

**Derived exit outcome** (FR-011):
- `Clean` iff `diagnostics.is_empty() && read_failures.is_empty()` and the target
  path itself was valid.
- `DiagnosticsFound` iff `diagnostics` is non-empty and `read_failures` is empty.
- `Fatal` iff `read_failures` is non-empty, or `TraversalOutcome.invalid_target` was
  set — takes precedence over `DiagnosticsFound` when both would otherwise apply.

### SarifLog

A SARIF 2.1.0 document derived one-for-one from a `CheckReport` (FR-009); see
`contracts/sarif-mapping.md` for the `DiagnosticKind` → `ruleId`/`level` mapping.
Not hand-modeled here beyond that mapping — its shape is a hand-written,
`#[derive(Serialize)]` struct set covering exactly the fields the mapping table
needs, serialized via `serde_json` (research.md §4 — superseded from an
originally-planned `serde-sarif` dependency after a build-script blocker on the
implementation machine).

## 5. `format` report types

### FormatOutcome (one per matched file)

| Variant | Fields | Meaning |
|---|---|---|
| `Unchanged` | — | `FormatResult.changed` was `false`; no action taken regardless of mode |
| `Changed` | `diff: Option<String>` | `FormatResult.changed` was `true`; `diff` is populated only in `--diff` mode (a unified diff via `similar`, FR-019). This is the correct per-file classification for both `EncodingFidelity::Recovered` and `::Lossy` files whenever the mode is *not* `--write` — content did change, so the informational display (stdout/`--check` listing/`--diff`) still shows it; a `Lossy` file's write-unsafety is tracked separately via `FormatReport.unsafe_encoding_files`, not by withholding this per-file outcome |
| `Written` | — | `--write` mode, and the file was successfully overwritten |
| `WriteFailed` | `message: String` | `--write` mode, and either the overwrite failed at the OS level (e.g. permission denied) **or was refused before being attempted** because `encoding_fidelity` was `Lossy` (FR-025) — the `message` distinguishes which; both funnel to the same `Fatal` exit outcome below, so no new variant is needed for the refusal case |

### FormatReport

| Field | Type | Notes |
|---|---|---|
| `outcomes` | `Vec<(PathBuf, FormatOutcome)>` | One entry per matched file |
| `read_failures` | `Vec<ReadFailure>` | Carried through from `TraversalOutcome` |
| `unsafe_encoding_files` | `Vec<PathBuf>` | Every matched file whose `FormatResult.encoding_fidelity` was `Lossy` — populated in **every** mode, not only `--write`, so `--check`/`--diff`/default runs also surface "this file can't be safely written" (FR-025) even though they never attempt a write themselves |
| `recovered_encoding_files` | `Vec<PathBuf>` | Every matched file whose `FormatResult.encoding_fidelity` was `Recovered` — populated in every mode, backing the visible summary line FR-024 requires even under plain `--write` |

**Derived exit outcome** (FR-020), mirroring `check`'s three-way shape:
- `Clean` iff every outcome is `Unchanged` or `Written`, `unsafe_encoding_files` is
  empty, `read_failures` is empty, and the target path was valid. (A non-empty
  `recovered_encoding_files` does **not** by itself prevent `Clean` — a
  `Recovered` file that's otherwise already canonically formatted still counts as
  clean, it's just also named in the FR-024 summary line.)
- `WouldReformat` iff `--check` mode and at least one outcome is `Changed`, with
  `unsafe_encoding_files` empty and no `WriteFailed`/read failures.
- `Fatal` iff any outcome is `WriteFailed`, `unsafe_encoding_files` is non-empty
  (**regardless of mode** — FR-025 folds into this outcome the same way whether or
  not `--write` was the mode used), `read_failures` is non-empty, or
  `TraversalOutcome.invalid_target` was set — same precedence rule as `check`.
  This is the one case where a `--check`/`--diff`/default run can still exit `2`:
  a `Lossy` file means `--write` would refuse, so the run reports that fact at the
  same severity a read/write failure would, even though nothing was actually
  written this time.

## 6. Exit code mapping

Shared by both subcommands (FR-011, FR-020), so a user or CI job only has to learn
one convention (SC-006):

| Outcome | Exit code | Applies to |
|---|---|---|
| `Clean` | `0` | Both |
| `DiagnosticsFound` (check) / `WouldReformat` (format `--check`) | `1` | Both |
| `Fatal` | `2` | Both |

**Validation rule**: `Fatal` always wins if it and the code-`1` outcome would both
independently apply in the same run (spec FR-011/FR-020's stated precedence) — a
caller checking `exit_code == 2` never has to also check for diagnostics/reformat
findings to know the run itself didn't complete cleanly.
