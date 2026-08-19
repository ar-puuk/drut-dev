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
    /// One of the seven reachable `DiagnosticKind` names, plus
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
        UnmatchedProcess => "UnmatchedProcess",
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
    fn unmatched_process_is_reported() {
        // 006-unmatched-process-diagnostic FR-007: proves drut-mcp's own
        // diagnose() tool surfaces the new kind end to end, not just that
        // voyager-core::parse itself reports it.
        let result = diagnose(&text_input("PROCESS PHASE=INPUT\nFILEI=ni.1\n")).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category, "UnmatchedProcess");
    }

    #[test]
    fn valid_script_has_zero_diagnostics() {
        let result = diagnose(&text_input("IF (a=b)\nENDIF\n")).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn undefined_token_hint_never_surfaces_here() {
        // 020-undefined-token-diagnostic SC-005: this stream is LSP-only,
        // built and published entirely inside drut-lsp/src/diagnostics.rs —
        // diagnose() must keep exposing exactly the six/seven real
        // DiagnosticKind names, even on a script with an unresolvable
        // @token@ reference.
        let result = diagnose(&text_input("MSG = @ScenarioDir@\n")).unwrap();
        assert!(
            result.is_empty(),
            "an unresolvable @token@ must never appear in diagnose()'s output, got: {result:?}"
        );
    }

    #[test]
    fn unused_token_hint_never_surfaces_here() {
        // 029-unused-token-diagnostic SC-005: this stream is LSP-only,
        // built and published entirely inside drut-lsp/src/diagnostics.rs —
        // diagnose() must keep exposing exactly the same DiagnosticKind
        // names as before this feature, even on a script with an assignment
        // that's never referenced via @token@.
        let result = diagnose(&text_input("ScenarioDir = 'X:\\model'\n")).unwrap();
        assert!(
            result.is_empty(),
            "an unused assignment must never appear in diagnose()'s output, got: {result:?}"
        );
    }

    #[test]
    fn empty_input_has_zero_diagnostics_not_an_error() {
        let result = diagnose(&text_input("")).unwrap();
        assert!(result.is_empty());
    }
}
