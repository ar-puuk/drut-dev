# Phase 1 Data Model: TOML-Based Configuration

## `drut_config::DrutConfig` (new, `drut-config`)

```rust
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DrutConfig {
    pub format: FormatConfig,
}

#[derive(Debug, Clone, Default)]
pub struct FormatConfig {
    pub casing: Option<voyager_core::CasingConvention>,
    pub top_level_indent: Option<voyager_core::TopLevelIndentMode>,
}
```

`FormatConfig` is **not** produced by a single `#[derive(Deserialize)]` — per
research.md §4, it's built field-by-field from a parsed `toml::Value`, so a bad
`top_level_indent` value doesn't invalidate an otherwise-valid `casing` in the same
file. `None` in either field means "not set in this file" (distinct from "set to the
default value") — the same "absent means built-in default" convention spec.md's
schema sketch already establishes.

| Field | Type | Meaning |
|---|---|---|
| `format.casing` | `Option<CasingConvention>` | `"upper"`/`"lower"` if present and valid; `None` if absent or invalid (with a `ConfigWarning` for the latter). |
| `format.top_level_indent` | `Option<TopLevelIndentMode>` | `"preserve"`/`"normalize"` if present and valid; `None` if absent or invalid. |

## `drut_config::ConfigWarning` (new)

```rust
pub enum ConfigWarning {
    ParseError { path: PathBuf, message: String },
    UnrecognizedKey { path: PathBuf, table: String, key: String },
    InvalidValue { path: PathBuf, table: String, key: String, message: String },
}
```

Never fatal by construction — there is no `ConfigError` variant that aborts
resolution; every path through `drut_config::load`/`resolve_format_options` that
encounters a problem produces a `ConfigWarning` and a fallback value, never a `Result::Err`
that stops the caller (research.md §4, spec.md FR-011). Each adapter renders these
in its own idiom (research.md §6).

## `drut_config::ExplicitFormatOverride` (new)

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct ExplicitFormatOverride {
    pub casing: Option<voyager_core::CasingConvention>,
    pub top_level_indent: Option<voyager_core::TopLevelIndentMode>,
}
```

Captures "what was explicitly supplied for this one call" — a CLI flag's value when
passed, an MCP parameter's value when supplied. `None` in either field means "nothing
explicit for this setting; consult the resolved config file, then the built-in
default" (FR-006's per-field precedence chain).

## `drut_config::resolve_format_options` (new, the one function every adapter calls)

```rust
pub fn resolve_format_options(
    file_path: Option<&Path>,
    isolated: bool,
    explicit: ExplicitFormatOverride,
) -> (voyager_core::FormatOptions, Vec<ConfigWarning>)
```

| Input | Meaning |
|---|---|
| `file_path` | The file being formatted, if it has a real on-disk location — `None` for an unsaved LSP buffer or an MCP call with only inline `text` (research.md §5, §8). |
| `isolated` | `true` skips discovery entirely (FR-008) — resolution proceeds straight to `explicit` → built-in default, `file_path` is not even consulted. |
| `explicit` | Per-call override (FR-006). |

**Resolution algorithm** (per field, independently — FR-006):
1. If `explicit`'s field is `Some`, use it. Done.
2. Else, if not `isolated` and a config file was discovered from `file_path`
   (research.md §7's walk-up) and parses (at least partially) with that field
   present and valid, use the file's value.
3. Else, use `voyager_core::FormatOptions::default()`'s value for that field
   (`None`/`Preserve`).

`Vec<ConfigWarning>` is empty in the overwhelmingly common case (no `drut.toml`
found, or one found and entirely valid) — populated only when something was actually
wrong (spec.md SC-005).

## Derivation flow (no new intermediate storage beyond `ServerState.workspace_root`)

```
file_path: Option<&Path>  (CLI: MatchedFile.path; LSP: uri_to_path(request URI),
                            falling back to ServerState.workspace_root; MCP:
                            ScriptSource.path, if set)
   │
   ├─▶ isolated? ──yes──▶ skip discovery entirely
   │        │no
   │        ▼
   ├─▶ drut_config::discover(file_path's directory)
   │        │  walk upward: drut.toml found? / .git boundary? / fs root?
   │        ▼
   │   Option<PathBuf>  (the resolved drut.toml, if any)
   │        │
   │        ▼
   ├─▶ drut_config::parse(path) ──▶ (FormatConfig, Vec<ConfigWarning>)
   │                                  (toml::Value-level, per-field fallback)
   │
   └─▶ merge with `explicit` (per field: explicit > file > default)
            │
            ▼
   (voyager_core::FormatOptions, Vec<ConfigWarning>)
            │
            ▼
   voyager_core::format(text, options)   ← unchanged from today
```

Every step before the final `voyager_core::format` call is new, lives in
`drut-config` (plus the small LSP-only `uri_to_path`/`workspace_root` pieces that stay
in `drut-lsp`, research.md §5), and produces a value `voyager-core` already knows how
to consume — no change to `voyager-core`'s own entry points.
