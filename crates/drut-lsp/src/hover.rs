//! `textDocument/hover` (FR-008–FR-011, `contracts/lsp-capabilities.md`;
//! 016-token-hover-value, `contracts/token-resolution-api.md`).
//!
//! The block-kind/matched-counterpart derivation itself lives in
//! `voyager_core::block_at` (moved there 2026-08-10,
//! `004-mcp-server/research.md` §5, `contracts/block-resolution-api.md`) —
//! this module is now a thin translation from that result into
//! `lsp_types::Hover` markdown, the same shape every other `drut-lsp`
//! handler already has over its own `voyager-core` entry point. The
//! `@token@` value-resolution branch added by 016-token-hover-value follows
//! the identical shape: `voyager_core::token_resolution` does the real
//! analysis (constitution Principle I), this module only translates the
//! result and — the one new piece of adapter-side work — reads a `READ
//! FILE` target off disk when needed (research.md §4).

use voyager_core::{BlockInfo, Node, Span, TokenValueSource};

use crate::document_store::{OpenDocument, ServerState};
use crate::position::{from_lsp_position, text_for_span, to_lsp_range};
use crate::spellcheck;
use crate::workspace;

/// Handles a `textDocument/hover` request (FR-008–FR-011; 016's FR-001–FR-010).
pub fn handle(state: &ServerState, params: &lsp_types::HoverParams) -> Option<lsp_types::Hover> {
    let uri = &params.text_document_position_params.text_document.uri;
    let doc = state.get(uri)?;

    let pos = from_lsp_position(&doc.text, params.text_document_position_params.position);

    // Tried first (016-token-hover-value research.md §6): a `@token@`
    // reference is never itself a block opener/closer, so `block_at` below
    // already returns `None` for it today — this branch changes nothing for
    // any position that isn't over a `@token@` reference (FR-010).
    if let Some(var_ref) = voyager_core::variable_ref_at(&doc.parse_result.nodes, pos) {
        if let Some(hover) = resolve_token_value_hover(uri, doc, pos, &var_ref.name) {
            return Some(hover);
        }
        // No resolvable value (FR-008) — fall through to the unchanged
        // block-info/spell-check chain below, exactly as if this branch
        // didn't exist.
    }

    let Some(fact): Option<BlockInfo> =
        voyager_core::block_at(&doc.parse_result.nodes, &doc.parse_result.diagnostics, pos)
    else {
        // Not a block opener/closer (FR-011) — try a spell-check nudge
        // instead (FR-014, `contracts/lsp-capabilities.md`'s "rides on
        // hover" decision) rather than fabricating block-structure info.
        let hint = spellcheck::hint_for(&doc.text, pos)?;
        return Some(lsp_types::Hover {
            contents: lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
                kind: lsp_types::MarkupKind::Markdown,
                value: format!("Did you mean **{}**?", hint.suggestion),
            }),
            range: None,
        });
    };

    let mut value = format!("**{}**", fact.kind);
    if fact.is_short_if {
        value.push_str(" (self-closing short-IF — no separate closer)"); // FR-010.
    } else if let Some(counterpart) = fact.counterpart {
        let range = to_lsp_range(&doc.text, counterpart);
        value.push_str(&format!(
            " — matched counterpart at line {}",
            range.start.line + 1
        ));
    }

    Some(lsp_types::Hover {
        contents: lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value,
        }),
        range: None,
    })
}

/// One `READ FILE` target successfully read and parsed off disk
/// (016-token-hover-value data-model.md, US2) — keeps the decoded text
/// alongside the parsed nodes (unlike `voyager_core::parse_bytes`, which
/// discards it) so a resolved value's `value_span` can be sliced back into
/// real display text, and keeps the literal path as written for FR-009's
/// "name the source file" requirement.
pub(crate) struct IncludedFile {
    pub(crate) read_file_statement_span: Span,
    pub(crate) display_name: String,
    pub(crate) text: String,
    pub(crate) nodes: Vec<Node>,
}

/// Strips one leading and one trailing quote character from `s`, if both are
/// present and match each other (`'...'` or `"..."`) — otherwise returns `s`
/// unchanged (016-token-hover-value research.md §3, contracts/
/// token-resolution-api.md).
fn strip_matching_quotes(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'\'' || first == b'"') && first == last {
            return &s[1..s.len() - 1];
        }
    }
    s
}

/// Reads and parses every literal-path `READ FILE` target the hovered
/// document directly contains (016-token-hover-value FR-003/FR-006/FR-007) —
/// one level only, never recursing into an included file's own `READ FILE`
/// statements. Any failure at any step for one entry (no real on-disk
/// location for the hovered document itself, e.g. an unsaved/untitled
/// buffer — research.md §7; the target doesn't exist or can't be read; it
/// doesn't parse meaningfully) simply omits that one entry — never an error,
/// never a panic.
pub(crate) fn collect_included_files(uri: &lsp_types::Uri, doc: &OpenDocument) -> Vec<IncludedFile> {
    let Some(base_dir) = workspace::uri_to_path(uri).and_then(|p| p.parent().map(|p| p.to_path_buf())) else {
        return Vec::new();
    };

    voyager_core::read_file_refs(&doc.parse_result.nodes)
        .into_iter()
        .filter_map(|read_ref| {
            let value_span = read_ref.literal_value_span?;
            let raw = text_for_span(&doc.text, value_span);
            let literal_path = strip_matching_quotes(&raw).to_string();
            let target_path = base_dir.join(&literal_path);
            let bytes = std::fs::read(&target_path).ok()?;
            let (text, _decode_diagnostics) = voyager_core::decode::decode_bytes(&bytes);
            let parsed = voyager_core::parse(&text);
            Some(IncludedFile {
                read_file_statement_span: read_ref.statement_span,
                display_name: literal_path,
                text,
                nodes: parsed.nodes,
            })
        })
        .collect()
}

fn resolve_token_value_hover(
    uri: &lsp_types::Uri,
    doc: &OpenDocument,
    pos: voyager_core::Position,
    name: &str,
) -> Option<lsp_types::Hover> {
    let included_files = collect_included_files(uri, doc);
    let included: Vec<(Span, Vec<Node>)> = included_files
        .iter()
        .map(|f| (f.read_file_statement_span, f.nodes.clone()))
        .collect();

    let resolved =
        voyager_core::resolve_token_value(&doc.parse_result.nodes, pos, &included, name)?;

    let (source_text, source_note) = match resolved.source {
        TokenValueSource::SameFile => (&doc.text, String::new()),
        TokenValueSource::ReadFile {
            read_file_statement_span,
        } => {
            let file = included_files
                .iter()
                .find(|f| f.read_file_statement_span == read_file_statement_span)?;
            (&file.text, format!(" (from `{}`)", file.display_name))
        }
    };

    let value_text = text_for_span(source_text, resolved.value_span);
    let assigning_line = to_lsp_range(source_text, resolved.statement_span).start.line + 1;

    Some(lsp_types::Hover {
        contents: lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value: format!(
                "`{name}` = **{value_text}**{source_note} — assigned at line {assigning_line}"
            ),
        }),
        range: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn params(uri: &str, line: u32, character: u32) -> lsp_types::HoverParams {
        lsp_types::HoverParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier {
                    uri: lsp_types::Uri::from_str(uri).unwrap(),
                },
                position: lsp_types::Position::new(line, character),
            },
            work_done_progress_params: Default::default(),
        }
    }

    #[test]
    fn hover_over_if_reports_kind_and_matched_endif() {
        let mut state = ServerState::new();
        state.did_open(
            lsp_types::Uri::from_str("file:///a.s").unwrap(),
            "IF (a=b)\nENDIF\n".to_string(),
            1,
        );
        let result = handle(&state, &params("file:///a.s", 0, 1)).unwrap();
        let lsp_types::HoverContents::Markup(m) = result.contents else {
            panic!("expected markup")
        };
        assert!(m.value.contains("If"));
        assert!(m.value.contains("line 2"));
    }

    #[test]
    fn hover_over_short_if_has_no_separate_closer() {
        let mut state = ServerState::new();
        state.did_open(
            lsp_types::Uri::from_str("file:///a.s").unwrap(),
            "IF (a=b) PRINT LIST=1\n".to_string(),
            1,
        );
        let result = handle(&state, &params("file:///a.s", 0, 1)).unwrap();
        let lsp_types::HoverContents::Markup(m) = result.contents else {
            panic!("expected markup")
        };
        assert!(m.value.contains("short-IF"));
    }

    #[test]
    fn hover_over_implicitly_closed_run_reports_resolved_location() {
        let mut state = ServerState::new();
        state.did_open(
            lsp_types::Uri::from_str("file:///a.s").unwrap(),
            "RUN PGM=MATRIX\nZONES=5\nRUN PGM=HWYASSIGN\nENDRUN\n".to_string(),
            1,
        );
        let result = handle(&state, &params("file:///a.s", 0, 1)).unwrap();
        let lsp_types::HoverContents::Markup(m) = result.contents else {
            panic!("expected markup")
        };
        assert!(m.value.contains("Run"));
        assert!(!m.value.contains("short-IF"));
        // The first RUN's body ends at line 2 (ZONES=5), right before the
        // second RUN implicitly closes it.
        assert!(m.value.contains("line 2"), "value was: {}", m.value);
    }

    #[test]
    fn hover_over_unrelated_token_returns_none() {
        let mut state = ServerState::new();
        // "PRINT" itself is now deliberately avoided here: with the real
        // 2026-08-10 census dictionary populated, "PRINT" is one edit away
        // from the real keyword "PRINTO" (observed under both FILEO and
        // PRINT), so it correctly triggers a spell-check nudge (Story 5) —
        // that's the feature working, not a bug; see hover.rs's dedicated
        // fallback test and drut-lsp/tests/spellcheck.rs. This test uses a
        // token with no plausible dictionary neighbor at all.
        state.did_open(
            lsp_types::Uri::from_str("file:///a.s").unwrap(),
            "IF (a=b)\nXYZZY123NOTHINGLIKEIT LIST=1\nENDIF\n".to_string(),
            1,
        );
        let result = handle(&state, &params("file:///a.s", 1, 1));
        assert!(result.is_none());
    }

    // -- 016-token-hover-value: User Story 1 (same-file) ---------------------

    #[test]
    fn hover_over_same_file_token_shows_its_assigned_value() {
        let mut state = ServerState::new();
        state.did_open(
            lsp_types::Uri::from_str("file:///a.s").unwrap(),
            "ZoneMsgRate = 50\nPRINT LIST='@ZoneMsgRate@'\n".to_string(),
            1,
        );
        // Line 2 (0-based line 1), inside "@ZoneMsgRate@".
        let result = handle(&state, &params("file:///a.s", 1, 14)).unwrap();
        let lsp_types::HoverContents::Markup(m) = result.contents else {
            panic!("expected markup")
        };
        assert!(m.value.contains("50"), "value was: {}", m.value);
        assert!(m.value.contains("line 1"), "value was: {}", m.value);
    }

    #[test]
    fn hover_shows_the_reassignment_closest_to_the_reference_not_the_first() {
        let mut state = ServerState::new();
        state.did_open(
            lsp_types::Uri::from_str("file:///a.s").unwrap(),
            "ZoneMsgRate = 50\nZoneMsgRate = 60\nPRINT LIST='@ZoneMsgRate@'\n".to_string(),
            1,
        );
        let result = handle(&state, &params("file:///a.s", 2, 14)).unwrap();
        let lsp_types::HoverContents::Markup(m) = result.contents else {
            panic!("expected markup")
        };
        assert!(m.value.contains("60"), "value was: {}", m.value);
        assert!(!m.value.contains("50"), "value was: {}", m.value);
    }

    #[test]
    fn hover_never_uses_an_assignment_that_comes_after_the_reference() {
        let mut state = ServerState::new();
        state.did_open(
            lsp_types::Uri::from_str("file:///a.s").unwrap(),
            "PRINT LIST='@ZoneMsgRate@'\nZoneMsgRate = 50\n".to_string(),
            1,
        );
        let result = handle(&state, &params("file:///a.s", 0, 14));
        // No same-file assignment exists before the reference, and there's
        // no plausible spell-check nudge either (real keyword, just unset) —
        // falls all the way through to no hover, exactly FR-008.
        assert!(result.is_none());
    }

    // -- 016-token-hover-value: User Story 2 (one-level READ FILE) -----------

    fn file_uri(path: &std::path::Path) -> lsp_types::Uri {
        let s = path.to_string_lossy().replace('\\', "/");
        let s = if s.starts_with('/') { s } else { format!("/{s}") };
        lsp_types::Uri::from_str(&format!("file://{s}")).unwrap()
    }

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("drut_lsp_hover_test_{}_{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn hover_resolves_a_value_from_a_directly_read_sibling_file() {
        let dir = temp_dir("sibling");
        std::fs::write(dir.join("sibling.block"), "UsedZones = 3629\n").unwrap();
        let main_path = dir.join("main.s");
        let main_text = "READ FILE = 'sibling.block'\nPRINT LIST='@UsedZones@'\n";
        std::fs::write(&main_path, main_text).unwrap();

        let mut state = ServerState::new();
        state.did_open(file_uri(&main_path), main_text.to_string(), 1);
        let result = handle(&state, &lsp_types::HoverParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: file_uri(&main_path) },
                position: lsp_types::Position::new(1, 14),
            },
            work_done_progress_params: Default::default(),
        })
        .unwrap();
        let lsp_types::HoverContents::Markup(m) = result.contents else {
            panic!("expected markup")
        };
        assert!(m.value.contains("3629"), "value was: {}", m.value);
        assert!(m.value.contains("sibling.block"), "value was: {}", m.value);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn hover_falls_back_gracefully_when_read_file_target_is_missing() {
        let dir = temp_dir("missing");
        let main_path = dir.join("main.s");
        let main_text = "READ FILE = 'missing.block'\nPRINT LIST='@UsedZones@'\n";
        std::fs::write(&main_path, main_text).unwrap();

        let mut state = ServerState::new();
        state.did_open(file_uri(&main_path), main_text.to_string(), 1);
        let result = handle(&state, &lsp_types::HoverParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: file_uri(&main_path) },
                position: lsp_types::Position::new(1, 14),
            },
            work_done_progress_params: Default::default(),
        });
        assert!(result.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn hover_does_not_resolve_through_a_token_built_read_file_path() {
        let dir = temp_dir("dynamic");
        let main_path = dir.join("main.s");
        let main_text =
            "ZoneMsgRate = 50\nREAD FILE = '@ParentDir@sub\\path.block'\nPRINT LIST='@ZoneMsgRate@ @UsedZones@'\n";
        std::fs::write(&main_path, main_text).unwrap();

        let mut state = ServerState::new();
        state.did_open(file_uri(&main_path), main_text.to_string(), 1);
        // @ZoneMsgRate@ (same-file) still resolves normally.
        let ok = handle(&state, &lsp_types::HoverParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: file_uri(&main_path) },
                position: lsp_types::Position::new(2, 14),
            },
            work_done_progress_params: Default::default(),
        })
        .unwrap();
        let lsp_types::HoverContents::Markup(m) = ok.contents else {
            panic!("expected markup")
        };
        assert!(m.value.contains("50"), "value was: {}", m.value);

        // @UsedZones@ (only ever set behind the dynamic path) does not.
        let none = handle(&state, &lsp_types::HoverParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: file_uri(&main_path) },
                position: lsp_types::Position::new(2, 27),
            },
            work_done_progress_params: Default::default(),
        });
        assert!(none.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn hover_prefers_a_same_file_reassignment_after_the_read_file_line() {
        let dir = temp_dir("override");
        std::fs::write(dir.join("sibling.block"), "UsedZones = 3629\n").unwrap();
        let main_path = dir.join("main.s");
        let main_text =
            "READ FILE = 'sibling.block'\nUsedZones = 1\nPRINT LIST='@UsedZones@'\n";
        std::fs::write(&main_path, main_text).unwrap();

        let mut state = ServerState::new();
        state.did_open(file_uri(&main_path), main_text.to_string(), 1);
        let result = handle(&state, &lsp_types::HoverParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: file_uri(&main_path) },
                position: lsp_types::Position::new(2, 14),
            },
            work_done_progress_params: Default::default(),
        })
        .unwrap();
        let lsp_types::HoverContents::Markup(m) = result.contents else {
            panic!("expected markup")
        };
        assert!(m.value.contains('1'), "value was: {}", m.value);
        assert!(!m.value.contains("3629"), "value was: {}", m.value);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn hover_on_an_untitled_buffer_still_resolves_same_file_values() {
        let mut state = ServerState::new();
        state.did_open(
            lsp_types::Uri::from_str("untitled:Untitled-1").unwrap(),
            "READ FILE = 'sibling.block'\nZoneMsgRate = 50\nPRINT LIST='@ZoneMsgRate@'\n"
                .to_string(),
            1,
        );
        let result = handle(&state, &params("untitled:Untitled-1", 2, 14)).unwrap();
        let lsp_types::HoverContents::Markup(m) = result.contents else {
            panic!("expected markup")
        };
        assert!(m.value.contains("50"), "value was: {}", m.value);
    }

    // -- 016-token-hover-value: User Story 3 (no fabricated value) -----------

    #[test]
    fn hover_over_never_assigned_token_shows_no_value() {
        let mut state = ServerState::new();
        state.did_open(
            lsp_types::Uri::from_str("file:///a.s").unwrap(),
            "PRINT LIST='@Nope@'\n".to_string(),
            1,
        );
        let result = handle(&state, &params("file:///a.s", 0, 14));
        assert!(result.is_none());
    }

    #[test]
    fn hover_over_near_miss_name_does_not_show_the_close_matchs_value() {
        let mut state = ServerState::new();
        state.did_open(
            lsp_types::Uri::from_str("file:///a.s").unwrap(),
            "ZoneMsgRate = 50\nPRINT LIST='@ZoneMsgRat@'\n".to_string(),
            1,
        );
        let result = handle(&state, &params("file:///a.s", 1, 14));
        match result {
            None => {}
            Some(lsp_types::Hover {
                contents: lsp_types::HoverContents::Markup(m),
                ..
            }) => {
                assert!(
                    !m.value.contains('5') || !m.value.contains('0'),
                    "must not fabricate the near-match's value: {}",
                    m.value
                );
            }
            _ => panic!("expected markup or none"),
        }
    }
}
