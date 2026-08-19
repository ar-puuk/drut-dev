//! Unit tests for `drut_config::parse::parse` (012-toml-configuration T008).

use std::path::{Path, PathBuf};

use drut_config::{parse::parse, ConfigWarning};
use voyager_core::{BlankLineMode, CasingConvention, LineWrapMode, LineWrapStyle, OperatorSpacing, IndentTopLevelMode};

fn write_config(name: &str, content: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("drut_config_parse_test_{}_{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("drut.toml");
    std::fs::write(&path, content).unwrap();
    path
}

fn cleanup(path: &Path) {
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn fully_valid_file_parses_both_keys_with_zero_warnings() {
    let path = write_config("valid", "[format]\ncasing_control_words = \"upper\"\nindent_top_level = \"auto\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing_control_words, Some(CasingConvention::Upper));
    assert_eq!(config.format.indent_top_level, Some(IndentTopLevelMode::Auto));
    assert!(warnings.is_empty(), "expected zero warnings, got {warnings:?}");

    cleanup(&path);
}

#[test]
fn casing_preserve_parses_cleanly_with_zero_warnings() {
    // 014-casing-preserve-mode FR-005/SC-005: "preserve" is a recognized
    // value, not an unrecognized one that happens to be a no-op.
    let path = write_config("casing_preserve", "[format]\ncasing_control_words = \"preserve\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing_control_words, Some(CasingConvention::Preserve));
    assert!(warnings.is_empty(), "expected zero warnings, got {warnings:?}");

    cleanup(&path);
}

#[test]
fn invalid_value_for_one_key_falls_back_only_for_that_key() {
    let path = write_config(
        "invalid_value",
        "[format]\ncasing_control_words = \"sideways\"\nindent_top_level = \"auto\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(
        config.format.casing_control_words, None,
        "invalid casing_control_words falls back to None"
    );
    assert_eq!(
        config.format.indent_top_level,
        Some(IndentTopLevelMode::Auto),
        "the other, valid key must still parse correctly"
    );
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::InvalidValue { key, .. } if key == "casing_control_words"));

    cleanup(&path);
}

#[test]
fn unrecognized_key_inside_format_warns_but_other_keys_still_apply() {
    let path = write_config(
        "unrecognized_key",
        "[format]\ncsing = \"upper\"\nindent_top_level = \"auto\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing_control_words, None);
    assert_eq!(config.format.indent_top_level, Some(IndentTopLevelMode::Auto));
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::UnrecognizedKey { key, table, .. } if key == "csing" && table == "format"));

    cleanup(&path);
}

#[test]
fn the_removed_legacy_casing_key_is_an_unrecognized_key_not_a_hard_failure() {
    // The flat `casing` field (superseded by casing_control_words/
    // casing_pair_keywords/casing_data_references) was removed -- a
    // drut.toml still using it degrades exactly like any other unknown
    // key: one warning, every other valid key in the same file still
    // applies.
    let path = write_config(
        "legacy_casing_removed",
        "[format]\ncasing = \"upper\"\nindent_top_level = \"auto\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing_control_words, None);
    assert_eq!(config.format.casing_pair_keywords, None);
    assert_eq!(config.format.indent_top_level, Some(IndentTopLevelMode::Auto));
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::UnrecognizedKey { key, table, .. } if key == "casing" && table == "format"));

    cleanup(&path);
}

#[test]
fn unrecognized_top_level_table_is_silently_ignored_not_warned() {
    let path = write_config(
        "unrecognized_table",
        "[lint]\nseverity = \"error\"\n\n[format]\ncasing_control_words = \"lower\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing_control_words, Some(CasingConvention::Lower));
    assert!(
        warnings.is_empty(),
        "an unrecognized top-level table must not warn (forward-compat), got {warnings:?}"
    );

    cleanup(&path);
}

#[test]
fn a_file_that_is_not_valid_toml_produces_one_parse_error_and_empty_config() {
    let path = write_config("invalid_syntax", "[format\ncasing_control_words = \"upper\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing_control_words, None);
    assert_eq!(config.format.indent_top_level, None);
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::ParseError { .. }));

    cleanup(&path);
}

#[test]
fn a_file_with_no_format_table_at_all_parses_to_empty_config_with_no_warnings() {
    let path = write_config("no_format_table", "[lint]\nseverity = \"error\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing_control_words, None);
    assert_eq!(config.format.indent_top_level, None);
    assert!(warnings.is_empty());

    cleanup(&path);
}

#[test]
fn an_unreadable_path_produces_a_parse_error_not_a_panic() {
    let path = Path::new("this/definitely/does/not/exist/drut.toml");
    let (config, warnings) = parse(path);
    assert_eq!(config.format.casing_control_words, None);
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::ParseError { .. }));
}

// -- 017-casing-categories-indent-width: the three new granular casing
// fields, all sharing parse_casing's shape (tasks.md T011/T014) --

#[test]
fn each_new_granular_casing_field_parses_cleanly_with_zero_warnings() {
    let path = write_config(
        "granular_casing",
        "[format]\ncasing_control_words = \"upper\"\ncasing_pair_keywords = \"lower\"\ncasing_data_references = \"preserve\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing_control_words, Some(CasingConvention::Upper));
    assert_eq!(config.format.casing_pair_keywords, Some(CasingConvention::Lower));
    assert_eq!(config.format.casing_data_references, Some(CasingConvention::Preserve));
    assert!(warnings.is_empty(), "expected zero warnings, got {warnings:?}");

    cleanup(&path);
}

#[test]
fn a_malformed_granular_casing_value_falls_back_only_for_that_key() {
    let path = write_config(
        "granular_casing_invalid",
        "[format]\ncasing_data_references = \"sideways\"\ncasing_control_words = \"upper\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing_data_references, None);
    assert_eq!(config.format.casing_control_words, Some(CasingConvention::Upper));
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::InvalidValue { key, .. } if key == "casing_data_references"));

    cleanup(&path);
}

#[test]
fn auto_is_rejected_as_an_unrecognized_casing_value_at_every_casing_field() {
    // 017-casing-categories-indent-width FR-003/tasks.md T039: this feature
    // ships no built-in preset -- "auto" is deliberately just another
    // unrecognized string, not a fourth accepted value, at any of the
    // three granular fields.
    let path = write_config(
        "auto_rejected",
        "[format]\ncasing_control_words = \"auto\"\ncasing_pair_keywords = \"auto\"\ncasing_data_references = \"auto\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing_control_words, None);
    assert_eq!(config.format.casing_pair_keywords, None);
    assert_eq!(config.format.casing_data_references, None);
    assert_eq!(warnings.len(), 3, "each of the three casing fields must independently warn: {warnings:?}");
    assert!(warnings.iter().all(|w| matches!(w, ConfigWarning::InvalidValue { .. })));

    cleanup(&path);
}

// -- 025-function-casing: the fourth granular casing field, same shape --

#[test]
fn casing_function_calls_parses_cleanly_with_zero_warnings() {
    let path = write_config("casing_function_calls_valid", "[format]\ncasing_function_calls = \"upper\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing_function_calls, Some(CasingConvention::Upper));
    assert!(warnings.is_empty(), "expected zero warnings, got {warnings:?}");

    cleanup(&path);
}

#[test]
fn casing_function_calls_rejects_an_invalid_value_without_affecting_other_keys() {
    let path = write_config(
        "casing_function_calls_invalid",
        "[format]\ncasing_function_calls = \"sideways\"\ncasing_control_words = \"upper\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing_function_calls, None);
    assert_eq!(config.format.casing_control_words, Some(CasingConvention::Upper));
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::InvalidValue { key, .. } if key == "casing_function_calls"));

    cleanup(&path);
}

// -- 017-casing-categories-indent-width: indent_width (tasks.md T011) --

#[test]
fn indent_width_parses_as_an_integer_with_zero_warnings() {
    let path = write_config("indent_width_valid", "[format]\nindent_width = 2\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.indent_width, Some(2));
    assert!(warnings.is_empty(), "expected zero warnings, got {warnings:?}");

    cleanup(&path);
}

#[test]
fn indent_width_wrong_type_is_an_invalid_value_not_a_panic() {
    let path = write_config("indent_width_wrong_type", "[format]\nindent_width = \"two\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.indent_width, None);
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::InvalidValue { key, .. } if key == "indent_width"));

    cleanup(&path);
}

#[test]
fn indent_width_that_does_not_even_fit_in_a_u8_is_an_invalid_value_at_parse_time() {
    // 500 doesn't fit in a u8 at all -- this is a parse-time InvalidValue,
    // distinct from a value that fits the type but is outside 1-16 (that
    // narrower range check is a resolve_format_options concern instead,
    // data-model.md §4 -- exercised by drut-config's own resolve.rs tests,
    // not here, since a value like `20` parses to `Some(20)` cleanly at
    // this layer).
    let path = write_config("indent_width_does_not_fit_u8", "[format]\nindent_width = 500\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.indent_width, None);
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::InvalidValue { key, .. } if key == "indent_width"));

    cleanup(&path);
}

// -- 018-operator-spacing: operator_spacing (tasks.md T015) --

#[test]
fn operator_spacing_parses_each_of_the_three_accepted_values_with_zero_warnings() {
    for (value, expected) in [
        ("preserve", OperatorSpacing::Preserve),
        ("fixed", OperatorSpacing::Fixed),
        ("auto", OperatorSpacing::Auto),
    ] {
        let path = write_config(&format!("operator_spacing_{value}"), &format!("[format]\noperator_spacing = \"{value}\"\n"));

        let (config, warnings) = parse(&path);
        assert_eq!(config.format.operator_spacing, Some(expected), "value: {value}");
        assert!(warnings.is_empty(), "expected zero warnings for {value:?}, got {warnings:?}");

        cleanup(&path);
    }
}

#[test]
fn operator_spacing_malformed_value_warns_and_falls_back_to_none() {
    let path = write_config("operator_spacing_malformed", "[format]\noperator_spacing = \"tight\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.operator_spacing, None);
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::InvalidValue { key, .. } if key == "operator_spacing"));

    cleanup(&path);
}

// -- 019-blank-line-normalization: blank_lines + both caps (tasks.md T014) --

#[test]
fn blank_lines_parses_both_accepted_values_with_zero_warnings() {
    for (value, expected) in [("preserve", BlankLineMode::Preserve), ("auto", BlankLineMode::Auto)] {
        let path = write_config(&format!("blank_lines_{value}"), &format!("[format]\nblank_lines = \"{value}\"\n"));

        let (config, warnings) = parse(&path);
        assert_eq!(config.format.blank_lines, Some(expected), "value: {value}");
        assert!(warnings.is_empty(), "expected zero warnings for {value:?}, got {warnings:?}");

        cleanup(&path);
    }
}

#[test]
fn blank_lines_malformed_value_warns_and_falls_back_to_none() {
    let path = write_config("blank_lines_malformed", "[format]\nblank_lines = \"sometimes\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.blank_lines, None);
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::InvalidValue { key, .. } if key == "blank_lines"));

    cleanup(&path);
}

#[test]
fn each_blank_line_cap_parses_a_plain_integer_cleanly() {
    let path = write_config(
        "blank_line_caps_valid",
        "[format]\nblank_lines_top_cap = 3\nblank_lines_nested_cap = 2\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.blank_lines_top_cap, Some(3));
    assert_eq!(config.format.blank_lines_nested_cap, Some(2));
    assert!(warnings.is_empty(), "expected zero warnings, got {warnings:?}");

    cleanup(&path);
}

#[test]
fn a_malformed_blank_line_cap_value_warns_and_falls_back_only_for_that_key() {
    let path = write_config(
        "blank_line_caps_malformed",
        "[format]\nblank_lines_top_cap = \"two\"\nblank_lines_nested_cap = 1\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.blank_lines_top_cap, None);
    assert_eq!(config.format.blank_lines_nested_cap, Some(1));
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::InvalidValue { key, .. } if key == "blank_lines_top_cap"));

    cleanup(&path);
}

// -- 030-auto-line-wrap: line_wrap + line_wrap_width + line_wrap_style --

#[test]
fn line_wrap_parses_both_accepted_values_with_zero_warnings() {
    for (value, expected) in [("preserve", LineWrapMode::Preserve), ("auto", LineWrapMode::Auto)] {
        let path = write_config(&format!("line_wrap_{value}"), &format!("[format]\nline_wrap = \"{value}\"\n"));

        let (config, warnings) = parse(&path);
        assert_eq!(config.format.line_wrap, Some(expected), "value: {value}");
        assert!(warnings.is_empty(), "expected zero warnings for {value:?}, got {warnings:?}");

        cleanup(&path);
    }
}

#[test]
fn line_wrap_malformed_value_warns_and_falls_back_to_none() {
    let path = write_config("line_wrap_malformed", "[format]\nline_wrap = \"sometimes\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.line_wrap, None);
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::InvalidValue { key, .. } if key == "line_wrap"));

    cleanup(&path);
}

#[test]
fn line_wrap_width_parses_a_plain_integer_cleanly() {
    let path = write_config("line_wrap_width_valid", "[format]\nline_wrap_width = 100\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.line_wrap_width, Some(100));
    assert!(warnings.is_empty(), "expected zero warnings, got {warnings:?}");

    cleanup(&path);
}

#[test]
fn line_wrap_width_malformed_value_warns_and_falls_back_to_none() {
    let path = write_config("line_wrap_width_malformed", "[format]\nline_wrap_width = \"wide\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.line_wrap_width, None);
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::InvalidValue { key, .. } if key == "line_wrap_width"));

    cleanup(&path);
}

#[test]
fn line_wrap_style_parses_both_accepted_values_with_zero_warnings() {
    for (value, expected) in [("fill", LineWrapStyle::Fill), ("one_per_line", LineWrapStyle::OnePerLine)] {
        let path = write_config(&format!("line_wrap_style_{value}"), &format!("[format]\nline_wrap_style = \"{value}\"\n"));

        let (config, warnings) = parse(&path);
        assert_eq!(config.format.line_wrap_style, Some(expected), "value: {value}");
        assert!(warnings.is_empty(), "expected zero warnings for {value:?}, got {warnings:?}");

        cleanup(&path);
    }
}

#[test]
fn line_wrap_style_malformed_value_warns_and_falls_back_to_none() {
    let path = write_config("line_wrap_style_malformed", "[format]\nline_wrap_style = \"packed\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.line_wrap_style, None);
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::InvalidValue { key, .. } if key == "line_wrap_style"));

    cleanup(&path);
}
