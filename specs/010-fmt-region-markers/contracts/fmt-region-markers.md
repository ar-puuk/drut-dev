# Contract: FMT Region Markers

Amends `002-cli-check-format/spec.md` and `002-cli-check-format/contracts/formatting-api.md` in place (matching how `007`, `008`, and `009` each amended `002`'s FR-012 — not a new, competing formatting contract file). Also touches `003-lsp-vscode-extension`'s LSP diagnostics contract and `004-mcp-server`'s MCP tool contract, additively — no existing behavior in either changes.

## `voyager-core` — new functions and one new `FormatResult` field

### Marker recognition (research.md §1, §4)

A `; FMT: OFF` / `; FMT: ON` marker is a `TokenKind::LineComment` token where:
1. No other token in the same tokenized stream has `span.start.line` equal to this token's `span.start.line` (whole-line, not trailing — FR-001/FR-002).
2. `token.text.trim_start_matches(';').trim()`, split once on `:`, with both sides trimmed and compared case-insensitively, equals `("FMT", "OFF")` or `("FMT", "ON")`.

### Region computation (research.md §2)

```rust
/// Internal — not part of the public API. Returns the set of protected line
/// numbers (inclusive of both marker lines, or through EOF if unmatched)
/// and the positions of any `; FMT: OFF` marker left unmatched.
fn protected_regions(tokens: &[Token]) -> (BTreeSet<u32>, Vec<Position>)
```

Single left-to-right scan over the token stream, tracking one `Option<u32>` ("currently-open marker's line, if any"): a recognized `FMT: OFF` while `None` opens a region (records the line, starts inserting every subsequent line into the protected set); `FMT: OFF` while already `Some` is a no-op (spec.md US1 Acceptance Scenario 4); `FMT: ON` while `Some` closes the region (inserts up through and including the `FMT: ON` line, clears the open-marker state); `FMT: ON` while `None` is a no-op (US1 Acceptance Scenario 5). Any region still open at end-of-scan contributes its opening marker's position to the second return value and every remaining line through EOF to the protected set.

### Public API additions

```rust
// crates/voyager-core/src/format.rs
pub struct FormatResult {
    pub text: String,
    pub changed: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub encoding_fidelity: EncodingFidelity,
    pub unclosed_fmt_off_markers: Vec<Position>,   // NEW
}

/// Standalone — for callers that want this signal without a full format
/// pass (drut-lsp's diagnostics.rs; research.md §3).
pub fn unclosed_fmt_off_markers(source: &str) -> Vec<Position>
```

`crates/voyager-core/src/lib.rs` re-exports `unclosed_fmt_off_markers` alongside the existing `format`/`format_bytes` re-exports — no new module needed, stays in `format.rs`.

### Gate points inside `render` (research.md §2) — exact diff shape

```rust
fn render(source: &str, nodes: &[Node], diagnostics: &[Diagnostic], options: FormatOptions) -> String {
    let raw_lines = split_lines(source);
    let char_lines: Vec<Vec<char>> = /* unchanged */;
    let tokens = tokenize(source);                              // NEW — already computed by parse() internally today; render() gains its own call since it doesn't currently receive the token stream
    let (protected, _unclosed) = protected_regions(&tokens);     // NEW

    let diagnosed_openers = diagnosed_block_openers(diagnostics);
    let mut indent_plan: IndentPlan = BTreeMap::new();
    plan_indentation(nodes, &char_lines, &diagnosed_openers, &protected, options.top_level_indent, &mut indent_plan);
    //                                                        ^^^^^^^^^ NEW param, threaded through plan_block/plan_children too

    let mut casing_edits: Vec<CasingEdit> = Vec::new();
    if let Some(convention) = options.casing {
        collect_casing_edits(nodes, &char_lines, &protected, convention, &mut casing_edits);
        //                                        ^^^^^^^^^ NEW param, threaded through to push_if_present
    }
    /* rest of render() unchanged — see research.md §2 */
}
```

`format`/`format_bytes` (the public entry points) compute `protected_regions` once and populate `FormatResult.unclosed_fmt_off_markers` from its second return value — `render` itself only needs the first (the line set), so it's an internal helper detail whether `render` recomputes or receives the set as a parameter; either is correct, final choice left to `/speckit-tasks`/implementation (no behavioral difference, pure internal wiring).

Every `plan.insert(line, value)` call site named in research.md §2 becomes `if !protected.contains(&line) { plan.insert(line, value); }`. `push_if_present` gains the same one-line guard at its top.

## `drut-cli` — new report field, new stderr notice (mirrors existing pattern exactly)

```rust
// crates/drut-cli/src/format_cmd.rs
pub struct FormatReport {
    // ...existing fields...
    pub unclosed_fmt_off_files: Vec<(PathBuf, Vec<Position>)>,   // NEW
}
```

Populated in the same per-file loop that already checks `result.encoding_fidelity` (`format_cmd.rs:100-108`): `if !result.unclosed_fmt_off_markers.is_empty() { report.unclosed_fmt_off_files.push((file.path.clone(), result.unclosed_fmt_off_markers)); }`.

`print_report` gains a third `eprintln!` block, same shape as the two existing ones (`format_cmd.rs:208-225`):

```text
N file(s) have an unclosed '; FMT: OFF' marker (protection extended to end of file):
  path/to/file.s (line 42)
```

No `derive_exit_outcome` change — this list, like `recovered_encoding_files`, never affects exit code (informational only; FR-010's "no error occurs").

## `drut-mcp` — new response field

```rust
// crates/drut-mcp/src/format.rs
pub struct FormatResultDto {
    pub text: String,
    pub changed: bool,
    pub encoding_fidelity: String,
    pub unclosed_fmt_off_lines: Vec<u32>,   // NEW — line numbers, Position::line only
}
```

`format()`'s existing body maps `result.unclosed_fmt_off_markers.iter().map(|p| p.line).collect()` into the new field — one line of new code at the existing `FormatResultDto { ... }` construction site.

## `drut-lsp` — new diagnostic source, independent publish path

`diagnostics.rs`'s `publish` function gains a second diagnostics source, merged into the same `PublishDiagnosticsParams` list as the existing structural diagnostics:

```rust
let fmt_marker_diagnostics = voyager_core::unclosed_fmt_off_markers(&doc.text)
    .into_iter()
    .map(|pos| lsp_types::Diagnostic {
        range: to_lsp_range(&doc.text, Span::new(pos, pos)),
        severity: Some(DiagnosticSeverity::HINT),
        code: Some(lsp_types::NumberOrString::String("UnclosedFmtOff".to_string())),
        code_description: None,
        source: Some("drut-fmt".to_string()),   // distinct from structural diagnostics' "drut"
        message: "'; FMT: OFF' has no matching '; FMT: ON' — formatting is suppressed through end of file".to_string(),
        related_information: None,
        tags: None,
        data: None,
    });
let diagnostics: Vec<_> = doc.parse_result.diagnostics.iter().map(/* existing */).chain(fmt_marker_diagnostics).collect();
```

`DiagnosticSeverity::HINT` (not `ERROR`, which every existing structural diagnostic uses) and the distinct `source: "drut-fmt"` tag are both deliberate — an editor client can filter/style these separately from real structural problems, and no existing test asserting `severity == ERROR` for the seven/eight real `DiagnosticKind` values is affected, since this is a parallel, additive stream, not a new arm in `kind_name`/the existing `map` closure.

`formatting.rs`/`range_formatting.rs` themselves need **no** change — the notice is entirely carried by the independent `diagnostics.rs` publish cycle (research.md §3), not the formatting response.

## Amendments to `002-cli-check-format/spec.md` / `contracts/formatting-api.md`

New FR (numbered at implementation time, following `002`'s own file-local sequence — see `009`'s "Fix FR number collision" precedent for why this must be checked against the live file, not assumed): `format`/`format_bytes` MUST leave every line inside a `; FMT: OFF`/`; FMT: ON` region untouched, and MUST report any region left open at end-of-file via a dedicated, non-`Diagnostic` signal. `contracts/formatting-api.md`'s prose description of what the renderer touches gains one sentence: "...except any line inside a `; FMT: OFF`/`; FMT: ON` region (`010-fmt-region-markers`), which is never touched regardless of any other rule in this list."
