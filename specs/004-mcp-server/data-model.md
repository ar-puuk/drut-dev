# Phase 1 Data Model: Drut MCP Server

Entities split across two crates. `voyager-core` gains one new, protocol-agnostic
entity (§1 — the extraction research.md §5 requires). `drut-mcp` owns every
MCP-facing tool input/output DTO (§2–§6), converting from `voyager-core`'s native
types at its own boundary (research.md §6), the same pattern `drut-lsp` already
established. Types already defined by `001`/`002`/`003` (`Diagnostic`,
`DiagnosticKind`, `Span`, `Position`, `FormatResult`, `FormatOptions`,
`CasingConvention`, `KeywordEntry`, `KeywordRole`, `CompletionContext`,
`completion_candidates`, `did_you_mean`, `Node`, `Block`, `BlockKind`,
`ParseResult`) are referenced, not redefined.

## 1. `voyager-core` addition — block-position resolution

### `BlockInfo`

The result of asking "which block (if any) encloses this position, and where is
its matched counterpart" — the same fact `drut-lsp`'s hover capability has always
derived, now computed by one shared function instead of being private to that one
adapter (research.md §5).

| Field | Type | Notes |
|---|---|---|
| `kind` | `BlockKindName` | Which of the seven block kinds (§1.1) |
| `is_short_if` | `bool` | `true` only when `kind == If` and this is a self-closing short-`IF` (no separate closer by construction) — mirrors `drut-lsp` hover's existing `is_short_if` fact |
| `counterpart` | `Option<Span>` | The resolved matched-counterpart location, per the same 5-rule derivation `003-lsp-vscode-extension/data-model.md` §4 already documents (`Block.closer` when present; `None` for a short-`IF` or a genuinely unmatched `If`/`Loop`/`JLoop`/`LinkLoop`/`DistributeMultistep`; the block's own resolved body extent for an implicitly-closed `Run` or unconditionally for `Process`) |

### `BlockKindName`

A small, protocol-agnostic enum naming which of the seven block kinds `BlockInfo.kind`
is — `If`, `Loop`, `Run`, `Process`, `JLoop`, `LinkLoop`, `DistributeMultistep` —
mirrors `drut-lsp/src/hover.rs`'s existing (currently private) `block_kind_name`
mapping, now the single shared source both `drut-lsp` and `drut-mcp` read from.

### `block_at`

```text
fn block_at(nodes: &[Node], diagnostics: &[Diagnostic], pos: Position) -> Option<BlockInfo>
```

- Recursively locates the innermost block whose opener or closer line contains
  `pos` (the same line-match approximation `003-lsp-vscode-extension/data-model.md`
  §4 already documents and justifies).
- Returns `None` when no block encloses `pos` — a normal, successful "nothing
  here" result for callers, not an error (both `drut-lsp`'s hover and, per
  FR-007, `drut-mcp`'s structural-query tool treat this the same way).
- `drut-lsp/src/hover.rs`'s `handle` becomes a thin wrapper: calls `block_at`,
  then translates `BlockInfo` into `lsp_types::Hover` markdown — no behavior
  change for any existing `drut-lsp` test, since the derivation itself is
  moved, not altered.

## 2. `drut-mcp` — shared conventions

Every tool accepts script content as **either** inline text **or** a file path,
never both (FR-002) — modeled as one shared input shape every tool's own input
struct includes:

### `ScriptSource`

| Field | Type | Notes |
|---|---|---|
| `text` | `Option<String>` | Inline script content |
| `path` | `Option<String>` | A file path to read script content from |

Exactly one of `text`/`path` MUST be set; both set or both unset is a structured
tool-call error (FR-002, Edge Cases), never a silent "prefer one" guess.
`InvalidEncoding` is reachable only via `path` (reading real bytes off disk,
`voyager_core::parse_bytes`/`format_bytes`), never via `text` (an MCP tool-call
argument is JSON, which cannot carry an invalid byte sequence any more than an
LSP payload can — spec.md Edge Cases, `003-lsp-vscode-extension/research.md`
§12's identical reasoning for why `InvalidEncoding` is unreachable through live
LSP editing).

## 3. Diagnostics tool

### `DiagnosticsInput`

| Field | Type | Notes |
|---|---|---|
| `source` | `ScriptSource` | §2 |

### `DiagnosticDto` (result element)

| Field | Type | Notes |
|---|---|---|
| `category` | `String` | One of the six reachable `DiagnosticKind` names (`UnmatchedIf`, `UnmatchedLoop`, `UnclosedBlockComment`, `InvalidContinuation`, `UnmatchedRun`, `MisplacedBreak`) plus `InvalidEncoding` when reachable via a `path` input |
| `message` | `String` | `voyager-core`'s own diagnostic message text, unmodified |
| `start_line` / `start_column` / `end_line` / `end_column` | `u32` | `voyager-core`'s own `Span` fields, flattened — 1-based, `char`-counted, exactly as `voyager-core` already defines them (no UTF-16 translation; that translation is specifically an LSP wire-protocol concern `003`'s `position.rs` owns, not something this feature's tool results need) |

## 4. Formatting tool

### `FormatInput`

| Field | Type | Notes |
|---|---|---|
| `source` | `ScriptSource` | §2 |
| `casing` | `Option<String>` | `"upper"` / `"lower"` / absent — maps to `CasingConvention`; absent means `FormatOptions::default()` (untouched casing, FR-005) |

### `FormatResultDto`

| Field | Type | Notes |
|---|---|---|
| `text` | `String` | The fully reformatted text |
| `changed` | `bool` | Byte-level comparison against the original input (FR-004) |
| `encoding_fidelity` | `String` | `"faithful"` / `"recovered"` / `"lossy"` — `voyager-core`'s own `EncodingFidelity`, always `"faithful"` for a `text`-sourced input |

## 5. Structural-query tool

### `StructuralQueryInput`

| Field | Type | Notes |
|---|---|---|
| `source` | `ScriptSource` | §2 |
| `line` | `u32` | 1-based, matching `voyager-core::Position`'s own convention — no UTF-16 translation needed or performed (§4's rationale applies identically here) |
| `column` | `u32` | 1-based `char` count |

### `BlockInfoDto` (result)

Directly mirrors §1's `BlockInfo`, flattened for JSON: `kind: Option<String>`
(`None`/absent when `block_at` returns `None` — FR-007's "no enclosing block is
a normal result"), `is_short_if: bool`, and `counterpart_start_line`/
`counterpart_start_column`/`counterpart_end_line`/`counterpart_end_column:
Option<u32>` (all absent together when `counterpart` is `None`).

## 6. Keyword-lookup tool

### `KeywordLookupInput`

| Field | Type | Notes |
|---|---|---|
| `enclosing_control_word` | `Option<String>` | Passed directly by the caller (resolved design question, not derived from a script+position — see spec.md) |
| `spellcheck_token` | `Option<String>` | If present, also run `did_you_mean` against this token |

At least one of `enclosing_control_word`/`spellcheck_token` being meaningfully
actionable is not required — `enclosing_control_word: None` is itself a valid,
meaningful request (FR-008's fallback case), not an error.

### `KeywordCandidateDto` (result element, one per candidate)

| Field | Type | Notes |
|---|---|---|
| `name` | `String` | Mirrors `KeywordEntry.name` |
| `role` | `String` | `"control_word"` / `"pair_keyword"` — mirrors `KeywordRole` |

### `SpellCheckSuggestionDto` (result, present only when `spellcheck_token` was supplied)

| Field | Type | Notes |
|---|---|---|
| `suggestion` | `Option<String>` | The suggested correct spelling, or `None` when the token already exactly matches a real keyword or no unique close match exists within threshold (mirrors `did_you_mean`'s existing `Option` return exactly) |
