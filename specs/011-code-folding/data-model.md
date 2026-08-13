# Phase 1 Data Model: Code Folding Support

No new persistent data of any kind — this feature computes a value derived entirely
from existing `voyager-core` structure, per request, and discards it. The entities
below describe that computation's shape, not stored state.

## `voyager-core::block_resolution::BlockFold` (new)

```rust
pub struct BlockFold {
    pub opener: Position,
    pub info: BlockInfo,
}
```

| Field | Type | Meaning |
|---|---|---|
| `opener` | `Position` (1-based line/column, existing type) | The block's own opener location — `Block.span.start`, unchanged from how `Block` already stores it. |
| `info` | `BlockInfo` (existing type, unmodified) | Exactly what `block_at` already returns for this block: `kind` (`BlockKindName`), `is_short_if`, `counterpart: Option<Span>`. |

**Produced by**: `pub fn all_blocks(nodes: &[Node], diagnostics: &[Diagnostic]) ->
Vec<BlockFold>` (new function, `block_resolution.rs` — research.md §1). One entry per
`Node::Block` encountered anywhere in the document's structure, including nested
blocks and blocks inside `If` branches. Order is not contractually meaningful (folding
consumers don't care about order — every fold range is independent).

**Validation rules**: None beyond what `Block`/`BlockInfo` already guarantee by
construction (`Span.end` never before `Span.start`, etc.) — this is a pure aggregation
of already-validated facts.

**State transitions**: N/A — recomputed fresh from `nodes`/`diagnostics` on every call,
never mutated or cached (FR-010).

## `lsp_types::FoldingRange` (existing type, `drut-lsp`'s output shape)

| Field | Value this feature sets | Notes |
|---|---|---|
| `start_line` | Opener's (or block comment's opening) 0-based line, via `to_lsp_position` | Kept visible when collapsed, per standard editor folding UI. |
| `start_character` | `None` | Line-based folding only (research.md §3) — matches FR-002 through FR-008's line-granularity phrasing. |
| `end_line` | Resolved counterpart's (or block comment's closing) 0-based line | Also kept visible; content strictly between the two lines collapses. |
| `end_character` | `None` | Same as `start_character`. |
| `kind` | `Some(FoldingRangeKind::Region)` for blocks, `Some(FoldingRangeKind::Comment)` for block comments | Matches FR-006's "standard comment-kind folding convention" and lets clients' "Fold All Comments" / "Fold All Regions" commands distinguish the two. |
| `collapsed_text` | `None` | No custom collapsed-text label — editor default (e.g. `{...}` or `...`) is used, matching every other `drut-lsp` capability's "no custom UI beyond the protocol default" posture. |

## Derivation flow (no new intermediate storage)

```
doc.text (ServerState)
   │
   ├─▶ doc.parse_result.nodes, doc.parse_result.diagnostics
   │        │
   │        └─▶ voyager_core::block_resolution::all_blocks(nodes, diagnostics)
   │                 │
   │                 └─▶ Vec<BlockFold>  (filter: counterpart.is_some())
   │                          │
   │                          └─▶ filter: start_line < end_line (FR-008)
   │                                   │
   └─▶ voyager_core::tokenize(&doc.text)                                     │
            │                                                                 │
            └─▶ filter TokenKind::BlockComment { unterminated: false }       │
                     │                                                        │
                     └─▶ span → FoldingRange { kind: Comment }                │
                              │                                               │
                              └─▶ filter: start_line < end_line (FR-008)      │
                                                                               ▼
                                                          both streams merged → Vec<lsp_types::FoldingRange>
                                                          (folding.rs::handle, returned as the request's Result)
```

**The `start_line < end_line` filter (FR-008) applies to both streams independently,
in the same place, before either is returned** — not only to blocks. For blocks this
is defensive today (no block kind's rules currently produce a same-line
`counterpart`); for block comments it is load-bearing (a single-line `/* note */`
comment genuinely has `span.start.line == span.end.line`, and nothing upstream of
this filter excludes it — see research.md §5).

Both streams (blocks, block comments) are independent and computed from the same
already-available `doc` fields every other `drut-lsp` handler reads — no new field is
added to `document_store.rs`'s `Document`/`ServerState`.
