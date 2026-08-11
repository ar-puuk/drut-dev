//! The `lookup_keyword` tool (FR-008, FR-009, data-model.md §6,
//! contracts/mcp-tools.md). No `ScriptSource` — resolved design question
//! from spec.md: the enclosing control word is passed directly as a
//! string, not derived from a script + cursor position.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use voyager_core::{CompletionContext, KeywordRole};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct KeywordLookupInput {
    pub enclosing_control_word: Option<String>,
    pub spellcheck_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct KeywordCandidateDto {
    pub name: String,
    /// `"control_word"` / `"pair_keyword"`.
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct SpellCheckSuggestionDto {
    /// The suggested correct spelling, or `None` when the token already
    /// exactly matches a real keyword or no unique close match exists
    /// within threshold (mirrors `did_you_mean`'s own `Option` return
    /// exactly).
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct KeywordLookupResult {
    pub candidates: Vec<KeywordCandidateDto>,
    pub spellcheck: Option<SpellCheckSuggestionDto>,
}

fn role_name(role: KeywordRole) -> &'static str {
    match role {
        KeywordRole::ControlWord => "control_word",
        KeywordRole::PairKeyword => "pair_keyword",
    }
}

/// Calls `voyager_core::keywords::completion_candidates` (FR-008) and,
/// when `spellcheck_token` is present, `did_you_mean` (FR-009) —
/// independently of each other.
pub fn lookup_keyword(input: &KeywordLookupInput) -> KeywordLookupResult {
    let ctx = CompletionContext {
        enclosing_control_word: input.enclosing_control_word.as_deref(),
    };
    let candidates = voyager_core::completion_candidates(ctx)
        .into_iter()
        .map(|entry| KeywordCandidateDto {
            name: entry.name.to_string(),
            role: role_name(entry.role).to_string(),
        })
        .collect();

    let spellcheck = input.spellcheck_token.as_deref().map(|token| SpellCheckSuggestionDto {
        suggestion: voyager_core::did_you_mean(token).map(|entry| entry.name.to_string()),
    });

    KeywordLookupResult { candidates, spellcheck }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_scoped_lookup_includes_real_census_data() {
        let result = lookup_keyword(&KeywordLookupInput {
            enclosing_control_word: Some("RUN".to_string()),
            spellcheck_token: None,
        });
        let names: Vec<&str> = result.candidates.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"PGM"));
        assert!(names.contains(&"MSG"));
        assert!(names.contains(&"PRNFILE"));
    }

    #[test]
    fn no_control_word_falls_back_to_general_syntax_list() {
        let result = lookup_keyword(&KeywordLookupInput {
            enclosing_control_word: None,
            spellcheck_token: None,
        });
        let names: Vec<&str> = result.candidates.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"IF"));
        assert!(names.contains(&"RUN"));
    }

    #[test]
    fn near_miss_token_gets_a_suggestion() {
        // "PRINT" is one edit from the real keyword "PRINTO" (already
        // established elsewhere in this session's census work).
        let result = lookup_keyword(&KeywordLookupInput {
            enclosing_control_word: None,
            spellcheck_token: Some("PRINT".to_string()),
        });
        assert!(result.spellcheck.unwrap().suggestion.is_some());
    }

    #[test]
    fn exact_match_token_has_no_suggestion() {
        let result = lookup_keyword(&KeywordLookupInput {
            enclosing_control_word: None,
            spellcheck_token: Some("RUN".to_string()),
        });
        assert!(result.spellcheck.unwrap().suggestion.is_none());
    }
}
