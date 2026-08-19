//! Unit tests for `drut_config::resolve_format_options`
//! (012-toml-configuration T009; extended by
//! 017-casing-categories-indent-width tasks.md T015/T029).

use std::path::{Path, PathBuf};

use drut_config::{resolve_format_options, ExplicitFormatOverride};
use voyager_core::{BlankLineMode, CasingConvention, LineWrapMode, LineWrapStyle, OperatorSpacing, IndentTopLevelMode};

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
    let target = test_file(
        "explicit_wins",
        "[format]\ncasing_control_words = \"lower\"\ncasing_pair_keywords = \"lower\"\nindent_top_level = \"preserve\"\n",
    );

    let (options, warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride {
            casing_control_words: Some(CasingConvention::Upper),
            casing_pair_keywords: Some(CasingConvention::Upper),
            ..Default::default()
        },
        ExplicitFormatOverride::default(),
    );
    assert_eq!(options.casing.control_words, CasingConvention::Upper, "explicit override must win");
    assert_eq!(options.casing.pair_keywords, CasingConvention::Upper, "explicit override must win");
    assert_eq!(
        options.indent_top_level,
        IndentTopLevelMode::Preserve,
        "indent_top_level (no explicit value) must come from the file"
    );
    assert!(warnings.is_empty());

    cleanup(&target);
}

#[test]
fn config_file_value_wins_over_the_built_in_default_when_no_explicit_value_given() {
    let target = test_file(
        "file_wins_over_default",
        "[format]\ncasing_control_words = \"lower\"\ncasing_pair_keywords = \"lower\"\nindent_top_level = \"auto\"\n",
    );

    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.casing.control_words, CasingConvention::Lower);
    assert_eq!(options.casing.pair_keywords, CasingConvention::Lower);
    assert_eq!(options.indent_top_level, IndentTopLevelMode::Auto);

    cleanup(&target);
}

#[test]
fn no_file_path_skips_discovery_and_resolves_straight_to_explicit_then_default() {
    let (options, warnings) = resolve_format_options(
        None,
        false,
        ExplicitFormatOverride {
            casing_control_words: Some(CasingConvention::Upper),
            ..Default::default()
        },
        ExplicitFormatOverride::default(),
    );
    assert_eq!(options.casing.control_words, CasingConvention::Upper);
    assert_eq!(options.indent_top_level, IndentTopLevelMode::Preserve, "no file, no explicit -> built-in default");
    assert!(warnings.is_empty());
}

#[test]
fn isolated_skips_discovery_even_with_a_valid_nearby_config_present() {
    let target = test_file("isolated", "[format]\ncasing_control_words = \"lower\"\nindent_top_level = \"auto\"\n");

    let (options, warnings) = resolve_format_options(Some(&target), true, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(
        options.casing.control_words,
        CasingConvention::Preserve,
        "isolated must ignore the file entirely, casing stays at built-in default"
    );
    assert_eq!(options.indent_top_level, IndentTopLevelMode::Preserve, "isolated must ignore the file's indent_top_level too");
    assert!(warnings.is_empty(), "isolated must not even attempt discovery, so no warnings either");

    cleanup(&target);
}

#[test]
fn isolated_still_honors_an_explicit_override() {
    let target = test_file("isolated_with_explicit", "[format]\ncasing_control_words = \"lower\"\n");

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        true,
        ExplicitFormatOverride {
            casing_control_words: Some(CasingConvention::Upper),
            indent_top_level: Some(IndentTopLevelMode::Auto),
            ..Default::default()
        },
        ExplicitFormatOverride::default(),
    );
    assert_eq!(options.casing.control_words, CasingConvention::Upper);
    assert_eq!(options.indent_top_level, IndentTopLevelMode::Auto);

    cleanup(&target);
}

#[test]
fn a_present_config_file_that_does_not_mention_casing_resolves_to_preserve() {
    // 014-casing-preserve-mode FR-004: a drut.toml can set indent_top_level
    // without ever mentioning casing at all -- the unset field must still
    // resolve to CasingConvention::Preserve, not panic or leave a stale
    // value.
    let target = test_file("casing_unset_in_file", "[format]\nindent_top_level = \"auto\"\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.casing.control_words, CasingConvention::Preserve);
    assert_eq!(options.indent_top_level, IndentTopLevelMode::Auto);
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

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.casing.control_words, CasingConvention::Preserve);
    assert_eq!(options.indent_top_level, IndentTopLevelMode::Preserve);
    assert!(warnings.is_empty());

    cleanup(&target);
}

// -- 017-casing-categories-indent-width: per-category precedence (tasks.md T015) --

#[test]
fn a_removed_legacy_casing_key_in_the_file_governs_nothing() {
    // The flat `casing` field (which used to cover control_words +
    // pair_keywords together, never data_references) was removed --
    // present in a drut.toml, it's now just an unrecognized key, so every
    // category resolves to its own built-in default instead.
    let target = test_file("legacy_casing_removed", "[format]\ncasing = \"upper\"\n");

    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.casing.control_words, CasingConvention::Preserve);
    assert_eq!(options.casing.pair_keywords, CasingConvention::Preserve);
    assert_eq!(options.casing.data_references, CasingConvention::Preserve);

    cleanup(&target);
}

#[test]
fn all_three_granular_fields_set_independently_resolve_independently() {
    let target = test_file(
        "all_granular",
        "[format]\ncasing_control_words = \"upper\"\ncasing_pair_keywords = \"preserve\"\ncasing_data_references = \"lower\"\n",
    );

    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.casing.control_words, CasingConvention::Upper);
    assert_eq!(options.casing.pair_keywords, CasingConvention::Preserve);
    assert_eq!(options.casing.data_references, CasingConvention::Lower);

    cleanup(&target);
}

#[test]
fn casing_function_calls_resolves_independently_of_the_other_three_categories() {
    // 025-function-casing: the fourth granular field, same precedence shape.
    let target = test_file(
        "function_calls_granular",
        "[format]\ncasing_control_words = \"upper\"\ncasing_function_calls = \"lower\"\n",
    );

    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.casing.control_words, CasingConvention::Upper);
    assert_eq!(options.casing.function_calls, CasingConvention::Lower);
    assert_eq!(options.casing.pair_keywords, CasingConvention::Preserve, "unset fields still default to Preserve");

    cleanup(&target);
}

#[test]
fn explicit_casing_function_calls_override_wins_over_both_config_layers_for_its_own_category_only() {
    let target = test_file(
        "explicit_function_calls_wins",
        "[format]\ncasing_control_words = \"upper\"\ncasing_function_calls = \"upper\"\n",
    );

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride {
            casing_function_calls: Some(CasingConvention::Lower),
            ..Default::default()
        },
        ExplicitFormatOverride::default(),
    );
    assert_eq!(options.casing.control_words, CasingConvention::Upper, "unaffected by the explicit function_calls override");
    assert_eq!(options.casing.function_calls, CasingConvention::Lower, "explicit granular override wins over both file layers");

    cleanup(&target);
}

#[test]
fn explicit_granular_override_wins_over_both_config_layers_for_its_own_category_only() {
    let target = test_file(
        "explicit_granular_wins",
        "[format]\ncasing_control_words = \"upper\"\ncasing_data_references = \"upper\"\n",
    );

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride {
            casing_data_references: Some(CasingConvention::Lower),
            ..Default::default()
        },
        ExplicitFormatOverride::default(),
    );
    assert_eq!(options.casing.control_words, CasingConvention::Upper, "unaffected by the explicit data_references override");
    assert_eq!(options.casing.data_references, CasingConvention::Lower, "explicit granular override wins over both file layers");

    cleanup(&target);
}

// -- 017-casing-categories-indent-width: indent_width (tasks.md T029) --

#[test]
fn indent_width_parses_from_config_and_resolves() {
    let target = test_file("indent_width_ok", "[format]\nindent_width = 2\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.indent_width, 2);
    assert!(warnings.is_empty());

    cleanup(&target);
}

#[test]
fn indent_width_zero_falls_back_to_default_with_a_warning() {
    let target = test_file("indent_width_zero", "[format]\nindent_width = 0\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.indent_width, 4, "must fall back to the built-in default, never fail");
    assert_eq!(warnings.len(), 1);

    cleanup(&target);
}

#[test]
fn indent_width_unreasonably_large_falls_back_to_default_with_a_warning() {
    let target = test_file("indent_width_large", "[format]\nindent_width = 500\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
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
        ExplicitFormatOverride::default(),
    );
    assert_eq!(options.indent_width, 8);

    cleanup(&target);
}

#[test]
fn indent_width_unset_resolves_to_four() {
    let target = test_file("indent_width_unset", "[format]\ncasing_control_words = \"upper\"\n");

    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
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
        ExplicitFormatOverride::default(),
    );
    assert_eq!(options.operator_spacing, OperatorSpacing::Fixed);

    cleanup(&target);
}

#[test]
fn operator_spacing_unset_anywhere_resolves_to_preserve() {
    let target = test_file("operator_spacing_unset", "[format]\ncasing_control_words = \"upper\"\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.operator_spacing, OperatorSpacing::Preserve);
    assert!(warnings.is_empty());

    cleanup(&target);
}

#[test]
fn operator_spacing_parses_from_config_and_resolves() {
    let target = test_file("operator_spacing_from_config", "[format]\noperator_spacing = \"auto\"\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
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
        ExplicitFormatOverride::default(),
    );
    assert_eq!(options.blank_lines, BlankLineMode::Auto);

    cleanup(&target);
}

#[test]
fn blank_lines_and_both_caps_unset_anywhere_resolve_to_built_in_defaults() {
    let target = test_file("blank_lines_unset", "[format]\ncasing_control_words = \"upper\"\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.blank_lines, BlankLineMode::Preserve);
    assert_eq!(options.blank_lines_top_cap, 2);
    assert_eq!(options.blank_lines_nested_cap, 1);
    assert!(warnings.is_empty());

    cleanup(&target);
}

#[test]
fn blank_lines_top_cap_explicit_overrides_config_file() {
    let target = test_file("blank_lines_top_cap_explicit_override", "[format]\nblank_lines_top_cap = 5\n");

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride { blank_lines_top_cap: Some(3), ..Default::default() },
        ExplicitFormatOverride::default(),
    );
    assert_eq!(options.blank_lines_top_cap, 3);

    cleanup(&target);
}

#[test]
fn blank_lines_nested_cap_explicit_overrides_config_file() {
    let target = test_file("blank_lines_nested_cap_explicit_override", "[format]\nblank_lines_nested_cap = 5\n");

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride { blank_lines_nested_cap: Some(3), ..Default::default() },
        ExplicitFormatOverride::default(),
    );
    assert_eq!(options.blank_lines_nested_cap, 3);

    cleanup(&target);
}

#[test]
fn blank_lines_top_cap_out_of_range_in_config_falls_back_to_default_with_a_warning() {
    let target = test_file("blank_lines_top_cap_zero", "[format]\nblank_lines_top_cap = 0\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.blank_lines_top_cap, 2, "must fall back to the built-in default, never fail");
    assert_eq!(warnings.len(), 1);

    cleanup(&target);
}

#[test]
fn blank_lines_nested_cap_unreasonably_large_falls_back_to_default_with_a_warning() {
    let target = test_file("blank_lines_nested_cap_large", "[format]\nblank_lines_nested_cap = 200\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.blank_lines_nested_cap, 1, "must fall back to the built-in default, never fail");
    assert_eq!(warnings.len(), 1);

    cleanup(&target);
}

#[test]
fn line_wrap_explicit_overrides_config_file() {
    let target = test_file("line_wrap_explicit_override", "[format]\nline_wrap = \"preserve\"\n");

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride { line_wrap: Some(LineWrapMode::Auto), ..Default::default() },
        ExplicitFormatOverride::default(),
    );
    assert_eq!(options.line_wrap, LineWrapMode::Auto);

    cleanup(&target);
}

#[test]
fn line_wrap_width_and_style_unset_anywhere_resolve_to_built_in_defaults() {
    let target = test_file("line_wrap_unset", "[format]\ncasing_control_words = \"upper\"\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.line_wrap, LineWrapMode::Preserve);
    assert_eq!(options.line_wrap_width, 120);
    assert_eq!(options.line_wrap_style, LineWrapStyle::Fill);
    assert!(warnings.is_empty());

    cleanup(&target);
}

#[test]
fn line_wrap_width_explicit_overrides_config_file() {
    let target = test_file("line_wrap_width_explicit_override", "[format]\nline_wrap_width = 200\n");

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride { line_wrap_width: Some(80), ..Default::default() },
        ExplicitFormatOverride::default(),
    );
    assert_eq!(options.line_wrap_width, 80);

    cleanup(&target);
}

#[test]
fn line_wrap_style_explicit_overrides_config_file() {
    let target = test_file("line_wrap_style_explicit_override", "[format]\nline_wrap_style = \"fill\"\n");

    let (options, _warnings) = resolve_format_options(
        Some(&target),
        false,
        ExplicitFormatOverride { line_wrap_style: Some(LineWrapStyle::OnePerLine), ..Default::default() },
        ExplicitFormatOverride::default(),
    );
    assert_eq!(options.line_wrap_style, LineWrapStyle::OnePerLine);

    cleanup(&target);
}

#[test]
fn line_wrap_width_out_of_range_in_config_falls_back_to_default_with_a_warning() {
    let target = test_file("line_wrap_width_zero", "[format]\nline_wrap_width = 5\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.line_wrap_width, 120, "must fall back to the built-in default, never fail");
    assert_eq!(warnings.len(), 1);

    cleanup(&target);
}

#[test]
fn line_wrap_width_unreasonably_large_falls_back_to_default_with_a_warning() {
    let target = test_file("line_wrap_width_large", "[format]\nline_wrap_width = 5000\n");

    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), ExplicitFormatOverride::default());
    assert_eq!(options.line_wrap_width, 120, "must fall back to the built-in default, never fail");
    assert_eq!(warnings.len(), 1);

    cleanup(&target);
}
