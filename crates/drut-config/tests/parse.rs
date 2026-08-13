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
