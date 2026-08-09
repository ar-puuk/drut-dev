//! `format` subcommand orchestration (spec.md FR-012–FR-025; data-model.md §5).

use std::path::{Path, PathBuf};

use similar::TextDiff;
use voyager_core::format::{format_bytes, CasingConvention, EncodingFidelity, FormatOptions};

use crate::cli::CasingArg;
use crate::exit::ExitOutcome;
use crate::io_util::{write_stdout, write_stdout_line};
use crate::traverse::{traverse, ReadFailure};

impl From<CasingArg> for CasingConvention {
    fn from(value: CasingArg) -> Self {
        match value {
            CasingArg::Upper => CasingConvention::Upper,
            CasingArg::Lower => CasingConvention::Lower,
        }
    }
}

/// One matched file's disposition (data-model.md §5).
pub enum FormatOutcome {
    Unchanged,
    Changed { diff: Option<String> },
    /// `--write` mode, successfully overwritten.
    Written,
    /// `--write` mode: either the OS-level write itself failed, or the
    /// write was refused before being attempted because `encoding_fidelity`
    /// was `Lossy` (FR-025) — the `message` distinguishes which.
    WriteFailed { message: String },
}

pub struct FormatReport {
    pub outcomes: Vec<(PathBuf, FormatOutcome)>,
    pub read_failures: Vec<ReadFailure>,
    /// Populated in every mode, not only `--write` (FR-025).
    pub unsafe_encoding_files: Vec<PathBuf>,
    /// Populated in every mode, not only `--diff` (FR-024).
    pub recovered_encoding_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Default,
    Write,
    Check,
    Diff,
}

pub fn run(path: &Path, write: bool, check: bool, diff: bool, casing: Option<CasingArg>) -> ExitOutcome {
    let mode = if write {
        Mode::Write
    } else if check {
        Mode::Check
    } else if diff {
        Mode::Diff
    } else {
        Mode::Default
    };
    let options = FormatOptions {
        casing: casing.map(CasingConvention::from),
    };

    let traversal = traverse(path);

    if let Some(reason) = traversal.invalid_target {
        eprintln!("error: {reason}");
        return ExitOutcome::Fatal;
    }

    for failure in &traversal.read_failures {
        eprintln!("error: could not read {}: {}", failure.path.display(), failure.message);
    }

    let mut report = FormatReport {
        outcomes: Vec::new(),
        read_failures: traversal.read_failures,
        unsafe_encoding_files: Vec::new(),
        recovered_encoding_files: Vec::new(),
    };

    for file in &traversal.matched_files {
        let result = format_bytes(&file.bytes, options);

        match result.encoding_fidelity {
            EncodingFidelity::Lossy => report.unsafe_encoding_files.push(file.path.clone()),
            EncodingFidelity::Recovered => report.recovered_encoding_files.push(file.path.clone()),
            EncodingFidelity::Faithful => {}
        }
        let is_lossy = result.encoding_fidelity == EncodingFidelity::Lossy;

        let outcome = match mode {
            Mode::Write => {
                if is_lossy {
                    FormatOutcome::WriteFailed {
                        message: "cannot safely format: file contains an undecodable byte (InvalidEncoding); write refused".to_string(),
                    }
                } else if result.changed {
                    match std::fs::write(&file.path, &result.text) {
                        Ok(()) => FormatOutcome::Written,
                        Err(e) => FormatOutcome::WriteFailed { message: e.to_string() },
                    }
                } else {
                    FormatOutcome::Unchanged
                }
            }
            Mode::Check => {
                if result.changed {
                    FormatOutcome::Changed { diff: None }
                } else {
                    FormatOutcome::Unchanged
                }
            }
            Mode::Diff => {
                if result.changed {
                    let original = String::from_utf8_lossy(&file.bytes);
                    let diff_text = unified_diff(&file.path, &original, &result.text);
                    FormatOutcome::Changed { diff: Some(diff_text) }
                } else {
                    FormatOutcome::Unchanged
                }
            }
            Mode::Default => {
                // Printed unconditionally, even for a Lossy file — nothing
                // is written in this mode, so showing it is informational,
                // not a safety concern (FR-025 only gates persistence).
                write_stdout(&result.text);
                if result.changed {
                    FormatOutcome::Changed { diff: None }
                } else {
                    FormatOutcome::Unchanged
                }
            }
        };

        report.outcomes.push((file.path.clone(), outcome));
    }

    print_report(&report, mode);
    derive_exit_outcome(&report, mode)
}

fn unified_diff(path: &Path, original: &str, formatted: &str) -> String {
    let label = path.display().to_string();
    TextDiff::from_lines(original, formatted)
        .unified_diff()
        .header(&label, &label)
        .to_string()
}

/// Stdout carries each mode's actual "result" (formatted content for the
/// default mode, diff text for `--diff`, brief status lines for `--write`/
/// `--check`); stderr carries notices and errors, including FR-024/FR-025
/// reporting in every mode — kept off stdout specifically so piping
/// `drut format file.s > file.s`-style default-mode output never gets
/// corrupted by a notice line landing in the middle of it.
fn print_report(report: &FormatReport, mode: Mode) {
    match mode {
        Mode::Write => {
            for (path, outcome) in &report.outcomes {
                match outcome {
                    FormatOutcome::Written => write_stdout_line(&format!("{}: formatted", path.display())),
                    FormatOutcome::WriteFailed { message } => {
                        eprintln!("error: {}: {message}", path.display());
                    }
                    FormatOutcome::Unchanged | FormatOutcome::Changed { .. } => {}
                }
            }
        }
        Mode::Check => {
            for (path, outcome) in &report.outcomes {
                if matches!(outcome, FormatOutcome::Changed { .. }) {
                    write_stdout_line(&format!("{}: would reformat", path.display()));
                }
            }
        }
        Mode::Diff => {
            for (_path, outcome) in &report.outcomes {
                if let FormatOutcome::Changed { diff: Some(text) } = outcome {
                    write_stdout(text);
                }
            }
        }
        Mode::Default => {
            // Each file's formatted content was already printed to stdout
            // during the main loop, interleaved in traversal order.
        }
    }

    if !report.recovered_encoding_files.is_empty() {
        eprintln!(
            "{} file(s) had legacy-encoding bytes normalized to UTF-8:",
            report.recovered_encoding_files.len()
        );
        for path in &report.recovered_encoding_files {
            eprintln!("  {}", path.display());
        }
    }
    if !report.unsafe_encoding_files.is_empty() {
        eprintln!(
            "{} file(s) refused: contains an undecodable byte, cannot safely format:",
            report.unsafe_encoding_files.len()
        );
        for path in &report.unsafe_encoding_files {
            eprintln!("  {}", path.display());
        }
    }
}

/// FR-020/data-model.md §5's three-way rule. `Fatal` wins whenever it
/// applies, `unsafe_encoding_files` included regardless of mode — this is
/// the one case a `--check`/`--diff`/default run can still exit `2`, since a
/// `Lossy` file means `--write` would refuse even though nothing was
/// actually written this time (FR-025).
fn derive_exit_outcome(report: &FormatReport, mode: Mode) -> ExitOutcome {
    let any_write_failed = report
        .outcomes
        .iter()
        .any(|(_, o)| matches!(o, FormatOutcome::WriteFailed { .. }));

    if any_write_failed || !report.unsafe_encoding_files.is_empty() || !report.read_failures.is_empty() {
        return ExitOutcome::Fatal;
    }

    let any_changed = report
        .outcomes
        .iter()
        .any(|(_, o)| matches!(o, FormatOutcome::Changed { .. }));
    if mode == Mode::Check && any_changed {
        return ExitOutcome::ProblemsFound;
    }

    ExitOutcome::Clean
}
