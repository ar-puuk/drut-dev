# Contract: Folding Range

Covers both the new `voyager-core` entry point and the `drut-lsp`
`textDocument/foldingRange` handler built on top of it.

## `voyager-core::block_resolution::all_blocks`

```rust
pub struct BlockFold {
    pub opener: Position,
    pub info: BlockInfo,
}

pub fn all_blocks(nodes: &[Node], diagnostics: &[Diagnostic]) -> Vec<BlockFold>
```

- **Never panics** on any `nodes`/`diagnostics` input, including an empty document
  (`nodes: &[]` returns `vec![]`) — same never-panic guarantee every other
  `voyager-core` public function already carries.
- **Deterministic**: identical input produces an identical (same elements; order not
  contractually significant) output every call — no ambient state.
- **One `BlockFold` per `Node::Block`** anywhere in the tree, including blocks nested
  inside other blocks' `children` and blocks nested inside an `If`'s branch
  `children` — the same recursive reach `find_block_at` already has, just collecting
  instead of short-circuiting.
- **`info.counterpart == None`** for: a short-`IF` (`is_short_if == true`), a
  genuinely unmatched block of any kind that supports being unmatched, i.e. would
  produce `UnmatchedIf`/`UnmatchedLoop`/`UnmatchedRun`. Callers filter on this to
  implement FR-004/FR-005 — `all_blocks` itself does not filter; it reports every
  block's true resolved state, mirroring `block_at`'s own "None is a normal result,
  not an error" contract.
- **Reuses, does not modify**: `counterpart_for`, `is_short_if`, `block_kind_name`
  (all pre-existing, private) — this contract adds no new derivation rule.

## `drut-lsp` capability: `textDocument/foldingRange`

**Server capability** (`lib.rs::server_capabilities`):

```rust
folding_range_provider: Some(lsp_types::FoldingRangeProviderCapability::Simple(true)),
```

**Request dispatch** (`lib.rs::handle_request`): routes
`FoldingRangeRequest::METHOD` to `folding::handle`, same `serde_json::from_value` →
`send_ok`/`send_err` pattern every existing handler already follows.

**Handler** (`crates/drut-lsp/src/folding.rs`):

```rust
pub fn handle(state: &ServerState, params: &lsp_types::FoldingRangeParams) -> Option<Vec<lsp_types::FoldingRange>>
```

- Returns `None` only when the requested document is not open in `ServerState`
  (matches `hover::handle`'s own "unknown document" behavior) — a document with zero
  foldable blocks/comments returns `Some(vec![])`, per FR-011, not `None`. `None` is
  reserved for "this document doesn't exist to me," not "this document has nothing to
  fold."
- Block ranges: `voyager_core::block_resolution::all_blocks(&doc.parse_result.nodes,
  &doc.parse_result.diagnostics)`, filtered to `info.counterpart.is_some()`, mapped
  to `FoldingRange { kind: Some(FoldingRangeKind::Region), .. }` via
  `to_lsp_position` for both endpoints, then filtered again to drop any
  `start_line >= end_line` result (FR-008 — defensive for this stream; not reachable
  by any block kind's current rules per research.md §5, but asserted structurally
  rather than by convention).
- Comment ranges: `voyager_core::tokenize(&doc.text)`, filtered to
  `TokenKind::BlockComment { unterminated: false }`, mapped to `FoldingRange { kind:
  Some(FoldingRangeKind::Comment), .. }` via the same `to_lsp_position` translation,
  **then filtered by the same `start_line >= end_line` check applied to the block
  stream above** (FR-008 — load-bearing for this stream, not merely defensive: a
  single-line block comment, e.g. `/* note */`, has `span.start.line ==
  span.end.line` and is `unterminated: false` like any other terminated comment, so
  nothing upstream of this filter excludes it — research.md §5).
- The two streams are concatenated into one `Vec<lsp_types::FoldingRange>` — no
  further sorting/deduplication contract (LSP clients accept ranges in any order; VS
  Code and every other mainstream client already sort/index them client-side).
- **Never panics**: every input path (`doc` lookup, `all_blocks`, `tokenize`,
  `to_lsp_position`) already carries a never-panic guarantee from its own layer;
  `folding::handle` introduces no new panicking operation (no indexing, no
  `.unwrap()` on option/result values derived from untrusted input).

## Non-goals (explicitly out of contract)

- No `foldingRange/refresh` server-initiated push — this server, like every other
  `drut-lsp` capability, is purely request/response; clients re-request on their own
  standard triggers (document open/change), matching FR-010.
- No `FoldingRangeKind::Imports` usage — this grammar has no import/include construct
  today.
- No per-`ELSEIF`/`ELSE`-branch fold ranges within an `If` chain — spec.md scopes
  folding to one range per top-level block (opener → resolved counterpart), not a
  sub-range per branch.
