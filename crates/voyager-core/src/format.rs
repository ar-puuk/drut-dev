//! Whitespace/casing normalization: `format`/`format_bytes` entry points
//! (002-cli-check-format/contracts/formatting-api.md; FR-012–FR-015,
//! FR-013(b), FR-024, FR-025 in that spec). Additive to `parse`/`parse_bytes`
//! — this module renders already-parsed structure, it does not change how
//! anything is recognized.
//!
//! **Scope, precisely** (see spec.md FR-012's seven concrete rules): this
//! renderer only ever touches (a) each line's *leading* whitespace, for
//! lines identified as the first line of a top-level-nested statement/block/
//! closer/branch, and (b) — only when `options.casing` is `Some` — the exact
//! character range of a recognized control-word/keyword-name token. Every
//! other byte of the input — continuation lines, comment-only lines, blank
//! lines, intra-line spacing between tokens, *trailing* whitespace on every
//! line (touched or not), line-ending style — is copied through unchanged.
//! Trailing whitespace is deliberately never stripped, even on a
//! re-indented line: an inline comment's own trailing padding is
//! indistinguishable from "trailing whitespace" without re-deriving comment
//! boundaries here, and FR-012 already requires comment content to be left
//! entirely untouched (confirmed against real corpus data — see
//! `trailing_whitespace_after_inline_comment_text_is_never_touched` below).
//! This is a deliberately narrower scope than "whitespace normalization"
//! might suggest in the abstract — it's exactly what FR-012's corpus-
//! survey-backed rules specify, no more.

use std::collections::{BTreeMap, BTreeSet};

use crate::block::{Block, BlockKind};
use crate::decode;
use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::span::{Position, Span};
use crate::statement::{pair_keyword_boundaries, Statement, StatementKind};
use crate::token::TokenKind;
use crate::{parse, Node};

/// 4 spaces per nesting level, relative to the enclosing block's own
/// opening-statement column — confirmed dominant in a 161-file corpus survey
/// (82.4% of real body-indent occurrences; spec.md FR-012).
const INDENT_WIDTH: usize = 4;

/// The two supported keyword-casing targets (spec.md FR-015 — no hardcoded
/// default; `FormatOptions.casing` being `None` is how "off" is represented,
/// not a third variant here).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CasingConvention {
    Upper,
    Lower,
}

/// Whether `format` leaves existing top-level (depth-0) indentation
/// untouched or unconditionally forces it to column 0 (spec.md FR-001/
/// FR-002 in `009-top-level-indent-toggle`). Two-valued, no "off" state —
/// `format` always does one or the other (research.md §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TopLevelIndentMode {
    /// Leave existing top-level indentation exactly as written — the
    /// `007`-era, and (since `009`) once again default, behavior.
    #[default]
    Preserve,
    /// Force every top-level line to column 0, unconditionally —
    /// `008`'s original behavior, unchanged, now opt-in.
    Normalize,
}

/// Caller-supplied configuration for one `format`/`format_bytes` call.
#[derive(Debug, Clone, Copy, Default)]
pub struct FormatOptions {
    /// `None` (default) leaves all keyword/control-word casing untouched,
    /// exactly as the current input has it (FR-015).
    pub casing: Option<CasingConvention>,
    /// Defaults to `Preserve` (FR-001) via `TopLevelIndentMode`'s own
    /// `#[default]` — every call site is still individually verified
    /// (`009-top-level-indent-toggle`/research.md §2), not trusted
    /// transitively from this derive alone.
    pub top_level_indent: TopLevelIndentMode,
}

/// How `format_bytes`'s decoding of the input relates to what's safe to
/// persist back to disk (spec.md FR-013(b), FR-024, FR-025). Always
/// `Faithful` for `format` (the `&str` entry point), since a `&str` is
/// already valid UTF-8 by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingFidelity {
    /// Decoding needed no fallback at all.
    Faithful,
    /// At least one byte needed (and succeeded under) the Windows-1252
    /// fallback, producing no diagnostic — `text` is a faithful *character*
    /// representation, but persisting it re-encodes that byte as UTF-8
    /// (FR-013(b)'s narrow, named exception).
    Recovered,
    /// At least one byte was undecodable under either encoding and was
    /// replaced with the Unicode replacement character (`InvalidEncoding`
    /// diagnostic present) — MUST NOT be persisted over the original file
    /// (FR-025; a CLI-layer policy, not something this crate refuses to
    /// compute — see contracts/formatting-api.md "Encoding safety").
    Lossy,
}

/// The aggregate value returned by `format`/`format_bytes` for one input
/// file's text — deliberately parallel in shape to `ParseResult`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatResult {
    /// The fully re-rendered source text.
    pub text: String,
    /// `true` iff `text.as_bytes()` differs from the original input's raw
    /// bytes at all — a byte-level comparison against the actual input, so a
    /// file whose only difference is an `EncodingFidelity::Recovered`
    /// re-encoding (no whitespace/casing change) still reports `true`.
    pub changed: bool,
    /// Whatever `parse`/`parse_bytes` would have reported for this input.
    pub diagnostics: Vec<Diagnostic>,
    pub encoding_fidelity: EncodingFidelity,
}

/// Parses `source` internally, then re-renders it per this module's scope
/// (see module docs). Never panics on any input, including a structurally
/// broken one — formatting proceeds best-effort over whatever structure was
/// recovered, the same way `parse` itself keeps going past a diagnosed
/// defect.
pub fn format(source: &str, options: FormatOptions) -> FormatResult {
    let parsed = parse(source);
    let text = render(source, &parsed.nodes, &parsed.diagnostics, options);
    let changed = text.as_bytes() != source.as_bytes();
    FormatResult {
        text,
        changed,
        diagnostics: parsed.diagnostics,
        encoding_fidelity: EncodingFidelity::Faithful,
    }
}

/// Decodes `source` the same way `parse_bytes` does (UTF-8 first, per-byte
/// Windows-1252 fallback) before formatting. See [`EncodingFidelity`] and
/// contracts/formatting-api.md's "Encoding safety" section for what this
/// means for a caller deciding whether to persist `text`.
pub fn format_bytes(source: &[u8], options: FormatOptions) -> FormatResult {
    let (text, decode_diagnostics) = decode::decode_bytes(source);
    let fidelity = if std::str::from_utf8(source).is_ok() {
        EncodingFidelity::Faithful
    } else if decode_diagnostics
        .iter()
        .any(|d| d.kind == DiagnosticKind::InvalidEncoding)
    {
        EncodingFidelity::Lossy
    } else {
        EncodingFidelity::Recovered
    };

    let mut result = format(&text, options);
    if !decode_diagnostics.is_empty() {
        let mut diagnostics = decode_diagnostics;
        diagnostics.extend(result.diagnostics);
        result.diagnostics = diagnostics;
    }
    result.encoding_fidelity = fidelity;
    // Recompute against the *raw bytes*, not the decoded text `format`
    // itself compared against — a pure encoding recovery with no
    // whitespace/casing change must still report `changed: true`.
    result.changed = result.text.as_bytes() != source;
    result
}

// ---------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------

type IndentPlan = BTreeMap<u32, usize>;
/// (line, 0-based char start, 0-based char end (exclusive), replacement text)
type CasingEdit = (u32, usize, usize, String);

fn render(source: &str, nodes: &[Node], diagnostics: &[Diagnostic], options: FormatOptions) -> String {
    let raw_lines = split_lines(source);
    let char_lines: Vec<Vec<char>> = raw_lines.iter().map(|(content, _)| content.chars().collect()).collect();

    let diagnosed_openers = diagnosed_block_openers(diagnostics);
    let mut indent_plan: IndentPlan = BTreeMap::new();
    plan_indentation(nodes, &char_lines, &diagnosed_openers, options.top_level_indent, &mut indent_plan);

    let mut casing_edits: Vec<CasingEdit> = Vec::new();
    if let Some(convention) = options.casing {
        collect_casing_edits(nodes, &char_lines, convention, &mut casing_edits);
    }
    let mut edits_by_line: BTreeMap<u32, Vec<(usize, usize, String)>> = BTreeMap::new();
    for (line, start, end, text) in casing_edits {
        edits_by_line.entry(line).or_default().push((start, end, text));
    }

    let mut out = String::with_capacity(source.len());
    for (idx, (content, terminator)) in raw_lines.iter().enumerate() {
        let line_num = (idx + 1) as u32;
        let mut chars: Vec<char> = content.chars().collect();

        if let Some(edits) = edits_by_line.get(&line_num) {
            for (start, end, replacement) in edits {
                let repl_chars: Vec<char> = replacement.chars().collect();
                if *end <= chars.len() && *start <= *end && repl_chars.len() == end - start {
                    chars[*start..*end].clone_from_slice(&repl_chars);
                }
            }
        }

        if let Some(&target) = indent_plan.get(&line_num) {
            // Leading whitespace only — never trailing. A line's trailing
            // content can be (or sit right after) an inline comment, and
            // FR-012 leaves comment content entirely untouched; there's no
            // way to tell "trailing whitespace" from "whitespace that's
            // part of a comment's own trailing padding" without re-deriving
            // comment boundaries here, so this never touches the tail of
            // the line at all — only where it starts.
            let current_leading = chars.iter().take_while(|c| **c == ' ' || **c == '\t').count();
            let rest = &chars[current_leading..];
            let mut new_line: Vec<char> = vec![' '; target];
            new_line.extend_from_slice(rest);
            chars = new_line;
        }

        out.extend(chars);
        out.push_str(terminator);
    }

    out
}

/// Splits `source` into `(content, terminator)` pairs, `terminator` being
/// `""` (last line, no trailing newline), `"\n"`, or `"\r\n"` — preserving
/// whichever line-ending style each individual line originally used. This
/// formatter does not normalize line endings, the same conservative
/// treatment it gives continuation lines and comments (FR-012).
fn split_lines(source: &str) -> Vec<(&str, &str)> {
    let mut result = Vec::new();
    let mut rest = source;
    while !rest.is_empty() {
        if let Some(pos) = rest.find('\n') {
            let (line, after) = rest.split_at(pos);
            let after = &after[1..];
            if let Some(stripped) = line.strip_suffix('\r') {
                result.push((stripped, "\r\n"));
            } else {
                result.push((line, "\n"));
            }
            rest = after;
        } else {
            result.push((rest, ""));
            rest = "";
        }
    }
    result
}

// ---------------------------------------------------------------------
// Indentation planning (FR-012)
// ---------------------------------------------------------------------

fn original_indent_width(lines: &[Vec<char>], line_num: u32) -> usize {
    lines
        .get((line_num - 1) as usize)
        .map(|l| l.iter().take_while(|c| **c == ' ' || **c == '\t').count())
        .unwrap_or(0)
}

/// A line's *effective* indent for anchoring purposes: its planned target if
/// one exists, otherwise its original (untouched) indent — this is what
/// makes "top-level baseline left untouched" and "4 spaces relative to the
/// enclosing opener" compose correctly regardless of nesting depth.
fn computed_indent(plan: &IndentPlan, lines: &[Vec<char>], line_num: u32) -> usize {
    plan.get(&line_num)
        .copied()
        .unwrap_or_else(|| original_indent_width(lines, line_num))
}

/// The opener positions of every block-level diagnostic
/// (`UnmatchedIf`/`UnmatchedLoop`/`UnmatchedRun`/`UnmatchedProcess`) in
/// `diagnostics` — used by `plan_block` (see its own doc comment) to skip
/// indentation-planning for a genuinely unmatched block's *children only*.
/// **Narrowed 2026-08-11 (008-top-level-indentation-normalization)**: this
/// never protected the block's own *opener* line — `plan_indentation`'s
/// unconditional top-level rule now owns that independently, and does so
/// unconditionally even for a diagnosed block's opener (verified: a
/// diagnosed block's own line is corrected to column 0 while its children
/// stay untouched, `crates/voyager-core/src/format.rs`'s own
/// `diagnosed_block_opener_is_normalized_but_children_stay_untouched`
/// test). This set exists solely to protect a diagnosed block's children,
/// whose structural relationship to that block remains genuinely
/// uncertain regardless of what column the opener itself sits at. A
/// dangling closer (e.g. a stray `ENDIF` with no open `IF`) also produces
/// one of these four kinds, but has no corresponding `Block` node at all —
/// its span never matches any real block's opener and is harmlessly
/// ignored here.
fn diagnosed_block_openers(diagnostics: &[Diagnostic]) -> BTreeSet<Position> {
    diagnostics
        .iter()
        .filter(|d| {
            matches!(
                d.kind,
                DiagnosticKind::UnmatchedIf
                    | DiagnosticKind::UnmatchedLoop
                    | DiagnosticKind::UnmatchedRun
                    | DiagnosticKind::UnmatchedProcess
            )
        })
        .map(|d| d.span.start)
        .collect()
}

/// Top-level nodes' own first lines are normalized to column 0 — every
/// top-level statement or block opener, on every format pass, regardless
/// of its current indentation or formatting history — **only when `mode`
/// is `Normalize`**. **Reversed 2026-08-11
/// (008-top-level-indentation-normalization)**: previously left untouched
/// (the original 161-file corpus survey found no dominant top-level
/// convention — only 20.4% at column 0, modal value column 8 — see
/// `002-cli-check-format/spec.md`'s FR-012 for the historical record of
/// that finding); the project deliberately traded preserving that
/// real-author diversity for predictability. **Reverted again 2026-08-12
/// (`009-top-level-indent-toggle`)**: that trade was the wrong *default*
/// — `Preserve` (never inserting a plan entry for a top-level line, so
/// `computed_indent` falls back to the line's real on-disk column) is
/// the default again; `008`'s unconditional behavior survives unchanged
/// as `Normalize`, opt-in only (research.md §1 in `009`'s own spec
/// confirms `plan_block`/`plan_children`/`computed_indent` need no
/// change at all to support both modes).
fn plan_indentation(
    nodes: &[Node],
    lines: &[Vec<char>],
    diagnosed_openers: &BTreeSet<Position>,
    mode: TopLevelIndentMode,
    plan: &mut IndentPlan,
) {
    for node in nodes {
        if mode == TopLevelIndentMode::Normalize {
            plan.insert(node.span().start.line, 0);
        }
        if let Node::Block(block) = node {
            plan_block(block, lines, diagnosed_openers, plan);
        }
    }
}

fn plan_block(block: &Block, lines: &[Vec<char>], diagnosed_openers: &BTreeSet<Position>, plan: &mut IndentPlan) {
    let opener_line = block.span.start.line;
    let base = computed_indent(plan, lines, opener_line);

    // Explicit closer aligns to its opener (delta 0) — never touched for an
    // implicit close or a genuinely unmatched block (`closer: None`), since
    // there's no real closer line to move; the block's *last child* still
    // gets the ordinary body-indent treatment via plan_children below.
    if let Some(closer_span) = block.closer {
        let closer_line = closer_span.start.line;
        if closer_line != opener_line {
            plan.insert(closer_line, base);
        }
    }

    // A genuinely unmatched block (`closer: None` *and* flagged by its own
    // diagnostic — distinct from the legitimate implicit-close pattern,
    // which is also `closer: None` but produces no diagnostic and is still
    // fully planned below) has an unreliable structural home for its
    // children. Confidently reindenting them now, based on a nesting
    // relationship the diagnostic itself says may not be what the author
    // intended, risks getting it wrong in a way the author never asked
    // for. **Narrowed 2026-08-11 (008-top-level-indentation-normalization)**:
    // this is no longer about preventing opener-line residue — the
    // block's own opener is now unconditionally corrected to column 0
    // regardless of diagnosis (`plan_indentation`'s own doc comment), a
    // stronger and more direct fix for that specific problem than this
    // skip ever was (007-formatter-diagnosed-block-indent-fix/research.md
    // §1 originally framed it that way; 008's own research.md §1 proves
    // the opener-residue case no longer needs this skip at all). What
    // remains genuinely necessary: not speculatively reindenting the
    // *children*, whose relationship to this block stays uncertain no
    // matter what column the opener itself lands on. A later format pass,
    // once the file is well-formed, indents this content correctly in one
    // shot instead.
    if diagnosed_openers.contains(&block.span.start) {
        return;
    }

    match &block.kind {
        BlockKind::If { branches } => {
            for (idx, branch) in branches.iter().enumerate() {
                let branch_line = branch.span.start.line;
                // idx == 0 is the IF itself, whose line is the block's own
                // opener line — already resolved into `base` above (or left
                // untouched at top level); only ELSEIF/ELSE get a fresh
                // target here, aligned to the IF (delta 0).
                if idx > 0 && branch_line != opener_line {
                    plan.insert(branch_line, base);
                }
                plan_children(&branch.children, branch_line, base, lines, diagnosed_openers, plan);
            }
        }
        _ => {
            plan_children(&block.children, opener_line, base, lines, diagnosed_openers, plan);
        }
    }
}

fn plan_children(
    children: &[Node],
    opener_line: u32,
    base: usize,
    lines: &[Vec<char>],
    diagnosed_openers: &BTreeSet<Position>,
    plan: &mut IndentPlan,
) {
    for child in children {
        let child_line = child.span().start.line;
        // A short-IF's trailing statement shares the IF's own line — never
        // touched (spec.md FR-012's body-indent rule only applies when the
        // child starts on its own line).
        if child_line != opener_line {
            plan.insert(child_line, base + INDENT_WIDTH);
        }
        if let Node::Block(b) = child {
            plan_block(b, lines, diagnosed_openers, plan);
        }
    }
}

// ---------------------------------------------------------------------
// Casing rewrite (FR-015) — only reachable when options.casing is Some
// ---------------------------------------------------------------------

/// Scans forward from `from` for the first maximal run of ASCII-alphabetic
/// characters — the lexical extent of a control-word token. Used for block
/// openers/closers/branches, which `voyager-core`'s `Block`/`IfBranch` types
/// retain only as `Span`s (the original `Token` is discarded once matched
/// into structure — see `Block::closer`'s doc comment), not as `Token`s with
/// already-known extents. Safe because every position this is called on is
/// *known*, by construction, to be exactly where a `FIXED_KEYWORDS` entry
/// starts (block-matching already validated that) — and every such keyword
/// is pure letters, no digits, so scanning for the alphabetic run recovers
/// its exact extent regardless of which case it was originally written in
/// (`RUN`/`run`/`!RUN` alike) or which of two synonym spellings was used
/// (`PROCESS`/`PHASE`, `ENDPROCESS`/`ENDPHASE`).
fn first_word_span(lines: &[Vec<char>], from: Position) -> Option<Span> {
    let line_chars = lines.get((from.line - 1) as usize)?;
    let mut start = (from.column - 1) as usize;
    while start < line_chars.len() && !line_chars[start].is_ascii_alphabetic() {
        start += 1;
    }
    if start >= line_chars.len() {
        return None;
    }
    let mut end = start;
    while end < line_chars.len() && line_chars[end].is_ascii_alphabetic() {
        end += 1;
    }
    Some(Span::new(
        Position::new(from.line, (start + 1) as u32),
        Position::new(from.line, (end + 1) as u32),
    ))
}

/// Builds a casing edit for `span`, or `None` if the target casing is
/// already what's there (no-op) or the span doesn't resolve to real content.
fn edit_for_span(lines: &[Vec<char>], span: Span, convention: CasingConvention) -> Option<CasingEdit> {
    let line_chars = lines.get((span.start.line - 1) as usize)?;
    let start = (span.start.column - 1) as usize;
    let end = (span.end.column - 1) as usize;
    if start > end || end > line_chars.len() {
        return None;
    }
    let original: String = line_chars[start..end].iter().collect();
    let replacement = match convention {
        CasingConvention::Upper => original.to_ascii_uppercase(),
        CasingConvention::Lower => original.to_ascii_lowercase(),
    };
    if replacement == original {
        return None;
    }
    Some((span.start.line, start, end, replacement))
}

fn push_if_present(edits: &mut Vec<CasingEdit>, lines: &[Vec<char>], span: Span, convention: CasingConvention) {
    if let Some(edit) = edit_for_span(lines, span, convention) {
        edits.push(edit);
    }
}

fn collect_casing_edits(nodes: &[Node], lines: &[Vec<char>], convention: CasingConvention, edits: &mut Vec<CasingEdit>) {
    for node in nodes {
        match node {
            Node::Statement(stmt) => collect_statement_casing_edits(stmt, lines, convention, edits),
            Node::Block(block) => collect_block_casing_edits(block, lines, convention, edits),
        }
    }
}

fn collect_block_casing_edits(block: &Block, lines: &[Vec<char>], convention: CasingConvention, edits: &mut Vec<CasingEdit>) {
    // The opener statement's own keyword=value pair names (RUN PGM=...,
    // etc.) — already exact token spans, no scanning needed.
    for span in &block.opener_pairs {
        push_if_present(edits, lines, *span, convention);
    }
    // The explicit closer's own word, if one exists.
    if let Some(closer_span) = block.closer {
        if let Some(word_span) = first_word_span(lines, closer_span.start) {
            push_if_present(edits, lines, word_span, convention);
        }
    }

    match &block.kind {
        BlockKind::If { branches } => {
            for branch in branches {
                // Covers IF (idx 0) and ELSEIF/ELSE (idx > 0) uniformly —
                // all are just "the word starting at this branch's span".
                if let Some(word_span) = first_word_span(lines, branch.span.start) {
                    push_if_present(edits, lines, word_span, convention);
                }
                collect_casing_edits(&branch.children, lines, convention, edits);
            }
        }
        _ => {
            if let Some(word_span) = first_word_span(lines, block.span.start) {
                push_if_present(edits, lines, word_span, convention);
            }
            collect_casing_edits(&block.children, lines, convention, edits);
        }
    }
}

fn collect_statement_casing_edits(stmt: &Statement, lines: &[Vec<char>], convention: CasingConvention, edits: &mut Vec<CasingEdit>) {
    if !matches!(stmt.kind, StatementKind::Control { .. }) {
        // Casing never targets Assignment/Label/ShellEscape content — none
        // of those are "control-word/keyword-name" tokens (FR-015).
        return;
    }
    // The control word: the first Word-kind token — handles `!RUN`
    // uniformly (tokens[0] is `!` Punctuation, tokens[1] is `RUN` Word) with
    // the ordinary case (tokens[0] itself is the Word) needing no special
    // branch.
    if let Some(word_tok) = stmt.tokens.iter().find(|t| t.kind == TokenKind::Word) {
        push_if_present(edits, lines, word_tok.span, convention);
    }
    // Pair keyword names — never their values, never subscript contents.
    for (kw_start, _eq_idx) in pair_keyword_boundaries(&stmt.tokens) {
        if let Some(tok) = stmt.tokens.get(kw_start) {
            push_if_present(edits, lines, tok.span, convention);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upper() -> FormatOptions {
        FormatOptions {
            casing: Some(CasingConvention::Upper),
            top_level_indent: TopLevelIndentMode::default(),
        }
    }

    fn normalize() -> FormatOptions {
        FormatOptions {
            casing: None,
            top_level_indent: TopLevelIndentMode::Normalize,
        }
    }

    // -- Indentation -----------------------------------------------------

    #[test]
    fn nested_if_loop_gets_four_space_increments() {
        let src = "IF (X=1)\nLOOP i=1,5\nY = 2\nENDLOOP\nENDIF\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(
            out,
            "IF (X=1)\n    LOOP i=1,5\n        Y = 2\n    ENDLOOP\nENDIF\n"
        );
    }

    #[test]
    fn already_canonical_indentation_is_idempotent() {
        let src = "IF (X=1)\n    LOOP i=1,5\n        Y = 2\n    ENDLOOP\nENDIF\n";
        let first = format(src, FormatOptions::default());
        assert_eq!(first.text, src);
        assert!(!first.changed);
        let second = format(&first.text, FormatOptions::default());
        assert_eq!(second.text, first.text);
        assert!(!second.changed);
    }

    #[test]
    fn double_formatting_a_messy_file_is_idempotent() {
        let src = "IF (X=1)\n LOOP i=1,5\n              Y = 2\n ENDLOOP\nENDIF\n";
        let once = format(src, FormatOptions::default()).text;
        let twice = format(&once, FormatOptions::default()).text;
        assert_eq!(once, twice);
    }

    #[test]
    fn format_options_default_top_level_indent_is_preserve() {
        // 009-top-level-indent-toggle FR-004(b): the single most direct
        // confirmation of the derived Default -- distinct from (and
        // cheaper than) the behavioral tests around it.
        assert_eq!(FormatOptions::default().top_level_indent, TopLevelIndentMode::Preserve);
    }

    #[test]
    fn top_level_baseline_is_always_normalized_to_zero() {
        // 008-top-level-indentation-normalization's own behavior, retargeted
        // 2026-08-12 (009-top-level-indent-toggle) to explicit Normalize
        // mode now that Preserve is the default -- this test exists to
        // keep proving 008's guarantee still holds, opt-in.
        let src = "        RUN PGM=MATRIX\n        X = 1\n        ENDRUN\n";
        let out = format(src, normalize()).text;
        assert_eq!(out, "RUN PGM=MATRIX\n    X = 1\nENDRUN\n");
    }

    #[test]
    fn top_level_baseline_is_left_untouched_by_default() {
        // 009-top-level-indent-toggle FR-001: the default reverts to
        // 007-era preserve -- revives this test's original pre-008
        // assertion (top_level_baseline_is_left_untouched, git history
        // 4f1d5fe~1): RUN keeps its original 8-space baseline; its body
        // still gets exactly +4 relative to *that* baseline (the
        // per-nesting-level rule is unaffected by this feature and stays
        // active regardless of mode); ENDRUN aligns to the same baseline
        // as its own opener.
        let src = "        RUN PGM=MATRIX\n        X = 1\n        ENDRUN\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(out, "        RUN PGM=MATRIX\n            X = 1\n        ENDRUN\n");
    }

    #[test]
    fn bare_top_level_statement_is_normalized_to_zero() {
        // Previously had zero code path touching it at all -- plan_indentation
        // only ever iterated Node::Block entries (research.md §1 in 008's
        // own spec). Retargeted 2026-08-12 (009-top-level-indent-toggle) to
        // explicit Normalize mode now that Preserve is the default.
        let src = "    X = 1\n";
        let out = format(src, normalize()).text;
        assert_eq!(out, "X = 1\n");
    }

    #[test]
    fn bare_top_level_statement_is_left_untouched_by_default() {
        // 009-top-level-indent-toggle FR-001.
        let src = "    X = 1\n";
        let result = format(src, FormatOptions::default());
        assert!(!result.changed);
        assert_eq!(result.text, src);
    }

    #[test]
    fn top_level_block_with_stale_children_corrects_both_together() {
        // spec.md Acceptance Scenario 2: a block opener already corrected
        // to column 0, but its children still carrying indentation
        // relative to the block's *old*, non-zero position -- both the
        // opener and its children must resolve correctly in one pass.
        let src = "RUN PGM=HWYASSIGN\n        FILEI NETI = 'net.net'\n    ENDRUN\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(out, "RUN PGM=HWYASSIGN\n    FILEI NETI = 'net.net'\nENDRUN\n");
    }

    #[test]
    fn already_column_zero_top_level_is_idempotent() {
        // spec.md Acceptance Scenario 3.
        let src = "RUN PGM=MATRIX\n    X = 1\nENDRUN\n";
        let result = format(src, FormatOptions::default());
        assert!(!result.changed);
        assert_eq!(result.text, src);
    }

    #[test]
    fn diagnosed_block_opener_is_normalized_but_children_stay_untouched() {
        // The explicit 007/008 interaction point (008's own tasks.md T006).
        // Retargeted 2026-08-12 (009-top-level-indent-toggle) to explicit
        // Normalize mode now that Preserve is the default: a genuinely
        // unmatched PROCESS whose own opener sits at non-zero indentation,
        // with both its legitimate body content (FILEI) and a swallowed
        // trailing RUN block also at non-zero indentation.
        let src = "    PROCESS PHASE=INPUT\n        FILEI = ni.1\n\n    RUN PGM=HWYASSIGN\n        FILEI NETI = 'net.net'\n    ENDRUN\n";
        let result = format(src, normalize());

        assert!(result.changed);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].kind, DiagnosticKind::UnmatchedProcess);

        let expected = "PROCESS PHASE=INPUT\n        FILEI = ni.1\n\n    RUN PGM=HWYASSIGN\n        FILEI NETI = 'net.net'\n    ENDRUN\n";
        assert_eq!(
            result.text, expected,
            "PROCESS's own opener must be corrected to column 0, but every child \
             (both the legitimate FILEI body content and the swallowed RUN block) \
             must stay byte-for-byte untouched"
        );
    }

    #[test]
    fn diagnosed_block_opener_and_children_both_stay_untouched_by_default() {
        // 009-top-level-indent-toggle FR-001: under the Preserve default,
        // nothing forces the opener's own line either (unlike the
        // Normalize-mode sibling above, where 008's unconditional rule
        // corrects it independently of 007's children-only skip) -- the
        // whole diagnosed subtree, opener included, is byte-for-byte
        // untouched, same as pre-008.
        let src = "    PROCESS PHASE=INPUT\n        FILEI = ni.1\n\n    RUN PGM=HWYASSIGN\n        FILEI NETI = 'net.net'\n    ENDRUN\n";
        let result = format(src, FormatOptions::default());

        assert!(!result.changed);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].kind, DiagnosticKind::UnmatchedProcess);
        assert_eq!(
            result.text, src,
            "under Preserve, the diagnosed block's opener must stay untouched too, not just its children"
        );
    }

    #[test]
    fn continuation_lines_are_left_untouched() {
        let src = "IF (X=1)\nY = 1 +\n        2\nENDIF\n";
        let out = format(src, FormatOptions::default()).text;
        // Y=... gets re-indented (it's a normal body statement), but its
        // continuation line ("2") is never touched, however it was written.
        assert_eq!(out, "IF (X=1)\n    Y = 1 +\n        2\nENDIF\n");
    }

    #[test]
    fn comment_only_lines_are_left_untouched() {
        let src = "IF (X=1)\n; a comment, deliberately unindented\nY = 2\nENDIF\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(
            out,
            "IF (X=1)\n; a comment, deliberately unindented\n    Y = 2\nENDIF\n"
        );
    }

    #[test]
    fn inline_trailing_comment_spacing_is_left_untouched() {
        let src = "IF (X=1)\nY = 2      ; five spaces before this\nENDIF\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(
            out,
            "IF (X=1)\n    Y = 2      ; five spaces before this\nENDIF\n"
        );
    }

    #[test]
    fn implicit_run_close_does_not_corrupt_indentation() {
        let src = "RUN PGM=MATRIX\nX = 1\nRUN PGM=HIGHWAY\nY = 2\nENDRUN\n";
        let out = format(src, FormatOptions::default()).text;
        // First RUN closes implicitly (no closer to align); its own body
        // still indents correctly, and does NOT get double-processed.
        assert_eq!(
            out,
            "RUN PGM=MATRIX\n    X = 1\nRUN PGM=HIGHWAY\n    Y = 2\nENDRUN\n"
        );
    }

    #[test]
    fn elseif_else_align_to_if_regardless_of_original_indent() {
        let src = "IF (X=1)\nA = 1\n  ELSEIF (X=2)\nB = 2\n        ELSE\nC = 3\nENDIF\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(
            out,
            "IF (X=1)\n    A = 1\nELSEIF (X=2)\n    B = 2\nELSE\n    C = 3\nENDIF\n"
        );
    }

    #[test]
    fn trailing_whitespace_is_never_touched_even_on_reindented_lines() {
        // FR-012 has no trailing-whitespace rule, and stripping it
        // unconditionally would corrupt a comment's own trailing padding
        // (discovered against real corpus data: `;hbc      ` style trailing
        // spaces after comment text) — so this is deliberately a no-op on
        // the tail of any line, only ever touching where it starts.
        let src = "IF (X=1)\nY = 2   \nENDIF\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(out, "IF (X=1)\n    Y = 2   \nENDIF\n");
    }

    #[test]
    fn trailing_whitespace_after_inline_comment_text_is_never_touched() {
        let src = "IF (X=1)\nY = 2    ;note      \nENDIF\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(out, "IF (X=1)\n    Y = 2    ;note      \nENDIF\n");
    }

    // -- Casing ------------------------------------------------------------

    #[test]
    fn casing_off_by_default_leaves_everything_alone() {
        let src = "if (x=1)\nrun pgm=matrix\nendrun\nendif\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(out, "if (x=1)\n    run pgm=matrix\n    endrun\nendif\n");
    }

    #[test]
    fn casing_upper_rewrites_control_words_and_closers() {
        let src = "if (x=1)\nendif\n";
        let out = format(src, upper()).text;
        assert_eq!(out, "IF (x=1)\nENDIF\n");
    }

    #[test]
    fn casing_upper_rewrites_run_pgm_pair_keyword() {
        let src = "run pgm=matrix zones=5\nendrun\n";
        let out = format(src, upper()).text;
        assert_eq!(out, "RUN PGM=matrix ZONES=5\nENDRUN\n");
    }

    #[test]
    fn casing_upper_rewrites_bang_run() {
        let src = "!run pgm=matrix\nendrun\n";
        let out = format(src, upper()).text;
        assert_eq!(out, "!RUN PGM=matrix\nENDRUN\n");
    }

    #[test]
    fn casing_upper_rewrites_elseif_else() {
        let src = "if (x=1)\na = 1\nelseif (x=2)\nb = 2\nelse\nc = 3\nendif\n";
        let out = format(src, upper()).text;
        assert_eq!(
            out,
            "IF (x=1)\n    a = 1\nELSEIF (x=2)\n    b = 2\nELSE\n    c = 3\nENDIF\n"
        );
    }

    #[test]
    fn casing_upper_never_touches_values_labels_or_variable_refs() {
        let src = ":if\nx = if\ny = @if@\n";
        let out = format(src, upper()).text;
        // The label ":if", the assignment value "if", and the @variable@
        // reference "@if@" all happen to spell a control word — none are
        // ever casing targets, since none are structurally a control word
        // or keyword name.
        assert_eq!(out, src);
    }

    #[test]
    fn casing_lower_rewrites_process_phase_shortcut() {
        let src = "PHASE=ILOOP\nENDPHASE\n";
        let out = format(
            src,
            FormatOptions {
                casing: Some(CasingConvention::Lower),
                top_level_indent: TopLevelIndentMode::default(),
            },
        )
        .text;
        assert_eq!(out, "phase=ILOOP\nendphase\n");
    }

    #[test]
    fn casing_rewrite_is_idempotent() {
        let src = "run pgm=matrix\nendrun\n";
        let once = format(src, upper()).text;
        let twice = format(&once, upper()).text;
        assert_eq!(once, twice);
    }

    // -- format_bytes / EncodingFidelity ------------------------------------

    #[test]
    fn format_bytes_pure_ascii_is_faithful() {
        let result = format_bytes(b"IF (X=1)\nENDIF\n", FormatOptions::default());
        assert_eq!(result.encoding_fidelity, EncodingFidelity::Faithful);
    }

    #[test]
    fn format_bytes_recovered_byte_is_written_through_and_flagged() {
        // 0x92 is Windows-1252's right single quotation mark.
        let mut src = b"X = 'author".to_vec();
        src.push(0x92);
        src.extend_from_slice(b"s note'\n");
        let result = format_bytes(&src, FormatOptions::default());
        assert_eq!(result.encoding_fidelity, EncodingFidelity::Recovered);
        assert!(result.text.contains('\u{2019}'));
        assert!(result.changed, "byte-level re-encoding must count as changed even with no whitespace/casing diff");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn format_bytes_undecodable_byte_is_lossy_and_diagnosed() {
        let src = vec![b'X', b'=', 0x81, b'\n'];
        let result = format_bytes(&src, FormatOptions::default());
        assert_eq!(result.encoding_fidelity, EncodingFidelity::Lossy);
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::InvalidEncoding));
    }

    #[test]
    fn format_str_entry_point_is_always_faithful() {
        let result = format("IF (X=1)\nENDIF\n", FormatOptions::default());
        assert_eq!(result.encoding_fidelity, EncodingFidelity::Faithful);
    }

    // -- changed / general -----------------------------------------------

    #[test]
    fn changed_is_false_when_nothing_needed_reformatting() {
        let src = "X = 1\n";
        let result = format(src, FormatOptions::default());
        assert!(!result.changed);
        assert_eq!(result.text, src);
    }

    #[test]
    fn empty_input_produces_no_panic_and_empty_output() {
        let result = format("", FormatOptions::default());
        assert_eq!(result.text, "");
        assert!(!result.changed);
    }

    #[test]
    fn structurally_broken_input_still_produces_best_effort_output() {
        // Unmatched IF — format must not panic or refuse; it still
        // re-renders whatever structure was recovered. Updated
        // 2026-08-11 (007-formatter-diagnosed-block-indent-fix): a
        // genuinely unmatched block's children are no longer confidently
        // reindented (`Y = 2` used to become `    Y = 2`) — that's exactly
        // what let stale, formatter-written indentation survive as
        // untouchable residue once a later edit resolved the block
        // boundary and revealed the content's true structure. "Best
        // effort" now means "leave it exactly as written" for a diagnosed
        // block's own subtree, not "guess a nesting depth that might be
        // wrong."
        let src = "IF (X=1)\nY = 2\n";
        let result = format(src, FormatOptions::default());
        assert!(!result.diagnostics.is_empty());
        assert_eq!(result.text, src);
        assert!(!result.changed);
    }

    #[test]
    fn crlf_line_endings_are_preserved() {
        let src = "IF (X=1)\r\nY = 2\r\nENDIF\r\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(out, "IF (X=1)\r\n    Y = 2\r\nENDIF\r\n");
    }

    #[test]
    fn behavior_preservation_reparses_to_the_same_structure() {
        let src = "  IF (X=1)\nRUN PGM=MATRIX\nX = 1\nENDRUN\nENDIF\n";
        let formatted = format(src, FormatOptions::default()).text;
        let before = parse(src);
        let after = parse(&formatted);
        assert_eq!(before.nodes.len(), after.nodes.len());
        assert_eq!(before.diagnostics.len(), after.diagnostics.len());
    }
}
