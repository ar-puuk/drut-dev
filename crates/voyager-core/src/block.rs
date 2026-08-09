//! Block matching: `If`-chain (incl. short-`IF`), `Loop`, `Run` (incl.
//! `!RUN` and implicit closing), `Process`/`PHASE`, `JLoop`, `LinkLoop`,
//! `DistributeMultistep`, nesting, and `BREAK` validity (data-model.md §
//! Block; FR-007, FR-008, FR-009, FR-020, FR-026, FR-028, FR-029, FR-030,
//! FR-033).
//!
//! Structural matching only — no per-program semantic validation (FR-019).
//! In particular, `JLoop`/`LinkLoop`'s documented nesting shapes (FR-029,
//! FR-033) are *not* enforced here: this crate has no diagnostic category
//! for them (contracts/diagnostics.md), so they open/close purely by keyword
//! regardless of what encloses them.

use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::span::Span;
use crate::statement::{Statement, StatementKind};
use crate::token::Token;
use crate::Node;

/// A structural grouping formed by opening and (explicit or implicit)
/// closing statements; may nest (data-model.md § Block).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    /// From the opening statement to the closing statement (explicit or
    /// implicit), or to the last statement seen before end-of-input if
    /// genuinely unmatched.
    pub span: Span,
    /// Nested statements/blocks in source order. Empty (and meaningless) for
    /// `BlockKind::If`, whose real content lives in each `IfBranch`.
    pub children: Vec<Node>,
    /// The explicit closing statement's own span (`ENDIF`/`ENDLOOP`/
    /// `ENDRUN`/`ENDPROCESS`/`ENDPHASE`/`ENDJLOOP`/`ENDLINKLOOP`/
    /// `EndDistributeMULTISTEP`) — `None` when this block closed implicitly
    /// (`Run`/`Process`, FR-009/FR-028) or is genuinely unmatched, in which
    /// case `span.end` falls back to the last child's own span rather than a
    /// real closer. Distinguishing these two cases isn't reconstructible from
    /// `span` alone, which is why this field exists — needed by the
    /// `002-cli-check-format` formatter (closer/opener alignment, FR-012) so
    /// it never re-derives block-matching logic outside this crate
    /// (constitution Principle I).
    pub closer: Option<Span>,
    /// The opening statement's own `keyword=value` pair-keyword-name token
    /// spans, if any (e.g. `PGM` in `RUN PGM=MATRIX ZONES=5` yields one
    /// entry per keyword, not per whole pair) — **not** its condition/value
    /// content. Always empty for `If` in practice (a condition's own `=`
    /// comparisons sit inside parentheses, at nonzero bracket depth, so
    /// they're never mistaken for a keyword=value pair). Exists for the same
    /// reason `closer` does: the opening `Statement` itself is discarded
    /// once matched into this `Block`, so anything a later consumer needs
    /// from it (here: `002-cli-check-format`'s casing rewrite, which must
    /// reach `RUN`/`LOOP`/`PROCESS`/etc.'s own pairs, not just non-block-
    /// forming statements) has to be captured at match time instead.
    pub opener_pairs: Vec<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfBranch {
    /// `None` for an `ELSE` branch; the raw condition tokens (including
    /// surrounding parens) for `IF`/`ELSEIF`.
    pub condition: Option<Vec<Token>>,
    pub children: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockKind {
    /// One `IfBranch` per `IF`/`ELSEIF`/`ELSE` clause. A short-`IF` (FR-007)
    /// is a single branch with no `ELSEIF`/`ELSE`, whose one child is
    /// exactly the statement that trailed it on the same physical line.
    If {
        branches: Vec<IfBranch>,
    },
    Loop {},
    /// `disabled` is true for `!RUN`, which does not get implicit closing.
    Run {
        pgm: Option<String>,
        disabled: bool,
    },
    /// `name` captures the phase name however it was written (`PROCESS
    /// PHASE=name` or the `PHASE=name` shortcut).
    Process {
        name: Option<String>,
    },
    JLoop {},
    LinkLoop {},
    DistributeMultistep {
        process_num: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    If,
    ElseIf,
    Else,
    EndIf,
    Loop,
    EndLoop,
    Run,
    BangRun,
    EndRun,
    Process,
    EndProcess,
    JLoop,
    EndJLoop,
    LinkLoop,
    EndLinkLoop,
    DistributeMultistep,
    EndDistributeMultistep,
    Break,
    Other,
}

fn role_of(stmt: &Statement) -> Role {
    let word = match &stmt.kind {
        StatementKind::Control { word, .. } => word,
        _ => return Role::Other,
    };
    match word.to_ascii_uppercase().as_str() {
        "IF" => Role::If,
        "ELSEIF" => Role::ElseIf,
        "ELSE" => Role::Else,
        "ENDIF" => Role::EndIf,
        "LOOP" => Role::Loop,
        "ENDLOOP" => Role::EndLoop,
        "BREAK" => Role::Break,
        "RUN" => Role::Run,
        "!RUN" => Role::BangRun,
        "ENDRUN" => Role::EndRun,
        "PROCESS" | "PHASE" => Role::Process,
        "ENDPROCESS" | "ENDPHASE" => Role::EndProcess,
        "JLOOP" => Role::JLoop,
        "ENDJLOOP" => Role::EndJLoop,
        "LINKLOOP" => Role::LinkLoop,
        "ENDLINKLOOP" => Role::EndLinkLoop,
        "DISTRIBUTEMULTISTEP" => Role::DistributeMultistep,
        "ENDDISTRIBUTEMULTISTEP" => Role::EndDistributeMultistep,
        _ => Role::Other,
    }
}

fn is_closer_role(role: Role) -> bool {
    matches!(
        role,
        Role::EndIf
            | Role::ElseIf
            | Role::Else
            | Role::EndLoop
            | Role::EndRun
            | Role::EndProcess
            | Role::EndJLoop
            | Role::EndLinkLoop
            | Role::EndDistributeMultistep
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BodyContext {
    /// Top level, or the body of any block kind other than `Run`/`Process`
    /// — no implicit-closer stop signal beyond the closer roles every
    /// context stops on.
    Generic,
    /// The body of an open, non-disabled `Run`: additionally stops
    /// (without consuming) on a sibling `RUN`/`!RUN` or shell-escape
    /// statement (FR-009).
    InsideRunBody,
    /// The body of an open `Process`: additionally stops (without
    /// consuming) on a sibling `PROCESS`/`PHASE=` statement (FR-028).
    InsideProcessBody,
}

fn end_span_or(children: &[Node], fallback: Span) -> Span {
    children.last().map(|n| n.span()).unwrap_or(fallback)
}

/// This statement's own `keyword=value` pair-keyword-name spans (see
/// `Block::opener_pairs`'s doc comment for why this is captured here rather
/// than reconstructed later, once the `Statement` itself is gone).
fn opener_pair_spans(stmt: &Statement) -> Vec<Span> {
    crate::statement::pair_keyword_boundaries(&stmt.tokens)
        .into_iter()
        .map(|(kw_start, _)| stmt.tokens[kw_start].span)
        .collect()
}

fn pair_value_text(stmt: &Statement, keyword: &str) -> Option<String> {
    match &stmt.kind {
        StatementKind::Control { pairs, .. } => pairs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(keyword))
            .and_then(|(_, v)| v.first().map(|t| t.text.clone())),
        _ => None,
    }
}

fn first_pair_value_text(stmt: &Statement) -> Option<String> {
    match &stmt.kind {
        StatementKind::Control { pairs, .. } => pairs
            .first()
            .and_then(|(_, v)| v.first().map(|t| t.text.clone())),
        _ => None,
    }
}

/// Matches the full block structure over an already-built statement
/// sequence (statement.rs), returning the top-level nodes plus every
/// block-matching diagnostic (FR-012, FR-013, FR-016, FR-026).
pub fn match_blocks(statements: Vec<Statement>) -> (Vec<Node>, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let mut nodes = Vec::new();
    let mut i = 0;
    while i < statements.len() {
        let (mut seq_nodes, next_i) = parse_sequence(
            &statements,
            i,
            BodyContext::Generic,
            false,
            &mut diagnostics,
        );
        nodes.append(&mut seq_nodes);
        i = next_i;
        if i < statements.len() {
            // A closer bubbled all the way up with nothing above to claim
            // it: it's dangling.
            let stmt = &statements[i];
            match role_of(stmt) {
                Role::EndIf | Role::ElseIf | Role::Else => {
                    diagnostics.push(Diagnostic::new(
                        DiagnosticKind::UnmatchedIf,
                        stmt.span,
                        "this statement has no open IF for it to belong to",
                    ));
                }
                Role::EndLoop => {
                    diagnostics.push(Diagnostic::new(
                        DiagnosticKind::UnmatchedLoop,
                        stmt.span,
                        "this ENDLOOP has no matching open LOOP",
                    ));
                }
                Role::EndRun => {
                    diagnostics.push(Diagnostic::new(
                        DiagnosticKind::UnmatchedRun,
                        stmt.span,
                        "this ENDRUN has no matching open RUN",
                    ));
                }
                // EndProcess/EndJLoop/EndLinkLoop/EndDistributeMultistep have
                // no diagnostic category (contracts/diagnostics.md) — a
                // dangling one is silently structurally ignored.
                _ => {}
            }
            i += 1;
        }
    }
    (nodes, diagnostics)
}

fn parse_sequence(
    statements: &[Statement],
    mut i: usize,
    context: BodyContext,
    enclosed: bool,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Vec<Node>, usize) {
    let mut nodes = Vec::new();
    while i < statements.len() {
        let role = role_of(&statements[i]);
        if is_closer_role(role) {
            break;
        }
        if context == BodyContext::InsideRunBody
            && (matches!(role, Role::Run | Role::BangRun)
                || matches!(statements[i].kind, StatementKind::ShellEscape { .. }))
        {
            break;
        }
        if context == BodyContext::InsideProcessBody && role == Role::Process {
            break;
        }

        match role {
            Role::If => {
                let (block, next_i) = parse_if_chain(statements, i, diagnostics);
                nodes.push(Node::Block(block));
                i = next_i;
            }
            Role::Loop => {
                let (block, next_i) = parse_simple_block(
                    statements,
                    i,
                    diagnostics,
                    |_| BlockKind::Loop {},
                    Role::EndLoop,
                    Some((
                        DiagnosticKind::UnmatchedLoop,
                        "this LOOP has no matching ENDLOOP before the end of the file",
                    )),
                );
                nodes.push(Node::Block(block));
                i = next_i;
            }
            Role::Run | Role::BangRun => {
                let (block, next_i) = parse_run(statements, i, diagnostics);
                nodes.push(Node::Block(block));
                i = next_i;
            }
            Role::Process => {
                let (block, next_i) = parse_process(statements, i, diagnostics);
                nodes.push(Node::Block(block));
                i = next_i;
            }
            Role::JLoop => {
                let (block, next_i) = parse_simple_block(
                    statements,
                    i,
                    diagnostics,
                    |_| BlockKind::JLoop {},
                    Role::EndJLoop,
                    None,
                );
                nodes.push(Node::Block(block));
                i = next_i;
            }
            Role::LinkLoop => {
                let (block, next_i) = parse_simple_block(
                    statements,
                    i,
                    diagnostics,
                    |_| BlockKind::LinkLoop {},
                    Role::EndLinkLoop,
                    None,
                );
                nodes.push(Node::Block(block));
                i = next_i;
            }
            Role::DistributeMultistep => {
                let (block, next_i) = parse_simple_block(
                    statements,
                    i,
                    diagnostics,
                    |opener| BlockKind::DistributeMultistep {
                        process_num: first_pair_value_text(opener),
                    },
                    Role::EndDistributeMultistep,
                    None,
                );
                nodes.push(Node::Block(block));
                i = next_i;
            }
            Role::Break => {
                if !enclosed {
                    diagnostics.push(Diagnostic::new(
                        DiagnosticKind::MisplacedBreak,
                        statements[i].span,
                        "this BREAK has no enclosing IF/LOOP/RUN/PROCESS/JLOOP/LINKLOOP block",
                    ));
                }
                nodes.push(Node::Statement(statements[i].clone()));
                i += 1;
            }
            Role::EndIf
            | Role::ElseIf
            | Role::Else
            | Role::EndLoop
            | Role::EndRun
            | Role::EndProcess
            | Role::EndJLoop
            | Role::EndLinkLoop
            | Role::EndDistributeMultistep
            | Role::Other => {
                nodes.push(Node::Statement(statements[i].clone()));
                i += 1;
            }
        }
    }
    (nodes, i)
}

fn condition_tokens(stmt: &Statement) -> Vec<Token> {
    stmt.tokens.get(1..).map(|s| s.to_vec()).unwrap_or_default()
}

fn parse_if_chain(
    statements: &[Statement],
    i: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Block, usize) {
    let opener_span = statements[i].span;
    let opener_pairs = opener_pair_spans(&statements[i]);

    // Short-`IF` (FR-007): statement.rs already split the trailing statement
    // off as its own `Statement` whenever one shares the IF's own physical
    // line — so detection here is just "does the very next statement start
    // on the same line this IF statement itself ends on?".
    if i + 1 < statements.len() && statements[i + 1].span.start.line == statements[i].span.end.line
    {
        let body = statements[i + 1].clone();
        let branch_span = statements[i].span.merge(body.span);
        let branches = vec![IfBranch {
            condition: Some(condition_tokens(&statements[i])),
            children: vec![Node::Statement(body)],
            span: branch_span,
        }];
        return (
            Block {
                kind: BlockKind::If { branches },
                span: branch_span,
                children: vec![],
                // A short-IF is a single self-closing branch — there's
                // never an ENDIF to close it.
                closer: None,
                opener_pairs: opener_pairs.clone(),
            },
            i + 2,
        );
    }

    let mut branches: Vec<IfBranch> = Vec::new();
    let mut branch_opener_span = opener_span;
    let mut cond = Some(condition_tokens(&statements[i]));
    let mut idx = i + 1;
    loop {
        let (children, next_i) =
            parse_sequence(statements, idx, BodyContext::Generic, true, diagnostics);
        idx = next_i;
        let branch_span = end_span_or(&children, branch_opener_span);
        branches.push(IfBranch {
            condition: cond.take(),
            children,
            span: branch_opener_span.merge(branch_span),
        });

        if idx >= statements.len() {
            diagnostics.push(Diagnostic::new(
                DiagnosticKind::UnmatchedIf,
                opener_span,
                "this IF has no matching ENDIF before the end of the file",
            ));
            let end = branches.last().map(|b| b.span).unwrap_or(opener_span);
            return (
                Block {
                    kind: BlockKind::If { branches },
                    span: opener_span.merge(end),
                    children: vec![],
                    closer: None,
                    opener_pairs: opener_pairs.clone(),
                },
                idx,
            );
        }

        match role_of(&statements[idx]) {
            Role::ElseIf => {
                branch_opener_span = statements[idx].span;
                cond = Some(condition_tokens(&statements[idx]));
                idx += 1;
            }
            Role::Else => {
                branch_opener_span = statements[idx].span;
                cond = None;
                idx += 1;
            }
            Role::EndIf => {
                let end_span = statements[idx].span;
                idx += 1;
                return (
                    Block {
                        kind: BlockKind::If { branches },
                        span: opener_span.merge(end_span),
                        children: vec![],
                        closer: Some(end_span),
                        opener_pairs: opener_pairs.clone(),
                    },
                    idx,
                );
            }
            _ => {
                // A closer belonging to an ancestor bubbled up to here; this
                // IF itself is unmatched. Don't consume it.
                diagnostics.push(Diagnostic::new(
                    DiagnosticKind::UnmatchedIf,
                    opener_span,
                    "this IF has no matching ENDIF before the end of the file",
                ));
                let end = branches.last().map(|b| b.span).unwrap_or(opener_span);
                return (
                    Block {
                        kind: BlockKind::If { branches },
                        span: opener_span.merge(end),
                        children: vec![],
                        closer: None,
                        opener_pairs: opener_pairs.clone(),
                    },
                    idx,
                );
            }
        }
    }
}

fn parse_run(
    statements: &[Statement],
    i: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Block, usize) {
    let opener_span = statements[i].span;
    let opener_pairs = opener_pair_spans(&statements[i]);
    let disabled = role_of(&statements[i]) == Role::BangRun;
    let pgm = pair_value_text(&statements[i], "PGM");
    let context = if disabled {
        BodyContext::Generic
    } else {
        BodyContext::InsideRunBody
    };
    let (children, mut idx) = parse_sequence(statements, i + 1, context, true, diagnostics);

    if idx < statements.len() && role_of(&statements[idx]) == Role::EndRun {
        let end_span = statements[idx].span;
        idx += 1;
        return (
            Block {
                kind: BlockKind::Run { pgm, disabled },
                span: opener_span.merge(end_span),
                children,
                closer: Some(end_span),
                opener_pairs: opener_pairs.clone(),
            },
            idx,
        );
    }

    if !disabled && idx < statements.len() {
        let r = role_of(&statements[idx]);
        let is_implicit_closer = matches!(r, Role::Run | Role::BangRun)
            || matches!(statements[idx].kind, StatementKind::ShellEscape { .. });
        if is_implicit_closer {
            let end = end_span_or(&children, opener_span);
            return (
                Block {
                    kind: BlockKind::Run { pgm, disabled },
                    span: opener_span.merge(end),
                    children,
                    closer: None,
                    opener_pairs: opener_pairs.clone(),
                },
                idx,
            );
        }
    }

    diagnostics.push(Diagnostic::new(
        DiagnosticKind::UnmatchedRun,
        opener_span,
        "this RUN has no matching ENDRUN and no following RUN/!RUN/shell-escape statement before the end of the file",
    ));
    let end = end_span_or(&children, opener_span);
    (
        Block {
            kind: BlockKind::Run { pgm, disabled },
            span: opener_span.merge(end),
            children,
            closer: None,
            opener_pairs: opener_pairs.clone(),
        },
        idx,
    )
}

fn parse_process(
    statements: &[Statement],
    i: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Block, usize) {
    let opener_span = statements[i].span;
    let opener_pairs = opener_pair_spans(&statements[i]);
    let name = pair_value_text(&statements[i], "PHASE");
    let (children, mut idx) = parse_sequence(
        statements,
        i + 1,
        BodyContext::InsideProcessBody,
        true,
        diagnostics,
    );

    if idx < statements.len() && role_of(&statements[idx]) == Role::EndProcess {
        let end_span = statements[idx].span;
        idx += 1;
        return (
            Block {
                kind: BlockKind::Process { name },
                span: opener_span.merge(end_span),
                children,
                closer: Some(end_span),
                opener_pairs: opener_pairs.clone(),
            },
            idx,
        );
    }

    // Implicit close (next PROCESS/PHASE=, not consumed) or genuinely
    // unmatched — either way, no diagnostic category exists for `Process`
    // (contracts/diagnostics.md), so both cases just resolve the span.
    let end = end_span_or(&children, opener_span);
    (
        Block {
            kind: BlockKind::Process { name },
            span: opener_span.merge(end),
            children,
            closer: None,
            opener_pairs: opener_pairs.clone(),
        },
        idx,
    )
}

fn parse_simple_block(
    statements: &[Statement],
    i: usize,
    diagnostics: &mut Vec<Diagnostic>,
    kind_ctor: impl FnOnce(&Statement) -> BlockKind,
    closer_role: Role,
    unmatched_diag: Option<(DiagnosticKind, &str)>,
) -> (Block, usize) {
    let opener_span = statements[i].span;
    let opener_pairs = opener_pair_spans(&statements[i]);
    let kind = kind_ctor(&statements[i]);
    let (children, mut idx) =
        parse_sequence(statements, i + 1, BodyContext::Generic, true, diagnostics);

    if idx < statements.len() && role_of(&statements[idx]) == closer_role {
        let end_span = statements[idx].span;
        idx += 1;
        return (
            Block {
                kind,
                span: opener_span.merge(end_span),
                children,
                closer: Some(end_span),
                opener_pairs: opener_pairs.clone(),
            },
            idx,
        );
    }

    if let Some((diag_kind, msg)) = unmatched_diag {
        diagnostics.push(Diagnostic::new(diag_kind, opener_span, msg));
    }
    let end = end_span_or(&children, opener_span);
    (
        Block {
            kind,
            span: opener_span.merge(end),
            children,
            closer: None,
            opener_pairs: opener_pairs.clone(),
        },
        idx,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::statement::build_statements;

    fn parse_nodes(src: &str) -> (Vec<Node>, Vec<Diagnostic>) {
        match_blocks(build_statements(tokenize(src)))
    }

    #[test]
    fn if_endif_matches_cleanly() {
        let (nodes, diags) = parse_nodes("IF (X=1)\nY = 2\nENDIF\n");
        assert_eq!(diags.len(), 0);
        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0], Node::Block(b) if matches!(b.kind, BlockKind::If { .. })));
    }

    #[test]
    fn short_if_needs_no_endif() {
        let (nodes, diags) = parse_nodes("IF (X=1) Y = 2\nZ = 3\n");
        assert_eq!(diags.len(), 0);
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn unmatched_if_diagnosed() {
        let (_nodes, diags) = parse_nodes("IF (X=1)\nY = 2\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].kind, DiagnosticKind::UnmatchedIf);
    }

    #[test]
    fn dangling_endif_diagnosed() {
        let (_nodes, diags) = parse_nodes("ENDIF\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].kind, DiagnosticKind::UnmatchedIf);
    }

    #[test]
    fn loop_endloop_matches_and_break_is_fine() {
        let (nodes, diags) = parse_nodes("LOOP i=1,5\nBREAK\nENDLOOP\n");
        assert_eq!(diags.len(), 0);
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn run_explicit_endrun() {
        let (nodes, diags) = parse_nodes("RUN PGM=MATRIX\nX = 1\nENDRUN\n");
        assert_eq!(diags.len(), 0);
        assert_eq!(nodes.len(), 1);
        assert!(
            matches!(&nodes[0], Node::Block(b) if matches!(&b.kind, BlockKind::Run { disabled: false, .. }))
        );
    }

    #[test]
    fn run_implicit_close_by_next_run() {
        let (nodes, diags) = parse_nodes("RUN PGM=MATRIX\nX = 1\nRUN PGM=HIGHWAY\nENDRUN\n");
        assert_eq!(diags.len(), 0);
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn run_implicit_close_by_shell_escape() {
        let (nodes, diags) = parse_nodes("RUN PGM=MATRIX\nX = 1\n*(ECHO done)\n");
        assert_eq!(diags.len(), 0);
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn run_unmatched_no_implicit_or_explicit_closer() {
        let (_nodes, diags) = parse_nodes("RUN PGM=MATRIX\nX = 1\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].kind, DiagnosticKind::UnmatchedRun);
    }

    #[test]
    fn bang_run_requires_explicit_endrun() {
        let (_nodes, diags) = parse_nodes("!RUN PGM=MATRIX\nX = 1\nRUN PGM=HIGHWAY\nENDRUN\n");
        // The !RUN never gets an ENDRUN of its own before the next RUN opens
        // a fresh, unrelated block — !RUN gets no implicit-closer exception.
        assert!(diags.iter().any(|d| d.kind == DiagnosticKind::UnmatchedRun));
    }

    #[test]
    fn process_phase_implicit_close_by_next_phase() {
        let (nodes, diags) =
            parse_nodes("PROCESS PHASE=LINKREAD\nX = 1\nPROCESS PHASE=ILOOP\nENDPROCESS\n");
        assert_eq!(diags.len(), 0);
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn phase_shortcut_and_endphase_spelling() {
        let (nodes, diags) = parse_nodes("PHASE=ILOOP\nX = 1\nENDPHASE\n");
        assert_eq!(diags.len(), 0);
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn jloop_nests_inside_if() {
        let (nodes, diags) = parse_nodes("IF (I=1)\nJLOOP\nX = 1\nENDJLOOP\nENDIF\n");
        assert_eq!(diags.len(), 0);
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn linkloop_nests_inside_loop() {
        let (nodes, diags) = parse_nodes("LOOP i=1,5\nLINKLOOP\nX = 1\nENDLINKLOOP\nENDLOOP\n");
        assert_eq!(diags.len(), 0);
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn distributemultistep_sequential_pair() {
        let (nodes, diags) = parse_nodes("DistributeMULTISTEP\nX = 1\nEndDistributeMULTISTEP\n");
        assert_eq!(diags.len(), 0);
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn break_with_no_enclosing_block_is_misplaced() {
        let (_nodes, diags) = parse_nodes("BREAK\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].kind, DiagnosticKind::MisplacedBreak);
    }

    #[test]
    fn break_inside_bare_if_is_fine() {
        let (_nodes, diags) = parse_nodes("IF (X=1)\nBREAK\nENDIF\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn break_inside_process_phase_is_fine() {
        let (_nodes, diags) = parse_nodes("PROCESS PHASE=ADJUST\nBREAK\nENDPROCESS\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn nested_run_at_deeper_depth_does_not_implicitly_close_outer_run() {
        // Same-nesting-depth-only assumption (spec.md Assumptions on
        // FR-009/FR-028): a RUN nested one level deeper (inside an IF within
        // the open RUN) must not close the outer RUN.
        let (_nodes, diags) =
            parse_nodes("RUN PGM=MATRIX\nIF (X=1)\nRUN PGM=HIGHWAY\nENDRUN\nENDIF\nENDRUN\n");
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn zero_top_level_blocks_is_fine() {
        let (nodes, diags) = parse_nodes("X = 1\nY = 2\n");
        assert_eq!(diags.len(), 0);
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn closer_span_is_none_for_implicit_run_close() {
        let (nodes, diags) = parse_nodes("RUN PGM=MATRIX\nX = 1\nRUN PGM=HIGHWAY\nENDRUN\n");
        assert_eq!(diags.len(), 0);
        let Node::Block(first) = &nodes[0] else {
            panic!("expected a block")
        };
        assert_eq!(first.closer, None, "implicitly-closed RUN must not report a closer span");
    }

    #[test]
    fn closer_span_is_some_and_correct_for_explicit_endrun() {
        let (nodes, _diags) = parse_nodes("RUN PGM=MATRIX\nX = 1\nENDRUN\n");
        let Node::Block(block) = &nodes[0] else {
            panic!("expected a block")
        };
        let closer = block.closer.expect("explicit ENDRUN must be captured");
        assert_eq!(closer.start.line, 3);
    }

    #[test]
    fn closer_span_is_none_for_genuinely_unmatched_block() {
        let (nodes, _diags) = parse_nodes("LOOP i=1,5\nX = 1\n");
        let Node::Block(block) = &nodes[0] else {
            panic!("expected a block")
        };
        assert_eq!(block.closer, None);
    }

    #[test]
    fn opener_pairs_captures_run_pgm_keyword_span() {
        let (nodes, _diags) = parse_nodes("RUN PGM=MATRIX ZONES=5\nX = 1\nENDRUN\n");
        let Node::Block(block) = &nodes[0] else {
            panic!("expected a block")
        };
        assert_eq!(block.opener_pairs.len(), 2, "PGM and ZONES should both be captured");
        // PGM starts right after "RUN " (line 1, column 5).
        assert_eq!(block.opener_pairs[0].start.line, 1);
        assert_eq!(block.opener_pairs[0].start.column, 5);
    }

    #[test]
    fn opener_pairs_is_empty_for_if_condition() {
        // The condition's own "=" sits inside parentheses (nonzero bracket
        // depth), so it must never be mistaken for a keyword=value pair.
        let (nodes, _diags) = parse_nodes("IF (X=1)\nY = 2\nENDIF\n");
        let Node::Block(block) = &nodes[0] else {
            panic!("expected a block")
        };
        assert!(block.opener_pairs.is_empty());
    }

    #[test]
    fn opener_pairs_captures_phase_shortcut_redundantly_with_control_word() {
        // "PHASE=ILOOP" makes PHASE itself both the control word and (per
        // pair_keyword_boundaries' independent scan) a "pair start" at the
        // same span — redundant, not wrong; a casing rewrite applying the
        // same case twice is idempotent.
        let (nodes, _diags) = parse_nodes("PHASE=ILOOP\nX = 1\nENDPHASE\n");
        let Node::Block(block) = &nodes[0] else {
            panic!("expected a block")
        };
        assert_eq!(block.opener_pairs.len(), 1);
        assert_eq!(block.opener_pairs[0].start.column, 1);
    }
}
