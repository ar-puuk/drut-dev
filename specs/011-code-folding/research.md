# Phase 0 Research: Code Folding Support

All findings below are measured against the real, current codebase
(`crates/voyager-core/src/block_resolution.rs`, `block.rs`, `token.rs`, `lexer.rs`,
`lib.rs`; `crates/drut-lsp/src/hover.rs`, `semantic_tokens.rs`, `lib.rs`,
`position.rs`), not estimated. The dependency claim in §3 is verified directly against
the vendored `lsp-types 0.97.0` source, not assumed from memory.

## §1. Direct answer to the owner's pre-`/speckit-tasks` question

**Question asked**: does folding-range computation genuinely call `block_resolution.rs`
as-is with zero new logic, or does it need adaptation — and if so, is that adaptation
purely a data-shape translation at the LSP boundary, or does it touch
`block_resolution.rs` itself?

**Answer**: The *derivation rules* (five-rule `counterpart_for`, `is_short_if`,
`block_kind_name`) are reused with **zero changes of any kind** — not rewritten,
not wrapped, not even lightly touched. But reaching them requires one small, additive
change to `block_resolution.rs` itself, not a change purely contained to the LSP
boundary. Here's why, and why the alternative that avoids touching `voyager-core`
entirely was rejected:

**The problem**: `block_resolution.rs`'s only public entry point today is

```rust
pub fn block_at(nodes: &[Node], diagnostics: &[Diagnostic], pos: Position) -> Option<BlockInfo>
```

— a **single-position query** ("what block, if any, encloses this one position").
Folding needs the opposite shape: **every** block in the document, all at once. The
three private helper functions that actually compute the interesting facts
(`counterpart_for`, `is_short_if`, `block_kind_name`) are not `pub` — they're free
functions inside `block_resolution.rs`, reachable only through `block_at`'s single-
position search (`find_block_at`). `drut-lsp` cannot call them directly today at any
number of call sites, one or many.

**Two alternatives were considered**:

- **(a) Zero `voyager-core` changes — call `block_at` once per block, from `drut-lsp`'s
  own tree walk.** `drut-lsp` already has real precedent for walking `Node`/`Block`/
  `BlockKind` directly using the already-`pub` types (`semantic_tokens.rs`'s own
  `walk` function does exactly this, to find short-IF and unreachable-statement
  spans). Folding could do the same walk to discover every block's own opener
  position, then call the existing `voyager_core::block_at(nodes, diagnostics,
  block.span.start)` once per discovered block to get its `BlockInfo` — genuinely
  zero new `voyager-core` code. Verified this produces the correct result: passing a
  block's own opener line as `pos` makes `find_block_at`'s recursion (which checks
  nested children first) correctly fall through to matching that exact block, since
  no nested child's own opener line can coincide with its parent's opener line.
  **Rejected** because it does two things this project's own precedent treats as a
  smell: it silently becomes O(n²) (a full-tree `find_block_at` search launched once
  per block, `n` blocks deep), and — more importantly per Principle I — it stands up
  a *second*, independent tree-walking implementation of "how do you find every block
  in this structure" in the adapter layer, duplicating a traversal shape
  `block_resolution.rs` already owns (`find_block_at`), even though the *decisions*
  made during that walk would still come from the reused private helpers. This is a
  weaker form of the same "grammar-adjacent logic split across two places" problem
  Principle I exists to prevent, not a clean escape from it.
- **(b) One new `pub` enumeration function in `block_resolution.rs`.** A single-pass
  traversal — structurally the same shape as `find_block_at`'s own recursion, just
  collecting every block instead of returning the first position match — calling the
  exact same three private helpers, unchanged, once per block instead of once per
  query. **Chosen.** This keeps exactly one tree-walking implementation of "block
  structure" in the codebase (in `voyager-core`, where Principle I says it belongs),
  costs a genuinely small amount of new code (a traversal wrapper, not a new rule),
  and is O(n) instead of O(n²).

**Decision**: Add to `block_resolution.rs`:

```rust
pub struct BlockFold {
    /// The block's own opener location (`Block.span.start`) — folding needs this;
    /// `BlockInfo` alone doesn't carry it, since `block_at`'s caller already knows
    /// the position they queried.
    pub opener: Position,
    pub info: BlockInfo,
}

pub fn all_blocks(nodes: &[Node], diagnostics: &[Diagnostic]) -> Vec<BlockFold>
```

— a recursive walk over `nodes` (mirroring `find_block_at`'s own recursion into
`block.children` and, for `If`, each branch's `children`), pushing one `BlockFold` per
`Node::Block` encountered, computed via the exact same `block_kind_name`,
`is_short_if`, and `counterpart_for` calls `block_at` already makes — literally the
same three function calls, just looped instead of short-circuited on first match.
`BlockInfo` itself is **not modified** — `hover.rs` and `drut-mcp`'s `query_structure`
keep consuming `block_at`/`BlockInfo` exactly as today, completely unaffected by this
addition.

**So, to directly restate the answer**: this is not "purely a data-shape translation at
the LSP boundary" — it does reach back into `voyager-core`, by design, because
Principle I requires the traversal-that-makes-block-matching-decisions to live in
exactly one place, and today's public surface doesn't yet expose an enumeration shape.
The reach-back is minimal and additive (one new function, zero changed lines in the
existing five-rule derivation), not a redesign.

## §2. Block-comment folding needs no `voyager-core` change at all

Unlike blocks, block-comment folding genuinely is a pure `drut-lsp`-boundary
translation, with no new `voyager-core` code:

- `tokenize(source) -> Vec<Token>` is already `pub` from `voyager-core`'s crate root.
- `TokenKind::BlockComment { unterminated: bool }` already carries exactly the
  information needed: `unterminated: false` is FR-006's foldable case; `unterminated:
  true` is FR-007's "no fold, matches `UnclosedBlockComment`'s own diagnosed case."
- `Token.span` already gives the comment's opening-line-to-closing-line extent
  directly — no new field, no new derivation.
- Nested `/* */` handling (per the lexer's own nested-comment support) is already
  baked into how a single `BlockComment` token's span is computed at the lexer level
  — `drut-lsp` never sees the inner nesting at all, it only sees one token per
  outermost comment, which is exactly the Edge Cases section's "one fold range per
  comment token" decision (spec.md).

`folding.rs` calls `voyager_core::tokenize(&doc.text)` directly (the same pattern
`010`'s `format.rs` uses internally, and the same "tokenize once per call, it's cheap
at this document scale" precedent every other feature in this codebase already
relies on), filters for `BlockComment { unterminated: false }`, and maps each token's
`span` straight into a `FoldingRange` via the existing `to_lsp_position` helper — no
new `voyager-core` surface required for this half of the feature.

## §3. `lsp-types` 0.97.0 already has everything needed — no dependency bump

Verified directly against the vendored crate source
(`~/.cargo/registry/src/.../lsp-types-0.97.0/src/folding_range.rs` and `request.rs`):

- `lsp_types::FoldingRange { start_line: u32, start_character: Option<u32>, end_line:
  u32, end_character: Option<u32>, kind: Option<FoldingRangeKind>, collapsed_text:
  Option<String> }` — exactly the shape needed; `start_character`/`end_character` are
  left `None` (line-based folding, matching every real editor's default behavior and
  this feature's own line-granularity requirements — FR-002 through FR-008 are all
  phrased in terms of lines, never columns).
- `FoldingRangeKind::{Comment, Imports, Region}` — `Comment` is used for block-comment
  ranges (matches FR-006's "standard comment-kind folding convention" wording
  directly); block/loop/run/process ranges use `Region` (the standard "this is a
  structural region, not a comment" kind — matches every general-purpose language
  server's own convention for control-flow-block folding).
- `FoldingRangeProviderCapability` (an enum: `Simple(bool)` is sufficient here, same
  pattern `hover_provider: Some(HoverProviderCapability::Simple(true))` already uses
  in `lib.rs`) and `request::FoldingRangeRequest` (`METHOD = "textDocument/
  foldingRange"`, `Params = FoldingRangeParams`, `Result = Option<Vec<FoldingRange>>`)
  are both already present. No `Cargo.toml` version bump needed anywhere in the
  workspace.

**Alternatives considered**: None — this is a direct verification task, not a design
choice; the crate already in use ships the exact standard-LSP-3.17 folding types
needed.

## §4. Line-position translation reuses `position.rs`'s existing helper unchanged

`to_lsp_position(text, pos) -> lsp_types::Position` (`crates/drut-lsp/src/
position.rs`) already converts `voyager-core`'s 1-based `Position` into LSP's 0-based
line number, with existing clamping behavior for out-of-range input (FR-004 of
`003-lsp-vscode-extension`). `folding.rs` needs only the `.line` field of that result
for both `start_line` and `end_line` — no new translation logic, and per
`position.rs`'s own doc comment ("No handler reimplements this independently"),
reusing it here rather than hand-rolling a second line-index computation is the
established pattern every other handler already follows.

**Fold-span line semantics** (verified against real editor behavior, since the LSP
spec text itself is not reproduced here per constitution Principle II): `start_line`
is the opener's own line (kept visible when collapsed, per every mainstream LSP
client's standard folding UI) and `end_line` is the resolved counterpart's own line
(also kept visible — content strictly *between* the two is what collapses). This
directly matches spec.md's own Acceptance Scenario 1 wording ("every line between,
exclusive of both, is hidden") without needing any off-by-one adjustment: `start_line
= to_lsp_position(text, block.opener).line`, `end_line = to_lsp_position(text,
counterpart_or_comment_end).line`.

## §5. Zero-span guard (FR-008) falls out of a single line-number comparison — applied to *both* streams

FR-008 ("never report a fold range spanning only a single line") is satisfied by one
guard applied at the point `folding.rs` builds each `FoldingRange`, in **both** the
block stream and the block-comment stream: skip whenever `start_line >= end_line`.

**For blocks**, this check is defensive-in-practice today: a short-`IF`'s
`BlockInfo.counterpart` is already `None` (rule 2/3 of `counterpart_for`, unchanged),
so it never reaches the guard at all — it's filtered out one step earlier, alongside
every other `counterpart: None` case (unmatched blocks, FR-005). No block kind's
current rules produce a same-line `counterpart`, so for the block stream this guard
exists as a second layer in case a future block kind's rules ever did, not because any
does today.

**For block comments, this same guard is load-bearing today, not merely defensive.**
A single-line block comment (`/* note */`, opening `/*` and closing `*/` on the same
physical line) is a real, common, currently-possible case — `TokenKind::BlockComment`'s
`span.start.line == span.end.line` whenever the comment doesn't cross a line break.
Unlike the block stream, there is no upstream filter (analogous to `counterpart:
None`) that already excludes this case before it would reach the guard — a
single-line comment is `unterminated: false` just like a multi-line one, so §2's
`unterminated: false` filter alone does not exclude it. **The `start_line >=
end_line` check must therefore be applied to the comment-derived `FoldingRange`s
exactly as it is to the block-derived ones, in the same place, before either stream
is returned** — this is not an optional refinement, it's required for FR-006/FR-008
to compose correctly (an earlier draft of this feature's design documents described
this guard as applying only to blocks, which would have shipped a nonsensical
same-line "fold" for every single-line block comment in a real script; caught during
`/speckit-analyze` review and corrected here and in `contracts/folding-range-api.md`,
`data-model.md`, and `spec.md`'s FR-006/FR-008/SC-002).

## §6. No new `DiagnosticKind`, no lexer/parser/grammar change

Confirmed directly: this feature reads existing `ParseResult.nodes`,
`ParseResult.diagnostics` (only to detect the *absence* of `UnmatchedIf`/
`UnmatchedRun`, exactly as `block_resolution.rs` already does — no new diagnostic
category needed, matching FR-005's "no resolvable counterpart" already being
observable via `BlockInfo.counterpart == None`), and `tokenize`'s existing
`BlockComment` token. Zero new `Node`/`Block`/`Statement`/`Token`/`Diagnostic` shapes.
This is a smaller core-crate footprint than `010` (which still added a new
`FormatResult` field and a new standalone function) — here, the one `voyager-core`
addition is a single enumeration function with no new data shape of its own beyond
the small `BlockFold` wrapper struct.
