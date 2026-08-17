# Phase 1 Data Model: Token Hover Shows Assigned Value

No new persistent state (matches `013`'s own "no storage" pattern — spec.md's own
Assumptions: reads are always fresh, never cached). This feature adds one new
`voyager-core` module of pure types/functions, plus one small new `drut-lsp`
helper. Everything below operates on data already produced by existing,
unmodified machinery (`voyager_core::parse`, `ServerState`, `workspace::uri_to_path`).

## `voyager-core::token_resolution` (new module)

### `VariableRefAt`

| Field | Type | Meaning |
|---|---|---|
| `name` | `String` | The token's name, exactly as written between the `@`s (casing preserved — matching is case-insensitive, but the name itself is not normalized). |
| `span` | `Span` | The full `@name@` reference's span, including both `@` delimiters. |

```rust
pub fn variable_ref_at(nodes: &[Node], pos: Position) -> Option<VariableRefAt>
```

Walks every `Statement.tokens` (recursing into `Block`'s own nested statements the
same way `block_resolution.rs`'s existing traversal already does) looking for a
`TokenKind::VariableRef` token whose `span` contains `pos`. Returns the first (and,
by construction, only — spans don't overlap) match. `None` if the hovered position
isn't over any `@name@` reference.

### `Assignment` (resolution-facing view, distinct from `statement::StatementKind::Assignment`)

| Field | Type | Meaning |
|---|---|---|
| `target` | `&str` | The assignment's target name (borrowed from the underlying `Statement`). |
| `value_span` | `Span` | The span of just the value portion (right-hand side), for `text_for_span`. |
| `statement_span` | `Span` | The whole statement's span, for "assigned at line N" reporting. |

```rust
pub fn all_assignments(nodes: &[Node]) -> Vec<Assignment<'_>>
```

Flattened, source-order walk of every `StatementKind::Assignment` in `nodes`,
regardless of nesting depth inside `IF`/`LOOP`/etc. blocks (a token assignment
inside a conditional block is still a real assignment Voyager will execute if that
branch runs — this function doesn't attempt to reason about whether a branch is
"reachable"; it only reports positions, and FR-004's own ordering rule — most
recent before the hover position — already handles the common real-world case
correctly without needing branch analysis).

### `ReadFileRef` (resolution-facing view of a `Control { word: "READ", .. }` statement)

| Field | Type | Meaning |
|---|---|---|
| `literal_value_span` | `Option<Span>` | `Some(span)` — the merged span of the `FILE` pair's raw value tokens, quote characters included — if that value contains no `VariableRef` token; `None` if dynamic (token-built) or the pair has no value. Deliberately a `Span`, not a reconstructed `String` — see `Assignment.value_span`'s own rationale above; the caller slices real source text and strips quotes itself. |
| `statement_span` | `Span` | The `READ FILE` statement's own span — this is the "effective position" every assignment found in the target file is stamped with for ordering purposes (spec.md FR-004). |

```rust
pub fn read_file_refs(nodes: &[Node]) -> Vec<ReadFileRef>
```

Source-order walk for every `Control` statement whose `word` case-insensitively
equals `"READ"` and that has a pair whose keyword case-insensitively equals
`"FILE"`. `drut-lsp` filters this list down to `literal_value_span.is_some()`
entries before attempting any disk read (spec.md FR-003/FR-007) — a `None` entry
is never resolved, per the permanent token-built-path exclusion.

### `ResolvedTokenValue`

| Field | Type | Meaning |
|---|---|---|
| `value_span` | `Span` | Span of the winning assignment's value, for `text_for_span`. |
| `statement_span` | `Span` | Span of the winning assignment's whole statement, for "assigned at line N". |
| `source` | `Source` | Where the winning assignment came from. |

```rust
pub enum Source {
    /// Found directly in the document passed as `nodes`.
    SameFile,
    /// Found in one of the `included` files, identified by the `READ FILE`
    /// statement's own span in the *original* document (`nodes`) — not by
    /// path or file name, keeping this module I/O- and filesystem-naming-free.
    ReadFile { read_file_statement_span: Span },
}

pub fn resolve_token_value<'a>(
    nodes: &'a [Node],
    pos: Position,
    included: &'a [(Span, Vec<Node>)],
    name: &str,
) -> Option<ResolvedTokenValue>
```

Combines `all_assignments(nodes)` (each keeping its own real `statement_span` for
ordering) with, for each `(read_file_span, included_nodes)` pair in `included`,
every `all_assignments(&included_nodes)` entry — each of *those* instead compared
for ordering purposes using `read_file_span` (spec.md FR-004's interleaving rule),
while still reporting its own real `value_span`/`statement_span` for display.
Filters this combined set to entries whose target case-insensitively matches `name`
(research.md §8) and whose *ordering position* is at or before `pos`, then returns
the one with the latest ordering position — `None` if the set is empty.

**Validation rules** (all upheld by construction, never by runtime assertion —
matching `span.rs`'s own "never panic by construction" precedent):
- `included`'s `Span` values are always spans of *actual* `READ FILE` statements
  found by a prior `read_file_refs(nodes)` call on the same `nodes` — `drut-lsp` is
  responsible for this pairing; `resolve_token_value` does not itself re-derive
  which spans are valid `READ FILE` positions, it only uses each one as a
  comparison key.
- An empty `included` slice degrades this function to same-file-only resolution
  (spec.md User Story 1) with no special-casing needed — the combine step is a
  no-op when there's nothing to combine.

## `drut-lsp::position` (existing module, one new helper)

```rust
pub fn text_for_span(text: &str, span: Span) -> String
```

Slices `text` for the substring `span` covers, using the same `text.lines().nth(..)`
+ `char`-walking approach `to_lsp_position` already uses internally (research.md
§3) — never panics on an out-of-range span (clamped the same way `to_lsp_position`
already clamps, consistent with this module's existing "never panic" guarantee for
stale/out-of-sync positions).

## `drut-lsp::hover` (existing module, new control flow)

No new stored state. New control flow only:

```
handle(state, params)
   │
   ├─▶ variable_ref_at(doc.parse_result.nodes, pos)?   (research.md §6, tried first)
   │        │
   │        ├─▶ Some(var_ref) — attempt token-value resolution:
   │        │        │
   │        │        ├─▶ read_file_refs(doc.parse_result.nodes)
   │        │        │        .filter(|r| r.literal_path.is_some())
   │        │        │        .filter_map(|r| {
   │        │        │            resolve path relative to doc's own directory
   │        │        │            (workspace::uri_to_path(uri)?.parent(), research.md §7);
   │        │        │            std::fs::read(..) + voyager_core::parse_bytes(..);
   │        │        │            Some((r.statement_span, parsed.nodes))
   │        │        │            — None (silently) on any read/parse failure (FR-007)
   │        │        │        })
   │        │        │        .collect::<Vec<_>>()  →  `included`
   │        │        │
   │        │        └─▶ resolve_token_value(nodes, pos, &included, &var_ref.name)
   │        │                 │
   │        │                 ├─▶ Some(resolved) → render hover markdown
   │        │                 │        (value via text_for_span on the right
   │        │                 │         source text for `resolved.source`;
   │        │                 │         "assigned at line N [in <file>]")
   │        │                 │
   │        │                 └─▶ None → fall through to block_at (unchanged)
   │        │
   │        └─▶ None (not hovering a @token@ at all) → fall through to block_at
   │                 (unchanged — today's exact behavior)
   │
   ▼
(unchanged from today below this point: block_at, then spellcheck::hint_for)
```

Every box that existed before this feature (`block_at`, `spellcheck::hint_for`) is
completely unmodified in its own internal logic — this feature only adds a new
branch tried before them, per spec.md FR-010.
