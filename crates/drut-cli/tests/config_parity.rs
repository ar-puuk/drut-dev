//! Three-surface parity for 012-toml-configuration: the same `drut.toml`,
//! governing the same file, must produce byte-identical formatted output
//! whether reached via the CLI, the LSP server, or the MCP `format` tool —
//! spec.md US1's own Independent Test, proven directly rather than
//! inferred from each surface's own separate test coverage passing.

use std::path::Path;
use std::process::Command;
use std::str::FromStr;

use drut_lsp::document_store::ServerState;
use drut_mcp::format::{format as mcp_format, FormatInput};
use drut_mcp::source::ScriptSource;

/// `name` must be unique per test in this file — `cargo test`'s default
/// parallel-threads-in-one-process runner means two tests sharing a PID-only
/// directory name (the original shape here, before this feature added a
/// second test to this file and exposed the collision) can race on the same
/// `drut.toml`/`a.s` files and silently corrupt each other's fixtures.
fn temp_project(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("drut_config_parity_test_{}_{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn file_uri_str(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    let s = if s.starts_with('/') { s } else { format!("/{s}") };
    format!("file://{s}")
}

#[test]
fn cli_lsp_and_mcp_resolve_the_same_drut_toml_identically() {
    let dir = temp_project("legacy_fields");
    std::fs::write(
        dir.join("drut.toml"),
        "[format]\ncasing = \"upper\"\ntop_level_indent = \"normalize\"\n",
    )
    .unwrap();
    let file = dir.join("a.s");
    let source_text = "    if (x=1)\n        y = 2\n    endif\n";
    std::fs::write(&file, source_text).unwrap();

    // (a) CLI, via the real compiled binary -- no flags passed.
    let cli_out = Command::new(env!("CARGO_BIN_EXE_drut"))
        .args(["format", file.to_str().unwrap()])
        .output()
        .expect("failed to run drut");
    assert_eq!(cli_out.status.code(), Some(0));
    let cli_text = String::from_utf8_lossy(&cli_out.stdout).into_owned();

    // (b) LSP, via textDocument/formatting's real handler.
    let uri = lsp_types::Uri::from_str(&file_uri_str(&file)).unwrap();
    let mut state = ServerState::new();
    state.did_open(uri.clone(), source_text.to_string(), 1);
    let params = lsp_types::DocumentFormattingParams {
        text_document: lsp_types::TextDocumentIdentifier { uri },
        options: lsp_types::FormattingOptions {
            tab_size: 4,
            insert_spaces: true,
            ..Default::default()
        },
        work_done_progress_params: Default::default(),
    };
    let edits = drut_lsp::formatting::handle(&state, &params).unwrap();
    assert_eq!(edits.len(), 1, "expected a single whole-document edit, got {edits:?}");
    let lsp_text = edits[0].new_text.clone();

    // (c) MCP, via the format tool's real entry point, path-sourced so
    // discovery actually runs (a text-sourced call would skip it).
    let mcp_result = mcp_format(&FormatInput {
        source: ScriptSource {
            text: None,
            path: Some(file.to_str().unwrap().to_string()),
        },
        casing: None,
        control_words_casing: None,
        pair_keywords_casing: None,
        data_references_casing: None,
        indent_width: None,
        top_level_indent: None,
        operator_spacing: None,
        isolated: None,
    })
    .unwrap();
    let mcp_text = mcp_result.text;

    assert_eq!(cli_text, lsp_text, "CLI and LSP must produce byte-identical output for the same drut.toml");
    assert_eq!(lsp_text, mcp_text, "LSP and MCP must produce byte-identical output for the same drut.toml");
    // Sanity check: this must actually be the config-driven result (casing
    // upper + top_level_indent normalize), not a coincidental three-way
    // match on unrelated/default output.
    assert_eq!(
        cli_text, "IF (x=1)\n    y = 2\nENDIF\n",
        "expected the drut.toml-driven result: top-level lines forced to column 0, IF/ENDIF uppercased"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_lsp_and_mcp_resolve_the_new_granular_settings_identically() {
    // 017-casing-categories-indent-width FR-013: the new data_references
    // casing category and indent_width setting must resolve identically
    // across all three surfaces too, not just the pre-existing fields
    // above -- proven the same direct way, not inferred.
    let dir = temp_project("granular_fields");
    std::fs::write(
        dir.join("drut.toml"),
        "[format]\ndata_references_casing = \"upper\"\nindent_width = 2\n",
    )
    .unwrap();
    let file = dir.join("a.s");
    // "idx", not "i" -- LOOP's variable-name slot is free-form, but "i"
    // specifically is also the reserved outer-zone-loop identifier
    // (research.md §6), so it would legitimately (if surprisingly for this
    // test's purpose) get swept up by data_references casing too.
    let source_text = "if (x=1)\nloop idx=1,5\nmw[1] = mi.1.1\nendloop\nendif\n";
    std::fs::write(&file, source_text).unwrap();

    let cli_out = Command::new(env!("CARGO_BIN_EXE_drut"))
        .args(["format", file.to_str().unwrap()])
        .output()
        .expect("failed to run drut");
    assert_eq!(cli_out.status.code(), Some(0));
    let cli_text = String::from_utf8_lossy(&cli_out.stdout).into_owned();

    let uri = lsp_types::Uri::from_str(&file_uri_str(&file)).unwrap();
    let mut state = ServerState::new();
    state.did_open(uri.clone(), source_text.to_string(), 1);
    let params = lsp_types::DocumentFormattingParams {
        text_document: lsp_types::TextDocumentIdentifier { uri },
        options: lsp_types::FormattingOptions {
            tab_size: 4,
            insert_spaces: true,
            ..Default::default()
        },
        work_done_progress_params: Default::default(),
    };
    let edits = drut_lsp::formatting::handle(&state, &params).unwrap();
    assert_eq!(edits.len(), 1, "expected a single whole-document edit, got {edits:?}");
    let lsp_text = edits[0].new_text.clone();

    let mcp_result = mcp_format(&FormatInput {
        source: ScriptSource {
            text: None,
            path: Some(file.to_str().unwrap().to_string()),
        },
        casing: None,
        control_words_casing: None,
        pair_keywords_casing: None,
        data_references_casing: None,
        indent_width: None,
        top_level_indent: None,
        operator_spacing: None,
        isolated: None,
    })
    .unwrap();
    let mcp_text = mcp_result.text;

    assert_eq!(cli_text, lsp_text, "CLI and LSP must agree on the new granular settings");
    assert_eq!(lsp_text, mcp_text, "LSP and MCP must agree on the new granular settings");
    assert_eq!(
        cli_text, "if (x=1)\n  loop idx=1,5\n    MW[1] = MI.1.1\n  endloop\nendif\n",
        "expected data_references uppercased and 2-space nesting, control_words/pair_keywords left untouched (Preserve)"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
