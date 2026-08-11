//! `ScriptSource` (data-model.md §2) — the shared "text or path, never both"
//! input shape every tool but `lookup_keyword` accepts (FR-002).

use schemars::JsonSchema;
use serde::Deserialize;

/// Exactly one of `text`/`path` MUST be set — both or neither is a
/// structured error (FR-002, Edge Cases), never a silent guess at which one
/// the caller "probably meant."
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ScriptSource {
    /// Inline script content.
    pub text: Option<String>,
    /// A file path to read script content from.
    pub path: Option<String>,
}

/// The resolved content, tagged by whether it's already text or needs the
/// same byte-level decode `voyager_core::parse_bytes`/`format_bytes` already
/// own — deliberately not pre-decoded here, so every tool calls the exact
/// `voyager-core` entry point (`parse`/`format` for `Text`, `parse_bytes`/
/// `format_bytes` for `Bytes`) that already handles decoding, encoding-
/// fidelity determination, and `InvalidEncoding` reporting correctly, with
/// zero duplication of that logic in this crate (research.md §6's
/// translate-at-the-boundary pattern, applied one level earlier than the
/// DTO conversion itself).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedSource {
    Text(String),
    Bytes(Vec<u8>),
}

/// `ScriptSource`'s own resolution failure — both/neither `text`/`path` set,
/// or a `path` that couldn't be read. Every tool converts this into its own
/// structured MCP tool-call error rather than ever panicking (FR-012).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceError {
    BothTextAndPathSet,
    NeitherTextNorPathSet,
    /// The `path`'s own string, plus the `io::Error`'s message text (not the
    /// `io::Error` itself, to keep this type `Clone`/`PartialEq`/`Eq` —
    /// matching every other DTO-adjacent type in this crate).
    PathUnreadable { path: String, message: String },
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceError::BothTextAndPathSet => {
                write!(f, "exactly one of `text`/`path` must be set, but both were")
            }
            SourceError::NeitherTextNorPathSet => {
                write!(f, "exactly one of `text`/`path` must be set, but neither was")
            }
            SourceError::PathUnreadable { path, message } => {
                write!(f, "couldn't read `path` {path:?}: {message}")
            }
        }
    }
}

impl std::error::Error for SourceError {}

impl ScriptSource {
    pub fn resolve(&self) -> Result<ResolvedSource, SourceError> {
        match (&self.text, &self.path) {
            (Some(_), Some(_)) => Err(SourceError::BothTextAndPathSet),
            (None, None) => Err(SourceError::NeitherTextNorPathSet),
            (Some(text), None) => Ok(ResolvedSource::Text(text.clone())),
            (None, Some(path)) => std::fs::read(path)
                .map(ResolvedSource::Bytes)
                .map_err(|err| SourceError::PathUnreadable {
                    path: path.clone(),
                    message: err.to_string(),
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_text_and_path_set_is_an_error() {
        let source = ScriptSource {
            text: Some("IF (a=b)\nENDIF\n".to_string()),
            path: Some("somewhere.s".to_string()),
        };
        assert_eq!(source.resolve(), Err(SourceError::BothTextAndPathSet));
    }

    #[test]
    fn neither_text_nor_path_set_is_an_error() {
        let source = ScriptSource { text: None, path: None };
        assert_eq!(source.resolve(), Err(SourceError::NeitherTextNorPathSet));
    }

    #[test]
    fn text_only_resolves_to_that_text() {
        let source = ScriptSource {
            text: Some("IF (a=b)\nENDIF\n".to_string()),
            path: None,
        };
        assert_eq!(
            source.resolve(),
            Ok(ResolvedSource::Text("IF (a=b)\nENDIF\n".to_string()))
        );
    }

    #[test]
    fn path_only_resolves_to_that_files_bytes() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("drut_mcp_source_test_{}.s", std::process::id()));
        std::fs::write(&path, b"IF (a=b)\nENDIF\n").unwrap();

        let source = ScriptSource {
            text: None,
            path: Some(path.to_string_lossy().to_string()),
        };
        assert_eq!(
            source.resolve(),
            Ok(ResolvedSource::Bytes(b"IF (a=b)\nENDIF\n".to_vec()))
        );

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn nonexistent_path_is_a_structured_error_not_a_panic() {
        let source = ScriptSource {
            text: None,
            path: Some("this/path/definitely/does/not/exist.s".to_string()),
        };
        let err = source.resolve().unwrap_err();
        assert!(matches!(err, SourceError::PathUnreadable { .. }));
    }
}
