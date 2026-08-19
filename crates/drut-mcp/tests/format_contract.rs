//! Contract tests for the `format` tool (contracts/mcp-tools.md's `format`
//! section, spec.md User Story 2's Acceptance Scenarios). Own file
//! (`/speckit-analyze` finding F1).

use drut_mcp::format::{format, FormatInput};
use drut_mcp::source::ScriptSource;

fn text_input(text: &str) -> FormatInput {
    FormatInput {
        source: ScriptSource {
            text: Some(text.to_string()),
            path: None,
        },
        casing_control_words: None,
        casing_pair_keywords: None,
        casing_data_references: None,
        casing_function_calls: None,
        indent_width: None,
        indent_top_level: None,
        operator_spacing: None,
        blank_lines: None,
        blank_lines_top_cap: None,
        blank_lines_nested_cap: None,
            line_wrap: None,
            line_wrap_width: None,
            line_wrap_style: None,
        isolated: None,
    }
}

#[test]
fn incorrect_indentation_is_corrected_with_changed_true() {
    let result = format(&text_input("IF (a=b)\nPRINT LIST=1\nENDIF\n")).unwrap();
    assert_eq!(result.text, "IF (a=b)\n    PRINT LIST=1\nENDIF\n");
    assert!(result.changed);
}

#[test]
fn already_correct_text_is_byte_identical_with_changed_false() {
    let text = "IF (a=b)\n    PRINT LIST=1\nENDIF\n";
    let result = format(&text_input(text)).unwrap();
    assert_eq!(result.text, text);
    assert!(!result.changed);
}

#[test]
fn feeding_the_tools_own_output_back_in_proves_idempotence() {
    let first = format(&text_input("IF (a=b)\nPRINT LIST=1\nENDIF\n")).unwrap();
    assert!(first.changed);
    let second = format(&text_input(&first.text)).unwrap();
    assert!(!second.changed);
}
