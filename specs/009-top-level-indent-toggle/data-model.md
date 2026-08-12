# Phase 1 Data Model: Top-Level Indent Default Revert

## New: `TopLevelIndentMode`

```rust
/// spec.md FR-001/FR-002. Two-valued, no "off" state -- format always
/// either preserves or normalizes top-level indentation (research.md §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TopLevelIndentMode {
    /// Leave existing top-level (depth-0) indentation exactly as written
    /// -- the 007-era, and now again default, behavior.
    #[default]
    Preserve,
    /// Force every top-level line to column 0, unconditionally -- 008's
    /// original behavior, unchanged, now opt-in.
    Normalize,
}
```

Lives in `crates/voyager-core/src/format.rs`, re-exported from `lib.rs`
alongside `CasingConvention`.

## Changed: `FormatOptions`

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct FormatOptions {
    pub casing: Option<CasingConvention>,
    /// NEW. Defaults to `Preserve` via `TopLevelIndentMode`'s own
    /// `#[default]` — `FormatOptions`'s derived `Default` impl picks this
    /// up automatically, but every call site is still individually
    /// verified (research.md §2), not trusted transitively.
    pub top_level_indent: TopLevelIndentMode,
}
```

Both fields are independent — `casing` unaffected by this feature,
`top_level_indent` unaffected by any future casing change.

## Changed: `IndentPlan` population (behavioral only, no shape change)

`IndentPlan` (`BTreeMap<u32, usize>`) itself is unchanged. What changes is
whether `plan_indentation` inserts a top-level line's entry at all:

- `Normalize`: inserts `0` for every top-level line (008's exact,
  unchanged behavior).
- `Preserve`: inserts nothing for a top-level line — `computed_indent`'s
  existing fallback to `original_indent_width` supplies the line's real
  on-disk column wherever it's later read as a `base` (research.md §1).

## Explicitly unchanged

- `Block`, `Node`, `Diagnostic`, `DiagnosticKind`, `diagnosed_block_openers`
  — untouched, same as `008` left them.
- `plan_block`, `plan_children`, `computed_indent`, `render` — no code
  changes (research.md §1).
- `CasingConvention`, `EncodingFidelity`, `FormatResult` — untouched.

## New (adapter layer): `TopLevelIndentArg` (drut-cli)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TopLevelIndentArg {
    Preserve,
    Normalize,
}
```

Mirrors `CasingArg`'s shape (a plain `clap::ValueEnum`), but is wired with
`default_value_t` on the `Format` subcommand's field — matching
`OutputFormat`'s pattern, not `CasingArg`'s `Option<...>` pattern
(research.md §4). Converted into `voyager_core::TopLevelIndentMode` in
`format_cmd.rs`, the same `impl From<...>` pattern `CasingArg` →
`CasingConvention` already establishes.
