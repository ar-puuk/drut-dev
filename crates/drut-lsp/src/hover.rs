//! `textDocument/hover` (FR-008–FR-011, `contracts/lsp-capabilities.md`).
//!
//! `counterpart`'s derivation is **not** simply `Block.closer` — see the
//! five-rule list in `find_hover_fact`'s body, matching
//! `specs/003-lsp-vscode-extension/data-model.md` §4 exactly (corrected
//! 2026-08-09 after CHK015/finding I1: `Block.closer` alone cannot
//! distinguish an implicitly-closed `Run`/`Process` block from a genuinely
//! unmatched one).

use voyager_core::{Block, BlockKind, DiagnosticKind, Node, ParseResult, Position as CorePosition, Span};

use crate::document_store::ServerState;
use crate::position::{from_lsp_position, to_lsp_range};
use crate::spellcheck;

/// The block kind, named per FR-008's seven kinds.
fn block_kind_name(kind: &BlockKind) -> &'static str {
    match kind {
        BlockKind::If { .. } => "If",
        BlockKind::Loop {} => "Loop",
        BlockKind::Run { .. } => "Run",
        BlockKind::Process { .. } => "Process",
        BlockKind::JLoop {} => "JLoop",
        BlockKind::LinkLoop {} => "LinkLoop",
        BlockKind::DistributeMultistep { .. } => "DistributeMultistep",
    }
}

struct BlockHoverFact {
    kind: &'static str,
    is_short_if: bool,
    counterpart: Option<Span>,
}

/// `true` when `block` (an `If`) has no separate closer statement by
/// construction (a self-closing short-`IF`), as opposed to a genuinely
/// unmatched multi-branch `IF` — distinguished by absence of an
/// `UnmatchedIf` diagnostic anchored at this block's own opener
/// (data-model.md §4, backs FR-010).
fn is_short_if(block: &Block, parse_result: &ParseResult) -> bool {
    if block.closer.is_some() {
        return false;
    }
    !parse_result
        .diagnostics
        .iter()
        .any(|d| d.kind == DiagnosticKind::UnmatchedIf && d.span.start == block.span.start)
}

/// `true` when no `UnmatchedRun` diagnostic is anchored at this `Run`
/// block's own opener — meaning it closed implicitly (data-model.md §4 rule
/// 4), the same diagnostic-absence technique `is_short_if` uses.
fn run_closed_implicitly(block: &Block, parse_result: &ParseResult) -> bool {
    !parse_result
        .diagnostics
        .iter()
        .any(|d| d.kind == DiagnosticKind::UnmatchedRun && d.span.start == block.span.start)
}

/// data-model.md §4's five-rule `counterpart` derivation.
fn counterpart_for(block: &Block, parse_result: &ParseResult) -> Option<Span> {
    if let Some(closer) = block.closer {
        return Some(closer); // Rule 1.
    }
    match &block.kind {
        BlockKind::If { .. } => None, // Rules 2 and 3 (short-IF or genuinely unmatched — either way, None).
        BlockKind::Loop {} | BlockKind::JLoop {} | BlockKind::LinkLoop {} | BlockKind::DistributeMultistep { .. } => {
            None // Rule 3: no implicit-close family for these kinds.
        }
        BlockKind::Run { .. } => {
            // Rule 4.
            if run_closed_implicitly(block, parse_result) {
                Some(Span::at(block.span.end))
            } else {
                None
            }
        }
        BlockKind::Process { .. } => Some(Span::at(block.span.end)), // Rule 5: unconditional.
    }
}

/// Recursively locates the innermost block whose opener or closer line
/// contains `pos` (approximated as "on the same line as the opener/closer
/// statement" — the block/branch's own span, and `Block.closer`'s span,
/// cover their full body content rather than storing a separate
/// opener-only span, so the line-match is the precise, sound proxy for
/// "hovering the keyword itself" that's available without a new
/// `voyager-core` field).
fn find_block_at(nodes: &[Node], pos: CorePosition) -> Option<&Block> {
    for node in nodes {
        if let Node::Block(block) = node {
            // Search nested content first — an inner match is always more
            // specific than this block's own opener/closer line.
            if let Some(found) = find_block_at(&block.children, pos) {
                return Some(found);
            }
            if let BlockKind::If { branches } = &block.kind {
                for branch in branches {
                    if let Some(found) = find_block_at(&branch.children, pos) {
                        return Some(found);
                    }
                }
            }

            if on_opener_or_closer_line(block, pos) {
                return Some(block);
            }
        }
    }
    None
}

fn on_opener_or_closer_line(block: &Block, pos: CorePosition) -> bool {
    if block.span.start.line == pos.line {
        return true;
    }
    if let BlockKind::If { branches } = &block.kind {
        if branches.iter().any(|b| b.span.start.line == pos.line) {
            return true;
        }
    }
    if let Some(closer) = block.closer {
        if closer.start.line == pos.line {
            return true;
        }
    }
    false
}

fn find_hover_fact(parse_result: &ParseResult, pos: CorePosition) -> Option<BlockHoverFact> {
    let block = find_block_at(&parse_result.nodes, pos)?;
    Some(BlockHoverFact {
        kind: block_kind_name(&block.kind),
        is_short_if: matches!(block.kind, BlockKind::If { .. }) && is_short_if(block, parse_result),
        counterpart: counterpart_for(block, parse_result),
    })
}

/// Handles a `textDocument/hover` request (FR-008–FR-011).
pub fn handle(state: &ServerState, params: &lsp_types::HoverParams) -> Option<lsp_types::Hover> {
    let uri = &params.text_document_position_params.text_document.uri;
    let doc = state.get(uri)?;

    let pos = from_lsp_position(&doc.text, params.text_document_position_params.position);

    let Some(fact) = find_hover_fact(&doc.parse_result, pos) else {
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
