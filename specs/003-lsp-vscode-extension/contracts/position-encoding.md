# Contract: Position Encoding Translation

This codifies research.md §1's resolved decision as an explicit contract, so it's
a single documented boundary rather than logic that could be reinvented ad hoc in
each handler (`diagnostics.rs`, `hover.rs`, `completion.rs`, `semantic_tokens.rs`).

## Ownership

`voyager-core::Span`/`Position` are **unchanged** by this feature — they continue
counting Unicode scalar values (`char`s), 1-based, per line
(`crates/voyager-core/src/span.rs`). All translation to/from the LSP wire format
(UTF-16 code units, 0-based) happens in exactly one place: `drut-lsp/src/
position.rs`.

## Entry points

```text
fn to_lsp_position(text: &str, pos: voyager_core::Position) -> lsp_types::Position
fn from_lsp_position(text: &str, pos: lsp_types::Position) -> voyager_core::Position
fn to_lsp_range(text: &str, span: voyager_core::Span) -> lsp_types::Range
```

- **Input**: `text` is the full current document content the position/span is
  relative to (`OpenDocument.text`) — both directions need the source text itself
  to count UTF-16 units correctly, since neither `Position` type stores a flat
  offset.
- **`to_lsp_position`**: Finds `pos.line`'s text (1-based → the `(pos.line - 1)`-th
  line of `text`). Iterates that line's `char`s up to (not including) the
  `pos.column`-th one (1-based), summing `char::len_utf16()` for each. Returns
  `lsp_types::Position { line: pos.line - 1, character: <that sum> }`.
- **`from_lsp_position`**: The inverse walk — iterates the target line's `char`s,
  accumulating `char::len_utf16()` per `char` until the running total reaches
  `pos.character`, and reports the 1-based `char` index reached as `column`, with
  `line: pos.line + 1`.
- **`to_lsp_range`**: `Range { start: to_lsp_position(text, span.start), end:
  to_lsp_position(text, span.end) }` — no independent logic beyond the two calls.

## Guarantees

- **Every** `voyager_core::Position`/`Span` value crossing the LSP boundary
  (diagnostics, hover, completion-item insert ranges, semantic-token positions)
  goes through one of these three functions — no handler computes a position
  translation independently (data-model.md/contracts' handler descriptions all
  reference this contract rather than restating the algorithm).
- Correct for supplementary-plane characters (FR-020): a `char` whose
  `len_utf16() == 2` advances `character`/the running total by 2, not 1 —
  verified by `drut-lsp/tests/position_encoding.rs` against fixtures containing
  such characters (spec.md Edge Cases).
- `lsp-server`'s declared `PositionEncodingKind` (in `InitializeResult
  .capabilities.position_encoding`) is always `Utf16` — never omitted-as-`None`
  and never any other value — since `vscode-languageclient` rejects anything else
  (research.md §1, point 4). This is a fixed constant in `drut-lsp`'s
  initialization response, not a negotiated value, even though `lsp-types`
  models the field as configurable.
- Never panics: an out-of-range `pos.column`/`pos.character` (e.g. a stale
  position from a client that hasn't yet received a `didChange`-triggered
  re-diagnosis) is clamped to the line's actual length rather than indexing past
  it — consistent with the crate-wide no-panic guarantee this feature inherits
  from `voyager-core` (FR-004) and extends to `drut-lsp` itself.

## Explicitly out of scope

- Grapheme-cluster-aware positioning (e.g. treating a multi-codepoint emoji
  sequence as one cursor stop) — LSP itself is defined in terms of UTF-16 code
  units, not graphemes; this contract matches the protocol, not visual cursor
  behavior.
- Caching per-line UTF-16 prefix sums — each call re-scans the target line
  (research.md §1's "revisit only if profiling shows otherwise" note).
