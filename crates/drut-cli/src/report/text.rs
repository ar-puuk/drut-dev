//! Plain-text diagnostic rendering (spec.md FR-008) — the default `check`
//! output mode.

use crate::check_cmd::CheckReport;
use crate::io_util::write_stdout_line;

/// One line per diagnostic (minimum): file path, location, kind, message
/// (FR-008). Zero diagnostics prints nothing — the "clean" outcome is
/// distinguished by exit code alone (SC-006), not by explicit "0 diagnostics"
/// text.
pub fn print_check_report(report: &CheckReport) {
    for (path, diag) in &report.diagnostics {
        write_stdout_line(&format!(
            "{}:{}:{}: {:?}: {}",
            path.display(),
            diag.span.start.line,
            diag.span.start.column,
            diag.kind,
            diag.message
        ));
    }
}
