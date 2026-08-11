//! SARIF 2.1.0 rendering (spec.md FR-009; contracts/sarif-mapping.md).
//!
//! Hand-written, `#[derive(Serialize)]` structs over `serde_json` rather than
//! the `serde-sarif` crate originally planned in research.md §4 — that
//! crate's `schemafy`-based build script hit an Application Control block on
//! the implementation machine. This covers exactly the SARIF subset
//! `contracts/sarif-mapping.md` specifies; schema conformance (SC-003) is
//! proven independently in `tests/sarif_schema.rs` via the `jsonschema`
//! crate, not by trusting either crate's types.

use std::path::Path;

use serde::Serialize;
use voyager_core::{Diagnostic, DiagnosticKind};

use crate::check_cmd::CheckReport;
use crate::io_util::write_stdout_line;

const ALL_KINDS: [DiagnosticKind; 8] = [
    DiagnosticKind::UnmatchedIf,
    DiagnosticKind::UnmatchedLoop,
    DiagnosticKind::UnclosedBlockComment,
    DiagnosticKind::InvalidContinuation,
    DiagnosticKind::UnmatchedRun,
    DiagnosticKind::UnmatchedProcess,
    DiagnosticKind::MisplacedBreak,
    DiagnosticKind::InvalidEncoding,
];

/// `ruleId`, kebab-case per contracts/sarif-mapping.md — original wording,
/// stable across releases (SARIF consumers key suppression state off this).
fn rule_id(kind: DiagnosticKind) -> &'static str {
    match kind {
        DiagnosticKind::UnmatchedIf => "unmatched-if",
        DiagnosticKind::UnmatchedLoop => "unmatched-loop",
        DiagnosticKind::UnclosedBlockComment => "unclosed-block-comment",
        DiagnosticKind::InvalidContinuation => "invalid-continuation",
        DiagnosticKind::UnmatchedRun => "unmatched-run",
        DiagnosticKind::UnmatchedProcess => "unmatched-process",
        DiagnosticKind::MisplacedBreak => "misplaced-break",
        DiagnosticKind::InvalidEncoding => "invalid-encoding",
    }
}

/// A short, original-wording description per kind, for the rule catalog
/// (`tool.driver.rules[].shortDescription`) — not copied from vendor
/// documentation (constitution Principle II).
fn short_description(kind: DiagnosticKind) -> &'static str {
    match kind {
        DiagnosticKind::UnmatchedIf => {
            "An IF has no matching ENDIF, or an ENDIF/ELSEIF/ELSE has no open IF."
        }
        DiagnosticKind::UnmatchedLoop => "A LOOP has no matching ENDLOOP, or an ENDLOOP has no open LOOP.",
        DiagnosticKind::UnclosedBlockComment => {
            "A block comment has no matching closing marker before end of file."
        }
        DiagnosticKind::InvalidContinuation => {
            "A line ends with a continuation character but no valid line follows it."
        }
        DiagnosticKind::UnmatchedRun => {
            "A RUN has no matching ENDRUN and no implicit closer, or an ENDRUN has no open RUN."
        }
        DiagnosticKind::UnmatchedProcess => {
            "A PROCESS/PHASE= has no matching ENDPROCESS/ENDPHASE and no implicit closer."
        }
        DiagnosticKind::MisplacedBreak => "A BREAK statement has no enclosing block of any kind.",
        DiagnosticKind::InvalidEncoding => {
            "A byte in the source could not be decoded as UTF-8 or Windows-1252."
        }
    }
}

#[derive(Serialize)]
struct SarifLog {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<Run>,
}

#[derive(Serialize)]
struct Run {
    tool: Tool,
    results: Vec<SarifResult>,
}

#[derive(Serialize)]
struct Tool {
    driver: Driver,
}

#[derive(Serialize)]
struct Driver {
    name: &'static str,
    version: &'static str,
    rules: Vec<ReportingDescriptor>,
}

#[derive(Serialize)]
struct ReportingDescriptor {
    id: String,
    #[serde(rename = "shortDescription")]
    short_description: ShortDescription,
}

#[derive(Serialize)]
struct ShortDescription {
    text: String,
}

#[derive(Serialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: &'static str,
    message: Message,
    locations: Vec<Location>,
}

#[derive(Serialize)]
struct Message {
    text: String,
}

#[derive(Serialize)]
struct Location {
    #[serde(rename = "physicalLocation")]
    physical_location: PhysicalLocation,
}

#[derive(Serialize)]
struct PhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: ArtifactLocation,
    region: Region,
}

#[derive(Serialize)]
struct ArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
struct Region {
    #[serde(rename = "startLine")]
    start_line: u32,
    #[serde(rename = "startColumn")]
    start_column: u32,
    #[serde(rename = "endLine")]
    end_line: u32,
    #[serde(rename = "endColumn")]
    end_column: u32,
}

/// Prints a single SARIF 2.1.0 log to stdout and nothing else (FR-009) — a
/// caller can pipe stdout directly into a SARIF-consuming tool.
pub fn print_check_report(report: &CheckReport) {
    let log = build(report);
    let json = serde_json::to_string_pretty(&log)
        .expect("SARIF log serialization cannot fail for this data (no non-finite floats, no cycles)");
    write_stdout_line(&json);
}

fn build(report: &CheckReport) -> SarifLog {
    let rules = ALL_KINDS
        .iter()
        .map(|kind| ReportingDescriptor {
            id: rule_id(*kind).to_string(),
            short_description: ShortDescription {
                text: short_description(*kind).to_string(),
            },
        })
        .collect();

    let results = report
        .diagnostics
        .iter()
        .map(|(path, diag)| to_result(path, diag))
        .collect();

    SarifLog {
        schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        version: "2.1.0",
        runs: vec![Run {
            tool: Tool {
                driver: Driver {
                    name: "drut",
                    version: env!("CARGO_PKG_VERSION"),
                    rules,
                },
            },
            results,
        }],
    }
}

fn to_result(path: &Path, diag: &Diagnostic) -> SarifResult {
    SarifResult {
        rule_id: rule_id(diag.kind).to_string(),
        level: "error",
        message: Message {
            text: diag.message.clone(),
        },
        locations: vec![Location {
            physical_location: PhysicalLocation {
                artifact_location: ArtifactLocation {
                    uri: path_to_uri(path),
                },
                region: Region {
                    start_line: diag.span.start.line,
                    start_column: diag.span.start.column,
                    end_line: diag.span.end.line,
                    end_column: diag.span.end.column,
                },
            },
        }],
    }
}

/// A best-effort URI form of a filesystem path — forward slashes, as SARIF
/// consumers generally expect (schema itself does not strictly enforce RFC
/// 3986 syntax; verified empirically in tests/sarif_schema.rs).
fn path_to_uri(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
