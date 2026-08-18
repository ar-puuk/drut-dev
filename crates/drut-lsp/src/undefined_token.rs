//! Finds `@token@` references with no resolvable definition
//! (020-undefined-token-diagnostic, `data-model.md` §2, `research.md` §1-§4)
//! — feeds `diagnostics.rs`'s fourth, Hint-severity, non-`DiagnosticKind`
//! stream. All real resolution logic is `voyager-core`'s own
//! `token_resolution` module (constitution Principle I); this module does
//! only the same adapter-side disk-I/O `hover.rs` already does for the same
//! purpose, reused rather than duplicated.

use voyager_core::{Node, Span, VariableRefAt};

use crate::document_store::OpenDocument;
use crate::hover;

/// Every `@token@` reference in `doc` that the existing resolution logic
/// (same-file assignment, or one level of static `READ FILE` inclusion)
/// cannot resolve. Each of the three documented resolver blind spots
/// (a block-opener reference, more than one level of inclusion, a
/// token-built inclusion path) is excluded automatically by reusing
/// `all_variable_refs`/`collect_included_files`/`resolve_token_value`
/// unmodified — not by any new suppression rule here (research.md §3).
pub fn undefined_token_positions(uri: &lsp_types::Uri, doc: &OpenDocument) -> Vec<VariableRefAt> {
    let included_files = hover::collect_included_files(uri, doc);
    let included: Vec<(Span, Vec<Node>)> = included_files
        .iter()
        .map(|f| (f.read_file_statement_span, f.nodes.clone()))
        .collect();

    voyager_core::all_variable_refs(&doc.parse_result.nodes)
        .into_iter()
        .filter(|var_ref| {
            voyager_core::resolve_token_value(
                &doc.parse_result.nodes,
                var_ref.span.start,
                &included,
                &var_ref.name,
            )
            .is_none()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document_store::ServerState;
    use std::path::PathBuf;
    use std::str::FromStr;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "drut_lsp_undefined_token_test_{}_{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn uri_for(path: &std::path::Path) -> lsp_types::Uri {
        let s = path.to_string_lossy().replace('\\', "/");
        let s = if s.starts_with('/') { s } else { format!("/{s}") };
        lsp_types::Uri::from_str(&format!("file://{s}")).unwrap()
    }

    fn open(state: &mut ServerState, uri: &lsp_types::Uri, text: &str) {
        state.did_open(uri.clone(), text.to_string(), 1);
    }

    #[test]
    fn as1_reference_with_no_assignment_anywhere_is_returned() {
        let mut state = ServerState::new();
        let dir = temp_dir("as1");
        let main_path = dir.join("main.s");
        let text = "MSG = @ScenarioDir@\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = undefined_token_positions(&uri, doc);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "ScenarioDir");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn as2_reference_with_a_same_file_assignment_is_not_returned() {
        let mut state = ServerState::new();
        let dir = temp_dir("as2");
        let main_path = dir.join("main.s");
        // @Prog@ referenced from a plain Assignment value (not a
        // block-opener position, unlike AS3 below) so it's actually visible
        // to all_variable_refs -- this test must exercise real resolution
        // succeeding, not just the block-opener exclusion trivially passing.
        let text = "Prog = MATRIX\nMSG = @Prog@\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = undefined_token_positions(&uri, doc);
        assert!(
            found.iter().all(|r| r.name != "Prog"),
            "Prog resolves via a same-file assignment and must not be flagged: {found:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn as3_block_opener_reference_is_not_returned() {
        let mut state = ServerState::new();
        let dir = temp_dir("as3");
        let main_path = dir.join("main.s");
        // No Prog = ... assignment anywhere -- @Prog@ here is unresolvable,
        // but it sits on a block-opener line, so it's structurally absent
        // from all_variable_refs's own traversal (research.md §3) and must
        // never appear in the result at all.
        let text = "RUN PGM=@Prog@\nENDRUN\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = undefined_token_positions(&uri, doc);
        assert!(
            found.is_empty(),
            "a block-opener @token@ must never be flagged: {found:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn one_level_read_file_inclusion_resolves_and_is_not_returned() {
        let mut state = ServerState::new();
        let dir = temp_dir("one-level");
        std::fs::write(dir.join("sibling.block"), "ScenarioDir = 'C:\\scenario'\n").unwrap();
        let main_path = dir.join("main.s");
        let text = "READ FILE = 'sibling.block'\nMSG = @ScenarioDir@\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = undefined_token_positions(&uri, doc);
        assert!(
            found.iter().all(|r| r.name != "ScenarioDir"),
            "a token resolvable through one level of READ FILE inclusion must not be flagged: {found:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn as4_two_level_read_file_inclusion_is_returned() {
        let mut state = ServerState::new();
        let dir = temp_dir("two-level");
        std::fs::write(dir.join("grandparent.block"), "ParentDir = 'C:\\parent'\n").unwrap();
        std::fs::write(
            dir.join("sibling.block"),
            "READ FILE = 'grandparent.block'\n",
        )
        .unwrap();
        let main_path = dir.join("main.s");
        let text = "READ FILE = 'sibling.block'\nMSG = @ParentDir@\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        // Only one level of inclusion is followed, so ParentDir (defined two
        // levels away) is correctly not resolved -- and therefore correctly
        // flagged, per the existing resolver's own documented boundary.
        let found = undefined_token_positions(&uri, doc);
        assert!(
            found.iter().any(|r| r.name == "ParentDir"),
            "a token resolvable only two levels of READ FILE away must be flagged: {found:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn as5_token_built_read_file_path_is_returned() {
        let mut state = ServerState::new();
        let dir = temp_dir("dynamic-path");
        std::fs::write(dir.join("sibling.block"), "ScenarioDir = 'C:\\scenario'\n").unwrap();
        let main_path = dir.join("main.s");
        let text = "SiblingName = 'sibling.block'\nREAD FILE = '@SiblingName@'\nMSG = @ScenarioDir@\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = undefined_token_positions(&uri, doc);
        assert!(
            found.iter().any(|r| r.name == "ScenarioDir"),
            "a token reachable only through a dynamic READ FILE path must be flagged: {found:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_document_returns_no_positions() {
        let mut state = ServerState::new();
        let dir = temp_dir("empty");
        let main_path = dir.join("main.s");
        let text = "X = 1\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        assert!(undefined_token_positions(&uri, doc).is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn same_unresolvable_name_referenced_twice_is_flagged_both_times() {
        let mut state = ServerState::new();
        let dir = temp_dir("repeated");
        let main_path = dir.join("main.s");
        let text = "MSG1 = @Missing@\nMSG2 = @Missing@\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = undefined_token_positions(&uri, doc);
        let count = found.iter().filter(|r| r.name == "Missing").count();
        assert_eq!(count, 2, "each occurrence must be flagged independently: {found:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
