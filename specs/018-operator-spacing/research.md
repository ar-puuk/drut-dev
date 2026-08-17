# Phase 0 Research: Operator Spacing Normalization

Grounded directly in `crates/voyager-core/src/{lexer,token,statement,format}.rs` as they exist
today (post-`017-casing-categories-indent-width`), not assumed from the spec alone — every
finding below was confirmed by reading the actual code before being treated as a planning fact.

## §1: Every operator character is already tokenized standalone — no lexer change for recognition

`lexer.rs`'s `is_delimiter` already includes every character this feature cares about: `, = + -
/ * ^ & | < >` (alongside brackets/parens/quotes/`:`/`!`). Each one is emitted as its own
single-character `TokenKind::Punctuation` token, splitting whatever `Word` run it interrupts.
This means `mi.1.1+mi.2.1` already tokenizes as `Word("mi.1.1")`, `Punctuation("+")`,
`Word("mi.2.1")` — three separate tokens, not one opaque blob. Same story for `=`, `<`, `>`,
and `,`.

**Decision**: recognizing where an operator sits, and what precedes/follows it, requires no
`token.rs`/`lexer.rs` change at all — mirrors `017`'s finding that dot-notation boundaries and
bracket characters were already usable without a `TokenKind` change. This feature is
meaningfully *smaller* on the recognition side than `017` feared its own lexer work might be.

## §2: Multi-character comparison operators are NOT single tokens today

There is no multi-char lookahead anywhere in `lexer.rs`'s main scan loop. `I==1` tokenizes as
`Word("I")`, `Punctuation("=")`, `Punctuation("=")`, `Word("1")` — **two** adjacent single-`=`
tokens, not one `==` token. Same for `<>` (two tokens: `<`, `>`) and `>=`/`<=` (two tokens each).
Naively spacing each `=` independently would produce `I = = 1` or `I= =1`-shaped garbage — a
real correctness bug if not handled deliberately.

**Decision**: add a merge-recognition step — not to the shared lexer (that would change
`TokenKind` semantics for every other consumer: diagnostics, LSP, hover, none of which care
about this distinction) — but as a small, local scan inside the new operator-spacing module
itself, over an already-tokenized statement's token list. Two `Punctuation` tokens merge into
one logical multi-char operator when: both texts are drawn from `{'=', '<', '>'}`, they sit on
the same line, and the first token's `span.end` exactly equals the second token's `span.start`
(zero-gap adjacency — anything with a space between them is two separate single-char operators,
e.g. `A < B` stays a `<` comparison, not merged with a following unrelated `=`). This is the
same "new self-contained module, no shared grammar/`TokenKind` change" shape `data_reference.rs`
already established for `017` — not a new architectural pattern, a repeat of an existing one.

## §3: Continuation markers reuse these exact operator characters

`Token::is_continuation_char_text` matches `, + - / * ^ & | =` — the *same* character set this
feature normalizes (minus `<`/`>`, which are never continuation characters). `lexer.rs`'s
`mark_continuation_markers` retags the last non-comment token on a physical line to
`ContinuationMarker` when its text is one of these. A `ContinuationMarker` sits at end-of-line;
its "operand" is the next physical line's content, not anything on its own line.

**Decision** (spec.md FR-012, amended before this research doc was written): only the leading
side of a `ContinuationMarker`-tagged operator gets normalized (ensure exactly one space before
it); never insert a trailing space, since nothing exists after it on that line to space against.
This only affects the minority case where an operator happens to be the very last token on its
line — mid-expression occurrences of the same characters are untouched by this rule and get
full two-sided spacing as normal.

## §4: The render pipeline's edit mechanism cannot represent this feature's edits as-is

`format.rs::render`'s `edits_by_line` applies every queued edit via a same-length in-place
column splice:

```rust
if *end <= chars.len() && *start <= *end && repl_chars.len() == end - start {
    chars[*start..*end].clone_from_slice(&repl_chars);
}
```

This is correct and sufficient for casing edits (`Upper`/`Lower` never change a token's
character count) but silently no-ops any edit whose replacement length differs from the
original span's length. Operator spacing is fundamentally a variable-length problem —
`MW[1]=x` → `MW[1] = x` *inserts* two characters; `MW[ 1 ]` → `MW[1]` *removes* two. Reusing
`edits_by_line` unchanged would silently drop every spacing edit.

**Decision**: add a second, independent edit list (a `SpacingEdit`, same `(line, start, end,
replacement)` shape as `CasingEdit` for consistency) applied through a *new* line-rebuild step:
walk each line's queued edits left-to-right in column order, copying the original untouched
segments between edit boundaries verbatim and splicing in each edit's replacement text — the
standard non-overlapping-replace-list-to-output-string algorithm, not a per-character slice.
Casing edits and spacing edits never target overlapping spans (one rewrites token *text*, the
other rewrites the *whitespace around* tokens), so both edit kinds can be resolved into one
unified per-line application pass without ordering conflicts between them. Indentation
(leading-whitespace-only, applied after both) is unaffected by either, exactly as today.

## §5: Unary vs. binary `+`/`-` is a token-lookback problem, not an expression-parsing one

`StatementKind::Assignment { value: Vec<Token> }` and `Control`'s `pairs: Vec<(String,
Vec<Token>)>` already store each value/pair as its own ordered token list — this feature never
needs to build an expression tree. A `+`/`-` `Punctuation` token is unary when the previous
token in that same list is absent (start of the value), or is itself `=`, `(`, `,`, or another
recognized operator token; otherwise it's binary. Matches spec.md's Assumptions (black/
prettier/`gofmt` convention).

## §6: `auto`'s "same block nesting depth" is free — it's just sibling adjacency

Block nesting in `voyager-core`'s `Node`/`Block` tree is *already* one `Vec<Node>` per nesting
level (a block's `children: Vec<Node>` is a separate list from its parent's). "Same depth" for
alignment-run purposes is therefore nothing more than "consecutive elements of the same
`Vec<Node>` slice" — no depth counter or extra bookkeeping needed, the tree shape already
enforces it. A run is a maximal run of consecutive `Node::Statement(Assignment)` entries in one
`Vec<Node>`, with the additional between-statement check (§3 above's sibling case) for a blank
line or comment-only line in the source between the two statements' spans — resolved the same
way `protected_regions` already classifies lines by whether they carry non-comment tokens, not
a new line-classification mechanism.

## §7: Comma spacing and bracket/paren interior padding reuse the same operator-adjacency shape

Commas between `Control` pairs, and `[`/`]`/`(`/`)` delimiters, are already standalone
`Punctuation` tokens (§1). Comma spacing (FR-004) and interior-padding removal (FR-005) are
just two more zero/one-space adjacency rules applied through the exact same token-pair-scan
and `SpacingEdit`-emission mechanism as arithmetic/comparison/assignment operators — no
separate implementation path.

## §9: Operator characters inside a quoted string literal are NOT distinguishable by `TokenKind`

Confirmed empirically (`tokenize("LIST='a+b'\n")`): the lexer's quote-tracking
(`in_single_quote`/`in_double_quote`) only gates `;`-comment-start and `/*`-comment-start
recognition (lexer.rs lines 132–147) — it does **not** gate delimiter/operator recognition or
word-scanning. A `+` sitting inside `'a+b'` tokenizes as a standalone `Punctuation("+")` token,
byte-for-byte indistinguishable at the `TokenKind` level from a real arithmetic operator
outside any string. Naively scanning a statement's token list for operator-shaped `Punctuation`
tokens (as §1/§2/§5 describe) would therefore let `Fixed`/`Auto` insert spacing *inside* a
quoted string's literal content — e.g. `LIST='a+b'` → `LIST='a + b'` — a real, silent behavior
change to program output, directly violating FR-010's "never altering values inside string/
quoted literals."

**Decision**: `operator_spacing.rs`'s recognition pass must track quote state itself, over the
statement's own token list, the same way `lexer.rs` already tracks it globally — walk the token
list maintaining an `in_quote` toggle keyed off `'`/`"` `Punctuation` tokens (odd count = inside
a string), and skip *every* operator/comma/bracket-paren recognition rule for a token whose
index falls between an unmatched opening quote and its closing counterpart. This is a second,
independent quote-tracking pass — not a shared cache with the lexer's own internal one, since
that state isn't exposed on `Token`/`TokenKind` and adding it there would be a `TokenKind`
change this feature deliberately avoids (§1/§2's decision not to touch shared grammar types for
a concern only this module cares about). An unterminated quote within one statement's token
list (should not happen given how statements are built, but not proven impossible) degrades
safely: everything after the unmatched opening quote is treated as "inside a string" and
excluded from recognition, never the reverse (never over-eagerly treating string content as
real operators when in doubt).

## §8: `format.rs`'s own scope-limiting doc comment needs updating

The module doc comment (`format.rs` lines 1–23) currently states as an unconditional fact that
"intra-line spacing between tokens... is copied through unchanged" — true today, but only
because no caller has ever set anything beyond casing/indentation. This claim becomes
conditional on `preserve` once this feature ships and must be reworded, not left stale — the
same kind of doc-accuracy discipline `017` applied when it extended `FormatOptions`.
