//! FR-012's no-panic guarantee, specifically for `drut-mcp`'s own tool
//! functions — every tool must return a well-formed result (an `Ok`
//! `DiagnosticDto`/`FormatResultDto`/`BlockInfoDto`/`KeywordLookupResult`
//! list, or a structured `Err`), never crash. Edge-case document shapes
//! reused directly from `drut-lsp/tests/no_panic.rs` (own convention this
//! crate's own no-panic sweep follows), not reinvented.

use drut_mcp::diagnose::{diagnose, DiagnosticsInput};
use drut_mcp::format::{format, FormatInput};
use drut_mcp::lookup_keyword::{lookup_keyword, KeywordLookupInput};
use drut_mcp::query_structure::{query_structure, StructuralQueryInput};
use drut_mcp::source::ScriptSource;

const EDGE_CASE_DOCUMENTS: &[(&str, &str)] = &[
    ("empty document", ""),
    ("only whitespace", "   \n\n\t\n"),
    ("only a comment", "; just a comment, nothing else"),
    ("truncated mid-statement", "IF (a=b"),
    ("unterminated block comment", "/* never closed"),
    ("replacement character as ordinary text", "PRINT LIST=\u{FFFD}\n"),
    ("supplementary-plane character at a boundary", "IF (a=b) PRINT LIST=😀\n"),
    ("deeply nested block comments", "/*/*/*/*/* five deep */*/*/*/*/\n"),
    ("only a BREAK, misplaced", "BREAK\n"),
    ("stray closer with nothing open", "ENDIF\nENDLOOP\nENDRUN\n"),
];

fn source(text: &str) -> ScriptSource {
    ScriptSource {
        text: Some(text.to_string()),
        path: None,
    }
}

#[test]
fn every_tool_survives_every_edge_case_document_without_panicking() {
    for (name, text) in EDGE_CASE_DOCUMENTS {
        let diag_result = diagnose(&DiagnosticsInput { source: source(text) });
        assert!(diag_result.is_ok(), "case {name:?}: diagnose must not error on malformed-but-valid text");

        let format_result = format(&FormatInput {
            source: source(text),
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
        });
        assert!(format_result.is_ok(), "case {name:?}: format must not error on malformed-but-valid text");

        // A spread of positions, including deliberately out-of-range ones
        // (clamping, not panicking, is the whole point of this sweep).
        for (line, column) in [(1, 1), (1, 999), (999, 1), (0, 0)] {
            let qs_result = query_structure(&StructuralQueryInput {
                source: source(text),
                line,
                column,
            });
            assert!(
                qs_result.is_ok(),
                "case {name:?} at ({line},{column}): query_structure must not error, only ever clamp"
            );
        }
    }

    // lookup_keyword takes no document text at all -- exercised separately
    // with a spread of plausible/implausible inputs, same no-panic bar.
    for control_word in [None, Some("RUN"), Some(""), Some("NOT_A_REAL_WORD_AT_ALL")] {
        let _ = lookup_keyword(&KeywordLookupInput {
            enclosing_control_word: control_word.map(str::to_string),
            spellcheck_token: Some("".to_string()),
        });
    }
}
