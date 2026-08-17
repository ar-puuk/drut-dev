//! Recognizes `data_references`-category tokens (spec.md FR-004–FR-006,
//! `017-casing-categories-indent-width`) — the Matrix/Line/Node/Zone/
//! Database abbreviation families, the output-record and link-endpoint
//! tokens, and the two reserved implicit loop-index identifiers. A
//! read-only pass over already-parsed data, the same architectural shape
//! `token_resolution.rs`/`block_resolution.rs` already use — no lexer or
//! `TokenKind` change (research.md §1).
//!
//! **One name, one occurrence, regardless of structural shape** (FR-005):
//! a token matches here whenever its own text (or, for dot-notation, the
//! text before the first `.`) case-insensitively equals a recognized name,
//! wherever it appears — a dot-notation read (`mi.1.1`), a pair-keyword
//! name (`PATHLOAD ... MW[201]=`), a block opener's own pair (`RUN
//! PGM=MATRIX ZONES=5`), an assignment target (`MW[1] = ...`), or a bare
//! value reference (`LIST=A(5),B(5)`, `IF (I=25)`). This module has no
//! concept of "which shape" a match came from in its own return type, by
//! design, so no caller can accidentally apply different casing to the
//! same name in different positions.
//!
//! **Quote-safety**: matching skips any `Word` token found while inside a
//! single- or double-quoted run, mirroring `statement.rs`'s own
//! `pair_keyword_boundaries` quote-tracking — without it, a data-reference-
//! shaped substring inside a `PRINT`ed string literal would be wrongly
//! rewritten, the exact class of bug `pair_keyword_boundaries` itself was
//! once fixed for (see `keywords.rs`'s module docs).
//!
//! **Overlap with `pair_keywords`/`control_words`**: several family members
//! (`MW`, `ZONES`, `Z`, `DBI`) can also appear in a genuine pair-keyword-
//! name position (`FILEI DBI=...`, `RUN PGM=MATRIX ZONES=5`). Per FR-005,
//! `data_references` — not `pair_keywords` — owns casing for those specific
//! occurrences; `format.rs`'s pair-keyword collection explicitly skips any
//! name this module recognizes (`is_data_reference_name`), so a token is
//! never queued for two different casing conventions at once.

use crate::block::{Block, BlockKind};
use crate::span::{Position, Span};
use crate::statement::{Statement, StatementKind};
use crate::token::{Token, TokenKind};
use crate::Node;

/// One recognized data-reference family member (research.md §6). Matching
/// against document text is case-insensitive; `name` is the canonical
/// uppercase spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataReferenceEntry {
    pub name: &'static str,
}

const fn entry(name: &'static str) -> DataReferenceEntry {
    DataReferenceEntry { name }
}

/// Every recognized data-reference name (research.md §6's family table):
/// Matrix (`MI`/`MO`/`MW`), Line (`LI`/`LW`), Node (`NI`/`NW`), Zone
/// (`ZI`/`ZONES`/`Z`), Database (`DBI`/`DBA`), the output-record token
/// (`RO`), the link-endpoint tokens (`A`/`B`), and the two reserved
/// implicit loop-index identifiers (`I`/`J`).
const DATA_REFERENCE_ENTRIES: &[DataReferenceEntry] = &[
    entry("MI"),
    entry("MO"),
    entry("MW"),
    entry("LI"),
    entry("LW"),
    entry("NI"),
    entry("NW"),
    entry("ZI"),
    entry("ZONES"),
    entry("Z"),
    entry("DBI"),
    entry("DBA"),
    entry("RO"),
    entry("A"),
    entry("B"),
    entry("I"),
    entry("J"),
];

/// Returns every recognized data-reference entry.
pub fn data_reference_entries() -> &'static [DataReferenceEntry] {
    DATA_REFERENCE_ENTRIES
}

/// Whether `text` case-insensitively matches a recognized data-reference
/// name exactly (no dot-notation prefix matching here — see
/// `dot_notation_prefix_len` for that). Used both by this module's own
/// occurrence collection and by `format.rs`'s pair-keyword casing
/// collection, to keep a dual-role name (e.g. `ZONES`) from being queued
/// for both `pair_keywords` and `data_references` casing at once.
pub(crate) fn is_data_reference_name(text: &str) -> bool {
    DATA_REFERENCE_ENTRIES.iter().any(|e| e.name.eq_ignore_ascii_case(text))
}

/// If `text` starts with a recognized data-reference name immediately
/// followed by `.` (dot-notation read, e.g. `mi.1.1`), returns the length
/// (in `char`s) of the matched name prefix — never including the `.`
/// itself or anything after it.
fn dot_notation_prefix_len(text: &str) -> Option<usize> {
    let chars: Vec<char> = text.chars().collect();
    let dot_idx = chars.iter().position(|&c| c == '.')?;
    let prefix: String = chars[..dot_idx].iter().collect();
    if is_data_reference_name(&prefix) {
        Some(dot_idx)
    } else {
        None
    }
}

/// One matched data-reference occurrence — `span` covers exactly the
/// matched name (never a `[...]` subscript, never text after a `.`), so a
/// caller can rewrite exactly that span's casing without touching anything
/// else (data-model.md §1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataReferenceOccurrence {
    pub name: String,
    pub span: Span,
}

/// Finds every data-reference occurrence in `nodes`. `lines` (the same
/// per-line `char` slices `format.rs`'s renderer already builds) is needed
/// only to recover a block opener's own pair-keyword-name text — `Block`
/// keeps only that text's `Span`, not the original token (see
/// `Block::opener_pairs`'s own doc comment) — every other shape works
/// directly from already-available `Token` text. Pure, no I/O, never
/// panics on any input.
pub fn data_reference_occurrences(nodes: &[Node], lines: &[Vec<char>]) -> Vec<DataReferenceOccurrence> {
    let mut out = Vec::new();
    collect(nodes, lines, &mut out);
    out
}

fn collect(nodes: &[Node], lines: &[Vec<char>], out: &mut Vec<DataReferenceOccurrence>) {
    for node in nodes {
        match node {
            Node::Statement(stmt) => collect_statement(stmt, out),
            Node::Block(block) => collect_block(block, lines, out),
        }
    }
}

fn collect_block(block: &Block, lines: &[Vec<char>], out: &mut Vec<DataReferenceOccurrence>) {
    for span in &block.opener_pairs {
        if let Some(text) = text_at_span(lines, *span) {
            if is_data_reference_name(&text) {
                out.push(DataReferenceOccurrence { name: text.to_ascii_uppercase(), span: *span });
            }
        }
    }
    match &block.kind {
        BlockKind::If { branches } => {
            for branch in branches {
                if let Some(condition) = &branch.condition {
                    collect_tokens(condition, out);
                }
                collect(&branch.children, lines, out);
            }
        }
        _ => collect(&block.children, lines, out),
    }
}

fn collect_statement(stmt: &Statement, out: &mut Vec<DataReferenceOccurrence>) {
    // Casing (of any category) never targets Label/ShellEscape content —
    // mirrors format.rs's own control_words/pair_keywords scope (FR-015 in
    // 002-cli-check-format). Control and Assignment statements are both
    // in scope: a data-reference token can be a pair-keyword name or a
    // value inside a Control statement, or the target/value of an
    // Assignment.
    if matches!(stmt.kind, StatementKind::Label { .. } | StatementKind::ShellEscape { .. }) {
        return;
    }
    collect_tokens(&stmt.tokens, out);
}

/// Quote-aware scan over `tokens`: every `Word` token found *outside* a
/// single-/double-quoted run is checked against the dot-notation and
/// exact-match rules. Mirrors `statement.rs`'s `pair_keyword_boundaries`
/// quote-tracking (module docs) — without it, data-reference-shaped text
/// inside a quoted string literal (e.g. a `PRINT LIST='...'` message) would
/// be wrongly recognized and rewritten.
fn collect_tokens(tokens: &[Token], out: &mut Vec<DataReferenceOccurrence>) {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    for tok in tokens {
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
        if let Some(prefix_len) = dot_notation_prefix_len(&tok.text) {
            let prefix: String = tok.text.chars().take(prefix_len).collect();
            let end = Position::new(tok.span.start.line, tok.span.start.column + prefix_len as u32);
            out.push(DataReferenceOccurrence {
                name: prefix.to_ascii_uppercase(),
                span: Span::new(tok.span.start, end),
            });
        } else if is_data_reference_name(&tok.text) {
            out.push(DataReferenceOccurrence {
                name: tok.text.to_ascii_uppercase(),
                span: tok.span,
            });
        }
    }
}

/// Extracts the literal text at `span` from `lines` — the same lookup
/// `format.rs`'s own `edit_for_span` performs, factored out here since
/// `Block::opener_pairs` carries only spans, never the original token.
/// Returns `None` (rather than panicking) if `span` doesn't resolve to
/// real content, matching this crate's never-panics contract. `pub(crate)`
/// so `format.rs`'s own pair-keyword collection can use the identical
/// lookup to decide whether a given opener-pair name is data-reference-
/// owned (FR-005) before applying `pair_keywords` casing to it.
pub(crate) fn text_at_span(lines: &[Vec<char>], span: Span) -> Option<String> {
    let line_chars = lines.get((span.start.line - 1) as usize)?;
    let start = (span.start.column - 1) as usize;
    let end = (span.end.column - 1) as usize;
    if start > end || end > line_chars.len() {
        return None;
    }
    Some(line_chars[start..end].iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn occurrences(source: &str) -> Vec<DataReferenceOccurrence> {
        let parsed = parse(source);
        let lines: Vec<Vec<char>> = source.lines().map(|l| l.chars().collect()).collect();
        data_reference_occurrences(&parsed.nodes, &lines)
    }

    #[test]
    fn matrix_family_dot_notation_read() {
        let occ = occurrences("X = mi.1.1 + mi.2.1\n");
        let names: Vec<&str> = occ.iter().map(|o| o.name.as_str()).collect();
        assert_eq!(names, vec!["MI", "MI"]);
    }

    #[test]
    fn matrix_family_assignment_target_and_readback() {
        let occ = occurrences("MW[1] = mi.1.1\nX = mw[1]\n");
        let names: Vec<&str> = occ.iter().map(|o| o.name.as_str()).collect();
        assert!(names.contains(&"MW"));
        assert!(names.contains(&"MI"));
    }

    #[test]
    fn mw_pair_keyword_shaped_and_assignment_target_both_match_as_mw() {
        // FR-005: one name, regardless of structural shape.
        let occ = occurrences("PATHLOAD PATH=TIME, MW[201]=mi.1.1\nMW[1] = mi.1.1\n");
        let mw_count = occ.iter().filter(|o| o.name == "MW").count();
        assert_eq!(mw_count, 2, "expected MW matched once per statement: {occ:?}");
    }

    #[test]
    fn line_family_dot_notation() {
        let occ = occurrences("IF (li.FT > 0) X = 1\n");
        assert!(occ.iter().any(|o| o.name == "LI"), "{occ:?}");
    }

    #[test]
    fn node_family_dot_notation() {
        let occ = occurrences("X = ni.CLASS\n");
        assert!(occ.iter().any(|o| o.name == "NI"), "{occ:?}");
    }

    #[test]
    fn zone_family_zi_zones_and_z() {
        let occ = occurrences("X = zi.1.HBWP2000\nZONES = 1\nSORT=Z\n");
        let names: Vec<&str> = occ.iter().map(|o| o.name.as_str()).collect();
        assert!(names.contains(&"ZI"), "{names:?}");
        assert!(names.contains(&"ZONES"), "{names:?}");
        assert!(names.contains(&"Z"), "{names:?}");
    }

    #[test]
    fn zones_matched_in_run_pgm_matrix_opener_pair() {
        let occ = occurrences("RUN PGM=MATRIX ZONES=3\nX = 1\nENDRUN\n");
        assert!(occ.iter().any(|o| o.name == "ZONES"), "{occ:?}");
    }

    #[test]
    fn database_family_dbi_and_dba() {
        let occ = occurrences("X = dba.1.field\n");
        assert!(occ.iter().any(|o| o.name == "DBA"), "{occ:?}");
    }

    #[test]
    fn record_output_family() {
        let occ = occurrences("RO.POP_2010 = 1\n");
        assert!(occ.iter().any(|o| o.name == "RO"), "{occ:?}");
    }

    #[test]
    fn link_endpoint_and_loop_index_identifiers() {
        let occ = occurrences("PRINT LIST=A(5),B(5)\nIF (I=25) X = J\n");
        let names: Vec<&str> = occ.iter().map(|o| o.name.as_str()).collect();
        assert!(names.contains(&"A"), "{names:?}");
        assert!(names.contains(&"B"), "{names:?}");
        assert!(names.contains(&"I"), "{names:?}");
        assert!(names.contains(&"J"), "{names:?}");
    }

    #[test]
    fn ordinary_user_variable_name_never_matches() {
        let occ = occurrences("ScenarioDir = 'C:\\path'\nX = ScenarioDir\n");
        assert!(occ.is_empty(), "{occ:?}");
    }

    #[test]
    fn quoted_string_content_never_matches() {
        // A/I/MW-shaped text inside a quoted PRINT string must never be
        // treated as a real occurrence (the same class of bug
        // pair_keyword_boundaries was once fixed for).
        let occ = occurrences("PRINT LIST='value is mi.1.1 and I and A'\n");
        assert!(occ.is_empty(), "{occ:?}");
    }

    #[test]
    fn numeric_literal_with_a_dot_never_misparsed_as_dot_notation() {
        let occ = occurrences("X = 1.5\n");
        assert!(occ.is_empty(), "{occ:?}");
    }
}
