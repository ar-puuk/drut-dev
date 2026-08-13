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

/// The `[format]` table's two known settings. `None` means "not set in this
/// file" — distinct from "set to the default value" (spec.md's own "absent
/// means built-in default" convention).
#[derive(Debug, Clone, Default)]
pub struct FormatConfig {
    pub casing: Option<voyager_core::CasingConvention>,
    pub top_level_indent: Option<voyager_core::TopLevelIndentMode>,
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
    pub top_level_indent: Option<voyager_core::TopLevelIndentMode>,
}

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
        return (default_options(explicit), Vec::new());
    }

    let Some(path) = file_path else {
        return (default_options(explicit), Vec::new());
    };

    let (config, warnings) = match discover(path) {
        Some(config_path) => parse::parse(&config_path),
        None => (DrutConfig::default(), Vec::new()),
    };

    let casing = explicit.casing.or(config.format.casing);
    let top_level_indent = explicit
        .top_level_indent
        .or(config.format.top_level_indent)
        .unwrap_or_default();

    (voyager_core::FormatOptions { casing, top_level_indent }, warnings)
}

fn default_options(explicit: ExplicitFormatOverride) -> voyager_core::FormatOptions {
    voyager_core::FormatOptions {
        casing: explicit.casing,
        top_level_indent: explicit.top_level_indent.unwrap_or_default(),
    }
}
