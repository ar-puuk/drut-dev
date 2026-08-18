//! End-to-end integration tests for blank-line-run normalization
//! (019-blank-line-normalization tasks.md T018, T020) — exercises spec.md's
//! US1/US2 Acceptance Scenarios directly via `format()`, on real-corpus-
//! shaped fixtures (not just the lower-level `blank_line.rs` unit tests),
//! mirroring `operator_spacing_integration.rs`'s own established shape.

use voyager_core::{format, BlankLineMode, FormatOptions};

fn auto() -> FormatOptions {
    FormatOptions { blank_lines: BlankLineMode::Auto, ..FormatOptions::default() }
}

fn auto_with_caps(top_level_cap: u8, nested_cap: u8) -> FormatOptions {
    FormatOptions {
        blank_lines: BlankLineMode::Auto,
        blank_lines_top_cap: top_level_cap,
        blank_lines_nested_cap: nested_cap,
        ..FormatOptions::default()
    }
}

// -- User Story 1 (top-level cap) -----------------------------------------

#[test]
fn us1_acceptance_scenario_1_a_run_of_five_between_top_level_blocks_contracts_to_the_default_cap_of_two() {
    // spec.md US1 AS1: a real-corpus-shaped pair of top-level RUN blocks
    // separated by an excessive blank-line run.
    let src = "RUN PGM=NETWORK\n    NETI = base.net\n    NETO = out.net\nENDRUN\n\n\n\n\n\nRUN PGM=MATRIX\n    ZONES = 5\nENDRUN\n";
    let out = format(src, auto()).text;
    assert_eq!(
        out,
        "RUN PGM=NETWORK\n    NETI = base.net\n    NETO = out.net\nENDRUN\n\n\nRUN PGM=MATRIX\n    ZONES = 5\nENDRUN\n",
        "exactly 2 blank lines must remain between the two top-level RUN blocks: {out}"
    );
}

#[test]
fn us1_acceptance_scenario_2_a_run_of_two_or_fewer_at_top_level_is_left_completely_untouched() {
    // spec.md US1 AS2 -- same file, a second, already-in-range run elsewhere
    // must be left untouched even while the excessive one above contracts.
    let src = "RUN PGM=NETWORK\nENDRUN\n\n\n\n\n\nRUN PGM=MATRIX\nENDRUN\n\nRUN PGM=HWYASSIGN\nENDRUN\n";
    let out = format(src, auto()).text;
    assert_eq!(
        out,
        "RUN PGM=NETWORK\nENDRUN\n\n\nRUN PGM=MATRIX\nENDRUN\n\nRUN PGM=HWYASSIGN\nENDRUN\n",
        "the single blank line between MATRIX and HWYASSIGN (already at/under cap) must survive untouched: {out}"
    );
}

#[test]
fn us1_acceptance_scenario_3_no_blank_lines_configuration_at_all_leaves_the_script_byte_identical() {
    // spec.md US1 AS3 -- FormatOptions::default() (Preserve, unconfigured)
    // must be a true no-op on however-long a blank-line run.
    let src = "RUN PGM=NETWORK\nENDRUN\n\n\n\n\n\n\n\n\n\nRUN PGM=MATRIX\nENDRUN\n";
    let result = format(src, FormatOptions::default());
    assert!(!result.changed, "no blank_lines configuration must be a true no-op, however long the run");
    assert_eq!(result.text, src);
}

// -- User Story 2 (nested cap) --------------------------------------------

#[test]
fn us2_acceptance_scenario_1_a_run_of_four_inside_a_blocks_body_contracts_to_the_default_cap_of_one() {
    // spec.md US2 AS1: a real-corpus-shaped RUN block whose body has an
    // excessive blank-line run between two child statements.
    let src = "RUN PGM=HWYASSIGN\n    FILEI NETI = base.net\n\n\n\n\n    FILEO NETO = out.net\nENDRUN\n";
    let out = format(src, auto()).text;
    assert_eq!(
        out,
        "RUN PGM=HWYASSIGN\n    FILEI NETI = base.net\n\n    FILEO NETO = out.net\nENDRUN\n",
        "exactly 1 blank line must remain inside the RUN block's own body: {out}"
    );
}

#[test]
fn us2_acceptance_scenario_2_a_doubly_nested_blocks_excessive_run_gets_the_same_nested_cap() {
    // spec.md US2 AS2: a LOOP nested inside a RUN, with its own excessive
    // blank-line run -- must get the SAME nested cap as a singly-nested
    // run, not a further-reduced one.
    let src = "RUN PGM=MATRIX\n    LOOP i = 1, 5\n        X = 1\n\n\n\n\n        Y = 2\n    ENDLOOP\nENDRUN\n";
    let out = format(src, auto()).text;
    assert_eq!(
        out,
        "RUN PGM=MATRIX\n    LOOP i = 1, 5\n        X = 1\n\n        Y = 2\n    ENDLOOP\nENDRUN\n",
        "the doubly-nested run must contract to the SAME nested cap (1), not a smaller one: {out}"
    );
}

#[test]
fn us2_acceptance_scenario_3_top_level_and_nested_excessive_runs_each_contract_independently() {
    // spec.md US2 AS3: distinct caps (top-level 3, nested 1) applied to a
    // single file with both an excessive top-level run and an excessive
    // nested run -- each contracts to its own applicable cap independently.
    let src = "RUN PGM=NETWORK\n    NETI = base.net\n\n\n\n\n    NETO = out.net\nENDRUN\n\n\n\n\n\n\nRUN PGM=MATRIX\nENDRUN\n";
    let out = format(src, auto_with_caps(3, 1)).text;
    assert_eq!(
        out,
        "RUN PGM=NETWORK\n    NETI = base.net\n\n    NETO = out.net\nENDRUN\n\n\n\nRUN PGM=MATRIX\nENDRUN\n",
        "the nested run (len 4) must contract to nested cap 1, and the top-level run (len 6) to top-level cap 3, independently: {out}"
    );
}
