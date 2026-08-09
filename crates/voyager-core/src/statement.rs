//! Groups tokens into statements (data-model.md § Statement; FR-003, FR-006,
//! FR-007, FR-011, FR-021–FR-023): joining continuation-joined physical
//! lines (trailing-operator or `{...}`-delimited), splitting the short-`IF`
//! and trailing-`ELSEIF`/`ELSE`/`ENDIF` shape onto its own statement, and
//! classifying each result as `Control`/`Assignment`/`Label`/`ShellEscape`.

use crate::span::Span;
use crate::token::{Token, TokenKind};

/// A logical unit of Voyager script, possibly spanning multiple physical
/// lines joined by continuation (data-model.md § Statement).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
    /// The statement's own tokens (comments excluded, continuation markers
    /// kept in place rather than stripped).
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementKind {
    /// A control word plus zero or more `keyword=value` pairs (FR-003).
    /// `word` and each pair's keyword are compared case-insensitively
    /// (FR-011) by callers; original casing is preserved here.
    Control {
        word: String,
        pairs: Vec<(String, Vec<Token>)>,
    },
    /// A plain `identifier = value` statement with no control word (FR-023).
    Assignment { target: String, value: Vec<Token> },
    /// A `:identifier` line (FR-021).
    Label { name: String },
    /// A `*`/`**` line; the command text that follows is stored opaquely,
    /// never parsed as Voyager grammar (FR-022).
    ShellEscape { command_tokens: Vec<Token> },
}

/// The literal block-structural keywords this crate recognizes by name
/// (case-insensitively) to drive block matching — distinct from, and much
/// smaller than, the open-ended per-program control-word vocabulary FR-023's
/// structural rule (word immediately followed by `=`, or not) already
/// disambiguates without needing a vocabulary at all (see spec.md
/// Assumptions on FR-003/CHK008).
const FIXED_KEYWORDS: &[&str] = &[
    "IF",
    "ELSEIF",
    "ELSE",
    "ENDIF",
    "LOOP",
    "ENDLOOP",
    "BREAK",
    "RUN",
    "ENDRUN",
    "PROCESS",
    "PHASE",
    "ENDPROCESS",
    "ENDPHASE",
    "JLOOP",
    "ENDJLOOP",
    "LINKLOOP",
    "ENDLINKLOOP",
    "DISTRIBUTEMULTISTEP",
    "ENDDISTRIBUTEMULTISTEP",
];

fn is_punct(tok: &Token, text: &str) -> bool {
    tok.kind == TokenKind::Punctuation && tok.text == text
}

/// Builds the statement sequence for a full token stream. Never panics: any
/// shape it doesn't recognize falls back to a permissive `Assignment` (or, in
/// pathological cases, an empty-target one) rather than failing.
pub fn build_statements(tokens: Vec<Token>) -> Vec<Statement> {
    let content: Vec<Token> = tokens
        .into_iter()
        .filter(|t| {
            !matches!(
                t.kind,
                TokenKind::LineComment | TokenKind::BlockComment { .. }
            )
        })
        .collect();

    let mut groups: Vec<Vec<Token>> = Vec::new();
    let mut i = 0;
    while i < content.len() {
        let (grp, next_i) = consume_one_statement(&content, i);
        i = next_i;
        groups.push(grp);
    }

    // Split the IF-family same-line-trailing-statement shape (FR-007) out of
    // each group, re-checking the split-off tail in case it needs splitting
    // again (defensive; real grammar only ever needs one split per group).
    let mut queue: std::collections::VecDeque<Vec<Token>> = groups.into_iter().collect();
    let mut final_groups: Vec<Vec<Token>> = Vec::new();
    while let Some(grp) = queue.pop_front() {
        match split_if_family_trailing(&grp) {
            Some((head, tail)) => {
                final_groups.push(head);
                queue.push_front(tail);
            }
            None => final_groups.push(grp),
        }
    }

    final_groups.into_iter().map(classify_statement).collect()
}

/// Assembles one statement's tokens starting at `content[start]`, applying
/// whichever continuation mechanism (if any) is in play, and returns the
/// index to resume scanning from.
fn consume_one_statement(content: &[Token], start: usize) -> (Vec<Token>, usize) {
    let mut acc: Vec<Token> = vec![content[start].clone()];
    let mut i = start + 1;

    let brace_mode =
        content[start].kind == TokenKind::Word && i < content.len() && is_punct(&content[i], "{");

    if brace_mode {
        acc.push(content[i].clone());
        i += 1;
        // FR-006: the next `}` always closes the body, even if another `{`
        // appears first inside — brace bodies do not nest.
        while i < content.len() {
            let tok = content[i].clone();
            let is_close = is_punct(&tok, "}");
            acc.push(tok);
            i += 1;
            if is_close {
                break;
            }
        }
        return (acc, i);
    }

    loop {
        if i >= content.len() {
            break;
        }
        let prev_line = acc
            .last()
            .expect("acc always has >=1 token")
            .span
            .start
            .line;
        let next_line = content[i].span.start.line;
        if next_line == prev_line {
            acc.push(content[i].clone());
            i += 1;
            continue;
        }
        // Line boundary: continue only if the last token added was a
        // continuation marker (FR-006) — blank lines in between never show
        // up here at all, since they contribute no tokens, so "the next
        // token" is already correctly "the next content-bearing line".
        let last_was_continuation =
            acc.last().expect("acc always has >=1 token").kind == TokenKind::ContinuationMarker;
        if last_was_continuation {
            acc.push(content[i].clone());
            i += 1;
        } else {
            break;
        }
    }
    (acc, i)
}

/// Detects and splits off a same-line trailing statement following a short-
/// form `IF`/`ELSEIF` condition or a bare `ELSE`/`ENDIF` (FR-007). Returns
/// `None` when there is nothing to split (the common case).
fn split_if_family_trailing(grp: &[Token]) -> Option<(Vec<Token>, Vec<Token>)> {
    let first = grp.first()?;
    if first.kind != TokenKind::Word {
        return None;
    }
    match first.text.to_ascii_uppercase().as_str() {
        "ELSE" | "ENDIF" => {
            if grp.len() > 1 {
                Some((grp[0..1].to_vec(), grp[1..].to_vec()))
            } else {
                None
            }
        }
        "IF" | "ELSEIF" => {
            let mut idx = 1;
            while idx < grp.len() && !is_punct(&grp[idx], "(") {
                idx += 1;
            }
            if idx >= grp.len() {
                return None; // no condition found; leave the group as-is
            }
            let mut depth = 0i32;
            let mut end_idx = None;
            let mut j = idx;
            while j < grp.len() {
                if is_punct(&grp[j], "(") {
                    depth += 1;
                } else if is_punct(&grp[j], ")") {
                    depth -= 1;
                    if depth == 0 {
                        end_idx = Some(j);
                        break;
                    }
                }
                j += 1;
            }
            let end_idx = end_idx?;
            if end_idx + 1 < grp.len() {
                Some((grp[0..=end_idx].to_vec(), grp[end_idx + 1..].to_vec()))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Extracts top-level (bracket-depth 0) `keyword=value` pairs from a token
/// slice — used for both `Control.pairs` and, indirectly, ignored entirely
/// for parenthesized conditions (`IF`/`ELSEIF`), where depth never returns
/// to 0 until the closing paren, so no pairs are found and the condition
/// tokens are preserved via `Statement.tokens` instead.
fn extract_pairs(tokens: &[Token]) -> Vec<(String, Vec<Token>)> {
    let mut depth: i32 = 0;
    let mut pair_starts: Vec<usize> = Vec::new();
    for (i, tok) in tokens.iter().enumerate() {
        if tok.kind == TokenKind::Punctuation {
            match tok.text.as_str() {
                "(" | "[" | "{" => depth += 1,
                ")" | "]" | "}" => depth -= 1,
                _ => {}
            }
        }
        if depth == 0
            && tok.kind == TokenKind::Word
            && i + 1 < tokens.len()
            && is_punct(&tokens[i + 1], "=")
        {
            pair_starts.push(i);
        }
    }
    let mut pairs = Vec::with_capacity(pair_starts.len());
    for (idx, &start) in pair_starts.iter().enumerate() {
        let keyword = tokens[start].text.clone();
        let value_begin = (start + 2).min(tokens.len());
        let value_end = pair_starts.get(idx + 1).copied().unwrap_or(tokens.len());
        let value = tokens[value_begin..value_end.min(tokens.len()).max(value_begin)].to_vec();
        pairs.push((keyword, value));
    }
    pairs
}

fn classify_statement(grp: Vec<Token>) -> Statement {
    let span = grp
        .first()
        .expect("groups are never empty by construction")
        .span
        .merge(
            grp.last()
                .expect("groups are never empty by construction")
                .span,
        );

    let kind = if is_punct(&grp[0], ":") {
        let name = grp.get(1).map(|t| t.text.clone()).unwrap_or_default();
        StatementKind::Label { name }
    } else if is_punct(&grp[0], "*") {
        let skip = if grp.get(1).map(|t| is_punct(t, "*")).unwrap_or(false) {
            2
        } else {
            1
        };
        let skip = skip.min(grp.len());
        StatementKind::ShellEscape {
            command_tokens: grp[skip..].to_vec(),
        }
    } else if is_punct(&grp[0], "!")
        && grp
            .get(1)
            .map(|t| t.kind == TokenKind::Word && t.text.eq_ignore_ascii_case("RUN"))
            .unwrap_or(false)
    {
        let word = format!("!{}", grp[1].text);
        let pairs = extract_pairs(&grp[2..]);
        StatementKind::Control { word, pairs }
    } else if grp[0].kind == TokenKind::Word
        && FIXED_KEYWORDS.contains(&grp[0].text.to_ascii_uppercase().as_str())
    {
        let word = grp[0].text.clone();
        let pairs =
            if word.eq_ignore_ascii_case("PHASE") && grp.len() >= 2 && is_punct(&grp[1], "=") {
                // The bare `PHASE=value` shortcut (FR-028): the control word
                // itself carries the `=value`, which `extract_pairs` can't see
                // since it only looks for a *following* word adjacent to `=`.
                // Synthesize the one pair directly so `PHASE`'s value is still
                // reachable via `Control.pairs`, the same as `PROCESS PHASE=value`.
                let value_begin = 2.min(grp.len());
                vec![("PHASE".to_string(), grp[value_begin..].to_vec())]
            } else {
                extract_pairs(&grp[1..])
            };
        StatementKind::Control { word, pairs }
    } else if grp[0].kind == TokenKind::Word && grp.len() >= 2 && is_punct(&grp[1], "=") {
        let target = grp[0].text.clone();
        let value = grp[2..].to_vec();
        StatementKind::Assignment { target, value }
    } else if grp[0].kind == TokenKind::Word {
        let word = grp[0].text.clone();
        let pairs = extract_pairs(&grp[1..]);
        StatementKind::Control { word, pairs }
    } else {
        // Pathological leading punctuation with no recognized shape — never
        // panics, still round-trips through `tokens`.
        StatementKind::Assignment {
            target: String::new(),
            value: grp.clone(),
        }
    };

    Statement {
        kind,
        span,
        tokens: grp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    fn statements_of(src: &str) -> Vec<Statement> {
        build_statements(tokenize(src))
    }

    #[test]
    fn control_statement_basic() {
        let stmts = statements_of("RUN PGM=MATRIX\n");
        assert_eq!(stmts.len(), 1);
        match &stmts[0].kind {
            StatementKind::Control { word, pairs } => {
                assert_eq!(word, "RUN");
                assert_eq!(pairs.len(), 1);
                assert_eq!(pairs[0].0, "PGM");
            }
            other => panic!("expected Control, got {other:?}"),
        }
    }

    #[test]
    fn assignment_statement_basic() {
        let stmts = statements_of("ScriptStartTime = currenttime()\n");
        assert_eq!(stmts.len(), 1);
        match &stmts[0].kind {
            StatementKind::Assignment { target, .. } => assert_eq!(target, "ScriptStartTime"),
            other => panic!("expected Assignment, got {other:?}"),
        }
    }

    #[test]
    fn label_statement_basic() {
        let stmts = statements_of(":STEP0\n");
        assert_eq!(stmts.len(), 1);
        match &stmts[0].kind {
            StatementKind::Label { name } => assert_eq!(name, "STEP0"),
            other => panic!("expected Label, got {other:?}"),
        }
    }

    #[test]
    fn shell_escape_parenthesized_and_bare() {
        let stmts = statements_of("*(ECHO hi)\n*DEL file.tmp\n");
        assert_eq!(stmts.len(), 2);
        assert!(matches!(stmts[0].kind, StatementKind::ShellEscape { .. }));
        assert!(matches!(stmts[1].kind, StatementKind::ShellEscape { .. }));
    }

    #[test]
    fn phase_shortcut_is_control_not_assignment() {
        let stmts = statements_of("PHASE=ILOOP\n");
        assert_eq!(stmts.len(), 1);
        match &stmts[0].kind {
            StatementKind::Control { word, .. } => assert_eq!(word, "PHASE"),
            other => panic!("expected Control, got {other:?}"),
        }
    }

    #[test]
    fn trailing_operator_continuation_joins_one_statement() {
        let stmts = statements_of("FILEI NETI=myfile.nam,\nZDATI=zonal.dat\n");
        assert_eq!(stmts.len(), 1);
    }

    #[test]
    fn blank_line_between_continuation_is_skipped() {
        let stmts = statements_of("FILEI NETI=myfile.nam,\n\n\nZDATI=zonal.dat\n");
        assert_eq!(stmts.len(), 1);
    }

    #[test]
    fn brace_continuation_joins_one_statement_and_does_not_nest() {
        let stmts = statements_of("FILEI {\nNETI = a\nZDATI = b\n}\n");
        assert_eq!(stmts.len(), 1);
        match &stmts[0].kind {
            StatementKind::Control { word, .. } => assert_eq!(word, "FILEI"),
            other => panic!("expected Control, got {other:?}"),
        }
    }

    #[test]
    fn short_if_splits_trailing_statement() {
        let stmts = statements_of("IF (X=1) Y = 2\n");
        assert_eq!(stmts.len(), 2);
        assert!(matches!(stmts[0].kind, StatementKind::Control { .. }));
        match &stmts[1].kind {
            StatementKind::Assignment { target, .. } => assert_eq!(target, "Y"),
            other => panic!("expected Assignment, got {other:?}"),
        }
    }

    #[test]
    fn else_with_trailing_statement_splits() {
        let stmts = statements_of("ELSE Z = 3\n");
        assert_eq!(stmts.len(), 2);
    }

    #[test]
    fn bang_run_is_disabled_control() {
        let stmts = statements_of("!RUN PGM=MATRIX\n");
        assert_eq!(stmts.len(), 1);
        match &stmts[0].kind {
            StatementKind::Control { word, .. } => assert_eq!(word, "!RUN"),
            other => panic!("expected Control, got {other:?}"),
        }
    }
}
