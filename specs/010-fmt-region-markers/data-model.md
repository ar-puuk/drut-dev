# Data Model: FMT Region Markers

No new `Node`/`Block`/`Statement`/`Token`/`Diagnostic` shapes (research.md §5) — this feature's only new data lives in `format.rs`'s internal render pipeline and in the three adapters' existing result types.

## `ProtectedRegions` (new, internal to `voyager-core::format`)

The output of the marker-scan pass (research.md §1-§2), computed once per `render` call from the token stream:

| Field | Type | Meaning |
|---|---|---|
| `lines` | `BTreeSet<u32>` | Every line number (1-based, matching `Position`'s own convention) from a recognized `; FMT: OFF` marker (inclusive) through its matching `; FMT: ON` marker (inclusive), or through the last line of the file if unmatched. |
| `unclosed` | `Vec<Position>` | The start position of every `; FMT: OFF` marker that had no matching `; FMT: ON` before end-of-file, in source order. |

Not a public type — an internal pair (or two return values) inside `format.rs`, since nothing outside the module needs the intermediate `lines` set itself, only its two effects: (a) gating `plan`/`edits` collection (research.md §2), and (b) the `unclosed` list, which *is* exposed publicly (below).

## `FormatResult` (existing type, `crates/voyager-core/src/format.rs`) — one new field

```rust
pub struct FormatResult {
    pub text: String,
    pub changed: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub encoding_fidelity: EncodingFidelity,
    pub unclosed_fmt_off_markers: Vec<Position>,   // NEW
}
```

- Empty `Vec` in the overwhelming common case (no markers, or all markers matched) — same "empty means nothing to report" convention `diagnostics: Vec<Diagnostic>` already uses.
- Deliberately **not** a `Vec<Diagnostic>` append — kept as its own field, its own type, outside the six-category `DiagnosticKind` enum entirely (research.md §3; spec.md FR-010/Assumptions).
- Populated identically by both `format` and `format_bytes` (the latter delegates to the former internally, `format.rs:148`, so no duplicate logic).

## `unclosed_fmt_off_markers` (new, public, standalone)

```rust
pub fn unclosed_fmt_off_markers(source: &str) -> Vec<Position>
```

The same detection `format`/`format_bytes` run internally, exposed standalone for callers that want the signal without paying for a full indentation/casing pass (research.md §3 — `drut-lsp`'s `diagnostics.rs` publish cycle is the motivating caller, since it runs independently of any formatting request).

## `drut-cli::FormatReport` (existing type, `crates/drut-cli/src/format_cmd.rs`) — one new field

```rust
pub struct FormatReport {
    pub outcomes: Vec<(PathBuf, FormatOutcome)>,
    pub read_failures: Vec<ReadFailure>,
    pub unsafe_encoding_files: Vec<PathBuf>,
    pub recovered_encoding_files: Vec<PathBuf>,
    pub unclosed_fmt_off_files: Vec<(PathBuf, Vec<Position>)>,   // NEW
}
```

Same shape and treatment as the two existing informational lists it sits beside: populated in every mode (not gated on `--write`/`--check`/`--diff`), printed via a dedicated `eprintln!` block in `print_report`, never affects `derive_exit_outcome` (informational, not an error — FR-010 explicitly requires "no error occurs").

## `drut-mcp::FormatResultDto` (existing type, `crates/drut-mcp/src/format.rs`) — one new field

```rust
pub struct FormatResultDto {
    pub text: String,
    pub changed: bool,
    pub encoding_fidelity: String,
    pub unclosed_fmt_off_lines: Vec<u32>,   // NEW
}
```

Line numbers only (not full `Position`) — a marker's column is never meaningful (`; FMT: OFF` always starts a comment-only line, so its column is always wherever the line's own leading whitespace ends; callers of the MCP tool have no use for that, only "which lines").

## `drut-lsp::diagnostics.rs` — no new stored type

The existing publish cycle gains one more source of `lsp_types::Diagnostic` values, built directly from `voyager_core::unclosed_fmt_off_markers`'s `Vec<Position>` output, each mapped to `DiagnosticSeverity::HINT` (Phase 1 contract confirms the exact severity) with a distinct `source` tag. No new struct — reuses the LSP protocol's own `Diagnostic` type, which already carries a `source` field for exactly this "which subsystem produced this" distinction.
