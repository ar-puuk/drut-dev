//! Lexical tokens (data-model.md § Token; FR-002, FR-004, FR-005, FR-006, FR-010).

use crate::span::Span;

/// The smallest recognized lexical unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    /// Raw source text this token covers, casing preserved (contracts/
    /// public-api.md's case-sensitivity guarantee).
    pub text: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: impl Into<String>) -> Self {
        Token {
            kind,
            span,
            text: text.into(),
        }
    }

    /// True for the continuation characters this crate recognizes at the end
    /// of a physical line (FR-006): `, + - / * ^ & | =`.
    pub fn is_continuation_char_text(text: &str) -> bool {
        matches!(text, "," | "+" | "-" | "/" | "*" | "^" | "&" | "|" | "=")
    }
}

/// See data-model.md § Token for the rationale behind each variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// A control word, keyword, identifier, or value fragment.
    Word,
    /// `; ...` running to the end of the physical line (FR-004).
    LineComment,
    /// `/* ... */`, possibly spanning multiple lines and possibly nested
    /// (FR-005). `unterminated` is true when end-of-input was reached before
    /// this comment's own matching `*/` was found.
    BlockComment { unterminated: bool },
    /// The trailing `, + - / * ^ & | =` character that joins a statement to
    /// its next physical line (FR-006).
    ContinuationMarker,
    /// An `@name@` substitution reference; `name` excludes the `@` delimiters
    /// (FR-010).
    VariableRef { name: String },
    /// Structural characters not covered above: brackets, parens, quotes,
    /// `:`, `!`, and `=`/`{`/`}` when not serving as a continuation marker or
    /// brace-continuation delimiter respectively.
    Punctuation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn variable_ref_token_has_no_evaluation_just_name_and_position() {
        let toks = tokenize("MSG = @AOC_Auto@\n");
        let var = toks
            .iter()
            .find(|t| matches!(&t.kind, TokenKind::VariableRef { .. }))
            .expect("expected a VariableRef token");
        match &var.kind {
            TokenKind::VariableRef { name } => assert_eq!(name, "AOC_Auto"),
            _ => unreachable!(),
        }
        assert!(var.span.start.line >= 1);
    }
}
