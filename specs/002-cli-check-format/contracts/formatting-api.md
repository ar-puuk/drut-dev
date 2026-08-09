# Contract: `voyager-core` Formatting API (addition)

This extends `001-voyager-script-parser/contracts/public-api.md` with two new
entry points this feature adds to `voyager-core`. It follows that contract's own
conventions: a conceptual signature contract, not final Rust source, but the shapes
and guarantees below are binding.

## New entry points

```text
fn format(source: &str, options: FormatOptions) -> FormatResult
fn format_bytes(source: &[u8], options: FormatOptions) -> FormatResult
```

- **Input**: Same shape as `parse`/`parse_bytes` — `source` is one file's full text
  (or raw bytes), already read into memory by the caller; `options` selects
  whether, and how, keyword casing is normalized (see data-model.md § FormatOptions).
  Like `parse`/`parse_bytes`, neither function performs file I/O.
- **`format`**: Parses `source` internally (the same way `parse` would), then
  re-renders it: whitespace is normalized to the canonical form spec.md FR-012
  defines concretely (4-space-per-nesting-level indentation relative to each
  block's own opener, zero-delta closer/`ELSEIF`/`ELSE` alignment, top-level
  baseline and continuation-line indentation left untouched, comments left
  entirely untouched) — not an unspecified "canonical form," but this exact,
  corpus-derived rule set. Only if `options.casing` is `Some`, matched
  control-word/keyword-name tokens are additionally rewritten to that casing.
  Returns the rendered text plus a `changed` flag plus whatever diagnostics parsing
  `source` would have produced (see data-model.md § FormatResult).
- **`format_bytes`**: Decodes `source` the same way `parse_bytes` does (UTF-8 first,
  per-byte Windows-1252 fallback, FR-034 in `001-voyager-script-parser`) before
  formatting. Any `InvalidEncoding` diagnostics come first in the result's
  `diagnostics`, matching `parse_bytes`'s existing ordering guarantee. The decode
  outcome also drives `FormatResult.encoding_fidelity` (data-model.md §1) — see
  "Encoding safety" below.
- **No panics**: Neither function panics on any input, including malformed,
  non-UTF-8, or arbitrary binary content — the same guarantee `parse`/`parse_bytes`
  make. A structurally broken input (e.g. an unmatched `IF`) still produces a
  best-effort `text` covering whatever `voyager-core` recovered, plus the same
  diagnostics `parse` would report for it — formatting never refuses to run just
  because the input has a diagnosed defect.
- **Determinism**: Calling `format`/`format_bytes` twice on identical `source` and
  `options` produces an identical `FormatResult` — same rationale as
  `parse`/`parse_bytes`'s determinism guarantee (it's what makes the golden-file
  test suite meaningful at all).
- **Idempotency** (constitution Principle III, FR-014): `format(format(source,
  opts).text, opts).text == format(source, opts).text` — reformatting already-
  formatted text is always a no-op (`changed: false`).
- **Behavior preservation** (constitution Principle III, FR-013): Parsing
  `format(source, opts).text` yields the same statement/block structure as parsing
  `source` — same continuations, same statement/block order, same non-whitespace
  token content, with exactly two named exceptions (FR-013(a)/(b)): casing, only
  for the tokens `options.casing` targets; and, for `format_bytes` specifically, a
  byte whose `EncodingFidelity` was `Recovered` (see "Encoding safety" below).
- **Case sensitivity carried through**: Because control words/keywords are already
  matched case-insensitively during parsing (FR-011 in `001-voyager-script-parser`),
  `format`'s casing rewrite only ever changes *presentation* of an already-correctly-
  recognized token — it never changes which token was recognized or how the
  statement/block structure was built.

## Encoding safety (FR-013(b), FR-024, FR-025)

`format_bytes` decodes exactly like `parse_bytes` (FR-034), which means its input
can require one of two different fallbacks — and this contract treats them
differently rather than uniformly:

- **Recovered** (`EncodingFidelity::Recovered`, data-model.md §1): a byte decoded
  successfully only via the Windows-1252 fallback, producing no diagnostic.
  `format_bytes` persists this in `text` as the decoded UTF-8 character — this is
  FR-013's one encoding-related carve-out (FR-013(b)): the *character* is a
  faithful recovery, so re-encoding it is not treated as "altering meaningful
  content," even though the file's exact raw bytes at that position do change if
  the caller writes `text` back to disk. Every consumer of `format_bytes` MUST
  surface this occurrence to the user rather than let it pass silently (FR-024) —
  this contract does not itself perform that reporting (it has no I/O), but a
  caller that discards `encoding_fidelity` without reporting it violates FR-024.
- **Lossy** (`EncodingFidelity::Lossy`): a byte undecodable under either encoding,
  replaced with the Unicode replacement character (`InvalidEncoding` diagnostic
  present, same as `parse_bytes`). `format_bytes` still computes and returns a
  best-effort `text` for this case too — consistent with this crate's
  never-refuses-to-run contract, the same guarantee `parse`/`parse_bytes` already
  make for any input — but that `text` MUST NOT be treated as safe to persist over
  the original file: the replacement character is a lossy substitution, not a
  faithful re-encoding, so writing it back is a real, unrecoverable content change
  FR-013 does not carve out. **This crate does not refuse to compute a result** —
  refusing to *write* one is a policy decision for the caller (spec.md FR-025
  requires `drut-cli` to refuse under `--write`), exactly the same
  core-computes/adapter-decides split constitution Principle I already draws
  between grammar/parsing logic and I/O policy.

## What this contract does *not* promise (by design, this phase)

- No reflow of line length / wrapping — this phase's "whitespace normalization" is
  spacing/indentation only, not line-width-driven rewrapping.
- No fix-it/suggested-edit data beyond the rendered `text` itself and the unified
  diff the CLI computes externally (via `similar`) between the original input and
  `text` — `FormatResult` itself carries no diff.
- No configurable indentation width/style beyond the one canonical form FR-012
  establishes (4-space-per-level, block-relative) — casing is the only
  configurable axis (FR-015).
- No refusal-to-run for any input, including `Lossy`-fidelity input — see
  "Encoding safety" above for why that's a caller-side policy, not a change to
  this function's own never-refuses contract.
- No partial/single-statement formatting API — same whole-document-in,
  whole-document-out shape as `parse`/`parse_bytes`.

## Stability expectations for adapters

`drut-cli`'s `format` subcommand is the first, but not necessarily only, future
consumer of `format`/`format_bytes` — a later LSP server's format-on-save (per the
constitution's Technology & Architecture Constraints) is expected to call the same
entry points rather than re-implementing rendering. Breaking changes to this
contract are a breaking change for every such consumer simultaneously, same as
`public-api.md`'s existing stability note for `parse`/`parse_bytes`.
