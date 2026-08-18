# Contract: Blank-Line-Run Normalization (addition)

Extends `001-voyager-script-parser/contracts/public-api.md` and
`002-cli-check-format/contracts/formatting-api.md`. A conceptual signature contract, not final
Rust source, but the shapes and guarantees below are binding — same convention every prior
contract doc in this repo follows.

## `voyager-core` additions

```text
pub enum BlankLineMode {
    Preserve,   // #[default]
    Auto,
}

pub struct FormatOptions {
    // ...existing fields unchanged...
    pub blank_lines: BlankLineMode,          // new
    pub top_level_blank_line_cap: u8,        // new, default 2
    pub nested_blank_line_cap: u8,           // new, default 1
}
```

- **`format`/`format_bytes` signatures are unchanged** — still `fn format(source: &str, options:
  FormatOptions) -> FormatResult`. Only `FormatOptions`'s own shape changes, same as every prior
  additive formatting feature in this project.
- **No panics, determinism, idempotency, behavior preservation**: every guarantee
  `002-cli-check-format/contracts/formatting-api.md` already makes for `format`/`format_bytes`
  holds unchanged, re-verified (not assumed) for `Auto` specifically.
- **`Preserve` (default) is a true no-op**: a project with no `blank_lines` configuration
  produces byte-identical output to before this feature existed (FR-009), confirmed by the full
  existing golden-fixture set and real corpus passing unmodified.
- **`Auto`'s scope, exactly**: contracts a run of consecutive blank lines (a whitespace-only line
  counts as blank) down to the applicable cap, *only* when the run's length exceeds that cap —
  never pads a shorter run up, never touches a non-blank line's own content, never touches
  anything inside a `; FMT: OFF`/`; FMT: ON` region. The applicable cap is
  `top_level_blank_line_cap` for a run between top-level statements/blocks, or
  `nested_blank_line_cap` for a run anywhere inside any block's own body, uniformly regardless of
  nesting depth (never a further-reduced cap at deeper levels).
- **Survivors are always the run's own first N lines**, left byte-for-byte as written — only the
  trailing excess lines are removed, no surviving line's content is altered (FR-006).

## `drut-config` additions

```text
pub struct FormatConfig {
    // ...existing fields unchanged...
    pub blank_lines: Option<BlankLineMode>,
    pub top_level_blank_line_cap: Option<u8>,
    pub nested_blank_line_cap: Option<u8>,
}
// ExplicitFormatOverride: identical new fields, same existing pattern as top_level_indent/
// indent_width.

pub fn resolve_format_options(
    config: &FormatConfig,
    explicit: &ExplicitFormatOverride,
) -> FormatOptions
```

- Precedence: explicit CLI flag/MCP param → `drut.toml` field → built-in default, independently
  per setting (data-model.md §3) — no legacy-field arbitration needed (unlike `casing`, none of
  these three settings have a prior flat equivalent to stay compatible with).
- Each cap's TOML parsing accepts a plain integer; out-of-range or wrong-type is discarded with a
  non-blocking notice and falls through to that cap's own built-in default, identical to
  `indent_width`'s existing pattern (FR-011). Exact valid range is a planning-phase detail.
- `blank_lines` TOML parsing accepts `"preserve"`/`"auto"`, matching `top_level_indent`'s
  existing case-sensitivity precedent (confirmed case-sensitive on inspection, `018`'s own
  research.md correction to its design docs).

## `drut-cli` additions

- `--blank-lines=<preserve|auto>` — new, same `ValueEnum` shape and "requires an explicit value,
  no bare flag" rule every other format flag already follows.
- `--top-level-blank-line-cap=<N>` / `--nested-blank-line-cap=<N>` — new, same
  "requires an explicit value, range-validated at the argument-parsing layer" rule
  `--indent-width` already established.

## `drut-mcp` additions

- `blank_lines` string parameter — new, same accepted-value shape as `top_level_indent`.
- `top_level_blank_line_cap` / `nested_blank_line_cap` integer parameters — new, same shape as
  `indent_width`.

## What this contract does *not* promise (by design, this phase)

- No third `fixed`-style mode — only `preserve`/`auto`, since there is exactly one non-preserve
  behavior to name.
- No per-nesting-depth scaling of the nested cap — one nested cap applies uniformly to every
  depth greater than zero.
- No blank-line *insertion* (padding a shorter run up, or inserting a blank line where none
  exists at all, e.g. before/after a block) — this feature is a maximum only, never a minimum,
  and never touches placement, only count.
