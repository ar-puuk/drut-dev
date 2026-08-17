# Phase 0 Research: Token Hover Shows Assigned Value

All findings below are measured directly against the real, current codebase on this
branch (`crates/voyager-core/src/{lib.rs,statement.rs,token.rs,span.rs,
block_resolution.rs}`, `crates/drut-lsp/src/{hover.rs,document_store.rs,
workspace.rs,position.rs}`) and, for scope decisions, against the real
`WF-TDM-Development` corpus (already used to ground spec.md — see its Assumptions
section for the full evidence trail). Not estimated.

## §1. `@token@` is already a first-class token kind — no new lexing needed

**Finding**: `voyager-core`'s lexer already tokenizes `@name@` as a single
`TokenKind::VariableRef { name: String }` token (`token.rs`), not three separate
punctuation/word tokens. Confirmed by `token.rs`'s own test,
`variable_ref_token_has_no_evaluation_just_name_and_position`.

**Decision**: Finding "the `@token@` the user is hovering" is a simple walk over
each `Statement.tokens` (or, for a `Control` statement, `Statement.tokens` still
holds every token including ones absorbed into `pairs`, confirmed by
`classify_statement`'s construction — it builds `kind` from `grp` but always stores
the original `grp` as `tokens` too) looking for a `VariableRef` token whose `span`
contains the hover position — the same "does this span contain the position" shape
`block_resolution.rs::find_block_at` already uses, just at token granularity instead
of block granularity.

## §2. `READ FILE = '<path>'` parses as `Control { word: "READ", pairs: [("FILE", value_tokens)] }` — confirmed, not assumed

**Finding**: Traced `classify_statement` (`statement.rs`) directly: `READ` is not in
`FIXED_KEYWORDS`, and is not itself immediately followed by `=` (the next token is
`FILE`), so it falls to the final `grp[0].kind == TokenKind::Word` arm — `word =
"READ"`, `pairs = extract_pairs(&grp[1..])`, which correctly finds `FILE` as a pair
keyword and captures everything after its `=` (the quote punctuation tokens and the
path's `Word`/`VariableRef` tokens) as that pair's value, unstripped.

**Decision**: A "literal `READ FILE`" detector scans `Control` statements for
`word.eq_ignore_ascii_case("READ")` with a pair whose keyword
`eq_ignore_ascii_case("FILE")`, then classifies that pair's value tokens: if any
token in the value is `TokenKind::VariableRef`, the path is dynamic (spec.md FR-003
— excluded); otherwise the literal path text is reconstructed by concatenating the
non-quote tokens' own `.text` (quote-punctuation tokens, if present, are stripped,
not included in the path string).

**Real corpus confirmation**: This exact shape (`READ FILE = '_ControlCenter.block'`
literal; `READ FILE = '@ParentDir@...block'` dynamic) is what the chain-depth
research already run against `WF-TDM-Development` (referenced from spec.md) was
built on — 324 literal, resolvable occurrences vs. 199 token-built ones, across 43
files.

## §3. No existing "value → display text" renderer — span-slicing is simpler and more accurate than token-joining

**Finding**: No existing helper renders a `Vec<Token>` back to its original source
text; `format.rs` has token-to-text logic, but it exists to *reformat* (control
whitespace/casing per formatter rules), not to reproduce the literal original
substring. Naively joining `Token.text` values with no separator would be wrong for
any value the lexer split into multiple tokens with source whitespace between them
(e.g. a bracketed subscript, an expression, or anything past the simple
single-word/single-quoted-string case) — silently dropping spacing information the
lexer never preserved once split into a token stream.

**Decision**: Instead of joining token text, resolution returns a `Span` (the merged
span of the value's tokens, or of the whole `Assignment`/`READ FILE` statement, per
what's needed), and `drut-lsp` slices the *exact* original source substring for that
span directly from whichever source text is authoritative for that span (the open
document's own `text` for a same-file result; the `READ FILE` target's own freshly-
read text for a cross-file result). This requires one small new helper,
`position::text_for_span(text: &str, span: Span) -> String`, extending
`position.rs`'s existing "one place, reused everywhere" charter (already true for
`to_lsp_position`/`to_lsp_range`) rather than adding ad hoc slicing logic inside
`hover.rs` itself.

**Alternatives considered**: Token-joining with heuristic whitespace re-insertion
was considered and rejected — it would need to reconstruct spacing rules the lexer
already discarded, for no benefit over just re-reading the real substring, which is
unambiguous and always exactly correct by construction.

## §4. Keeping `voyager-core` I/O-free while still resolving a cross-file value

**Finding**: `CLAUDE.md`'s binding contract: `voyager-core`'s two public functions
operate "on in-memory text only... no file I/O... inside the crate itself."
Resolving a `READ FILE` target inherently requires reading a second file, which
cannot happen inside `voyager-core` without violating this.

**Decision**: Split the responsibility at the same boundary `012-toml-
configuration` and `013-lsp-config-file-watch` already established for
`drut-config`/`drut-lsp` (I/O and orchestration in the adapter; pure logic in the
core crate): `voyager-core::token_resolution::resolve_token_value` takes the open
document's own `&[Node]`, the hover `Position`, `name: &str`, and a caller-supplied
list of `(Span, Vec<Node>)` — one entry per literal `READ FILE` statement's own span
in the open document, paired with whatever `Vec<Node>` resulted from parsing that
statement's target file (or simply omitted from the list if `drut-lsp` couldn't read
it — spec.md FR-007). `drut-lsp`'s `hover.rs` does the actual disk read (via
`std::fs::read` + `workspace::uri_to_path`-derived parent directory) and
`voyager_core::parse_bytes` call (reusing the crate's own existing encoding-fallback
decode path, `parse_bytes`, rather than assuming UTF-8 for a file that — unlike an
open LSP document — was never guaranteed valid UTF-8 by the protocol layer; see
`document_store.rs`'s own doc comment on exactly this distinction), then passes the
result in. `voyager-core` itself still never touches a filesystem.

## §5. Ordering: "most recent, per real interleaved execution order" is expressible as one pure comparison

**Finding**: Both an `Assignment`'s own `span` (via a synthesized `Statement`-like
wrapper — see `data-model.md`) and each literal `READ FILE`'s span are already
ordinary `voyager_core::Span`/`Position` values, which already implement `Ord`
(`span.rs`: `#[derive(... PartialOrd, Ord ...)]` on `Position`). Spec.md FR-004's
rule — a `READ FILE`'d file's own assignments are treated as occurring exactly at
that `READ FILE` statement's own position, interleaved with the open document's own
assignments — reduces to: build one combined, sorted-by-effective-position list of
candidate assignments (same-file ones keep their own real position; included-file
ones are all stamped with their originating `READ FILE` statement's position), then
pick the last one at or before the hover position.

**Decision**: `resolve_token_value` builds this combined list internally (not
exposed as a separate public function — spec.md doesn't require inspecting the full
list, only the single resolved answer) and returns `Option<ResolvedTokenValue>`
carrying enough to render hover text: the value's own `Span` (for `text_for_span`),
the assignment statement's own `Span` (for "assigned at line N"), and a `Source`
enum (`SameFile` or `ReadFile { read_file_statement_span: Span }`, letting `hover.rs`
report which file the value came from per spec.md FR-009 without `voyager-core`
itself knowing any file *name* — it only knows spans within whichever `&[Node]` the
caller passed for that source).

## §6. `hover.rs`'s existing fallback order — token-value resolution slots in first, cleanly

**Finding**: `hover.rs::handle` today tries `block_at` first, falling back to
`spellcheck::hint_for` only if that returns `None`. A `@token@` reference is never
itself a block opener/closer, so `block_at` already correctly returns `None` for it
today — meaning today's hover over `@token@` always falls through to the
spell-check-nudge path (and usually finds nothing there either, since a token name
is rarely one edit away from a keyword).

**Decision**: Add the new token-value check as a new first branch, before
`block_at` — `variable_ref_at` is a cheap, narrow check (only fires when the cursor
is actually over a `VariableRef` token) and is strictly more specific than
`block_at`'s own check, so trying it first changes nothing about `block_at`'s or
`spellcheck`'s existing behavior for every case that isn't hovering a `@token@`
reference (spec.md FR-010).

## §7. Untitled/unsaved buffers: same-file resolution still works; cross-file gracefully doesn't apply

**Finding**: `workspace::uri_to_path` already returns `None` for a non-`file`-scheme
URI (e.g. `untitled:Untitled-1`, confirmed by its own existing test). Cross-file
`READ FILE` resolution needs a real on-disk parent directory to resolve a relative
path against.

**Decision**: When `uri_to_path` returns `None` for the hovered document's own URI,
`drut-lsp` simply passes an empty `included` list to `resolve_token_value` — same-
file resolution (spec.md User Story 1) is completely unaffected, since it needs no
path at all; only the cross-file part (User Story 2) silently doesn't apply, which
is already exactly spec.md FR-008's documented fallback behavior for "not found in
scope," not a new failure mode requiring its own FR.

## §8. Case-insensitive matching — reusing the crate's own established convention

**Finding**: `voyager-core` already compares every keyword/identifier
case-insensitively via `str::eq_ignore_ascii_case` throughout `block.rs`,
`statement.rs`, and `keywords.rs` (confirmed by direct grep — no
`to_lowercase`/Unicode-aware casing anywhere in this crate, consistent with Voyager
identifiers being ASCII by construction).

**Decision**: `all_assignments`'s target-name matching and `variable_ref_at`'s own
name comparison both use `eq_ignore_ascii_case`, matching this existing convention
exactly rather than introducing a second casing rule.
