# Phase 1 Data Model: Casing Gains an Explicit `Preserve` Mode

## Changed: `CasingConvention`

```rust
/// The three supported keyword-casing targets (spec.md FR-001). `Preserve`
/// is the `#[default]` -- format always either preserves, uppercases, or
/// lowercases keyword/control-word casing, the same non-optional shape
/// TopLevelIndentMode already uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CasingConvention {
    /// Leave existing control-word/pair-keyword casing exactly as written
    /// -- the previous `FormatOptions.casing == None` behavior, now a real
    /// named variant instead of an absent value (research.md §4).
    #[default]
    Preserve,
    Upper,
    Lower,
}
```

Lives in `crates/voyager-core/src/format.rs`, re-exported from `lib.rs`
alongside `TopLevelIndentMode` — no change to the re-export itself.

## Changed: `FormatOptions`

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct FormatOptions {
    /// CHANGED. No longer Option-wrapped -- defaults to `Preserve` via
    /// `CasingConvention`'s own `#[default]`, the same non-optional shape
    /// `top_level_indent` already has on this same struct. Every call site
    /// is still individually verified (research.md §2), not trusted
    /// transitively from the derive alone.
    pub casing: CasingConvention,
    pub top_level_indent: TopLevelIndentMode,
}
```

Both fields remain independent — this feature touches only `casing`'s
type; `top_level_indent` is untouched.

## Changed: `render`'s casing-edit gate (behavioral shape only, no output change)

```rust
// Before:
if let Some(convention) = options.casing {
    collect_casing_edits(nodes, &char_lines, &protected, convention, &mut casing_edits);
}

// After:
if options.casing != CasingConvention::Preserve {
    collect_casing_edits(nodes, &char_lines, &protected, options.casing, &mut casing_edits);
}
```

`options.casing != CasingConvention::Preserve` is `true` in exactly the
cases `options.casing.is_some()` was `true` before (FR-003, research.md
§4) — `Preserve` is the only new state, and it maps exactly onto the old
`None`. Everything downstream of this gate (`collect_casing_edits` and
its callees) already takes a bare `CasingConvention`, unchanged.

## Changed: `edit_for_span`'s match (compile-exhaustiveness only, practically unreachable)

```rust
let replacement = match convention {
    CasingConvention::Upper => original.to_ascii_uppercase(),
    CasingConvention::Lower => original.to_ascii_lowercase(),
    // NEW -- required for exhaustiveness; never actually reached, since
    // render()'s guard above means this function is never called with
    // Preserve in practice (research.md §1).
    CasingConvention::Preserve => original.clone(),
};
```

`original.clone()` makes this arm a no-op by construction — the existing
`if replacement == original { return None; }` check two lines below
already turns a same-as-original replacement into "no edit," so this arm
needs no special-casing beyond satisfying the match.

## Unchanged: `drut_config::FormatConfig`/`ExplicitFormatOverride`

```rust
pub struct FormatConfig {
    pub casing: Option<voyager_core::CasingConvention>,       // unchanged
    pub top_level_indent: Option<voyager_core::TopLevelIndentMode>,
}

pub struct ExplicitFormatOverride {
    pub casing: Option<voyager_core::CasingConvention>,       // unchanged
    pub top_level_indent: Option<voyager_core::TopLevelIndentMode>,
}
```

`Option` here means "this layer (a `drut.toml` file, or an explicit
CLI-flag/MCP-param) stated no casing preference" — a distinct concept from
`CasingConvention`'s own `Preserve` variant, per spec.md FR-004. Both
structs are otherwise untouched.

## Changed: `resolve_format_options`'s two `casing` lines

```rust
// resolve_format_options, before:
let casing = explicit.casing.or(config.format.casing);

// after (matches top_level_indent's existing line 96-99 shape exactly):
let casing = explicit.casing.or(config.format.casing).unwrap_or_default();

// default_options, before:
casing: explicit.casing,

// after:
casing: explicit.casing.unwrap_or_default(),
```

## Changed (adapter layer): `CasingArg` (drut-cli)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CasingArg {
    Preserve,   // NEW
    Upper,
    Lower,
}
```

Stays `Option<CasingArg>`-wrapped on the `Format` subcommand (unchanged —
`012-toml-configuration` already made this `Option`, for the same
flag-omitted-vs-explicit-value distinction `top_level_indent` needs against
a `drut.toml` layer). `impl From<CasingArg> for CasingConvention` gains a
`CasingArg::Preserve => CasingConvention::Preserve` arm.

## Changed (adapter layer): TOML/MCP string values

Both `drut-config/src/parse.rs`'s `parse_casing` and `drut-mcp/src/
format.rs`'s `explicit_override` casing match gain a
`Some("preserve") => Some(CasingConvention::Preserve)` arm, alongside the
existing `"upper"`/`"lower"` arms — their error messages updated to name
all three valid values.

## Explicitly unchanged

- `Block`, `Node`, `Diagnostic`, `DiagnosticKind`, `IndentPlan`,
  `TopLevelIndentMode`, `EncodingFidelity`, `FormatResult` — untouched.
- `plan_indentation`, `plan_block`, `plan_children`, `computed_indent` —
  no code changes; this feature touches only casing, not indentation.
- `drut-lsp` — no source changes at all (research.md §2).
