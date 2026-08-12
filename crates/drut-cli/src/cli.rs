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
        /// Top-level (depth-0) indentation policy — `preserve` (default,
        /// FR-001) leaves it exactly as written; `normalize` forces every
        /// top-level line to column 0 (FR-002/FR-003,
        /// 009-top-level-indent-toggle). Unlike `--casing`, always has a
        /// value — omitting the flag is not a third "off" state
        /// (research.md §4).
        #[arg(long, value_enum, default_value_t = TopLevelIndentArg::Preserve)]
        top_level_indent: TopLevelIndentArg,
    },
    /// Speak the Language Server Protocol over stdio (003-lsp-vscode-extension
    /// FR-001) — no flags; launchable by an LSP client with no configuration
    /// beyond pointing it at this binary.
    Server,
    /// Speak the Model Context Protocol over stdio (004-mcp-server FR-001)
    /// — no flags; launchable by any MCP-capable client with no
    /// configuration beyond pointing it at this binary. Exposes four
    /// read-only tools (diagnose/format/query_structure/lookup_keyword)
    /// over `voyager-core`, entirely independent of `Server` above (no
    /// shared state, no dependency on a running LSP session, FR-011).
    Mcp,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TopLevelIndentArg {
    Preserve,
    Normalize,
}

// The CasingArg -> voyager_core::CasingConvention and TopLevelIndentArg ->
// voyager_core::TopLevelIndentMode conversions live in format_cmd.rs (added
// alongside the voyager-core format module) rather than here, so this
// Foundational-phase module has no dependency on those not-yet-built types.
