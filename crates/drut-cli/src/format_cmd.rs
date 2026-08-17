//! `format` subcommand orchestration (spec.md FR-012–FR-025; data-model.md §5).

use std::path::{Path, PathBuf};

use drut_config::{resolve_format_options, ConfigWarning, ExplicitFormatOverride};
use similar::TextDiff;
use voyager_core::format::{format_bytes, CasingConvention, EncodingFidelity, TopLevelIndentMode};
use voyager_core::Position;

use crate::cli::{CasingArg, TopLevelIndentArg};
use crate::exit::ExitOutcome;
use crate::io_util::{write_stdout, write_stdout_line};
use crate::traverse::{traverse, ReadFailure};

impl From<CasingArg> for CasingConvention {
    fn from(value: CasingArg) -> Self {
        match value {
            CasingArg::Preserve => CasingConvention::Preserve,
            CasingArg::Upper => CasingConvention::Upper,
            CasingArg::Lower => CasingConvention::Lower,
        }
    }
}

impl From<TopLevelIndentArg> for TopLevelIndentMode {
    fn from(value: TopLevelIndentArg) -> Self {
        match value {
            TopLevelIndentArg::Preserve => TopLevelIndentMode::Preserve,
            TopLevelIndentArg::Normalize => TopLevelIndentMode::Normalize,
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
    /// Populated in every mode — 010-fmt-region-markers FR-010. Informational
    /// only; never affects the exit code (mirrors `recovered_encoding_files`'
    /// own treatment, not `unsafe_encoding_files`', since an unclosed marker
    /// is not an error, just a fact worth surfacing).
    pub unclosed_fmt_off_files: Vec<(PathBuf, Vec<Position>)>,
    /// Populated in every mode — 012-toml-configuration FR-011. A malformed
    /// `drut.toml` never blocks formatting (per-field fallback still
    /// applies); this is purely informational, same treatment as
    /// `unclosed_fmt_off_files` above — never affects the exit code
    /// (research.md §6, confirmed directly against `exit.rs`).
    pub config_warnings: Vec<(PathBuf, Vec<ConfigWarning>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Default,
    Write,
    Check,
    Diff,
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    path: &Path,
    write: bool,
    check: bool,
    diff: bool,
    casing: Option<CasingArg>,
    control_words_casing: Option<CasingArg>,
    pair_keywords_casing: Option<CasingArg>,
    data_references_casing: Option<CasingArg>,
    indent_width: Option<u8>,
    top_level_indent: Option<TopLevelIndentArg>,
    isolated: bool,
) -> ExitOutcome {
    let mode = if write {
        Mode::Write
    } else if check {
        Mode::Check
    } else if diff {
        Mode::Diff
    } else {
        Mode::Default
    };
    let explicit = ExplicitFormatOverride {
        casing: casing.map(CasingConvention::from),
        control_words_casing: control_words_casing.map(CasingConvention::from),
        pair_keywords_casing: pair_keywords_casing.map(CasingConvention::from),
        data_references_casing: data_references_casing.map(CasingConvention::from),
        top_level_indent: top_level_indent.map(TopLevelIndentMode::from),
        indent_width,
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
        unclosed_fmt_off_files: Vec::new(),
        config_warnings: Vec::new(),
    };

    for file in &traversal.matched_files {
        let (options, warnings) = resolve_format_options(Some(&file.path), isolated, explicit);
        if !warnings.is_empty() {
            report.config_warnings.push((file.path.clone(), warnings));
        }
        let result = format_bytes(&file.bytes, options);

        match result.encoding_fidelity {
            EncodingFidelity::Lossy => report.unsafe_encoding_files.push(file.path.clone()),
            EncodingFidelity::Recovered => report.recovered_encoding_files.push(file.path.clone()),
            EncodingFidelity::Faithful => {}
        }
        if !result.unclosed_fmt_off_markers.is_empty() {
            report
                .unclosed_fmt_off_files
                .push((file.path.clone(), result.unclosed_fmt_off_markers.clone()));
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
    if !report.unclosed_fmt_off_files.is_empty() {
        eprintln!(
            "{} file(s) have an unclosed '; FMT: OFF' marker (protection extended to end of file):",
            report.unclosed_fmt_off_files.len()
        );
        for (path, positions) in &report.unclosed_fmt_off_files {
            let lines: Vec<String> = positions.iter().map(|p| format!("line {}", p.line)).collect();
            eprintln!("  {} ({})", path.display(), lines.join(", "));
        }
    }
    if !report.config_warnings.is_empty() {
        eprintln!(
            "{} file(s) have a drut.toml problem (built-in defaults used for the affected setting(s)):",
            report.config_warnings.len()
        );
        for (path, warnings) in &report.config_warnings {
            eprintln!("  {}:", path.display());
            for warning in warnings {
                eprintln!("    {warning}");
            }
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
