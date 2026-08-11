//! Contract tests for the `lookup_keyword` tool (contracts/mcp-tools.md's
//! `lookup_keyword` section, spec.md User Story 4's Acceptance Scenarios).
//! Own file (`/speckit-analyze` finding F1).

use drut_mcp::lookup_keyword::{lookup_keyword, KeywordLookupInput};

#[test]
fn run_scoped_lookup_includes_pgm_msg_prnfile() {
    let result = lookup_keyword(&KeywordLookupInput {
        enclosing_control_word: Some("RUN".to_string()),
        spellcheck_token: None,
    });
    let names: Vec<&str> = result.candidates.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"PGM"));
    assert!(names.contains(&"MSG"));
    assert!(names.contains(&"PRNFILE"));
}

#[test]
fn no_control_word_returns_general_syntax_fallback_list() {
    let result = lookup_keyword(&KeywordLookupInput {
        enclosing_control_word: None,
        spellcheck_token: None,
    });
    let names: Vec<&str> = result.candidates.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"IF"));
    assert!(names.contains(&"ENDIF"));
}

#[test]
fn near_miss_token_yields_a_suggestion_naming_the_real_keyword() {
    let result = lookup_keyword(&KeywordLookupInput {
        enclosing_control_word: None,
        spellcheck_token: Some("PRINT".to_string()),
    });
    assert!(result.spellcheck.unwrap().suggestion.is_some());
}

#[test]
fn exact_match_token_yields_no_suggestion() {
    let result = lookup_keyword(&KeywordLookupInput {
        enclosing_control_word: None,
        spellcheck_token: Some("RUN".to_string()),
    });
    assert_eq!(result.spellcheck.unwrap().suggestion, None);
}
