//! Read-only guarantee (FR-010, SC-005): every tool, called against a
//! fixture file marked read-only for the test's duration, still succeeds —
//! proving no tool attempts a write, not merely documenting that none
//! should. `format` gets the most scrutiny (the one tool most tempting to
//! implement as "format and save").

use drut_mcp::diagnose::{diagnose, DiagnosticsInput};
use drut_mcp::format::{format, FormatInput};
use drut_mcp::lookup_keyword::{lookup_keyword, KeywordLookupInput};
use drut_mcp::query_structure::{query_structure, StructuralQueryInput};
use drut_mcp::source::ScriptSource;

const FIXTURE: &str = "IF (a=b)\nPRINT LIST=1\nENDIF\n";

fn make_readonly_fixture() -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("drut_mcp_no_disk_writes_{}.s", std::process::id()));
    std::fs::write(&path, FIXTURE).unwrap();

    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&path, perms).unwrap();
    path
}

/// Clears the readonly attribute this test itself set, purely so the temp
/// file can be deleted afterward — never a real permissions grant (this is
/// test-cleanup-only code for a single-owner temp file about to be removed,
/// not the world-writable-on-Unix footgun `clippy::permissions_set_readonly_false`
/// exists to catch in real, persistent-file code).
#[allow(clippy::permissions_set_readonly_false)]
fn clear_readonly_and_remove(path: &std::path::Path) {
    if let Ok(metadata) = std::fs::metadata(path) {
        let mut perms = metadata.permissions();
        perms.set_readonly(false);
        let _ = std::fs::set_permissions(path, perms);
    }
    let _ = std::fs::remove_file(path);
}

#[test]
fn every_tool_succeeds_against_a_read_only_fixture_and_leaves_it_unchanged() {
    let path = make_readonly_fixture();
    let path_str = path.to_string_lossy().to_string();
    let before = std::fs::read(&path).unwrap();

    let diag_input = DiagnosticsInput {
        source: ScriptSource {
            text: None,
            path: Some(path_str.clone()),
        },
    };
    let diag_result = diagnose(&diag_input);
    assert!(diag_result.is_ok(), "diagnose should succeed against a read-only file: {diag_result:?}");

    let format_input = FormatInput {
        source: ScriptSource {
            text: None,
            path: Some(path_str.clone()),
        },
        casing: None,
        control_words_casing: None,
        pair_keywords_casing: None,
        data_references_casing: None,
        indent_width: None,
        top_level_indent: None,
        operator_spacing: None,
        isolated: None,
    };
    let format_result = format(&format_input);
    assert!(
        format_result.is_ok(),
        "format should succeed against a read-only file (it returns text, never writes it): {format_result:?}"
    );
    // format's own result reports the *would-be* reformatted text -- it
    // must never have attempted to write that text back to `path` itself.
    assert!(format_result.unwrap().changed, "fixture is deliberately misindented, so a real change is expected in the *returned* text");

    let query_input = StructuralQueryInput {
        source: ScriptSource {
            text: None,
            path: Some(path_str.clone()),
        },
        line: 1,
        column: 2,
    };
    let query_result = query_structure(&query_input);
    assert!(query_result.is_ok(), "query_structure should succeed against a read-only file: {query_result:?}");

    // lookup_keyword takes no ScriptSource at all -- included for
    // completeness (every tool, per T029's own scope), even though it was
    // never going to touch the filesystem in the first place.
    let _ = lookup_keyword(&KeywordLookupInput {
        enclosing_control_word: Some("RUN".to_string()),
        spellcheck_token: None,
    });

    let after = std::fs::read(&path).unwrap();
    assert_eq!(before, after, "the fixture file's own content on disk must be byte-identical after every tool ran");

    clear_readonly_and_remove(&path);
}
