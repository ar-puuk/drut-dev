//! The `format` tool (FR-004, FR-005, data-model.md §4, contracts/mcp-tools.md).

use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::source::{ResolvedSource, ScriptSource, SourceError};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FormatInput {
    #[serde(flatten)]
    pub source: ScriptSource,
    /// `"upper"` / `"lower"` / absent — absent means "consult the resolved
    /// `drut.toml`, then the built-in default" (012-toml-configuration),
    /// same precedence CLI flags follow.
    pub casing: Option<String>,
    /// `"preserve"` / `"normalize"` / absent — same precedence as `casing`
    /// above. Closes the former CLI/MCP asymmetry (012-toml-configuration
    /// FR-010): this tool previously had no way to reach `top_level_indent`
    /// at all.
    pub top_level_indent: Option<String>,
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

fn explicit_override(input: &FormatInput) -> Result<drut_config::ExplicitFormatOverride, String> {
    let casing = match input.casing.as_deref() {
        None => None,
        Some("upper") => Some(voyager_core::CasingConvention::Upper),
        Some("lower") => Some(voyager_core::CasingConvention::Lower),
        Some(other) => return Err(format!("`casing` must be \"upper\" or \"lower\" if given, got {other:?}")),
    };
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
    Ok(drut_config::ExplicitFormatOverride { casing, top_level_indent })
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
    let (options, warnings) =
        drut_config::resolve_format_options(file_path, input.isolated.unwrap_or(false), explicit);

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
            top_level_indent: None,
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
            top_level_indent: top_level_indent.map(str::to_string),
            isolated,
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
}
