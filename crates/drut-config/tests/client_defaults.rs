//! Unit tests for `resolve_format_options`'s `client_defaults` tier
//! (021-editor-settings-config tasks.md T006, contracts/
//! editor-settings-config.md). `client_defaults` sits between `drut.toml`
//! and the built-in default: it applies only when neither `explicit` nor
//! `drut.toml` set a field, and never wins over a `drut.toml` value for the
//! same field.

use std::path::{Path, PathBuf};

use drut_config::{resolve_format_options, ExplicitFormatOverride};
use voyager_core::{CasingConvention, OperatorSpacing, TopLevelIndentMode};

fn test_file(name: &str, config: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("drut_config_client_defaults_test_{}_{name}", std::process::id()));
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

/// US1 AS1/AS3 shape: with no `drut.toml` present at all, a `client_defaults`
/// value applies since neither `explicit` nor a config file set the field.
#[test]
fn client_defaults_value_applies_with_no_drut_toml_and_no_explicit_value() {
    let dir = std::env::temp_dir().join(format!("drut_config_client_defaults_test_{}_no_toml", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(".git")).unwrap(); // stops drut.toml discovery from walking further up.
    let target = dir.join("a.s");
    std::fs::write(&target, "IF (a=b)\nENDIF\n").unwrap();

    let client_defaults = ExplicitFormatOverride {
        control_words_casing: Some(CasingConvention::Upper),
        indent_width: Some(2),
        operator_spacing: Some(OperatorSpacing::Fixed),
        ..Default::default()
    };
    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), client_defaults);
    assert_eq!(options.casing.control_words, CasingConvention::Upper, "client_defaults must apply with nothing else set");
    assert_eq!(options.indent_width, 2);
    assert_eq!(options.operator_spacing, OperatorSpacing::Fixed);
    assert!(warnings.is_empty());

    cleanup(&target);
}

/// US2 AS1: a `drut.toml` value wins over a conflicting `client_defaults`
/// value for the same field.
#[test]
fn drut_toml_value_wins_over_a_conflicting_client_defaults_value_for_the_same_field() {
    let target = test_file("toml_wins", "[format]\nindent_width = 2\n");

    let client_defaults = ExplicitFormatOverride { indent_width: Some(8), ..Default::default() };
    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), client_defaults);
    assert_eq!(options.indent_width, 2, "drut.toml's own value must win over client_defaults");

    cleanup(&target);
}

/// US2 AS2: a `client_defaults` value wins for a *different* field
/// `drut.toml` doesn't set at all, in the same resolution call.
#[test]
fn client_defaults_wins_for_a_field_drut_toml_does_not_set_even_when_toml_governs_another_field() {
    let target = test_file("toml_partial", "[format]\nindent_width = 2\n");

    let client_defaults = ExplicitFormatOverride {
        indent_width: Some(8),
        top_level_indent: Some(TopLevelIndentMode::Auto),
        ..Default::default()
    };
    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), client_defaults);
    assert_eq!(options.indent_width, 2, "drut.toml still wins for the field it sets");
    assert_eq!(
        options.top_level_indent,
        TopLevelIndentMode::Auto,
        "client_defaults wins for the field drut.toml never mentions"
    );

    cleanup(&target);
}

/// An out-of-range `client_defaults` numeric value falls back to the
/// built-in default with a non-blocking notice, same degrade-not-fail
/// contract an invalid `drut.toml` value already has (spec.md FR-005).
#[test]
fn out_of_range_client_defaults_indent_width_falls_back_to_default_with_a_warning() {
    let dir = std::env::temp_dir().join(format!("drut_config_client_defaults_test_{}_oor_indent", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    let target = dir.join("a.s");
    std::fs::write(&target, "IF (a=b)\nENDIF\n").unwrap();

    let client_defaults = ExplicitFormatOverride { indent_width: Some(0), ..Default::default() };
    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), client_defaults);
    assert_eq!(options.indent_width, 4, "must fall back to the built-in default, never fail");
    assert_eq!(warnings.len(), 1, "expected a non-blocking notice, got: {warnings:?}");

    cleanup(&target);
}

/// Same shape as the indent_width case above, for a blank-line cap.
#[test]
fn out_of_range_client_defaults_blank_line_cap_falls_back_to_default_with_a_warning() {
    let dir = std::env::temp_dir().join(format!("drut_config_client_defaults_test_{}_oor_cap", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    let target = dir.join("a.s");
    std::fs::write(&target, "IF (a=b)\nENDIF\n").unwrap();

    let client_defaults = ExplicitFormatOverride { top_level_blank_line_cap: Some(200), ..Default::default() };
    let (options, warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), client_defaults);
    assert_eq!(options.top_level_blank_line_cap, 2, "must fall back to the built-in default, never fail");
    assert_eq!(warnings.len(), 1, "expected a non-blocking notice, got: {warnings:?}");

    cleanup(&target);
}

/// A `client_defaults` value for one granular casing field must resolve
/// independently of the others — no legacy `casing` field exists to
/// short-circuit through anymore (it was removed; each `*_casing` field
/// gets the same plain per-field fallback every other setting already has).
#[test]
fn client_defaults_granular_casing_field_resolves_independently() {
    let dir = std::env::temp_dir().join(format!("drut_config_client_defaults_test_{}_granular_casing", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    let target = dir.join("a.s");
    std::fs::write(&target, "IF (a=b)\nENDIF\n").unwrap();

    let client_defaults = ExplicitFormatOverride { control_words_casing: Some(CasingConvention::Upper), ..Default::default() };
    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), client_defaults);
    assert_eq!(options.casing.control_words, CasingConvention::Upper, "client_defaults must apply with nothing else set");
    assert_eq!(
        options.casing.pair_keywords,
        CasingConvention::Preserve,
        "a client_defaults value for one granular field must not leak into another"
    );

    cleanup(&target);
}

/// A `drut.toml` that sets the granular `control_words_casing` field wins
/// over `client_defaults`' own value for that same field; a *different*
/// granular field `drut.toml` never mentions still falls through to
/// `client_defaults` — each field resolves its own four-tier chain
/// independently.
#[test]
fn drut_toml_granular_field_wins_over_client_defaults_for_that_field_only() {
    let target = test_file("toml_granular_wins", "[format]\ncontrol_words_casing = \"lower\"\n");

    let client_defaults = ExplicitFormatOverride {
        control_words_casing: Some(CasingConvention::Upper),
        pair_keywords_casing: Some(CasingConvention::Upper),
        ..Default::default()
    };
    let (options, _warnings) = resolve_format_options(Some(&target), false, ExplicitFormatOverride::default(), client_defaults);
    assert_eq!(options.casing.control_words, CasingConvention::Lower, "drut.toml's granular field must win");
    // pair_keywords isn't set in drut.toml at all -- client_defaults' own
    // value for that field must still reach it.
    assert_eq!(options.casing.pair_keywords, CasingConvention::Upper, "client_defaults must still reach a field drut.toml never sets");

    cleanup(&target);
}

/// Every existing CLI/MCP-shaped call (an explicit value with no
/// `client_defaults`) must still resolve exactly as before this feature —
/// confirms the new tier is purely additive.
#[test]
fn explicit_value_still_wins_over_both_config_and_client_defaults() {
    let target = test_file("explicit_wins_over_both", "[format]\nindent_width = 2\n");

    let client_defaults = ExplicitFormatOverride { indent_width: Some(6), ..Default::default() };
    let explicit = ExplicitFormatOverride { indent_width: Some(10), ..Default::default() };
    let (options, _warnings) = resolve_format_options(Some(&target), false, explicit, client_defaults);
    assert_eq!(options.indent_width, 10, "explicit must win over both drut.toml and client_defaults");

    cleanup(&target);
}
