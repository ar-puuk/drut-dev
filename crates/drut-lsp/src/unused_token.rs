//! Finds `Assignment` statements whose target name is never referenced via
//! `@name@` anywhere in scope (029-unused-token-diagnostic, `data-model.md`
//! §2, `research.md` §1-§4) — feeds `diagnostics.rs`'s fifth, Hint-severity,
//! non-`DiagnosticKind` stream. All real resolution logic is `voyager-core`'s
//! own `token_resolution` module (constitution Principle I); this module
//! does only the same adapter-side disk-I/O `hover.rs`/`undefined_token.rs`
//! already do for the same purpose, reused rather than duplicated.

use std::collections::HashSet;

use voyager_core::{Span, VariableRefAt};

use crate::document_store::OpenDocument;
use crate::hover;

/// One `Assignment` statement whose target has no `@name@` reference
/// anywhere in scope.
pub struct UnusedAssignment {
    pub target: String,
    pub statement_span: Span,
}

fn referenced_names(nodes: &[voyager_core::Node]) -> impl Iterator<Item = String> + '_ {
    voyager_core::all_variable_refs_including_openers(nodes)
        .into_iter()
        .map(|r: VariableRefAt| r.name.to_ascii_uppercase())
}

/// Every `Assignment` in `doc` whose target name has no `@name@` reference
/// anywhere in scope: same file (including block-opener positions, FR-003),
/// plus one level of directly-included, statically-resolvable `READ FILE`
/// files. Every dead assignment site is returned independently
/// (Clarification Q1) — no dedup to one-per-name. Applies unconditionally,
/// regardless of whether `doc` itself participates in any `READ FILE`
/// relationship (Clarification Q2).
pub fn unused_token_assignments(uri: &lsp_types::Uri, doc: &OpenDocument) -> Vec<UnusedAssignment> {
    let mut referenced: HashSet<String> = referenced_names(&doc.parse_result.nodes).collect();

    for included in hover::collect_included_files(uri, doc) {
        referenced.extend(referenced_names(&included.nodes));
    }

    voyager_core::all_assignments(&doc.parse_result.nodes)
        .into_iter()
        .filter(|a| !referenced.contains(&a.target.to_ascii_uppercase()))
        .map(|a| UnusedAssignment {
            target: a.target.to_string(),
            statement_span: a.statement_span,
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
            "drut_lsp_unused_token_test_{}_{name}",
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
    fn as1_assignment_with_no_reference_anywhere_is_returned() {
        let mut state = ServerState::new();
        let dir = temp_dir("as1");
        let main_path = dir.join("main.s");
        let text = "ScenarioDir = 'X:\\model'\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = unused_token_assignments(&uri, doc);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].target, "ScenarioDir");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn as2_assignment_referenced_in_same_file_is_not_returned() {
        let mut state = ServerState::new();
        let dir = temp_dir("as2");
        let main_path = dir.join("main.s");
        let text = "Prog = MATRIX\nMSG = @Prog@\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = unused_token_assignments(&uri, doc);
        assert!(
            found.iter().all(|a| a.target != "Prog"),
            "Prog is referenced in this same file and must not be flagged: {}",
            found.len()
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn as2_assignment_referenced_only_on_block_opener_line_is_not_returned() {
        // The correctness fix this feature makes: Prog is used only in
        // RUN PGM=@Prog@'s value position, which all_variable_refs_including_openers
        // (unlike all_variable_refs) can see.
        let mut state = ServerState::new();
        let dir = temp_dir("as2-opener");
        let main_path = dir.join("main.s");
        let text = "Prog = MATRIX\nRUN PGM=@Prog@\nENDRUN\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = unused_token_assignments(&uri, doc);
        assert!(
            found.iter().all(|a| a.target != "Prog"),
            "Prog is referenced on a block-opener line and must not be flagged"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn as3_reassigned_twice_with_one_reference_after_both_is_not_returned() {
        let mut state = ServerState::new();
        let dir = temp_dir("as3");
        let main_path = dir.join("main.s");
        let text = "ScenarioDir = 'X:\\old'\nScenarioDir = 'X:\\new'\nMSG = @ScenarioDir@\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = unused_token_assignments(&uri, doc);
        assert!(
            found.iter().all(|a| a.target != "ScenarioDir"),
            "ScenarioDir has a genuine use, so neither assignment site should be flagged"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn as4_no_references_and_no_read_file_returns_the_assignment() {
        let mut state = ServerState::new();
        let dir = temp_dir("as4");
        let main_path = dir.join("main.s");
        let text = "ScenarioDir = 'X:\\model'\nPRINT LIST='hello'\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = unused_token_assignments(&uri, doc);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].target, "ScenarioDir");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn as5_reassigned_twice_with_zero_references_flags_both_sites() {
        let mut state = ServerState::new();
        let dir = temp_dir("as5");
        let main_path = dir.join("main.s");
        let text = "ScenarioDir = 'X:\\old'\nScenarioDir = 'X:\\new'\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = unused_token_assignments(&uri, doc);
        let count = found.iter().filter(|a| a.target == "ScenarioDir").count();
        assert_eq!(count, 2, "every dead assignment site must be flagged independently: found {count}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn as6_unreferenced_assignment_in_a_file_with_read_file_is_still_returned() {
        let mut state = ServerState::new();
        let dir = temp_dir("as6");
        std::fs::write(dir.join("sibling.block"), "PRINT LIST='hello'\n").unwrap();
        let main_path = dir.join("main.s");
        let text = "READ FILE = 'sibling.block'\nScenarioDir = 'X:\\model'\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = unused_token_assignments(&uri, doc);
        assert!(
            found.iter().any(|a| a.target == "ScenarioDir"),
            "the check must not suppress itself for a file with a READ FILE statement"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn positive_one_level_read_file_inclusion_suppresses_the_notice() {
        let mut state = ServerState::new();
        let dir = temp_dir("one-level");
        std::fs::write(dir.join("sibling.block"), "MSG = @ScenarioDir@\n").unwrap();
        let main_path = dir.join("main.s");
        let text = "READ FILE = 'sibling.block'\nScenarioDir = 'X:\\model'\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        let found = unused_token_assignments(&uri, doc);
        assert!(
            found.iter().all(|a| a.target != "ScenarioDir"),
            "ScenarioDir is referenced in a directly-included file and must not be flagged"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_document_returns_no_assignments() {
        let mut state = ServerState::new();
        let dir = temp_dir("empty");
        let main_path = dir.join("main.s");
        let text = "PRINT LIST='hello'\n";
        std::fs::write(&main_path, text).unwrap();
        let uri = uri_for(&main_path);
        open(&mut state, &uri, text);
        let doc = state.get(&uri).unwrap();

        assert!(unused_token_assignments(&uri, doc).is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
