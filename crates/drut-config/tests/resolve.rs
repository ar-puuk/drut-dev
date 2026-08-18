//! Unit tests for `drut_config::resolve_format_options`
//! (012-toml-configuration T009; extended by
//! 017-casing-categories-indent-width tasks.md T015/T029).

use std::path::{Path, PathBuf};

use drut_config::{resolve_format_options, ExplicitFormatOverride};
use voyager_core::{BlankLineMode, CasingConvention, OperatorSpacing, TopLevelIndentMode};

fn test_file(name: &str, config: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("drut_config_resolve_test_{}_{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("drut.toml"), config).unwrap();
    let target = dir.join("a.s");
    std::fs::write(&target, "IF (a=b)\nENDIF\n").unwrap();
    target
}

fn cleanup(target: &Path) {
    let _ = std::fs::remove_dir_all(target.parent().unwrap());
}

#[test]
fn explicit_value_wins_over_a_present_valid_config_file_per_field() {
    let target = test_file("explicit_wins", "[format]\ncasing = \"lower\"\ntop_level_indent = \"preserve\"\n");

    let (options, warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride {
            casing: Some(CasingConvention::Upper),
            ..Default::default()
        },
    );
    assert_eq!(options.casing.control_words, CasingConvention::Upper, "explicit casing must win");
    assert_eq!(options.casing.pair_keywords, CasingConvention::Upper, "explicit casing must win");
    assert_eq!(
        options.top_level_indent,
        TopLevelIndentMode::Preserve,
        "top_level_indent (no explicit value) must come from the file"
    );
    assert!(warnings.is_empty());

    cleanup(&target);
}

#[test]
fn config_file_value_wins_over_the_built_in_default_when_no_explicit_value_given() {
    let target = test_file("file_wins_over_default", "[format]\ncasing = \"lower\"\ntop_level_indent = \"normalize\"\n");

    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.casing.control_words, CasingConvention::Lower);
    assert_eq!(options.casing.pair_keywords, CasingConvention::Lower);
    assert_eq!(options.top_level_indent, TopLevelIndentMode::Normalize);

    cleanup(&target);
}

#[test]
fn no_file_path_skips_discovery_and_resolves_straight_to_explicit_then_default() {
    let (options, warnings) = resolve_format_options(
        None,
        false,
        ExplicitFormatOverride {
            casing: Some(CasingConvention::Upper),
            ..Default::default()
        },
    );
    assert_eq!(options.casing.control_words, CasingConvention::Upper);
    assert_eq!(options.top_level_indent, TopLevelIndentMode::Preserve, "no file, no explicit -> built-in default");
    assert!(warnings.is_empty());
}

#[test]
fn isolated_skips_discovery_even_with_a_valid_nearby_config_present() {
    let target = test_file("isolated", "[format]\ncasing = \"lower\"\ntop_level_indent = \"normalize\"\n");

    let (options, warnings) = resolve_format_options(Some(&target), true, ExplicitFormatOverride::default());
    assert_eq!(
        options.casing.control_words,
        CasingConvention::Preserve,
        "isolated must ignore the file entirely, casing stays at built-in default"
    );
    assert_eq!(options.top_level_indent, TopLevelIndentMode::Preserve, "isolated must ignore the file's top_level_indent too");
    assert!(warnings.is_empty(), "isolated must not even attempt discovery, so no warnings either");

    cleanup(&target);
}

#[test]
fn isolated_still_honors_an_explicit_override() {
    let target = test_file("isolated_with_explicit", "[format]\ncasing = \"lower\"\n");

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        true,
        ExplicitFormatOverride {
            casing: Some(CasingConvention::Upper),
            top_level_indent: Some(TopLevelIndentMode::Normalize),
            ..Default::default()
        },
    );
    assert_eq!(options.casing.control_words, CasingConvention::Upper);
    assert_eq!(options.top_level_indent, TopLevelIndentMode::Normalize);

    cleanup(&target);
}

#[test]
fn a_present_config_file_that_does_not_mention_casing_resolves_to_preserve() {
    // 014-casing-preserve-mode FR-004: a drut.toml can set top_level_indent
    // without ever mentioning casing at all -- the unset field must still
    // resolve to CasingConvention::Preserve, not panic or leave a stale
    // value.
    let target = test_file("casing_unset_in_file", "[format]\ntop_level_indent = \"normalize\"\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.casing.control_words, CasingConvention::Preserve);
    assert_eq!(options.top_level_indent, TopLevelIndentMode::Normalize);
    assert!(warnings.is_empty());

    cleanup(&target);
}

#[test]
fn a_missing_config_file_resolves_to_explicit_then_default_with_no_warnings() {
    let dir = std::env::temp_dir().join(format!("drut_config_resolve_test_{}_no_config", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    let target = dir.join("a.s");
    std::fs::write(&target, "IF (a=b)\nENDIF\n").unwrap();

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.casing.control_words, CasingConvention::Preserve);
    assert_eq!(options.top_level_indent, TopLevelIndentMode::Preserve);
    assert!(warnings.is_empty());

    cleanup(&target);
}

// -- 017-casing-categories-indent-width: per-category precedence (tasks.md T015) --

#[test]
fn legacy_casing_alone_covers_control_words_and_pair_keywords_but_never_data_references() {
    let target = test_file("legacy_only", "[format]\ncasing = \"upper\"\n");

    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.casing.control_words, CasingConvention::Upper);
    assert_eq!(options.casing.pair_keywords, CasingConvention::Upper);
    assert_eq!(
        options.casing.data_references,
        CasingConvention::Preserve,
        "legacy casing never reached data_references before this feature and still doesn't"
    );

    cleanup(&target);
}

#[test]
fn granular_data_references_field_governs_data_references_while_legacy_governs_the_other_two() {
    let target = test_file(
        "legacy_plus_granular",
        "[format]\ncasing = \"upper\"\ndata_references_casing = \"lower\"\n",
    );

    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.casing.control_words, CasingConvention::Upper, "still governed by legacy casing");
    assert_eq!(options.casing.pair_keywords, CasingConvention::Upper, "still governed by legacy casing");
    assert_eq!(options.casing.data_references, CasingConvention::Lower, "granular field, not legacy, governs this one");

    cleanup(&target);
}

#[test]
fn all_three_granular_fields_set_independently_resolve_independently() {
    let target = test_file(
        "all_granular",
        "[format]\ncontrol_words_casing = \"upper\"\npair_keywords_casing = \"preserve\"\ndata_references_casing = \"lower\"\n",
    );

    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.casing.control_words, CasingConvention::Upper);
    assert_eq!(options.casing.pair_keywords, CasingConvention::Preserve);
    assert_eq!(options.casing.data_references, CasingConvention::Lower);

    cleanup(&target);
}

#[test]
fn explicit_granular_override_wins_over_both_config_layers_for_its_own_category_only() {
    let target = test_file(
        "explicit_granular_wins",
        "[format]\ncasing = \"upper\"\ndata_references_casing = \"upper\"\n",
    );

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride {
            data_references_casing: Some(CasingConvention::Lower),
            ..Default::default()
        },
    );
    assert_eq!(options.casing.control_words, CasingConvention::Upper, "unaffected by the explicit data_references override");
    assert_eq!(options.casing.data_references, CasingConvention::Lower, "explicit granular override wins over both file layers");

    cleanup(&target);
}

// -- 017-casing-categories-indent-width: indent_width (tasks.md T029) --

#[test]
fn indent_width_parses_from_config_and_resolves() {
    let target = test_file("indent_width_ok", "[format]\nindent_width = 2\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.indent_width, 2);
    assert!(warnings.is_empty());

    cleanup(&target);
}

#[test]
fn indent_width_zero_falls_back_to_default_with_a_warning() {
    let target = test_file("indent_width_zero", "[format]\nindent_width = 0\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.indent_width, 4, "must fall back to the built-in default, never fail");
    assert_eq!(warnings.len(), 1);

    cleanup(&target);
}

#[test]
fn indent_width_unreasonably_large_falls_back_to_default_with_a_warning() {
    let target = test_file("indent_width_large", "[format]\nindent_width = 500\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.indent_width, 4);
    assert_eq!(warnings.len(), 1);

    cleanup(&target);
}

#[test]
fn explicit_indent_width_overrides_config_file() {
    let target = test_file("indent_width_explicit_override", "[format]\nindent_width = 2\n");

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride { indent_width: Some(8), ..Default::default() },
    );
    assert_eq!(options.indent_width, 8);

    cleanup(&target);
}

#[test]
fn indent_width_unset_resolves_to_four() {
    let target = test_file("indent_width_unset", "[format]\ncasing = \"upper\"\n");

    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.indent_width, 4);

    cleanup(&target);
}

// -- 018-operator-spacing (tasks.md T016) --

#[test]
fn operator_spacing_explicit_overrides_config_file() {
    let target = test_file("operator_spacing_explicit_override", "[format]\noperator_spacing = \"preserve\"\n");

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride { operator_spacing: Some(OperatorSpacing::Fixed), ..Default::default() },
    );
    assert_eq!(options.operator_spacing, OperatorSpacing::Fixed);

    cleanup(&target);
}

#[test]
fn operator_spacing_unset_anywhere_resolves_to_preserve() {
    let target = test_file("operator_spacing_unset", "[format]\ncasing = \"upper\"\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.operator_spacing, OperatorSpacing::Preserve);
    assert!(warnings.is_empty());

    cleanup(&target);
}

#[test]
fn operator_spacing_parses_from_config_and_resolves() {
    let target = test_file("operator_spacing_from_config", "[format]\noperator_spacing = \"auto\"\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.operator_spacing, OperatorSpacing::Auto);
    assert!(warnings.is_empty());

    cleanup(&target);
}

// -- 019-blank-line-normalization (tasks.md T015) --

#[test]
fn blank_lines_explicit_overrides_config_file() {
    let target = test_file("blank_lines_explicit_override", "[format]\nblank_lines = \"preserve\"\n");

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride { blank_lines: Some(BlankLineMode::Auto), ..Default::default() },
    );
    assert_eq!(options.blank_lines, BlankLineMode::Auto);

    cleanup(&target);
}

#[test]
fn blank_lines_and_both_caps_unset_anywhere_resolve_to_built_in_defaults() {
    let target = test_file("blank_lines_unset", "[format]\ncasing = \"upper\"\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.blank_lines, BlankLineMode::Preserve);
    assert_eq!(options.top_level_blank_line_cap, 2);
    assert_eq!(options.nested_blank_line_cap, 1);
    assert!(warnings.is_empty());

    cleanup(&target);
}

#[test]
fn top_level_blank_line_cap_explicit_overrides_config_file() {
    let target = test_file("top_level_blank_line_cap_explicit_override", "[format]\ntop_level_blank_line_cap = 5\n");

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride { top_level_blank_line_cap: Some(3), ..Default::default() },
    );
    assert_eq!(options.top_level_blank_line_cap, 3);

    cleanup(&target);
}

#[test]
fn nested_blank_line_cap_explicit_overrides_config_file() {
    let target = test_file("nested_blank_line_cap_explicit_override", "[format]\nnested_blank_line_cap = 5\n");

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride { nested_blank_line_cap: Some(3), ..Default::default() },
    );
    assert_eq!(options.nested_blank_line_cap, 3);

    cleanup(&target);
}

#[test]
fn top_level_blank_line_cap_out_of_range_in_config_falls_back_to_default_with_a_warning() {
    let target = test_file("top_level_blank_line_cap_zero", "[format]\ntop_level_blank_line_cap = 0\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.top_level_blank_line_cap, 2, "must fall back to the built-in default, never fail");
    assert_eq!(warnings.len(), 1);

    cleanup(&target);
}

#[test]
fn nested_blank_line_cap_unreasonably_large_falls_back_to_default_with_a_warning() {
    let target = test_file("nested_blank_line_cap_large", "[format]\nnested_blank_line_cap = 200\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default());
    assert_eq!(options.nested_blank_line_cap, 1, "must fall back to the built-in default, never fail");
    assert_eq!(warnings.len(), 1);

    cleanup(&target);
}
