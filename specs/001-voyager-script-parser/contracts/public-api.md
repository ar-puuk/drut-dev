# Contract: `voyager-core` Public API

This is the contract downstream adapters (CLI, LSP server, MCP server, formatter — per
constitution Principle I) rely on. It is a conceptual signature contract, not final
Rust source; exact naming is decided during implementation but must preserve these
shapes and guarantees.

## Entry points

```text
fn tokenize(source: &str) -> Vec<Token>
fn parse(source: &str) -> ParseResult
```

- **Input**: `source` is the full text of one `.s` or `.block` file, already read
  into memory by the caller. The crate never reads a file, opens a socket, or
  otherwise performs I/O itself (FR-001) — this is the contract's central guarantee.
- **`tokenize`**: returns the flat token stream (see data-model.md § Token). Useful on
  its own for editor-style features that only need lexical detail (spec User Story 3)
  without paying for full structural parsing.
- **`parse`**: returns the full `ParseResult` (see data-model.md § ParseResult) —
  the statement/block tree plus the diagnostic list. This is the primary entry point
  for structural consumers (spec User Story 1 and 2).
- **No panics**: Neither function panics on any `&str` input, including empty input,
  input that is only comments/whitespace, or arbitrarily malformed script text.
  Malformed input produces diagnostics, not a panic or an `Err` that aborts the whole
  call (there is no `Result` return type at this boundary — errors are data in
  `ParseResult.diagnostics`, per FR-012–FR-018).
- **Determinism**: Calling `parse` twice on identical input produces an identical
  `ParseResult` (no reliance on ambient state, clock, locale, or file paths) — this
  is required for the fixture-corpus test to be meaningful at all, and for LSP-style
  callers to safely re-parse on every edit.
- **Case sensitivity**: Control words and keywords are matched case-insensitively
  (FR-011). The token/statement's `text`/original casing is preserved in the returned
  data even though matching ignores case, so a future formatter (out of scope this
  phase) can still see what casing the author actually used.

## What this contract does *not* promise (by design, this phase)

- No per-program-box keyword validation (e.g. it does not know that `RUN PGM=MATRIX`
  takes a `ZONES=` keyword) — FR-019.
- No semantic or reference checking (e.g. it does not know whether `@AOC_Auto@` is
  ever defined) — FR-019.
- No formatting output — FR-019.
- No file-inclusion resolution (`READ FILE='...'` is parsed as an ordinary control
  statement with a `FILE=` keyword; the referenced file is never opened) — this
  phase's scope is one file's text at a time.
- No streaming/incremental re-parse API — whole-document in, whole-document result
  out, only (see data-model.md § ParseResult validation rule).

## Stability expectations for adapters

Because this crate is the single source of truth (constitution Principle I), any
adapter (CLI/LSP/MCP/formatter) built in a later phase depends directly on these two
entry points and the `Token`/`ParseResult`/`Diagnostic` shapes in data-model.md.
Breaking changes to this contract are a breaking change for every adapter
simultaneously — there is no adapter-local grammar to fall back on.
