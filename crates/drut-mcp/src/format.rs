//! The `format` tool (FR-004, FR-005, data-model.md §4, contracts/mcp-tools.md).

use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::source::{ResolvedSource, ScriptSource, SourceError};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FormatInput {
    #[serde(flatten)]
    pub source: ScriptSource,
    /// `"preserve"` / `"upper"` / `"lower"` / absent — absent means
    /// "consult the resolved `drut.toml`, then the built-in default"
    /// (012-toml-configuration), same precedence CLI flags follow.
    /// `"preserve"` added by 014-casing-preserve-mode FR-007. Unchanged by
    /// 017-casing-categories-indent-width: still applies to
    /// `control_words` + `pair_keywords` only, exactly as before — the
    /// three parameters below are new, independent, granular overrides.
    pub casing: Option<String>,
    /// Independent override for the control-words category
    /// (017-casing-categories-indent-width FR-001) — wins over `casing`
    /// for this category specifically when both are given.
    pub control_words_casing: Option<String>,
    /// Independent override for the pair-keywords category
    /// (017-casing-categories-indent-width FR-001) — wins over `casing`
    /// for this category specifically when both are given.
    pub pair_keywords_casing: Option<String>,
    /// Independent override for the data-references category — Matrix/
    /// Line/Node/Zone/Database abbreviations, the output-record and
    /// link-endpoint tokens, and the two reserved loop-index identifiers
    /// (017-casing-categories-indent-width FR-004). Not reachable by
    /// `casing` at all.
    pub data_references_casing: Option<String>,
    /// Spaces per nesting level of block indentation
    /// (017-casing-categories-indent-width FR-009), 1–16 if given. Same
    /// absent-means-"consult drut.toml, then default (4)" precedence as
    /// every other setting here.
    pub indent_width: Option<u8>,
    /// `"preserve"` / `"normalize"` / absent — same precedence as `casing`
    /// above. Closes the former CLI/MCP asymmetry (012-toml-configuration
    /// FR-010): this tool previously had no way to reach `top_level_indent`
    /// at all.
    pub top_level_indent: Option<String>,
    /// `"preserve"` / `"fixed"` / `"auto"` / absent — same absent-means-
    /// "consult drut.toml, then default (preserve)" precedence as `casing`/
    /// `top_level_indent` above (018-operator-spacing). `"preserve"` leaves
    /// operator/comma/bracket-paren spacing exactly as written; `"fixed"`
    /// normalizes it; `"auto"` does everything `"fixed"` does plus aligning
    /// consecutive `Assignment` statements' `=`.
    pub operator_spacing: Option<String>,
    /// `"preserve"` / `"auto"` / absent — same absent-means-"consult
    /// drut.toml, then default (preserve)" precedence as `casing`/
    /// `operator_spacing` above (019-blank-line-normalization). `"preserve"`
    /// leaves every blank-line run exactly as written, however long;
    /// `"auto"` contracts a run down to the applicable cap (see the two
    /// parameters below) only when it exceeds that cap.
    pub blank_lines: Option<String>,
    /// The maximum number of consecutive blank lines `auto` allows between
    /// top-level statements/blocks before contracting the run
    /// (019-blank-line-normalization FR-002), 1–50 if given. Same
    /// absent-means-"consult drut.toml, then default (2)" precedence as
    /// every other setting here.
    pub top_level_blank_line_cap: Option<u8>,
    /// The maximum number of consecutive blank lines `auto` allows inside
    /// any block's own body, uniformly regardless of nesting depth, before
    /// contracting the run (019-blank-line-normalization FR-002/FR-008),
    /// 1–50 if given. Same precedence as `top_level_blank_line_cap` above,
    /// independently — built-in default `1`.
    pub nested_blank_line_cap: Option<u8>,
    /// Skip `drut.toml` discovery entirely for this call, using built-in
    /// defaults plus `casing`/`top_level_indent` above if given
    /// (012-toml-configuration US3, mirroring the CLI's `--isolated`).
    /// Absent is treated as `false`.
    pub isolated: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct FormatResultDto {
    pub text: String,
    pub changed: bool,
    /// `"faithful"` / `"recovered"` / `"lossy"` — always `"faithful"` for a
    /// `text`-sourced input.
    pub encoding_fidelity: String,
    /// Line numbers of every `; FMT: OFF` marker left unmatched at
    /// end-of-file (010-fmt-region-markers FR-010). Empty in the common
    /// case. Line only — a marker's column is never meaningful, since it
    /// always starts a comment-only line.
    pub unclosed_fmt_off_lines: Vec<u32>,
    /// Human-readable rendering of every problem found in a resolved
    /// `drut.toml` (012-toml-configuration FR-011). Empty in the common
    /// case (no file, or a fully valid one). Never blocks `format` from
    /// completing — the affected setting(s) simply fall back to the
    /// built-in default, same as `unclosed_fmt_off_lines`'s own
    /// informational-only treatment.
    pub config_warnings: Vec<String>,
}

fn fidelity_name(f: voyager_core::EncodingFidelity) -> &'static str {
    use voyager_core::EncodingFidelity::*;
    match f {
        Faithful => "faithful",
        Recovered => "recovered",
        Lossy => "lossy",
    }
}

/// Shared by `casing` and the three new granular `*_casing` parameters
/// (017-casing-categories-indent-width) — identical accepted-value shape at
/// every one of them. `"auto"` (or any other string) is deliberately just
/// another unrecognized value here, not a special case — this feature
/// ships no built-in preset (FR-003).
fn parse_casing_param(field: &str, value: Option<&str>) -> Result<Option<voyager_core::CasingConvention>, String> {
    match value {
        None => Ok(None),
        Some("preserve") => Ok(Some(voyager_core::CasingConvention::Preserve)),
        Some("upper") => Ok(Some(voyager_core::CasingConvention::Upper)),
        Some("lower") => Ok(Some(voyager_core::CasingConvention::Lower)),
        Some(other) => Err(format!("`{field}` must be \"preserve\", \"upper\", or \"lower\" if given, got {other:?}")),
    }
}

fn explicit_override(input: &FormatInput) -> Result<drut_config::ExplicitFormatOverride, String> {
    let casing = parse_casing_param("casing", input.casing.as_deref())?;
    let control_words_casing = parse_casing_param("control_words_casing", input.control_words_casing.as_deref())?;
    let pair_keywords_casing = parse_casing_param("pair_keywords_casing", input.pair_keywords_casing.as_deref())?;
    let data_references_casing =
        parse_casing_param("data_references_casing", input.data_references_casing.as_deref())?;
    if let Some(width) = input.indent_width {
        if !(1..=16).contains(&width) {
            return Err(format!("`indent_width` must be between 1 and 16 if given, got {width}"));
        }
    }
    let top_level_indent = match input.top_level_indent.as_deref() {
        None => None,
        Some("preserve") => Some(voyager_core::TopLevelIndentMode::Preserve),
        Some("normalize") => Some(voyager_core::TopLevelIndentMode::Normalize),
        Some(other) => {
            return Err(format!(
                "`top_level_indent` must be \"preserve\" or \"normalize\" if given, got {other:?}"
            ))
        }
    };
    let operator_spacing = match input.operator_spacing.as_deref() {
        None => None,
        Some("preserve") => Some(voyager_core::OperatorSpacing::Preserve),
        Some("fixed") => Some(voyager_core::OperatorSpacing::Fixed),
        Some("auto") => Some(voyager_core::OperatorSpacing::Auto),
        Some(other) => {
            return Err(format!(
                "`operator_spacing` must be \"preserve\", \"fixed\", or \"auto\" if given, got {other:?}"
            ))
        }
    };
    let blank_lines = match input.blank_lines.as_deref() {
        None => None,
        Some("preserve") => Some(voyager_core::BlankLineMode::Preserve),
        Some("auto") => Some(voyager_core::BlankLineMode::Auto),
        Some(other) => {
            return Err(format!("`blank_lines` must be \"preserve\" or \"auto\" if given, got {other:?}"))
        }
    };
    if let Some(cap) = input.top_level_blank_line_cap {
        if !(1..=50).contains(&cap) {
            return Err(format!("`top_level_blank_line_cap` must be between 1 and 50 if given, got {cap}"));
        }
    }
    if let Some(cap) = input.nested_blank_line_cap {
        if !(1..=50).contains(&cap) {
            return Err(format!("`nested_blank_line_cap` must be between 1 and 50 if given, got {cap}"));
        }
    }
    Ok(drut_config::ExplicitFormatOverride {
        casing,
        control_words_casing,
        pair_keywords_casing,
        data_references_casing,
        top_level_indent,
        indent_width: input.indent_width,
        operator_spacing,
        blank_lines,
        top_level_blank_line_cap: input.top_level_blank_line_cap,
        nested_blank_line_cap: input.nested_blank_line_cap,
    })
}

/// Runs `voyager_core::format`/`format_bytes` (depending on whether
/// `FormatInput`'s source resolved to text or bytes) and converts the
/// result into a `FormatResultDto` (FR-004: text, `changed`, and
/// `encoding_fidelity` always together).
///
/// `casing`/`top_level_indent` settings are resolved via `drut_config::
/// resolve_format_options` (012-toml-configuration): explicit parameters
/// above win, else a `drut.toml` discovered from `input.source.path` (if
/// set — a `text`-sourced call has no real location, so no discovery is
/// attempted, matching the LSP untitled-buffer case), else the built-in
/// default.
pub fn format(input: &FormatInput) -> Result<FormatResultDto, String> {
    let explicit = explicit_override(input)?;
    let file_path = input.source.path.as_deref().map(Path::new);
    // 021-editor-settings-config: MCP has no client-settings tier of its own
    // (spec.md FR-007) — always the empty default, never any other value.
    let (options, warnings) = drut_config::resolve_format_options(
        file_path,
        input.isolated.unwrap_or(false),
        explicit,
        drut_config::ExplicitFormatOverride::default(),
    );

    let source = input.source.resolve().map_err(|e: SourceError| e.to_string())?;
    let result = match source {
        ResolvedSource::Text(text) => voyager_core::format(&text, options),
        ResolvedSource::Bytes(bytes) => voyager_core::format_bytes(&bytes, options),
    };
    Ok(FormatResultDto {
        text: result.text,
        changed: result.changed,
        encoding_fidelity: fidelity_name(result.encoding_fidelity).to_string(),
        unclosed_fmt_off_lines: result.unclosed_fmt_off_markers.iter().map(|p| p.line).collect(),
        config_warnings: warnings.iter().map(ToString::to_string).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_input(text: &str, casing: Option<&str>) -> FormatInput {
        FormatInput {
            source: ScriptSource {
                text: Some(text.to_string()),
                path: None,
            },
            casing: casing.map(str::to_string),
            control_words_casing: None,
            pair_keywords_casing: None,
            data_references_casing: None,
            indent_width: None,
            top_level_indent: None,
            operator_spacing: None,
            blank_lines: None,
            top_level_blank_line_cap: None,
            nested_blank_line_cap: None,
            isolated: None,
        }
    }

    fn path_input(path: &str, casing: Option<&str>, top_level_indent: Option<&str>, isolated: Option<bool>) -> FormatInput {
        FormatInput {
            source: ScriptSource {
                text: None,
                path: Some(path.to_string()),
            },
            casing: casing.map(str::to_string),
            control_words_casing: None,
            pair_keywords_casing: None,
            data_references_casing: None,
            indent_width: None,
            top_level_indent: top_level_indent.map(str::to_string),
            operator_spacing: None,
            blank_lines: None,
            top_level_blank_line_cap: None,
            nested_blank_line_cap: None,
            isolated,
        }
    }

    fn granular_input(
        text: &str,
        control_words_casing: Option<&str>,
        pair_keywords_casing: Option<&str>,
        data_references_casing: Option<&str>,
        indent_width: Option<u8>,
    ) -> FormatInput {
        FormatInput {
            source: ScriptSource {
                text: Some(text.to_string()),
                path: None,
            },
            casing: None,
            control_words_casing: control_words_casing.map(str::to_string),
            pair_keywords_casing: pair_keywords_casing.map(str::to_string),
            data_references_casing: data_references_casing.map(str::to_string),
            indent_width,
            top_level_indent: None,
            operator_spacing: None,
            blank_lines: None,
            top_level_blank_line_cap: None,
            nested_blank_line_cap: None,
            isolated: None,
        }
    }

    #[test]
    fn incorrect_body_indentation_is_corrected() {
        let result = format(&text_input("IF (a=b)\nPRINT LIST=1\nENDIF\n", None)).unwrap();
        assert_eq!(result.text, "IF (a=b)\n    PRINT LIST=1\nENDIF\n");
        assert!(result.changed);
    }

    #[test]
    fn already_correct_text_is_byte_identical_and_unchanged() {
        let text = "IF (a=b)\n    PRINT LIST=1\nENDIF\n";
        let result = format(&text_input(text, None)).unwrap();
        assert_eq!(result.text, text);
        assert!(!result.changed);
    }

    #[test]
    fn feeding_the_result_back_in_is_idempotent() {
        let first = format(&text_input("IF (a=b)\nPRINT LIST=1\nENDIF\n", None)).unwrap();
        assert!(first.changed);
        let second = format(&text_input(&first.text, None)).unwrap();
        assert!(!second.changed);
        assert_eq!(second.text, first.text);
    }

    #[test]
    fn casing_defaults_to_preserve_not_upper_or_lower() {
        // 014-casing-preserve-mode FR-008/SC-003 (point 3 of 3)/User Story
        // 3 -- mirrors top_level_indentation_defaults_to_preserve_not_
        // normalize's shape for the sibling setting. No casing param, no
        // governing drut.toml.
        let text = "if (a=b)\nendif\n";
        let result = format(&text_input(text, None)).unwrap();
        assert_eq!(result.text, text, "lowercase control words must be left untouched by default");
        assert!(!result.changed);
    }

    #[test]
    fn top_level_indentation_defaults_to_preserve_not_normalize() {
        // 009-top-level-indent-toggle FR-004(c): the MCP format tool has
        // no top-level-indent toggle of its own, so it must pick up
        // FormatOptions::default() -- confirmed here directly, not
        // inferred from any other adapter's own test passing.
        let text = "    IF (a=b)\n        PRINT LIST=1\n    ENDIF\n";
        let result = format(&text_input(text, None)).unwrap();
        assert_eq!(result.text, text, "non-zero top-level indentation must be left untouched by default");
        assert!(!result.changed);
    }

    // -- FMT region markers (010-fmt-region-markers) -------------------------

    #[test]
    fn protected_range_survives_through_the_mcp_format_tool() {
        // 010-fmt-region-markers FR-007/US3, added after /speckit-analyze
        // review (G2): FR-007 requires identical protection at every
        // adapter surface including MCP -- this previously had no
        // assertion that MCP's format() actually protects a range, only
        // that the notice field populates (see the test below).
        let text = "IF (X=1)\nY = 1\n; FMT: OFF\n  weird = 1\n; FMT: ON\nZ = 2\nENDIF\n";
        let result = format(&text_input(text, None)).unwrap();
        assert_eq!(
            result.text,
            "IF (X=1)\n    Y = 1\n; FMT: OFF\n  weird = 1\n; FMT: ON\n    Z = 2\nENDIF\n",
            "the protected range must stay byte-for-byte unchanged while everything \
             else normalizes"
        );
    }

    #[test]
    fn unclosed_fmt_off_lines_is_populated_and_empty_in_the_common_case() {
        let unclosed = format(&text_input("IF (X=1)\n; FMT: OFF\nY = 1\nENDIF\n", None)).unwrap();
        assert_eq!(unclosed.unclosed_fmt_off_lines, vec![2]);

        let clean = format(&text_input("IF (X=1)\nY = 1\nENDIF\n", None)).unwrap();
        assert!(clean.unclosed_fmt_off_lines.is_empty());

        let matched = format(&text_input(
            "IF (X=1)\n; FMT: OFF\nY = 1\n; FMT: ON\nENDIF\n",
            None,
        ))
        .unwrap();
        assert!(matched.unclosed_fmt_off_lines.is_empty());
    }

    // -- 012-toml-configuration (T023, T027, T032) ---------------------------

    fn write_config(dir: &std::path::Path, content: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("drut.toml"), content).unwrap();
    }

    fn temp_project(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("drut_mcp_format_test_{}_{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn path_sourced_call_is_governed_by_a_nearby_drut_toml_with_no_params_passed() {
        let dir = temp_project("governed");
        write_config(&dir, "[format]\ncasing = \"upper\"\n");
        let file = dir.join("x.s");
        std::fs::write(&file, "if (a=b)\nendif\n").unwrap();

        let result = format(&path_input(file.to_str().unwrap(), None, None, None)).unwrap();
        assert_eq!(result.text, "IF (a=b)\nENDIF\n");
        assert!(result.config_warnings.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn text_sourced_call_never_attempts_discovery_at_all() {
        // A `text`-sourced input has no real on-disk location
        // (`input.source.path` is `None`), so `resolve_format_options`
        // receives `file_path: None` and skips discovery entirely --
        // proven directly against `resolve_format_options`'s own contract
        // (data-model.md: "file_path: None ... also skips discovery"),
        // not by planting a real drut.toml somewhere in the test process's
        // actual working directory, which would risk polluting real repo
        // state across parallel test runs.
        let result = format(&text_input("if (a=b)\nendif\n", None)).unwrap();
        assert_eq!(result.text, "if (a=b)\nendif\n", "text-sourced calls must resolve to built-in defaults only");
    }

    #[test]
    fn explicit_casing_param_overrides_a_present_drut_toml() {
        let dir = temp_project("override");
        write_config(&dir, "[format]\ncasing = \"lower\"\n");
        let file = dir.join("x.s");
        std::fs::write(&file, "IF (X=1)\nENDIF\n").unwrap();

        let overridden = format(&path_input(file.to_str().unwrap(), Some("upper"), None, None)).unwrap();
        assert_eq!(overridden.text, "IF (X=1)\nENDIF\n");

        let reverted = format(&path_input(file.to_str().unwrap(), None, None, None)).unwrap();
        assert_eq!(reverted.text, "if (X=1)\nendif\n", "no explicit param -- the file's own setting applies again");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn explicit_casing_preserve_param_overrides_a_present_drut_toml() {
        // 014-casing-preserve-mode FR-007/FR-009/User Story 1.
        let dir = temp_project("override_preserve");
        write_config(&dir, "[format]\ncasing = \"upper\"\n");
        let file = dir.join("x.s");
        std::fs::write(&file, "if (x=1)\nendif\n").unwrap();

        let overridden = format(&path_input(file.to_str().unwrap(), Some("preserve"), None, None)).unwrap();
        assert_eq!(overridden.text, "if (x=1)\nendif\n", "explicit preserve must win over the file's upper setting");

        let reverted = format(&path_input(file.to_str().unwrap(), None, None, None)).unwrap();
        assert_eq!(reverted.text, "IF (x=1)\nENDIF\n", "no explicit param -- the file's own setting applies again");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- 017-casing-categories-indent-width (tasks.md T017/T030/T039) --

    #[test]
    fn granular_data_references_casing_reaches_previously_unreachable_tokens() {
        // US2: mw/li/ni/i/j -- unreachable by any casing setting before
        // this feature -- become reachable via data_references_casing.
        let input = granular_input("mw[1] = mi.1.1\nx = li.FT\nif (i=25) y = j\n", None, None, Some("upper"), None);
        let result = format(&input).unwrap();
        assert_eq!(result.text, "MW[1] = MI.1.1\nx = LI.FT\nif (I=25) y = J\n");
    }

    #[test]
    fn explicit_granular_override_wins_for_its_own_category_only() {
        let input = granular_input(
            "if (x=1)\nMW[1] = 1\nendif\n",
            Some("upper"),
            None,
            Some("lower"),
            None,
        );
        let result = format(&input).unwrap();
        assert_eq!(
            result.text, "IF (x=1)\n    mw[1] = 1\nENDIF\n",
            "control_words upper and data_references lower each independently applied"
        );
    }

    #[test]
    fn auto_is_rejected_as_an_invalid_casing_value_at_every_casing_param() {
        // FR-003: this feature ships no built-in preset -- "auto" is just
        // another unrecognized string at every one of the four params.
        for field_value in [
            granular_input("x = 1\n", Some("auto"), None, None, None),
            granular_input("x = 1\n", None, Some("auto"), None, None),
            granular_input("x = 1\n", None, None, Some("auto"), None),
        ] {
            let err = format(&field_value).unwrap_err();
            assert!(err.contains("auto"), "expected an error naming the rejected value, got: {err}");
        }
        let legacy = text_input("x = 1\n", Some("auto"));
        assert!(format(&legacy).is_err(), "legacy casing param must also reject auto");
    }

    #[test]
    fn indent_width_param_overrides_config_and_out_of_range_is_a_clean_error() {
        let dir = temp_project("indent_width");
        write_config(&dir, "[format]\nindent_width = 4\n");
        let file = dir.join("x.s");
        std::fs::write(&file, "IF (X=1)\nY = 1\nENDIF\n").unwrap();

        let mut overridden = path_input(file.to_str().unwrap(), None, None, None);
        overridden.indent_width = Some(2);
        let result = format(&overridden).unwrap();
        assert_eq!(result.text, "IF (X=1)\n  Y = 1\nENDIF\n");

        let mut out_of_range = path_input(file.to_str().unwrap(), None, None, None);
        out_of_range.indent_width = Some(0);
        assert!(format(&out_of_range).is_err(), "an explicit out-of-range indent_width must be a clean error, not silently clamped");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn isolated_true_ignores_a_present_drut_toml_entirely() {
        let dir = temp_project("isolated");
        write_config(&dir, "[format]\ncasing = \"upper\"\ntop_level_indent = \"normalize\"\n");
        let file = dir.join("x.s");
        std::fs::write(&file, "    if (x=1)\n        y = 2\n    endif\n").unwrap();

        let result = format(&path_input(file.to_str().unwrap(), None, None, Some(true))).unwrap();
        assert_eq!(
            result.text, "    if (x=1)\n        y = 2\n    endif\n",
            "isolated must match built-in defaults exactly, ignoring the file"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn malformed_drut_toml_is_reported_but_format_still_completes() {
        let dir = temp_project("malformed");
        write_config(&dir, "[format]\ncasing = \"sideways\"\n");
        let file = dir.join("x.s");
        std::fs::write(&file, MESSY_FOR_CONFIG_TEST).unwrap();

        let result = format(&path_input(file.to_str().unwrap(), None, None, None)).unwrap();
        assert_eq!(result.text, CLEAN_FOR_CONFIG_TEST, "formatting must still complete using the built-in default");
        assert_eq!(result.config_warnings.len(), 1);
        assert!(result.config_warnings[0].contains("casing"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    const MESSY_FOR_CONFIG_TEST: &str = "IF (X=1)\nY = 2\nENDIF\n";
    const CLEAN_FOR_CONFIG_TEST: &str = "IF (X=1)\n    Y = 2\nENDIF\n";

    // -- 018-operator-spacing (tasks.md T018, T021) --

    #[test]
    fn operator_spacing_param_overrides_a_drut_toml_resolved_value() {
        let dir = temp_project("operator_spacing");
        write_config(&dir, "[format]\noperator_spacing = \"preserve\"\n");
        let file = dir.join("x.s");
        std::fs::write(&file, "ZONES   = 1\n").unwrap();

        let mut overridden = path_input(file.to_str().unwrap(), None, None, None);
        overridden.operator_spacing = Some("fixed".to_string());
        let result = format(&overridden).unwrap();
        assert_eq!(result.text, "ZONES = 1\n");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn operator_spacing_invalid_value_is_a_clean_error() {
        // FR-011/SC-004: same closed-set shape as casing/top_level_indent --
        // an invalid value is a clean tool-call error, not a silent
        // fallback (that softer behavior is drut.toml-only).
        let mut input = text_input("ZONES   = 1\n", None);
        input.operator_spacing = Some("tight".to_string());
        let err = format(&input).unwrap_err();
        assert!(err.contains("operator_spacing"), "expected an error naming the field, got: {err}");
    }

    // -- 019-blank-line-normalization (tasks.md T017) --

    #[test]
    fn blank_lines_param_overrides_a_drut_toml_resolved_preserve() {
        let dir = temp_project("blank_lines");
        write_config(&dir, "[format]\nblank_lines = \"preserve\"\n");
        let file = dir.join("x.s");
        std::fs::write(&file, "X = 1\n\n\n\n\n\nY = 2\n").unwrap();

        let mut overridden = path_input(file.to_str().unwrap(), None, None, None);
        overridden.blank_lines = Some("auto".to_string());
        let result = format(&overridden).unwrap();
        assert_eq!(result.text, "X = 1\n\n\nY = 2\n", "the run of 5 must contract to the default top-level cap (2)");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn blank_line_cap_params_override_the_built_in_defaults() {
        let mut input = text_input("X = 1\n\n\n\n\n\nY = 2\n", None);
        input.blank_lines = Some("auto".to_string());
        input.top_level_blank_line_cap = Some(1);
        let result = format(&input).unwrap();
        assert_eq!(result.text, "X = 1\n\nY = 2\n");
    }

    #[test]
    fn blank_lines_invalid_value_is_a_clean_error() {
        // FR-011/SC-004: same closed-set shape as casing/operator_spacing --
        // an invalid value is a clean tool-call error, not a silent
        // fallback (that softer behavior is drut.toml-only).
        let mut input = text_input("X = 1\n", None);
        input.blank_lines = Some("sometimes".to_string());
        let err = format(&input).unwrap_err();
        assert!(err.contains("blank_lines"), "expected an error naming the field, got: {err}");
    }

    #[test]
    fn blank_line_cap_out_of_range_is_a_clean_error_not_a_silent_clamp() {
        let mut input = text_input("X = 1\n", None);
        input.top_level_blank_line_cap = Some(0);
        assert!(format(&input).is_err(), "an explicit out-of-range top_level_blank_line_cap must be a clean error");

        let mut input = text_input("X = 1\n", None);
        input.nested_blank_line_cap = Some(51);
        assert!(format(&input).is_err(), "an explicit out-of-range nested_blank_line_cap must be a clean error");
    }
}
