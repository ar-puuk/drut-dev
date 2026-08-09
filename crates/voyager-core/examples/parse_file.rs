//! Manual spot-check: reads a `.s`/`.block` file's path from the command
//! line, calls `voyager_core::parse()`, and prints the resulting top-level
//! node count and any diagnostics.
//!
//! All file I/O happens here, in the example — `voyager_core` itself never
//! touches the filesystem (FR-001).
//!
//! Usage: `cargo run -p voyager-core --example parse_file -- path\to\some.s`

use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args();
    let _program = args.next();
    let Some(path) = args.next() else {
        eprintln!("usage: parse_file <path-to-.s-or-.block-file>");
        return ExitCode::FAILURE;
    };

    let source = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("could not read {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let result = voyager_core::parse(&source);
    println!(
        "{path}: {} top-level node(s), {} diagnostic(s)",
        result.nodes.len(),
        result.diagnostics.len()
    );
    for diag in &result.diagnostics {
        println!(
            "  {:?} at {}:{} — {}",
            diag.kind, diag.span.start.line, diag.span.start.column, diag.message
        );
    }

    if result.diagnostics.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
