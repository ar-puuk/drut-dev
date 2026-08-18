# drut-config

`drut.toml` project configuration for [Drut](https://github.com/ar-puuk/drut-dev):
discovery (walking up from a file being processed to find the nearest
`drut.toml`), parsing, and resolution against explicit CLI flags or MCP
parameters.

## What it resolves

A `drut.toml` file's `[format]` table (`casing_control_words`,
`indent_top_level`, and the rest) is
merged with any explicit override the caller provides — an explicit flag or
parameter always wins over the config file. A malformed value in one field
warns (via `ConfigWarning`) and falls back to that field's built-in default
rather than failing the whole run.

```rust
pub struct DrutConfig { /* ... */ }
pub struct ExplicitFormatOverride { /* ... */ }

pub fn resolve_format_options(/* ... */) -> (FormatConfig, Vec<ConfigWarning>);
```

Depends only on [`voyager-core`](https://crates.io/crates/voyager-core) and
`toml` — `drut.toml` is parsed field-by-field against `toml::Value` rather
than via a derived `Deserialize`, so one malformed field never invalidates
the rest of the file.

## Part of the Drut workspace

Used by the `drut` CLI, the Language Server, and the MCP server so that all
three surfaces resolve the exact same configuration the exact same way. See
the [main repository](https://github.com/ar-puuk/drut-dev) for the full
toolchain.

Licensed under either of [Apache License, Version 2.0](https://github.com/ar-puuk/drut-dev/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/ar-puuk/drut-dev/blob/main/LICENSE-MIT) at your option.
