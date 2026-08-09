//! `clap`-derive CLI surface (data-model.md §2; contracts/cli-contract.md).

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "drut",
    about = "Structural linter/formatter for Cube Voyager control-statement scripts"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Report structural diagnostics for every `.s`/`.block` file under <PATH>.
    Check {
        /// A file or directory (FR-001).
        path: PathBuf,
        /// text (default, FR-008) or sarif (FR-009). Default holds in every
        /// context, interactive or not (FR-010).
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// Normalize whitespace (and, opt-in, keyword casing) for every `.s`/
    /// `.block` file under <PATH>.
    Format {
        /// Same traversal/filtering rules as `check` (FR-001–FR-003).
        path: PathBuf,
        /// Overwrite each matched file in place (FR-017).
        #[arg(long, conflicts_with_all = ["check", "diff"])]
        write: bool,
        /// Report which files would change; write nothing (FR-018).
        #[arg(long, conflicts_with_all = ["write", "diff"])]
        check: bool,
        /// Print a unified diff per changed file; write nothing (FR-019).
        #[arg(long, conflicts_with_all = ["write", "check"])]
        diff: bool,
        /// Opt-in keyword-casing convention — must be `upper` or `lower`
        /// when given; no bare `--casing` (FR-015).
        #[arg(long, value_enum)]
        casing: Option<CasingArg>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Sarif,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CasingArg {
    Upper,
    Lower,
}

// The CasingArg -> voyager_core::CasingConvention conversion lives in
// format_cmd.rs (added alongside the voyager-core format module, US2) rather
// than here, so this Foundational-phase module has no dependency on US2's
// not-yet-built types.
