# Data Model: Undefined `@token@` Diagnostic

## §1. `voyager-core` additions

### `token_resolution::all_variable_refs` (new)

```rust
/// Every `@name@` reference in `nodes`, source order, at any nesting depth
/// (research.md §2) — the "all matches" counterpart to `variable_ref_at`'s
/// "first match at a position", same traversal, same "all instead of one"
/// shape `all_assignments` already established.
pub fn all_variable_refs(nodes: &[Node]) -> Vec<VariableRefAt>;
```

- Pure, no I/O, never panics — same contract every other function in this module already has.
- Reuses `collect_statements`/`collect_if_condition_token_slices` exactly as `variable_ref_at`
  does — a block-opener `@token@` is therefore structurally absent from the result, not
  filtered out by new logic (research.md §3).
- `VariableRefAt`, `resolve_token_value`, `read_file_refs`, `Source`, `ResolvedTokenValue`:
  **unchanged** — this feature adds one new function, no existing type or signature changes.

## §2. `drut-lsp` additions

### `hover.rs` visibility change

```rust
// was: fn collect_included_files(...) -> Vec<IncludedFile>
// was: struct IncludedFile { ... }
pub(crate) fn collect_included_files(uri: &lsp_types::Uri, doc: &OpenDocument) -> Vec<IncludedFile>;
pub(crate) struct IncludedFile { /* fields unchanged */ }
```

- No behavior change — same function, same struct, now reachable from a sibling module
  (research.md §4). `hover.rs`'s own call sites are unaffected.

### `undefined_token.rs` (new module)

```rust
/// Every unresolvable `@token@` reference in `doc`, ready to become an LSP
/// Hint diagnostic (research.md §1–§4). Pure with respect to `doc`/`nodes`;
/// the disk I/O for READ FILE inclusion is `collect_included_files`'s own
/// existing, already-graceful-on-failure responsibility.
pub(crate) fn undefined_token_positions(
    uri: &lsp_types::Uri,
    doc: &OpenDocument,
) -> Vec<voyager_core::token_resolution::VariableRefAt> {
    let included_files = hover::collect_included_files(uri, doc);
    let included: Vec<(Span, Vec<Node>)> = included_files
        .iter()
        .map(|f| (f.read_file_statement_span, f.nodes.clone()))
        .collect();

    voyager_core::token_resolution::all_variable_refs(&doc.parse_result.nodes)
        .into_iter()
        .filter(|var_ref| {
            voyager_core::token_resolution::resolve_token_value(
                &doc.parse_result.nodes,
                var_ref.span.start,
                &included,
                &var_ref.name,
            )
            .is_none()
        })
        .collect()
}
```

- One disk-I/O pass (`collect_included_files`) shared across every reference in the document,
  not re-read per reference — same cost shape hover already pays once per hover request, just
  amortized across every reference instead of one.
- `filter`, not `filter_map`/manual loop — every step is already total (never panics, never
  needs per-item error handling beyond what `resolve_token_value`/`collect_included_files`
  already do internally).

## §3. `diagnostics.rs` — fourth chained stream

```rust
// New, alongside the existing fmt_marker_diagnostics/config_warnings streams
// (research.md §1) — same DiagnosticSeverity::HINT shape, new source string.
let undefined_token_diagnostics: Vec<lsp_types::Diagnostic> =
    undefined_token::undefined_token_positions(uri, doc)
        .into_iter()
        .map(|var_ref| lsp_types::Diagnostic {
            range: to_lsp_range(&doc.text, var_ref.span),
            severity: Some(DiagnosticSeverity::HINT),
            code: Some(lsp_types::NumberOrString::String("UndefinedToken".to_string())),
            code_description: None,
            source: Some("drut-token".to_string()),
            message: format!(
                "'@{}@' has no assignment this tool can find in this file or a directly \
                 included one — it may still be defined elsewhere Drut can't see",
                var_ref.name
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
    .collect();
```

- Message wording deliberately hedges ("may still be defined elsewhere Drut can't see") rather
  than asserting non-existence — matches the Hint severity's own honesty about this check's
  bounded confidence (spec.md Assumptions), constitution Principle II (own words, not vendor
  phrasing) trivially satisfied since there's no vendor concept being described at all here.
- `uri`/`doc` are already in scope at this exact point in `publish()` — no new parameter
  threading needed beyond calling the new function.

## §4. What this feature does *not* touch

- `voyager_core::Diagnostic`/`DiagnosticKind`: unchanged, zero new variants.
- `drut-cli`, `drut-mcp`: unchanged, zero new flags/params/DTO fields (FR-005 — LSP-only).
- `drut-config`: unchanged, zero new `[format]` (or any other) fields — this capability has no
  configuration surface at all (research.md §5).
- `002-cli-check-format`'s `check`/`diagnose` "never a narrowed subset of `DiagnosticKind`"
  contract: unaffected, since this stream never reaches either command.
