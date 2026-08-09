//! `check` subcommand orchestration (spec.md FR-006–FR-011; data-model.md §4).

use std::path::{Path, PathBuf};

use voyager_core::{parse_bytes, Diagnostic};

use crate::cli::OutputFormat;
use crate::exit::ExitOutcome;
use crate::report;
use crate::traverse::{traverse, ReadFailure};

/// The aggregate result of a `check` run (data-model.md §4).
pub struct CheckReport {
    pub diagnostics: Vec<(PathBuf, Diagnostic)>,
    pub read_failures: Vec<ReadFailure>,
}

pub fn run(path: &Path, format: OutputFormat) -> ExitOutcome {
    let outcome = traverse(path);

    if let Some(reason) = outcome.invalid_target {
        eprintln!("error: {reason}");
        return ExitOutcome::Fatal;
    }

    for failure in &outcome.read_failures {
        eprintln!("error: could not read {}: {}", failure.path.display(), failure.message);
    }

    // FR-006: parse_bytes, never parse — handles non-UTF-8 script content the
    // same way the underlying parser already guarantees.
    let mut diagnostics = Vec::new();
    for file in &outcome.matched_files {
        let result = parse_bytes(&file.bytes);
        for diag in result.diagnostics {
            diagnostics.push((file.path.clone(), diag));
        }
    }

    let report = CheckReport {
        diagnostics,
        read_failures: outcome.read_failures,
    };

    match format {
        OutputFormat::Text => report::text::print_check_report(&report),
        OutputFormat::Sarif => report::sarif::print_check_report(&report),
    }

    derive_exit_outcome(&report)
}

/// FR-011's three-way rule: `Fatal` takes precedence over `ProblemsFound`.
fn derive_exit_outcome(report: &CheckReport) -> ExitOutcome {
    if !report.read_failures.is_empty() {
        ExitOutcome::Fatal
    } else if !report.diagnostics.is_empty() {
        ExitOutcome::ProblemsFound
    } else {
        ExitOutcome::Clean
    }
}
