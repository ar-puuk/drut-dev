# Contract: `drut-lsp` LSP Capabilities Surface

This is `drut-lsp`'s contract with any LSP client (the `editors/vscode/` extension
in this feature, but nothing here is VS Code-specific — Principle VI). It
describes capability declarations and per-method behavior; exact Rust function
names are an implementation detail, not part of this contract.

## `initialize`

`ServerCapabilities` declares:

| Capability | Value | Backs |
|---|---|---|
| `position_encoding` | `Utf16` (fixed constant — `contracts/position-encoding.md`) | All position-bearing responses |
| `text_document_sync` | `Full` (whole-document sync on every change) | FR-002; matches research.md §2's whole-document re-parse decision — no incremental sync complexity this phase |
| `hover_provider` | `true` | FR-008–FR-011 |
| `completion_provider` | `trigger_characters: [" ", "="]` | FR-012–FR-013 — a space (new statement/keyword boundary) or `=` (a pair keyword just closed, plausible start of a value the user might still want keyword suggestions before) re-triggers |
| `semantic_tokens_provider` | `legend` naming `shortIf` (token type) and `unreachable` (token modifier) alongside `lsp-types`' standard set actually used; `full: true` | FR-016–FR-018, research.md §6 |
| `document_formatting_provider` | `true` | Added 2026-08-10, outside this feature's original scope — see spec.md's dated Assumptions entry. Whole-document formatting only; no range or on-type variant |
| `diagnostic_provider` | *(not declared — diagnostics are server-pushed, not pull-based)* | FR-005–FR-007 use `textDocument/publishDiagnostics`, the simpler, universally-supported push model; no client capability negotiation needed |

Spell-check (FR-014–FR-015) is **not** a distinct LSP capability — it rides on
the existing `hover_provider`/`completion_provider` responses (a "did you mean"
hint is surfaced as part of a hover response over the misspelled token, and/or as
a completion item ranked first with a distinguishing label) rather than
inventing a non-standard method, per constitution Principle VI.

## `textDocument/didOpen`, `didChange`, `didClose`

- **`didOpen`**: inserts an `OpenDocument` into `ServerState.documents`
  (data-model.md §2), immediately parses via `voyager_core::parse` (always —
  never `parse_bytes`, per data-model.md §2/§3 and research.md §12), and
  publishes diagnostics (below).
- **`didChange`** (full sync — `TextDocumentSyncKind::Full`): replaces
  `OpenDocument.text` with the new full content, re-parses, re-publishes
  diagnostics. A `didChange` whose `version` is not greater than the document's
  current tracked `version` is ignored (data-model.md §2's staleness guard).
- **`didClose`**: removes the document from `ServerState.documents` and publishes
  an empty `publishDiagnostics` for it (FR-006's "clear all diagnostics for a
  document when it is closed").

## `textDocument/publishDiagnostics` (server → client, FR-005–FR-007)

One notification per `didOpen`/`didChange`, containing every
`OpenDocument.parse_result.diagnostics` entry translated via
`contracts/position-encoding.md`'s `to_lsp_range`, with `severity: Error` for
each of the six reachable categories (mirrors `002-cli-check-format/
research.md`'s SARIF severity-mapping precedent: a structural parse defect is
never merely stylistic) and `message` carried through unchanged from
`Diagnostic.message` (no new wording invented at this layer, Principle II).
`InvalidEncoding` (the seventh `voyager-core` category) never appears here —
not filtered, structurally absent, since `parse_result` always comes from
`parse()` (data-model.md §3, research.md §12).

## `textDocument/hover` (FR-008–FR-011)

1. Translate the request's `position` via `from_lsp_position`.
2. Look up the `Block` (if any) whose opener/closer span contains that
   `voyager-core` position (data-model.md §4's `BlockHoverFact`).
3. No `Block` found → respond with `null` (no hover) — never fabricate a
   response for an unrelated token (FR-011).
4. `Block` found → respond with hover contents naming `kind` (and, for a
   short-`IF`, explicitly noting there is no separate closer — FR-010),
   plus, when `counterpart` is `Some`, that location rendered as a
   `to_lsp_range`-translated `Location`. `counterpart`'s five-way derivation
   rule (`Block.closer` alone is not sufficient — it does not distinguish
   "implicitly closed" from "genuinely unmatched" for `Run`/`Process`) lives
   entirely in data-model.md §4's **Derivation** list — this contract
   intentionally does not restate it, to avoid the two documents drifting
   independently if one is corrected without the other.

## `textDocument/completion` (FR-012–FR-013)

1. If the token/context at the cursor is inside a comment or string
   (`data-model.md` §5's `in_comment_or_string`), respond with an empty list.
2. Otherwise, resolve `CompletionRequestContext.enclosing_control_word`
   (research.md §2) and call `voyager_core::keywords::completion_candidates`.
3. Map each returned `KeywordEntry` to an LSP `CompletionItem` (`label: name`,
   `kind: Keyword`).

## `textDocument/semanticTokens/full` (FR-016–FR-018)

Walks `OpenDocument.parse_result` once per request, emitting the standard
LSP semantic-tokens delta-encoded array (relative line/`character` — the
`character` deltas go through the same UTF-16 counting `contracts/
position-encoding.md` specifies, since LSP's semantic-token encoding is
itself UTF-16-code-unit-based) for every token, tagging `shortIf`/`unreachable`
per data-model.md §6's derivation rules.

Also tags every `@name@` reference in the document with the *standard* LSP
`variable` semantic type (added 2026-08-10, outside the original `shortIf`/
`unreachable` scope — see spec.md's dated Assumptions entry). Found via real
manual VS Code testing, not planned in advance: the static grammar's own
`variable.other.readwrite.drut-voyager` TextMate scope only renders with a
distinct color under a theme that happens to already have a rule for it —
several real themes don't. VS Code's editor ships a built-in baseline color
for every *standard* semantic token type (the ~20-name vocabulary the LSP
spec defines, `variable` among them) that applies even when the active
theme defines no rule of its own, which a custom, extension-defined
TextMate scope can never get for free. Computed independently of `collect`'s
structural walk — re-tokenizes the whole document text directly
(`voyager_core::tokenize`), since a variable reference can appear in any
token position (a condition, a pair's value, even inside a quoted string),
not only the structural positions the short-IF/unreachable walk visits.
Covers only the name, not the `@` delimiters, which stay under the static
grammar's own `punctuation.definition.variable` scope (semantic tokens take
priority over TextMate coloring wherever both apply to the same range, so
covering the delimiters too would silently steal their own distinct color).

## `textDocument/formatting` (added 2026-08-10, outside original scope)

Thin wrapper over `voyager_core::format` (`002-cli-check-format`'s already-built,
already-golden-fixture-tested formatting engine; `crates/voyager-core/src/
format.rs`'s own doc comments cover its whitespace/indentation rules and
guarantees — idempotence, no reordering, no continuation changes). `casing` is
always `None` (untouched) — an opt-in casing setting stays a `drut-cli`-only
concern (FR-015) until a real settings surface exists for LSP-triggered
formatting. Always returns either `Some([])` (already formatted — zero edits,
not a missing-capability `None`) or `Some([one TextEdit spanning the whole
document])`; never a minimal per-line diff, since `voyager_core::format`
produces a whole re-rendered `String`, not an edit list, and the whole-
document replacement is trivially correct regardless of how much changed.

## Error handling (FR-004)

Any handler that would otherwise need to index into a document's text beyond its
bounds (e.g. a stale position from a client that hasn't caught up to the latest
`didChange`) returns the closest valid result (a clamped position, per
`contracts/position-encoding.md`, or an empty response) rather than an LSP error
response or a panic — consistent with `voyager-core`'s own no-panic contract
extended to this crate.
