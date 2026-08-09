//! `format` subcommand orchestration (spec.md FR-012–FR-025; data-model.md §5).
//! Filled in during User Story 2 — see specs/002-cli-check-format/tasks.md.

use std::path::Path;

use crate::cli::CasingArg;
use crate::exit::ExitOutcome;

#[allow(unused_variables, clippy::fn_params_excessive_bools)]
pub fn run(path: &Path, write: bool, check: bool, diff: bool, casing: Option<CasingArg>) -> ExitOutcome {
    ExitOutcome::Clean
}
