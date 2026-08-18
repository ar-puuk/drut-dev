//! Whitespace/casing normalization: `format`/`format_bytes` entry points
//! (002-cli-check-format/contracts/formatting-api.md; FR-012–FR-015,
//! FR-013(b), FR-024, FR-025 in that spec). Additive to `parse`/`parse_bytes`
//! — this module renders already-parsed structure, it does not change how
//! anything is recognized.
//!
//! **Scope, precisely, under `operator_spacing: Preserve`** (see spec.md
//! FR-012's seven concrete rules — still exactly true for `Preserve`, the
//! default and every existing configuration's own behavior): this renderer
//! only ever touches (a) each line's *leading* whitespace, for lines
//! identified as the first line of a top-level-nested statement/block/
//! closer/branch, and (b) — only when `options.casing` is `Some` — the exact
//! character range of a recognized control-word/keyword-name token. Every
//! other byte of the input — continuation lines, comment-only lines, blank
//! lines, intra-line spacing between tokens, *trailing* whitespace on every
//! line (touched or not), line-ending style — is copied through unchanged.
//! **`Fixed`/`Auto` (018-operator-spacing) widen this deliberately**: when
//! `options.operator_spacing` is `Fixed` or `Auto`, intra-line spacing
//! around recognized operators/commas/bracket-paren interiors is also
//! normalized — see `operator_spacing.rs` for exactly what that covers, and
//! FR-012 (in that feature's spec) for what still doesn't (comment content,
//! string/quoted-literal content, everything else this paragraph already
//! excludes). Trailing whitespace is deliberately never stripped, even on a
//! re-indented line: an inline comment's own trailing padding is
//! indistinguishable from "trailing whitespace" without re-deriving comment
//! boundaries here, and FR-012 already requires comment content to be left
//! entirely untouched (confirmed against real corpus data — see
//! `trailing_whitespace_after_inline_comment_text_is_never_touched` below).
//! This is a deliberately narrower scope than "whitespace normalization"
//! might suggest in the abstract — it's exactly what FR-012's corpus-
//! survey-backed rules (plus 018's own FRs, when opted into) specify, no
//! more.

use std::collections::{BTreeMap, BTreeSet};

use crate::block::{Block, BlockKind};
use crate::data_reference;
use crate::decode;
use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::lexer::tokenize;
use crate::operator_spacing;
use crate::span::{Position, Span};
use crate::statement::{build_statements, pair_keyword_boundaries, Statement, StatementKind};
use crate::token::{Token, TokenKind};
use crate::{parse, Node};

/// The three supported keyword-casing targets (spec.md FR-015, amended by
/// `014-casing-preserve-mode` FR-001). `Preserve` is the `#[default]` —
/// `format` always either preserves, uppercases, or lowercases
/// keyword/control-word casing, the same non-optional shape
/// `TopLevelIndentMode` already uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CasingConvention {
    /// Leave existing control-word/pair-keyword casing exactly as written
    /// — the previous `FormatOptions.casing == None` behavior, now a real
    /// named variant instead of an absent value.
    #[default]
    Preserve,
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

/// Three independently-configurable casing categories (spec.md FR-001,
/// `017-casing-categories-indent-width`) — replaces the single flat
/// `CasingConvention` `FormatOptions.casing` used to be. Each field defaults
/// to `Preserve` via `CasingConvention`'s own `#[default]`, so this struct
/// needs no manual `Default` impl of its own. No built-in opinionated
/// preset/"auto" value exists anywhere in this shape (FR-003) — every field
/// is only ever `Preserve`, `Upper`, or `Lower`, the project's own explicit
/// choice or nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CasingSettings {
    /// Block-structural reserved syntax — `IF`, `LOOP`, `RUN`, etc.
    pub control_words: CasingConvention,
    /// `keyword=value` statement parameter names — `FILE=`, `LIST=`, etc.
    pub pair_keywords: CasingConvention,
    /// Matrix/Line/Node/Zone/Database abbreviations, the output-record and
    /// link-endpoint tokens, and the two reserved implicit loop-index
    /// identifiers (`I`/`J`) — data_reference.rs's recognized-name table.
    /// One value applies uniformly regardless of which structural shape a
    /// given occurrence takes (FR-005).
    pub data_references: CasingConvention,
}

/// Whether/how `format` normalizes whitespace around operators, commas, and
/// bracket/paren interiors (spec.md FR-001, `018-operator-spacing`).
/// `Preserve` is the `#[default]`, same non-optional three-value shape
/// `CasingConvention`/`TopLevelIndentMode` already use. `Auto` is
/// implemented as a strict superset of `Fixed` (data-model.md §1) — never a
/// second, independent spacing decision that could drift from `Fixed`'s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OperatorSpacing {
    /// Leave existing operator/comma/bracket-paren spacing exactly as
    /// written — the default, and the only behavior when this feature is
    /// unconfigured (FR-009).
    #[default]
    Preserve,
    /// Normalize every in-scope operator to exactly one space on each side
    /// (leading-only for a trailing continuation-position operator),
    /// comma spacing between `Control` pairs, and zero interior padding
    /// inside brackets/parens (FR-002–FR-005, FR-012).
    Fixed,
    /// Everything `Fixed` does, plus vertical alignment of consecutive
    /// `Assignment` statements' `=` within an alignment run (FR-006–FR-008).
    Auto,
}

/// Whether/how `format` contracts excessive runs of consecutive blank lines
/// (spec.md FR-001, `019-blank-line-normalization`). Two-valued, matching
/// `TopLevelIndentMode`'s own shape exactly — there is only one real
/// non-`Preserve` behavior here (contract a run down to the applicable cap),
/// so no third tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlankLineMode {
    /// Leave every blank-line run exactly as written, however long — the
    /// default, and the only behavior when this feature is unconfigured
    /// (FR-009).
    #[default]
    Preserve,
    /// Contract a run of consecutive blank lines down to the applicable cap
    /// (`top_level_blank_line_cap` between top-level statements/blocks,
    /// `nested_blank_line_cap` anywhere inside any block's own body) only
    /// when the run's length exceeds that cap (FR-003/FR-004).
    Auto,
}

/// Caller-supplied configuration for one `format`/`format_bytes` call.
///
/// **No longer `#[derive(Default)]`** (`017-casing-categories-indent-width`):
/// `indent_width`'s correct default is `4`, not `u8::default()`'s `0` — the
/// first field on this struct whose default isn't just its own type's
/// `Default::default()` (research.md §4). See the `impl Default` below.
#[derive(Debug, Clone, Copy)]
pub struct FormatOptions {
    /// Defaults to `CasingSettings::default()` (all three fields
    /// `Preserve`) — `017-casing-categories-indent-width` widened this from
    /// a single `CasingConvention` to three independently-configurable
    /// categories; every already-shipped caller that only ever set/read the
    /// old single value now reads/writes `control_words`/`pair_keywords`
    /// (the two categories the old value already reached).
    pub casing: CasingSettings,
    /// Defaults to `Preserve` (FR-001) via `TopLevelIndentMode`'s own
    /// `#[default]` — every call site is still individually verified
    /// (`009-top-level-indent-toggle`/research.md §2), not trusted
    /// transitively from this derive alone.
    pub top_level_indent: TopLevelIndentMode,
    /// Spaces per nesting level of block indentation, relative to the
    /// enclosing block's own opening-statement column (spec.md FR-009,
    /// `017-casing-categories-indent-width`). Defaults to `4` (see `impl
    /// Default` below) — confirmed dominant in a 161-file corpus survey
    /// (82.4% of real body-indent occurrences; `002-cli-check-format`
    /// FR-012), the fixed value every format call used before this field
    /// existed. Accepts any `u8` here; the 1–16 valid-range bound is a
    /// `drut-config`-layer policy decision, not a fact this crate enforces
    /// (research.md §4).
    pub indent_width: u8,
    /// Whether/how operator/comma/bracket-paren spacing is normalized
    /// (spec.md FR-001, `018-operator-spacing`). Defaults to `Preserve`
    /// (see `impl Default` below) — a project with nothing configured sees
    /// zero behavior change from before this field existed (FR-009).
    pub operator_spacing: OperatorSpacing,
    /// Whether/how excessive blank-line runs are contracted (spec.md FR-001,
    /// `019-blank-line-normalization`). Defaults to `Preserve` (see `impl
    /// Default` below) — a project with nothing configured sees zero
    /// behavior change from before this field existed (FR-009).
    pub blank_lines: BlankLineMode,
    /// The maximum number of consecutive blank lines `auto` allows between
    /// top-level statements/blocks before contracting the run (spec.md
    /// FR-002). Defaults to `2` (see `impl Default` below) — accepts any
    /// `u8` here; the valid-range bound is a `drut-config`-layer policy
    /// decision, not a fact this crate enforces, the same `indent_width`
    /// precedent.
    pub top_level_blank_line_cap: u8,
    /// The maximum number of consecutive blank lines `auto` allows inside
    /// any block's own body, uniformly regardless of nesting depth, before
    /// contracting the run (spec.md FR-002/FR-008). Defaults to `1` (see
    /// `impl Default` below).
    pub nested_blank_line_cap: u8,
}

impl Default for FormatOptions {
    fn default() -> Self {
        FormatOptions {
            casing: CasingSettings::default(),
            top_level_indent: TopLevelIndentMode::default(),
            indent_width: 4,
            operator_spacing: OperatorSpacing::default(),
            blank_lines: BlankLineMode::default(),
            top_level_blank_line_cap: 2,
            nested_blank_line_cap: 1,
        }
    }
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
    /// The start position of every `; FMT: OFF` marker left unmatched at
    /// end-of-file (010-fmt-region-markers FR-010) — deliberately **not**
    /// a `Diagnostic`/`DiagnosticKind` (spec.md Assumptions); empty in the
    /// overwhelming common case (no markers, or every marker matched).
    pub unclosed_fmt_off_markers: Vec<Position>,
}

/// Parses `source` internally, then re-renders it per this module's scope
/// (see module docs). Never panics on any input, including a structurally
/// broken one — formatting proceeds best-effort over whatever structure was
/// recovered, the same way `parse` itself keeps going past a diagnosed
/// defect.
pub fn format(source: &str, options: FormatOptions) -> FormatResult {
    let parsed = parse(source);
    let (text, unclosed_fmt_off_markers) = render(source, &parsed.nodes, &parsed.diagnostics, options);
    let changed = text.as_bytes() != source.as_bytes();
    FormatResult {
        text,
        changed,
        diagnostics: parsed.diagnostics,
        encoding_fidelity: EncodingFidelity::Faithful,
        unclosed_fmt_off_markers,
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

/// The start position of every `; FMT: OFF` marker left unmatched at
/// end-of-file in `source` (010-fmt-region-markers FR-010) — the same
/// detection `format`/`format_bytes` run internally, exposed standalone
/// for callers that want this signal without paying for a full
/// indentation/casing pass (e.g. `drut-lsp`'s independent diagnostics
/// publish cycle, which runs on every document change, not only on a
/// formatting request).
pub fn unclosed_fmt_off_markers(source: &str) -> Vec<Position> {
    let tokens = tokenize(source);
    let total_lines = split_lines(source).len() as u32;
    protected_regions(&tokens, total_lines).1
}

// ---------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------

type IndentPlan = BTreeMap<u32, usize>;
/// (line, 0-based char start, 0-based char end (exclusive), replacement text)
type CasingEdit = (u32, usize, usize, String);
/// Same shape as `CasingEdit`, but — unlike `CasingEdit` — `replacement.len()`
/// is NOT required to equal `end - start` (data-model.md §2, 018-operator-
/// spacing): this is the edit type that actually needs insertion/removal,
/// applied via `apply_line_edits`'s left-to-right rebuild rather than
/// `CasingEdit`'s same-length in-place splice. `pub(crate)` so
/// `operator_spacing.rs` (a sibling module) can construct these directly.
pub(crate) type SpacingEdit = (u32, usize, usize, String);

fn render(source: &str, nodes: &[Node], diagnostics: &[Diagnostic], options: FormatOptions) -> (String, Vec<Position>) {
    let raw_lines = split_lines(source);
    let char_lines: Vec<Vec<char>> = raw_lines.iter().map(|(content, _)| content.chars().collect()).collect();
    let tokens = tokenize(source);
    let (protected, unclosed) = protected_regions(&tokens, raw_lines.len() as u32);

    let diagnosed_openers = diagnosed_block_openers(diagnostics);
    let mut indent_plan: IndentPlan = BTreeMap::new();
    plan_indentation(
        nodes,
        &char_lines,
        &diagnosed_openers,
        &protected,
        options.top_level_indent,
        options.indent_width as usize,
        &mut indent_plan,
    );

    let mut casing_edits: Vec<CasingEdit> = Vec::new();
    // Performance short-circuit only, not a correctness requirement —
    // edit_for_span's Preserve arm already produces a guaranteed no-op
    // (replacement == original) for any category left at Preserve.
    if options.casing.control_words != CasingConvention::Preserve
        || options.casing.pair_keywords != CasingConvention::Preserve
        || options.casing.data_references != CasingConvention::Preserve
    {
        collect_casing_edits(nodes, &char_lines, &protected, options.casing, &mut casing_edits);
        for occurrence in data_reference::data_reference_occurrences(nodes, &char_lines) {
            push_if_present(
                &mut casing_edits,
                &char_lines,
                &protected,
                occurrence.span,
                options.casing.data_references,
            );
        }
    }

    // Operator-spacing edits (018-operator-spacing) — short-circuited on
    // `Preserve` (the default), same performance-only gate shape the casing
    // pass above already uses; `Preserve` does exactly the same work as
    // before this feature existed (FR-009/SC-003). Uses the *flat* statement
    // list, not `nodes`, since a block opener's own tokens (e.g. `RUN
    // PGM=MATRIX ZONES=5`'s pairs, or an `IF(x==1)`'s condition) are only
    // fully retained there — `Block`'s own `opener_pairs: Vec<Span>` keeps
    // keyword spans only (research.md §1 in that feature's own research).
    let mut spacing_edits: Vec<SpacingEdit> = Vec::new();
    if options.operator_spacing != OperatorSpacing::Preserve {
        let statements = build_statements(tokens.clone());
        let mut fixed_edits: Vec<SpacingEdit> = Vec::new();
        for stmt in &statements {
            operator_spacing::collect_fixed_edits(stmt, &char_lines, &mut fixed_edits);
        }
        // Two independently-recognized operators can legitimately queue an
        // edit for the exact same zero-width gap (e.g. two adjacent binary
        // `*`/`/` characters with nothing between them: the first's
        // "after" edit and the second's "before" edit both target that
        // same gap) — both always compute the identical replacement for
        // the same original span, so deduplicating by (line, start, end)
        // is always safe and never loses a real edit. Without this, both
        // copies would apply during the per-line rebuild, silently
        // inserting the padding twice. Done once, here, before this list
        // is used for anything else (including Auto's rendered-column
        // delta sum below, which would otherwise double-count too).
        let fixed_edits: Vec<SpacingEdit> = {
            let by_key: BTreeMap<(u32, usize, usize), String> =
                fixed_edits.into_iter().map(|(l, s, e, r)| ((l, s, e), r)).collect();
            by_key.into_iter().map(|((l, s, e), r)| (l, s, e, r)).collect()
        };
        if options.operator_spacing == OperatorSpacing::Auto {
            // Alignment is always computed on top of Fixed's own edits,
            // never a divergent spacing decision (contracts/operator-
            // spacing.md) — merged by exact (line, start, end) key so an
            // alignment-padded gap replaces (never duplicates) whatever
            // Fixed alone would have queued for that same gap.
            let mut alignment_edits: Vec<SpacingEdit> = Vec::new();
            operator_spacing::collect_alignment_edits(nodes, &protected, &fixed_edits, &mut alignment_edits);
            let mut by_key: BTreeMap<(u32, usize, usize), String> =
                fixed_edits.iter().map(|(l, s, e, r)| ((*l, *s, *e), r.clone())).collect();
            for (l, s, e, r) in alignment_edits {
                by_key.insert((l, s, e), r);
            }
            spacing_edits = by_key.into_iter().map(|((l, s, e), r)| (l, s, e, r)).collect();
        } else {
            spacing_edits = fixed_edits;
        }
        // Same protected-line funnel every other edit kind already goes
        // through (`push_if_present`'s pattern) — a protected line never
        // receives an operator-spacing edit either.
        spacing_edits.retain(|(line, _, _, _)| !protected.contains(line));
    }

    // Blank-line-run normalization (019-blank-line-normalization) —
    // short-circuited on `Preserve` (the default), same performance-only
    // gate shape every other axis already uses; `Preserve` does exactly the
    // same work as before this feature existed (FR-009/SC-003).
    let mut lines_to_delete: BTreeSet<u32> = BTreeSet::new();
    if options.blank_lines != BlankLineMode::Preserve {
        lines_to_delete = crate::blank_line::lines_to_delete(
            nodes,
            &char_lines,
            &protected,
            options.top_level_blank_line_cap,
            options.nested_blank_line_cap,
        );
    }

    let mut edits_by_line: BTreeMap<u32, Vec<(usize, usize, String)>> = BTreeMap::new();
    for (line, start, end, text) in casing_edits {
        edits_by_line.entry(line).or_default().push((start, end, text));
    }
    let mut spacing_by_line: BTreeMap<u32, Vec<(usize, usize, String)>> = BTreeMap::new();
    for (line, start, end, text) in spacing_edits {
        spacing_by_line.entry(line).or_default().push((start, end, text));
    }

    let mut out = String::with_capacity(source.len());
    for (idx, (content, terminator)) in raw_lines.iter().enumerate() {
        let line_num = (idx + 1) as u32;
        if lines_to_delete.contains(&line_num) {
            // This line contributes nothing to `out` at all — a true
            // deletion, not a blanked-but-still-emitted line (research.md
            // §1). Every other per-line computation above (indentation
            // lookup, casing/spacing edits) was already only ever keyed by
            // line number, so skipping emission here needs no other change.
            continue;
        }
        let orig_chars: Vec<char> = content.chars().collect();

        let mut chars: Vec<char> = if let Some(spacing) = spacing_by_line.get(&line_num) {
            // Variable-length rebuild path (data-model.md §2): merge this
            // line's casing edits (if any) and spacing edits into one
            // sorted, left-to-right rebuild — both kinds operate on
            // disjoint spans (token text vs. the whitespace around it), so
            // a single merged pass is safe. Only used for lines that
            // actually have a spacing edit; every other line keeps using
            // the cheaper same-length splice below, unchanged.
            let mut combined: Vec<(usize, usize, String)> = edits_by_line.get(&line_num).cloned().unwrap_or_default();
            combined.extend(spacing.iter().cloned());
            combined.sort_by_key(|(start, _, _)| *start);
            let mut rebuilt: Vec<char> = Vec::with_capacity(orig_chars.len());
            let mut cursor = 0usize;
            for (start, end, replacement) in &combined {
                if *start < cursor || *end < *start || *end > orig_chars.len() {
                    // Malformed/overlapping edit — never panic, just skip
                    // this one and keep going with the rest of the line.
                    continue;
                }
                rebuilt.extend_from_slice(&orig_chars[cursor..*start]);
                rebuilt.extend(replacement.chars());
                cursor = *end;
            }
            rebuilt.extend_from_slice(&orig_chars[cursor..]);
            rebuilt
        } else {
            let mut chars = orig_chars.clone();
            if let Some(edits) = edits_by_line.get(&line_num) {
                for (start, end, replacement) in edits {
                    let repl_chars: Vec<char> = replacement.chars().collect();
                    if *end <= chars.len() && *start <= *end && repl_chars.len() == end - start {
                        chars[*start..*end].clone_from_slice(&repl_chars);
                    }
                }
            }
            chars
        };

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

    (out, unclosed)
}

// ---------------------------------------------------------------------
// FMT region markers (010-fmt-region-markers)
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FmtMarker {
    Off,
    On,
}

/// Whether `text` (a `TokenKind::LineComment` token's raw text, including
/// its leading `;`) is a `; FMT: OFF` / `; FMT: ON` marker — case-insensitive
/// on `FMT`/`OFF`/`ON`, flexible whitespace around the leading `;` and the
/// colon (spec.md FR-001/FR-002, research.md §4). Whether the comment is
/// the *entire* content of its line is a separate check, made by
/// `protected_regions` using the full token stream, not here.
fn fmt_marker_kind(text: &str) -> Option<FmtMarker> {
    let body = text.trim_start_matches(';').trim();
    let (left, right) = body.split_once(':')?;
    if !left.trim().eq_ignore_ascii_case("fmt") {
        return None;
    }
    match right.trim().to_ascii_uppercase().as_str() {
        "OFF" => Some(FmtMarker::Off),
        "ON" => Some(FmtMarker::On),
        _ => None,
    }
}

/// Scans `tokens` for `; FMT: OFF`/`; FMT: ON` markers and returns every
/// protected line number (inclusive of both marker lines, or through
/// `total_lines` if a region is left open at end-of-file) plus the start
/// position of every unmatched `; FMT: OFF` (research.md §2, contracts/
/// fmt-region-markers.md). A `LineComment` only counts as a marker when it
/// is the *entire* content of its physical line — a trailing
/// `; FMT: OFF` after real statement content is not recognized (FR-001/
/// FR-002). A second `; FMT: OFF` while a region is already open, or a
/// stray `; FMT: ON` while none is open, is a no-op (FR-005) — this is a
/// simple on/off state transition, not balanced-pair counting.
fn protected_regions(tokens: &[Token], total_lines: u32) -> (BTreeSet<u32>, Vec<Position>) {
    let mut lines_with_other_tokens: BTreeSet<u32> = BTreeSet::new();
    for tok in tokens {
        if tok.kind != TokenKind::LineComment {
            lines_with_other_tokens.insert(tok.span.start.line);
        }
    }

    let mut protected: BTreeSet<u32> = BTreeSet::new();
    let mut unclosed: Vec<Position> = Vec::new();
    let mut open: Option<Position> = None;

    for tok in tokens {
        if tok.kind != TokenKind::LineComment || lines_with_other_tokens.contains(&tok.span.start.line) {
            continue;
        }
        let Some(marker) = fmt_marker_kind(&tok.text) else {
            continue;
        };
        match marker {
            FmtMarker::Off => {
                if open.is_none() {
                    open = Some(tok.span.start);
                }
            }
            FmtMarker::On => {
                if let Some(start) = open.take() {
                    for line in start.line..=tok.span.start.line {
                        protected.insert(line);
                    }
                }
            }
        }
    }

    if let Some(start) = open {
        let end_line = total_lines.max(start.line);
        for line in start.line..=end_line {
            protected.insert(line);
        }
        unclosed.push(start);
    }

    (protected, unclosed)
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
    protected: &BTreeSet<u32>,
    mode: TopLevelIndentMode,
    indent_width: usize,
    plan: &mut IndentPlan,
) {
    for node in nodes {
        if mode == TopLevelIndentMode::Normalize {
            let line = node.span().start.line;
            if !protected.contains(&line) {
                plan.insert(line, 0);
            }
        }
        if let Node::Block(block) = node {
            plan_block(block, lines, diagnosed_openers, protected, indent_width, plan);
        }
    }
}

fn plan_block(
    block: &Block,
    lines: &[Vec<char>],
    diagnosed_openers: &BTreeSet<Position>,
    protected: &BTreeSet<u32>,
    indent_width: usize,
    plan: &mut IndentPlan,
) {
    let opener_line = block.span.start.line;
    let base = computed_indent(plan, lines, opener_line);

    // Explicit closer aligns to its opener (delta 0) — never touched for an
    // implicit close or a genuinely unmatched block (`closer: None`), since
    // there's no real closer line to move; the block's *last child* still
    // gets the ordinary body-indent treatment via plan_children below.
    if let Some(closer_span) = block.closer {
        let closer_line = closer_span.start.line;
        if closer_line != opener_line && !protected.contains(&closer_line) {
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
                if idx > 0 && branch_line != opener_line && !protected.contains(&branch_line) {
                    plan.insert(branch_line, base);
                }
                plan_children(&branch.children, branch_line, base, lines, diagnosed_openers, protected, indent_width, plan);
            }
        }
        _ => {
            plan_children(&block.children, opener_line, base, lines, diagnosed_openers, protected, indent_width, plan);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn plan_children(
    children: &[Node],
    opener_line: u32,
    base: usize,
    lines: &[Vec<char>],
    diagnosed_openers: &BTreeSet<Position>,
    protected: &BTreeSet<u32>,
    indent_width: usize,
    plan: &mut IndentPlan,
) {
    for child in children {
        let child_line = child.span().start.line;
        // A short-IF's trailing statement shares the IF's own line — never
        // touched (spec.md FR-012's body-indent rule only applies when the
        // child starts on its own line).
        if child_line != opener_line && !protected.contains(&child_line) {
            plan.insert(child_line, base + indent_width);
        }
        if let Node::Block(b) = child {
            plan_block(b, lines, diagnosed_openers, protected, indent_width, plan);
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
        // Exhaustiveness only -- render()'s guard above means this
        // function's call chain is never actually reached with `Preserve`
        // (014-casing-preserve-mode research.md §1).
        CasingConvention::Preserve => original.clone(),
    };
    if replacement == original {
        return None;
    }
    Some((span.start.line, start, end, replacement))
}

fn push_if_present(
    edits: &mut Vec<CasingEdit>,
    lines: &[Vec<char>],
    protected: &BTreeSet<u32>,
    span: Span,
    convention: CasingConvention,
) {
    // Single funnel point for every casing edit this module ever produces
    // (010-fmt-region-markers) — a protected line never receives a casing
    // edit, regardless of which caller reached here.
    if protected.contains(&span.start.line) {
        return;
    }
    if let Some(edit) = edit_for_span(lines, span, convention) {
        edits.push(edit);
    }
}

fn collect_casing_edits(
    nodes: &[Node],
    lines: &[Vec<char>],
    protected: &BTreeSet<u32>,
    settings: CasingSettings,
    edits: &mut Vec<CasingEdit>,
) {
    for node in nodes {
        match node {
            Node::Statement(stmt) => collect_statement_casing_edits(stmt, lines, protected, settings, edits),
            Node::Block(block) => collect_block_casing_edits(block, lines, protected, settings, edits),
        }
    }
}

fn collect_block_casing_edits(
    block: &Block,
    lines: &[Vec<char>],
    protected: &BTreeSet<u32>,
    settings: CasingSettings,
    edits: &mut Vec<CasingEdit>,
) {
    // The opener statement's own keyword=value pair names (RUN PGM=...,
    // etc.) — already exact token spans, no scanning needed. Pair-keyword
    // names, not control words, so `settings.pair_keywords` applies here —
    // except a name data_reference.rs also recognizes (e.g. `ZONES` in
    // `RUN PGM=MATRIX ZONES=5`), which data_references casing owns
    // exclusively (FR-005); skipped here so it's never queued for two
    // different conventions at once.
    for span in &block.opener_pairs {
        if let Some(text) = data_reference::text_at_span(lines, *span) {
            if data_reference::is_data_reference_name(&text) {
                continue;
            }
        }
        push_if_present(edits, lines, protected, *span, settings.pair_keywords);
    }
    // The explicit closer's own word, if one exists — a control word.
    if let Some(closer_span) = block.closer {
        if let Some(word_span) = first_word_span(lines, closer_span.start) {
            push_if_present(edits, lines, protected, word_span, settings.control_words);
        }
    }

    match &block.kind {
        BlockKind::If { branches } => {
            for branch in branches {
                // Covers IF (idx 0) and ELSEIF/ELSE (idx > 0) uniformly —
                // all are just "the word starting at this branch's span",
                // and all are control words.
                if let Some(word_span) = first_word_span(lines, branch.span.start) {
                    push_if_present(edits, lines, protected, word_span, settings.control_words);
                }
                collect_casing_edits(&branch.children, lines, protected, settings, edits);
            }
        }
        _ => {
            if let Some(word_span) = first_word_span(lines, block.span.start) {
                push_if_present(edits, lines, protected, word_span, settings.control_words);
            }
            collect_casing_edits(&block.children, lines, protected, settings, edits);
        }
    }
}

fn collect_statement_casing_edits(
    stmt: &Statement,
    lines: &[Vec<char>],
    protected: &BTreeSet<u32>,
    settings: CasingSettings,
    edits: &mut Vec<CasingEdit>,
) {
    if !matches!(stmt.kind, StatementKind::Control { .. }) {
        // Casing never targets Assignment/Label/ShellEscape content — none
        // of those are "control-word/keyword-name" tokens (FR-015).
        // (data_references casing on an Assignment target is handled
        // separately, in render(), via data_reference_occurrences.)
        return;
    }
    // The control word: the first Word-kind token — handles `!RUN`
    // uniformly (tokens[0] is `!` Punctuation, tokens[1] is `RUN` Word) with
    // the ordinary case (tokens[0] itself is the Word) needing no special
    // branch.
    if let Some(word_tok) = stmt.tokens.iter().find(|t| t.kind == TokenKind::Word) {
        push_if_present(edits, lines, protected, word_tok.span, settings.control_words);
    }
    // Pair keyword names — never their values, never subscript contents.
    // A name data_reference.rs also recognizes (e.g. `DBI` in `FILEI
    // DBI=...`) is skipped here — data_references casing owns it
    // exclusively (FR-005), never queued for two conventions at once.
    for (kw_start, _eq_idx) in pair_keyword_boundaries(&stmt.tokens) {
        if let Some(tok) = stmt.tokens.get(kw_start) {
            if data_reference::is_data_reference_name(&tok.text) {
                continue;
            }
            push_if_present(edits, lines, protected, tok.span, settings.pair_keywords);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upper() -> FormatOptions {
        FormatOptions {
            // Exactly reproduces pre-017 `CasingConvention::Upper`'s reach:
            // control words + pair keywords, never data-references — so
            // every existing assertion below that used upper() keeps
            // passing unmodified (017-casing-categories-indent-width
            // tasks.md T007).
            casing: CasingSettings {
                control_words: CasingConvention::Upper,
                pair_keywords: CasingConvention::Upper,
                data_references: CasingConvention::Preserve,
            },
            top_level_indent: TopLevelIndentMode::default(),
            indent_width: 4,
            operator_spacing: OperatorSpacing::default(),
            blank_lines: BlankLineMode::default(),
            top_level_blank_line_cap: 2,
            nested_blank_line_cap: 1,
        }
    }

    fn normalize() -> FormatOptions {
        FormatOptions {
            casing: CasingSettings::default(),
            top_level_indent: TopLevelIndentMode::Normalize,
            indent_width: 4,
            operator_spacing: OperatorSpacing::default(),
            blank_lines: BlankLineMode::default(),
            top_level_blank_line_cap: 2,
            nested_blank_line_cap: 1,
        }
    }

    fn blank_lines_auto() -> FormatOptions {
        FormatOptions {
            blank_lines: BlankLineMode::Auto,
            ..FormatOptions::default()
        }
    }

    // -- Blank-line-run normalization (019-blank-line-normalization) -----

    #[test]
    fn deleted_line_is_genuinely_absent_not_just_blanked() {
        // T009: the render-pipeline's one genuinely new capability
        // (research.md §1) -- confirms the output's *total line count*
        // actually decreases, catching a half-implementation that clears a
        // line's content but still emits an empty output line instead of
        // truly removing it.
        let src = "X = 1\n\n\n\n\n\nY = 2\n";
        let result = format(src, blank_lines_auto());
        let out = result.text;
        let input_line_count = src.lines().count();
        let output_line_count = out.lines().count();
        assert_eq!(
            input_line_count - output_line_count,
            3,
            "3 excess blank lines (run of 5, cap 2) must genuinely disappear from the output's line count, not just go blank"
        );
        assert_eq!(out, "X = 1\n\n\nY = 2\n");
    }

    #[test]
    fn surviving_line_edits_are_unaffected_by_deletion_elsewhere_in_the_same_file() {
        // T009: indentation/casing/spacing edits for a *surviving* line are
        // still applied correctly even when other lines in the same file
        // are deleted -- deletion is a final filter over otherwise-
        // unmodified per-line output, not a renumbering (research.md §1).
        let src = "if (x=1)\n\n\n\n\n\nrun pgm=matrix\ny = 2\nendrun\nendif\n";
        let mut options = blank_lines_auto();
        options.casing = CasingSettings {
            control_words: CasingConvention::Upper,
            ..CasingSettings::default()
        };
        let out = format(src, options).text;
        assert_eq!(
            out,
            "IF (x=1)\n\n    RUN pgm=matrix\n        y = 2\n    ENDRUN\nENDIF\n",
            "the surviving lines' own indentation and control-word casing must still be correctly applied: {out}"
        );
    }

    #[test]
    fn blank_lines_auto_is_idempotent_for_an_over_cap_top_level_run() {
        // 019-blank-line-normalization tasks.md T019 (applying 018's own
        // post-/speckit-analyze lesson up front): a second format pass over
        // an already-contracted run must be a stable no-op.
        let src = "X = 1\n\n\n\n\n\nY = 2\n";
        let once = format(src, blank_lines_auto()).text;
        let twice = format(&once, blank_lines_auto()).text;
        assert_eq!(once, twice);
    }

    #[test]
    fn blank_lines_auto_respects_fmt_off_on_for_a_top_level_run() {
        // 019-blank-line-normalization tasks.md T019: a protected region's
        // over-cap top-level run is left exactly as written, while an
        // unprotected over-cap top-level run elsewhere in the same file
        // contracts normally.
        let src = "X = 1\n; FMT: OFF\n\n\n\n\n\nY = 2\n; FMT: ON\nZ = 3\n\n\n\n\n\nW = 4\n";
        let out = format(src, blank_lines_auto()).text;
        assert_eq!(
            out,
            "X = 1\n; FMT: OFF\n\n\n\n\n\nY = 2\n; FMT: ON\nZ = 3\n\n\nW = 4\n",
            "the protected run must survive exactly as written while the unprotected run contracts to the default cap (2): {out}"
        );
    }

    #[test]
    fn blank_lines_auto_is_idempotent_for_a_multi_level_nested_over_cap_run() {
        // 019-blank-line-normalization tasks.md T021: a doubly-nested
        // over-cap run must also settle to a stable fixed point on a
        // second format pass, not just the singly-nested case T019 already
        // covers.
        let src = "RUN PGM=MATRIX\n    LOOP i=1,5\n        X = 1\n\n\n\n\n        Y = 2\n    ENDLOOP\nENDRUN\n";
        let once = format(src, blank_lines_auto()).text;
        let twice = format(&once, blank_lines_auto()).text;
        assert_eq!(once, twice);
    }

    #[test]
    fn blank_lines_auto_respects_fmt_off_on_for_a_nested_run_specifically() {
        // 019-blank-line-normalization tasks.md T021: a protected region
        // containing an over-cap *nested* run is left exactly as written,
        // while an unprotected nested run elsewhere in the same file
        // contracts to the nested cap normally.
        let src = "RUN PGM=MATRIX\n    LOOP i=1,5\n; FMT: OFF\n        X = 1\n\n\n\n\n        Y = 2\n; FMT: ON\n    ENDLOOP\nENDRUN\n\nRUN PGM=NETWORK\n    A = 1\n\n\n\n\n    B = 2\nENDRUN\n";
        let out = format(src, blank_lines_auto()).text;
        assert_eq!(
            out,
            "RUN PGM=MATRIX\n    LOOP i=1,5\n; FMT: OFF\n        X = 1\n\n\n\n\n        Y = 2\n; FMT: ON\n    ENDLOOP\nENDRUN\n\nRUN PGM=NETWORK\n    A = 1\n\n    B = 2\nENDRUN\n",
            "the protected nested run must survive exactly as written while the unprotected nested run contracts to the default nested cap (1): {out}"
        );
    }

    #[test]
    fn blank_lines_preserve_default_is_byte_identical_across_several_over_cap_runs() {
        // T009 (FR-009/SC-003 regression case): `Preserve` (the default)
        // leaves every blank-line run exactly as written, however long --
        // a project with no `blank_lines` configuration produces
        // byte-identical output to before this feature existed.
        let src = "X = 1\n\n\n\n\n\nY = 2\n\n\n\nIF (A=1)\n\n\n\n\nENDIF\n";
        let result = format(src, FormatOptions::default());
        assert!(!result.changed, "Preserve (the default) must be a true no-op on excessive blank-line runs");
        assert_eq!(result.text, src);
    }

    // -- Indentation -----------------------------------------------------

    #[test]
    fn indent_width_default_is_four() {
        // 017-casing-categories-indent-width tasks.md T028.
        assert_eq!(FormatOptions::default().indent_width, 4);
    }

    #[test]
    fn nested_blocks_advance_by_configured_indent_width_not_the_old_fixed_four() {
        // 017-casing-categories-indent-width tasks.md T028, US3 AS1.
        let src = "IF (X=1)\nLOOP i=1,5\nY = 2\nENDLOOP\nENDIF\n";
        let options = FormatOptions { indent_width: 2, ..FormatOptions::default() };
        let out = format(src, options).text;
        assert_eq!(out, "IF (X=1)\n  LOOP i=1,5\n    Y = 2\n  ENDLOOP\nENDIF\n");
    }

    #[test]
    fn indent_width_unconfigured_is_byte_identical_to_pre_017_behavior() {
        // 017-casing-categories-indent-width tasks.md T028, FR-012, US3 AS3
        // -- FormatOptions::default() must reproduce exactly what the old
        // fixed INDENT_WIDTH constant produced, for every existing golden
        // fixture (proven at scale by format_corpus.rs's own golden-file
        // suite; this is the single most direct unit-level confirmation).
        let src = "IF (X=1)\nLOOP i=1,5\nY = 2\nENDLOOP\nENDIF\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(
            out,
            "IF (X=1)\n    LOOP i=1,5\n        Y = 2\n    ENDLOOP\nENDIF\n"
        );
    }

    #[test]
    fn indent_width_is_idempotent() {
        // 017-casing-categories-indent-width tasks.md T040 (post-/speckit-
        // analyze finding I1) -- re-verified directly for a non-default
        // width, not assumed to hold transitively from the default-width
        // idempotence tests elsewhere in this module.
        let src = "IF (X=1)\nLOOP i=1,5\nY = 2\nENDLOOP\nENDIF\n";
        let options = FormatOptions { indent_width: 2, ..FormatOptions::default() };
        let once = format(src, options).text;
        let twice = format(&once, options).text;
        assert_eq!(once, twice);
    }

    #[test]
    fn indent_width_respects_fmt_off_on() {
        // 017-casing-categories-indent-width tasks.md T040 (finding I1): a
        // nested block inside a `; FMT: OFF` region keeps its exact
        // original indentation regardless of indent_width, while an
        // unprotected nested region elsewhere in the same file reflects
        // the configured width.
        let src = "IF (X=1)\n; FMT: OFF\nLOOP i=1,5\n      Y = 2\nENDLOOP\n; FMT: ON\nENDIF\nIF (X=2)\nLOOP j=1,5\nZ = 3\nENDLOOP\nENDIF\n";
        let options = FormatOptions { indent_width: 2, ..FormatOptions::default() };
        let out = format(src, options).text;
        assert_eq!(
            out,
            "IF (X=1)\n; FMT: OFF\nLOOP i=1,5\n      Y = 2\nENDLOOP\n; FMT: ON\nENDIF\nIF (X=2)\n  LOOP j=1,5\n    Z = 3\n  ENDLOOP\nENDIF\n"
        );
    }

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
    fn casing_convention_default_is_preserve() {
        // 014-casing-preserve-mode FR-001/SC-003 (point 1 of 3).
        assert_eq!(CasingConvention::default(), CasingConvention::Preserve);
    }

    #[test]
    fn format_options_default_casing_is_preserve() {
        // 014-casing-preserve-mode FR-002/SC-003 (point 2 of 3) -- the
        // single most direct confirmation of User Story 3, distinct from
        // the behavioral tests around it. 017-casing-categories-indent-
        // width: casing is now three independent fields, all Preserve.
        assert_eq!(FormatOptions::default().casing, CasingSettings::default());
        assert_eq!(FormatOptions::default().casing.control_words, CasingConvention::Preserve);
        assert_eq!(FormatOptions::default().casing.pair_keywords, CasingConvention::Preserve);
        assert_eq!(FormatOptions::default().casing.data_references, CasingConvention::Preserve);
    }

    #[test]
    fn casing_explicit_preserve_matches_the_old_none_based_output_exactly() {
        // 014-casing-preserve-mode FR-003/User Story 2: byte-identical to
        // what FormatOptions.casing == None produced before this feature --
        // same fixture as casing_off_by_default_leaves_everything_alone
        // above, but with Preserve passed explicitly rather than relying on
        // FormatOptions::default().
        let src = "if (x=1)\nrun pgm=matrix\nendrun\nendif\n";
        let options = FormatOptions {
            casing: CasingSettings::default(),
            top_level_indent: TopLevelIndentMode::default(),
            indent_width: 4,
            operator_spacing: OperatorSpacing::default(),
            ..FormatOptions::default()
        };
        let out = format(src, options).text;
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
        // "msg", not "zones" -- 017-casing-categories-indent-width: ZONES is
        // now a data_references-category name (FR-005), so with upper()'s
        // data_references left at Preserve, a ZONES pair here would no
        // longer be touched by pair_keywords casing (see
        // zones_pair_keyword_is_owned_by_data_references_not_pair_keywords
        // below for that exact behavior, deliberately proven).
        let src = "run pgm=matrix msg=hi\nendrun\n";
        let out = format(src, upper()).text;
        assert_eq!(out, "RUN PGM=matrix MSG=hi\nENDRUN\n");
    }

    #[test]
    fn zones_pair_keyword_is_owned_by_data_references_not_pair_keywords() {
        // FR-005: ZONES appears in RUN's own opener pairs, but it's a
        // data_references-category name -- pair_keywords casing must skip
        // it, and data_references casing must be the one that reaches it.
        let src = "run pgm=matrix zones=5\nendrun\n";
        let pair_keywords_only = FormatOptions {
            casing: CasingSettings {
                control_words: CasingConvention::Preserve,
                pair_keywords: CasingConvention::Upper,
                data_references: CasingConvention::Preserve,
            },
            top_level_indent: TopLevelIndentMode::default(),
            indent_width: 4,
            operator_spacing: OperatorSpacing::default(),
            ..FormatOptions::default()
        };
        let out = format(src, pair_keywords_only).text;
        assert_eq!(out, "run PGM=matrix zones=5\nendrun\n", "pair_keywords alone must not touch zones: {out}");

        let data_references_only = FormatOptions {
            casing: CasingSettings {
                control_words: CasingConvention::Preserve,
                pair_keywords: CasingConvention::Preserve,
                data_references: CasingConvention::Upper,
            },
            top_level_indent: TopLevelIndentMode::default(),
            indent_width: 4,
            operator_spacing: OperatorSpacing::default(),
            ..FormatOptions::default()
        };
        let out = format(src, data_references_only).text;
        assert_eq!(out, "run pgm=matrix ZONES=5\nendrun\n", "data_references alone must reach zones: {out}");
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
                casing: CasingSettings {
                    control_words: CasingConvention::Lower,
                    pair_keywords: CasingConvention::Lower,
                    data_references: CasingConvention::Preserve,
                },
                top_level_indent: TopLevelIndentMode::default(),
                indent_width: 4,
                operator_spacing: OperatorSpacing::default(),
                ..FormatOptions::default()
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

    fn data_references_upper() -> FormatOptions {
        FormatOptions {
            casing: CasingSettings {
                control_words: CasingConvention::Preserve,
                pair_keywords: CasingConvention::Preserve,
                data_references: CasingConvention::Upper,
            },
            top_level_indent: TopLevelIndentMode::default(),
            indent_width: 4,
            operator_spacing: OperatorSpacing::default(),
            ..FormatOptions::default()
        }
    }

    #[test]
    fn data_references_casing_is_idempotent() {
        // 017-casing-categories-indent-width tasks.md T038 (post-/speckit-
        // analyze finding I1) -- re-verified directly, not assumed to hold
        // transitively from the control_words/pair_keywords idempotence
        // test above.
        let src = "mw[1] = mi.1.1 + mi.2.1\nx = li.FT\n";
        let once = format(src, data_references_upper()).text;
        let twice = format(&once, data_references_upper()).text;
        assert_eq!(once, twice);
    }

    #[test]
    fn data_references_casing_respects_fmt_off_on() {
        // 017-casing-categories-indent-width tasks.md T038 (finding I1):
        // a data-reference token inside a `; FMT: OFF` region must be left
        // exactly as written, the same guarantee every other casing
        // category already has -- while an unprotected occurrence
        // elsewhere in the same file still gets rewritten.
        let src = "; FMT: OFF\nmw[1] = mi.1.1\n; FMT: ON\nmw[2] = mi.2.1\n";
        let out = format(src, data_references_upper()).text;
        assert_eq!(out, "; FMT: OFF\nmw[1] = mi.1.1\n; FMT: ON\nMW[2] = MI.2.1\n");
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

    // -- FMT region markers (010-fmt-region-markers) ------------------------

    #[test]
    fn protected_range_is_left_untouched_while_everything_else_normalizes() {
        // `y`/`z` are Assignment targets, never a casing target (matches
        // `casing_upper_never_touches_values_labels_or_variable_refs`
        // above) — only `IF`/`ENDIF` are control words here.
        let src = "if (x=1)\ny = 1\n; FMT: OFF\n  weird = 1\n    myvar = 2\n; FMT: ON\nz = 3\nendif\n";
        let out = format(src, upper()).text;
        assert_eq!(
            out,
            "IF (x=1)\n    y = 1\n; FMT: OFF\n  weird = 1\n    myvar = 2\n; FMT: ON\n    z = 3\nENDIF\n"
        );
    }

    #[test]
    fn file_with_no_fmt_markers_formats_exactly_as_before_this_feature() {
        let src = "if (x=1)\ny = 1\nendif\n";
        let out = format(src, upper()).text;
        assert_eq!(out, "IF (x=1)\n    y = 1\nENDIF\n");
    }

    #[test]
    fn multiple_non_overlapping_regions_are_each_independently_protected() {
        let src = "if (x=1)\na = 1\n; FMT: OFF\n  b = 2\n; FMT: ON\nc = 3\n; FMT: OFF\n  d = 4\n; FMT: ON\ne = 5\nendif\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(
            out,
            "if (x=1)\n    a = 1\n; FMT: OFF\n  b = 2\n; FMT: ON\n    c = 3\n; FMT: OFF\n  d = 4\n; FMT: ON\n    e = 5\nendif\n"
        );
    }

    #[test]
    fn duplicate_fmt_off_while_already_open_is_a_no_op() {
        // US1 Acceptance Scenario 4 — a single subsequent FMT: ON must
        // close the whole region; if the duplicate OFF were incorrectly
        // treated as needing its own paired ON, `c = 3` below would stay
        // protected too instead of being normalized.
        let src = "if (x=1)\n; FMT: OFF\n  a = 1\n; FMT: OFF\n  b = 2\n; FMT: ON\nc = 3\nendif\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(
            out,
            "if (x=1)\n; FMT: OFF\n  a = 1\n; FMT: OFF\n  b = 2\n; FMT: ON\n    c = 3\nendif\n"
        );
    }

    #[test]
    fn stray_fmt_on_with_no_open_region_is_a_no_op() {
        // US1 Acceptance Scenario 5.
        let src = "if (x=1)\n; FMT: ON\na = 1\nendif\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(out, "if (x=1)\n; FMT: ON\n    a = 1\nendif\n");
    }

    #[test]
    fn protected_region_straddling_a_block_boundary_is_allowed() {
        let src = "if (x=1)\n; FMT: OFF\n  a = 1\nendif\n; FMT: ON\nb = 2\n";
        let result = format(src, FormatOptions::default());
        assert_eq!(
            result.text, src,
            "region opens inside the IF block and closes after it — every line from \
             FMT: OFF through FMT: ON stays untouched, including the block's own ENDIF"
        );
        assert!(
            result.diagnostics.is_empty(),
            "a marker straddling a block boundary must not produce a diagnostic (FR-006)"
        );
    }

    #[test]
    fn whole_file_is_one_protected_region_with_closing_marker() {
        let src = "; FMT: OFF\nif (x=1)\n  a=1\nendif\n; FMT: ON\n";
        let result = format(src, upper());
        assert_eq!(result.text, src);
        assert!(!result.changed);
        assert!(result.unclosed_fmt_off_markers.is_empty());
    }

    #[test]
    fn whole_file_is_one_protected_region_without_closing_marker() {
        let src = "; FMT: OFF\nif (x=1)\n  a=1\nendif\n";
        let result = format(src, upper());
        assert_eq!(result.text, src);
        assert!(!result.changed);
        assert_eq!(result.unclosed_fmt_off_markers, vec![Position::new(1, 1)]);
    }

    #[test]
    fn marker_text_inside_a_real_block_comment_is_not_treated_as_a_marker() {
        // FR-009's own named example — the exact false-positive
        // research.md §1 chose tokenizer-based recognition to avoid: block
        // comments are never tokenized as `LineComment`, so text that only
        // *looks* like a marker inside one is invisible to the scan.
        let src = "if (x=1)\n/* ; FMT: OFF inside a block comment */\na = 1\nendif\n";
        let out = format(src, FormatOptions::default()).text;
        assert_eq!(
            out,
            "if (x=1)\n/* ; FMT: OFF inside a block comment */\n    a = 1\nendif\n",
            "marker-looking text inside a real block comment must not suppress \
             formatting for anything after it"
        );
    }

    #[test]
    fn opener_residue_child_anchors_to_protected_openers_true_on_disk_column_not_a_discarded_planned_value() {
        // research.md §2's load-bearing finding, tasks.md T006: protection
        // must be gated at *collection* time, not filtered at final render
        // time. Under Normalize mode, a top-level opener would normally be
        // forced to column 0 — but this opener is protected, so it must
        // keep its real on-disk column (6), and the out-of-region child
        // must anchor to THAT column, not to a discarded, would-have-been-0
        // planned value.
        let src = "; FMT: OFF\n      if (x=1)\n; FMT: ON\na = 1\nendif\n";
        let out = format(src, normalize()).text;
        assert_eq!(
            out,
            "; FMT: OFF\n      if (x=1)\n; FMT: ON\n          a = 1\n      endif\n",
            "the protected IF opener must keep its true on-disk column (6), not be \
             forced to 0 by Normalize mode; the out-of-region child must be indented \
             4 spaces relative to that TRUE column (10), and the closer must align \
             to that same true column (6) — none of these may anchor to a discarded \
             planned value of 0"
        );
    }

    #[test]
    fn protected_top_level_line_stays_untouched_under_normalize_mode() {
        // 009-top-level-indent-toggle interaction, tasks.md T007: gate
        // point #1 (plan_indentation's top-level insert, only reached
        // under Normalize) must be guarded by `protected` exactly like the
        // other three gate points.
        let src = "; FMT: OFF\n      a = 1\n; FMT: ON\nb = 2\n";
        let out = format(src, normalize()).text;
        assert_eq!(
            out,
            "; FMT: OFF\n      a = 1\n; FMT: ON\nb = 2\n",
            "the protected top-level statement must keep its real on-disk column (6), \
             not be forced to column 0 by Normalize mode"
        );
    }

    #[test]
    fn protected_region_containing_a_diagnosed_block_composes_correctly_with_007s_skip() {
        // 007-formatter-diagnosed-block-indent-fix interaction, tasks.md
        // T008: 007's diagnosed_block_openers-gated children-skip and this
        // feature's marker-gate are both independent "don't touch this
        // range" mechanisms that can apply to overlapping or adjacent
        // territory. Here the FMT region covers only PART of a diagnosed
        // (unmatched PROCESS) subtree — the opener and its direct FILEI
        // child — while a swallowed RUN block (still structurally part of
        // the same diagnosed subtree) sits outside the marked region
        // entirely, protected only by 007's own unchanged mechanism.
        let src = "; FMT: OFF\n    PROCESS PHASE=INPUT\n        FILEI = ni.1\n; FMT: ON\n\n    RUN PGM=HWYASSIGN\n        FILEI NETI = 'net.net'\n    ENDRUN\n";
        let result = format(src, normalize());

        assert_eq!(
            result.diagnostics.len(),
            1,
            "the marker region must not affect parse's diagnostic output at all (FR-006)"
        );
        assert_eq!(result.diagnostics[0].kind, DiagnosticKind::UnmatchedProcess);
        assert!(!result.changed);
        assert_eq!(
            result.text, src,
            "content inside the marked region (PROCESS opener + FILEI) stays untouched by \
             marker-protection regardless of also being diagnosed; the swallowed RUN block \
             outside the marked region stays independently untouched by 007's own unchanged \
             diagnosed-children skip, unaffected by this feature"
        );
    }

    #[test]
    fn parse_output_is_unaffected_by_fmt_markers() {
        // FR-006, tasks.md T009: markers are a pure formatting-only
        // concern with zero effect on tokenize/parse output.
        let with_markers = "if (x=1)\n; FMT: OFF\na = 1\n; FMT: ON\nendif\n";
        let without_markers = "if (x=1)\na = 1\nendif\n";
        let with = parse(with_markers);
        let without = parse(without_markers);
        assert_eq!(
            with.nodes.len(),
            without.nodes.len(),
            "markers must not add/remove top-level nodes"
        );
        assert!(with.diagnostics.is_empty());
        assert!(without.diagnostics.is_empty());
    }

    #[test]
    fn parse_diagnostic_kind_is_unaffected_by_fmt_markers_around_a_diagnosed_block() {
        let with_markers = "; FMT: OFF\nif (x=1)\n; FMT: ON\na = 1\n";
        let without_markers = "if (x=1)\na = 1\n";
        let with = parse(with_markers);
        let without = parse(without_markers);
        assert_eq!(with.diagnostics.len(), 1);
        assert_eq!(without.diagnostics.len(), 1);
        assert_eq!(with.diagnostics[0].kind, without.diagnostics[0].kind);
    }

    #[test]
    fn unclosed_fmt_off_markers_standalone_matches_format_result() {
        let src = "; FMT: OFF\na = 1\n";
        assert_eq!(unclosed_fmt_off_markers(src), vec![Position::new(1, 1)]);
        assert_eq!(
            format(src, FormatOptions::default()).unclosed_fmt_off_markers,
            vec![Position::new(1, 1)]
        );
    }

    #[test]
    fn unclosed_fmt_off_markers_is_empty_in_the_common_case() {
        assert!(unclosed_fmt_off_markers("a = 1\n").is_empty());
        assert!(unclosed_fmt_off_markers("; FMT: OFF\na = 1\n; FMT: ON\n").is_empty());
    }

    #[test]
    fn idempotency_holds_with_protected_and_unclosed_regions() {
        let src = "if (x=1)\n; FMT: OFF\n  a = 1\nendif\n";
        let once = format(src, FormatOptions::default());
        let twice = format(&once.text, FormatOptions::default());
        assert_eq!(once.text, twice.text);
        assert!(!twice.changed);
        assert_eq!(once.unclosed_fmt_off_markers, twice.unclosed_fmt_off_markers);
    }

    // -- Operator spacing (018-operator-spacing) ----------------------------

    fn fixed() -> FormatOptions {
        FormatOptions { operator_spacing: OperatorSpacing::Fixed, ..FormatOptions::default() }
    }

    fn auto() -> FormatOptions {
        FormatOptions { operator_spacing: OperatorSpacing::Auto, ..FormatOptions::default() }
    }

    #[test]
    fn operator_spacing_default_is_preserve() {
        assert_eq!(OperatorSpacing::default(), OperatorSpacing::Preserve);
        assert_eq!(FormatOptions::default().operator_spacing, OperatorSpacing::Preserve);
    }

    #[test]
    fn operator_spacing_preserve_is_byte_identical_to_before_this_feature_existed() {
        // FR-009/SC-003 regression case -- a fixture exercising every
        // operator kind this feature recognizes, confirmed untouched when
        // operator_spacing is left at its default. Deliberately flat (no
        // block nesting) so indentation -- an entirely separate axis --
        // can't also contribute a change here.
        let src = "ZONES   = 1\nMATI=a.mat,MATO=b.mat\nMW[ 1 ]=mi.1.1+mi.2.1\n";
        let result = format(src, FormatOptions::default());
        assert!(!result.changed);
        assert_eq!(result.text, src);
    }

    #[test]
    fn spacing_rebuild_path_handles_multiple_edits_on_one_line_without_corrupting_offsets() {
        let src = "IF ( x==1 )\nENDIF\n";
        let out = format(src, fixed()).text;
        assert_eq!(out, "IF(x == 1)\nENDIF\n");
    }

    #[test]
    fn casing_edit_and_spacing_edit_coexist_correctly_on_one_line() {
        let src = "if ( x==1 )\nendif\n";
        let options = FormatOptions {
            casing: CasingSettings { control_words: CasingConvention::Upper, ..CasingSettings::default() },
            operator_spacing: OperatorSpacing::Fixed,
            ..FormatOptions::default()
        };
        let out = format(src, options).text;
        assert_eq!(out, "IF(x == 1)\nENDIF\n");
    }

    #[test]
    fn spacing_edits_respect_fmt_off_on() {
        let src = "; FMT: OFF\nZONES   = 1\n; FMT: ON\nY   =   2\n";
        let out = format(src, fixed()).text;
        assert_eq!(
            out,
            "; FMT: OFF\nZONES   = 1\n; FMT: ON\nY = 2\n",
            "protected region's spacing untouched, unprotected region normalized"
        );
    }

    #[test]
    fn operator_spacing_fixed_is_idempotent() {
        // 018-operator-spacing tasks.md T020 (post-/speckit-analyze finding
        // I1) -- re-verified directly, not assumed to hold transitively.
        let src = "ZONES   = 1\nMATI=a.mat,MATO=b.mat\nIF ( x==1 )\nMW[ 1 ]=mi.1.1+mi.2.1\nENDIF\n";
        let once = format(src, fixed()).text;
        let twice = format(&once, fixed()).text;
        assert_eq!(once, twice);
    }

    #[test]
    fn operator_spacing_fixed_respects_fmt_off_on_a_full_fixture() {
        // 018-operator-spacing tasks.md T020: a protected region's
        // unnormalized spacing stays exactly as written, while an
        // unprotected region elsewhere in the same file normalizes. `W`'s
        // own 4-space body indent (default indent_width, unrelated to this
        // feature) is expected too -- it's IF's child, indented regardless
        // of operator_spacing.
        let src = "IF (X==1)\n; FMT: OFF\nY   =   2\n; FMT: ON\nENDIF\nIF (Z==1)\nW   =   3\nENDIF\n";
        let out = format(src, fixed()).text;
        assert_eq!(
            out,
            "IF(X == 1)\n; FMT: OFF\nY   =   2\n; FMT: ON\nENDIF\nIF(Z == 1)\n    W = 3\nENDIF\n"
        );
    }

    #[test]
    fn operator_spacing_auto_is_idempotent() {
        // 018-operator-spacing tasks.md T027 -- aligned padding must not
        // grow further on a second pass.
        let src = "A = 1\nBB = 2\nCCC = 3\n";
        let once = format(src, auto()).text;
        let twice = format(&once, auto()).text;
        assert_eq!(once, twice);
    }

    #[test]
    fn operator_spacing_auto_respects_fmt_off_on_a_full_fixture() {
        // 018-operator-spacing tasks.md T027: a protected run's spacing
        // stays exactly as written (never aligned), while an unprotected
        // run elsewhere in the same file aligns normally.
        let src = "; FMT: OFF\nA = 1\nBB = 2\n; FMT: ON\nCCC = 3\nDDDD = 4\n";
        let out = format(src, auto()).text;
        assert_eq!(
            out,
            "; FMT: OFF\nA = 1\nBB = 2\n; FMT: ON\nCCC  = 3\nDDDD = 4\n",
            "protected run untouched; unprotected run aligns to its own longest member"
        );
    }
}
