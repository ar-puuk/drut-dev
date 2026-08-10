//! Spell-check "did you mean" (FR-014–FR-015). Not a distinct LSP
//! capability — rides on hover and completion responses
//! (`contracts/lsp-capabilities.md`, constitution Principle VI).
//!
//! `hover.rs`/`completion.rs` call [`hint_for`] directly at the point they
//! build their own responses, appending its suggestion when present, rather
//! than this module owning a request handler of its own.

use voyager_core::{keywords, Position as CorePosition, Span, StatementKind, Token, TokenKind};

/// A "did you mean" suggestion for one misspelled token.
pub struct SpellCheckHint {
    pub token_span: Span,
    pub suggestion: &'static str,
}

/// Looks up a "did you mean" hint for the `Word` token at `pos`, if any
/// (FR-014). Only ever produced for a token that isn't already a dictionary
/// exact-match — `did_you_mean` itself already returns `None` for an exact
/// match, so no separate check is needed here (FR-015).
pub fn hint_for(text: &str, pos: CorePosition) -> Option<SpellCheckHint> {
    let tokens = voyager_core::tokenize(text);
    let token = tokens.iter().find(|t: &&Token| {
        matches!(t.kind, TokenKind::Word) && t.span.start <= pos && pos <= t.span.end
    })?;

    let entry = keywords::did_you_mean(&token.text)?;
    Some(SpellCheckHint {
        token_span: token.span,
        suggestion: entry.name,
    })
}

/// `true` when `word` is already a recognized control word or a token
/// structurally classified as a `Control` statement's own first word — used
/// by callers to decide whether spell-check is even relevant for a given
/// token (a `Word` inside an ordinary value position is never checked).
pub fn is_control_word_position(kind: &StatementKind, word: &str) -> bool {
    matches!(kind, StatementKind::Control { word: w, .. } if w == word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_for_close_typo() {
        let text = "FI (a=b)\nENDIF\n";
        let result = hint_for(text, CorePosition::new(1, 1));
        assert_eq!(result.map(|h| h.suggestion), Some("IF"));
    }

    #[test]
    fn hint_for_exact_match_is_none() {
        let text = "IF (a=b)\nENDIF\n";
        let result = hint_for(text, CorePosition::new(1, 1));
        assert!(result.is_none());
    }

    #[test]
    fn hint_for_unrelated_token_is_none() {
        let text = "PATHLOAD FILE=x.mat\n";
        let result = hint_for(text, CorePosition::new(1, 15)); // inside "x.mat"
        assert!(result.is_none());
    }
}
