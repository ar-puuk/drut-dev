//! The `query_structure` tool (FR-006, FR-007, data-model.md §5,
//! contracts/mcp-tools.md).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use voyager_core::{BlockKindName, Position};

use crate::source::{ResolvedSource, ScriptSource, SourceError};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct StructuralQueryInput {
    #[serde(flatten)]
    pub source: ScriptSource,
    /// 1-based, matching `voyager_core::Position`'s own convention — no
    /// UTF-16 translation needed or performed (that's an LSP wire-protocol
    /// concern `drut-lsp`'s own `position.rs` owns, not this tool's).
    pub line: u32,
    /// 1-based `char` count.
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct BlockInfoDto {
    /// Absent when no block encloses the position — a normal, successful
    /// result (FR-007), never an error.
    pub kind: Option<String>,
    pub is_short_if: bool,
    pub counterpart_start_line: Option<u32>,
    pub counterpart_start_column: Option<u32>,
    pub counterpart_end_line: Option<u32>,
    pub counterpart_end_column: Option<u32>,
}

fn kind_name(kind: BlockKindName) -> String {
    kind.as_str().to_string()
}

/// Clamps `pos` to the nearest position that actually exists in `text` —
/// the same no-panic discipline `contracts/block-resolution-api.md`
/// requires of every caller of `voyager_core::block_at` (that function
/// itself doesn't clamp; it's each caller's own translation-boundary
/// concern, matching how `drut-lsp`'s `position.rs` already clamps before
/// ever constructing the `Position` `block_at` receives).
fn clamp_position(text: &str, line: u32, column: u32) -> Position {
    let lines: Vec<&str> = text.lines().collect();
    let line_count = lines.len().max(1) as u32;
    let clamped_line = line.clamp(1, line_count);
    let line_text = lines.get((clamped_line - 1) as usize).copied().unwrap_or("");
    let max_column = line_text.chars().count() as u32 + 1;
    let clamped_column = column.clamp(1, max_column);
    Position::new(clamped_line, clamped_column)
}

/// Parses the source, resolves `block_at` for the (clamped) position, and
/// converts the result into a `BlockInfoDto`.
pub fn query_structure(input: &StructuralQueryInput) -> Result<BlockInfoDto, SourceError> {
    let source = input.source.resolve()?;
    let text_for_clamping;
    let result = match &source {
        ResolvedSource::Text(text) => {
            text_for_clamping = text.clone();
            voyager_core::parse(text)
        }
        ResolvedSource::Bytes(bytes) => {
            let parsed = voyager_core::parse_bytes(bytes);
            // Re-decode once more here purely to get a `&str` for position
            // clamping -- `parse_bytes` already did the real decode
            // internally; this is not a second, divergent decode decision,
            // just reusing the same lossless text `parse_bytes` itself
            // produced, needed because it doesn't hand that text back.
            text_for_clamping = String::from_utf8_lossy(bytes).into_owned();
            parsed
        }
    };

    let pos = clamp_position(&text_for_clamping, input.line, input.column);
    let info = voyager_core::block_at(&result.nodes, &result.diagnostics, pos);

    Ok(match info {
        None => BlockInfoDto {
            kind: None,
            is_short_if: false,
            counterpart_start_line: None,
            counterpart_start_column: None,
            counterpart_end_line: None,
            counterpart_end_column: None,
        },
        Some(info) => BlockInfoDto {
            kind: Some(kind_name(info.kind)),
            is_short_if: info.is_short_if,
            counterpart_start_line: info.counterpart.map(|c| c.start.line),
            counterpart_start_column: info.counterpart.map(|c| c.start.column),
            counterpart_end_line: info.counterpart.map(|c| c.end.line),
            counterpart_end_column: info.counterpart.map(|c| c.end.column),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(text: &str, line: u32, column: u32) -> StructuralQueryInput {
        StructuralQueryInput {
            source: ScriptSource {
                text: Some(text.to_string()),
                path: None,
            },
            line,
            column,
        }
    }

    #[test]
    fn explicit_if_reports_kind_and_endif_location() {
        let result = query_structure(&input("IF (a=b)\nENDIF\n", 1, 2)).unwrap();
        assert_eq!(result.kind.as_deref(), Some("If"));
        assert_eq!(result.counterpart_start_line, Some(2));
    }

    #[test]
    fn implicitly_closed_run_reports_resolved_body_extent_not_the_next_opener() {
        let result = query_structure(&input(
            "RUN PGM=MATRIX\nZONES=5\nRUN PGM=HWYASSIGN\nENDRUN\n",
            1,
            2,
        ))
        .unwrap();
        assert_eq!(result.kind.as_deref(), Some("Run"));
        assert_eq!(result.counterpart_start_line, Some(2));
    }

    #[test]
    fn no_enclosing_block_is_a_normal_result_not_an_error() {
        let result = query_structure(&input("IF (a=b)\nXYZZY LIST=1\nENDIF\n", 2, 2)).unwrap();
        assert!(result.kind.is_none());
    }

    #[test]
    fn out_of_range_position_clamps_rather_than_panics() {
        let result = query_structure(&input("IF (a=b)\nENDIF\n", 999, 999));
        assert!(result.is_ok());
    }
}
