//! SC-003: `check --format=sarif` output validates against the official
//! SARIF 2.1.0 JSON Schema, for both a clean and a broken fixture set.
//!
//! The schema is vendored at `tests/schemas/sarif-2.1.0.json` (fetched from
//! the OASIS-published errata01/OS schema — the canonical, standards-body
//! source, not a third-party mirror) so this test runs offline/reproducibly
//! rather than fetching it over the network on every run.

use std::path::PathBuf;
use std::process::Command;

fn schema() -> serde_json::Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/schemas/sarif-2.1.0.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read vendored SARIF schema at {}: {e}", path.display()));
    serde_json::from_str(&text).expect("vendored SARIF schema is valid JSON")
}

fn run_check_sarif(target: &str) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_drut"))
        .args(["check", target, "--format=sarif"])
        .output()
        .expect("failed to run drut");
    let stdout = String::from_utf8(output.stdout).expect("drut's SARIF output is valid UTF-8");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("drut check --format=sarif did not print a single JSON document: {e}\n{stdout}"))
}

#[test]
fn clean_fixture_set_produces_schema_valid_sarif_with_empty_results() {
    let schema = schema();
    let validator = jsonschema::validator_for(&schema).expect("SARIF schema itself compiles");

    let log = run_check_sarif("../voyager-core/tests/fixtures/valid");
    let errors: Vec<_> = validator.iter_errors(&log).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "SARIF log failed schema validation: {errors:#?}\n{log:#}");

    let results = log["runs"][0]["results"].as_array().expect("runs[0].results is an array");
    assert!(results.is_empty(), "clean fixture set should produce zero SARIF results");
}

#[test]
fn broken_fixture_set_produces_schema_valid_sarif_with_results() {
    let schema = schema();
    let validator = jsonschema::validator_for(&schema).expect("SARIF schema itself compiles");

    let log = run_check_sarif("../voyager-core/tests/fixtures/broken");
    let errors: Vec<_> = validator.iter_errors(&log).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "SARIF log failed schema validation: {errors:#?}\n{log:#}");

    let results = log["runs"][0]["results"].as_array().expect("runs[0].results is an array");
    assert!(!results.is_empty(), "broken fixture set should produce at least one SARIF result");

    // Every result's ruleId must be declared in tool.driver.rules (SC-003
    // implies this — an undeclared ruleId is a real SARIF-consumer problem
    // even where the bare schema wouldn't catch it).
    let declared_rule_ids: Vec<&str> = log["runs"][0]["tool"]["driver"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    for result in results {
        let rule_id = result["ruleId"].as_str().expect("result has a ruleId");
        assert!(
            declared_rule_ids.contains(&rule_id),
            "result ruleId {rule_id:?} is not declared in tool.driver.rules"
        );
    }
}

#[test]
fn check_never_surfaces_the_lsp_only_undefined_token_hint() {
    // 020-undefined-token-diagnostic SC-005: this stream is LSP-only, built
    // and published entirely inside drut-lsp/src/diagnostics.rs — `check`
    // must keep exposing exactly the six/seven real DiagnosticKind names
    // and nothing else, even on a document containing an unresolvable
    // @token@ reference.
    let dir = std::env::temp_dir().join(format!(
        "drut-cli-sarif-undefined-token-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.s"), "MSG = @ScenarioDir@\n").unwrap();

    let log = run_check_sarif(dir.to_str().unwrap());
    let results = log["runs"][0]["results"].as_array().expect("runs[0].results is an array");
    assert!(
        results.is_empty(),
        "an unresolvable @token@ must never appear in check's output, got: {results:#?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn check_never_surfaces_the_lsp_only_unused_token_hint() {
    // 029-unused-token-diagnostic SC-005: this stream is LSP-only, built
    // and published entirely inside drut-lsp/src/diagnostics.rs — `check`
    // must keep exposing exactly the same DiagnosticKind names as before
    // this feature, even on a document containing an assignment that's
    // never referenced via @token@.
    let dir = std::env::temp_dir().join(format!(
        "drut-cli-sarif-unused-token-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.s"), "ScenarioDir = 'X:\\model'\n").unwrap();

    let log = run_check_sarif(dir.to_str().unwrap());
    let results = log["runs"][0]["results"].as_array().expect("runs[0].results is an array");
    assert!(
        results.is_empty(),
        "an unused assignment must never appear in check's output, got: {results:#?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
