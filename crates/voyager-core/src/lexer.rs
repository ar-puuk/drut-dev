//! Char-level scanning: comments (including nesting, FR-005), continuation-
//! character detection (including blank-line skipping, FR-006), `@variable@`
//! tokenization (FR-010), and the two lexer-level diagnostics (FR-014, FR-015).

use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::span::{Position, Span};
use crate::token::{Token, TokenKind};

/// Characters that split a word run into a standalone, single-character
/// `Punctuation` token. `;` and `@` are handled separately (they open a
/// line comment / variable reference respectively) and are not in this set.
fn is_delimiter(ch: char) -> bool {
    matches!(
        ch,
        ',' | '='
            | '+'
            | '-'
            | '/'
            | '*'
            | '^'
            | '&'
            | '|'
            | '{'
            | '}'
            | '('
            | ')'
            | '['
            | ']'
            | ':'
            | '\''
            | '"'
            | '!'
            | '<'
            | '>'
    )
}

fn pos_at(chars: &[(char, Position)], idx: usize) -> Position {
    if idx < chars.len() {
        chars[idx].1
    } else if let Some(&(ch, p)) = chars.last() {
        p.advance(ch)
    } else {
        Position::new(1, 1)
    }
}

fn text_of(chars: &[(char, Position)], a: usize, b: usize) -> String {
    chars[a..b].iter().map(|&(c, _)| c).collect()
}

fn span_of(chars: &[(char, Position)], a: usize, b: usize) -> Span {
    Span::new(pos_at(chars, a), pos_at(chars, b))
}

/// Normalizes source into `(char, Position)` pairs, dropping `\r` so CRLF and
/// bare-CR line endings behave like LF (real fixtures are Windows-authored).
fn build_char_positions(source: &str) -> Vec<(char, Position)> {
    let mut out = Vec::with_capacity(source.len());
    let mut pos = Position::new(1, 1);
    for ch in source.chars() {
        if ch == '\r' {
            continue;
        }
        out.push((ch, pos));
        pos = pos.advance(ch);
    }
    out
}

/// Tokenizes `source` into a flat, position-tracked token stream
/// (contracts/public-api.md). Never panics on any input.
pub fn tokenize(source: &str) -> Vec<Token> {
    tokenize_with_diagnostics(source).0
}

/// Same as [`tokenize`], but also returns the lexer-level diagnostics
/// (`UnclosedBlockComment` FR-014, `InvalidContinuation` FR-015) that
/// `parse()` folds into its result.
pub fn tokenize_with_diagnostics(source: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let chars = build_char_positions(source);
    let n = chars.len();
    let mut tokens: Vec<Token> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut i = 0usize;
    // Stack of char-indices where each currently-open `/*` began. LIFO
    // matching correctly reproduces nested-comment semantics (FR-005): the
    // next `*/` always closes the most recently opened comment.
    let mut open_comments: Vec<usize> = Vec::new();

    while i < n {
        if !open_comments.is_empty() {
            let ch = chars[i].0;
            if ch == '/' && i + 1 < n && chars[i + 1].0 == '*' {
                open_comments.push(i);
                i += 2;
                continue;
            }
            if ch == '*' && i + 1 < n && chars[i + 1].0 == '/' {
                let start = open_comments.pop().expect("stack checked non-empty above");
                let end = i + 2;
                tokens.push(Token::new(
                    TokenKind::BlockComment {
                        unterminated: false,
                    },
                    span_of(&chars, start, end),
                    text_of(&chars, start, end),
                ));
                i = end;
                continue;
            }
            i += 1;
            continue;
        }

        let ch = chars[i].0;
        match ch {
            ';' => {
                let start = i;
                let mut j = i;
                while j < n && chars[j].0 != '\n' {
                    j += 1;
                }
                tokens.push(Token::new(
                    TokenKind::LineComment,
                    span_of(&chars, start, j),
                    text_of(&chars, start, j),
                ));
                i = j;
            }
            '/' if i + 1 < n && chars[i + 1].0 == '*' => {
                open_comments.push(i);
                i += 2;
            }
            '@' => {
                let mut j = i + 1;
                while j < n && chars[j].0 != '@' && chars[j].0 != '\n' {
                    j += 1;
                }
                if j < n && chars[j].0 == '@' {
                    let end = j + 1;
                    let name = text_of(&chars, i + 1, j);
                    tokens.push(Token::new(
                        TokenKind::VariableRef { name },
                        span_of(&chars, i, end),
                        text_of(&chars, i, end),
                    ));
                    i = end;
                } else {
                    // No closing `@` before newline/EOF — fall back to a
                    // bare punctuation token and keep scanning normally.
                    tokens.push(Token::new(
                        TokenKind::Punctuation,
                        span_of(&chars, i, i + 1),
                        text_of(&chars, i, i + 1),
                    ));
                    i += 1;
                }
            }
            c if c.is_whitespace() => {
                i += 1;
            }
            c if is_delimiter(c) => {
                tokens.push(Token::new(
                    TokenKind::Punctuation,
                    span_of(&chars, i, i + 1),
                    text_of(&chars, i, i + 1),
                ));
                i += 1;
            }
            _ => {
                let start = i;
                let mut j = i;
                while j < n {
                    let c = chars[j].0;
                    if c.is_whitespace() || c == ';' || c == '@' || is_delimiter(c) {
                        break;
                    }
                    j += 1;
                }
                tokens.push(Token::new(
                    TokenKind::Word,
                    span_of(&chars, start, j),
                    text_of(&chars, start, j),
                ));
                i = j;
            }
        }
    }

    // Any comments still open at end-of-input never found their own match
    // (FR-005/FR-014) — innermost first, matching close-time emission order.
    while let Some(start) = open_comments.pop() {
        let span = span_of(&chars, start, n);
        tokens.push(Token::new(
            TokenKind::BlockComment { unterminated: true },
            span,
            text_of(&chars, start, n),
        ));
        diagnostics.push(Diagnostic::new(
            DiagnosticKind::UnclosedBlockComment,
            span,
            "this block comment has no matching closing `*/` before the end of the file",
        ));
    }

    mark_continuation_markers(&mut tokens);
    diagnostics.extend(find_invalid_continuations(&tokens));

    (tokens, diagnostics)
}

/// Retags, per physical line, the last non-comment token as a
/// `ContinuationMarker` when its text is one of the nine recognized
/// continuation characters (FR-006). A trailing comment on the line doesn't
/// count — the lexer looks past it (data-model.md § Token validation rules).
fn mark_continuation_markers(tokens: &mut [Token]) {
    // For each line, find the last non-comment token that starts on it, then
    // retag it if it qualifies. A `BTreeMap` naturally keeps "last write
    // wins" per key as we scan in source order.
    let mut last_non_comment_per_line: std::collections::BTreeMap<u32, usize> =
        std::collections::BTreeMap::new();
    for (idx, tok) in tokens.iter().enumerate() {
        let is_comment = matches!(
            tok.kind,
            TokenKind::LineComment | TokenKind::BlockComment { .. }
        );
        if !is_comment {
            last_non_comment_per_line.insert(tok.span.start.line, idx);
        }
    }
    for idx in last_non_comment_per_line.values().copied() {
        let tok = &tokens[idx];
        if tok.kind == TokenKind::Punctuation
            && tok.span.start.line == tok.span.end.line
            && Token::is_continuation_char_text(&tok.text)
        {
            tokens[idx].kind = TokenKind::ContinuationMarker;
        }
    }
}

/// FR-015: a `ContinuationMarker` with no valid following line — either
/// end-of-input immediately follows (skipping any number of fully blank
/// lines, FR-006), or no further line ever produces a token before
/// end-of-input.
fn find_invalid_continuations(tokens: &[Token]) -> Vec<Diagnostic> {
    let mut lines_with_tokens: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for tok in tokens {
        lines_with_tokens.insert(tok.span.start.line);
        lines_with_tokens.insert(tok.span.end.line);
    }
    let max_line = lines_with_tokens.iter().next_back().copied().unwrap_or(0);

    let mut out = Vec::new();
    for tok in tokens {
        if tok.kind != TokenKind::ContinuationMarker {
            continue;
        }
        let marker_line = tok.span.start.line;
        let mut found = false;
        let mut line = marker_line + 1;
        while line <= max_line {
            if lines_with_tokens.contains(&line) {
                found = true;
                break;
            }
            line += 1;
        }
        if !found {
            out.push(Diagnostic::new(
                DiagnosticKind::InvalidContinuation,
                tok.span,
                "this line ends with a continuation character but no further content follows it",
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trailing_line_comment_is_its_own_token_and_does_not_affect_continuation() {
        let toks = tokenize("RUN PGM=MATRIX ; a trailing comment\n");
        assert!(toks.iter().any(|t| t.kind == TokenKind::LineComment));
        // The last non-comment token on the line is Word("MATRIX"), not a
        // continuation character, so nothing gets retagged.
        assert!(!toks.iter().any(|t| t.kind == TokenKind::ContinuationMarker));
    }

    #[test]
    fn multiline_block_comment_is_one_token_spanning_start_and_end() {
        let toks = tokenize("/* line one\nline two */\n");
        let comment = toks
            .iter()
            .find(|t| matches!(t.kind, TokenKind::BlockComment { .. }))
            .expect("expected a BlockComment token");
        assert_eq!(comment.span.start, Position::new(1, 1));
        assert_eq!(comment.span.end.line, 2);
    }

    #[test]
    fn nested_block_comment_inner_span_sits_inside_outer_span() {
        let toks = tokenize("/* outer /* inner */ still outer */\n");
        let comments: Vec<&Token> = toks
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::BlockComment { .. }))
            .collect();
        assert_eq!(comments.len(), 2);
        // Inner closes first (LIFO), so it's emitted before the outer one.
        let inner = comments[0];
        let outer = comments[1];
        assert!(outer.span.start <= inner.span.start);
        assert!(inner.span.end <= outer.span.end);
    }

    #[test]
    fn unterminated_block_comment_is_flagged_and_diagnosed() {
        let (toks, diags) = tokenize_with_diagnostics("/* never closed\nmore text\n");
        let comment = toks
            .iter()
            .find(|t| matches!(t.kind, TokenKind::BlockComment { unterminated: true }))
            .expect("expected an unterminated BlockComment token");
        assert!(!comment.text.is_empty());
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].kind, DiagnosticKind::UnclosedBlockComment);
    }

    #[test]
    fn deeply_unterminated_nested_comments_each_get_their_own_diagnostic() {
        let (_toks, diags) = tokenize_with_diagnostics("/* one /* two /* three\n");
        assert_eq!(diags.len(), 3);
        assert!(diags
            .iter()
            .all(|d| d.kind == DiagnosticKind::UnclosedBlockComment));
    }

    #[test]
    fn variable_ref_captures_name_and_position_bare() {
        let toks = tokenize("X = @foo@\n");
        let var = toks
            .iter()
            .find(|t| matches!(&t.kind, TokenKind::VariableRef { .. }))
            .expect("expected a VariableRef token");
        match &var.kind {
            TokenKind::VariableRef { name } => assert_eq!(name, "foo"),
            _ => unreachable!(),
        }
        assert_eq!(var.text, "@foo@");
    }

    #[test]
    fn variable_ref_recognized_inside_quoted_string() {
        let toks = tokenize("FILEO NETO = '@ParentDir@@ScenarioDir@file.mtx'\n");
        let names: Vec<String> = toks
            .iter()
            .filter_map(|t| match &t.kind {
                TokenKind::VariableRef { name } => Some(name.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            names,
            vec!["ParentDir".to_string(), "ScenarioDir".to_string()]
        );
    }

    #[test]
    fn trailing_continuation_character_is_marked() {
        let toks = tokenize("FILEI NETI=myfile.nam,\nZDATI=zonal.dat\n");
        let marker = toks
            .iter()
            .find(|t| t.kind == TokenKind::ContinuationMarker);
        assert!(marker.is_some());
        assert_eq!(marker.unwrap().text, ",");
    }

    #[test]
    fn continuation_with_no_following_line_is_invalid() {
        let (_toks, diags) = tokenize_with_diagnostics("FILEI NETI=myfile.nam,\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].kind, DiagnosticKind::InvalidContinuation);
    }

    #[test]
    fn continuation_across_blank_lines_is_valid() {
        let (_toks, diags) =
            tokenize_with_diagnostics("FILEI NETI=myfile.nam,\n\n\nZDATI=zonal.dat\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn empty_input_produces_no_tokens_and_no_panic() {
        let toks = tokenize("");
        assert!(toks.is_empty());
    }

    #[test]
    fn whitespace_and_comment_only_input_produces_no_diagnostics() {
        let (_toks, diags) = tokenize_with_diagnostics("   \n; just a comment\n\t\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn case_insensitive_words_preserve_original_casing() {
        let toks = tokenize("If iF IF\n");
        let words: Vec<&str> = toks.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(words, vec!["If", "iF", "IF"]);
    }
}
