//! `drut`: thin CLI adapter over `voyager-core` (constitution Principle I) —
//! `check` and `format` subcommands. No grammar/parsing/formatting-decision
//! logic lives here; this crate only does I/O, traversal, and output
//! rendering (see specs/002-cli-check-format/plan.md). See `lib.rs` for the
//! actual implementation — this binary is a thin wrapper so integration
//! tests can reach the same code.

fn main() {
    std::process::exit(drut_cli::run());
}
