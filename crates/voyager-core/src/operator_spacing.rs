//! Operator/comma/bracket-paren spacing normalization (018-operator-spacing;
//! spec.md FR-002–FR-008). A read-only recognition pass over an
//! already-tokenized `Statement`'s own token list — no lexer/`TokenKind`
//! change (research.md §1, §2, §5) — mirroring `data_reference.rs`'s
//! self-contained-module shape from `017`.
//!
//! **Quoted-literal safety (research.md §9)**: confirmed by direct testing
//! that `tokenize("LIST='a+b'\n")` emits a standalone `Punctuation("+")`
//! token for the `+` *inside* the quotes, indistinguishable from a real
//! operator at the `TokenKind` level. Every recognition function here
//! consults [`quoted_token_mask`] first and never treats a masked token as
//! an operator/comma/bracket-paren occurrence.

use std::collections::BTreeSet;

use crate::block::BlockKind;
use crate::format::SpacingEdit;
use crate::span::Position;
use crate::statement::{pair_keyword_boundaries, Statement, StatementKind};
use crate::token::{Token, TokenKind};
use crate::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperatorKind {
    Assignment,
    Comparison,
    Arithmetic,
}

struct OperatorOccurrence {
    start_index: usize,
    /// Exclusive — 1 past the last token forming this operator (2 for a
    /// merged multi-char comparison, 1 otherwise).
    end_index: usize,
    span: crate::span::Span,
    /// True when the *last* token forming this occurrence carries
    /// `TokenKind::ContinuationMarker` (research.md §3) — suppresses the
    /// trailing-side edit entirely, since nothing follows it on this line.
    is_continuation: bool,
    /// Target gap width on both sides of this occurrence (023-range-dash-
    /// spacing data-model.md §1). `1` for every operator kind except a
    /// qualifying range dash, which wants `0` — a binary `-` inside a
    /// `Control` statement's pair-keyword value with a bare integer literal
    /// directly adjacent on both sides (see `is_range_dash`).
    want_spaces: usize,
}

/// Tracks which token indices in `tokens` fall inside an open string/quoted
/// literal (research.md §9) — an odd running count of `'`/`"` tokens seen so
/// far means "inside a string"; an unmatched trailing quote treats every
/// token after it as inside a string too (fail toward exclusion). Mirrors
/// the same naive per-quote-character toggle `statement.rs`'s
/// `pair_keyword_boundaries` already uses for the identical reason.
pub(crate) fn quoted_token_mask(tokens: &[Token]) -> Vec<bool> {
    let mut mask = vec![false; tokens.len()];
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    for (i, tok) in tokens.iter().enumerate() {
        mask[i] = in_single_quote || in_double_quote;
        if tok.kind == TokenKind::Punctuation {
            match tok.text.as_str() {
                "'" if !in_double_quote => in_single_quote = !in_single_quote,
                "\"" if !in_single_quote => in_double_quote = !in_double_quote,
                _ => {}
            }
        }
    }
    mask
}

fn is_operator_punct(tok: &Token) -> bool {
    matches!(tok.kind, TokenKind::Punctuation | TokenKind::ContinuationMarker)
}

/// A `+`/`-` at `tokens[index]` is binary unless nothing precedes it (start
/// of the value) or the immediately preceding token is itself `=`, `(`, `,`,
/// or another recognized operator (research.md §5) — matches spec.md
/// FR-003/Assumptions exactly, including the "or another operator" case
/// (`A + -B`) a first draft of the spec's own wording once omitted.
fn is_binary_arithmetic(tokens: &[Token], index: usize) -> bool {
    if index == 0 {
        return false;
    }
    let prev = &tokens[index - 1];
    if prev.kind == TokenKind::ContinuationMarker {
        return false;
    }
    if prev.kind == TokenKind::Punctuation
        && matches!(prev.text.as_str(), "=" | "(" | "," | "+" | "-" | "*" | "/" | "<" | ">")
    {
        return false;
    }
    true
}

/// A bare integer literal (023-range-dash-spacing data-model.md §1,
/// research.md §3): a `Word` token whose text is non-empty and consists
/// entirely of ASCII digits. `.` is not a lexer delimiter (`lexer.rs::
/// is_delimiter`), so a decimal number (`1.5`) or a dotted data-reference
/// (`mi.1.1`) already tokenizes as one `Word` token containing a non-digit
/// character — excluded here by construction, no extra logic needed.
fn is_bare_integer_literal(tok: &Token) -> bool {
    tok.kind == TokenKind::Word && !tok.text.is_empty() && tok.text.chars().all(|c| c.is_ascii_digit())
}

/// Whether the binary `-` at `stmt.tokens[index]` is a range dash
/// (023-range-dash-spacing FR-001/FR-002): inside a `Control` statement's
/// pair-keyword value, with a bare integer literal directly adjacent on
/// both sides. Reuses `pair_keyword_boundaries` — the same value-boundary
/// data `collect_comma_edits` already derives for the identical statement
/// (FR-010) — rather than a separately-maintained notion of where a value
/// starts and ends. Only ever called for an occurrence `is_binary_arithmetic`
/// already accepted, so a unary `-` never reaches this check (FR-005).
fn is_range_dash(stmt: &Statement, index: usize) -> bool {
    let StatementKind::Control { .. } = &stmt.kind else {
        return false;
    };
    let tokens = &stmt.tokens;
    if index == 0 || index + 1 >= tokens.len() {
        return false;
    }
    let boundaries = pair_keyword_boundaries(tokens);
    let in_a_value = boundaries.iter().enumerate().any(|(i, &(_, eq_idx))| {
        let value_start = eq_idx + 1;
        let value_end = boundaries.get(i + 1).map(|p| p.0).unwrap_or(tokens.len());
        (value_start..value_end).contains(&index)
    });
    in_a_value && is_bare_integer_literal(&tokens[index - 1]) && is_bare_integer_literal(&tokens[index + 1])
}

/// Recognizes assignment/comparison/binary-arithmetic operator occurrences
/// in `stmt.tokens` (research.md §1, §2, §5) — never commas (a separate,
/// pair-boundary-aware rule, [`collect_comma_edits`]) and never a token
/// [`quoted_token_mask`] marks as inside a string literal.
fn recognize_operators(stmt: &Statement) -> Vec<OperatorOccurrence> {
    let tokens = &stmt.tokens;
    let mask = quoted_token_mask(tokens);
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if mask[i] || !is_operator_punct(&tokens[i]) {
            i += 1;
            continue;
        }
        let tok = &tokens[i];
        let text = tok.text.as_str();

        // Multi-char comparison merge: two zero-gap-adjacent single tokens
        // drawn from {=, <, >} forming ==, <>, >=, <= (research.md §2).
        if matches!(text, "=" | "<" | ">") && i + 1 < tokens.len() && !mask[i + 1] {
            let next = &tokens[i + 1];
            if is_operator_punct(next) && matches!(next.text.as_str(), "=" | "<" | ">") && tok.span.end == next.span.start {
                let merged = format!("{}{}", text, next.text);
                if matches!(merged.as_str(), "==" | "<>" | ">=" | "<=") {
                    out.push(OperatorOccurrence {
                        start_index: i,
                        end_index: i + 2,
                        span: tok.span.merge(next.span),
                        is_continuation: next.kind == TokenKind::ContinuationMarker,
                        want_spaces: 1,
                    });
                    i += 2;
                    continue;
                }
            }
        }

        let kind = match text {
            "=" => Some(OperatorKind::Assignment),
            "<" | ">" => Some(OperatorKind::Comparison),
            "+" | "-" => is_binary_arithmetic(tokens, i).then_some(OperatorKind::Arithmetic),
            "*" | "/" => Some(OperatorKind::Arithmetic),
            _ => None,
        };
        if kind.is_some() {
            // 023-range-dash-spacing FR-001: a binary `-` that's also a
            // range dash wants zero surrounding whitespace instead of the
            // one space every other operator occurrence wants.
            let want_spaces = if text == "-" && is_range_dash(stmt, i) { 0 } else { 1 };
            out.push(OperatorOccurrence {
                start_index: i,
                end_index: i + 1,
                span: tok.span,
                is_continuation: tok.kind == TokenKind::ContinuationMarker,
                want_spaces,
            });
        }
        i += 1;
    }
    out
}

/// The characters in `lines[from.line-1][from.column-1 .. to.column-1]`, or
/// `None` if `from`/`to` don't resolve to a single-line, well-formed range.
fn gap_chars(lines: &[Vec<char>], from: Position, to: Position) -> Option<&[char]> {
    if from.line != to.line {
        return None;
    }
    let line_chars = lines.get((from.line - 1) as usize)?;
    let start = (from.column - 1) as usize;
    let end = (to.column - 1) as usize;
    if start > end || end > line_chars.len() {
        return None;
    }
    Some(&line_chars[start..end])
}

/// Queues an edit normalizing the gap from `from` to `to` (assumed to be
/// two adjacent tokens' boundary positions) to exactly `want_spaces` spaces
/// — but only when that gap is confirmed pure whitespace first. A gap that
/// contains anything else (in practice: a mid-expression block comment,
/// e.g. `MW[1] = /* note */ 5` — tokens are adjacent in the token stream
/// but not in the source) is left untouched rather than corrupted; this is
/// a deliberate safety guard beyond what any single FR states outright, in
/// the same spirit as `quoted_token_mask`'s "fail toward exclusion, never
/// toward false recognition."
fn push_gap_edit(lines: &[Vec<char>], edits: &mut Vec<SpacingEdit>, from: Position, to: Position, want_spaces: usize) {
    let Some(chars) = gap_chars(lines, from, to) else {
        return;
    };
    if !chars.iter().all(|c| *c == ' ' || *c == '\t') {
        return;
    }
    if chars.len() == want_spaces {
        return; // already correct -- no-op, preserves idempotence
    }
    edits.push((from.line, (from.column - 1) as usize, (to.column - 1) as usize, " ".repeat(want_spaces)));
}

/// `Fixed`'s operator-spacing rule (FR-002/FR-003/FR-012): one space on each
/// side of every recognized operator, leading-side-only for a trailing
/// continuation-position occurrence — except a range dash
/// (023-range-dash-spacing FR-001), which wants zero on each side instead.
pub(crate) fn collect_operator_edits(stmt: &Statement, lines: &[Vec<char>], edits: &mut Vec<SpacingEdit>) {
    let tokens = &stmt.tokens;
    for occ in recognize_operators(stmt) {
        if occ.start_index > 0 {
            push_gap_edit(lines, edits, tokens[occ.start_index - 1].span.end, occ.span.start, occ.want_spaces);
        }
        if !occ.is_continuation && occ.end_index < tokens.len() {
            push_gap_edit(lines, edits, occ.span.end, tokens[occ.end_index].span.start, occ.want_spaces);
        }
    }
}

/// `Fixed`'s comma-spacing rule (FR-004): exactly one space after, none
/// before, a `,` that separates two `keyword=value` pairs on one `Control`
/// statement. **Scoped precisely to pair-separator commas** — a comma
/// *inside* a single pair's own value (e.g. `LOOP i=1,5,1`'s start/end/
/// increment commas) is never touched, since it never sits immediately
/// before another pair's keyword. Reuses `pair_keyword_boundaries` (the
/// same pair-start detection `format.rs`'s casing rewrite and `block.rs`'s
/// opener-pair capture already share) rather than a naive "every top-level
/// comma" scan, which would have wrongly touched `LOOP`'s internal commas.
pub(crate) fn collect_comma_edits(stmt: &Statement, lines: &[Vec<char>], edits: &mut Vec<SpacingEdit>) {
    let StatementKind::Control { .. } = &stmt.kind else {
        return;
    };
    let boundaries = pair_keyword_boundaries(&stmt.tokens);
    let mask = quoted_token_mask(&stmt.tokens);
    for &(kw_start, _eq_idx) in boundaries.iter().skip(1) {
        if kw_start == 0 || mask[kw_start - 1] {
            continue;
        }
        let comma_idx = kw_start - 1;
        let comma = &stmt.tokens[comma_idx];
        if !is_operator_punct(comma) || comma.text != "," {
            continue;
        }
        if comma_idx > 0 {
            push_gap_edit(lines, edits, stmt.tokens[comma_idx - 1].span.end, comma.span.start, 0);
        }
        if comma.kind != TokenKind::ContinuationMarker {
            push_gap_edit(lines, edits, comma.span.end, stmt.tokens[kw_start].span.start, 1);
        }
    }
}

/// `Fixed`'s bracket/paren rules (FR-005): zero interior padding inside
/// `[...]`/`(...)`, and zero space between a leading control word and an
/// immediately-following `(` (the short-form `IF(x)` case). Both are local,
/// adjacent-token-pair rules — no bracket *matching*/depth-tracking is
/// needed, since "the token right after an opener" and "the token right
/// before a closer" are well-defined without knowing which opener a given
/// closer pairs with.
pub(crate) fn collect_bracket_paren_edits(tokens: &[Token], lines: &[Vec<char>], edits: &mut Vec<SpacingEdit>) {
    let mask = quoted_token_mask(tokens);

    if tokens.len() >= 2 && !mask[0] && !mask[1] && tokens[0].kind == TokenKind::Word && is_operator_punct(&tokens[1]) && tokens[1].text == "(" {
        push_gap_edit(lines, edits, tokens[0].span.end, tokens[1].span.start, 0);
    }

    for i in 0..tokens.len() {
        if mask[i] || !is_operator_punct(&tokens[i]) {
            continue;
        }
        let text = tokens[i].text.as_str();
        if matches!(text, "[" | "(") && i + 1 < tokens.len() && !mask[i + 1] {
            push_gap_edit(lines, edits, tokens[i].span.end, tokens[i + 1].span.start, 0);
        }
        if matches!(text, "]" | ")") && i > 0 && !mask[i - 1] {
            let prev = &tokens[i - 1];
            // Skip when prev is itself an opener (`()`/`[]`/`( )`) -- that
            // exact gap was already queued by the opener-side rule above;
            // queuing it again here would duplicate the same (line, start,
            // end) edit.
            let prev_is_opener = is_operator_punct(prev) && matches!(prev.text.as_str(), "[" | "(");
            if !prev_is_opener {
                push_gap_edit(lines, edits, prev.span.end, tokens[i].span.start, 0);
            }
        }
    }
}

/// All of `Fixed`'s edits for one statement (spec.md FR-002–FR-005,
/// FR-012) — the entry point `format.rs::render` calls once per statement
/// in the flat statement list (research.md §1: this needs the *flat* list,
/// not the parsed `Node`/`Block` tree, since a block opener's own tokens
/// — e.g. `RUN PGM=MATRIX ZONES=5`'s pairs, or an `IF(x==1)`'s condition —
/// are only fully retained on the flat `Statement`, not on `Block`'s own
/// lossy `opener_pairs: Vec<Span>`).
///
/// **`ShellEscape` is entirely excluded** — a real corpus bug this crate's
/// own module docs already warn about: `statement.rs`'s `ShellEscape`
/// variant stores "the command text that follows... opaquely, never parsed
/// as Voyager grammar" (FR-022), but its `Statement.tokens` still includes
/// that raw shell text like any other statement's tokens field would. Found
/// via the real 161-file corpus (`AssignHwy/09_TAZ_Based_Metrics.s`):
/// without this exclusion, a `**` double-star shell-escape marker gets
/// misread as a multiplication operator, and `1>&2` shell redirection syntax
/// gets misread as a comparison — silently corrupting a shell command,
/// exactly the kind of "never parsed as Voyager grammar" violation FR-022
/// exists to prevent. Confirmed by direct testing, not merely reasoned
/// about: this fixture failed the real-corpus idempotence check before this
/// exclusion existed.
pub(crate) fn collect_fixed_edits(stmt: &Statement, lines: &[Vec<char>], edits: &mut Vec<SpacingEdit>) {
    if matches!(stmt.kind, StatementKind::ShellEscape { .. }) {
        return;
    }
    collect_operator_edits(stmt, lines, edits);
    collect_bracket_paren_edits(&stmt.tokens, lines, edits);
    collect_comma_edits(stmt, lines, edits);
}

// ---------------------------------------------------------------------
// Alignment (Auto only, spec.md FR-006-FR-008, data-model.md §3)
// ---------------------------------------------------------------------

fn is_protected_line(protected: &BTreeSet<u32>, line: u32) -> bool {
    protected.contains(&line)
}

/// For an `Assignment` statement, the index of its own `=` token within
/// `stmt.tokens` — `value` is always a suffix of `tokens` by construction
/// (`statement.rs::classify_statement`), so this needs no independent
/// re-derivation of `assignment_equals_index`.
fn assignment_eq_index(stmt: &Statement, value_len: usize) -> Option<usize> {
    let idx = stmt.tokens.len().checked_sub(value_len)?.checked_sub(1)?;
    (idx < stmt.tokens.len()).then_some(idx)
}

/// The rendered (post-`Fixed`-edit) 0-based column immediately after an
/// `Assignment` statement's left-hand side, on the line its own `=` sits
/// on — `None` if the target and `=` aren't on the same line (a rare,
/// conservatively-skipped case; data-model.md §3 only defines alignment in
/// terms of the `=`'s own line position). Computed by summing the
/// character-length delta of every already-known `Fixed` edit that lies
/// entirely before this position on the same line — casing edits never
/// contribute (same-length by construction), so only `fixed_edits` matters.
/// Returns `(line, raw_target_end_col, rendered_target_end_col)` — the raw
/// value is the target's real end column in *this line's own original
/// text* (needed for the edit span this function's caller ultimately
/// queues, which — like every other edit in this module — must be
/// expressed in original source coordinates); the rendered value is that
/// same position after accounting for `fixed_edits`, used only to *compute
/// how much padding is needed*. Conflating the two was a real bug (found by
/// this feature's own idempotence test, T020/T027): comparing a rendered
/// quantity against a raw one made an already-correctly-aligned run look
/// "still needs padding" or "no longer needs padding" inconsistently
/// between passes, since `Fixed`'s own edit for the same gap doesn't know
/// what alignment decided.
fn rendered_target_end_column(stmt: &Statement, eq_idx: usize, fixed_edits: &[SpacingEdit]) -> Option<(u32, usize, usize)> {
    if eq_idx == 0 {
        return None;
    }
    let last_target_tok = &stmt.tokens[eq_idx - 1];
    let eq_tok = &stmt.tokens[eq_idx];
    let line = last_target_tok.span.end.line;
    if eq_tok.span.start.line != line {
        return None;
    }
    let raw_col = (last_target_tok.span.end.column - 1) as usize;
    let mut delta: isize = 0;
    for (l, start, end, replacement) in fixed_edits {
        // Strict `<`, not `<=`: an edit whose `end` lands exactly at
        // `raw_col` is the "before =" gap edit itself (its span is
        // `(raw_col, eq_start_col)`, and when the original gap is
        // zero-width — target immediately followed by `=`, no space —
        // `eq_start_col == raw_col` too, so `end <= raw_col` would
        // wrongly match this edit against itself). A genuinely earlier
        // edit (e.g. bracket-interior padding inside the target) always
        // ends strictly before `raw_col`, so this exclusion never drops a
        // real contributor — found via the real corpus
        // (`Distribute/4pd_mainbody_distribution.block`'s `lw.RampPen_10=
        // max(...)`, an original zero-gap assignment): without it, this
        // one member's own padding edit got double-counted into its own
        // rendered width, inflating `align_col` for the whole run by one
        // on the first format pass only — a real idempotence bug, not a
        // hypothetical one.
        if *l == line && *end < raw_col {
            delta += replacement.chars().count() as isize - (*end as isize - *start as isize);
        }
    }
    Some((line, raw_col, (raw_col as isize + delta).max(0) as usize))
}

/// Computes and queues alignment padding for one run (data-model.md §3): the
/// run's `target_column` is one past the widest *rendered* left-hand side;
/// every member's "before `=`" gap gets an edit setting it to exactly the
/// padding needed to reach that column — **unconditionally, for every
/// member of a multi-member run**, never skipped even when a member
/// already looks correctly positioned. Skipping would leave `Fixed`'s own
/// (unaware-of-alignment) edit for that same gap unchallenged in the merged
/// edit set, silently reverting the alignment on a second pass — the exact
/// idempotence bug this comment's sibling doc comment describes. A no-op
/// case (nothing actually changes) still renders identically; it's simply
/// never assumed to be one without a matching edit to make it official.
/// A member whose rendered position can't be safely computed (see
/// `rendered_target_end_column`) causes the *whole run* to be skipped —
/// conservative, but never wrong.
fn emit_alignment_padding(siblings: &[Node], run_members: &[usize], fixed_edits: &[SpacingEdit], edits: &mut Vec<SpacingEdit>) {
    struct MemberInfo {
        line: u32,
        raw_target_end_col: usize,
        rendered_target_end_col: usize,
        eq_start_col: usize,
    }
    let mut infos = Vec::with_capacity(run_members.len());
    for &idx in run_members {
        let Node::Statement(stmt) = &siblings[idx] else {
            return;
        };
        let StatementKind::Assignment { value, .. } = &stmt.kind else {
            return;
        };
        let Some(eq_idx) = assignment_eq_index(stmt, value.len()) else {
            return;
        };
        let eq_tok = &stmt.tokens[eq_idx];
        let Some((line, raw_target_end_col, rendered_target_end_col)) = rendered_target_end_column(stmt, eq_idx, fixed_edits) else {
            return;
        };
        if eq_tok.span.start.line != line {
            return;
        }
        infos.push(MemberInfo {
            line,
            raw_target_end_col,
            rendered_target_end_col,
            eq_start_col: (eq_tok.span.start.column - 1) as usize,
        });
    }
    let Some(align_col) = infos.iter().map(|m| m.rendered_target_end_col + 1).max() else {
        return;
    };
    for m in &infos {
        // Always emit — even a "no visible change" case must still exist as
        // its own edit at this exact (line, raw_target_end_col, eq_start_col)
        // span, so it wins the by-key merge in format.rs::render against
        // whatever Fixed independently computed for that same span (which
        // has no idea alignment exists). Skipping "because it looks
        // already correct" is exactly the bug this function's doc comment
        // describes — the replacement content may be unchanged, but the
        // *edit's presence* is what keeps Fixed's own conflicting default
        // from silently winning instead.
        let padding = align_col.saturating_sub(m.rendered_target_end_col).max(1);
        edits.push((m.line, m.raw_target_end_col, m.eq_start_col, " ".repeat(padding)));
    }
}

/// Walks one `Vec<Node>` sibling slice, grouping maximal runs of
/// consecutive `Node::Statement(Assignment)` entries and emitting alignment
/// padding for every run with more than one member (data-model.md §3). A
/// run breaks at: any non-`Assignment` sibling (including a pair-keyword-
/// shaped `Control` statement, FR-007), a protected (`; FMT: OFF`) member
/// (FR-008 — excluded entirely, never counted toward a neighbor's column),
/// or a gap between two statements' lines wider than exactly one (which
/// uniformly covers a blank-line break *and* a comment-only-line break —
/// either way, the next statement's first line is more than one past the
/// previous statement's last line, so a single adjacency check handles
/// both without needing to separately classify what's on the line between
/// them).
fn collect_alignment_edits_in_slice(siblings: &[Node], protected: &BTreeSet<u32>, fixed_edits: &[SpacingEdit], edits: &mut Vec<SpacingEdit>) {
    let mut i = 0;
    while i < siblings.len() {
        let is_run_start = match &siblings[i] {
            Node::Statement(stmt) => matches!(stmt.kind, StatementKind::Assignment { .. }) && !is_protected_line(protected, stmt.span.start.line),
            Node::Block(_) => false,
        };
        if !is_run_start {
            if let Node::Block(block) = &siblings[i] {
                match &block.kind {
                    BlockKind::If { branches } => {
                        for branch in branches {
                            collect_alignment_edits_in_slice(&branch.children, protected, fixed_edits, edits);
                        }
                    }
                    _ => collect_alignment_edits_in_slice(&block.children, protected, fixed_edits, edits),
                }
            }
            i += 1;
            continue;
        }

        let mut run: Vec<usize> = vec![i];
        let mut prev_end_line = siblings[i].span().end.line;
        let mut j = i + 1;
        while j < siblings.len() {
            let Node::Statement(next_stmt) = &siblings[j] else {
                break;
            };
            if !matches!(next_stmt.kind, StatementKind::Assignment { .. }) || is_protected_line(protected, next_stmt.span.start.line) {
                break;
            }
            if next_stmt.span.start.line != prev_end_line + 1 {
                break;
            }
            run.push(j);
            prev_end_line = next_stmt.span.end.line;
            j += 1;
        }
        if run.len() > 1 {
            emit_alignment_padding(siblings, &run, fixed_edits, edits);
        }
        i = j;
    }
}

/// `Auto`'s alignment pass (data-model.md §3) — call after `Fixed`'s edits
/// for every statement are already known, so alignment padding is always
/// computed on top of `Fixed`'s own spacing decisions, never a different
/// one (contracts/operator-spacing.md).
pub(crate) fn collect_alignment_edits(nodes: &[Node], protected: &BTreeSet<u32>, fixed_edits: &[SpacingEdit], edits: &mut Vec<SpacingEdit>) {
    collect_alignment_edits_in_slice(nodes, protected, fixed_edits, edits);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::statement::build_statements;

    fn stmt_tokens(src: &str) -> Vec<Token> {
        build_statements(tokenize(src)).remove(0).tokens
    }

    fn char_lines(src: &str) -> Vec<Vec<char>> {
        src.lines().map(|l| l.chars().collect()).collect()
    }

    fn fixed_edits_for(src: &str) -> Vec<SpacingEdit> {
        let stmt = build_statements(tokenize(src)).remove(0);
        let lines = char_lines(src);
        let mut edits = Vec::new();
        collect_fixed_edits(&stmt, &lines, &mut edits);
        edits
    }

    fn apply(src: &str, edits: &[SpacingEdit]) -> String {
        let mut by_line: std::collections::BTreeMap<u32, Vec<(usize, usize, String)>> = Default::default();
        for (l, s, e, r) in edits {
            by_line.entry(*l).or_default().push((*s, *e, r.clone()));
        }
        let mut out_lines = Vec::new();
        for (idx, line) in src.lines().enumerate() {
            let line_num = (idx + 1) as u32;
            let chars: Vec<char> = line.chars().collect();
            if let Some(line_edits) = by_line.get(&line_num) {
                let mut sorted = line_edits.clone();
                sorted.sort_by_key(|(s, _, _)| *s);
                let mut rebuilt = String::new();
                let mut cursor = 0usize;
                for (s, e, r) in &sorted {
                    rebuilt.push_str(&chars[cursor..*s].iter().collect::<String>());
                    rebuilt.push_str(r);
                    cursor = *e;
                }
                rebuilt.push_str(&chars[cursor..].iter().collect::<String>());
                out_lines.push(rebuilt);
            } else {
                out_lines.push(line.to_string());
            }
        }
        out_lines.join("\n")
    }

    // -- quoted_token_mask ------------------------------------------------

    #[test]
    fn plus_inside_single_quoted_string_is_masked() {
        let toks = stmt_tokens("LIST='a+b'\n");
        let mask = quoted_token_mask(&toks);
        let plus_idx = toks.iter().position(|t| t.kind == TokenKind::Punctuation && t.text == "+").expect("expected a + token");
        assert!(mask[plus_idx], "the + inside 'a+b' must be masked as inside a string");
    }

    // -- operator recognition / Fixed edits --------------------------------

    #[test]
    fn assignment_equals_gets_one_space_each_side() {
        let src = "ZONES   = 1";
        let out = apply(src, &fixed_edits_for(src));
        assert_eq!(out, "ZONES = 1");
    }

    #[test]
    fn two_char_comparison_merges_into_one_gap_not_two() {
        // Note: the control-word-paren rule (FR-005) also strips the space
        // between IF and its `(` unconditionally -- expected output has no
        // space there, same as control_word_paren_adjacency_removed below.
        let src = "IF (I==1)\nENDIF";
        let out = apply(src, &fixed_edits_for(src));
        assert_eq!(out, "IF(I == 1)\nENDIF", "must not produce 'I = = 1' or similar");
    }

    #[test]
    fn all_four_multichar_comparisons_normalize_cleanly() {
        for (input, expected) in [
            ("IF (A<>B)\nENDIF", "IF(A <> B)\nENDIF"),
            ("IF (A>=B)\nENDIF", "IF(A >= B)\nENDIF"),
            ("IF (A<=B)\nENDIF", "IF(A <= B)\nENDIF"),
        ] {
            let out = apply(input, &fixed_edits_for(input));
            assert_eq!(out, expected, "input: {input}");
        }
    }

    #[test]
    fn unary_minus_stays_tight_binary_minus_gets_spaced() {
        let src = "MW[1] = -5";
        let out = apply(src, &fixed_edits_for(src));
        assert_eq!(out, "MW[1] = -5");

        let src2 = "MW[1] = A-B";
        let out2 = apply(src2, &fixed_edits_for(src2));
        assert_eq!(out2, "MW[1] = A - B");
    }

    #[test]
    fn unary_minus_after_another_operator_stays_tight() {
        // spec.md Assumptions: "or another operator" case (A + -B).
        let src = "MW[1] = A+-B";
        let out = apply(src, &fixed_edits_for(src));
        assert_eq!(out, "MW[1] = A + -B");
    }

    // -- 023-range-dash-spacing ---------------------------------------------

    #[test]
    fn range_dash_between_bare_integers_in_a_pair_value_renders_tight() {
        // research.md §5: a bare `SELECTLINK=...` with nothing before it
        // parses as Assignment, not Control -- a leading control word
        // (FILEO here, matching the real corpus shape) is required for this
        // to be a pair-keyword value at all. The pair's own `=` still gets
        // 018's ordinary one-space-each-side treatment (unrelated to this
        // feature) -- only the range dashes themselves render tight.
        let src = "FILEO SELECTLINK=1-50,75,90-100";
        let out = apply(src, &fixed_edits_for(src));
        assert_eq!(
            out, "FILEO SELECTLINK = 1-50,75,90-100",
            "the ranges stay tight; only the pair's own = gets 018's existing spacing"
        );

        for spaced in ["FILEO NODES=200 - 300", "FILEO NODES=200- 300", "FILEO NODES=200 -300"] {
            let out = apply(spaced, &fixed_edits_for(spaced));
            assert_eq!(out, "FILEO NODES = 200-300", "input: {spaced}");
        }
    }

    #[test]
    fn same_pair_internal_commas_are_never_touched_by_either_rule() {
        // The comma rule only ever touches a pair-*boundary* comma
        // (018-operator-spacing FR-004) -- these three commas are all
        // inside SELECTLINK's own single value, never candidates for it.
        // The only edit present is the pair's own `=` (018's existing,
        // unrelated behavior) -- confirm no edit touches any of the commas.
        let src = "FILEO SELECTLINK=1-50,75,90-100";
        let edits = fixed_edits_for(src);
        let comma_cols: Vec<usize> = src.match_indices(',').map(|(i, _)| i).collect();
        for (_, start, _, _) in &edits {
            assert!(
                !comma_cols.contains(start),
                "no edit should touch a same-pair-internal comma, got {edits:?}"
            );
        }
    }

    #[test]
    fn range_dash_only_applies_inside_a_pair_keyword_value() {
        let src = "X = 100-1";
        let out = apply(src, &fixed_edits_for(src));
        assert_eq!(out, "X = 100 - 1", "an Assignment's RHS keeps ordinary binary-arithmetic spacing");

        let src2 = "IF (COUNT-1 == 0)\nENDIF";
        let out2 = apply(src2, &fixed_edits_for(src2));
        assert_eq!(out2, "IF(COUNT - 1 == 0)\nENDIF", "a condition is never a pair-keyword value");
    }

    #[test]
    fn range_dash_requires_a_bare_integer_literal_on_both_sides() {
        let src = "FILEO SELECTLINK=@START@-50";
        let out = apply(src, &fixed_edits_for(src));
        assert_eq!(
            out, "FILEO SELECTLINK = @START@ - 50",
            "a @token@ reference is not a bare integer literal"
        );

        let src2 = "FILEO THRESHOLD=1.5-2.5";
        let out2 = apply(src2, &fixed_edits_for(src2));
        assert_eq!(
            out2, "FILEO THRESHOLD = 1.5 - 2.5",
            "a decimal number is one Word token containing '.', never a bare integer literal"
        );
    }

    #[test]
    fn leading_unary_minus_in_a_pair_value_is_never_a_range_dash_candidate() {
        let src = "FILEO OFFSET=-100,50";
        let out = apply(src, &fixed_edits_for(src));
        assert_eq!(
            out, "FILEO OFFSET = -100,50",
            "unary minus is never a binary occurrence at all -- only the pair's own = is spaced"
        );
    }

    #[test]
    fn range_dash_composes_with_pair_boundary_comma_spacing() {
        // spec.md Acceptance Scenario 6 / FR-006: a pair-boundary comma (a
        // real candidate for 018's comma rule, unlike same-pair-internal
        // commas) sitting immediately next to range-dash values -- both
        // rules apply independently to their own disjoint gaps.
        let src = "FILEO NODES=1-50 ,SELECTLINK=75 - 100";
        let out = apply(src, &fixed_edits_for(src));
        assert_eq!(out, "FILEO NODES = 1-50, SELECTLINK = 75-100");
    }

    #[test]
    fn comma_between_pairs_gets_normalized_comma_inside_loop_value_does_not() {
        // Needs a real Control-shaped statement (a leading word not
        // immediately followed by `=`) for pair_keyword_boundaries to find
        // two distinct pairs -- a bare "MATI=a.mat,..." with no leading
        // control word instead parses as a single Assignment (target
        // "MATI", the rest -- including the comma -- as its opaque value),
        // where the comma-between-pairs rule (Control-only) never applies.
        let src = "FILEI MATI=a.mat,MATO=b.mat";
        let out = apply(src, &fixed_edits_for(src));
        assert_eq!(out, "FILEI MATI = a.mat, MATO = b.mat");

        let loop_src = "LOOP i=1,5,1";
        let loop_out = apply(loop_src, &fixed_edits_for(loop_src));
        assert_eq!(loop_out, "LOOP i = 1,5,1", "LOOP's own start,end,increment commas are not pair separators");
    }

    #[test]
    fn bracket_and_paren_interior_padding_removed() {
        // The control-word-paren rule (FR-005) also strips the space
        // between IF and `(` unconditionally here -- see
        // control_word_paren_adjacency_removed for that rule in isolation.
        let src = "IF ( x==1 )\nENDIF";
        let out = apply(src, &fixed_edits_for(src));
        assert_eq!(out, "IF(x == 1)\nENDIF");

        let src2 = "MW[ 1 ]=mi.1.1+mi.2.1";
        let out2 = apply(src2, &fixed_edits_for(src2));
        assert_eq!(out2, "MW[1] = mi.1.1 + mi.2.1");
    }

    #[test]
    fn control_word_paren_adjacency_removed() {
        let src = "IF (x==1)\nENDIF";
        let out = apply(src, &fixed_edits_for(src));
        assert_eq!(out, "IF(x == 1)\nENDIF");
    }

    #[test]
    fn trailing_continuation_operator_gets_leading_only_spacing() {
        // The continuation exception (FR-012) only suppresses the space
        // *after* the comma itself (nothing follows it on line 1) -- it has
        // no bearing on line 2's own, entirely separate `=` pair-keyword
        // operator, which is normalized independently and normally.
        let src = "FILEI NETI=x,\nZDATI=zonal.dat";
        let out = apply(src, &fixed_edits_for(src));
        assert_eq!(
            out,
            "FILEI NETI = x,\nZDATI = zonal.dat",
            "first line's own = gets full spacing, trailing comma gets nothing inserted after it, second line's own = is unaffected by the continuation and still gets spaced"
        );
    }

    #[test]
    fn quoted_literal_regression_zero_edits_inside_quotes() {
        // The statement's own real `=` (right after LIST, outside the
        // quotes) legitimately gets spaced -- only content *inside* the
        // quotes must never be touched. Assert precisely that: no edit's
        // span starts at or after the opening quote's own column.
        let src = "LIST='a+b'";
        let quote_col = src.find('\'').expect("fixture has a quote");
        let edits = fixed_edits_for(src);
        assert!(
            edits.iter().all(|(_, start, _, _)| *start <= quote_col),
            "expected no edit touching a+b inside quotes, got {edits:?}"
        );

        let src2 = "LIST='x=y'";
        let quote_col2 = src2.find('\'').expect("fixture has a quote");
        let edits2 = fixed_edits_for(src2);
        assert!(
            edits2.iter().all(|(_, start, _, _)| *start <= quote_col2),
            "expected no edit touching x=y inside quotes, got {edits2:?}"
        );
    }

    // -- alignment (Auto) ---------------------------------------------------

    fn alignment_edits_for(src: &str) -> (Vec<SpacingEdit>, Vec<SpacingEdit>) {
        use crate::parse;
        let parsed = parse(src);
        let statements = build_statements(tokenize(src));
        let lines = char_lines(src);
        let mut fixed = Vec::new();
        for stmt in &statements {
            collect_fixed_edits(stmt, &lines, &mut fixed);
        }
        let mut alignment = Vec::new();
        collect_alignment_edits(&parsed.nodes, &BTreeSet::new(), &fixed, &mut alignment);
        (fixed, alignment)
    }

    fn apply_merged(src: &str, fixed: &[SpacingEdit], alignment: &[SpacingEdit]) -> String {
        let mut by_key: std::collections::BTreeMap<(u32, usize, usize), String> = fixed.iter().map(|(l, s, e, r)| ((*l, *s, *e), r.clone())).collect();
        for (l, s, e, r) in alignment {
            by_key.insert((*l, *s, *e), r.clone());
        }
        let merged: Vec<SpacingEdit> = by_key.into_iter().map(|((l, s, e), r)| (l, s, e, r)).collect();
        apply(src, &merged)
    }

    #[test]
    fn consecutive_assignments_align_to_longest_lhs() {
        let src = "A = 1\nBB = 2\nCCC = 3";
        let (fixed, alignment) = alignment_edits_for(src);
        let out = apply_merged(src, &fixed, &alignment);
        assert_eq!(out, "A   = 1\nBB  = 2\nCCC = 3");
    }

    #[test]
    fn blank_line_breaks_alignment_run() {
        let src = "A = 1\nBB = 2\n\nCCC = 3";
        let (fixed, alignment) = alignment_edits_for(src);
        let out = apply_merged(src, &fixed, &alignment);
        assert_eq!(out, "A  = 1\nBB = 2\n\nCCC = 3");
    }

    #[test]
    fn comment_only_line_breaks_alignment_run() {
        let src = "A = 1\nBB = 2\n; a comment\nCCC = 3";
        let (fixed, alignment) = alignment_edits_for(src);
        let out = apply_merged(src, &fixed, &alignment);
        assert_eq!(out, "A  = 1\nBB = 2\n; a comment\nCCC = 3");
    }

    #[test]
    fn control_statement_among_assignments_splits_the_run() {
        let src = "A = 1\nPHASE=ILOOP\nCCC = 3";
        let (fixed, alignment) = alignment_edits_for(src);
        assert!(alignment.is_empty(), "PHASE's own = must never join or extend a run: {alignment:?}");
        let out = apply_merged(src, &fixed, &alignment);
        assert_eq!(out, "A = 1\nPHASE = ILOOP\nCCC = 3");
    }

    #[test]
    fn lone_assignment_renders_identically_to_fixed_alone() {
        let src = "A = 1";
        let (fixed, alignment) = alignment_edits_for(src);
        assert!(alignment.is_empty());
        assert!(fixed.is_empty(), "already correctly spaced, Fixed itself is a no-op too");
    }

    // -- Real-corpus regressions found during Polish (tasks.md T031) --------

    #[test]
    fn shell_escape_command_content_is_never_touched() {
        // Real corpus bug (AssignHwy/09_TAZ_Based_Metrics.s): a `**`
        // double-star shell-escape marker was misread as multiplication,
        // and `1>&2` shell redirection syntax was misread as a comparison
        // -- both silently corrupting a shell command. FR-022: ShellEscape
        // content is never parsed as Voyager grammar, full stop.
        let src = "**\"python.exe\" \"script.py\" 1>&2";
        let edits = fixed_edits_for(src);
        assert!(edits.is_empty(), "ShellEscape command text must never be touched, got {edits:?}");
    }

    #[test]
    fn shell_escape_with_bang_marker_is_also_never_touched() {
        let src = "* \"cmd.exe\" /c \"echo 1*2\"";
        let edits = fixed_edits_for(src);
        assert!(edits.is_empty(), "got {edits:?}");
    }

    #[test]
    fn alignment_run_with_a_zero_gap_member_computes_correct_padding() {
        // Real corpus bug (Distribute/4pd_mainbody_distribution.block): a
        // run's own longest member had literally no space before its `=`
        // in the original source (`lw.RampPen_10= max(...)`, not `... = ...`).
        // The delta-sum that computes each member's *rendered* width
        // double-counted that member's own about-to-be-emitted padding
        // edit, inflating align_col by one -- but only on the very first
        // format pass (a fresh idempotence-breaking miscomputation, not a
        // stable wrong answer), since a second pass started from text that
        // already had a real gap there and no longer triggered the
        // zero-width boundary case.
        let src = "lw.RampPen_1 = max(1)\nlw.RampPen_10= max(2)";
        let (fixed, alignment) = alignment_edits_for(src);
        let out = apply_merged(src, &fixed, &alignment);
        assert_eq!(
            out, "lw.RampPen_1  = max(1)\nlw.RampPen_10 = max(2)",
            "both members must align to one column past the longer (13-char) name, not 14"
        );
        // Idempotence: re-running on the already-aligned output must be a
        // pure no-op, not drift by one column again.
        let (fixed2, alignment2) = alignment_edits_for(&out);
        let out2 = apply_merged(&out, &fixed2, &alignment2);
        assert_eq!(out2, out);
    }
}
