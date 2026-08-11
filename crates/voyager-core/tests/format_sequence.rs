//! Format-then-structural-edit-then-format-again sequence tests
//! (007-formatter-diagnosed-block-indent-fix).
//!
//! `format_corpus.rs`'s existing golden/idempotency/structure-preservation
//! tests are all single-shot: one fixture, one `format` call, compared
//! either to itself repeated or to a static committed golden file. None of
//! them ever apply a structural edit *between* two format calls — so a bug
//! that only manifests across a sequence of edits (format, edit the source
//! in a way that changes block boundaries, format again) had zero coverage
//! before this file. That gap is why the residue bug this file's tests
//! guard against shipped undetected: `format(x)` on the buggy output was
//! already a stable no-op (`changed: false`), so `format(format(x)) ==
//! format(x)` — the existing idempotency check — held trivially. Idempotence
//! proves *stability* of whatever fixed point a formatter settles into,
//! never the *correctness* of that fixed point. These tests check the
//! fixed point itself against an independently-derived correct expectation
//! after a real structural edit, which is the piece that was missing.

use voyager_core::{format, FormatOptions};

/// The exact real-world sequence that surfaced the bug: a `PROCESS
/// PHASE=INPUT` left unclosed swallows a trailing `RUN PGM=HWYASSIGN` as
/// its own child (correct, given the genuinely broken structure at that
/// point — `UnmatchedProcess` is diagnosed). Format once: with
/// `007-formatter-diagnosed-block-indent-fix` in place, `RUN` is *not*
/// speculatively reindented (the diagnosed subtree is left untouched, not
/// confidently nested) — `changed` is `false`. Add `ENDPROCESS`, the
/// realistic fix a user types, closing `PROCESS` correctly and revealing
/// `RUN` as a genuine top-level sibling. Format again: `RUN` must already
/// be correctly indented, because it was never touched by the first pass.
#[test]
fn process_run_residue_is_fixed_after_endprocess_is_added() {
    let step1 = "PROCESS PHASE=INPUT\n    FILEI = ni.1\n    LOOP DAY = 1, 5\n        PRINT LIST='Day = ', DAY\n    ENDLOOP\n\nRUN PGM=HWYASSIGN\n    FILEI NETI = 'net.net'\nENDRUN\n";

    let pass1 = format(step1, FormatOptions::default());
    assert!(
        !pass1.changed,
        "pass 1 (PROCESS still unclosed) must leave RUN untouched, not speculatively reindent it: {:?}",
        pass1.text
    );
    assert_eq!(
        pass1.text, step1,
        "pass 1 must be a byte-for-byte no-op while PROCESS is genuinely unmatched"
    );

    let step2 = pass1.text.replacen("    ENDLOOP\n\n", "    ENDLOOP\nENDPROCESS\n\n", 1);
    let pass2 = format(&step2, FormatOptions::default());

    let expected = "PROCESS PHASE=INPUT\n    FILEI = ni.1\n    LOOP DAY = 1, 5\n        PRINT LIST='Day = ', DAY\n    ENDLOOP\nENDPROCESS\n\nRUN PGM=HWYASSIGN\n    FILEI NETI = 'net.net'\nENDRUN\n";
    assert_eq!(
        pass2.text, expected,
        "pass 2 (PROCESS now closed) must leave RUN correctly at top level, not stuck at a stale nested indent"
    );
}

/// The mirrored case for `RUN`/`IF`, proving the fix is general — keyed off
/// "this block has a diagnostic," not special-cased to `Process`. An
/// unclosed `RUN` swallows a trailing `IF` as its child; format once
/// (`IF` untouched, `changed: false`); add `ENDRUN`; format again (`IF`
/// correctly resolves to top level).
#[test]
fn run_if_residue_is_fixed_after_endrun_is_added() {
    let step1 = "RUN PGM=MATRIX\n    X = 1\n\nIF (a=b)\n    Y = 2\nENDIF\n";

    let pass1 = format(step1, FormatOptions::default());
    assert!(
        !pass1.changed,
        "pass 1 (RUN still unclosed) must leave IF untouched, not speculatively reindent it: {:?}",
        pass1.text
    );
    assert_eq!(pass1.text, step1);

    let step2 = pass1.text.replacen("    X = 1\n\n", "    X = 1\nENDRUN\n\n", 1);
    let pass2 = format(&step2, FormatOptions::default());

    let expected = "RUN PGM=MATRIX\n    X = 1\nENDRUN\n\nIF (a=b)\n    Y = 2\nENDIF\n";
    assert_eq!(
        pass2.text, expected,
        "pass 2 (RUN now closed) must leave IF correctly at top level, not stuck at a stale nested indent"
    );
}

/// Unlike `process_run_residue_is_fixed_after_endprocess_is_added` above
/// (whose `RUN` was already correctly positioned once revealed at top
/// level, because `007`'s own no-speculative-write behavior never touched
/// it), this covers the harder shape `007` alone never corrected: `RUN`
/// left at *stale*, non-zero indentation after `ENDPROCESS` is added —
/// the shape `008-top-level-indentation-normalization`'s unconditional
/// top-level rule fixes directly, in the same single pass.
#[test]
fn process_run_residue_with_stale_run_indentation_resolves_in_one_pass() {
    let step2 = "PROCESS PHASE=INPUT\n    FILEI = ni.1\n    LOOP DAY = 1, 5\n        PRINT LIST='Day = ', DAY\n    ENDLOOP\nENDPROCESS\n\n    RUN PGM=HWYASSIGN\n        FILEI NETI = 'net.net'\n    ENDRUN\n";
    let pass2 = format(step2, FormatOptions::default());

    let expected = "PROCESS PHASE=INPUT\n    FILEI = ni.1\n    LOOP DAY = 1, 5\n        PRINT LIST='Day = ', DAY\n    ENDLOOP\nENDPROCESS\n\nRUN PGM=HWYASSIGN\n    FILEI NETI = 'net.net'\nENDRUN\n";
    assert!(
        pass2.changed,
        "a RUN block left at stale non-zero indentation after ENDPROCESS is added must be corrected, not left as residue: {:?}",
        pass2.text
    );
    assert_eq!(
        pass2.text, expected,
        "stale RUN/FILEI NETI/ENDRUN indentation must fully resolve in the single pass that reveals RUN as top-level"
    );
}

/// Confirms the fix doesn't overcorrect: a still-broken file (no closer
/// ever added) is left exactly as the author wrote it for the diagnosed
/// block's own children — not reindented in some *other*, new way either.
/// Same shape as `process_run_residue_is_fixed_after_endprocess_is_added`'s
/// own pass-1 assertion, kept here as its own dedicated test per this
/// fix's own explicit regression-test requirement (distinct from being an
/// incidental assertion inside a two-pass test).
#[test]
fn still_broken_process_leaves_swallowed_content_untouched_not_overcorrected() {
    let source = "PROCESS PHASE=INPUT\n    FILEI = ni.1\n    LOOP DAY = 1, 5\n        PRINT LIST='Day = ', DAY\n    ENDLOOP\n\nRUN PGM=HWYASSIGN\n    FILEI NETI = 'net.net'\nENDRUN\n";
    let result = format(source, FormatOptions::default());

    assert!(
        !result.changed,
        "a still-broken (never closed) PROCESS must never touch its diagnosed subtree's indentation, got: {:?}",
        result.text
    );
    assert_eq!(result.text, source);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].kind, voyager_core::DiagnosticKind::UnmatchedProcess);
}
