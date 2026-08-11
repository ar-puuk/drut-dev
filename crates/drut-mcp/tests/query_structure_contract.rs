//! Contract tests for the `query_structure` tool (contracts/mcp-tools.md's
//! `query_structure` section, spec.md User Story 3's Acceptance Scenarios).
//! Own file (`/speckit-analyze` finding F1).

use drut_mcp::query_structure::{query_structure, StructuralQueryInput};
use drut_mcp::source::ScriptSource;

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
fn explicit_if_endif_reports_kind_if_and_endif_location() {
    let result = query_structure(&input("IF (a=b)\nENDIF\n", 1, 2)).unwrap();
    assert_eq!(result.kind.as_deref(), Some("If"));
    assert_eq!(result.counterpart_start_line, Some(2));
}

#[test]
fn implicitly_closed_run_reports_resolved_body_extent_not_the_next_runs_opener() {
    let result = query_structure(&input(
        "RUN PGM=MATRIX\nZONES=5\nRUN PGM=HWYASSIGN\nENDRUN\n",
        1,
        2,
    ))
    .unwrap();
    assert_eq!(result.kind.as_deref(), Some("Run"));
    // Line 2 (ZONES=5, the first RUN's own last body line) -- not line 3,
    // where the second RUN opens.
    assert_eq!(result.counterpart_start_line, Some(2));
}

#[test]
fn position_with_no_enclosing_block_reports_kind_absent_not_an_error() {
    let result = query_structure(&input("IF (a=b)\nXYZZY LIST=1\nENDIF\n", 2, 2)).unwrap();
    assert!(result.kind.is_none());
}
