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
    /// `018-operator-spacing`. Single new setting, no legacy field to stay
    /// compatible with (unlike `casing`) — precedence is just `explicit >
    /// this field > built-in default` (data-model.md §4).
    pub operator_spacing: Option<voyager_core::OperatorSpacing>,
    /// `019-blank-line-normalization`. Single new setting, no legacy field —
    /// precedence is `explicit > this field > built-in default (preserve)`,
    /// same shape as `operator_spacing` above.
    pub blank_lines: Option<voyager_core::BlankLineMode>,
    /// Valid range is enforced by `resolve_format_options`, not here — this
    /// field carries whatever integer `drut.toml` actually had, valid or
    /// not (data-model.md §3), mirroring `indent_width`'s own precedent.
    pub top_level_blank_line_cap: Option<u8>,
    /// Same range-validated-with-fallback treatment as
    /// `top_level_blank_line_cap` above, independently.
    pub nested_blank_line_cap: Option<u8>,
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
    pub operator_spacing: Option<voyager_core::OperatorSpacing>,
    pub blank_lines: Option<voyager_core::BlankLineMode>,
    pub top_level_blank_line_cap: Option<u8>,
    pub nested_blank_line_cap: Option<u8>,
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

/// Built-in default caps (`FormatOptions::default()`'s own
/// `top_level_blank_line_cap`/`nested_blank_line_cap`, spec.md FR-002) —
/// the fallback used whenever no layer supplies a value, or the supplied
/// value is out of the valid range.
const DEFAULT_TOP_LEVEL_BLANK_LINE_CAP: u8 = 2;
const DEFAULT_NESTED_BLANK_LINE_CAP: u8 = 1;
/// The valid range for each blank-line cap (spec.md Assumptions: "a sane
/// upper bound, not unlimited" is a planning-phase detail, not fixed by the
/// spec) — both caps are "positive-integer" per spec.md's own framing, so
/// `0` is out of range same as any other malformed value; `50` is a
/// generous sane ceiling no real project's own style would plausibly
/// exceed, mirroring `INDENT_WIDTH_RANGE`'s own precedent shape.
const BLANK_LINE_CAP_RANGE: std::ops::RangeInclusive<u8> = 1..=50;

/// The one entry point every adapter calls (contracts/toml-config-api.md).
/// Per field, independently: explicit override wins, else the resolved
/// config file's value, else `client_defaults`'s value (021-editor-settings-
/// config — an editor-level personal-preference fallback, reachable only
/// through `drut-lsp`; every CLI/MCP call site always passes
/// `ExplicitFormatOverride::default()` for this parameter, so those two
/// surfaces are completely unaffected), else the built-in default.
/// `isolated: true` skips discovery entirely — `file_path` is not even
/// consulted (this only zeroes out the `drut.toml` tier; `explicit` and
/// `client_defaults` still apply). `file_path: None` (no real on-disk
/// location) also skips discovery, falling straight to
/// explicit-then-client_defaults-then-default. Never panics on any input.
pub fn resolve_format_options(
    file_path: Option<&Path>,
    isolated: bool,
    explicit: ExplicitFormatOverride,
    client_defaults: ExplicitFormatOverride,
) -> (voyager_core::FormatOptions, Vec<ConfigWarning>) {
    if isolated {
        return (default_options(explicit, client_defaults, None, &mut Vec::new()), Vec::new());
    }

    let Some(path) = file_path else {
        return (default_options(explicit, client_defaults, None, &mut Vec::new()), Vec::new());
    };

    let discovered = discover(path);
    let (config, mut warnings) = match &discovered {
        Some(config_path) => parse::parse(config_path),
        None => (DrutConfig::default(), Vec::new()),
    };
    let config_path = discovered.as_deref();

    let options = resolve_casing_and_indent(&explicit, &config, &client_defaults, config_path, &mut warnings);
    (options, warnings)
}

/// Shared by both the discovered-`drut.toml` path and the isolated/no-file
/// path above — `config` is `DrutConfig::default()` (i.e. "nothing set") in
/// the latter case, which collapses every field to
/// `explicit.or(None).or(client_defaults)`, exactly reproducing the old
/// two-field behavior for anyone not using any of this feature's new
/// settings (and `client_defaults` itself stays `Default` — all `None` — for
/// every CLI/MCP call site, per `resolve_format_options`'s own doc comment).
///
/// `client_defaults` (021-editor-settings-config, contracts/
/// editor-settings-config.md) is consulted only after both `explicit` and
/// `config` (`drut.toml`) have had a chance to set a field — a lower-
/// precedence-but-one tier, never before either. `control_words`/
/// `pair_keywords` are the two fields with a legacy-`casing`-then-granular
/// fallback *within* each tier already (research.md §1's checklist-review
/// correction) — `client_defaults` gets the identical two-step treatment
/// (`.or(client_defaults.control_words_casing).or(client_defaults.casing)`),
/// not just one more `.or()` like every other field.
fn resolve_casing_and_indent(
    explicit: &ExplicitFormatOverride,
    config: &DrutConfig,
    client_defaults: &ExplicitFormatOverride,
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
        .or(client_defaults.control_words_casing)
        .or(client_defaults.casing)
        .unwrap_or_default();
    let pair_keywords = explicit
        .pair_keywords_casing
        .or(explicit.casing)
        .or(config.format.pair_keywords_casing)
        .or(config.format.casing)
        .or(client_defaults.pair_keywords_casing)
        .or(client_defaults.casing)
        .unwrap_or_default();
    let data_references = explicit
        .data_references_casing
        .or(config.format.data_references_casing)
        .or(client_defaults.data_references_casing)
        .unwrap_or_default();
    let top_level_indent = explicit
        .top_level_indent
        .or(config.format.top_level_indent)
        .or(client_defaults.top_level_indent)
        .unwrap_or_default();
    let indent_width = resolve_indent_width(
        explicit.indent_width,
        config.format.indent_width,
        client_defaults.indent_width,
        config_path,
        warnings,
    );
    let operator_spacing = explicit
        .operator_spacing
        .or(config.format.operator_spacing)
        .or(client_defaults.operator_spacing)
        .unwrap_or_default();
    let blank_lines = explicit
        .blank_lines
        .or(config.format.blank_lines)
        .or(client_defaults.blank_lines)
        .unwrap_or_default();
    let top_level_blank_line_cap = resolve_blank_line_cap(
        explicit.top_level_blank_line_cap,
        config.format.top_level_blank_line_cap,
        client_defaults.top_level_blank_line_cap,
        "top_level_blank_line_cap",
        DEFAULT_TOP_LEVEL_BLANK_LINE_CAP,
        config_path,
        warnings,
    );
    let nested_blank_line_cap = resolve_blank_line_cap(
        explicit.nested_blank_line_cap,
        config.format.nested_blank_line_cap,
        client_defaults.nested_blank_line_cap,
        "nested_blank_line_cap",
        DEFAULT_NESTED_BLANK_LINE_CAP,
        config_path,
        warnings,
    );

    voyager_core::FormatOptions {
        casing: voyager_core::CasingSettings { control_words, pair_keywords, data_references },
        top_level_indent,
        indent_width,
        operator_spacing,
        blank_lines,
        top_level_blank_line_cap,
        nested_blank_line_cap,
    }
}

/// Not a real on-disk file — a client (editor) setting has no filesystem
/// location of its own (021-editor-settings-config, spec.md FR-005). This
/// sentinel exists solely so an out-of-range `client_defaults` numeric
/// value can still produce a `ConfigWarning::InvalidValue` (which requires
/// a `path` to render) — the same non-blocking-degrade-to-default outcome
/// an out-of-range `drut.toml` value already gets, just attributed to a
/// distinguishable, non-filesystem source rather than a real config file.
fn client_setting_pseudo_path() -> std::path::PathBuf {
    std::path::PathBuf::from("<client setting>")
}

/// `explicit` is trusted (CLI/MCP are expected to validate their own
/// `--indent-width`/`indent_width` inputs at their own layer before ever
/// constructing an `ExplicitFormatOverride`) but re-checked here anyway,
/// defensively, since this function never panics on any input by contract
/// — an out-of-range `explicit` value simply falls through to the next
/// tier rather than propagating. An out-of-range `config` (`drut.toml`) or
/// `client` (021-editor-settings-config) value is where this matters in
/// practice: either degrades to the built-in default with a
/// `ConfigWarning`, the same non-blocking pattern every other malformed
/// `[format]` value in this crate already uses (data-model.md §4).
fn resolve_indent_width(
    explicit: Option<u8>,
    config: Option<u8>,
    client: Option<u8>,
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
    if let Some(value) = client {
        if INDENT_WIDTH_RANGE.contains(&value) {
            return value;
        }
        warnings.push(ConfigWarning::InvalidValue {
            path: client_setting_pseudo_path(),
            table: "format".to_string(),
            key: "indent_width".to_string(),
            message: format!(
                "{value} is outside the valid range {}-{}; using the default ({DEFAULT_INDENT_WIDTH})",
                INDENT_WIDTH_RANGE.start(),
                INDENT_WIDTH_RANGE.end()
            ),
        });
    }
    DEFAULT_INDENT_WIDTH
}

/// Shared by `top_level_blank_line_cap` and `nested_blank_line_cap`
/// (019-blank-line-normalization) — identical range-validated-with-fallback
/// shape as `resolve_indent_width` above, parameterized by `key`/`default`
/// since there are two independent caps rather than one.
#[allow(clippy::too_many_arguments)]
fn resolve_blank_line_cap(
    explicit: Option<u8>,
    config: Option<u8>,
    client: Option<u8>,
    key: &str,
    default: u8,
    config_path: Option<&Path>,
    warnings: &mut Vec<ConfigWarning>,
) -> u8 {
    if let Some(value) = explicit {
        if BLANK_LINE_CAP_RANGE.contains(&value) {
            return value;
        }
    }
    if let Some(value) = config {
        if BLANK_LINE_CAP_RANGE.contains(&value) {
            return value;
        }
        if let Some(path) = config_path {
            warnings.push(ConfigWarning::InvalidValue {
                path: path.to_path_buf(),
                table: "format".to_string(),
                key: key.to_string(),
                message: format!(
                    "{value} is outside the valid range {}-{}; using the default ({default})",
                    BLANK_LINE_CAP_RANGE.start(),
                    BLANK_LINE_CAP_RANGE.end()
                ),
            });
        }
    }
    if let Some(value) = client {
        if BLANK_LINE_CAP_RANGE.contains(&value) {
            return value;
        }
        warnings.push(ConfigWarning::InvalidValue {
            path: client_setting_pseudo_path(),
            table: "format".to_string(),
            key: key.to_string(),
            message: format!(
                "{value} is outside the valid range {}-{}; using the default ({default})",
                BLANK_LINE_CAP_RANGE.start(),
                BLANK_LINE_CAP_RANGE.end()
            ),
        });
    }
    default
}

fn default_options(
    explicit: ExplicitFormatOverride,
    client_defaults: ExplicitFormatOverride,
    config_path: Option<&Path>,
    warnings: &mut Vec<ConfigWarning>,
) -> voyager_core::FormatOptions {
    resolve_casing_and_indent(&explicit, &DrutConfig::default(), &client_defaults, config_path, warnings)
}
