//! Block-position resolution — "which block (if any) encloses this
//! position, and where is its matched counterpart" (data-model.md §1,
//! `contracts/block-resolution-api.md`).
//!
//! Extracted 2026-08-10 from `drut-lsp/src/hover.rs` (004-mcp-server
//! research.md §5), not new logic — `drut-lsp`'s hover capability
//! previously had this as private, LSP-coupled logic; a second adapter
//! (`drut-mcp`'s `query_structure` tool) needing the identical fact meant
//! it belonged here instead, per constitution Principle I, rather than
//! being duplicated a second time. The derivation itself — including the
//! five-rule `counterpart` logic below — is unchanged from its original
//! form; only its location and reachability moved.

use crate::block::{Block, BlockKind};
use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::span::{Position, Span};
use crate::Node;

/// Which of the seven block kinds a `BlockInfo` describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKindName {
    If,
    Loop,
    Run,
    Process,
    JLoop,
    LinkLoop,
    DistributeMultistep,
}

impl BlockKindName {
    /// The canonical spelling used by every caller that needs to render
    /// this as text (`drut-lsp`'s hover markdown, `drut-mcp`'s DTO).
    pub fn as_str(&self) -> &'static str {
        match self {
            BlockKindName::If => "If",
            BlockKindName::Loop => "Loop",
            BlockKindName::Run => "Run",
            BlockKindName::Process => "Process",
            BlockKindName::JLoop => "JLoop",
            BlockKindName::LinkLoop => "LinkLoop",
            BlockKindName::DistributeMultistep => "DistributeMultistep",
        }
    }
}

impl std::fmt::Display for BlockKindName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

fn block_kind_name(kind: &BlockKind) -> BlockKindName {
    match kind {
        BlockKind::If { .. } => BlockKindName::If,
        BlockKind::Loop {} => BlockKindName::Loop,
        BlockKind::Run { .. } => BlockKindName::Run,
        BlockKind::Process { .. } => BlockKindName::Process,
        BlockKind::JLoop {} => BlockKindName::JLoop,
        BlockKind::LinkLoop {} => BlockKindName::LinkLoop,
        BlockKind::DistributeMultistep { .. } => BlockKindName::DistributeMultistep,
    }
}

/// The result of resolving a position against a document's parsed structure
/// (data-model.md §1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockInfo {
    pub kind: BlockKindName,
    /// `true` only when `kind == If` and this is a self-closing short-`IF`
    /// (no separate closer by construction).
    pub is_short_if: bool,
    /// The resolved matched-counterpart location, per the five-rule
    /// derivation below.
    pub counterpart: Option<Span>,
}

/// `true` when `block` (an `If`) has no separate closer statement by
/// construction (a self-closing short-`IF`), as opposed to a genuinely
/// unmatched multi-branch `IF` — distinguished by absence of an
/// `UnmatchedIf` diagnostic anchored at this block's own opener.
fn is_short_if(block: &Block, diagnostics: &[Diagnostic]) -> bool {
    if block.closer.is_some() {
        return false;
    }
    !diagnostics
        .iter()
        .any(|d| d.kind == DiagnosticKind::UnmatchedIf && d.span.start == block.span.start)
}

/// `true` when no `UnmatchedRun` diagnostic is anchored at this `Run`
/// block's own opener — meaning it closed implicitly (rule 4 below), the
/// same diagnostic-absence technique `is_short_if` uses.
fn run_closed_implicitly(block: &Block, diagnostics: &[Diagnostic]) -> bool {
    !diagnostics
        .iter()
        .any(|d| d.kind == DiagnosticKind::UnmatchedRun && d.span.start == block.span.start)
}

/// The five-rule `counterpart` derivation (unchanged from its original
/// form — see this module's own doc comment).
fn counterpart_for(block: &Block, diagnostics: &[Diagnostic]) -> Option<Span> {
    if let Some(closer) = block.closer {
        return Some(closer); // Rule 1.
    }
    match &block.kind {
        BlockKind::If { .. } => None, // Rules 2 and 3 (short-IF or genuinely unmatched — either way, None).
        BlockKind::Loop {} | BlockKind::JLoop {} | BlockKind::LinkLoop {} | BlockKind::DistributeMultistep { .. } => {
            None // Rule 3: no implicit-close family for these kinds.
        }
        BlockKind::Run { .. } => {
            // Rule 4.
            if run_closed_implicitly(block, diagnostics) {
                Some(Span::at(block.span.end))
            } else {
                None
            }
        }
        BlockKind::Process { .. } => Some(Span::at(block.span.end)), // Rule 5: unconditional.
    }
}

/// Recursively locates the innermost block whose opener or closer line
/// contains `pos` (approximated as "on the same line as the opener/closer
/// statement" — the block/branch's own span, and `Block.closer`'s span,
/// cover their full body content rather than storing a separate
/// opener-only span, so the line-match is the precise, sound proxy for
/// "the position falls on the keyword itself" that's available without a
/// new dedicated field).
fn find_block_at(nodes: &[Node], pos: Position) -> Option<&Block> {
    for node in nodes {
        if let Node::Block(block) = node {
            // Search nested content first — an inner match is always more
            // specific than this block's own opener/closer line.
            if let Some(found) = find_block_at(&block.children, pos) {
                return Some(found);
            }
            if let BlockKind::If { branches } = &block.kind {
                for branch in branches {
                    if let Some(found) = find_block_at(&branch.children, pos) {
                        return Some(found);
                    }
                }
            }

            if on_opener_or_closer_line(block, pos) {
                return Some(block);
            }
        }
    }
    None
}

fn on_opener_or_closer_line(block: &Block, pos: Position) -> bool {
    if block.span.start.line == pos.line {
        return true;
    }
    if let BlockKind::If { branches } = &block.kind {
        if branches.iter().any(|b| b.span.start.line == pos.line) {
            return true;
        }
    }
    if let Some(closer) = block.closer {
        if closer.start.line == pos.line {
            return true;
        }
    }
    false
}

/// Locates the innermost block enclosing `pos` and resolves its `BlockInfo`
/// — `None` when no block encloses `pos` at all, a normal, successful
/// result for callers, not an error (`contracts/block-resolution-api.md`).
pub fn block_at(nodes: &[Node], diagnostics: &[Diagnostic], pos: Position) -> Option<BlockInfo> {
    let block = find_block_at(nodes, pos)?;
    Some(BlockInfo {
        kind: block_kind_name(&block.kind),
        is_short_if: matches!(block.kind, BlockKind::If { .. }) && is_short_if(block, diagnostics),
        counterpart: counterpart_for(block, diagnostics),
    })
}

// Unit tests for `block_at` live in `tests/block_resolution.rs` (an
// integration test, exercising only this module's public surface) rather
// than an inline `#[cfg(test)]` module here (T019's specified location).
