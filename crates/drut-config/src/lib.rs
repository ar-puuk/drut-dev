//! Drut project configuration: `drut.toml` discovery, parsing, and
//! per-field resolution (012-toml-configuration, `contracts/
//! toml-config-api.md`).

use std::path::Path;

pub mod discover;
pub mod parse;

pub use discover::discover;

/// A parsed `drut.toml`'s content (data-model.md).
#[derive(Debug, Clone, Default)]
pub struct DrutConfig {
    pub format: FormatConfig,
}

/// The `[format]` table's known settings. `None` means "not set in this
/// file" — distinct from "set to the default value" (spec.md's own "absent
/// means built-in default" convention).
///
/// `casing` is the legacy, already-shipped setting — unchanged, still
/// applies to `control_words` + `pair_keywords` only (the two categories it
/// already reached before `017-casing-categories-indent-width`). The three
/// `*_casing` fields below are new, independent, granular overrides — see
/// `resolve_format_options`'s own doc comment for the full precedence.
#[derive(Debug, Clone, Default)]
pub struct FormatConfig {
    pub casing: Option<voyager_core::CasingConvention>,
    pub control_words_casing: Option<voyager_core::CasingConvention>,
    pub pair_keywords_casing: Option<voyager_core::CasingConvention>,
    pub data_references_casing: Option<voyager_core::CasingConvention>,
    pub top_level_indent: Option<voyager_core::TopLevelIndentMode>,
    /// Valid range 1–16 is enforced by `resolve_format_options`, not here —
    /// this field carries whatever integer `drut.toml` actually had, valid
    /// or not (data-model.md §4).
    pub indent_width: Option<u8>,
}

/// A non-fatal problem found while parsing a `drut.toml` (spec.md FR-011).
/// Never accompanies a hard error — every path that produces one of these
/// also produces a best-effort fallback `DrutConfig`/`FormatOptions`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigWarning {
    ParseError { path: std::path::PathBuf, message: String },
    UnrecognizedKey {
        path: std::path::PathBuf,
        table: String,
        key: String,
    },
    InvalidValue {
        path: std::path::PathBuf,
        table: String,
        key: String,
        message: String,
    },
}

impl std::fmt::Display for ConfigWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigWarning::ParseError { path, message } => {
                write!(f, "{}: could not parse as TOML: {message}", path.display())
            }
            ConfigWarning::UnrecognizedKey { path, table, key } => {
                write!(f, "{}: unrecognized key `{key}` in [{table}], ignored", path.display())
            }
            ConfigWarning::InvalidValue { path, table, key, message } => {
                write!(f, "{}: invalid value for `{table}.{key}`: {message}", path.display())
            }
        }
    }
}

/// What was explicitly supplied for one call — a CLI flag's value when
/// passed, an MCP parameter's value when supplied. `None` means "consult
/// the resolved config file, then the built-in default" (spec.md FR-006).
#[derive(Debug, Clone, Copy, Default)]
pub struct ExplicitFormatOverride {
    pub casing: Option<voyager_core::CasingConvention>,
    pub control_words_casing: Option<voyager_core::CasingConvention>,
    pub pair_keywords_casing: Option<voyager_core::CasingConvention>,
    pub data_references_casing: Option<voyager_core::CasingConvention>,
    pub top_level_indent: Option<voyager_core::TopLevelIndentMode>,
    pub indent_width: Option<u8>,
}

/// Built-in default indentation width (`FormatOptions::default().indent_width`,
/// 4 spaces per nesting level) — the fallback used whenever no layer
/// supplies a value, or the supplied value is out of the valid range.
const DEFAULT_INDENT_WIDTH: u8 = 4;
/// The valid `indent_width` range (data-model.md §4, `ROADMAP.md` item 9's
/// carried-forward recommendation) — outside this, a value is discarded
/// with a non-blocking warning, the same fallback pattern every other
/// malformed `[format]` value in this crate already uses.
const INDENT_WIDTH_RANGE: std::ops::RangeInclusive<u8> = 1..=16;

/// The one entry point every adapter calls (contracts/toml-config-api.md).
/// Per field, independently: explicit override wins, else the resolved
/// config file's value, else the built-in default. `isolated: true` skips
/// discovery entirely — `file_path` is not even consulted. `file_path:
/// None` (no real on-disk location) also skips discovery, falling straight
/// to explicit-then-default. Never panics on any input.
pub fn resolve_format_options(
    file_path: Option<&Path>,
    isolated: bool,
    explicit: ExplicitFormatOverride,
) -> (voyager_core::FormatOptions, Vec<ConfigWarning>) {
    if isolated {
        return (default_options(explicit, None, &mut Vec::new()), Vec::new());
    }

    let Some(path) = file_path else {
        return (default_options(explicit, None, &mut Vec::new()), Vec::new());
    };

    let discovered = discover(path);
    let (config, mut warnings) = match &discovered {
        Some(config_path) => parse::parse(config_path),
        None => (DrutConfig::default(), Vec::new()),
    };
    let config_path = discovered.as_deref();

    let options = resolve_casing_and_indent(&explicit, &config, config_path, &mut warnings);
    (options, warnings)
}

/// Shared by both the discovered-`drut.toml` path and the isolated/no-file
/// path above — `config` is `DrutConfig::default()` (i.e. "nothing set") in
/// the latter case, which collapses every field to `explicit.or(None)`,
/// exactly reproducing the old two-field behavior for anyone not using any
/// of this feature's new settings.
fn resolve_casing_and_indent(
    explicit: &ExplicitFormatOverride,
    config: &DrutConfig,
    config_path: Option<&Path>,
    warnings: &mut Vec<ConfigWarning>,
) -> voyager_core::FormatOptions {
    // Legacy `casing` (already-shipped) covers control_words + pair_keywords
    // only — it structurally never reached data_references, so it's never
    // part of that category's own fallback chain (data-model.md §3).
    let control_words = explicit
        .control_words_casing
        .or(explicit.casing)
        .or(config.format.control_words_casing)
        .or(config.format.casing)
        .unwrap_or_default();
    let pair_keywords = explicit
        .pair_keywords_casing
        .or(explicit.casing)
        .or(config.format.pair_keywords_casing)
        .or(config.format.casing)
        .unwrap_or_default();
    let data_references = explicit
        .data_references_casing
        .or(config.format.data_references_casing)
        .unwrap_or_default();
    let top_level_indent = explicit
        .top_level_indent
        .or(config.format.top_level_indent)
        .unwrap_or_default();
    let indent_width = resolve_indent_width(explicit.indent_width, config.format.indent_width, config_path, warnings);

    voyager_core::FormatOptions {
        casing: voyager_core::CasingSettings { control_words, pair_keywords, data_references },
        top_level_indent,
        indent_width,
    }
}

/// `explicit` is trusted (CLI/MCP are expected to validate their own
/// `--indent-width`/`indent_width` inputs at their own layer before ever
/// constructing an `ExplicitFormatOverride`) but re-checked here anyway,
/// defensively, since this function never panics on any input by contract
/// — an out-of-range `explicit` value simply falls through to the next
/// tier rather than propagating. An out-of-range `config` value is where
/// this matters in practice: it degrades to the built-in default with a
/// `ConfigWarning`, the same non-blocking pattern every other malformed
/// `[format]` value in this crate already uses (data-model.md §4).
fn resolve_indent_width(
    explicit: Option<u8>,
    config: Option<u8>,
    config_path: Option<&Path>,
    warnings: &mut Vec<ConfigWarning>,
) -> u8 {
    if let Some(value) = explicit {
        if INDENT_WIDTH_RANGE.contains(&value) {
            return value;
        }
    }
    if let Some(value) = config {
        if INDENT_WIDTH_RANGE.contains(&value) {
            return value;
        }
        if let Some(path) = config_path {
            warnings.push(ConfigWarning::InvalidValue {
                path: path.to_path_buf(),
                table: "format".to_string(),
                key: "indent_width".to_string(),
                message: format!(
                    "{value} is outside the valid range {}-{}; using the default ({DEFAULT_INDENT_WIDTH})",
                    INDENT_WIDTH_RANGE.start(),
                    INDENT_WIDTH_RANGE.end()
                ),
            });
        }
    }
    DEFAULT_INDENT_WIDTH
}

fn default_options(
    explicit: ExplicitFormatOverride,
    config_path: Option<&Path>,
    warnings: &mut Vec<ConfigWarning>,
) -> voyager_core::FormatOptions {
    resolve_casing_and_indent(&explicit, &DrutConfig::default(), config_path, warnings)
}
