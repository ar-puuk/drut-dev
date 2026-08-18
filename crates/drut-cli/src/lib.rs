//! `drut-cli` library surface — exists so `tests/*.rs` integration tests can
//! reach the same modules `main.rs` uses, per the standard testable-binary
//! pattern (a binary-only crate's modules are private to it). `main.rs` is a
//! thin wrapper over [`run`].

pub mod check_cmd;
pub mod cli;
pub mod exit;
pub mod format_cmd;
pub mod io_util;
pub mod mcp_cmd;
pub mod report;
pub mod server_cmd;
pub mod traverse;

use clap::Parser;

use cli::{Cli, Command};

/// Parses `std::env::args()`, dispatches to the selected subcommand, and
/// returns the process exit code to use (spec.md FR-011, FR-020).
pub fn run() -> i32 {
    let cli = Cli::parse();
    let outcome = match cli.command {
        Command::Check { path, format } => check_cmd::run(&path, format),
        Command::Format {
            path,
            write,
            check,
            diff,
            casing_control_words,
            casing_pair_keywords,
            casing_data_references,
            indent_width,
            indent_top_level,
            operator_spacing,
            blank_lines,
            blank_lines_top_cap,
            blank_lines_nested_cap,
            isolated,
        } => format_cmd::run(
            &path,
            write,
            check,
            diff,
            casing_control_words,
            casing_pair_keywords,
            casing_data_references,
            indent_width,
            indent_top_level,
            operator_spacing,
            blank_lines,
            blank_lines_top_cap,
            blank_lines_nested_cap,
            isolated,
        ),
        Command::Server => return server_cmd::run(),
        Command::Mcp => return mcp_cmd::run(),
    };
    outcome.code()
}
