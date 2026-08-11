//! Structured parsing problems (data-model.md § Diagnostic; FR-017;
//! contracts/diagnostics.md).

use crate::span::Span;

/// One of the structural defect categories this phase recognizes
/// (contracts/diagnostics.md). Consumers must not assume this set is closed —
/// new kinds may be added later within the same non-semantic scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticKind {
    /// FR-012: an `IF` with no matching `ENDIF`, or a dangling `ENDIF`/
    /// `ELSEIF`/`ELSE` with no open `IF`.
    UnmatchedIf,
    /// FR-013: a `LOOP` with no matching `ENDLOOP`, or a dangling `ENDLOOP`.
    UnmatchedLoop,
    /// FR-014: a block comment with no matching `*/` before end-of-input.
    UnclosedBlockComment,
    /// FR-015: a continuation character with no valid following line.
    InvalidContinuation,
    /// FR-016: a non-disabled `RUN` with no `ENDRUN` and no implicit closer,
    /// a disabled `!RUN` missing its required explicit `ENDRUN`, or a
    /// dangling `ENDRUN`.
    UnmatchedRun,
    /// 006-unmatched-process-diagnostic FR-002: a `PROCESS`/`PHASE=` with no
    /// matching `ENDPROCESS`/`ENDPHASE` and no following `PROCESS`/`PHASE=`
    /// statement (the legitimate implicit-close pattern) before either
    /// end-of-input or the enclosing block's own closer forces an early
    /// stop — mirrors `UnmatchedRun`'s firing condition exactly.
    UnmatchedProcess,
    /// FR-026: a `BREAK` with no enclosing block of any kind.
    MisplacedBreak,
    /// FR-034: a raw input byte (`tokenize_bytes`/`parse_bytes` only) that is
    /// not valid UTF-8 and has no defined Windows-1252 interpretation either,
    /// replaced with the Unicode replacement character.
    InvalidEncoding,
}

/// A structured record of a parsing problem (FR-017). Every field is always
/// populated — there is no "message-only" diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    /// Anchored at the offending statement/token (FR-012–FR-016, FR-026), or
    /// at the offending decoded character (FR-034).
    pub span: Span,
    /// Original wording, composed once per kind (constitution Principle II,
    /// FR-024) — never copied from vendor documentation.
    pub message: String,
}

impl Diagnostic {
    pub fn new(kind: DiagnosticKind, span: Span, message: impl Into<String>) -> Self {
        Diagnostic {
            kind,
            span,
            message: message.into(),
        }
    }
}
