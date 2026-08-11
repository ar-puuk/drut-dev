# Contract: `voyager-core` Block-Resolution Public API

A new addition to `voyager-core`'s existing public contract
(`001-voyager-script-parser/contracts/public-api.md`) — this section supplements
that file rather than replacing it. Extracted from `drut-lsp/src/hover.rs`
(research.md §5), not new logic — the derivation itself is unchanged, only its
location and reachability.

## Entry point

```text
fn block_at(nodes: &[Node], diagnostics: &[Diagnostic], pos: Position) -> Option<BlockInfo>
```

See `data-model.md` §1 for `BlockInfo`/`BlockKindName`'s exact shape.

- **Input**: `nodes` and `diagnostics` are exactly what `ParseResult` already
  carries (`result.nodes`, `result.diagnostics`) — this function takes them
  destructured rather than a whole `ParseResult` only because `drut-lsp`'s
  existing call sites already have them destructured that way; either shape is
  a one-line adaptation for a caller, not a meaningful contract choice.
  `pos` is a `voyager-core::Position` — 1-based line, 1-based `char`-counted
  column, the crate's own native convention, never a UTF-16 position (that
  translation is `drut-lsp`'s own boundary concern, per
  `003-lsp-vscode-extension/contracts/position-encoding.md`, and stays there —
  this function is not where it happens).
- **Output**: `None` when no block encloses `pos` — a normal, valid result, not
  an error or a signal of malformed input.
- **The 5-rule counterpart derivation is unchanged, byte-for-byte, from
  `003-lsp-vscode-extension/data-model.md` §4** — this contract does not
  redefine those rules, only relocates the code that implements them. Any
  future change to the derivation rules themselves happens here, in
  `voyager-core`, and is inherited automatically by every caller (today:
  `drut-lsp`'s hover, `drut-mcp`'s structural-query tool) rather than needing
  to be ported to each adapter separately.
- **No panics**: never panics on any input, including a position beyond the
  document's actual extent (per `voyager-core`'s crate-wide no-panic
  guarantee) — clamping/out-of-range handling for a *caller's* stale position
  is that caller's own translation-boundary concern (e.g. `drut-lsp`'s
  `position.rs` already clamps before ever constructing the `Position` this
  function receives), not something this function itself needs to guard
  against beyond simply returning `None` for a position matching nothing.
- **Determinism**: pure and side-effect-free, same input always produces the
  same output — required for both callers' own tests to be meaningful.

## What this contract does *not* promise (unchanged from before the extraction)

- **Still a best-effort approximation for `Process` blocks specifically**
  (`003-lsp-vscode-extension/spec.md`'s dated Assumptions entry on this exact
  point still applies unchanged — the derivation was moved, not tightened).
- **Still no per-program-box semantic knowledge** — `block_at` reports
  structural facts only (which block kind, where its counterpart is), never
  anything about what a specific `PGM=`/`PHASE=` value means.

## Stability expectations for adapters

`drut-lsp`'s hover capability and `drut-mcp`'s structural-query tool both
depend on this one entry point and `BlockInfo`/`BlockKindName`'s shape — the
same single-source-of-truth guarantee `contracts/keyword-dictionary-api.md`
already states for `completion_candidates`/`did_you_mean`. Any future adapter
needing "what block encloses this position" calls this function directly;
none may re-derive the 5-rule logic independently (constitution Principle I).
