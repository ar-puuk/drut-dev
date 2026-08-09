//! Source-location primitives (data-model.md § Span).

/// A single 1-based line/column location in source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number (counts `char`s, not bytes).
    pub column: u32,
}

impl Position {
    pub const fn new(line: u32, column: u32) -> Self {
        Position { line, column }
    }

    /// The position one would be at after consuming a single, non-newline
    /// character at this position.
    pub const fn advance(self, ch: char) -> Position {
        if ch == '\n' {
            Position::new(self.line + 1, 1)
        } else {
            Position::new(self.line, self.column + 1)
        }
    }
}

/// A start/end range over source text (FR-002). `end` is never before `start`
/// (data-model.md § Span validation rule) — every constructor in this crate
/// upholds that by construction rather than by runtime assertion, so no input
/// can make one panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    /// Builds a span from `start` to `end`, swapping the two if `end` would
    /// otherwise come before `start` — this keeps the "end is never before
    /// start" rule true for every caller without ever panicking.
    pub fn new(start: Position, end: Position) -> Self {
        if end < start {
            Span {
                start: end,
                end: start,
            }
        } else {
            Span { start, end }
        }
    }

    /// A zero-width span at a single position.
    pub const fn at(pos: Position) -> Self {
        Span {
            start: pos,
            end: pos,
        }
    }

    /// The smallest span containing both `self` and `other`.
    pub fn merge(self, other: Span) -> Span {
        let start = if self.start <= other.start {
            self.start
        } else {
            other.start
        };
        let end = if self.end >= other.end {
            self.end
        } else {
            other.end
        };
        Span { start, end }
    }
}
