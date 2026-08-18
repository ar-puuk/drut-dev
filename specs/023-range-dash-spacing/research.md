# Research: Range-Dash Spacing Exemption

## §1. Where this hooks into the existing `018-operator-spacing` implementation

**Decision**: Add the range-dash exemption as an additional condition inside
`operator_spacing.rs`'s existing binary-`-` recognition path
(`recognize_operators`/`collect_operator_edits`), not as a new, separately-scanned rule. A
qualifying occurrence gets `want_spaces = 0` on both its leading and trailing gap (via the
already-existing `push_gap_edit` helper) instead of the `1` every other binary-arithmetic
occurrence gets.

**Rationale**: `push_gap_edit` already normalizes "the gap between two adjacent tokens" to a
target width, regardless of what that gap originally contained (0, 1, or many spaces) — this is
exactly the "strip any existing spacing" behavior the feature needs, with zero new gap-handling
logic. The only new work is *deciding* the target width (0 vs 1) for one specific shape of binary
`-` occurrence; everything else (unary/binary distinction, quoted-literal masking, continuation
handling, idempotence via `push_gap_edit`'s no-op-when-already-correct check) is inherited for
free from the existing pass.

**Alternatives considered**:
- A separate, independently-scanned "range list" rule (its own function, its own pass over
  tokens) — rejected: would duplicate `is_binary_arithmetic`'s unary/binary distinction and
  `quoted_token_mask`'s quote-safety logic rather than reusing them, risking the two passes
  disagreeing about whether a given `-` is even an operator at all.
- A regex/text scan over each pair-keyword value's rendered source text — rejected: every other
  rule in this module is token-based and quote-aware specifically because `research.md §9` (in
  `018`) found that a text-level scan cannot safely distinguish real operator characters from
  the same character sitting inside a quoted string; reintroducing that risk here for one new
  rule would contradict the module's own established safety pattern.

## §2. Recognizing "inside a pair-keyword's value"

**Decision**: Reuse `pair_keyword_boundaries(&stmt.tokens)` — the exact same function
`collect_comma_edits` already calls to find pair-separator commas — to derive each pair's value
token-index range (`[eq_idx + 1, next_pair.kw_start)`, or `[eq_idx + 1, tokens.len())` for the
last pair on the statement). A `-` occurrence is a range-dash candidate only when its own index
falls inside one of these ranges on a `StatementKind::Control` statement.

**Rationale**: `pair_keyword_boundaries` is already the project's one source of truth for "where
does this pair's value start and end" (also used by `format.rs`'s casing rewrite and
`block.rs`'s opener-pair capture, per that function's own doc comment) — deriving a second,
independent notion of value boundaries for this feature would risk the two rules disagreeing
about the same statement's own structure, the exact failure mode FR-010 exists to rule out.

**Alternatives considered**: Extending `extract_pairs` (which already exists in `statement.rs`
but is private and allocates a fresh `Vec<Token>` per pair) — rejected as unnecessary indirection
for what only needs an index-range membership check, not owned token copies; `collect_comma_edits`
already demonstrates the lighter-weight pattern (iterate `pair_keyword_boundaries`'s output
directly) this feature should follow instead.

## §3. Recognizing "bare integer literal"

**Decision**: A token qualifies as a bare integer literal when its `TokenKind` is `Word` and its
`text` is non-empty and consists entirely of ASCII digit characters (`'0'..='9'`) — no separate
lexer/`TokenKind` change.

**Rationale, confirmed by direct inspection of `lexer.rs`'s `is_delimiter`**: `.` is not a
delimiter character — a word-run only breaks at `is_delimiter` characters (`,=+-/*^&|{}()[]:'"!<>`)
or whitespace/comment/variable-reference starts. This means a decimal number like `1.5` and a
dotted data-reference like `mi.1.1` or `lw.RampPen_10` already tokenize as **one** `Word` token
each, containing a `.` — which the all-digits check naturally rejects with no extra logic. The
spec's decimal-number edge case (`THRESHOLD=1.5-2.5` not being treated as a range) is therefore
satisfied by construction, not by an additional rule that has to be separately written and
tested.

**Alternatives considered**: A new `TokenKind::Number` distinguishing numeric words at the lexer
level — rejected: `018-operator-spacing`'s own module doc already commits this feature area to
"no lexer/`TokenKind` change" (mirroring `data_reference.rs`'s established pattern of doing
lexical classification at a higher layer instead), and the all-digits text check on the existing
`Word` kind is sufficient and strictly simpler.

## §4. Interaction with the existing comma-spacing rule (FR-006)

**Decision**: No change needed to `collect_comma_edits` — it already only touches the gap
immediately around a pair-separator comma (the comma directly before the *next* pair's keyword),
never the interior of a single value like `1-50,75,90-100`'s own list-item commas. The range-dash
rule and the comma rule operate on disjoint token-gap spans within the same value by construction
(one targets `-` gaps, the other targets pair-boundary `,` gaps), so they compose without any new
coordination logic — confirmed by tracing `collect_comma_edits`'s existing `boundaries.iter().
skip(1)` loop, which only ever looks at gaps adjacent to a `kw_start` token, never at a `-`.

## §5. Confirming the spec's core acceptance scenarios against real token shapes

Traced by hand against the actual tokenizer/statement-builder output (`tokenize` +
`build_statements`), not merely reasoned about:

- `FILEO SELECTLINK=1-50,75,90-100` → `classify_statement`'s branch order matters here: a
  statement's very first token being immediately followed by `=` (`assignment_equals_index`,
  checked *before* the generic-leading-word `Control` fallback) makes it `Assignment`, not
  `Control` — confirmed directly against `statement.rs`. A **bare** `SELECTLINK=1-50,75,90-100`
  (nothing before it) is therefore `Assignment{target: "SELECTLINK", value: "1-50,75,90-100"}`,
  and this feature — scoped to `Control` statements only (FR-001) — correctly does **not** apply
  to it (its `-`s get ordinary binary-arithmetic spacing, same as any other `Assignment`). A
  leading word *not* itself immediately followed by `=` (`FILEO`, `FILEI`, or any real control
  word) falls through to `Control{word, pairs: extract_pairs(...)}` instead — this is the real
  shape every real-corpus range-list value actually appears in (confirmed directly against the
  fixture corpus: `FILEO MATO[1] = '...', mo=31-60, name=..., ...` — `mo` is one pair among
  several on one `FILEO`-opened `Control` statement, never a bare standalone pair). Every
  illustrative example in this feature's docs uses a leading control word for this reason, not
  as a stylistic choice. `pair_keyword_boundaries` then finds `SELECTLINK`'s own
  `(kw_start, eq_idx)` pair, so the entire `1-50,75,90-100` run is that one pair's value — both
  `-` occurrences inside it are range-dash candidates, independently.
- `X = 100-1` → an `Assignment` statement (`statement.rs::classify_statement` recognizes the
  `target = value` shape without a leading bare control word), never reaches
  `pair_keyword_boundaries` at all in `collect_comma_edits`'s/this feature's sense (that function
  only fires for `StatementKind::Control`) — so `100-1`'s `-` is never a range-dash candidate,
  confirming Acceptance Scenario 3 without any extra guard needed beyond "only `Control`
  statements have pair-keyword values."
- `FILEO OFFSET=-100,50` → the `-` immediately follows `=` in the token stream, so
  `is_binary_arithmetic` already classifies it as unary (existing `018` behavior, unchanged) —
  it never becomes an `OperatorOccurrence` at all, so the range-dash question never arises for it,
  confirming the spec's "leading negative number" edge case.
