//! Contract tests for the `diagnose` tool (contracts/mcp-tools.md's
//! `diagnose` section, spec.md User Story 1's Acceptance Scenarios). Own
//! file (not shared with the other three tools' contract tests) so this
//! story's test task is genuinely parallel with its siblings
//! (`/speckit-analyze` finding F1).

use drut_mcp::diagnose::diagnose;
use drut_mcp::diagnose::DiagnosticsInput;
use drut_mcp::source::ScriptSource;

fn text_input(text: &str) -> DiagnosticsInput {
    DiagnosticsInput {
        source: ScriptSource {
            text: Some(text.to_string()),
            path: None,
        },
    }
}

fn path_input(path: &std::path::Path) -> DiagnosticsInput {
    DiagnosticsInput {
        source: ScriptSource {
            text: None,
            path: Some(path.to_string_lossy().to_string()),
        },
    }
}

#[test]
fn unmatched_if_reports_exactly_one_diagnostic_at_the_right_location() {
    let result = diagnose(&text_input("IF (a=b)\n; no ENDIF\n")).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].category, "UnmatchedIf");
    assert_eq!(result[0].start_line, 1);
}

#[test]
fn valid_script_produces_zero_diagnostics() {
    let result = diagnose(&text_input("IF (a=b)\nENDIF\n")).unwrap();
    assert!(result.is_empty());
}

#[test]
fn path_input_matches_that_files_own_text_content() {
    let text = "IF (a=b)\n; no ENDIF\n";
    let dir = std::env::temp_dir();
    let path = dir.join(format!("drut_mcp_diagnose_contract_{}.s", std::process::id()));
    std::fs::write(&path, text).unwrap();

    let via_text = diagnose(&text_input(text)).unwrap();
    let via_path = diagnose(&path_input(&path)).unwrap();
    assert_eq!(via_text, via_path);

    std::fs::remove_file(&path).unwrap();
}

/// `/speckit-analyze` finding C2: `InvalidEncoding` is only reachable via
/// `path` (an MCP tool-call argument is JSON, which cannot carry an invalid
/// byte sequence, Edge Cases) — deliberately in this same file as the tests
/// above (both are `diagnose`-only concerns), not marked `[P]` against them
/// per tasks.md's own note.
#[test]
fn invalid_encoding_is_reported_via_path_not_reachable_via_text() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("drut_mcp_diagnose_contract_encoding_{}.s", std::process::id()));
    // 0xFF is not valid UTF-8 and has no Windows-1252 interpretation either
    // (Windows-1252 leaves 0x81/0x8D/0x8F/0x90/0x9D undefined; 0xFF alone
    // *is* defined in Windows-1252 as 'ÿ' -- use an actually-undefined byte
    // instead so this really exercises InvalidEncoding, not a silent
    // Windows-1252 fallback).
    std::fs::write(&path, [b'I', b'F', b' ', 0x81, b'\n']).unwrap();

    let result = diagnose(&path_input(&path)).unwrap();
    assert!(
        result.iter().any(|d| d.category == "InvalidEncoding"),
        "expected an InvalidEncoding diagnostic, got {result:?}"
    );

    std::fs::remove_file(&path).unwrap();
}
