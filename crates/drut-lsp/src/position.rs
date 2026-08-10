//! Position-encoding translation (`contracts/position-encoding.md`,
//! research.md §1) — the single place `voyager-core::Position`/`Span`
//! (1-based, `char`-count) is converted to/from `lsp_types::Position`/`Range`
//! (0-based, UTF-16 code-unit count). No handler reimplements this
//! independently (FR-019, FR-020).

use voyager_core::{Position as CorePosition, Span};

/// Converts a `voyager-core` `Position` (1-based line, `char`-count column)
/// into an LSP `Position` (0-based line, UTF-16-code-unit `character`),
/// relative to `text`'s current content.
///
/// Never panics: an out-of-range line/column (e.g. a stale position from a
/// client that hasn't caught up to the latest `didChange`) is clamped to the
/// nearest valid position rather than indexing past it (FR-004).
pub fn to_lsp_position(text: &str, pos: CorePosition) -> lsp_types::Position {
    let line_idx = pos.line.saturating_sub(1);
    let line_text = text.lines().nth(line_idx as usize).unwrap_or("");

    // pos.column is 1-based; walk up to (not including) the column-th char,
    // summing each one's UTF-16 width.
    let target_char_idx = pos.column.saturating_sub(1) as usize;
    let character: u32 = line_text
        .chars()
        .take(target_char_idx)
        .map(|c| c.len_utf16() as u32)
        .sum();

    lsp_types::Position {
        line: line_idx,
        character,
    }
}

/// The inverse of [`to_lsp_position`]: converts an LSP `Position` (0-based
/// line, UTF-16-code-unit `character`) into a `voyager-core` `Position`
/// (1-based line, `char`-count column), relative to `text`'s current content.
///
/// Never panics: a `character` offset beyond the line's actual UTF-16 length
/// is clamped to the line's end rather than indexing past it (FR-004).
pub fn from_lsp_position(text: &str, pos: lsp_types::Position) -> CorePosition {
    let line_text = text.lines().nth(pos.line as usize).unwrap_or("");

    // Walk the line's chars, accumulating UTF-16 units consumed until we
    // reach (or would exceed) the requested character offset.
    let mut utf16_consumed: u32 = 0;
    let mut char_count: u32 = 0;
    for c in line_text.chars() {
        if utf16_consumed >= pos.character {
            break;
        }
        utf16_consumed += c.len_utf16() as u32;
        char_count += 1;
    }

    CorePosition::new(pos.line.saturating_add(1), char_count.saturating_add(1))
}

/// Converts a `voyager-core` `Span` into an LSP `Range`, per
/// [`to_lsp_position`] for both endpoints.
pub fn to_lsp_range(text: &str, span: Span) -> lsp_types::Range {
    lsp_types::Range {
        start: to_lsp_position(text, span.start),
        end: to_lsp_position(text, span.end),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_round_trip() {
        let text = "IF (a=b)\nENDIF\n";
        let core_pos = CorePosition::new(1, 4);
        let lsp_pos = to_lsp_position(text, core_pos);
        assert_eq!(lsp_pos, lsp_types::Position::new(0, 3));
        assert_eq!(from_lsp_position(text, lsp_pos), core_pos);
    }

    #[test]
    fn supplementary_plane_character_counts_as_two_utf16_units() {
        // U+1F600 (😀) is one `char`, two UTF-16 code units.
        let text = "; 😀 comment\n";
        // char index 0 = ';', 1 = ' ', 2 = '😀', 3 = ' ', ...
        // voyager-core column is 1-based char count, so column 4 is right
        // after the emoji.
        let core_pos = CorePosition::new(1, 4);
        let lsp_pos = to_lsp_position(text, core_pos);
        // UTF-16 units before column 4 (chars 0..3, i.e. ';', ' ', '😀'):
        // 1 + 1 + 2 = 4.
        assert_eq!(lsp_pos, lsp_types::Position::new(0, 4));
        assert_eq!(from_lsp_position(text, lsp_pos), core_pos);
    }

    #[test]
    fn out_of_range_column_clamps_rather_than_panics() {
        let text = "short\n";
        let core_pos = CorePosition::new(1, 999);
        let lsp_pos = to_lsp_position(text, core_pos);
        assert_eq!(lsp_pos, lsp_types::Position::new(0, 5));
    }

    #[test]
    fn out_of_range_line_clamps_rather_than_panics() {
        let text = "one line only\n";
        let core_pos = CorePosition::new(50, 1);
        let lsp_pos = to_lsp_position(text, core_pos);
        assert_eq!(lsp_pos, lsp_types::Position::new(49, 0));
    }

    #[test]
    fn out_of_range_lsp_character_clamps_rather_than_panics() {
        let text = "abc\n";
        let lsp_pos = lsp_types::Position::new(0, 999);
        let core_pos = from_lsp_position(text, lsp_pos);
        assert_eq!(core_pos, CorePosition::new(1, 4));
    }

    #[test]
    fn range_translates_both_endpoints() {
        let text = "IF (a=b)\nENDIF\n";
        let span = Span::new(CorePosition::new(1, 1), CorePosition::new(2, 6));
        let range = to_lsp_range(text, span);
        assert_eq!(range.start, lsp_types::Position::new(0, 0));
        assert_eq!(range.end, lsp_types::Position::new(1, 5));
    }
}
