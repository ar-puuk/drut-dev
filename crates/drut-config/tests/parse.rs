//! Unit tests for `drut_config::parse::parse` (012-toml-configuration T008).

use std::path::{Path, PathBuf};

use drut_config::{parse::parse, ConfigWarning};
use voyager_core::{CasingConvention, TopLevelIndentMode};

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
    let path = write_config("valid", "[format]\ncasing = \"upper\"\ntop_level_indent = \"normalize\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing, Some(CasingConvention::Upper));
    assert_eq!(config.format.top_level_indent, Some(TopLevelIndentMode::Normalize));
    assert!(warnings.is_empty(), "expected zero warnings, got {warnings:?}");

    cleanup(&path);
}

#[test]
fn casing_preserve_parses_cleanly_with_zero_warnings() {
    // 014-casing-preserve-mode FR-005/SC-005: "preserve" is a recognized
    // value, not an unrecognized one that happens to be a no-op.
    let path = write_config("casing_preserve", "[format]\ncasing = \"preserve\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing, Some(CasingConvention::Preserve));
    assert!(warnings.is_empty(), "expected zero warnings, got {warnings:?}");

    cleanup(&path);
}

#[test]
fn invalid_value_for_one_key_falls_back_only_for_that_key() {
    let path = write_config(
        "invalid_value",
        "[format]\ncasing = \"sideways\"\ntop_level_indent = \"normalize\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing, None, "invalid casing falls back to None");
    assert_eq!(
        config.format.top_level_indent,
        Some(TopLevelIndentMode::Normalize),
        "the other, valid key must still parse correctly"
    );
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::InvalidValue { key, .. } if key == "casing"));

    cleanup(&path);
}

#[test]
fn unrecognized_key_inside_format_warns_but_other_keys_still_apply() {
    let path = write_config(
        "unrecognized_key",
        "[format]\ncsing = \"upper\"\ntop_level_indent = \"normalize\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing, None);
    assert_eq!(config.format.top_level_indent, Some(TopLevelIndentMode::Normalize));
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::UnrecognizedKey { key, table, .. } if key == "csing" && table == "format"));

    cleanup(&path);
}

#[test]
fn unrecognized_top_level_table_is_silently_ignored_not_warned() {
    let path = write_config(
        "unrecognized_table",
        "[lint]\nseverity = \"error\"\n\n[format]\ncasing = \"lower\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing, Some(CasingConvention::Lower));
    assert!(
        warnings.is_empty(),
        "an unrecognized top-level table must not warn (forward-compat), got {warnings:?}"
    );

    cleanup(&path);
}

#[test]
fn a_file_that_is_not_valid_toml_produces_one_parse_error_and_empty_config() {
    let path = write_config("invalid_syntax", "[format\ncasing = \"upper\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing, None);
    assert_eq!(config.format.top_level_indent, None);
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::ParseError { .. }));

    cleanup(&path);
}

#[test]
fn a_file_with_no_format_table_at_all_parses_to_empty_config_with_no_warnings() {
    let path = write_config("no_format_table", "[lint]\nseverity = \"error\"\n");

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing, None);
    assert_eq!(config.format.top_level_indent, None);
    assert!(warnings.is_empty());

    cleanup(&path);
}

#[test]
fn an_unreadable_path_produces_a_parse_error_not_a_panic() {
    let path = Path::new("this/definitely/does/not/exist/drut.toml");
    let (config, warnings) = parse(path);
    assert_eq!(config.format.casing, None);
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::ParseError { .. }));
}

// -- 017-casing-categories-indent-width: the three new granular casing
// fields, all sharing parse_casing's shape (tasks.md T011/T014) --

#[test]
fn each_new_granular_casing_field_parses_cleanly_with_zero_warnings() {
    let path = write_config(
        "granular_casing",
        "[format]\ncontrol_words_casing = \"upper\"\npair_keywords_casing = \"lower\"\ndata_references_casing = \"preserve\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.control_words_casing, Some(CasingConvention::Upper));
    assert_eq!(config.format.pair_keywords_casing, Some(CasingConvention::Lower));
    assert_eq!(config.format.data_references_casing, Some(CasingConvention::Preserve));
    assert!(warnings.is_empty(), "expected zero warnings, got {warnings:?}");

    cleanup(&path);
}

#[test]
fn a_malformed_granular_casing_value_falls_back_only_for_that_key() {
    let path = write_config(
        "granular_casing_invalid",
        "[format]\ndata_references_casing = \"sideways\"\ncontrol_words_casing = \"upper\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.data_references_casing, None);
    assert_eq!(config.format.control_words_casing, Some(CasingConvention::Upper));
    assert_eq!(warnings.len(), 1);
    assert!(matches!(&warnings[0], ConfigWarning::InvalidValue { key, .. } if key == "data_references_casing"));

    cleanup(&path);
}

#[test]
fn auto_is_rejected_as_an_unrecognized_casing_value_at_every_casing_field() {
    // 017-casing-categories-indent-width FR-003/tasks.md T039: this feature
    // ships no built-in preset -- "auto" is deliberately just another
    // unrecognized string, not a fourth accepted value, at the legacy
    // field and all three new granular fields alike.
    let path = write_config(
        "auto_rejected",
        "[format]\ncasing = \"auto\"\ncontrol_words_casing = \"auto\"\npair_keywords_casing = \"auto\"\ndata_references_casing = \"auto\"\n",
    );

    let (config, warnings) = parse(&path);
    assert_eq!(config.format.casing, None);
    assert_eq!(config.format.control_words_casing, None);
    assert_eq!(config.format.pair_keywords_casing, None);
    assert_eq!(config.format.data_references_casing, None);
    assert_eq!(warnings.len(), 4, "each of the four casing fields must independently warn: {warnings:?}");
    assert!(warnings.iter().all(|w| matches!(w, ConfigWarning::InvalidValue { .. })));

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
