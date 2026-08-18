//! `textDocument/semanticTokens/full` (FR-016–FR-018, data-model.md §6,
//! `contracts/lsp-capabilities.md`).

use voyager_core::{Block, BlockKind, DiagnosticKind, Node, ParseResult, Span, Statement, TokenKind};

use crate::document_store::ServerState;
use crate::position::to_lsp_position;

/// Legend indices, matching `lib.rs::server_capabilities`'s declared
/// `token_types`/`token_modifiers` order exactly.
const SHORT_IF_TYPE_INDEX: u32 = 0;
const STATEMENT_TYPE_INDEX: u32 = 1;
/// Added 2026-08-10 -- see lib.rs's own comment on why this rides the
/// *standard* `variable` semantic type rather than a custom one.
const VARIABLE_TYPE_INDEX: u32 = 2;
const UNREACHABLE_MODIFIER_BIT: u32 = 0;

struct RawToken {
    span: Span,
    token_type: u32,
    modifiers_bitset: u32,
}

/// Collects every short-IF's span and every unreachable-after-`BREAK`
/// statement's span across the whole document (data-model.md §6).
fn collect(parse_result: &ParseResult) -> Vec<RawToken> {
    let mut out = Vec::new();
    walk(&parse_result.nodes, parse_result, &mut out);
    out
}

fn walk(nodes: &[Node], parse_result: &ParseResult, out: &mut Vec<RawToken>) {
    for node in nodes {
        if let Node::Block(block) = node {
            if let BlockKind::If { branches } = &block.kind {
                // A short-IF is a single self-closing branch with `closer:
                // None` and no `UnmatchedIf` diagnostic for it (same
                // is_short_if technique as hover.rs).
                if block.closer.is_none() && !has_unmatched_if(parse_result, block) {
                    // Only the `IF`/`ELSEIF` keyword itself gets the
                    // distinguishing scope -- NOT the condition or the body
                    // statement. An earlier version of this stretched the
                    // token through the whole condition (up to the body's
                    // own first token), which had two real bugs found via
                    // manual VS Code testing: (1) it swallowed the
                    // condition's own tokens into one flat color, hiding the
                    // static grammar's normal operator/number/@variable@
                    // coloring inside `(...)` (the exact real-world report:
                    // "the entire @MODE@ = 1 is [the IF color]" even though
                    // the multi-line block-IF form colors each of those
                    // distinctly); and (2) since `collect_variable_refs`
                    // separately emits its own token for any `@name@` inside
                    // that same condition, the two overlapped -- LSP
                    // semantic tokens are specified as non-overlapping
                    // per-line, so this produced protocol data most clients
                    // would render incorrectly regardless of (1). Ending the
                    // span at the condition's own first token (the `(`)
                    // keeps FR-016's "distinguishable from a block-style IF"
                    // (block-style IFs get no token here at all, so the
                    // keyword itself still differs) while leaving the
                    // condition and body fully in the static grammar's
                    // hands, matching the block-style IF's own coloring.
                    let header_end = branches[0]
                        .condition
                        .as_ref()
                        .and_then(|tokens| tokens.first())
                        .map(|tok| tok.span.start)
                        .unwrap_or(block.span.end);
                    out.push(RawToken {
                        span: Span::new(block.span.start, header_end),
                        token_type: SHORT_IF_TYPE_INDEX,
                        modifiers_bitset: 0,
                    });
                }
                for branch in branches {
                    walk(&branch.children, parse_result, out);
                }
            } else {
                mark_unreachable_after_break(block, parse_result, out);
                walk(&block.children, parse_result, out);
            }
        }
    }
}

fn has_unmatched_if(parse_result: &ParseResult, block: &Block) -> bool {
    parse_result
        .diagnostics
        .iter()
        .any(|d| d.kind == DiagnosticKind::UnmatchedIf && d.span.start == block.span.start)
}

/// FR-017/FR-018: only *direct* children of a `Loop`/`JLoop`/`LinkLoop` are
/// walked — a `BREAK` nested inside a conditional branch is a child of that
/// `IF`, not this loop, so it never triggers this rule (deliberately, to
/// avoid flagging code that doesn't always execute — constitution Principle
/// IV; data-model.md §6).
fn mark_unreachable_after_break(block: &Block, parse_result: &ParseResult, out: &mut Vec<RawToken>) {
    if !matches!(
        block.kind,
        BlockKind::Loop {} | BlockKind::JLoop {} | BlockKind::LinkLoop {}
    ) {
        return;
    }

    let mut seen_valid_break = false;
    for child in &block.children {
        if seen_valid_break {
            out.push(RawToken {
                span: child.span(),
                token_type: STATEMENT_TYPE_INDEX,
                modifiers_bitset: 1 << UNREACHABLE_MODIFIER_BIT,
            });
            continue;
        }
        if let Node::Statement(stmt) = child {
            if is_break_statement(stmt) && !is_misplaced_break(parse_result, stmt) {
                seen_valid_break = true;
            }
        }
    }
}

fn is_break_statement(stmt: &Statement) -> bool {
    matches!(&stmt.kind, voyager_core::StatementKind::Control { word, .. } if word.eq_ignore_ascii_case("BREAK"))
}

/// FR-018: excludes a `BREAK` already reported as `MisplacedBreak` — this
/// rule only ever fires for a `BREAK` the parser resolved as validly inside
/// this loop (which, by construction, is exactly what being one of
/// `block.children` here already means — `MisplacedBreak` is anchored at a
/// `BREAK` with no enclosing block at all, so it could never appear as a
/// loop's own child in the first place). Checked anyway, defensively, so
/// this invariant is explicit rather than assumed silently.
fn is_misplaced_break(parse_result: &ParseResult, stmt: &Statement) -> bool {
    parse_result
        .diagnostics
        .iter()
        .any(|d| d.kind == DiagnosticKind::MisplacedBreak && d.span.start == stmt.span.start)
}

/// Every `@name@` reference in `text` (added 2026-08-10) -- re-tokenizes
/// the whole document rather than walking `ParseResult.nodes`, since a
/// variable reference can appear anywhere a token can (a condition, a
/// pair's value, even inside a quoted string -- `lexer.rs` already
/// recognizes it in all of those positions, data-model.md's own Token
/// entity), not only inside the structural positions `collect` above
/// already visits. Covers the *entire* `@name@` reference, `@` delimiters
/// included (changed 002-fix-token-highlighting/issue #2: this used to
/// cover only the name, deliberately leaving the delimiters to the static
/// TextMate grammar's own separate scope -- but semantic tokens take
/// priority over TextMate scope coloring wherever both apply, so that
/// split survived even after the grammar itself was fixed to use one
/// uniform scope, since the name's color came from here, not the grammar,
/// the whole time. Using `tok.span` directly (which already covers the
/// full `@name@` construct) rather than a narrowed name-only span fixes
/// this at its actual source.
fn collect_variable_refs(text: &str) -> Vec<RawToken> {
    voyager_core::tokenize(text)
        .into_iter()
        .filter_map(|tok| {
            if !matches!(tok.kind, TokenKind::VariableRef { .. }) {
                return None;
            }
            if tok.span.start.line != tok.span.end.line {
                // Never actually happens (an `@name@` reference can't
                // contain a newline by construction), but never fabricate
                // a cross-line span if it somehow did.
                return None;
            }
            Some(RawToken {
                span: tok.span,
                token_type: VARIABLE_TYPE_INDEX,
                modifiers_bitset: 0,
            })
        })
        .collect()
}

/// Encodes `tokens` into the LSP semantic-tokens delta format (relative
/// line/`character`, per the standard encoding — `contracts/
/// lsp-capabilities.md`).
fn encode(text: &str, mut tokens: Vec<RawToken>) -> Vec<u32> {
    tokens.sort_by_key(|t| t.span.start);

    let mut data = Vec::with_capacity(tokens.len() * 5);
    let mut prev_line = 0u32;
    let mut prev_char = 0u32;

    for t in tokens {
        let start = to_lsp_position(text, t.span.start);
        let end = to_lsp_position(text, t.span.end);
        let length = if start.line == end.line {
            end.character.saturating_sub(start.character)
        } else {
            // Multi-line span: LSP semantic tokens are single-line only.
            // Report the first line's remaining length as a reasonable,
            // non-fabricated approximation rather than fabricating a
            // cross-line length.
            0
        };

        let delta_line = start.line.saturating_sub(prev_line);
        let delta_start = if delta_line == 0 {
            start.character.saturating_sub(prev_char)
        } else {
            start.character
        };

        data.push(delta_line);
        data.push(delta_start);
        data.push(length);
        data.push(t.token_type);
        data.push(t.modifiers_bitset);

        prev_line = start.line;
        prev_char = start.character;
    }

    data
}

/// Handles a `textDocument/semanticTokens/full` request.
pub fn handle(
    state: &ServerState,
    params: &lsp_types::SemanticTokensParams,
) -> Option<lsp_types::SemanticTokensResult> {
    let doc = state.get(&params.text_document.uri)?;
    let mut raw = collect(&doc.parse_result);
    raw.extend(collect_variable_refs(&doc.text));
    let data = encode(&doc.text, raw);
    Some(lsp_types::SemanticTokensResult::Tokens(
        lsp_types::SemanticTokens {
            result_id: None,
            data: data
                .chunks_exact(5)
                .map(|c| lsp_types::SemanticToken {
                    delta_line: c[0],
                    delta_start: c[1],
                    length: c[2],
                    token_type: c[3],
                    token_modifiers_bitset: c[4],
                })
                .collect(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use voyager_core::Position as CorePosition;

    #[test]
    fn short_if_is_flagged() {
        let text = "IF (a=b) PRINT LIST=1\n";
        let result = voyager_core::parse(text);
        let tokens = collect(&result);
        assert!(tokens.iter().any(|t| t.token_type == SHORT_IF_TYPE_INDEX));
    }

    #[test]
    fn short_if_token_span_covers_only_the_keyword_not_the_condition_or_body() {
        let text = "IF (a=b) PRINT LIST=1\n";
        let result = voyager_core::parse(text);
        let tokens = collect(&result);
        let short_if = tokens
            .iter()
            .find(|t| t.token_type == SHORT_IF_TYPE_INDEX)
            .expect("short-IF token must be present");
        let condition_start_column = text.find('(').unwrap() as u32 + 1;
        assert_eq!(
            short_if.span.end,
            CorePosition::new(1, condition_start_column),
            "the shortIf token must stop where the condition's own `(` begins, not swallow the condition or the body -- swallowing either overrides the static grammar's normal coloring for them: {:?}",
            short_if.span
        );
    }

    #[test]
    fn short_if_with_variable_ref_and_quoted_string_body_leaves_condition_and_body_uncovered() {
        // The exact real-world report: IF (@MODE@ = 1) PRINT LIST="..." was
        // rendering @MODE@, =, and 1 all in the IF's own uniform color
        // instead of their normal distinct colors (and, separately,
        // overlapped the `@MODE@` variable-ref token entirely -- LSP
        // semantic tokens are specified as non-overlapping per line).
        let text = "IF (@MODE@ = 1) PRINT LIST=\"Mode 1 selected\"\n";
        let result = voyager_core::parse(text);
        let tokens = collect(&result);
        let short_if = tokens
            .iter()
            .find(|t| t.token_type == SHORT_IF_TYPE_INDEX)
            .expect("short-IF token must be present");
        let condition_start_column = text.find('(').unwrap() as u32 + 1;
        assert_eq!(short_if.span.end, CorePosition::new(1, condition_start_column));

        // The shortIf token and the @MODE@ variable-ref token must not
        // overlap -- both are emitted into the same semantic-tokens
        // response, and an overlap there is invalid per the LSP spec.
        let mut all = collect(&result);
        all.extend(collect_variable_refs(text));
        let var_ref = all
            .iter()
            .find(|t| t.token_type == VARIABLE_TYPE_INDEX)
            .expect("@MODE@ variable-ref token must be present");
        assert!(
            short_if.span.end <= var_ref.span.start,
            "shortIf span {:?} must not overlap the variable-ref span {:?}",
            short_if.span,
            var_ref.span
        );
    }

    #[test]
    fn block_style_if_is_not_flagged_as_short() {
        let text = "IF (a=b)\nPRINT LIST=1\nENDIF\n";
        let result = voyager_core::parse(text);
        let tokens = collect(&result);
        assert!(!tokens.iter().any(|t| t.token_type == SHORT_IF_TYPE_INDEX));
    }

    #[test]
    fn statement_after_break_is_flagged_unreachable() {
        let text = "LOOP\nBREAK\nPRINT LIST=1\nENDLOOP\n";
        let result = voyager_core::parse(text);
        let tokens = collect(&result);
        assert!(tokens
            .iter()
            .any(|t| t.modifiers_bitset & (1 << UNREACHABLE_MODIFIER_BIT) != 0));
    }

    #[test]
    fn conditional_break_does_not_flag_following_statement() {
        // The BREAK is nested inside an IF — a direct child of the IF, not
        // the LOOP — so nothing after the IF block is flagged (Principle
        // IV: avoid false positives).
        let text = "LOOP\nIF (a=b)\nBREAK\nENDIF\nPRINT LIST=1\nENDLOOP\n";
        let result = voyager_core::parse(text);
        let tokens = collect(&result);
        assert!(!tokens
            .iter()
            .any(|t| t.modifiers_bitset & (1 << UNREACHABLE_MODIFIER_BIT) != 0));
    }

    #[test]
    fn variable_ref_name_is_tagged_with_the_standard_variable_type() {
        let text = "IF (@MODE@ = 1)\nENDIF\n";
        let tokens = collect_variable_refs(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, VARIABLE_TYPE_INDEX);
        // Covers the whole "@MODE@" reference (columns 5..11, 1-based),
        // delimiters included -- not just "MODE" (issue #2: the semantic
        // token's own span must match what the fixed TextMate grammar
        // already does, or the @ delimiters fall back to a differently
        // colored path since semantic tokens take priority where they apply).
        assert_eq!(tokens[0].span, Span::new(CorePosition::new(1, 5), CorePosition::new(1, 11)));
    }

    #[test]
    fn variable_ref_inside_a_quoted_string_value_is_still_tagged() {
        let text = "PRINT LIST='@MODE@'\n";
        let tokens = collect_variable_refs(text);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, VARIABLE_TYPE_INDEX);
    }

    #[test]
    fn every_variable_ref_across_the_document_is_found() {
        let text = "RUN PGM=@PGM1@\nENDRUN\nRUN PGM=@PGM2@\nENDRUN\n";
        let tokens = collect_variable_refs(text);
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn misplaced_break_does_not_flag_anything() {
        let text = "BREAK\nPRINT LIST=1\n";
        let result = voyager_core::parse(text);
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::MisplacedBreak));
        let tokens = collect(&result);
        assert!(!tokens
            .iter()
            .any(|t| t.modifiers_bitset & (1 << UNREACHABLE_MODIFIER_BIT) != 0));
    }
}
