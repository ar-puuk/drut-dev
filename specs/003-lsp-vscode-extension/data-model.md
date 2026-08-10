# Phase 1 Data Model: Drut LSP Server & VS Code/Open VSX Extension

This feature's entities split across two crates plus the extension package.
`voyager-core` gains one new module (§1); `drut-lsp` owns document/session state
and the LSP-facing derived views (§2–§6); the extension (§7) is configuration, not
runtime data. Types already defined by `001-voyager-script-parser`/
`002-cli-check-format` (`Token`, `Statement`, `StatementKind`, `Block`,
`ParseResult`, `Diagnostic`, `DiagnosticKind`, `Span`, `Position`) are referenced,
not redefined.

## 1. `voyager-core` additions — the `keywords` module

### KeywordEntry

One dictionary entry (FR-012), built once at compile time from the FR-012 corpus
census — not derived at runtime from any single document.

| Field | Type | Notes |
|---|---|---|
| `name` | `&'static str` | The keyword or control word's canonical spelling, as observed dominant in the census (case reported as-surveyed; matching against document text is case-insensitive, mirroring FR-011 in `001-voyager-script-parser`) |
| `role` | `KeywordRole` | `ControlWord` or `PairKeyword` — which completion position this entry is valid for (spec Story 4 Acceptance Scenarios 1 vs 2) |
| `observed_with` | `&'static [&'static str]` | For `PairKeyword` entries: the control word(s) this keyword name was observed paired with during the census (research.md §2's context-scoping data). Empty for `ControlWord` entries. |

### KeywordRole

| Value | Meaning |
|---|---|
| `ControlWord` | Valid as the first word of a new statement |
| `PairKeyword` | Valid as a `keyword=value` pair's keyword name, scoped by `observed_with` |

### CompletionContext

The caller-supplied (i.e. `drut-lsp`-supplied) description of *where* in a
document completion was requested — deliberately narrow, so this module never
needs to know about documents, URIs, or LSP types (Principle I: `voyager-core`
has no protocol dependency).

| Field | Type | Notes |
|---|---|---|
| `enclosing_control_word` | `Option<&str>` | `Some(word)` when the cursor falls inside a `Statement` whose `kind` is `Control { word, .. }` (research.md §2) — `None` when no enclosing `Control` statement was found (start of a new statement, or inside a `Label`/`ShellEscape`/`Assignment`) |

### `completion_candidates`

```text
fn completion_candidates(ctx: CompletionContext) -> Vec<&'static KeywordEntry>
```

- `ctx.enclosing_control_word == None`: returns every `ControlWord` entry (the
  general-syntax fallback list — spec Story 4 Acceptance Scenario 1).
- `ctx.enclosing_control_word == Some(word)`: returns every `PairKeyword` entry
  whose `observed_with` contains `word` (case-insensitive), i.e. the context-
  scoped list (Acceptance Scenario 2). If that set is empty (a control word the
  census never observed with any recorded pair keyword), falls back to every
  `PairKeyword` entry regardless of `observed_with` — the documented general
  fallback (spec.md Assumptions), never an empty suggestion list.

### `did_you_mean`

```text
fn did_you_mean(token: &str) -> Option<&'static KeywordEntry>
```

Implements research.md §5's Damerau-Levenshtein, unique-minimum-within-2 rule
across the full dictionary (both roles) — the caller (`drut-lsp`) decides
separately whether the token's position makes a `ControlWord` or `PairKeyword`
suggestion the relevant one to surface. Returns `None` for an exact match (case-
insensitive) or when no entry is within distance 2, or when more than one entry
ties for the lowest distance (spec Story 5 Acceptance Scenarios 2/3, Edge Cases).

**Validation rules**:
- The dictionary is static (`&'static`) — no runtime mutation, no per-document
  variation; this keeps `completion_candidates`/`did_you_mean` pure functions
  with no hidden state, consistent with `voyager-core`'s existing `tokenize`/
  `parse` determinism guarantee (`001-voyager-script-parser/contracts/
  public-api.md`).
- Neither function panics on any input, including an empty string or a token
  containing non-ASCII bytes (mirrors the crate-wide no-panic guarantee).

## 2. `drut-lsp` — session/document state

### ServerState

Owns everything the running `drut server` process needs across requests.

| Field | Type | Notes |
|---|---|---|
| `documents` | `HashMap<Uri, OpenDocument>` | Keyed by the LSP document URI (FR-002) |

### OpenDocument

One currently-open document (FR-002), re-derived on every content change.

| Field | Type | Notes |
|---|---|---|
| `text` | `String` | The document's current in-editor content, from `textDocument/didOpen`/`didChange` — not necessarily saved to disk (FR-002) |
| `parse_result` | `ParseResult` | The result of `voyager_core::parse(&text)` against `text`, re-computed on every change (FR-002). **Always `parse`, never `parse_bytes`** — `text` is a Rust `String`, guaranteed valid UTF-8 by construction, and `didOpen`/`didChange`'s JSON payload cannot carry anything else (research.md §12) — there is no "if the document's encoding requires it" branch for a live LSP document; that hedge existed in an earlier draft of this table and has been resolved (see §3). |
| `version` | `i32` | The LSP document version last applied, so a stale/out-of-order `didChange` can be detected and ignored rather than corrupting `text` (FR-002, FR-006) |

**Validation rules**:
- `parse_result` is always in sync with `text` — there is no code path that reads
  `parse_result` for a `text` value it wasn't derived from (every mutation to
  `text` immediately re-derives `parse_result` in the same handler, before any
  other request can observe the document).
- Removing a document from `documents` (on `textDocument/didClose`) also clears
  its published diagnostics (FR-006) — a closed document is never left showing
  stale diagnostics in the editor.

## 3. Diagnostics (FR-005–FR-007)

No new type — `OpenDocument.parse_result.diagnostics` (`Vec<Diagnostic>`, from
`001-voyager-script-parser/data-model.md`) is translated one-for-one into LSP
`Diagnostic` values via the position translation contract (§5,
`contracts/position-encoding.md`) and a fixed `DiagnosticKind -> message` mapping
that reuses the same original wording `voyager-core`'s own `Diagnostic.message`
already carries (no new hover/help text invented at this layer, Principle II).

**Six of seven categories reachable in practice, by construction, not by
filtering**: because `OpenDocument.parse_result` always comes from `parse()`
(§2), never `parse_bytes()`, `ParseResult.diagnostics` can never actually
contain an `InvalidEncoding` value for a live document — `voyager-core`'s
own `parse()` entry point has no encoding-fallback code path to produce one
(`001-voyager-script-parser/contracts/public-api.md`: `InvalidEncoding` is
"only reachable via `parse_bytes`"). There is no dedicated filtering logic
in `drut-lsp` that excludes it; it is structurally absent from the input
`drut-lsp` ever receives. See spec.md FR-005/Assumptions and research.md §12
for the full architectural reasoning.

## 4. Hover (FR-008–FR-011)

### BlockHoverFact

Derived, not stored — computed on each `textDocument/hover` request by locating
the `Block` whose opener or closer span contains the requested position.

**Correction (2026-08-10)**: earlier drafts of this section (and of
`contracts/keyword-dictionary-api.md`/research.md §2) referred to a flat
`ParseResult.blocks`/`ParseResult.statements` list. No such field exists.
`001-voyager-script-parser/data-model.md`'s own `ParseResult` — correct there
all along — is `{ nodes: Vec<Node>, diagnostics: Vec<Diagnostic> }`, where
`Node` is `Statement(Statement) | Block(Block)`; `Block.children: Vec<Node>`
nests the same type recursively, and for `BlockKind::If` specifically, the
real body lives in `branches: Vec<IfBranch>`, each with its own
`children: Vec<Node>` (`Block.children` is unused/empty for `If`). Locating
"the Block at this position" or "the enclosing control word" therefore means
a **recursive walk over `ParseResult.nodes`**, not a lookup into a flat list
— `hover.rs`'s `find_block_at` and `completion.rs`'s `find_in` (§5) both walk
child content first (a nested match is always more specific), then check the
current node itself. This was an error introduced during this feature's own
`/speckit-plan` pass, not something inherited from `001`, whose own
`ParseResult` documentation was accurate throughout.

| Field | Type | Notes |
|---|---|---|
| `kind` | `BlockKind` | Reused from `voyager-core` unchanged (If/Loop/Run/Process/JLoop/LinkLoop/DistributeMultistep) — backs FR-008 |
| `is_short_if` | `bool` | `true` when `kind == If` and the block has no separate closer statement (`Block.closer` is `None` by construction for this shape, not because it's unmatched — distinguished from a genuinely unmatched `If` by `ParseResult.diagnostics` containing no `UnmatchedIf` for this block) — backs FR-010 |
| `counterpart` | `Option<Span>` | See **Derivation** below — `Block.closer` alone is *not* sufficient (corrected 2026-08-09; see Note) |

**Derivation** (corrects an earlier draft of this table that derived
`counterpart` directly from `Block.closer` "including through Run/Process's
implicit-close path" — that claim was inconsistent with `Block.closer`'s own
documented semantics, which are `None` for exactly that case; see Note below):

1. `Block.closer` is `Some(span)` → `counterpart = Some(span)` (an explicit
   closer statement exists; unambiguous).
2. `Block.closer` is `None` and `kind == If` and `is_short_if` → `counterpart
   = None` (no separate closer construct exists for this shape at all — not
   a resolution failure, backs FR-010).
3. `Block.closer` is `None` and `kind` is `If` (non-short), `Loop`, `JLoop`,
   `LinkLoop`, or `DistributeMultistep` → `counterpart = None`. For `If`/
   `Loop` this means genuinely unmatched (`UnmatchedIf`/`UnmatchedLoop` is
   present in `ParseResult.diagnostics` for this block). For `JLoop`/
   `LinkLoop`/`DistributeMultistep`, `closer == None` always means genuinely
   unmatched — these three are not among FR-009's named implicit-close
   families (only `Run`/`Process` are), so there is no other case to handle.
4. `Block.closer` is `None` and `kind == Run` → if `ParseResult.diagnostics`
   contains no `UnmatchedRun` referencing this block, the block closed
   implicitly (by the next `RUN`/`!RUN` opener or a shell-escape statement)
   and `counterpart = Some(Block.span.end)` — the same diagnostic-absence
   technique rule 2 already uses for `is_short_if`, applied to `Run`'s own
   `UnmatchedRun` category. If `UnmatchedRun` *is* present for this block,
   `counterpart = None` (genuinely unmatched).
5. `Block.closer` is `None` and `kind == Process` → `counterpart =
   Some(Block.span.end)` unconditionally. Process has no "unmatched"
   diagnostic category at all (`001-voyager-script-parser`'s six-category
   list has no `UnmatchedProcess`), so unlike `Run` there is no signal
   available to distinguish "implicitly closed by the next
   `PROCESS`/`PHASE` opener" from "genuinely unmatched, reached end-of-file
   with no closer of any kind." Rule 5 reports `Block.span.end` — literally
   `voyager-core`'s own resolved extent for the block's body, via the same
   `end_span_or` fallback `002-cli-check-format/research.md` §8 documents —
   in both cases. This is a deliberate, documented best-effort choice, not a
   fabrication: the value reported is exactly what `voyager-core` itself
   computed as this block's extent, so it satisfies SC-004's "100% agreement
   with `voyager-core`'s own resolution" by construction, even in the rare
   genuinely-unmatched-at-EOF sub-case. See spec.md Assumptions for the
   product-level acknowledgment of this limitation.

**Note (2026-08-09)**: The original version of this table claimed
`counterpart` was `Block.closer` "when resolved, including through
Run/Process's implicit-close path." That was incorrect —
`001-voyager-script-parser/data-model.md`'s `Block.closer` is documented
`None` "when the block closed implicitly (`Run`/`Process`) *or* is genuinely
unmatched," which is the opposite of what FR-009/Story 3 AS3/SC-004 require.
Found via `/speckit-checklist` (CHK015/CHK016) and confirmed independently
via `/speckit-analyze` (finding I1); corrected here rather than left for
implementation to discover as a failing test.

**Validation rule**: A hover request over a position not covered by any `Block`'s
opener/closer span produces no `BlockHoverFact` (FR-011) — `drut-lsp` returns an
empty hover response rather than fabricating one.

## 5. Completion & spell-check (FR-012–FR-015)

### CompletionRequestContext (`drut-lsp`-local, feeds `voyager_core::keywords::CompletionContext`)

| Field | Type | Notes |
|---|---|---|
| `in_comment_or_string` | `bool` | **Not** simply "the `Token` at the cursor has kind `Comment`/`String`" — no such `TokenKind` variants exist. `voyager-core` has `TokenKind::LineComment`/`BlockComment{unterminated}` for comments, but no dedicated string-content kind at all: `'`/`"` are individual `Punctuation` tokens (`crates/voyager-core/src/lexer.rs`), with quoted content itself tokenized as ordinary `Word`/`Punctuation` like anything else. `completion.rs`'s `in_comment_or_string` instead does quote-parity counting over already-`tokenize`d `Punctuation` output up to the cursor (an odd count of `'` or `"` seen so far means "currently inside an open quote") — a derived view over already-classified tokens, not a new grammar decision (Principle I). When `true`, `drut-lsp` returns no completion items at all (FR-013), without calling into `voyager_core::keywords` |
| `enclosing_control_word` | `Option<String>` | Derived per research.md §2's mechanism, adapted to the real `ParseResult` shape (§4's Correction note): a recursive walk over `OpenDocument.parse_result.nodes`, not a lookup into a flat `.statements` list — owns the allocation since `voyager-core`'s borrowed `&str` doesn't outlive the walk |

### SpellCheckHint

| Field | Type | Notes |
|---|---|---|
| `token_span` | `Span` | The misspelled token's location (`voyager-core` coordinates, translated per `contracts/position-encoding.md`'s contract before going on the wire) — backs FR-014 |
| `suggestion` | `&'static str` | `KeywordEntry.name` from `voyager_core::keywords::did_you_mean` — backs FR-014 |

**Validation rule**: A `SpellCheckHint` is only produced for a `Word` token that
is not already a recognized keyword (i.e. `did_you_mean` was consulted precisely
because an exact-match lookup already failed) — never for a token that would
otherwise resolve cleanly (FR-015).

## 6. Semantic tokens (FR-016–FR-018)

### SemanticTokenKind

| Value | Encoding | Meaning |
|---|---|---|
| `ShortIf` | Custom token *type* (legend index, research.md §6) | A self-closing short-`IF`'s tokens (FR-016) |
| `Unreachable` | Custom token *modifier* | Applied to every token of a statement following a validly-resolved `BREAK` within its enclosing loop, before that loop's closer (FR-017) |

**Derivation**: `ShortIf` is read directly off `BlockHoverFact.is_short_if`-style
information already computed for hover (§4) — no separate structural pass.
`Unreachable` is derived by, for each `Loop`/`JLoop`/`LinkLoop` block, walking
its **direct** child statements only, in order, and marking every direct child
strictly after the first *direct* child `Statement` whose first token is
`BREAK` **and** that `BREAK` is not itself the subject of a `MisplacedBreak`
diagnostic for this document (FR-018) — both facts (loop membership,
`MisplacedBreak` presence) already come from `ParseResult`, so this is a
linear scan over already-parsed structure, not a new grammar pass.

**Precisely, against `Block.children: Vec<Node>` (§4's Correction note)**: the
per-loop scan iterates `block.children` and only ever advances its
"seen a valid `BREAK` yet" state on an item that pattern-matches
`Node::Statement(stmt)` where `stmt` is a `BREAK`. A `Node::Block(_)` item at
that same level (e.g. a nested `IF`) never matches `Node::Statement` at all,
so it neither sets nor is affected by that state, regardless of what `BREAK`
statements exist arbitrarily deep inside its own `children`/branches — this
is *why* a `BREAK` nested inside a conditional never marks anything in the
enclosing loop unreachable: it isn't one of the loop's own direct
`Node::Statement` children in the first place, not because of a separate
depth check. The recursive walk that visits nested blocks (to find *their*
own short-`IF`/`Unreachable` tokens) is a different, independent traversal
one level down — it never feeds back into an outer loop's own
already-completed scan.
**Deliberately does not recurse into a nested block's own children**: a
`BREAK` inside a nested `IF`/`ELSEIF`/`ELSE` branch within the loop body is a
child of that `IF` block, not a direct child of the enclosing loop, so it
never triggers this rule at the loop level — correctly, since a conditional
`BREAK` doesn't always execute, and flagging code after it as unreachable
would be a false positive (contrary to constitution Principle IV, which this
narrowing exists specifically to satisfy). This also means a statement
*inside* the same conditional branch as the `BREAK`, after it, is not
flagged either (per the same reasoning) — only statements that are
unconditionally reached after an unconditional `BREAK` are ever flagged.

## 7. Extension (`editors/vscode/`) — configuration, not runtime data

No data model — `package.json`'s `contributes.languages`/`contributes.grammars`/
`contributes.semanticTokenScopes` and `language-configuration.json`'s
brackets/comments are static configuration read by VS Code itself
(`contracts/extension-manifest.md` specifies the exact contribution points).
