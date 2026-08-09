//! Byte-oriented decoding for `tokenize_bytes`/`parse_bytes` (FR-034): UTF-8
//! first, falling back per-byte to Windows-1252 wherever UTF-8 is invalid —
//! real production Voyager scripts are not guaranteed to be valid UTF-8 (a
//! single stray Windows-1252 "smart quote" was found in this project's own
//! real fixture corpus, T049).
//!
//! Decoding is surgical, not whole-file: only the specific invalid byte is
//! reinterpreted, leaving every other valid byte — including legitimate
//! non-ASCII UTF-8 elsewhere in the same file — untouched. A byte with no
//! defined Windows-1252 interpretation is replaced with the Unicode
//! replacement character and produces an `InvalidEncoding` diagnostic; a byte
//! that resolves successfully under either encoding produces none —
//! recovering from an encoding quirk is not itself a defect.

use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::span::{Position, Span};

/// Windows-1252's interpretation of a single byte, or `None` for the five
/// code points (`0x81`, `0x8D`, `0x8F`, `0x90`, `0x9D`) it leaves undefined.
/// `0x00..=0x7F` is plain ASCII; `0xA0..=0xFF` matches Latin-1 (ISO-8859-1)
/// exactly, so only the `0x80..=0x9F` range needs its own table.
fn windows1252_char(byte: u8) -> Option<char> {
    match byte {
        0x00..=0x7F => Some(byte as char),
        0x80 => Some('\u{20AC}'),
        0x81 => None,
        0x82 => Some('\u{201A}'),
        0x83 => Some('\u{0192}'),
        0x84 => Some('\u{201E}'),
        0x85 => Some('\u{2026}'),
        0x86 => Some('\u{2020}'),
        0x87 => Some('\u{2021}'),
        0x88 => Some('\u{02C6}'),
        0x89 => Some('\u{2030}'),
        0x8A => Some('\u{0160}'),
        0x8B => Some('\u{2039}'),
        0x8C => Some('\u{0152}'),
        0x8D => None,
        0x8E => Some('\u{017D}'),
        0x8F => None,
        0x90 => None,
        0x91 => Some('\u{2018}'),
        0x92 => Some('\u{2019}'),
        0x93 => Some('\u{201C}'),
        0x94 => Some('\u{201D}'),
        0x95 => Some('\u{2022}'),
        0x96 => Some('\u{2013}'),
        0x97 => Some('\u{2014}'),
        0x98 => Some('\u{02DC}'),
        0x99 => Some('\u{2122}'),
        0x9A => Some('\u{0161}'),
        0x9B => Some('\u{203A}'),
        0x9C => Some('\u{0153}'),
        0x9D => None,
        0x9E => Some('\u{017E}'),
        0x9F => Some('\u{0178}'),
        0xA0..=0xFF => Some(byte as char),
    }
}

/// Decodes `source`, never failing: valid UTF-8 passes through untouched,
/// and any invalid byte is individually reinterpreted under Windows-1252 (or
/// replaced with U+FFFD, with a diagnostic, if even that has no defined
/// interpretation for it). The returned `Position`/`Span` values use the same
/// running `char` count as every other diagnostic in this crate — not a raw
/// byte offset — so `InvalidEncoding` doesn't introduce a second,
/// inconsistent position scheme (see data-model.md § Span).
pub fn decode_bytes(source: &[u8]) -> (String, Vec<Diagnostic>) {
    let mut out = String::with_capacity(source.len());
    let mut diagnostics = Vec::new();
    let mut pos = Position::new(1, 1);
    let mut rest = source;

    while !rest.is_empty() {
        match std::str::from_utf8(rest) {
            Ok(valid) => {
                for ch in valid.chars() {
                    pos = pos.advance(ch);
                }
                out.push_str(valid);
                break;
            }
            Err(err) => {
                // Deliberately not reading `err.error_len()`: whether the
                // failure is a genuinely invalid sequence (`Some(n)`) or a
                // multi-byte sequence truncated by end-of-input (`None`),
                // per-byte Windows-1252 fallback treats both identically —
                // only `rest[valid_up_to]` is reinterpreted, one byte at a
                // time, regardless of how many bytes Rust's UTF-8 validator
                // groups into "the same" error. A truncated file is just
                // another byte that couldn't complete a valid codepoint.
                let valid_up_to = err.valid_up_to();
                if valid_up_to > 0 {
                    let valid_part = std::str::from_utf8(&rest[..valid_up_to])
                        .expect("prefix already validated by valid_up_to");
                    for ch in valid_part.chars() {
                        pos = pos.advance(ch);
                    }
                    out.push_str(valid_part);
                }

                let bad_byte = rest[valid_up_to];
                let resolved = windows1252_char(bad_byte);
                let ch = resolved.unwrap_or('\u{FFFD}');
                if resolved.is_none() {
                    diagnostics.push(Diagnostic::new(
                        DiagnosticKind::InvalidEncoding,
                        Span::at(pos),
                        "this byte is not valid UTF-8 and has no defined Windows-1252 \
                         interpretation either, so it was replaced with the Unicode \
                         replacement character",
                    ));
                }
                out.push(ch);
                pos = pos.advance(ch);

                rest = &rest[valid_up_to + 1..];
            }
        }
    }

    (out, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_ascii_round_trips_with_no_diagnostics() {
        let (text, diags) = decode_bytes(b"RUN PGM=MATRIX\nENDRUN\n");
        assert_eq!(text, "RUN PGM=MATRIX\nENDRUN\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn valid_multibyte_utf8_passes_through_untouched() {
        let src = "X = 'caf\u{00E9}'\n".as_bytes();
        let (text, diags) = decode_bytes(src);
        assert_eq!(text, "X = 'caf\u{00E9}'\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn defined_windows1252_byte_recovers_silently() {
        // 0x92 is Windows-1252's right single quotation mark, ’ (U+2019) —
        // the exact real-world byte found in T049's fixture corpus.
        let mut src = b"iteration".to_vec();
        src.push(0x92);
        src.extend_from_slice(b"s volume\n");
        let (text, diags) = decode_bytes(&src);
        assert_eq!(text, "iteration\u{2019}s volume\n");
        assert!(
            diags.is_empty(),
            "a resolvable byte should not produce a diagnostic"
        );
    }

    #[test]
    fn undefined_windows1252_byte_becomes_replacement_char_with_diagnostic() {
        // 0x81 has no defined Windows-1252 interpretation.
        let src = vec![b'X', b'=', 0x81, b'\n'];
        let (text, diags) = decode_bytes(&src);
        assert_eq!(text, "X=\u{FFFD}\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].kind, DiagnosticKind::InvalidEncoding);
    }

    #[test]
    fn position_after_substitution_is_char_based_not_byte_based() {
        // A multi-byte UTF-8 char before the bad byte must not desync the
        // reported column from a plain char count.
        let mut src = "\u{00E9}X=".as_bytes().to_vec(); // é (2 UTF-8 bytes, 1 char)
        src.push(0x81);
        let (_text, diags) = decode_bytes(&src);
        assert_eq!(diags.len(), 1);
        // Line 1: 'é'(col1) 'X'(col2) '='(col3) <bad byte>(col4)
        assert_eq!(diags[0].span.start.column, 4);
    }

    #[test]
    fn truncated_multibyte_sequence_at_eof_is_treated_like_any_other_invalid_byte() {
        // 0xE2 alone is a valid lead byte for a 3-byte UTF-8 sequence (e.g. the
        // start of '€', E2 82 AC) but the file ends right there with no
        // continuation bytes — `error_len()` would report `None` for this,
        // unlike the other tests' immediately-invalid bytes (`Some(1)`). Both
        // paths are exercised identically here: this lone lead byte falls in
        // Windows-1252's identity-mapped 0xA0..=0xFF range, so it recovers
        // silently as 'â' (U+00E2), the same as any other resolvable byte.
        let mut src = b"X".to_vec();
        src.push(0xE2);
        let (text, diags) = decode_bytes(&src);
        assert_eq!(text, "X\u{00E2}");
        assert!(
            diags.is_empty(),
            "a resolvable truncated byte should not produce a diagnostic"
        );
    }

    #[test]
    fn empty_input_produces_no_panic() {
        let (text, diags) = decode_bytes(&[]);
        assert_eq!(text, "");
        assert!(diags.is_empty());
    }
}
