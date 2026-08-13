//! `toml::Table`-level parsing with per-field fallback
//! (012-toml-configuration/research.md §4) — a full, real TOML-spec
//! document parse (never a hand-rolled subset), walked by hand so one bad
//! key never invalidates every other valid key in the same file.
//!
//! Parses into `toml::Table`, not `toml::Value` — `Value`'s own `FromStr`
//! parses a single TOML *value* expression (e.g. `[1, 2, 3]`), not a
//! multi-line document with `[table]` headers; `Table::from_str` is the
//! real top-level-document parser (confirmed directly against the toml
//! crate's own source, `table.rs`'s `FromStr` impl delegating to
//! `crate::from_str`).

use std::path::Path;

use crate::{ConfigWarning, DrutConfig, FormatConfig};

/// Parse `path`'s content. Never returns a hard error for a content
/// problem (I/O failure, invalid TOML syntax, an unrecognized key, an
/// invalid value) — every such problem becomes a `ConfigWarning` alongside
/// a best-effort `DrutConfig` (fields that couldn't be resolved are
/// `None`). A file that fails to parse as TOML at all produces one
/// `ConfigWarning::ParseError` and an empty `FormatConfig`.
pub fn parse(path: &Path) -> (DrutConfig, Vec<ConfigWarning>) {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            return (
                DrutConfig::default(),
                vec![parse_error(path, err.to_string())],
            );
        }
    };

    let table: toml::Table = match content.parse() {
        Ok(table) => table,
        Err(err) => {
            return (
                DrutConfig::default(),
                vec![parse_error(path, err.to_string())],
            );
        }
    };

    let mut warnings = Vec::new();
    let format = table
        .get("format")
        .and_then(toml::Value::as_table)
        .map(|table| parse_format_table(path, table, &mut warnings))
        .unwrap_or_default();
    // Any other top-level table (e.g. a hypothetical future `[lint]`) is
    // silently ignored, not warned — forward-compatibility; a whole extra
    // bracketed section is a much less plausible accidental typo than one
    // key inside a table already in active use (research.md §4).

    (DrutConfig { format }, warnings)
}

fn parse_error(path: &Path, message: String) -> ConfigWarning {
    ConfigWarning::ParseError {
        path: path.to_path_buf(),
        message,
    }
}

fn parse_format_table(path: &Path, table: &toml::Table, warnings: &mut Vec<ConfigWarning>) -> FormatConfig {
    let mut format = FormatConfig::default();

    for (key, value) in table {
        match key.as_str() {
            "casing" => format.casing = parse_casing(path, value, warnings),
            "top_level_indent" => format.top_level_indent = parse_top_level_indent(path, value, warnings),
            other => warnings.push(ConfigWarning::UnrecognizedKey {
                path: path.to_path_buf(),
                table: "format".to_string(),
                key: other.to_string(),
            }),
        }
    }

    format
}

fn invalid_value(path: &Path, key: &str, message: String) -> ConfigWarning {
    ConfigWarning::InvalidValue {
        path: path.to_path_buf(),
        table: "format".to_string(),
        key: key.to_string(),
        message,
    }
}

fn parse_casing(
    path: &Path,
    value: &toml::Value,
    warnings: &mut Vec<ConfigWarning>,
) -> Option<voyager_core::CasingConvention> {
    match value.as_str() {
        Some("upper") => Some(voyager_core::CasingConvention::Upper),
        Some("lower") => Some(voyager_core::CasingConvention::Lower),
        Some(other) => {
            warnings.push(invalid_value(
                path,
                "casing",
                format!("must be \"upper\" or \"lower\", got {other:?}"),
            ));
            None
        }
        None => {
            warnings.push(invalid_value(
                path,
                "casing",
                format!("must be a string (\"upper\" or \"lower\"), got {value:?}"),
            ));
            None
        }
    }
}

fn parse_top_level_indent(
    path: &Path,
    value: &toml::Value,
    warnings: &mut Vec<ConfigWarning>,
) -> Option<voyager_core::TopLevelIndentMode> {
    match value.as_str() {
        Some("preserve") => Some(voyager_core::TopLevelIndentMode::Preserve),
        Some("normalize") => Some(voyager_core::TopLevelIndentMode::Normalize),
        Some(other) => {
            warnings.push(invalid_value(
                path,
                "top_level_indent",
                format!("must be \"preserve\" or \"normalize\", got {other:?}"),
            ));
            None
        }
        None => {
            warnings.push(invalid_value(
                path,
                "top_level_indent",
                format!("must be a string (\"preserve\" or \"normalize\"), got {value:?}"),
            ));
            None
        }
    }
}
