//! End-to-end integration tests for operator spacing normalization
//! (018-operator-spacing tasks.md T019, T026) — exercises spec.md's US1/US2
//! Acceptance Scenarios directly via `format()`, on real-corpus-shaped
//! fixtures (not just the lower-level `operator_spacing.rs` unit tests).

use voyager_core::format::{format, CasingSettings, FormatOptions, OperatorSpacing, TopLevelIndentMode};

fn fixed() -> FormatOptions {
    FormatOptions { operator_spacing: OperatorSpacing::Fixed, ..FormatOptions::default() }
}

fn auto() -> FormatOptions {
    FormatOptions { operator_spacing: OperatorSpacing::Auto, ..FormatOptions::default() }
}

// -- User Story 1 (Fixed) -------------------------------------------------

#[test]
fn us1_acceptance_scenario_1_every_shape_normalizes_in_one_pass() {
    // spec.md US1 AS1: ZONES spacing, multi-pair comma spacing, IF/paren
    // interior padding + control-word-paren adjacency, and an assignment
    // with bracket padding + an arithmetic expression, all in one file --
    // a real corpus shape (a RUN block with mixed pair/assignment content).
    let src = "\
RUN PGM=MATRIX
ZONES   = 1
FILEI MATI=a.mat,MATO=b.mat
IF ( x==1 )
MW[ 1 ]=mi.1.1+mi.2.1
ENDIF
ENDRUN
";
    let out = format(src, fixed()).text;
    assert_eq!(
        out,
        "\
RUN PGM = MATRIX
    ZONES = 1
    FILEI MATI = a.mat, MATO = b.mat
    IF(x == 1)
        MW[1] = mi.1.1 + mi.2.1
    ENDIF
ENDRUN
"
    );
}

#[test]
fn us1_acceptance_scenario_2_unconfigured_is_byte_identical() {
    // spec.md US1 AS2: no operator_spacing configuration -> zero change,
    // even though the fixture is deliberately messy.
    let src = "ZONES   = 1\nFILEI MATI=a.mat,MATO=b.mat\nMW[ 1 ]=mi.1.1+mi.2.1\n";
    let result = format(src, FormatOptions::default());
    assert!(!result.changed);
    assert_eq!(result.text, src);
}

#[test]
fn us1_acceptance_scenario_3_unary_minus_stays_tight() {
    // spec.md US1 AS3: a negative literal's sign is never spaced apart from
    // its operand, unlike a genuine binary operator.
    let src = "MW[1] = -5\n";
    let out = format(src, fixed()).text;
    assert_eq!(out, "MW[1] = -5\n");
}

#[test]
fn us1_quoted_literal_content_is_never_touched() {
    // FR-010a/research.md §9's own verified finding, re-proven end-to-end
    // through format() rather than just operator_spacing.rs's own units.
    let src = "PRINT LIST='a+b=c', FILE='x.txt'\n";
    let out = format(src, fixed()).text;
    assert_eq!(out, "PRINT LIST = 'a+b=c', FILE = 'x.txt'\n", "content inside the quotes must be byte-identical");
}

// -- User Story 2 (Auto) ---------------------------------------------------

#[test]
fn us2_acceptance_scenario_1_consecutive_assignments_align_to_the_longest() {
    let src = "A = 1\nBB = 2\nCCC = 3\n";
    let out = format(src, auto()).text;
    assert_eq!(out, "A   = 1\nBB  = 2\nCCC = 3\n");
}

#[test]
fn us2_acceptance_scenario_2_blank_comment_and_depth_change_each_reset_alignment() {
    let src = "\
A = 1
BB = 2

CCC = 3
; a comment
DDDD = 4
IF (X==1)
E = 5
FF = 6
ENDIF
";
    let out = format(src, auto()).text;
    assert_eq!(
        out,
        "\
A  = 1
BB = 2

CCC = 3
; a comment
DDDD = 4
IF(X == 1)
    E  = 5
    FF = 6
ENDIF
"
    );
}

#[test]
fn us2_acceptance_scenario_3_a_control_statement_splits_the_run() {
    let src = "A = 1\nPHASE=ILOOP\nCCC = 3\n";
    let out = format(src, auto()).text;
    // PHASE's own = is spaced per Fixed only; A and CCC are each their own
    // one-member run (no alignment padding needed).
    assert_eq!(out, "A = 1\nPHASE = ILOOP\nCCC = 3\n");
}

#[test]
fn us2_acceptance_scenario_4_a_lone_assignment_matches_fixed_alone() {
    let src = "A = 1\n";
    let out = format(src, auto()).text;
    assert_eq!(out, "A = 1\n");
}

#[test]
fn auto_never_diverges_from_fixed_on_a_two_member_run() {
    // contracts/operator-spacing.md: Auto is Fixed plus alignment, never a
    // different base spacing decision.
    let src = "MW[ 1 ]=mi.1.1+mi.2.1\nMWW[1]=mi.2.1\n";
    let fixed_out = format(src, fixed()).text;
    let auto_out = format(src, auto()).text;
    assert_eq!(
        fixed_out,
        "MW[1] = mi.1.1 + mi.2.1\nMWW[1] = mi.2.1\n",
        "sanity check on Fixed's own base spacing"
    );
    // Auto adds alignment padding on top -- MW[1] (5 chars) is longer than
    // MWW[1] wait, "MW[1]" is 5 chars and "MWW[1]" is 6 -- MWW[1] is
    // longer, so MW[1]'s = gets one extra space of padding.
    assert_eq!(auto_out, "MW[1]  = mi.1.1 + mi.2.1\nMWW[1] = mi.2.1\n");
}

// -- Cross-cutting: unconfigured full-corpus-style file is unaffected ------

#[test]
fn casing_indentation_and_operator_spacing_all_apply_independently_together() {
    let src = "if (x=1)\nzones   = 1\nendif\n";
    let options = FormatOptions {
        casing: CasingSettings { control_words: voyager_core::CasingConvention::Upper, ..CasingSettings::default() },
        top_level_indent: TopLevelIndentMode::default(),
        indent_width: 4,
        operator_spacing: OperatorSpacing::Fixed,
        ..FormatOptions::default()
    };
    let out = format(src, options).text;
    assert_eq!(
        out,
        "IF(x = 1)\n    zones = 1\nENDIF\n",
        "control-word casing (if/endif -> IF/ENDIF), body indentation (zones -> 4 spaces), \
         and operator spacing (IF's own control-word-paren adjacency + both = signs normalized) \
         all apply together, none interfering with another"
    );
}
