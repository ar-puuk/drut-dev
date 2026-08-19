//! Recognizes `function_calls`-category tokens (spec.md FR-002–FR-004,
//! `025-function-casing`) — a Cube Voyager built-in function name,
//! immediately followed by `(` with zero intervening whitespace. A
//! read-only pass over already-parsed data, the same architectural shape
//! `data_reference.rs` already uses for its own category (research.md §2) —
//! no lexer or `TokenKind` change.
//!
//! **Name list provenance**: the 138 entries below are `024-function-call-
//! highlighting/research.md` §2 verbatim (identifier names only, never
//! vendor documentation prose — constitution Principle II) — built from a
//! complete reading of two vendor documentation editions (Cube Voyager
//! 6.5.1 and OpenPaths Cube/CUBE CONNECT Edition), cross-validated against
//! each other, plus one real-corpus-confirmed addition
//! (`PRINTPROGRESS`) absent from both editions. This module is now the
//! single source of truth for that list (Constitution Principle I) —
//! `editors/vscode/syntaxes/drut.tmLanguage.json`'s `#function-calls`
//! pattern is a documented, manually-synced mirror of it, the same
//! relationship `#control-words` already has with `statement.rs`'s
//! `FIXED_KEYWORDS`. Not exhaustive by construction (`024` research.md §5):
//! a real Cube Voyager built-in function absent from this list is simply
//! never recognized here, exactly as before this module existed.
//!
//! **The `(`-adjacency requirement is load-bearing, not stylistic**
//! (research.md §3): two names in this list collide with real,
//! independently-recognized `voyager-core` vocabulary —
//! `FORMAT` (a `FILEO` pair-keyword, `keywords.rs`'s `PAIR_KEYWORDS`) and
//! `LOG` (a control/statement word, recorded in `keywords.rs`'s `PAIR_KEYWORDS`
//! as a control word `VAR` pairs with). A pair-keyword name is always
//! followed by `=`; a control word leads a statement, followed by
//! whitespace; a function call is followed immediately by `(`. These three
//! trigger conditions are mutually exclusive by construction, so a given
//! occurrence of `FORMAT`/`LOG` is claimed by exactly one category, never
//! zero, never two (spec.md FR-004, SC-004) — verified by tests in
//! `format.rs`, not merely assumed.
//!
//! **Quote-safety**: matching skips any `Word` token found while inside a
//! single- or double-quoted run, mirroring `data_reference.rs`'s (and, in
//! turn, `statement.rs`'s `pair_keyword_boundaries`'s) own quote-tracking —
//! without it, a function-shaped substring inside a `PRINT`ed string
//! literal (e.g. `PRINT LIST='calling replacestr(x) here'`) would be
//! wrongly recognized and rewritten.

use crate::block::{Block, BlockKind};
use crate::span::Span;
use crate::statement::{Statement, StatementKind};
use crate::token::{Token, TokenKind};
use crate::Node;

/// One recognized function-call name (research.md §2). Matching against
/// document text is case-insensitive; `name` is the canonical uppercase
/// spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionCallEntry {
    pub name: &'static str,
}

const fn entry(name: &'static str) -> FunctionCallEntry {
    FunctionCallEntry { name }
}

/// Every recognized function-call name — 138 entries, grouped exactly as
/// `024-function-call-highlighting/research.md` §2 categorizes them (module
/// docs above). Group comments are documentation only; matching itself
/// treats this as one flat, case-insensitive set (`is_function_call_name`).
const FUNCTION_CALL_ENTRIES: &[FunctionCallEntry] = &[
    // Numeric (26).
    entry("ABS"),
    entry("CMPNUMRETNUM"),
    entry("CURRENTTIME"),
    entry("EXP"),
    entry("EXPDIST"),
    entry("EXPINV"),
    entry("GAMMADIST"),
    entry("GAMMAINV"),
    entry("INLIST"),
    entry("INT"),
    entry("LN"),
    entry("LOG"),
    entry("LOGNORMDIST"),
    entry("LOGNORMINV"),
    entry("MAX"),
    entry("MIN"),
    entry("NORMDIST"),
    entry("NORMINV"),
    entry("POISSONDIST"),
    entry("POISSONINV"),
    entry("POW"),
    entry("RAND"),
    entry("RANDOM"),
    entry("RANDSEED"),
    entry("ROUND"),
    entry("SQRT"),
    // Trigonometric (6).
    entry("ARCCOS"),
    entry("ARCSIN"),
    entry("ARCTAN"),
    entry("COS"),
    entry("SIN"),
    entry("TAN"),
    // Character/String (20).
    entry("DELETESTR"),
    entry("DUPSTR"),
    entry("FORMAT"),
    entry("FORMATDATETIME"),
    entry("INSERTSTR"),
    entry("LEFTSTR"),
    entry("LTRIM"),
    entry("REPLACESTR"),
    entry("REPLACESTRIC"),
    entry("REVERSESTR"),
    entry("RIGHTSTR"),
    entry("STR"),
    entry("STRLEN"),
    entry("STRLOWER"),
    entry("STRPOS"),
    entry("STRPOSEX"),
    entry("STRUPPER"),
    entry("SUBSTR"),
    entry("TRIM"),
    entry("VAL"),
    // Highway/Matrix (21).
    entry("ARRAYSUM"),
    entry("CAPACITYFOR"),
    entry("CHECKNAME"),
    entry("GETMATRIXROW"),
    entry("GETVALUE"),
    entry("LINKNUM"),
    entry("LOWEST"),
    entry("MATVAL"),
    entry("PATHTRACE"),
    entry("ROWADD"),
    entry("ROWAVE"),
    entry("ROWCNT"),
    entry("ROWDIV"),
    entry("ROWFAC"),
    entry("ROWFIX"),
    entry("ROWMAX"),
    entry("ROWMIN"),
    entry("ROWMPY"),
    entry("ROWREAD"),
    entry("ROWSUM"),
    entry("SPEEDFOR"),
    // Public Transport skims (19).
    entry("BRDINGS"),
    entry("BRDPEN"),
    entry("COMPCOST"),
    entry("CWDCOSTP"),
    entry("CWDWAITA"),
    entry("CWDWAITP"),
    entry("DIST"),
    entry("FAREA"),
    entry("FAREP"),
    entry("GCOST"),
    entry("IWAITA"),
    entry("IWAITP"),
    entry("TIMEA"),
    entry("TIMEP"),
    entry("VALOFCHOICE"),
    entry("XFERPENA"),
    entry("XFERPENP"),
    entry("XWAITA"),
    entry("XWAITP"),
    // CONVERGE-phase iteration statistics (42).
    entry("GAPCHANGE"),
    entry("RGAPCHANGE"),
    entry("AADCHANGE"),
    entry("RAADCHANGE"),
    entry("PDIFFCHANGE"),
    entry("RMSECHANGE"),
    entry("GAPMIN"),
    entry("GAPMAX"),
    entry("GAPAVE"),
    entry("GAPCHANGEMIN"),
    entry("GAPCHANGEMAX"),
    entry("GAPCHANGEAVE"),
    entry("RGAPMIN"),
    entry("RGAPMAX"),
    entry("RGAPAVE"),
    entry("RGAPCHANGEMIN"),
    entry("RGAPCHANGEMAX"),
    entry("RGAPCHANGEAVE"),
    entry("AADMIN"),
    entry("AADMAX"),
    entry("AADAVE"),
    entry("AADCHANGEMIN"),
    entry("AADCHANGEMAX"),
    entry("AADCHANGEAVE"),
    entry("RAADMIN"),
    entry("RAADMAX"),
    entry("RAADAVE"),
    entry("RAADCHANGEMIN"),
    entry("RAADCHANGEMAX"),
    entry("RAADCHANGEAVE"),
    entry("PDIFFMIN"),
    entry("PDIFFMAX"),
    entry("PDIFFAVE"),
    entry("PDIFFCHANGEMIN"),
    entry("PDIFFCHANGEMAX"),
    entry("PDIFFCHANGEAVE"),
    entry("RMSEMIN"),
    entry("RMSEMAX"),
    entry("RMSEAVE"),
    entry("RMSECHANGEMIN"),
    entry("RMSECHANGEMAX"),
    entry("RMSECHANGEAVE"),
    // CUBE Cluster utility (3).
    entry("FILESEXIST"),
    entry("FIRSTREADYNODE"),
    entry("NUMREADYNODES"),
    // Corpus-confirmed, absent from both vendor doc editions (1).
    entry("PRINTPROGRESS"),
];

/// Returns every recognized function-call entry.
pub fn function_call_entries() -> &'static [FunctionCallEntry] {
    FUNCTION_CALL_ENTRIES
}

/// Whether `text` case-insensitively matches a recognized function-call
/// name exactly.
fn is_function_call_name(text: &str) -> bool {
    FUNCTION_CALL_ENTRIES.iter().any(|e| e.name.eq_ignore_ascii_case(text))
}

/// One matched function-call occurrence — `span` covers exactly the
/// matched name (never the following `(`), so a caller can rewrite exactly
/// that span's casing without touching anything else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCallOccurrence {
    pub name: String,
    pub span: Span,
}

/// Finds every function-call occurrence in `nodes`. Pure, no I/O, never
/// panics on any input.
pub fn function_call_occurrences(nodes: &[Node]) -> Vec<FunctionCallOccurrence> {
    let mut out = Vec::new();
    collect(nodes, &mut out);
    out
}

fn collect(nodes: &[Node], out: &mut Vec<FunctionCallOccurrence>) {
    for node in nodes {
        match node {
            Node::Statement(stmt) => collect_statement(stmt, out),
            Node::Block(block) => collect_block(block, out),
        }
    }
}

fn collect_block(block: &Block, out: &mut Vec<FunctionCallOccurrence>) {
    // Unlike data_reference.rs, a block opener's own pair-keyword names
    // (block.opener_pairs) are never scanned here -- a function call is
    // never a pair-keyword name itself (it's followed by `(`, a
    // pair-keyword name is followed by `=`), so there is nothing in that
    // position this module could ever match.
    match &block.kind {
        BlockKind::If { branches } => {
            for branch in branches {
                if let Some(condition) = &branch.condition {
                    collect_tokens(condition, out);
                }
                collect(&branch.children, out);
            }
        }
        _ => collect(&block.children, out),
    }
}

fn collect_statement(stmt: &Statement, out: &mut Vec<FunctionCallOccurrence>) {
    // Casing (of any category) never targets Label/ShellEscape content --
    // mirrors data_reference.rs's own scope exactly (FR-015 in
    // 002-cli-check-format). Control and Assignment statements are both in
    // scope: a function call routinely appears on an Assignment's
    // right-hand side (`RouteName = REPLACESTR(...)`), the exact case
    // format.rs's control_words/pair_keywords AST walk does not reach
    // (research.md §2) -- the reason this module exists as a separate,
    // token-scanning pass rather than an addition to that walk.
    if matches!(stmt.kind, StatementKind::Label { .. } | StatementKind::ShellEscape { .. }) {
        return;
    }
    collect_tokens(&stmt.tokens, out);
}

/// Quote-aware scan over `tokens`: every `Word` token found *outside* a
/// single-/double-quoted run, whose text matches a recognized function-call
/// name, is checked for the one condition unique to this category -- an
/// immediately-following `(` with zero intervening whitespace (module docs;
/// research.md §3/§4). Mirrors `data_reference.rs`'s `collect_tokens`
/// quote-tracking exactly.
fn collect_tokens(tokens: &[Token], out: &mut Vec<FunctionCallOccurrence>) {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    for (i, tok) in tokens.iter().enumerate() {
        if tok.kind == TokenKind::Punctuation {
            match tok.text.as_str() {
                "'" if !in_double_quote => in_single_quote = !in_single_quote,
                "\"" if !in_single_quote => in_double_quote = !in_double_quote,
                _ => {}
            }
            continue;
        }
        if tok.kind != TokenKind::Word || in_single_quote || in_double_quote {
            continue;
        }
        if !is_function_call_name(&tok.text) {
            continue;
        }
        let Some(next) = tokens.get(i + 1) else {
            continue;
        };
        let is_call = next.kind == TokenKind::Punctuation && next.text == "(" && next.span.start == tok.span.end;
        if is_call {
            out.push(FunctionCallOccurrence {
                name: tok.text.to_ascii_uppercase(),
                span: tok.span,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn occurrences(source: &str) -> Vec<FunctionCallOccurrence> {
        let parsed = parse(source);
        function_call_occurrences(&parsed.nodes)
    }

    #[test]
    fn function_call_list_has_138_entries() {
        assert_eq!(FUNCTION_CALL_ENTRIES.len(), 138);
    }

    #[test]
    fn function_call_list_has_no_duplicate_entries() {
        let mut names: Vec<&str> = FUNCTION_CALL_ENTRIES.iter().map(|e| e.name).collect();
        names.sort_unstable();
        let mut deduped = names.clone();
        deduped.dedup();
        assert_eq!(names.len(), deduped.len(), "duplicate entries found: {names:?}");
    }

    #[test]
    fn assignment_rhs_function_call_recognized() {
        let occ = occurrences("RouteName = REPLACESTR(RouteName,'-','',0)\n");
        assert_eq!(occ.len(), 1);
        assert_eq!(occ[0].name, "REPLACESTR");
    }

    #[test]
    fn nested_function_calls_both_recognized() {
        let occ = occurrences("if (RIGHTSTR(TRIM(RouteName),1)='-') X = 1\n");
        let names: Vec<&str> = occ.iter().map(|o| o.name.as_str()).collect();
        assert!(names.contains(&"RIGHTSTR"), "{names:?}");
        assert!(names.contains(&"TRIM"), "{names:?}");
    }

    #[test]
    fn function_call_inside_if_condition_recognized() {
        let occ = occurrences("if (STRLEN(TRIM(@SEGIDExField@))>0) X = 1\n");
        let names: Vec<&str> = occ.iter().map(|o| o.name.as_str()).collect();
        assert!(names.contains(&"STRLEN"), "{names:?}");
        assert!(names.contains(&"TRIM"), "{names:?}");
    }

    #[test]
    fn bareword_with_no_following_paren_is_not_a_function_call() {
        let occ = occurrences("MAX = 100\n");
        assert!(occ.is_empty(), "{occ:?}");
    }

    #[test]
    fn function_shaped_text_inside_a_quoted_string_is_not_recognized() {
        let occ = occurrences("PRINT LIST='calling REPLACESTR(x) here'\n");
        assert!(occ.is_empty(), "{occ:?}");
    }

    #[test]
    fn whitespace_before_paren_is_not_a_function_call() {
        let occ = occurrences("X = REPLACESTR (a,b,c,0)\n");
        assert!(occ.is_empty(), "{occ:?}");
    }

    #[test]
    fn format_as_a_pair_keyword_value_position_is_not_a_function_call() {
        // FORMAT is a real dual-category name (research.md §3) -- as a
        // FILEO pair-keyword (`FORMAT=csv`, followed by `=`) it must never
        // be recognized here.
        let occ = occurrences("FILEO FORMAT=csv\n");
        assert!(occ.is_empty(), "{occ:?}");
    }

    #[test]
    fn format_as_a_function_call_is_recognized() {
        let occ = occurrences("X = FORMAT(volume,8,2,',')\n");
        assert_eq!(occ.len(), 1);
        assert_eq!(occ[0].name, "FORMAT");
    }

    #[test]
    fn log_as_a_control_word_position_is_not_a_function_call() {
        // LOG is the other real dual-category name (research.md §3) -- as
        // a control word (`LOG VAR=x`, leading the statement, followed by
        // whitespace) it must never be recognized here.
        let occ = occurrences("LOG VAR=x\n");
        assert!(occ.is_empty(), "{occ:?}");
    }

    #[test]
    fn log_as_a_function_call_is_recognized() {
        let occ = occurrences("Y = LOG(5)\n");
        assert_eq!(occ.len(), 1);
        assert_eq!(occ[0].name, "LOG");
    }

    #[test]
    fn every_recognized_name_is_matched_when_called() {
        for e in FUNCTION_CALL_ENTRIES {
            let source = format!("X = {}(1,2,3)\n", e.name);
            let occ = occurrences(&source);
            assert_eq!(occ.len(), 1, "expected exactly one occurrence for {}: {occ:?}", e.name);
            assert_eq!(occ[0].name, e.name);
        }
    }

    #[test]
    fn is_function_call_name_is_case_insensitive() {
        assert!(is_function_call_name("replacestr"));
        assert!(is_function_call_name("ReplaceStr"));
        assert!(is_function_call_name("REPLACESTR"));
        assert!(!is_function_call_name("not_a_real_function"));
    }
}
