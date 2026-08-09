//! `voyager-core`: a dependency-free tokenizer and structural parser for
//! Cube Voyager control-statement scripts (`.s` / `.block`).
//!
//! This crate is the single source of truth for Voyager grammar and parsing
//! logic (constitution Principle I) — every adapter (CLI, LSP, MCP,
//! formatter) is expected to depend on it rather than re-implementing any of
//! this. See `specs/001-voyager-script-parser/contracts/public-api.md` for
//! the binding contract behind [`tokenize`] and [`parse`].

pub mod block;
pub mod diagnostic;
pub mod grammar_notes;
pub mod lexer;
pub mod span;
pub mod statement;
pub mod token;

pub use block::{Block, BlockKind};
pub use diagnostic::{Diagnostic, DiagnosticKind};
pub use span::{Position, Span};
pub use statement::{Statement, StatementKind};
pub use token::{Token, TokenKind};

/// Tokenizes `source` into a flat, position-tracked token stream.
///
/// Never panics on any `&str` input, including empty input or arbitrarily
/// malformed text — malformed input simply produces the tokens that make
/// sense of it (comments, `@variable@` references, punctuation, ...); actual
/// defects are only surfaced by [`parse`]'s diagnostics, since `tokenize`
/// itself returns no diagnostics (contracts/public-api.md).
///
/// Calling `tokenize` twice on identical input produces an identical result
/// (no ambient state, clock, locale, or file-path dependency).
pub fn tokenize(source: &str) -> Vec<Token> {
    lexer::tokenize(source)
}

/// A node in [`ParseResult`]'s top-level sequence: either a bare statement or
/// a matched block (data-model.md § ParseResult).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Statement(Statement),
    Block(Block),
}

impl Node {
    pub fn span(&self) -> Span {
        match self {
            Node::Statement(s) => s.span,
            Node::Block(b) => b.span,
        }
    }
}

/// The aggregate value returned by [`parse`] for one input file's text
/// (data-model.md § ParseResult).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParseResult {
    /// Top-level statements/blocks, in source order (FR-020) — zero or more,
    /// with no requirement that they be wrapped in a single `Run` block.
    pub nodes: Vec<Node>,
    /// Possibly empty (SC-001). Never causes parsing to abort early — the
    /// parser accumulates diagnostics and keeps going wherever structurally
    /// feasible (FR-018).
    pub diagnostics: Vec<Diagnostic>,
}

/// Structurally parses `source`, returning the statement/block tree plus any
/// diagnostics.
///
/// Never panics on any `&str` input (contracts/public-api.md) — every defect
/// this phase recognizes (FR-012–FR-016, FR-026) surfaces as a
/// [`Diagnostic`] in the result, never as a panic or an aborted call.
/// Determinism and case-insensitive keyword matching follow the same
/// guarantees as [`tokenize`].
pub fn parse(source: &str) -> ParseResult {
    let (tokens, mut diagnostics) = lexer::tokenize_with_diagnostics(source);
    let statements = statement::build_statements(tokens);
    let (nodes, block_diagnostics) = block::match_blocks(statements);
    diagnostics.extend(block_diagnostics);
    ParseResult { nodes, diagnostics }
}
