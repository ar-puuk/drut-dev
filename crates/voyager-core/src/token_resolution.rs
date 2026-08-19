//! `@name@` hover-value resolution (016-token-hover-value, data-model.md,
//! contracts/token-resolution-api.md) — pure, I/O-free parse-tree analysis,
//! the same category `block_resolution.rs` already occupies (constitution
//! Principle I). Any disk I/O needed to read a `READ FILE` target that isn't
//! already open belongs entirely to the caller (`drut-lsp`), never to this
//! module — every function here operates only on already-parsed `&[Node]`
//! data the caller supplies.
//!
//! **Known, deliberate gap**: a `@token@` reference sitting on a block
//! *opener* line for `RUN`/`LOOP`/`PROCESS`/`JLoop`/`LinkLoop`/
//! `DistributeMultistep` (e.g. `RUN PGM=@Prog@`) is not found by
//! [`variable_ref_at`] — `Block` (see `block.rs`) discards its opener
//! statement's value tokens once matched, keeping only `opener_pairs`' own
//! keyword-name spans, so there is nothing left to scan there. This is a
//! pre-existing data-model constraint, not a scope decision this feature
//! makes — hovering such a position simply finds nothing (falls back to
//! existing behavior, per FR-008), the same as any other unresolvable
//! position. `IF`/`ELSEIF` condition tokens *are* retained (`IfBranch.
//! condition`) and are included in the scan below.

use crate::block::BlockKind;
use crate::span::{Position, Span};
use crate::statement::{Statement, StatementKind};
use crate::token::{Token, TokenKind};
use crate::Node;

/// The `@name@` reference found at a hover position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableRefAt {
    pub name: String,
    pub span: Span,
}

/// One `Assignment` statement, resolution-facing (distinct from
/// `statement::StatementKind::Assignment`, which this borrows from).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Assignment<'a> {
    pub target: &'a str,
    pub value_span: Span,
    pub statement_span: Span,
}

/// One `READ FILE = ...`-shaped `Control` statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadFileRef {
    /// The merged span of the `FILE` pair's raw value tokens (quote
    /// characters included), if that value contains no `VariableRef` token —
    /// `None` if dynamic (token-built) or the pair has no value at all.
    /// Deliberately a `Span`, not a reconstructed `String` (research.md §3):
    /// the lexer splits a quoted value on internal whitespace into multiple
    /// tokens, so naive token-joining would silently drop real spaces (e.g. a
    /// space-bearing directory name) — the caller slices real source text
    /// via `Span` instead.
    pub literal_value_span: Option<Span>,
    pub statement_span: Span,
}

/// Where a [`ResolvedTokenValue`] came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Found directly in the document passed as `nodes`.
    SameFile,
    /// Found in one of the `included` files, identified by the `READ FILE`
    /// statement's own span in the *original* document — not by path or file
    /// name, keeping this module filesystem-naming-free.
    ReadFile { read_file_statement_span: Span },
}

/// The winning assignment for one `@token@` hover resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedTokenValue {
    pub value_span: Span,
    pub statement_span: Span,
    pub source: Source,
}

fn span_contains(span: Span, pos: Position) -> bool {
    span.start <= pos && pos < span.end
}

/// Depth-first walk collecting every real `Statement` in `nodes`, at any
/// nesting depth (inside `Block.children` and, for `If`, each branch's own
/// `children`) — the same traversal shape `block_resolution.rs` already uses.
fn collect_statements<'a>(nodes: &'a [Node], out: &mut Vec<&'a Statement>) {
    for node in nodes {
        match node {
            Node::Statement(s) => out.push(s),
            Node::Block(b) => {
                collect_statements(&b.children, out);
                if let BlockKind::If { branches } = &b.kind {
                    for branch in branches {
                        collect_statements(&branch.children, out);
                    }
                }
            }
        }
    }
}

/// Depth-first walk collecting every `IfBranch.condition` token slice in
/// `nodes` — these aren't wrapped in a `Statement` at all (see `block.rs`),
/// so they need their own, separate collection for `variable_ref_at`.
fn collect_if_condition_token_slices<'a>(nodes: &'a [Node], out: &mut Vec<&'a [Token]>) {
    for node in nodes {
        if let Node::Block(b) = node {
            collect_if_condition_token_slices(&b.children, out);
            if let BlockKind::If { branches } = &b.kind {
                for branch in branches {
                    if let Some(condition) = &branch.condition {
                        out.push(condition);
                    }
                    collect_if_condition_token_slices(&branch.children, out);
                }
            }
        }
    }
}

fn find_variable_ref_in_tokens(tokens: &[Token], pos: Position) -> Option<VariableRefAt> {
    tokens.iter().find_map(|t| match &t.kind {
        TokenKind::VariableRef { name } if span_contains(t.span, pos) => Some(VariableRefAt {
            name: name.clone(),
            span: t.span,
        }),
        _ => None,
    })
}

/// Finds the `@name@` reference at `pos`, if any (data-model.md). Never
/// panics for any `nodes`/`pos` combination.
pub fn variable_ref_at(nodes: &[Node], pos: Position) -> Option<VariableRefAt> {
    let mut statements = Vec::new();
    collect_statements(nodes, &mut statements);
    for s in &statements {
        if let Some(found) = find_variable_ref_in_tokens(&s.tokens, pos) {
            return Some(found);
        }
    }

    let mut condition_slices = Vec::new();
    collect_if_condition_token_slices(nodes, &mut condition_slices);
    for tokens in condition_slices {
        if let Some(found) = find_variable_ref_in_tokens(tokens, pos) {
            return Some(found);
        }
    }

    None
}

/// Depth-first walk collecting every non-empty `Block::opener_tokens` slice in
/// `nodes` — structurally identical to `collect_if_condition_token_slices`,
/// but for the opener statement's own token stream instead of an `IfBranch`'s
/// condition. `If` blocks contribute nothing here: their `opener_tokens` is
/// always empty by construction (`block.rs`), since their condition is
/// already covered by `collect_if_condition_token_slices` -- no
/// double-counting risk to guard against explicitly.
fn collect_opener_token_slices<'a>(nodes: &'a [Node], out: &mut Vec<&'a [Token]>) {
    for node in nodes {
        if let Node::Block(b) = node {
            if !b.opener_tokens.is_empty() {
                out.push(&b.opener_tokens);
            }
            collect_opener_token_slices(&b.children, out);
            if let BlockKind::If { branches } = &b.kind {
                for branch in branches {
                    collect_opener_token_slices(&branch.children, out);
                }
            }
        }
    }
}

fn push_variable_refs_in_tokens(tokens: &[Token], out: &mut Vec<VariableRefAt>) {
    for t in tokens {
        if let TokenKind::VariableRef { name } = &t.kind {
            out.push(VariableRefAt {
                name: name.clone(),
                span: t.span,
            });
        }
    }
}

/// Every `@name@` reference in `nodes`, source order, at any nesting depth
/// (020-undefined-token-diagnostic data-model.md §1) — the "all matches"
/// counterpart to [`variable_ref_at`]'s "first match at a position", same
/// traversal (a block-opener `@token@` is therefore absent from the result,
/// for the same reason `variable_ref_at` can't find it either — `Block`
/// discards its opener statement's value tokens once matched). Never panics
/// for any `nodes`.
pub fn all_variable_refs(nodes: &[Node]) -> Vec<VariableRefAt> {
    let mut statements = Vec::new();
    collect_statements(nodes, &mut statements);
    let mut out = Vec::new();
    for s in &statements {
        push_variable_refs_in_tokens(&s.tokens, &mut out);
    }

    let mut condition_slices = Vec::new();
    collect_if_condition_token_slices(nodes, &mut condition_slices);
    for tokens in condition_slices {
        push_variable_refs_in_tokens(tokens, &mut out);
    }

    // Statements and if-condition slices are collected as two separate
    // passes above, so a naive concatenation wouldn't be true source order
    // (a condition physically precedes its own branch's child statements,
    // but conditions are appended after every statement here) — sorted
    // explicitly so callers can rely on the "source order" guarantee this
    // function documents.
    out.sort_by_key(|r| r.span.start);
    out
}

/// Every `@name@` reference in `nodes`, INCLUDING a reference on a
/// block-opener statement's own line (e.g. `RUN PGM=@Prog@`) —
/// 029-unused-token-diagnostic research.md §1-2. Unlike [`all_variable_refs`],
/// which structurally excludes that position (a pre-existing, separately
/// tested data-model constraint `020-undefined-token-diagnostic` relies on
/// and this function must not disturb), this is a genuinely different
/// function, not a modification: `all_variable_refs` is unchanged. Same
/// "source order, any nesting depth, never panics" contract otherwise.
pub fn all_variable_refs_including_openers(nodes: &[Node]) -> Vec<VariableRefAt> {
    let mut out = all_variable_refs(nodes);

    let mut opener_slices = Vec::new();
    collect_opener_token_slices(nodes, &mut opener_slices);
    for tokens in opener_slices {
        push_variable_refs_in_tokens(tokens, &mut out);
    }

    out.sort_by_key(|r| r.span.start);
    out
}

fn span_of_tokens(tokens: &[Token], fallback_end: Position) -> Span {
    match (tokens.first(), tokens.last()) {
        (Some(first), Some(last)) => first.span.merge(last.span),
        _ => Span::at(fallback_end),
    }
}

/// Every `StatementKind::Assignment` in `nodes`, source order, at any nesting
/// depth. Empty `Vec` (never a panic) for a document with none.
pub fn all_assignments(nodes: &[Node]) -> Vec<Assignment<'_>> {
    let mut statements = Vec::new();
    collect_statements(nodes, &mut statements);

    statements
        .into_iter()
        .filter_map(|s| match &s.kind {
            StatementKind::Assignment { target, value } => Some(Assignment {
                target,
                value_span: span_of_tokens(value, s.span.end),
                statement_span: s.span,
            }),
            _ => None,
        })
        .collect()
}

/// `true` if `tokens` contains at least one `VariableRef` token — a
/// token-built (dynamic) value, per FR-003's permanent exclusion.
fn contains_variable_ref(tokens: &[Token]) -> bool {
    tokens
        .iter()
        .any(|t| matches!(t.kind, TokenKind::VariableRef { .. }))
}

/// Every `READ FILE = ...`-shaped `Control` statement in `nodes`, source
/// order, at any nesting depth.
pub fn read_file_refs(nodes: &[Node]) -> Vec<ReadFileRef> {
    let mut statements = Vec::new();
    collect_statements(nodes, &mut statements);

    statements
        .into_iter()
        .filter_map(|s| match &s.kind {
            StatementKind::Control { word, pairs } if word.eq_ignore_ascii_case("READ") => pairs
                .iter()
                .find(|(keyword, _)| keyword.eq_ignore_ascii_case("FILE"))
                .map(|(_, value)| ReadFileRef {
                    literal_value_span: if value.is_empty() || contains_variable_ref(value) {
                        None
                    } else {
                        Some(span_of_tokens(value, s.span.end))
                    },
                    statement_span: s.span,
                }),
            _ => None,
        })
        .collect()
}

/// One candidate assignment under consideration, tagged with the position it
/// should be ordered by (its own real position for a same-file candidate, or
/// its originating `READ FILE` statement's position for an included one —
/// spec.md FR-004's interleaving rule) and where it came from.
struct Candidate {
    order_pos: Position,
    value_span: Span,
    statement_span: Span,
    source: Source,
}

/// Resolves `name`'s most-recent value visible at `pos` in `nodes`, given
/// zero or more `included` files (each a `(read_file_statement_span,
/// parsed_nodes)` pair the caller already read and parsed off disk for a
/// literal `READ FILE` statement found in `nodes` — spec.md FR-003).
/// Case-insensitive name matching (FR-005). Returns `None` if no assignment
/// in scope, ordered at or before `pos`, matches `name` — never a fabricated
/// or near-match value (FR-008).
pub fn resolve_token_value(
    nodes: &[Node],
    pos: Position,
    included: &[(Span, Vec<Node>)],
    name: &str,
) -> Option<ResolvedTokenValue> {
    let mut candidates: Vec<Candidate> = all_assignments(nodes)
        .into_iter()
        .filter(|a| a.target.eq_ignore_ascii_case(name))
        .map(|a| Candidate {
            order_pos: a.statement_span.start,
            value_span: a.value_span,
            statement_span: a.statement_span,
            source: Source::SameFile,
        })
        .collect();

    for (read_file_span, included_nodes) in included {
        candidates.extend(
            all_assignments(included_nodes)
                .into_iter()
                .filter(|a| a.target.eq_ignore_ascii_case(name))
                .map(|a| Candidate {
                    order_pos: read_file_span.start,
                    value_span: a.value_span,
                    statement_span: a.statement_span,
                    source: Source::ReadFile {
                        read_file_statement_span: *read_file_span,
                    },
                }),
        );
    }

    candidates
        .into_iter()
        .filter(|c| c.order_pos <= pos)
        .max_by_key(|c| c.order_pos)
        .map(|c| ResolvedTokenValue {
            value_span: c.value_span,
            statement_span: c.statement_span,
            source: c.source,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn variable_ref_at_finds_reference_and_misses_just_outside_it() {
        let result = parse("PRINT LIST='@MODE@'\n");
        // "@MODE@" starts at column 13 (1-based) on line 1.
        let inside = variable_ref_at(&result.nodes, Position::new(1, 15)).unwrap();
        assert_eq!(inside.name, "MODE");
        assert!(variable_ref_at(&result.nodes, Position::new(1, 1)).is_none());
    }

    #[test]
    fn variable_ref_at_finds_reference_inside_if_condition() {
        let result = parse("IF (@MODE@ = 1)\nENDIF\n");
        let found = variable_ref_at(&result.nodes, Position::new(1, 6)).unwrap();
        assert_eq!(found.name, "MODE");
    }

    #[test]
    fn variable_ref_at_finds_reference_nested_in_loop() {
        let result = parse("LOOP I=1,2\nPRINT LIST='@MODE@'\nENDLOOP\n");
        let found = variable_ref_at(&result.nodes, Position::new(2, 15)).unwrap();
        assert_eq!(found.name, "MODE");
    }

    #[test]
    fn all_assignments_finds_top_level_and_nested() {
        let result = parse("ZoneMsgRate = 50\nIF (a=b)\nUsedZones = 3629\nENDIF\n");
        let assignments = all_assignments(&result.nodes);
        let targets: Vec<&str> = assignments.iter().map(|a| a.target).collect();
        assert_eq!(targets, vec!["ZoneMsgRate", "UsedZones"]);
    }

    #[test]
    fn all_assignments_empty_for_document_with_none() {
        let result = parse("PRINT LIST='hello'\n");
        assert!(all_assignments(&result.nodes).is_empty());
    }

    #[test]
    fn all_variable_refs_finds_every_reference_in_source_order() {
        let result = parse("MSG1 = @First@\nIF (@Second@ = 1)\nMSG2 = @Third@\nENDIF\n");
        let refs = all_variable_refs(&result.nodes);
        let names: Vec<&str> = refs.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["First", "Second", "Third"]);
        // Confirms the explicit sort-by-position in all_variable_refs is
        // actually doing something: statements and if-condition slices are
        // collected as two separate passes internally, so without the sort
        // this would come back as ["First", "Third", "Second"] instead.
        for pair in refs.windows(2) {
            assert!(pair[0].span.start < pair[1].span.start);
        }
    }

    #[test]
    fn all_variable_refs_excludes_a_block_opener_reference() {
        // No Prog = ... assignment anywhere; @Prog@ appears only on the
        // RUN block-opener line, which Block discards its opener
        // statement's value tokens for once matched -- so this reference
        // must be structurally absent from the result (research.md §3),
        // the same reason variable_ref_at can't find it either.
        let result = parse("RUN PGM=@Prog@\nENDRUN\n");
        assert!(all_variable_refs(&result.nodes).is_empty());
    }

    #[test]
    fn all_variable_refs_empty_for_document_with_none() {
        let result = parse("X = 1\n");
        assert!(all_variable_refs(&result.nodes).is_empty());
    }

    #[test]
    fn all_variable_refs_including_openers_covers_everything_all_variable_refs_does() {
        let result = parse("MSG1 = @First@\nIF (@Second@ = 1)\nMSG2 = @Third@\nENDIF\n");
        let plain = all_variable_refs(&result.nodes);
        let with_openers = all_variable_refs_including_openers(&result.nodes);
        let plain_names: Vec<&str> = plain.iter().map(|r| r.name.as_str()).collect();
        let with_openers_names: Vec<&str> = with_openers.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(plain_names, with_openers_names);
    }

    #[test]
    fn all_variable_refs_including_openers_finds_a_block_opener_reference() {
        // The one behavioral difference from all_variable_refs: @Prog@ here
        // sits only on the RUN block-opener line, which Block::opener_tokens
        // now preserves.
        let result = parse("RUN PGM=@Prog@\nENDRUN\n");
        let refs = all_variable_refs_including_openers(&result.nodes);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].name, "Prog");
    }

    #[test]
    fn all_variable_refs_including_openers_preserves_source_order() {
        let result = parse("MSG1 = @First@\nRUN PGM=@Second@\nENDRUN\nMSG2 = @Third@\n");
        let refs = all_variable_refs_including_openers(&result.nodes);
        let names: Vec<&str> = refs.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["First", "Second", "Third"]);
        for pair in refs.windows(2) {
            assert!(pair[0].span.start < pair[1].span.start);
        }
    }

    #[test]
    fn all_variable_refs_including_openers_empty_for_document_with_none() {
        let result = parse("X = 1\n");
        assert!(all_variable_refs_including_openers(&result.nodes).is_empty());
    }

    #[test]
    fn read_file_refs_classifies_literal_and_dynamic_paths() {
        let result = parse(
            "READ FILE = '_ControlCenter.block'\nREAD FILE = '@ParentDir@sub\\path.block'\n",
        );
        let refs = read_file_refs(&result.nodes);
        assert_eq!(refs.len(), 2);
        assert!(refs[0].literal_value_span.is_some());
        assert!(refs[1].literal_value_span.is_none());
    }

    #[test]
    fn read_file_refs_preserves_internal_whitespace_via_span() {
        let source = "READ FILE = 'Network Processing Tools\\x.block'\n";
        let result = parse(source);
        let refs = read_file_refs(&result.nodes);
        let span = refs[0].literal_value_span.unwrap();
        // Reconstruct via naive line-slicing, mirroring what drut-lsp's
        // text_for_span will do, to prove the space survives.
        let line = source.lines().next().unwrap();
        let start_col = span.start.column as usize - 1;
        let end_col = span.end.column as usize - 1;
        let sliced = &line[start_col..end_col];
        assert_eq!(sliced, "'Network Processing Tools\\x.block'");
    }

    #[test]
    fn read_file_refs_empty_for_document_with_none() {
        let result = parse("PRINT LIST='hello'\n");
        assert!(read_file_refs(&result.nodes).is_empty());
    }

    #[test]
    fn resolve_token_value_picks_most_recent_before_position() {
        let result = parse("ZoneMsgRate = 50\nZoneMsgRate = 60\nPRINT LIST='@ZoneMsgRate@'\n");
        let pos = Position::new(3, 13); // inside @ZoneMsgRate@ on line 3.
        let resolved = resolve_token_value(&result.nodes, pos, &[], "ZoneMsgRate").unwrap();
        // Must pick line 2's reassignment (60), not line 1's original (50).
        assert_eq!(resolved.statement_span.start.line, 2);
        assert_eq!(resolved.value_span.start.line, 2);
    }

    #[test]
    fn resolve_token_value_never_selects_an_assignment_after_pos() {
        let result = parse("PRINT LIST='@ZoneMsgRate@'\nZoneMsgRate = 50\n");
        let pos = Position::new(1, 13);
        assert!(resolve_token_value(&result.nodes, pos, &[], "ZoneMsgRate").is_none());
    }

    #[test]
    fn resolve_token_value_case_insensitive() {
        let result = parse("ParentDir = 'C:\\proj'\nPRINT LIST='@PARENTDIR@'\n");
        let pos = Position::new(2, 13);
        assert!(resolve_token_value(&result.nodes, pos, &[], "PARENTDIR").is_some());
    }

    #[test]
    fn resolve_token_value_none_when_nothing_matches() {
        let result = parse("PRINT LIST='@Nope@'\n");
        let pos = Position::new(1, 13);
        assert!(resolve_token_value(&result.nodes, pos, &[], "Nope").is_none());
    }

    #[test]
    fn resolve_token_value_interleaves_included_file_by_read_file_position() {
        // Same-file document: READ FILE at line 1, own reassignment at line 3,
        // hover at line 4. Included file assigns UsedZones = 3629.
        let doc = parse("READ FILE = 'sibling.block'\n\nUsedZones = 1\nPRINT LIST='@UsedZones@'\n");
        let included_doc = parse("UsedZones = 3629\n");
        let read_file_span = read_file_refs(&doc.nodes)[0].statement_span;
        let included = vec![(read_file_span, included_doc.nodes)];
        let pos = Position::new(4, 13);
        let resolved = resolve_token_value(&doc.nodes, pos, &included, "UsedZones").unwrap();
        // The document's own later reassignment (line 3) must win over the
        // included file's value, which is ordered at the READ FILE line (1).
        assert_eq!(resolved.statement_span.start.line, 3);
        assert_eq!(resolved.source, Source::SameFile);
    }

    #[test]
    fn resolve_token_value_uses_included_file_when_no_same_file_override() {
        let doc = parse("READ FILE = 'sibling.block'\nPRINT LIST='@UsedZones@'\n");
        let included_doc = parse("UsedZones = 3629\n");
        let read_file_span = read_file_refs(&doc.nodes)[0].statement_span;
        let included = vec![(read_file_span, included_doc.nodes)];
        let pos = Position::new(2, 13);
        let resolved = resolve_token_value(&doc.nodes, pos, &included, "UsedZones").unwrap();
        assert!(matches!(resolved.source, Source::ReadFile { .. }));
    }

    #[test]
    fn resolve_token_value_empty_included_degrades_to_same_file_only() {
        let result = parse("ZoneMsgRate = 50\nPRINT LIST='@ZoneMsgRate@'\n");
        let pos = Position::new(2, 13);
        assert!(resolve_token_value(&result.nodes, pos, &[], "ZoneMsgRate").is_some());
    }
}
