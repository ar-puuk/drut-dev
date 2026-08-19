# Data Model: Unused `@token@` Diagnostic

## §1. `voyager-core` additions

### `token_resolution::all_variable_refs_including_openers` (new)

```rust
/// Every `@name@` reference in `nodes`, source order, at any nesting depth,
/// INCLUDING a reference on a block-opener statement's own line (research.md
/// §1-2) -- unlike `all_variable_refs`, which excludes that position by a
/// pre-existing, documented, and separately-tested data-model constraint
/// this feature must not disturb. Scans `Block::opener_tokens` (added for
/// the 028-adjacent casing fix) via a new `collect_opener_token_slices`
/// helper, structurally identical to the existing
/// `collect_if_condition_token_slices`.
pub fn all_variable_refs_including_openers(nodes: &[Node]) -> Vec<VariableRefAt>;
```

- Pure, no I/O, never panics — same contract every other function in this module already has.
- `all_variable_refs` itself: **unchanged**, byte-for-byte — its own existing test
  (`all_variable_refs_excludes_a_block_opener_reference`) keeps passing unmodified, proving this.
- `VariableRefAt`, `Assignment`, `all_assignments`, `resolve_token_value`, `read_file_refs`,
  `Source`, `ResolvedTokenValue`: **unchanged** — this feature adds one new function alongside
  the existing ones, no existing type or signature changes.

```rust
// New private helper, alongside collect_if_condition_token_slices:
fn collect_opener_token_slices<'a>(nodes: &'a [Node], out: &mut Vec<&'a [Token]>) {
    for node in nodes {
        if let Node::Block(b) = node {
            if !b.opener_tokens.is_empty() {
                out.push(&b.opener_tokens);
            }
            collect_opener_token_slices(&b.children, out);
            if let BlockKind::If { branches } = &b.kind {
                for branch in branches {
                    collect_opener_token_slices(&branch.children, out);
                }
            }
        }
    }
}
```

- `If` blocks contribute nothing here (`opener_tokens` is always empty for them, by construction
  in `block.rs` — condition tokens are separately scanned via `collect_if_condition_token_slices`
  already), so no double-counting risk to guard against explicitly.
- `all_variable_refs_including_openers` itself: identical body to `all_variable_refs`, plus one
  more loop over `collect_opener_token_slices`'s output before the final source-order sort.

## §2. `drut-lsp` additions

### `unused_token.rs` (new module)

```rust
/// One unused assignment, ready to become an LSP Hint diagnostic.
pub(crate) struct UnusedAssignment {
    pub target: String,
    pub value_span: Span,
    pub statement_span: Span,
}

/// Every `Assignment` in `doc` whose target name has no `@name@` reference
/// anywhere in scope: same file (including block-opener positions, FR-003),
/// plus one level of directly-included, statically-resolvable `READ FILE`
/// files (research.md §4). Every dead assignment site is returned
/// independently (Clarification Q1) -- no dedup to one-per-name. Applies
/// unconditionally, regardless of whether `doc` itself participates in any
/// `READ FILE` relationship (Clarification Q2).
pub(crate) fn unused_token_assignments(
    uri: &lsp_types::Uri,
    doc: &OpenDocument,
) -> Vec<UnusedAssignment> {
    let mut referenced: std::collections::HashSet<String> =
        voyager_core::all_variable_refs_including_openers(&doc.parse_result.nodes)
            .into_iter()
            .map(|r| r.name.to_ascii_uppercase())
            .collect();

    for included in hover::collect_included_files(uri, doc) {
        referenced.extend(
            voyager_core::all_variable_refs_including_openers(&included.nodes)
                .into_iter()
                .map(|r| r.name.to_ascii_uppercase()),
        );
    }

    voyager_core::all_assignments(&doc.parse_result.nodes)
        .into_iter()
        .filter(|a| !referenced.contains(&a.target.to_ascii_uppercase()))
        .map(|a| UnusedAssignment {
            target: a.target.to_string(),
            value_span: a.value_span,
            statement_span: a.statement_span,
        })
        .collect()
}
```

- One disk-I/O pass (`collect_included_files`, reused unmodified from `hover.rs`), same shape
  `020` already pays for its own stream.
- Case-insensitive matching via uppercase normalization — same convention `resolve_token_value`
  already uses internally (`eq_ignore_ascii_case`), expressed here as a `HashSet` membership
  check instead of a per-item comparison since this is a set-difference, not a per-position
  resolve.
- Deliberately does **not** call `resolve_token_value` at all — this feature only needs "is this
  name referenced anywhere in scope," not "which specific assignment does a given reference
  resolve to," so the cheaper aggregate-set shape is correct, not merely convenient
  (research.md §4).

## §3. `diagnostics.rs` — fifth chained stream

```rust
// New, alongside the existing four streams -- same DiagnosticSeverity::HINT
// shape, source shared with UndefinedToken ("drut-token"), new code
// ("UnusedToken", research.md §5).
let unused_token_diagnostics: Vec<lsp_types::Diagnostic> =
    unused_token::unused_token_assignments(uri, doc)
        .into_iter()
        .map(|a| lsp_types::Diagnostic {
            range: to_lsp_range(&doc.text, a.statement_span),
            severity: Some(DiagnosticSeverity::HINT),
            code: Some(lsp_types::NumberOrString::String("UnusedToken".to_string())),
            code_description: None,
            source: Some("drut-token".to_string()),
            message: format!(
                "'{}' is assigned but never referenced via '@{}@' in this file or a directly \
                 included one — it may still be used elsewhere Drut can't see",
                a.target, a.target
            ),
            related_information: None,
            tags: None,
            data: None,
        })
        .collect();

let diagnostics = structural_diagnostics
    .chain(fmt_marker_diagnostics)
    .chain(config_warnings)
    .chain(undefined_token_diagnostics)
    .chain(unused_token_diagnostics)
    .collect();
```

- Range covers the whole assignment statement (`a.statement_span`), not just the target name or
  value — an assignment is the thing that's "unused," and underlining the full statement is the
  clearer signal for a whole dead line, matching how `UnclosedFmtOff`/`DrutTomlProblem` also span
  more than a single token where the notice is about the statement/file as a whole.
- Message wording hedges the same way `UndefinedToken`'s does ("may still be used elsewhere Drut
  can't see") — honest about the Clarification Q2 blind spot rather than asserting the name is
  genuinely dead project-wide.
- `uri`/`doc` already in scope at this point in `publish()` — no new parameter threading.

## §4. What this feature does *not* touch

- `voyager_core::Diagnostic`/`DiagnosticKind`: unchanged, zero new variants.
- `voyager_core::token_resolution::all_variable_refs`/`variable_ref_at`: unchanged — a new
  function is added alongside them, not a modification (research.md §1).
- `drut-cli`, `drut-mcp`: unchanged, zero new flags/params/DTO fields (FR-005 — LSP-only).
- `drut-config`: unchanged, zero new `[format]` (or any other) fields — no configuration surface.
- `002-cli-check-format`'s `check`/`diagnose` "never a narrowed subset of `DiagnosticKind`"
  contract: unaffected, since this stream never reaches either command.
