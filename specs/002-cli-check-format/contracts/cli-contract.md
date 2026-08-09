# Contract: `drut` Command-Line Interface

This is the contract for `drut`'s invocation surface — subcommands, flags, and exit
codes. It's the CLI-adapter analogue of `001-voyager-script-parser/contracts/
public-api.md`: downstream consumers (CI pipelines, pre-commit hooks, humans) rely
on this shape; exact flag short-names are an implementation detail as long as these
long-form names and behaviors hold.

## `drut check <path>`

```text
drut check <PATH> [--format=text|sarif]
```

| Argument/flag | Required | Default | Behavior |
|---|---|---|---|
| `<PATH>` | Yes | — | A file or directory (FR-001) |
| `--format` | No | `text` | `text` = plain-text diagnostic listing (FR-008); `sarif` = SARIF 2.1.0 log on stdout (FR-009). Default is `text` in every context, interactive or not (FR-010) |

**Output (text mode)**: One line per diagnostic (minimum), each showing the file
path, location, `DiagnosticKind`, and message (FR-008). Zero diagnostics produces no
per-diagnostic output, but MUST still make the "clean" outcome distinguishable from
"the run didn't complete" (e.g. via exit code alone, per SC-006 — text output itself
isn't required to say "0 diagnostics" explicitly).

**Output (SARIF mode)**: A single SARIF 2.1.0 JSON document on stdout — see
`sarif-mapping.md`. Nothing else is written to stdout in this mode, so a caller can
pipe it directly into a SARIF-consuming tool.

**Exit codes**: See the shared table below.

## `drut format <path>`

```text
drut format <PATH> [--write | --check | --diff] [--casing=upper|lower]
```

| Argument/flag | Required | Default | Behavior |
|---|---|---|---|
| `<PATH>` | Yes | — | Same traversal/filtering rules as `check` (FR-001–FR-003) |
| `--write` | No | off | Overwrite each matched file in place with its formatted content (FR-017) |
| `--check` | No | off | Report which files would change; write nothing; print no full file content (FR-018) |
| `--diff` | No | off | Print a unified diff per changed file; write nothing (FR-019) |
| `--casing` | No | unset (off) | `upper` or `lower`; when given, must have exactly one of these two values — no bare `--casing` and no other value (FR-015; spec Edge Cases) |

**Mutual exclusivity**: `--write`, `--check`, and `--diff` are mutually exclusive —
passing more than one is a usage error, not a run that "picks one." Omitting all
three is the default mode: print each matched file's formatted content to stdout
(FR-016) — for a single-file target this is a self-contained formatted file; for a
directory target, per-file content is concatenated with a file-boundary marker (see
plan.md Assumptions carried from spec.md).

**`--casing` usage error**: `--casing` with no value, or a value outside
`upper`/`lower`, exits with a usage error before any file is touched — this is a
`clap`-level parse failure, not one of the three run-outcome exit codes below (spec
Edge Cases: "the command exits with a usage error before touching any file").
**Implementation note**: `clap`'s own usage-error exit code happens to also be `2`
— the same numeric value the table below uses for `Fatal` — which is a coincidence
of two independent conventions, not a designed distinction; the guarantee this
contract actually makes is "no file touched," not "a numerically distinct code."
A caller that must tell the two apart needs another signal (e.g. `clap`'s own
usage message on stderr), not the bare exit code.

**Encoding-fallback reporting is automatic, not flag-gated** (FR-024, FR-025 — see
"Encoding-fallback behavior" below): every mode, including plain `drut format
<path>` with no flags at all, surfaces a matched file whose bytes needed FR-034's
decode fallback. There is no opt-out flag for this reporting.

**Exit codes**: See the shared table below, using the `format`-specific outcome
names.

## Encoding-fallback behavior (`format` only; FR-024, FR-025)

`format`/`format_bytes` (`001-voyager-script-parser`'s FR-034) can decode a byte
two different ways, and `drut format` treats them differently — in every mode, not
just `--write`:

| `EncodingFidelity` | What happened | `drut format`'s behavior |
|---|---|---|
| `Recovered` | A byte decoded only via the Windows-1252 fallback, no diagnostic | The file is formatted and, under `--write`, written with that byte in decoded UTF-8 form — this is FR-013(b)'s one named content exception. **The run's output names this file explicitly** (e.g. "N file(s) had legacy-encoding bytes normalized to UTF-8"), in every mode — default, `--write`, `--check`, and `--diff` alike, so this byte-level change is never a side effect discovered only by re-diffing later (FR-024). |
| `Lossy` | A byte decoded under neither UTF-8 nor Windows-1252, replaced with the Unicode replacement character (`InvalidEncoding` diagnostic) | `drut format` refuses to persist this file — under `--write`, the file on disk is left completely unchanged and the write is not attempted. This refusal is reported the same way in **every** mode, including `--check`/`--diff`/default, which never write anyway: even there, the file is flagged distinctly from an ordinary "would reformat"/"changed" result, since it tells the caller `--write` would refuse if used (FR-025). |

A `Faithful` file (no decode fallback needed at all — the common case; the real
161-file corpus never triggers either fallback) is unaffected by either row above.

## Shared exit-code contract

Both subcommands report exactly one of three outcomes via process exit code, so a
caller never has to parse output text to know which happened (SC-006):

| Exit code | `check` meaning | `format` meaning |
|---|---|---|
| `0` | Every matched file read successfully; zero diagnostics (FR-011a) | Every matched file read (and, for `--write`, written) successfully; nothing needed a change, or `--write` applied every needed change (FR-020a). A `Recovered`-encoding file (see "Encoding-fallback behavior" above) does **not** by itself prevent `0` — it's reported, not treated as a failure. |
| `1` | Every matched file read successfully; at least one diagnostic found (FR-011b) | `--check` mode found at least one file that would change (FR-020b). *(In default/`--write`/`--diff` modes, finding changes is not itself a failure — this code is specific to `--check`'s "would this pass CI" question.)* |
| `2` | The given path was invalid, or at least one matched file could not be read (FR-011c) | Same as `check`'s `2`, plus: a matched file could not be written under `--write` (FR-020c), **or a matched file's decoding is `Lossy` and `format` refused to write it — regardless of which mode encountered the file** (FR-025). This is the one case where `--check`/`--diff`/default can still exit `2`: it signals "`--write` would refuse here" even on a run that never attempted a write. |

**Precedence**: If a run would otherwise qualify for both `1` and `2` (e.g. one file
has a diagnostic and a different file couldn't be read, or a `format --check` run
finds both an ordinary reformat candidate and a `Lossy` file), `2` wins
(FR-011/FR-020).

## Traversal/filtering behavior (both subcommands)

- Recurses through directories, honoring `.gitignore` (including nested
  `.gitignore` files) the same way `git` would (FR-002).
- Only `.s`/`.block` files (case-insensitive extension) are opened, read, or
  reported on; every other file — including known binary Cube types (`.mat`,
  `.net`, `.dbd`, `.prj`) and anything else — is skipped without comment (FR-003).
- An empty directory, or one with no matching files, is a `0`-exit "clean" run, not
  an error (spec Edge Cases).

## What this contract does *not* promise (by design, this phase)

- No configuration-file-driven defaults (spec Assumptions: "no configuration file
  support in this phase") — every flag must be given explicitly per invocation.
- No `--fix` / auto-fix flag for `check` — `check` only reports; `format` is the
  only subcommand that changes files, and only under `--write`.
- No multiple `<PATH>` arguments in one invocation — one file-or-directory target
  per run, matching the spec's own FR-001 wording.
- No streaming/incremental output — both subcommands complete the full traversal
  before producing their report (matching `voyager-core`'s own whole-document
  contract).
- No override flag to force-write a `Lossy`-encoding file anyway, and no flag to
  suppress the `Recovered`-encoding summary reporting — both are unconditional
  (FR-024, FR-025); a future phase could add an explicit `--force` if a real need
  emerges, but this phase doesn't build one.
