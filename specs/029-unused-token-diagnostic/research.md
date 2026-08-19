# Research: Unused `@token@` Diagnostic

## §1: Why a new function, not a modified `all_variable_refs`

`all_variable_refs` (`crates/voyager-core/src/token_resolution.rs`) is documented, tested
(`all_variable_refs_excludes_a_block_opener_reference`), and consumed by
`020-undefined-token-diagnostic`'s `undefined_token_positions` with an explicit, relied-upon
contract: a `@name@` reference on a block-opener line is absent from its result. For
`020`'s own purpose (flag a reference with no resolvable definition) that absence is a safe,
documented false-negative — the reference is simply never checked at all, so it's never wrongly
flagged as undefined either.

For *this* feature the same absence is not safe. If a name is used only on a block-opener line
(`RUN PGM=@Prog@`) and this feature reused `all_variable_refs` as its "is this name used"
signal, that name's assignment would look completely unreferenced and get flagged — a genuine
false positive on a valid script, which constitution Principle IV treats as strictly worse than
a missed detection.

Two ways to fix this were considered:

- **Modify `all_variable_refs` in place** to also scan block-opener tokens. Rejected: it would
  invert an existing, explicitly-tested contract another shipped feature (`020`) depends on,
  requiring that feature's own test to be rewritten as part of an unrelated feature's work —
  exactly the kind of unplanned cross-feature coupling `020`'s own FR-004 (「keep every existing
  consumer... completely untouched」) warns against for `DiagnosticKind`, applied here by the
  same reasoning to a plain function's contract.
- **Add a new function alongside it** (`all_variable_refs_including_openers`) that this feature
  alone consumes. Chosen: `020` and its tests are provably untouched (confirmed by running its
  existing suite unmodified after this change), and the new function is a thin, obviously-correct
  composition of existing pieces (see §2) rather than new traversal logic.

## §2: The opener-token scan is nearly free

`Block::opener_tokens: Vec<Token>` (added in `crates/voyager-core/src/block.rs` earlier this
session, for the `028`-adjacent `casing_data_references` block-opener-value-blindness fix) already
carries the full token stream of every `Run`/`Process`/`Loop`/`JLoop`/`LinkLoop`/
`DistributeMultistep` block's opener statement — `If` gets `Vec::new()` deliberately, since its
condition is separately tracked via `IfBranch.condition` (already scanned by
`collect_if_condition_token_slices`, avoiding double-counting the same tokens twice).

`token_resolution.rs` doesn't read `opener_tokens` at all today — it wasn't the field's original
purpose, but it already contains exactly what this feature needs. The new function needs only
one new traversal helper, `collect_opener_token_slices`, structurally identical to the existing
`collect_if_condition_token_slices` (same recursive `Node::Block` walk, pushing one `&[Token]`
slice per block instead of per `IfBranch`), plus reuse of the existing
`push_variable_refs_in_tokens` helper `all_variable_refs` already uses. No new parsing, no new
`Block` field, no change to any existing collector.

Verified directly (not assumed): `RUN PGM=@Prog@\nENDRUN\n` parses to a `Run`-kind `Block` whose
`opener_tokens` contains the `VariableRef { name: "Prog" }` token — confirmed by reading
`parse_run`'s construction in `block.rs` (the same code path `opener_tokens: opener_tokens.clone()`
lines feed).

## §3: `all_assignments` needs no changes

`all_assignments` already returns every `Assignment` statement's target name, span, and
statement span, at any nesting depth, in source order — exactly the shape this feature diffs
against. Confirmed directly against its existing implementation and test coverage
(`all_assignments_finds_top_level_and_nested`): no gap analogous to the block-opener one exists
here, because an `Assignment` statement is never itself a block opener (block openers are
`Control`-shaped in this grammar, not `Assignment`-shaped) — there's no equivalent "discarded
tokens" case for assignment targets to worry about.

## §4: `READ FILE` inclusion reuse

`hover::collect_included_files` (already `pub(crate)`, widened for `020`'s own use) is reused
unmodified for the same one-level, static-path-only inclusion this feature's scope needs
(spec.md FR-001). Unlike `020`, this feature doesn't need `resolve_token_value`'s per-position
"most recent assignment visible at pos" logic at all — it only needs the *set of every name
referenced* across the same file and each included file, which
`all_variable_refs_including_openers` already returns directly for a given `nodes` — called once
for the open document, once per included file, and the resulting name sets unioned
(case-insensitively) before diffing against `all_assignments`' targets.

## §5: Diagnostic source/code naming

`"drut-token"` (shared with `UndefinedToken`) rather than a new source string: both diagnostics
are about the same conceptual domain (`@token@` definition/reference bookkeeping), and
`lsp_types::Diagnostic.code` (`"UnusedToken"` vs. `"UndefinedToken"`) is what actually
distinguishes them programmatically — matching how a single linter typically uses one source
name across several related rule codes, rather than minting a new source per rule.
