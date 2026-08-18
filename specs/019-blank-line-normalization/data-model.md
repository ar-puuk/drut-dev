# Data Model: Blank-Line-Run Normalization

## §1. `voyager-core` types

### `BlankLineMode` (new)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlankLineMode {
    #[default]
    Preserve,
    Auto,
}
```

- Two-valued, matching `TopLevelIndentMode`'s own shape exactly (not `OperatorSpacing`'s
  three-valued one) — there is only one real non-`Preserve` behavior here (contract to the
  applicable cap), so no third tier.

### `FormatOptions` (modified)

```rust
pub struct FormatOptions {
    pub casing: CasingSettings,
    pub top_level_indent: TopLevelIndentMode,
    pub indent_width: u8,
    pub operator_spacing: OperatorSpacing,
    pub blank_lines: BlankLineMode,               // new
    pub top_level_blank_line_cap: u8,              // new, default 2
    pub nested_blank_line_cap: u8,                 // new, default 1
}
```

- Both caps are bare `u8`s here, unvalidated — `voyager-core` accepts whatever its caller
  passes, the same `indent_width` precedent (the 1–N valid-range bound is a `drut-config`-layer
  policy decision, not a fact this crate enforces).

### `blank_line` module (new)

```rust
/// Every line number (1-based) that falls strictly inside some top-level
/// block's own span (research.md §4 — top-level blocks only, no recursion
/// needed; a nested block's span is always contained within its parent's).
fn nested_lines(nodes: &[Node]) -> BTreeSet<u32>;

/// A maximal run of consecutive blank lines (research.md §2's whitespace
/// convention) — `is_nested`/`is_protected` are uniform across the whole
/// run (research.md §3), so each is a single bool, not per-line.
struct BlankRun {
    first_line: u32,
    len: u32,
    is_nested: bool,
    is_protected: bool,
}

fn find_blank_runs(lines: &[Vec<char>], nested: &BTreeSet<u32>, protected: &BTreeSet<u32>) -> Vec<BlankRun>;

/// For each run whose length exceeds the applicable cap (and that isn't
/// protected), the line numbers to delete — the run's own trailing
/// `len - cap` lines (research.md §5: survivors are always the first N).
pub(crate) fn lines_to_delete(
    nodes: &[Node],
    lines: &[Vec<char>],
    protected: &BTreeSet<u32>,
    top_level_cap: u8,
    nested_cap: u8,
) -> BTreeSet<u32>;
```

- Pure functions over already-parsed `Node`/line data, no I/O, never panics — same contract
  shape every other `voyager-core` public/internal function already has.
- Lives in its own module (`src/blank_line.rs`), mirroring `data_reference.rs`/
  `operator_spacing.rs`'s established self-contained-module pattern.

## §2: `render()` integration (research.md §1)

```rust
let mut lines_to_delete: BTreeSet<u32> = BTreeSet::new();
if options.blank_lines != BlankLineMode::Preserve {
    lines_to_delete = blank_line::lines_to_delete(
        nodes, &char_lines, &protected,
        options.top_level_blank_line_cap, options.nested_blank_line_cap,
    );
}
```

The main emission loop gains one early-exit check per iteration, before any other per-line
processing: `if lines_to_delete.contains(&line_num) { continue; }` — every other computation
already run against this line's original number (indentation plan lookups, casing/spacing edits
for this line) simply never executes for a deleted line, since the line is never emitted.
Short-circuited identically to every other axis when `options.blank_lines ==
BlankLineMode::Preserve` — a `Preserve`-configured or unconfigured call does exactly the same
work as before this feature existed (FR-009/SC-003).

## §3. Configuration precedence

Two new caps plus one new mode, same shape `top_level_indent` already has (one setting, no
legacy-field arbitration needed):

| Setting | Precedence (highest wins) |
|---|---|
| `blank_lines` (mode) | explicit CLI flag/MCP param → `drut.toml` field → built-in default `preserve` |
| `top_level_blank_line_cap` | explicit → `drut.toml` (validated against a sane range, else discarded with a notice) → built-in default `2` |
| `nested_blank_line_cap` | explicit → `drut.toml` (validated against a sane range, else discarded with a notice) → built-in default `1` |

### `drut_config::FormatConfig` / `ExplicitFormatOverride` (modified)

```rust
pub struct FormatConfig {
    // ...existing fields unchanged...
    pub blank_lines: Option<BlankLineMode>,
    pub top_level_blank_line_cap: Option<u8>,
    pub nested_blank_line_cap: Option<u8>,
}
// ExplicitFormatOverride mirrors the same shape.
```

Each cap's valid range and fallback behavior mirrors `indent_width`'s existing
`resolve_indent_width` pattern exactly — out-of-range or malformed degrades to that cap's own
built-in default with a `ConfigWarning::InvalidValue`, never a hard failure (FR-011).
