//! The `diagnose` tool (FR-003, data-model.md §3, contracts/mcp-tools.md).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::source::{ResolvedSource, ScriptSource, SourceError};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DiagnosticsInput {
    #[serde(flatten)]
    pub source: ScriptSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct DiagnosticDto {
    /// One of the six reachable `DiagnosticKind` names, plus
    /// `InvalidEncoding` when reachable via a `path` input (FR-003).
    pub category: String,
    pub message: String,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

fn category_name(kind: voyager_core::DiagnosticKind) -> &'static str {
    use voyager_core::DiagnosticKind::*;
    match kind {
        UnmatchedIf => "UnmatchedIf",
        UnmatchedLoop => "UnmatchedLoop",
        UnclosedBlockComment => "UnclosedBlockComment",
        InvalidContinuation => "InvalidContinuation",
        UnmatchedRun => "UnmatchedRun",
        MisplacedBreak => "MisplacedBreak",
        InvalidEncoding => "InvalidEncoding",
    }
}

fn to_dto(d: &voyager_core::Diagnostic) -> DiagnosticDto {
    DiagnosticDto {
        category: category_name(d.kind).to_string(),
        message: d.message.clone(),
        start_line: d.span.start.line,
        start_column: d.span.start.column,
        end_line: d.span.end.line,
        end_column: d.span.end.column,
    }
}

/// Runs `voyager_core::parse`/`parse_bytes` (depending on whether
/// `DiagnosticsInput`'s source resolved to text or bytes) and converts every
/// `Diagnostic` in the result — never a narrowed subset (FR-003). Empty
/// input produces an empty list, not an error (Edge Cases).
pub fn diagnose(input: &DiagnosticsInput) -> Result<Vec<DiagnosticDto>, SourceError> {
    let result = match input.source.resolve()? {
        ResolvedSource::Text(text) => voyager_core::parse(&text),
        ResolvedSource::Bytes(bytes) => voyager_core::parse_bytes(&bytes),
    };
    Ok(result.diagnostics.iter().map(to_dto).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_input(text: &str) -> DiagnosticsInput {
        DiagnosticsInput {
            source: ScriptSource {
                text: Some(text.to_string()),
                path: None,
            },
        }
    }

    #[test]
    fn unmatched_if_is_reported() {
        let result = diagnose(&text_input("IF (a=b)\n; no ENDIF\n")).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category, "UnmatchedIf");
    }

    #[test]
    fn valid_script_has_zero_diagnostics() {
        let result = diagnose(&text_input("IF (a=b)\nENDIF\n")).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn empty_input_has_zero_diagnostics_not_an_error() {
        let result = diagnose(&text_input("")).unwrap();
        assert!(result.is_empty());
    }
}
