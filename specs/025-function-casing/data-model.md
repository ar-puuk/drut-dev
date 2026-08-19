# Data Model: Function-Call Casing Normalization

## 1. `FunctionCallEntry` / `FUNCTION_CALL_ENTRIES` (new, `voyager-core::function_call`)

Mirrors `data_reference.rs`'s `DataReferenceEntry`/`DATA_REFERENCE_ENTRIES` exactly in
shape:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionCallEntry {
    pub name: &'static str, // canonical uppercase spelling
}
```

**Membership**: the 138 names from `024-function-call-highlighting/research.md` §2 /
`data-model.md` §1 verbatim — Numeric (26), Trigonometric (6), Character/String (20),
Highway/Matrix (21), Public Transport skims (19), CONVERGE-phase iteration statistics
(42), CUBE Cluster utility (3), corpus-confirmed `PRINTPROGRESS` (1). Not re-listed here
in full — single source of truth is `024`'s `research.md` §2; this module's own
`FUNCTION_CALL_ENTRIES` table doc comment cites it directly rather than re-deriving it,
the same "provenance note" convention `keywords.rs`'s own module docs already use for
`PAIR_KEYWORDS`.

**Public API** (mirrors `data_reference_entries()`):

```rust
pub fn function_call_entries() -> &'static [FunctionCallEntry];
pub(crate) fn is_function_call_name(text: &str) -> bool; // case-insensitive exact match
```

## 2. `FunctionCallOccurrence` (new, `voyager-core::function_call`)

Mirrors `DataReferenceOccurrence` exactly in shape:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCallOccurrence {
    pub name: String,  // canonical uppercase spelling
    pub span: Span,     // covers exactly the matched name -- never the "(" itself
}

pub fn function_call_occurrences(nodes: &[Node]) -> Vec<FunctionCallOccurrence>;
```

**Implementation-time simplification** (no `lines` parameter, unlike
`data_reference_occurrences(nodes, lines)`): `lines` exists on that function only to
recover a block opener's own pair-keyword-name *text* via `text_at_span` (`Block` keeps
only `Span`s for `opener_pairs`, not original tokens). This module never scans
`opener_pairs` at all (§2 below) — a function call is never a pair-keyword name itself
(disjoint trigger conditions, research.md §3) — so there is nothing in that position to
recover text for, and an unused parameter would fail this workspace's `-D warnings`
clippy gate.

**Match rule** (research.md §2–§4): a `Word` token, found outside a single-/double-quoted
run (quote-tracking identical to `data_reference.rs`'s `collect_tokens`), whose text
case-insensitively equals a `FUNCTION_CALL_ENTRIES` name, **and** is immediately followed
by a `TokenKind::Punctuation` token with text `"("` whose `span.start` exactly equals the
matched token's own `span.end` (zero-width gap — no dot-notation prefix matching, unlike
`data_reference.rs`'s `dot_notation_prefix_len`; a function name is never dot-prefixed).

**Scope**: both `Control` and `Assignment` statements (research.md §2) — `Label`/
`ShellEscape` excluded, mirroring `data_reference.rs`'s own `collect_statement` scope
exactly.

## 3. `CasingSettings.function_calls` (amends `voyager-core::format::CasingSettings`)

```rust
pub struct CasingSettings {
    pub control_words: CasingConvention,
    pub pair_keywords: CasingConvention,
    pub data_references: CasingConvention,
    pub function_calls: CasingConvention,  // NEW
}
```

Same `CasingConvention` enum (`Preserve` `#[default]` / `Upper` / `Lower`) every other
field already uses — no new enum, no new value shape.

## 4. Casing-edit wiring (amends `voyager-core::format::render`)

`render()` gains one more call, structurally identical to its existing
`data_reference_occurrences` call (`format.rs`'s existing gate:
`for occurrence in data_reference::data_reference_occurrences(nodes, &char_lines) { ... }`):

```rust
if options.casing.function_calls != CasingConvention::Preserve {
    for occurrence in function_call::function_call_occurrences(nodes) {
        push_if_present(&mut casing_edits, &char_lines, &protected, occurrence.span, options.casing.function_calls);
    }
}
```

Gated behind a `!= Preserve` check, the same performance-only early-out
`collect_casing_edits`'s own call site already uses for the other three fields
(`format.rs`'s existing `if options.casing.control_words != CasingConvention::Preserve
|| ...` gate extends to include `function_calls`).

## 5. Adapter surface additions (no new type shapes — direct mirrors)

| Crate | File(s) | Addition |
|---|---|---|
| `drut-config` | `lib.rs`, `parse.rs` | `casing_function_calls: Option<CasingConvention>` field, same merge/parse/default handling as `casing_pair_keywords` |
| `drut-cli` | `cli.rs`, `format_cmd.rs`, `lib.rs` | `--casing-function-calls` flag (`Option<CasingArg>`), same wiring as `--casing-pair-keywords` |
| `drut-mcp` | `format` tool schema/handler | `casing_function_calls` parameter, same pattern as the existing three |
| `editors/vscode` | `package.json`, client-settings passthrough | `drut.format.casingFunctionCalls` setting, same generic passthrough mechanism `829d065` (editor client-settings support) already established for every `[format]` field — **not** a grammar/highlighting change (FR-009) |
