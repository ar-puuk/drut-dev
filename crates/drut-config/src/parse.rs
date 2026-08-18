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
            // The legacy flat `casing` key was removed (superseded by the
            // three granular fields below) -- it now falls through to the
            // `other` catch-all arm below like any other unrecognized key.
            "casing_control_words" => {
                format.casing_control_words = parse_casing(path, "casing_control_words", value, warnings)
            }
            "casing_pair_keywords" => {
                format.casing_pair_keywords = parse_casing(path, "casing_pair_keywords", value, warnings)
            }
            "casing_data_references" => {
                format.casing_data_references = parse_casing(path, "casing_data_references", value, warnings)
            }
            "indent_top_level" => format.indent_top_level = parse_indent_top_level(path, value, warnings),
            "indent_width" => format.indent_width = parse_indent_width(path, value, warnings),
            "operator_spacing" => format.operator_spacing = parse_operator_spacing(path, value, warnings),
            "blank_lines" => format.blank_lines = parse_blank_lines(path, value, warnings),
            "blank_lines_top_cap" => {
                format.blank_lines_top_cap = parse_blank_line_cap(path, "blank_lines_top_cap", value, warnings)
            }
            "blank_lines_nested_cap" => {
                format.blank_lines_nested_cap = parse_blank_line_cap(path, "blank_lines_nested_cap", value, warnings)
            }
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

/// Shared by the three granular `*_casing` fields
/// (`017-casing-categories-indent-width`) — identical accepted-value shape
/// (`"preserve"`/`"upper"`/`"lower"`) at every one of them; `"auto"` (or any
/// other string) is deliberately just another unrecognized value here, not
/// a special case — this feature ships no built-in preset (FR-003), so
/// there is nothing for a fourth named value to mean.
fn parse_casing(
    path: &Path,
    key: &str,
    value: &toml::Value,
    warnings: &mut Vec<ConfigWarning>,
) -> Option<voyager_core::CasingConvention> {
    match value.as_str() {
        Some("preserve") => Some(voyager_core::CasingConvention::Preserve),
        Some("upper") => Some(voyager_core::CasingConvention::Upper),
        Some("lower") => Some(voyager_core::CasingConvention::Lower),
        Some(other) => {
            warnings.push(invalid_value(
                path,
                key,
                format!("must be \"preserve\", \"upper\", or \"lower\", got {other:?}"),
            ));
            None
        }
        None => {
            warnings.push(invalid_value(
                path,
                key,
                format!("must be a string (\"preserve\", \"upper\", or \"lower\"), got {value:?}"),
            ));
            None
        }
    }
}

/// `017-casing-categories-indent-width`. Accepts any TOML integer here —
/// the 1–16 valid-range bound is enforced later, at
/// `resolve_format_options`'s resolve layer (data-model.md §4), not during
/// parsing; a value here that's merely the wrong *type* (not an integer at
/// all) is still a parse-level `InvalidValue`, same as every other field.
fn parse_indent_width(path: &Path, value: &toml::Value, warnings: &mut Vec<ConfigWarning>) -> Option<u8> {
    match value.as_integer() {
        Some(n) => match u8::try_from(n) {
            Ok(width) => Some(width),
            Err(_) => {
                warnings.push(invalid_value(
                    path,
                    "indent_width",
                    format!("must be an integer between 1 and 16, got {n}"),
                ));
                None
            }
        },
        None => {
            warnings.push(invalid_value(
                path,
                "indent_width",
                format!("must be an integer between 1 and 16, got {value:?}"),
            ));
            None
        }
    }
}

/// `018-operator-spacing`. Same exact-lowercase-string shape `casing`/
/// `indent_top_level` already use (both are, on inspection, case-sensitive
/// today — an exact match against "preserve"/"upper"/"lower" and
/// "preserve"/"auto" respectively, not the case-insensitive behavior this
/// feature's own design docs assumed of them; matched here for consistency
/// with the real existing behavior rather than introducing a one-off
/// case-insensitive exception).
fn parse_operator_spacing(
    path: &Path,
    value: &toml::Value,
    warnings: &mut Vec<ConfigWarning>,
) -> Option<voyager_core::OperatorSpacing> {
    match value.as_str() {
        Some("preserve") => Some(voyager_core::OperatorSpacing::Preserve),
        Some("fixed") => Some(voyager_core::OperatorSpacing::Fixed),
        Some("auto") => Some(voyager_core::OperatorSpacing::Auto),
        Some(other) => {
            warnings.push(invalid_value(
                path,
                "operator_spacing",
                format!("must be \"preserve\", \"fixed\", or \"auto\", got {other:?}"),
            ));
            None
        }
        None => {
            warnings.push(invalid_value(
                path,
                "operator_spacing",
                format!("must be a string (\"preserve\", \"fixed\", or \"auto\"), got {value:?}"),
            ));
            None
        }
    }
}

/// `019-blank-line-normalization`. Same exact-lowercase-string shape
/// `operator_spacing`/`indent_top_level` already use — case-sensitive, only
/// `"preserve"`/`"auto"` recognized (there is exactly one non-preserve
/// behavior here, so no third named value).
fn parse_blank_lines(
    path: &Path,
    value: &toml::Value,
    warnings: &mut Vec<ConfigWarning>,
) -> Option<voyager_core::BlankLineMode> {
    match value.as_str() {
        Some("preserve") => Some(voyager_core::BlankLineMode::Preserve),
        Some("auto") => Some(voyager_core::BlankLineMode::Auto),
        Some(other) => {
            warnings.push(invalid_value(
                path,
                "blank_lines",
                format!("must be \"preserve\" or \"auto\", got {other:?}"),
            ));
            None
        }
        None => {
            warnings.push(invalid_value(
                path,
                "blank_lines",
                format!("must be a string (\"preserve\" or \"auto\"), got {value:?}"),
            ));
            None
        }
    }
}

/// Shared by `blank_lines_top_cap`/`blank_lines_nested_cap`
/// (019-blank-line-normalization) — accepts any TOML integer here; the
/// valid-range bound is enforced later, at `resolve_format_options`'s
/// resolve layer (data-model.md §3), not during parsing, the same
/// `indent_width` precedent.
fn parse_blank_line_cap(path: &Path, key: &str, value: &toml::Value, warnings: &mut Vec<ConfigWarning>) -> Option<u8> {
    match value.as_integer() {
        Some(n) => match u8::try_from(n) {
            Ok(cap) => Some(cap),
            Err(_) => {
                warnings.push(invalid_value(path, key, format!("must be a positive integer, got {n}")));
                None
            }
        },
        None => {
            warnings.push(invalid_value(path, key, format!("must be a positive integer, got {value:?}")));
            None
        }
    }
}

fn parse_indent_top_level(
    path: &Path,
    value: &toml::Value,
    warnings: &mut Vec<ConfigWarning>,
) -> Option<voyager_core::IndentTopLevelMode> {
    match value.as_str() {
        Some("preserve") => Some(voyager_core::IndentTopLevelMode::Preserve),
        // Renamed from "normalize" for `preserve`/`auto` naming consistency
        // with `operator_spacing`/`blank_lines` above -- old drut.toml files
        // written with "normalize" no longer parse; they fall through to
        // the `other` arm below like any other invalid value (warns, falls
        // back to the built-in default) rather than being silently accepted
        // under two names.
        Some("auto") => Some(voyager_core::IndentTopLevelMode::Auto),
        Some(other) => {
            warnings.push(invalid_value(
                path,
                "indent_top_level",
                format!("must be \"preserve\" or \"auto\", got {other:?}"),
            ));
            None
        }
        None => {
            warnings.push(invalid_value(
                path,
                "indent_top_level",
                format!("must be a string (\"preserve\" or \"auto\"), got {value:?}"),
            ));
            None
        }
    }
}
