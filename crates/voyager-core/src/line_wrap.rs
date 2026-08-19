//! Automatic line-width wrapping for `Control` statements
//! (030-auto-line-wrap, data-model.md §1, research.md §4) — a read-only
//! recognition/planning pass over an already-tokenized `Statement`'s flat
//! token list, feeding `format.rs::render`'s existing `SpacingEdit`
//! mechanism. No I/O, no parsing, never panics.

use crate::format::{LineWrapStyle, SpacingEdit};
use crate::operator_spacing::quoted_token_mask;
use crate::span::Span;
use crate::token::{Token, TokenKind};

/// A comma token eligible as a wrap split point: a `,` `Punctuation` token
/// at a `Control` statement's own top level (research.md §4) — never one
/// nested inside a function call's parentheses or a bracketed subscript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SplitPoint {
    pub(crate) span: Span,
}

/// `true` if `tokens` contains any `TokenKind::ContinuationMarker` — FR-005's
/// "already continued" check. A statement for which this is `true` is never
/// touched by this module at all; callers MUST check this before calling any
/// other function here (data-model.md §1, the mechanism this feature's
/// idempotence relies on).
pub(crate) fn already_continued(tokens: &[Token]) -> bool {
    tokens.iter().any(|t| t.kind == TokenKind::ContinuationMarker)
}

/// Every top-level `,` `Punctuation` token in `tokens` — a `Control`
/// statement's own flat token list. Tracks paren `(`/`)` and bracket `[`/`]`
/// depth; a comma nested inside either is never collected. **Correction
/// during implementation** (research.md §4's original claim was wrong,
/// caught by a dedicated test rather than assumed): a quoted pair-value is
/// *not* lexed as one atomic token in this grammar — `'a, b'` lexes as
/// separate `'`/`a`/`,`/`b`/`'` tokens, the exact same shape
/// `operator_spacing.rs` already had to solve for. This function therefore
/// reuses that module's own `quoted_token_mask` (odd running count of
/// `'`/`"` `Punctuation` tokens seen so far == inside a string) rather than
/// duplicating that logic — a comma masked as "inside a string" is never
/// collected, regardless of paren/bracket depth.
pub(crate) fn top_level_split_points(tokens: &[Token]) -> Vec<SplitPoint> {
    let mask = quoted_token_mask(tokens);
    let mut depth: i32 = 0;
    let mut out = Vec::new();
    for (i, t) in tokens.iter().enumerate() {
        if t.kind != TokenKind::Punctuation || mask[i] {
            continue;
        }
        match t.text.as_str() {
            "(" | "[" => depth += 1,
            ")" | "]" => depth -= 1,
            "," if depth == 0 => out.push(SplitPoint { span: t.span }),
            _ => {}
        }
    }
    out
}

/// Which split points (indices into `split_points`) become line breaks,
/// given the statement's own already-resolved rendering geometry. Returns
/// `None` when the statement is already at or under `width`, or when there
/// are no split points at all — both mean "leave untouched," not "wrap with
/// zero breaks" (data-model.md §1).
///
/// All positions are content-relative offsets (0-based chars from the
/// statement's own first non-whitespace character) — `original_indent` is
/// the caller-supplied original leading-whitespace width used to derive
/// them, never re-derived here. Width/geometry is measured against the
/// statement's *original* token text, not a hypothetical post-casing/
/// post-operator-spacing simulation — a deliberate, documented
/// simplification (spec.md Assumptions): width thresholds are large
/// relative to what those other axes could shift a line by, and neither
/// correctness nor idempotence depends on this precision, only the
/// aesthetic exactness of where a line breaks when several axes are
/// configured together.
pub(crate) fn plan_wrap(
    split_points: &[SplitPoint],
    original_indent: usize,
    total_content_len: usize,
    target_indent: usize,
    continuation_indent: usize,
    width: usize,
    style: LineWrapStyle,
) -> Option<Vec<usize>> {
    if split_points.is_empty() {
        return None;
    }
    if target_indent + total_content_len <= width {
        return None;
    }

    let content_offset = |sp: &SplitPoint| -> usize {
        (sp.span.end.column as usize).saturating_sub(1).saturating_sub(original_indent)
    };

    match style {
        LineWrapStyle::OnePerLine => Some((0..split_points.len()).collect()),
        LineWrapStyle::Fill => {
            let mut chosen: Vec<usize> = Vec::new();
            let mut line_start = 0usize;
            let mut indent = target_indent;
            let mut last_fit: Option<usize> = None;
            let mut i = 0usize;
            while i < split_points.len() {
                let end = content_offset(&split_points[i]);
                let len_if_included = indent + (end - line_start);
                if len_if_included <= width {
                    last_fit = Some(i);
                    i += 1;
                } else if let Some(j) = last_fit {
                    chosen.push(j);
                    line_start = content_offset(&split_points[j]);
                    indent = continuation_indent;
                    last_fit = None;
                    // Retry the same i against the new, deeper-indented line.
                } else {
                    // Doesn't fit even as the first item on a fresh line —
                    // accept the overflow (spec.md Edge Case: never
                    // truncated/altered), move on without breaking here.
                    i += 1;
                }
            }
            // The loop above only ever evaluates "up to a comma" — it never
            // checks the trailing segment after the *last* split point
            // (there's no comma there to trigger a check). Bug caught by
            // manual end-to-end testing during implementation, not by any
            // unit test written before this fix: a statement whose every
            // individual pair fits comma-to-comma, but whose combined
            // content still exceeds the width once the final (comma-less)
            // pair is included, silently produced zero breaks without this
            // check. If the tail doesn't fit and something already fits on
            // the current line, commit that last fit as a final break.
            let tail_len = indent + (total_content_len - line_start);
            if tail_len > width {
                if let Some(j) = last_fit {
                    chosen.push(j);
                }
            }
            Some(chosen)
        }
    }
}

/// Builds the `SpacingEdit`-shaped tuple for one chosen split point:
/// replaces the span from immediately after the comma through `consume_end`
/// (0-based, exclusive — the caller's own responsibility to compute as "skip
/// past any spaces/tabs immediately following the comma on that line," so
/// the original single space that already separated the comma from the next
/// pair is *consumed*, not left in place alongside the new indentation;
/// otherwise the continuation line would end up with one extra leading
/// space beyond the intended indent width — caught by manual end-to-end
/// testing during implementation). Replacement is `terminator` (the
/// specific original line's own already-captured CRLF/LF style, never a
/// hardcoded `\n`) followed by `continuation_indent` (the continuation
/// line's own indentation spaces, computed independently of `indent_plan`
/// by the caller). No further logic here — terminator/indent/consume-end
/// resolution is `format.rs::render`'s own responsibility (data-model.md
/// §1-2); this function only assembles the edit tuple.
pub(crate) fn wrap_edit(split: &SplitPoint, consume_end: usize, terminator: &str, continuation_indent: &str) -> SpacingEdit {
    let start = (split.span.end.column as usize).saturating_sub(1);
    let end = consume_end.max(start);
    let line = split.span.end.line;
    let mut replacement = String::with_capacity(terminator.len() + continuation_indent.len());
    replacement.push_str(terminator);
    replacement.push_str(continuation_indent);
    (line, start, end, replacement)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::statement::build_statements;

    fn control_tokens(source: &str) -> Vec<Token> {
        let tokens = tokenize(source);
        let statements = build_statements(tokens);
        statements
            .into_iter()
            .find(|s| matches!(s.kind, crate::statement::StatementKind::Control { .. }))
            .expect("expected a Control statement")
            .tokens
    }

    #[test]
    fn already_continued_true_for_a_manually_continued_statement() {
        let tokens = control_tokens("RUN PGM=MATRIX,\nZONES=5\nENDRUN\n");
        assert!(already_continued(&tokens));
    }

    #[test]
    fn already_continued_false_for_a_plain_single_line_statement() {
        let tokens = control_tokens("RUN PGM=MATRIX, ZONES=5\nENDRUN\n");
        assert!(!already_continued(&tokens));
    }

    #[test]
    fn top_level_split_points_finds_every_comma_between_pairs() {
        let tokens = control_tokens("RUN PGM=MATRIX, ZONES=5, PRINT=1\nENDRUN\n");
        let points = top_level_split_points(&tokens);
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn top_level_split_points_ignores_comma_in_function_call_parens() {
        let tokens = control_tokens("RUN PGM=MATRIX, MSG=REPLACESTR(A,B,C)\nENDRUN\n");
        let points = top_level_split_points(&tokens);
        // Only the one real top-level comma (before MSG=) is eligible; the
        // two commas inside REPLACESTR(...) are never collected.
        assert_eq!(points.len(), 1);
    }

    #[test]
    fn top_level_split_points_ignores_comma_in_bracketed_subscript() {
        let tokens = control_tokens("RUN PGM=MATRIX, MW=MW[1,2]\nENDRUN\n");
        let points = top_level_split_points(&tokens);
        assert_eq!(points.len(), 1);
    }

    #[test]
    fn top_level_split_points_ignores_comma_inside_a_quoted_value() {
        // Confirmed by direct testing, not assumed: 'a, b' lexes as
        // separate '/a/,/b/' tokens (a quoted value is NOT one atomic
        // token in this grammar -- research.md §4's original claim was
        // wrong, corrected during implementation). quoted_token_mask
        // (reused from operator_spacing.rs) is what actually excludes this
        // comma.
        let tokens = control_tokens("RUN PGM=MATRIX, MSG='a, b'\nENDRUN\n");
        let points = top_level_split_points(&tokens);
        assert_eq!(points.len(), 1);
    }

    #[test]
    fn top_level_split_points_ignores_parens_inside_a_quoted_value() {
        // A stray '(' inside a quoted value must not corrupt paren-depth
        // tracking either -- quoted_token_mask excludes it from depth
        // accounting entirely, the same as it excludes a quoted comma.
        let tokens = control_tokens("RUN PGM=MATRIX, MSG='call(x', ZONES=5\nENDRUN\n");
        let points = top_level_split_points(&tokens);
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn top_level_split_points_empty_for_no_comma() {
        let tokens = control_tokens("RUN PGM=MATRIX\nENDRUN\n");
        assert!(top_level_split_points(&tokens).is_empty());
    }

    #[test]
    fn plan_wrap_none_when_under_width() {
        let sp = vec![SplitPoint { span: Span::new(crate::span::Position::new(1, 16), crate::span::Position::new(1, 17)) }];
        assert!(plan_wrap(&sp, 0, 30, 0, 4, 120, LineWrapStyle::Fill).is_none());
    }

    #[test]
    fn plan_wrap_none_when_no_split_points() {
        assert!(plan_wrap(&[], 0, 200, 0, 4, 120, LineWrapStyle::Fill).is_none());
    }

    #[test]
    fn plan_wrap_one_per_line_selects_every_split_point() {
        let sp = vec![
            SplitPoint { span: Span::new(crate::span::Position::new(1, 16), crate::span::Position::new(1, 17)) },
            SplitPoint { span: Span::new(crate::span::Position::new(1, 26), crate::span::Position::new(1, 27)) },
            SplitPoint { span: Span::new(crate::span::Position::new(1, 36), crate::span::Position::new(1, 37)) },
        ];
        let chosen = plan_wrap(&sp, 0, 150, 0, 4, 120, LineWrapStyle::OnePerLine).unwrap();
        assert_eq!(chosen, vec![0, 1, 2]);
    }

    #[test]
    fn plan_wrap_fill_packs_multiple_short_segments_per_line() {
        // Four split points at content offsets 10, 20, 30, 40, total content
        // length 50, width 25, indent 0 / continuation 4:
        // - Line 1 (indent 0): includes offsets up to 20 (len 20 <=25);
        //   offset 30 would make it 30 (>25) -- break at index 1 (offset 20).
        // - Line 2 (indent 4, starting at offset 20): up to offset 40 fits
        //   (4+20=24 <=25); the loop ends there since there are no more
        //   split points, but the TRAILING segment (offset 40 to the
        //   statement's true end, 50) still needs checking -- 4+(50-20)=34
        //   >25, so a second break is also needed at the last split point
        //   that still fit (index 3, offset 40).
        // - Line 3 (indent 4, starting at offset 40): the remaining 10
        //   chars, 4+10=14 <=25.
        // Three lines total, breaks at indices 1 and 3.
        let mk = |col: u32| SplitPoint { span: Span::new(crate::span::Position::new(1, col), crate::span::Position::new(1, col + 1)) };
        let sp = vec![mk(11), mk(21), mk(31), mk(41)]; // end.column - 1 == 10, 20, 30, 40
        let chosen = plan_wrap(&sp, 0, 50, 0, 4, 25, LineWrapStyle::Fill).unwrap();
        assert_eq!(chosen, vec![1, 3]);
    }

    #[test]
    fn plan_wrap_fill_checks_the_trailing_segment_after_the_last_comma() {
        // Regression test for a real bug caught by manual end-to-end
        // testing: every individual pair fits comma-to-comma (so the main
        // loop never triggers mid-scan overflow), but the combined content
        // -- including the final, comma-less pair -- still exceeds the
        // width. Without the post-loop tail check, this silently produced
        // zero breaks.
        let mk = |col: u32| SplitPoint { span: Span::new(crate::span::Position::new(1, col), crate::span::Position::new(1, col + 1)) };
        let sp = vec![mk(16)]; // "RUN PGM=MATRIX," -- comma end at content offset 15
        let chosen = plan_wrap(&sp, 0, 23, 0, 4, 20, LineWrapStyle::Fill).unwrap();
        assert_eq!(chosen, vec![0]);
    }

    #[test]
    fn wrap_edit_embeds_the_given_terminator_and_indent() {
        let sp = SplitPoint { span: Span::new(crate::span::Position::new(3, 15), crate::span::Position::new(3, 16)) };
        let (line, start, end, replacement) = wrap_edit(&sp, 15, "\r\n", "        ");
        assert_eq!(line, 3);
        assert_eq!(start, 15);
        assert_eq!(end, 15); // zero-width insertion when consume_end == start
        assert_eq!(replacement, "\r\n        ");
    }

    #[test]
    fn wrap_edit_consumes_trailing_whitespace_after_the_comma() {
        // "consume_end" past the comma's own end must be used as the edit's
        // end, so the original space between the comma and the next pair
        // is replaced, not left in place alongside the new indentation.
        let sp = SplitPoint { span: Span::new(crate::span::Position::new(1, 15), crate::span::Position::new(1, 16)) };
        let (_, start, end, replacement) = wrap_edit(&sp, 16, "\n", "    ");
        assert_eq!(start, 15);
        assert_eq!(end, 16); // consumes exactly the one original space
        assert_eq!(replacement, "\n    ");
    }

    #[test]
    fn plan_wrap_fill_never_breaks_when_the_first_segment_alone_overflows() {
        // A single pair longer than the width -- accepted as an overflow,
        // never truncated or forced to break somewhere nonsensical
        // (spec.md Edge Case).
        let mk = |col: u32| SplitPoint { span: Span::new(crate::span::Position::new(1, col), crate::span::Position::new(1, col + 1)) };
        let sp = vec![mk(151)]; // content offset 150, width 100
        let chosen = plan_wrap(&sp, 0, 200, 0, 4, 100, LineWrapStyle::Fill).unwrap();
        assert!(chosen.is_empty());
    }
}
