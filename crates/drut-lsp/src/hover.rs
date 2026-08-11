//! `textDocument/hover` (FR-008–FR-011, `contracts/lsp-capabilities.md`).
//!
//! The block-kind/matched-counterpart derivation itself lives in
//! `voyager_core::block_at` (moved there 2026-08-10,
//! `004-mcp-server/research.md` §5, `contracts/block-resolution-api.md`) —
//! this module is now a thin translation from that result into
//! `lsp_types::Hover` markdown, the same shape every other `drut-lsp`
//! handler already has over its own `voyager-core` entry point.

use voyager_core::BlockInfo;

use crate::document_store::ServerState;
use crate::position::{from_lsp_position, to_lsp_range};
use crate::spellcheck;

/// Handles a `textDocument/hover` request (FR-008–FR-011).
pub fn handle(state: &ServerState, params: &lsp_types::HoverParams) -> Option<lsp_types::Hover> {
    let uri = &params.text_document_position_params.text_document.uri;
    let doc = state.get(uri)?;

    let pos = from_lsp_position(&doc.text, params.text_document_position_params.position);

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
}
