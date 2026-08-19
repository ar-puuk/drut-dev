# Research: Function-Call Casing Normalization

## 1. Name list: reused, not re-derived

The 138-name, category-grouped function list is `024-function-call-highlighting/
research.md` §2 verbatim (names only) — built from a complete reading of two vendor
documentation editions (Cube Voyager 6.5.1, OpenPaths Cube/CUBE CONNECT Edition),
cross-validated against each other. No new vendor-doc research is performed for this
feature; the list is ported into `voyager-core` as-is (§2 below decides exactly where).

## 2. Architectural precedent: `data_reference.rs`, not `collect_statement_casing_edits`

Two existing mechanisms in `voyager-core` apply casing to a category of recognized name:

- **`control_words`/`pair_keywords`** (`format.rs`'s `collect_statement_casing_edits`/
  `collect_block_casing_edits`): walk the parsed `Statement`/`Block` AST directly.
  Critically, `collect_statement_casing_edits` **returns early for any non-`Control`
  statement** (`if !matches!(stmt.kind, StatementKind::Control { .. }) { return; }`) —
  correct for those two categories, since a control word or pair-keyword name only ever
  appears inside a `Control` statement or a `Block` opener.
- **`data_references`** (`data_reference.rs`'s `data_reference_occurrences`, wired
  separately into `format.rs`'s `render()` alongside the AST walk, not through it): a
  **quote-aware scan over every statement's raw token list**, explicitly covering both
  `Control` and `Assignment` statements (module docs: "a data-reference token can be a
  pair-keyword name or a value inside a Control statement, or the target/value of an
  Assignment").

**Decision: `function_calls` follows the `data_references` shape, not the
`control_words`/`pair_keywords` shape.** A function call routinely appears on an
`Assignment`'s right-hand side (`RouteName = REPLACESTR(...)`, spec.md Acceptance
Scenario 1) — the exact case `collect_statement_casing_edits`'s early return would skip
entirely. `data_reference_occurrences` already solves this identical problem for its own
category; `function_call_occurrences` (new, in a new `function_call.rs` module) mirrors
its structure: a `collect_tokens`-style quote-aware scan (single-/double-quote tracking,
identical to `data_reference.rs`'s own, itself mirroring `statement.rs`'s
`pair_keyword_boundaries`), producing one `FunctionCallOccurrence { name, span }` per
match, wired into `format.rs`'s `render()` the same way `data_reference_occurrences` is.

## 3. The one condition `data_references` doesn't need: `(`-adjacency

`data_references` matches a recognized name in any position, unconditionally. This
category cannot: a function name and a pair-keyword name/control word share the same
lexical space (both are just `Word` tokens), and two real names in the 138-entry list
collide with existing `voyager-core` vocabulary by coincidence:

- **`FORMAT`**: a real `FILEO` pair-keyword (`keywords.rs`: `pair_entry("FORMAT",
  &["FILEO"])`) *and* a real Character/String function (`FORMAT(x,w,dec,str)`).
- **`LOG`**: a real control/statement word (`keywords.rs`: `pair_entry("VAR", &["FILEI",
  "LOG"])` records `LOG` as a control word `VAR` pairs with) *and* a real Numeric
  function, natural logarithm (`LOG(x)`).

Both are structurally disambiguated by what immediately follows: a pair-keyword name is
followed by `=` (`pair_keyword_boundaries`'s own signal); a control word leads a
statement, followed by whitespace then its first keyword; a function call is followed by
`(` with **zero** intervening whitespace (`024`'s own `research.md` §5/§6 decision,
confirmed against both vendor doc editions and real corpus usage). `FORMAT=CSV` can never
also satisfy "followed immediately by `(`"; `X = FORMAT(volume,8,2,',')` can never also
satisfy "followed immediately by `=`". The `(`-adjacency check is therefore not merely
consistent with `024`'s highlighting design — it is the load-bearing mechanism that keeps
`FORMAT`/`LOG`'s two real roles from ever colliding under this feature (spec.md FR-002/
FR-004, User Story 2, SC-004).

## 4. Detecting `(`-adjacency: reuse the existing gap-measurement technique

`voyager-core` has no whitespace token — gaps between tokens are implicit, recovered by
comparing `Span` positions (`token.rs`'s `TokenKind` has no whitespace variant;
`operator_spacing.rs`'s `push_gap_edit` already measures an inter-token gap this way for
its own, unrelated purpose). `function_call_occurrences` reuses the identical technique:
a `Word` token at index `i` is a function-call occurrence only when `tokens[i+1]` exists,
is `TokenKind::Punctuation` with text `"("`, **and** `tokens[i+1].span.start ==
tokens[i].span.end` (zero-width gap — the same equality check, not a new concept).

## 5. Ownership when a name is claimed by another category: nothing to add

Unlike `data_references`' own overlap handling (`format.rs` explicitly skips a
pair-keyword-shaped name when `data_reference::is_data_reference_name` claims it first —
FR-004's "single ownership" requirement), `function_calls` needs **no equivalent skip
logic** in the *other* direction: `collect_statement_casing_edits`/
`collect_block_casing_edits`'s existing pair-keyword/control-word collection never even
considers a token followed by `(` (they key off `=`-boundaries and statement-leading
position respectively), and `function_call_occurrences` only ever fires on a token
followed by `(`. The two collectors are naturally disjoint by construction (§3) — FR-004
is satisfied structurally, not by an added conditional, but MUST still be verified by a
real test (`FORMAT`/`LOG` fixtures under differing category conventions, spec.md SC-004),
not merely asserted.

## 6. Golden-fixture convention: one variant, `Upper`, matches existing precedent

Every prior casing/formatter feature's golden-fixture addition
(`format_corpus.rs` module comments: `018`, `019`, `023`) adds **exactly one** new golden
directory, applied to the same fixed, already-human-reviewed 9-file `real_corpus` subset
`golden_normalize`/`golden_data_references` established — not the full 161-file corpus,
and not one directory per `CasingConvention` variant (`golden_data_references` itself
tests only `Upper`, never a separate `_lower` directory). This feature follows the same
pattern: one `golden_casing_function_calls/real_corpus` directory,
`function_calls: CasingConvention::Upper`, mirroring `data_references_upper_
indent_2_options()`'s existing shape (`format_corpus.rs`). `Lower`'s byte-for-byte
output is verified by targeted unit tests in `function_call.rs`/`format.rs` (the
transformation is mechanically symmetric — `to_ascii_uppercase`/`to_ascii_lowercase` on
the exact same matched span, `format.rs`'s existing `edit_for_span` already implements
both branches generically), not by a second golden-fixture directory.

**`Lower`'s real-corpus *idempotence* is still checked, at zero extra fixture cost.**
`check_idempotent` (unlike the golden-diff comparison) needs no maintained "expected
output" file — it formats a fixture twice and diffs the two results against each other,
against whatever `FormatOptions` it's called with. `real_corpus_fixtures_are_idempotent_
under_normalize`/`..._under_data_references_upper_indent_2` already demonstrate this: the
idempotence test for a given options-variant doesn't require that variant to have its own
golden directory. So `real_corpus_fixtures_are_idempotent_under_function_calls_lower`
(`FormatOptions { casing: CasingSettings { function_calls: CasingConvention::Lower,
..Default::default() }, ..Default::default() }`) is added alongside the `Upper` golden
variant's own idempotence test, closing spec.md SC-003's full "every non-`preserve`
value... for every real corpus fixture" claim without a second golden directory.

## 7. Surface naming: mirrors the three existing categories exactly

| Surface | Existing name (e.g. `pair_keywords`) | New name |
|---|---|---|
| `drut.toml` `[format]` field | `casing_pair_keywords` | `casing_function_calls` |
| CLI flag | `--casing-pair-keywords` | `--casing-function-calls` |
| MCP `format` tool parameter | `casing_pair_keywords` | `casing_function_calls` |
| VS Code setting | `drut.format.casingPairKeywords` | `drut.format.casingFunctionCalls` |
| `voyager-core` `CasingSettings` field | `pair_keywords` | `function_calls` |

No naming decision needed beyond direct substitution into the already-established
group-word-leads pattern (`CHANGELOG.md`'s `[Unreleased]` rename entry documents this
convention's rationale).
