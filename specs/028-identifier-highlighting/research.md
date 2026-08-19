# Research: Data-Reference & User-Variable Highlighting

## 1. Both categories reuse `026`'s `TextMate`-scope mechanism, not `027`'s semantic-token one

`027`'s workspace-scoped, semantic-token approach exists only because `drut-lsp`
already emits an unconditional semantic `variable` token for `@name@` that visually
layers over (and hides) a `TextMate`-scope color — `026`'s own research.md §3 finding.
Neither the data-reference family nor generic user identifiers have any existing
semantic-token emission to conflict with (`drut-lsp`'s `semantic_tokens.rs` only emits
`SHORT_IF`, `STATEMENT`, and `VARIABLE` — the last one is `@name@`-only, per
`crates/drut-lsp/src/lib.rs`'s `SemanticTokensLegend`). So both new categories slot
into `026`'s existing `HighlightCategory`/`CATEGORY_SCOPES` machinery in
`highlightCustomization.ts` as two more ordinary entries — `applyHighlightCustomizations`
already iterates `Object.keys(CATEGORY_SCOPES)` generically, so adding two keys is the
entire wiring change; no new function, no new `workspaceState`, no new listener.

## 2. `dataReferences` grammar pattern: one regex, mirrors `data_reference.rs`'s own two match shapes

`voyager-core`'s `data_reference.rs` recognizes a name two ways: exact match, or a
dot-notation prefix (`dba.2.field` → `DBA`). A single `TextMate` `match` pattern covers
both, the same way `#variable-ref` is a single flat match with no `begin`/`end`:

```
(?i)(?<![A-Za-z0-9_])(MI|MO|MW|LI|LW|NI|NW|ZI|ZONES|Z|DBI|DBA|RO|A|B|I|J)(?=\.|(?![A-Za-z0-9_]))
```

The trailing lookahead (`\.` for dot-notation, or a non-identifier boundary for a bare
occurrence) means alternative order inside the group doesn't matter for correctness even
though some names are string-prefixes of others (`Z` of `ZI`/`ZONES`) — Oniguruma
backtracks per-position across the whole alternation-plus-lookahead until one alternative
satisfies the full pattern, so a premature `Z`-only match against `ZONES` fails its
lookahead (next char `O` is a word char) and falls through to the `ZONES` alternative.
Scope: `variable.language.data-reference.drut` — the `variable.language.*` `TextMate`
convention (used elsewhere for a language's own built-in pseudo-identifiers, e.g. `self`/
`this`) is the right semantic fit: this is Voyager's own reserved identifier family, not
a value the script author invented.

**Quote-safety is structural, not a new rule**: `#strings`' `begin`/`end` blocks only
recurse into their own two child patterns (`#variable-ref`, `#string-escape`); a
top-level `#data-references` include is never reachable from inside an already-open
string region, the same guarantee `#pair-keywords`/`#pair-values`/etc. already get for
free from the existing pattern-array structure. No quote-tracking regex needed (unlike
`data_reference.rs`'s own token-level quote-tracking, which exists only because that
code walks a flat token stream with no nested-region concept).

## 3. `dataReferences` must out-rank `pairKeywords`/`pairValues` for the same name (FR-003)

`TextMate` resolves same-start-position ties by array order (`#pair-keywords`' own
existing comment already documents this convention for itself, re: control-words/
statement-words). Placing `#data-references` before `#pair-keywords`/`#pair-values` in
the top-level `patterns` array is the entire mechanism — no regex change to either
existing pattern required. This mirrors `data_reference.rs`'s own FR-005 ownership rule
(`format.rs`'s pair-keyword collection already skips any name `is_data_reference_name`
recognizes, for casing) — same one-name-one-owner principle, now carried into
highlighting for consistency between the two mechanisms.

## 4. `userVariables` grammar pattern: last-resort catch-all, ordering does the filtering

A bareword identifier (`(?<![A-Za-z0-9_])[A-Za-z_][A-Za-z0-9_]*(?![A-Za-z0-9_])`) placed
as the **last** entry in the top-level `patterns` array only ever matches text no earlier,
more specific pattern already claimed — `TextMate` only offers this pattern a starting
position once every earlier pattern has failed to match there. This is what makes
FR-004's "none of: control word / statement word / function call / pair-keyword /
pair-value / data-reference" definition free: it falls out of pattern-list order, not a
hand-written negative-lookahead blacklist (which would have to duplicate every other
pattern's word list inline and go stale the moment any of them changes). Scope:
`variable.other.identifier.drut` — distinct from `variable.other.readwrite.drut`
(`@name@`, semantic-token layered), `variable.parameter.drut` (`pairKeywords`),
`constant.other.drut` (`pairValues`), and the new `variable.language.data-reference.drut`
above.

**The `=`-adjacency trade-off (spec.md Assumptions) falls out of the same ordering
mechanism, deliberately left as-is**: `#pair-keywords`/`#pair-values` are listed well
before `#user-identifiers`, so a bareword immediately before/after `=` is always claimed
by whichever of those two already claims that shape today — `#user-identifiers` never
even gets offered that position. Moving the catch-all earlier to reclaim those positions
for `userVariables` was considered and rejected: it would silently change
`drut.highlight.pairKeywords`/`drut.highlight.values`'s existing, already-shipped
behavior for every current user of those two settings, which FR-006 forbids.

## 5. Label/`ShellEscape` exclusion (FR-004a) needs two small new line-scoped patterns

Unlike `data_reference.rs`, this grammar has no real `Statement`/`StatementKind` to
branch on — it is regex-over-text, so "skip `Label`/`ShellEscape` content" has to be
implemented the same structural way `#strings`/`#block-comment` already exclude their
own interiors from the top-level patterns: give each its own whole-region match/scope
placed early in the array, so the new catch-all identifier pattern (and, for
`ShellEscape`, every other pattern too) never gets offered a starting position inside it.

- **`#shell-escape`** (new): `^[ \t]*(\*.*)$` — a `ShellEscape` statement
  (`statement.rs`'s `classify_statement`: first token is a bare `*` punctuation) is one
  physical line of literal OS shell text, not Voyager syntax at all. One flat `match`
  covering the whole line (leading `*` included) under its own scope
  (`meta.embedded.shell-escape.drut`) means nothing else — not `#operators`, not the two
  new categories — reaches inside it. This is a small, deliberate, in-scope
  side-effect: today the leading `*` incidentally renders as `keyword.operator.drut`
  (it's just "a `*` character" to the un-anchored `#operators` pattern); after this
  change it's part of the shell-escape line's own scope instead — a minor, correct
  fix, not a regression, since that `*` was never really "multiplication" to begin with.
  Known, accepted approximation: a continuation line that happens to start with a bare
  `*` mid-expression would be misclassified — the same class of shape-over-truth
  trade-off `#pair-keywords`'s own doc comment already accepts for itself; no worse than
  the grammar's existing precedent.
- **`#label`** (new): `^[ \t]*(:)[ \t]*([A-Za-z_][A-Za-z0-9_]*)` — a `Label` statement
  (`statement.rs`: first token is `:` punctuation, second is the name) is exactly this
  shape. One flat match, own scope (`entity.name.label.drut`, the conventional `TextMate`
  category for a jump-target/definition name — distinct from every value-ish scope
  above), consuming the name so `#user-identifiers` never reaches it. Bonus: labels get
  real, dedicated highlighting for the first time (previously unstyled), not just
  exclusion.

Both new patterns are placed immediately after `#comments` in the top-level array (before
`#strings`), matching where `#block-comment`'s own whole-region-consuming sibling already
sits. Placement relative to `#strings` doesn't actually matter for correctness — both new
patterns' matches start at true line-start (position 0 after optional leading
whitespace), which is always the earliest possible start on that line, so `TextMate`'s
leftmost-match rule already guarantees they win regardless of array position relative to
patterns that could only match later on the same line.

**`GOTO`'s target-name argument** (`GOTO STEP0` — a *reference* to a label, not a
declaration) is deliberately left alone by `#label` (which only matches a leading `:`)
and falls through to `#user-identifiers` like any other bareword — reasonable, since a
label reference is closer to "a user-defined name used as a value" than to the
declaration site itself, and this wasn't part of the FR-004a clarification's scope.

## 6. Existing test harnesses cover both categories with no new tooling

`editors/vscode/test/grammar.test.ts` already drives the real grammar file through
`vscode-textmate`/`vscode-oniguruma` (no `VS Code` instance needed) — new tokenization
spot-checks for `#data-references`/`#user-identifiers`/`#shell-escape`/`#label` are more
cases in that same file, same pattern `024-function-call-highlighting`'s own grammar
tests already used. `editors/vscode/test/highlightCustomization.test.ts` already unit
-tests `CATEGORY_SCOPES`/`mergeHighlightRules` via plain `ts-node`, zero `vscode` package
dependency — extending `HighlightCategory`/`CATEGORY_SCOPES` with two more entries is
covered by that file's existing assertions plus a couple of new ones for the two new
keys.
