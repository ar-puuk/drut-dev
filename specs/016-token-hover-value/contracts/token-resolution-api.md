# Contract: `voyager-core::token_resolution`

New public module, `crates/voyager-core/src/token_resolution.rs`, re-exported from
`lib.rs` alongside the existing `block_resolution` re-exports. Pure, I/O-free —
upholds the same "never panics on any input" guarantee `tokenize`/`parse` already
make (constitution Principle I; `CLAUDE.md`'s public-contract note that neither
public function may panic on malformed input).

## Types

```rust
pub struct VariableRefAt {
    pub name: String,
    pub span: Span,
}

pub struct Assignment<'a> {
    pub target: &'a str,
    pub value_span: Span,
    pub statement_span: Span,
}

pub struct ReadFileRef {
    pub literal_value_span: Option<Span>,
    pub statement_span: Span,
}

pub enum Source {
    SameFile,
    ReadFile { read_file_statement_span: Span },
}

pub struct ResolvedTokenValue {
    pub value_span: Span,
    pub statement_span: Span,
    pub source: Source,
}
```

## Functions

### `variable_ref_at`

```rust
pub fn variable_ref_at(nodes: &[Node], pos: Position) -> Option<VariableRefAt>
```

- Returns `None` if `pos` is not over any `@name@` reference anywhere in `nodes`
  (including inside nested blocks).
- Never panics for any `nodes`/`pos` combination, including a `pos` entirely
  outside the document's real range (matches `block_at`'s own existing guarantee
  for the same kind of input).

### `all_assignments`

```rust
pub fn all_assignments(nodes: &[Node]) -> Vec<Assignment<'_>>
```

- Returns every `StatementKind::Assignment` in `nodes`, source-order, at any
  nesting depth. Empty `Vec` for a document with none — never `None`/panic.

### `read_file_refs`

```rust
pub fn read_file_refs(nodes: &[Node]) -> Vec<ReadFileRef>
```

- Returns every `READ FILE = ...`-shaped `Control` statement in `nodes`,
  source-order, at any nesting depth. `literal_value_span` is `Some` (the merged
  span of the `FILE` pair's raw value tokens, quote punctuation included) only
  when that value contains no `VariableRef` token (research.md §2) — a
  `READ FILE` with a token-built path still appears in this list (with
  `literal_value_span: None`), so a caller can distinguish "no `READ FILE` here
  at all" from "a `READ FILE` exists but its path can't be statically resolved"
  if it ever needs to (this feature's own caller filters the latter out per
  spec.md FR-003, but the function itself doesn't hide it).
- **Deliberately returns a `Span`, not a reconstructed `String`** — the same
  reason `Assignment` returns `value_span` rather than a joined string
  (research.md §3): the lexer splits a quoted value on internal whitespace into
  multiple tokens (confirmed real: quoted `READ FILE` targets in
  `WF-TDM-Development` contain space-bearing directory names), so naively
  joining token text would silently drop those spaces. The caller slices the
  real source substring via `text_for_span` and strips the single pair of
  surrounding quote characters, if present, itself.

### `resolve_token_value`

```rust
pub fn resolve_token_value<'a>(
    nodes: &'a [Node],
    pos: Position,
    included: &'a [(Span, Vec<Node>)],
    name: &str,
) -> Option<ResolvedTokenValue>
```

- `name` matching is case-insensitive (research.md §8).
- Ordering: a same-file assignment is compared by its own real position; an
  included-file assignment is compared by its originating `(Span, _)` entry's
  span (the `READ FILE` statement's own position in `nodes`) — never by its own
  real position within the included file (spec.md FR-004's interleaving rule).
- Only assignments whose ordering position is at or before `pos` are eligible —
  an assignment that would only take effect "later" than the hover position
  (same-file, or via a `READ FILE` statement appearing after `pos`) is never
  selected (spec.md US1 Acceptance Scenario 3, US2 Acceptance Scenario 2's "own
  later value wins" case, and the `READ FILE`-after-hover edge case).
- Returns `None` if no eligible assignment exists — never fabricates or guesses
  a near-match (spec.md FR-008, US3).
- `included` is caller-assembled; this function does no filesystem or path work
  and does not itself re-validate that each `Span` came from a real `READ FILE`
  statement in `nodes` (data-model.md's stated caller responsibility).

## Non-goals (explicitly out of contract)

- No recursion into an included file's own `READ FILE` statements — `included`'s
  `Vec<Node>` entries are used only for their own direct `all_assignments`, never
  scanned for further `read_file_refs` (spec.md's permanent one-level scope
  boundary).
- No reverse ("who reads me") resolution — this module only ever operates
  forward, over whatever `nodes`/`included` the caller supplies for one
  document's own perspective.
- No token-built (`@...@`-containing) `READ FILE` path evaluation of any kind —
  `literal_path: None` entries are informational only; nothing in this module
  attempts to resolve what such a path might mean.

---

# Contract: `drut-lsp::position::text_for_span`

```rust
pub fn text_for_span(text: &str, span: Span) -> String
```

- Returns the substring of `text` covered by `span`, using the same line/`char`
  semantics `to_lsp_position`/`from_lsp_position` already use (1-based line,
  `char`-count column).
- Never panics for a `span` outside `text`'s real range — clamps the same way
  `to_lsp_position` already clamps for an out-of-range `Position` (this module's
  existing, established guarantee, extended to a second entry point rather than
  given a new/different failure mode).

---

# Contract: `drut-lsp::hover` (extended, not replaced)

`handle`'s existing signature, request shape, and response shape
(`lsp_types::Hover`) are unchanged — this is a new internal branch, not a new
capability or request type (spec.md Assumptions; constitution Principle VI).

## New behavior

- If the hover position is over an `@token@` reference (`variable_ref_at` returns
  `Some`) **and** `resolve_token_value` finds a value for it, the hover response's
  markdown includes that value and where it was assigned (spec.md FR-009) —
  format left to implementation, but MUST include both the concrete value text
  (via `text_for_span`) and a line-number reference, and, for a cross-file result,
  MUST name the source file distinctly from a same-file result (spec.md US2
  Acceptance Scenario 1).
- If the hover position is over an `@token@` reference but no value resolves
  (`resolve_token_value` returns `None`), `handle` falls through to today's
  existing `block_at` → `spellcheck::hint_for` chain, completely unchanged (spec.md
  FR-008, US3).
- If the hover position is not over an `@token@` reference at all
  (`variable_ref_at` returns `None`), behavior is **byte-for-byte identical** to
  today — this feature adds no new code path for that case (spec.md FR-010).

## Disk access (new — first time `drut-lsp` reads a file it didn't open)

- For each `read_file_refs` entry with `literal_value_span: Some(span)` found in
  the hovered document: slice the raw value text via `text_for_span(&doc.text,
  span)`, strip one leading and one trailing quote character if both are
  present and matching (`'...'` or `"..."`), then resolve the result as a path
  relative to the hovered document's own directory (`workspace::
  uri_to_path(uri)?.parent()`, research.md §7); on any failure at any step
  (`uri_to_path` returns `None` — e.g. an unsaved/untitled buffer; `std::
  fs::read` fails — file missing, permission denied; the bytes don't parse
  meaningfully) that specific `READ FILE` entry is simply omitted from
  `included` — never an error surfaced to the user, never a panic (spec.md
  FR-007, Edge Cases).
- Uses `voyager_core::parse_bytes` (not `parse`) for the target file's raw bytes —
  unlike an open LSP document's `text` (guaranteed valid UTF-8 by the protocol
  layer), a file read directly off disk carries no such guarantee (research.md
  §4).
- This read is always fresh — no cache of a previously-read `READ FILE` target
  persists across hover requests (spec.md Assumptions).
