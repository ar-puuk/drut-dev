# Contract: Operator Spacing Normalization (addition)

Extends `001-voyager-script-parser/contracts/public-api.md` and
`002-cli-check-format/contracts/formatting-api.md`. A conceptual signature contract, not final
Rust source, but the shapes and guarantees below are binding — same convention every prior
contract doc in this repo follows.

## `voyager-core` additions

```text
pub enum OperatorSpacing {
    Preserve,   // #[default]
    Fixed,
    Auto,
}

pub struct FormatOptions {
    pub casing: CasingSettings,
    pub top_level_indent: TopLevelIndentMode,
    pub indent_width: u8,
    pub operator_spacing: OperatorSpacing,   // new
}
```

- **`format`/`format_bytes` signatures are unchanged** — still `fn format(source: &str, options:
  FormatOptions) -> FormatResult`. Only `FormatOptions`'s own shape changes, same as every prior
  additive formatting feature in this project.
- **No panics, determinism, idempotency, behavior preservation**: every guarantee
  `002-cli-check-format/contracts/formatting-api.md` already makes for `format`/`format_bytes`
  holds unchanged, re-verified (not assumed) for `Fixed` and `Auto` specifically.
- **`Preserve` (default) is a true no-op**: a project with no `operator_spacing` configuration
  produces byte-identical output to before this feature existed (FR-009) — confirmed by the full
  existing golden-fixture set and real corpus passing unmodified, the same standard `017`
  already established for its own new axes.
- **`Fixed`'s scope, exactly**: single-space normalization of `=`, `==`, `<>`, `>=`, `<=`, `<`,
  `>`, binary `+`/`-`/`*`/`/`; comma spacing between `Control` pairs; zero interior padding
  inside `[...]`/`(...)`; zero space between a control word and its opening `(`. This is a
  closed set — `^`, `&`, `|` are never touched even though they're lexer delimiter characters
  too. Nothing else — in particular, never touches values inside string/quoted literals or
  comments, never reorders or removes statement content, never crosses into indentation or
  casing's territory.
- **Quoted-literal safety is independently verified, not assumed from `TokenKind`** (FR-010a):
  an operator-shaped character inside an open string literal (e.g. the `+` in `LIST='a+b'`)
  tokenizes identically to a real operator at the `TokenKind` level — confirmed by direct testing
  (research.md §9), not a theoretical concern. Every recognition rule in this module consults its
  own local quote-state tracking before treating anything as an operator/comma/bracket-paren
  occurrence.
- **`Auto` is `Fixed` plus alignment, never a divergent path**: implemented as "run `Fixed`'s
  edit collection first, then compute additional alignment padding" — there is no code path
  where `Auto` produces a *different* base spacing decision than `Fixed` would for the same
  token pair, only additional padding before an `Assignment` statement's `=`.
- **Unary vs. binary `+`/`-`** (FR-003): a `+`/`-` immediately following `=`, `(`, `,`, another
  operator, or nothing (start of the value) is unary and receives no surrounding space; every
  other occurrence is binary and receives exactly one space on each side.
- **Continuation-position operators** (FR-012): when an in-scope operator character is also the
  line's trailing continuation marker, only its leading side is normalized — never a trailing
  space, since no operand follows it on that physical line.
- **Alignment-run boundaries** (FR-007/FR-008): a run is a maximal sequence of consecutive
  `Node::Statement(Assignment)` siblings within one `Vec<Node>` slice (block children, or
  top-level), broken by any non-`Assignment` sibling (including a pair-keyword-shaped `Control`
  statement), a blank source line, a comment-only source line between two siblings, or an
  `Assignment` statement sitting inside a `; FMT: OFF`/`; FMT: ON` protected region (excluded
  from the run entirely — never padded, never counted toward a neighbor's alignment column). A
  run of one member receives no alignment padding — it's already correctly spaced by `Fixed`
  alone.

## `drut-config` additions

```text
pub struct FormatConfig {
    // ...existing fields unchanged...
    pub operator_spacing: Option<OperatorSpacing>,   // new
}
// ExplicitFormatOverride: identical new field, same existing pattern as top_level_indent.

pub fn resolve_format_options(
    config: &FormatConfig,
    explicit: &ExplicitFormatOverride,
) -> FormatOptions
```

- Precedence: explicit CLI flag/MCP param → `drut.toml`'s `operator_spacing` field → built-in
  default `preserve` (data-model.md §4) — single-setting precedence, no legacy-field
  arbitration needed (unlike `017`'s `casing`, this setting has no prior flat equivalent to stay
  compatible with).
- TOML parsing accepts `"preserve"`/`"fixed"`/`"auto"`, case-insensitive (matching `casing`'s
  existing case-insensitivity). An unrecognized value is discarded with a non-blocking notice
  and falls through to the built-in default, identical to every other malformed `[format]`
  field (FR-011).

## `drut-cli` additions

- `--operator-spacing=<preserve|fixed|auto>` — new, same `ValueEnum` shape and "requires an
  explicit value, no bare flag" rule `002-cli-check-format` FR-015 already established for
  `--casing`.

## `drut-mcp` additions

- `operator_spacing` string parameter — new, same accepted-value shape as `casing`.

## What this contract does *not* promise (by design, this phase)

- No configurable interior-padding/control-word-paren axis independent of `Fixed`/`Auto`
  (`ROADMAP.md` item 12's related cases (4)/(5)) — folded into `Fixed`/`Auto` directly, by
  explicit owner decision; `Preserve` remains unaffected either way.
- No comma-spacing axis independent of `Fixed`/`Auto` — same folding decision.
- No alignment of anything other than `Assignment` statements' `=` — pair-keyword `Control`
  lines, comparison operators, and arithmetic expressions are never alignment-run participants,
  even under `Auto`.
- No configurable alignment-run-break behavior — blank line, comment-only line, and
  non-`Assignment` sibling are the fixed, non-configurable break conditions.
