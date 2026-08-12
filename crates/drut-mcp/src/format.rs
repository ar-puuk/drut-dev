//! The `format` tool (FR-004, FR-005, data-model.md §4, contracts/mcp-tools.md).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::source::{ResolvedSource, ScriptSource, SourceError};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FormatInput {
    #[serde(flatten)]
    pub source: ScriptSource,
    /// `"upper"` / `"lower"` / absent — absent means
    /// `FormatOptions::default()` (untouched casing, FR-005).
    pub casing: Option<String>,
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
}

fn fidelity_name(f: voyager_core::EncodingFidelity) -> &'static str {
    use voyager_core::EncodingFidelity::*;
    match f {
        Faithful => "faithful",
        Recovered => "recovered",
        Lossy => "lossy",
    }
}

fn casing_option(casing: &Option<String>) -> Result<voyager_core::FormatOptions, String> {
    let convention = match casing.as_deref() {
        None => None,
        Some("upper") => Some(voyager_core::CasingConvention::Upper),
        Some("lower") => Some(voyager_core::CasingConvention::Lower),
        Some(other) => return Err(format!("`casing` must be \"upper\" or \"lower\" if given, got {other:?}")),
    };
    Ok(voyager_core::FormatOptions {
        casing: convention,
        // No MCP-side top-level-indent toggle in scope
        // (009-top-level-indent-toggle/spec.md Assumptions) — explicit,
        // not spread from `..Default::default()`, so the choice is
        // visible in the diff rather than implicit.
        top_level_indent: voyager_core::TopLevelIndentMode::default(),
    })
}

/// Runs `voyager_core::format`/`format_bytes` (depending on whether
/// `FormatInput`'s source resolved to text or bytes) and converts the
/// result into a `FormatResultDto` (FR-004: text, `changed`, and
/// `encoding_fidelity` always together).
pub fn format(input: &FormatInput) -> Result<FormatResultDto, String> {
    let options = casing_option(&input.casing)?;
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
}
