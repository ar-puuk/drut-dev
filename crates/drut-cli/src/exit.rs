//! Shared three-way exit-code convention (spec.md FR-011, FR-020;
//! data-model.md §6) — both `check` and `format` report exactly one of these
//! via process exit code, so a caller only has to learn one convention
//! (SC-006).

/// One of the three run-wide outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitOutcome {
    /// Nothing to report — exit `0`.
    Clean,
    /// The run completed but found something to report (diagnostics for
    /// `check`; files needing formatting for `format --check`) — exit `1`.
    ProblemsFound,
    /// The run itself could not (safely) complete for at least one target
    /// (bad path, unreadable/unwritable file, or — for `format` — a file
    /// refused for writing under FR-025) — exit `2`. Always wins over
    /// `ProblemsFound` when both would otherwise apply.
    Fatal,
}

impl ExitOutcome {
    pub fn code(self) -> i32 {
        match self {
            ExitOutcome::Clean => 0,
            ExitOutcome::ProblemsFound => 1,
            ExitOutcome::Fatal => 2,
        }
    }

    /// Combines two independently-derived outcomes under the
    /// `Fatal`-always-wins precedence rule (FR-011, FR-020).
    pub fn combine(self, other: ExitOutcome) -> ExitOutcome {
        use ExitOutcome::*;
        match (self, other) {
            (Fatal, _) | (_, Fatal) => Fatal,
            (ProblemsFound, _) | (_, ProblemsFound) => ProblemsFound,
            (Clean, Clean) => Clean,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fatal_always_wins() {
        assert_eq!(ExitOutcome::Fatal.combine(ExitOutcome::Clean), ExitOutcome::Fatal);
        assert_eq!(
            ExitOutcome::ProblemsFound.combine(ExitOutcome::Fatal),
            ExitOutcome::Fatal
        );
    }

    #[test]
    fn problems_found_beats_clean() {
        assert_eq!(
            ExitOutcome::Clean.combine(ExitOutcome::ProblemsFound),
            ExitOutcome::ProblemsFound
        );
    }

    #[test]
    fn codes_match_convention() {
        assert_eq!(ExitOutcome::Clean.code(), 0);
        assert_eq!(ExitOutcome::ProblemsFound.code(), 1);
        assert_eq!(ExitOutcome::Fatal.code(), 2);
    }
}
