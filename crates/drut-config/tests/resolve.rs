//! Unit tests for `drut_config::resolve_format_options`
//! (012-toml-configuration T009).

use std::path::{Path, PathBuf};

use drut_config::{resolve_format_options, ExplicitFormatOverride};
use voyager_core::{CasingConvention, TopLevelIndentMode};

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
            top_level_indent: None,
        },
    );
    assert_eq!(options.casing, Some(CasingConvention::Upper), "explicit casing must win");
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
    assert_eq!(options.casing, Some(CasingConvention::Lower));
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
            top_level_indent: None,
        },
    );
    assert_eq!(options.casing, Some(CasingConvention::Upper));
    assert_eq!(options.top_level_indent, TopLevelIndentMode::Preserve, "no file, no explicit -> built-in default");
    assert!(warnings.is_empty());
}

#[test]
fn isolated_skips_discovery_even_with_a_valid_nearby_config_present() {
    let target = test_file("isolated", "[format]\ncasing = \"lower\"\ntop_level_indent = \"normalize\"\n");

    let (options, warnings) = resolve_format_options(Some(&target), true, ExplicitFormatOverride::default());
    assert_eq!(options.casing, None, "isolated must ignore the file entirely, casing stays at built-in default");
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
        },
    );
    assert_eq!(options.casing, Some(CasingConvention::Upper));
    assert_eq!(options.top_level_indent, TopLevelIndentMode::Normalize);

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
    assert_eq!(options.casing, None);
    assert_eq!(options.top_level_indent, TopLevelIndentMode::Preserve);
    assert!(warnings.is_empty());

    cleanup(&target);
}
