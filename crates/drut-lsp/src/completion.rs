//! `textDocument/completion` (FR-012–FR-013, `contracts/lsp-capabilities.md`;
//! research.md §2's resolved control-word-scoped mechanism).

use voyager_core::{
    keywords::{self, CompletionContext},
    BlockKind, Node, ParseResult, Position as CorePosition, Statement, StatementKind, TokenKind,
};

use crate::document_store::ServerState;
use crate::position::from_lsp_position;

/// Derives the control word a `Block`'s own opener represents, for
/// completion-scoping purposes — mirrors the literal spelling
/// `statement.rs`'s `FIXED_KEYWORDS`/`block.rs`'s `role_of` recognize.
fn block_control_word(kind: &BlockKind) -> &'static str {
    match kind {
        BlockKind::If { .. } => "IF",
        BlockKind::Loop {} => "LOOP",
        BlockKind::Run { .. } => "RUN", // `!RUN` scopes the same as `RUN`.
        BlockKind::Process { .. } => "PROCESS", // covers the `PHASE=` shortcut too.
        BlockKind::JLoop {} => "JLOOP",
        BlockKind::LinkLoop {} => "LINKLOOP",
        BlockKind::DistributeMultistep { .. } => "DISTRIBUTEMULTISTEP",
    }
}

/// Resolves the control word enclosing `pos`, per research.md §2's
/// mechanism: a span-containment scan over already-parsed structure, no new
/// structural inference. Exposed at module level (not just inlined into
/// [`handle`]) so it's directly unit-testable independent of the keyword
/// dictionary's current content.
fn resolve_enclosing_control_word(parse_result: &ParseResult, pos: CorePosition) -> Option<String> {
    find_in(&parse_result.nodes, pos)
}

fn find_in(nodes: &[Node], pos: CorePosition) -> Option<String> {
    for node in nodes {
        match node {
            Node::Statement(stmt) => {
                if let StatementKind::Control { word, .. } = &stmt.kind {
                    if span_contains_line(stmt, pos) {
                        return Some(word.clone());
                    }
                }
            }
            Node::Block(block) => {
                // Nested content is more specific — check it first.
                if let Some(found) = find_in(&block.children, pos) {
                    return Some(found);
                }
                if let BlockKind::If { branches } = &block.kind {
                    for branch in branches {
                        if let Some(found) = find_in(&branch.children, pos) {
                            return Some(found);
                        }
                    }
                    if branches.iter().any(|b| b.span.start.line == pos.line) {
                        return Some("IF".to_string());
                    }
                } else if block.span.start.line == pos.line {
                    return Some(block_control_word(&block.kind).to_string());
                }
            }
        }
    }
    None
}

fn span_contains_line(stmt: &Statement, pos: CorePosition) -> bool {
    stmt.span.start.line <= pos.line && pos.line <= stmt.span.end.line
}

/// `true` when `pos` falls inside a comment or a quoted string literal
/// (FR-013).
///
/// `voyager-core` has no dedicated `TokenKind` for quoted-string content —
/// `'`/`"` are tokenized as individual `Punctuation` tokens and everything
/// between them is ordinary token content (`crates/voyager-core/src/
/// lexer.rs`), the same way the lexer itself tracks quote state internally
/// just to suppress `;`/`/*` comment-start recognition inside a string. This
/// mirrors that same quote-parity counting over already-tokenized
/// `Punctuation` output (not a new grammar decision, just counting
/// already-classified tokens) rather than re-deriving lexer-level rules,
/// consistent with Principle I.
fn in_comment_or_string(text: &str, pos: CorePosition) -> bool {
    let tokens = voyager_core::tokenize(text);

    let mut in_single_quote = false;
    let mut in_double_quote = false;
    for t in &tokens {
        if t.span.end <= pos {
            match (&t.kind, t.text.as_str()) {
                (TokenKind::Punctuation, "'") => in_single_quote = !in_single_quote,
                (TokenKind::Punctuation, "\"") => in_double_quote = !in_double_quote,
                _ => {}
            }
            continue;
        }
        if t.span.start <= pos && pos <= t.span.end {
            if matches!(t.kind, TokenKind::LineComment | TokenKind::BlockComment { .. }) {
                return true;
            }
            break;
        }
    }
    in_single_quote || in_double_quote
}

/// Handles a `textDocument/completion` request.
pub fn handle(
    state: &ServerState,
    params: &lsp_types::CompletionParams,
) -> Option<lsp_types::CompletionResponse> {
    let uri = &params.text_document_position.text_document.uri;
    let doc = state.get(uri)?;
    let pos = from_lsp_position(&doc.text, params.text_document_position.position);

    if in_comment_or_string(&doc.text, pos) {
        return Some(lsp_types::CompletionResponse::Array(Vec::new()));
    }

    let enclosing = resolve_enclosing_control_word(&doc.parse_result, pos);
    let ctx = CompletionContext {
        enclosing_control_word: enclosing.as_deref(),
    };
    let items = keywords::completion_candidates(ctx)
        .into_iter()
        .map(|entry| lsp_types::CompletionItem {
            label: entry.name.to_string(),
            kind: Some(lsp_types::CompletionItemKind::KEYWORD),
            ..Default::default()
        })
        .collect();

    Some(lsp_types::CompletionResponse::Array(items))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> ParseResult {
        voyager_core::parse(text)
    }

    #[test]
    fn resolves_ordinary_control_statement() {
        let result = parse("PATHLOAD FILE=x.mat VOL=mw[1]\n");
        let pos = CorePosition::new(1, 25); // inside the statement.
        assert_eq!(
            resolve_enclosing_control_word(&result, pos),
            Some("PATHLOAD".to_string())
        );
    }

    #[test]
    fn resolves_block_opener_control_word() {
        let result = parse("RUN PGM=MATRIX\nZONES=5\nENDRUN\n");
        let pos = CorePosition::new(1, 10); // on the RUN line itself.
        assert_eq!(resolve_enclosing_control_word(&result, pos), Some("RUN".to_string()));
    }

    #[test]
    fn resolves_none_before_any_control_word() {
        let result = parse("");
        let pos = CorePosition::new(1, 1);
        assert_eq!(resolve_enclosing_control_word(&result, pos), None);
    }

    #[test]
    fn run_pgm_hwyassign_and_run_pgm_matrix_resolve_identically() {
        // Regression guard (tasks.md T025): completion scoping MUST NOT
        // vary by a control word's PGM= value — both resolve to "RUN".
        let a = parse("RUN PGM=HWYASSIGN\nENDRUN\n");
        let b = parse("RUN PGM=MATRIX\nENDRUN\n");
        let pos = CorePosition::new(1, 5);
        assert_eq!(resolve_enclosing_control_word(&a, pos), Some("RUN".to_string()));
        assert_eq!(resolve_enclosing_control_word(&a, pos), resolve_enclosing_control_word(&b, pos));
    }

    #[test]
    fn in_comment_or_string_detects_comment() {
        let text = "; a comment\nIF (a=b)\nENDIF\n";
        assert!(in_comment_or_string(text, CorePosition::new(1, 5)));
        assert!(!in_comment_or_string(text, CorePosition::new(2, 2)));
    }
}
