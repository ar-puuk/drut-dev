//! `voyager-core`: a dependency-free tokenizer and structural parser for
//! Cube Voyager control-statement scripts (`.s` / `.block`).
//!
//! This crate is the single source of truth for Voyager grammar and parsing
//! logic (constitution Principle I) — every adapter (CLI, LSP, MCP,
//! formatter) is expected to depend on it rather than re-implementing any of
//! this. See `specs/001-voyager-script-parser/contracts/public-api.md` for
//! the binding contract behind [`tokenize`] and [`parse`].

pub mod blank_line;
pub mod block;
pub mod block_resolution;
pub mod data_reference;
pub mod decode;
pub mod diagnostic;
pub mod format;
pub mod function_call;
pub mod grammar_notes;
pub mod keywords;
pub mod lexer;
pub mod line_wrap;
pub mod operator_spacing;
pub mod span;
pub mod statement;
pub mod token;
pub mod token_resolution;

pub use block::{Block, BlockKind};
pub use block_resolution::{all_blocks, block_at, BlockFold, BlockInfo, BlockKindName};
pub use data_reference::{data_reference_entries, data_reference_occurrences, DataReferenceEntry, DataReferenceOccurrence};
pub use diagnostic::{Diagnostic, DiagnosticKind};
pub use format::{
    format, format_bytes, unclosed_fmt_off_markers, BlankLineMode, CasingConvention, CasingSettings, EncodingFidelity,
    FormatOptions, FormatResult, LineWrapMode, LineWrapStyle, OperatorSpacing, IndentTopLevelMode,
};
pub use function_call::{function_call_entries, function_call_occurrences, FunctionCallEntry, FunctionCallOccurrence};
pub use keywords::{completion_candidates, did_you_mean, CompletionContext, KeywordEntry, KeywordRole};
pub use span::{Position, Span};
pub use statement::{Statement, StatementKind};
pub use token::{Token, TokenKind};
pub use token_resolution::{
    all_assignments, all_bareword_reads, all_variable_refs, all_variable_refs_including_openers,
    assignments_outside_run_bodies, read_file_refs, resolve_token_value, variable_ref_at,
    Assignment, ReadFileRef, ResolvedTokenValue, Source as TokenValueSource, VariableRefAt,
};

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

/// Decodes raw `source` bytes (UTF-8 first, falling back per-byte to
/// Windows-1252, FR-034) and tokenizes the result — for callers who only
/// have bytes, not an already-valid `&str`. Any decoding fallback is silent
/// here, same as [`tokenize`]'s own no-diagnostics contract; use
/// [`parse_bytes`] if you need to know about undecodable bytes.
///
/// Never panics on any `&[u8]` input, including arbitrary non-text bytes.
pub fn tokenize_bytes(source: &[u8]) -> Vec<Token> {
    let (text, _decode_diagnostics) = decode::decode_bytes(source);
    tokenize(&text)
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

/// Decodes raw `source` bytes (UTF-8 first, falling back per-byte to
/// Windows-1252, FR-034) and structurally parses the result — for callers
/// who only have bytes, not an already-valid `&str`. A byte undecodable
/// under either encoding produces an [`DiagnosticKind::InvalidEncoding`]
/// diagnostic in the result, ahead of any tokenizing/parsing diagnostics,
/// rather than a rejected call.
///
/// Never panics on any `&[u8]` input (contracts/public-api.md), the same
/// guarantee [`parse`] makes for `&str` input.
pub fn parse_bytes(source: &[u8]) -> ParseResult {
    let (text, decode_diagnostics) = decode::decode_bytes(source);
    let mut result = parse(&text);
    if !decode_diagnostics.is_empty() {
        let mut diagnostics = decode_diagnostics;
        diagnostics.extend(result.diagnostics);
        result.diagnostics = diagnostics;
    }
    result
}
