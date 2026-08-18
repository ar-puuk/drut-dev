//! `textDocument/formatting` — added 2026-08-10 during manual VS Code
//! verification (see `specs/003-lsp-vscode-extension/spec.md`'s dated
//! Assumptions entry for the full rationale: this capability didn't exist
//! in the original phase scope, but "Format Document"/format-on-save not
//! working at all, with `voyager_core::format` already fully built and
//! tested by `002-cli-check-format`, was a real, concrete gap surfaced by
//! hands-on testing, not a hypothetical).
//!
//! Thin wrapper over `voyager_core::format` (Principle I) — no
//! whitespace/casing logic lives here. Always returns a single `TextEdit`
//! spanning the whole document (never a set of minimal diffs): simplest to
//! reason about, and correct regardless of how much or little changed,
//! since `voyager_core::format` itself already guarantees idempotence
//! (`002-cli-check-format`'s own golden-fixture corpus) — reformatting an
//! already-formatted document is always a safe no-op edit.

use crate::document_store::ServerState;
use crate::position::to_lsp_range;
use crate::workspace::resolve_path;

/// Handles a `textDocument/formatting` request.
///
/// Casing/top-level-indent settings are resolved via `drut_config::
/// resolve_format_options` (012-toml-configuration) — a `drut.toml` found
/// from the document's own real path (falling back to the client's
/// workspace root) drives these settings; with no `drut.toml` anywhere,
/// behavior is unchanged from before that feature (built-in defaults).
/// `isolated` is always `false` here — no per-request LSP isolation
/// mechanism exists (contracts/toml-config-api.md's explicit non-goal).
pub fn handle(
    state: &ServerState,
    params: &lsp_types::DocumentFormattingParams,
) -> Option<Vec<lsp_types::TextEdit>> {
    let uri = &params.text_document.uri;
    let doc = state.get(uri)?;

    let (options, _warnings) = drut_config::resolve_format_options(
        resolve_path(uri, state).as_deref(),
        false,
        drut_config::ExplicitFormatOverride::default(),
        // 021-editor-settings-config T009: the real cached client-settings
        // value, populated (if at all) by the `workspace/configuration`
        // pull in lib.rs — `Default` (every field `None`) before the first
        // successful pull completes, self-correcting once it does.
        state.client_format_defaults(),
    );
    let result = voyager_core::format(&doc.text, options);
    if !result.changed {
        // Already formatted -- an empty edit list, not `None` (`None` would
        // mean "this document has no formatter opinion at all", which isn't
        // true here; there's just nothing left to change).
        return Some(Vec::new());
    }

    let range = to_lsp_range(&doc.text, whole_document_span(&doc.text));
    Some(vec![lsp_types::TextEdit {
        range,
        new_text: result.text,
    }])
}

/// The `voyager-core` `Span` covering all of `text`, start to end — built by
/// walking every char once (1-based line/column, matching `Span`'s own
/// convention, `end` one past the last char). Deliberately local to this
/// module rather than a sentinel-position trick (e.g. `Position::MAX`)
/// through `position.rs`'s existing clamping: that clamping only clamps a
/// requested *column* to its line's real length, it does not clamp an
/// out-of-range *line* down to the document's real last line (verified
/// directly against `position.rs`'s own `out_of_range_line_clamps_rather_
/// than_panicking` test) — a huge sentinel line number would silently
/// survive translation as a bogus, too-large `Range`, not a safely-clamped
/// one.
fn whole_document_span(text: &str) -> voyager_core::Span {
    use voyager_core::Position;

    let mut line = 1u32;
    let mut column = 1u32;
    for c in text.chars() {
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    voyager_core::Span::new(Position::new(1, 1), Position::new(line, column))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn params(uri: &str) -> lsp_types::DocumentFormattingParams {
        lsp_types::DocumentFormattingParams {
            text_document: lsp_types::TextDocumentIdentifier {
                uri: lsp_types::Uri::from_str(uri).unwrap(),
            },
            options: lsp_types::FormattingOptions {
                tab_size: 4,
                insert_spaces: true,
                ..Default::default()
            },
            work_done_progress_params: Default::default(),
        }
    }

    #[test]
    fn misindented_body_statement_is_corrected_relative_to_its_opener() {
        // format.rs's own documented design (FR-012): a *top-level*
        // statement's own indentation is deliberately left untouched (see
        // format.rs's `plan_indentation` doc comment) -- only a nested
        // child's indentation is normalized, relative to its block's own
        // (possibly-untouched) opener line. `PRINT` here is wrongly flush
        // with `IF` instead of one level in.
        let mut state = ServerState::new();
        state.did_open(
            lsp_types::Uri::from_str("file:///a.s").unwrap(),
            "IF (a=b)\nPRINT LIST=1\nENDIF\n".to_string(),
            1,
        );
        let edits = handle(&state, &params("file:///a.s")).unwrap();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].new_text, "IF (a=b)\n    PRINT LIST=1\nENDIF\n");
    }

    #[test]
    fn already_formatted_document_returns_no_edits() {
        let mut state = ServerState::new();
        let text = "IF (a=b)\n    PRINT LIST=1\nENDIF\n".to_string();
        state.did_open(lsp_types::Uri::from_str("file:///a.s").unwrap(), text, 1);
        let edits = handle(&state, &params("file:///a.s")).unwrap();
        assert!(edits.is_empty(), "expected no edits for an already-formatted document, got {edits:?}");
    }

    #[test]
    fn non_zero_top_level_indentation_is_left_untouched_by_default() {
        // 009-top-level-indent-toggle FR-004(c)/User Story 3: no compiler
        // forcing function exists for this call site (it's a bare
        // FormatOptions::default(), not a struct literal) -- confirmed
        // directly rather than inferred from any other adapter's own test
        // passing.
        let mut state = ServerState::new();
        let text = "    IF (a=b)\n        PRINT LIST=1\n    ENDIF\n".to_string();
        state.did_open(lsp_types::Uri::from_str("file:///a.s").unwrap(), text.clone(), 1);
        let edits = handle(&state, &params("file:///a.s")).unwrap();
        assert!(edits.is_empty(), "non-zero top-level indentation must be left untouched by default, got {edits:?}");
    }

    #[test]
    fn protected_range_survives_textdocument_formatting() {
        // 010-fmt-region-markers FR-007/US3: protection is inherited from
        // voyager-core with no code change to this handler -- confirmed
        // directly through the real handle() function, not inferred.
        let mut state = ServerState::new();
        let text = "IF (a=b)\nY = 1\n; FMT: OFF\n  weird = 1\n; FMT: ON\nZ = 2\nENDIF\n".to_string();
        state.did_open(lsp_types::Uri::from_str("file:///a.s").unwrap(), text, 1);
        let edits = handle(&state, &params("file:///a.s")).unwrap();
        assert_eq!(edits.len(), 1);
        assert_eq!(
            edits[0].new_text,
            "IF (a=b)\n    Y = 1\n; FMT: OFF\n  weird = 1\n; FMT: ON\n    Z = 2\nENDIF\n"
        );
    }

    #[test]
    fn unopened_document_returns_none() {
        let state = ServerState::new();
        assert!(handle(&state, &params("file:///never-opened.s")).is_none());
    }

    // -- 012-toml-configuration (T021) ---------------------------------------

    fn file_uri(path: &std::path::Path) -> lsp_types::Uri {
        let s = path.to_string_lossy().replace('\\', "/");
        let s = if s.starts_with('/') { s } else { format!("/{s}") };
        lsp_types::Uri::from_str(&format!("file://{s}")).unwrap()
    }

    fn temp_project(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("drut_lsp_formatting_test_{}_{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn document_under_a_drut_toml_governed_directory_picks_up_its_settings() {
        let dir = temp_project("governed");
        std::fs::write(dir.join("drut.toml"), "[format]\ncontrol_words_casing = \"upper\"\n").unwrap();
        let file = dir.join("a.s");
        let uri = file_uri(&file);
        let uri_str = uri.as_str().to_string();

        let mut state = ServerState::new();
        state.did_open(uri, "if (a=b)\nendif\n".to_string(), 1);
        let edits = handle(&state, &params(&uri_str)).unwrap();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].new_text, "IF (a=b)\nENDIF\n");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn untitled_document_with_no_workspace_root_uses_built_in_defaults() {
        let mut state = ServerState::new();
        state.did_open(
            lsp_types::Uri::from_str("untitled:Untitled-1").unwrap(),
            "if (a=b)\nendif\n".to_string(),
            1,
        );
        let edits = handle(&state, &params("untitled:Untitled-1")).unwrap();
        assert!(edits.is_empty(), "no real path, no workspace root -- must resolve to built-in defaults (unchanged)");
    }

    #[test]
    fn untitled_document_falls_back_to_the_workspace_root_s_drut_toml() {
        let dir = temp_project("workspace-root-fallback");
        std::fs::write(dir.join("drut.toml"), "[format]\ncontrol_words_casing = \"upper\"\n").unwrap();

        let mut state = ServerState::new();
        state.set_workspace_root(Some(dir.clone()));
        state.did_open(
            lsp_types::Uri::from_str("untitled:Untitled-1").unwrap(),
            "if (a=b)\nendif\n".to_string(),
            1,
        );
        let edits = handle(&state, &params("untitled:Untitled-1")).unwrap();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].new_text, "IF (a=b)\nENDIF\n");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn whole_document_span_covers_a_multiline_document() {
        let text = "IF (a=b)\nENDIF\n";
        let span = whole_document_span(text);
        assert_eq!(span.start, voyager_core::Position::new(1, 1));
        // Three lines by char-count: "IF (a=b)\n", "ENDIF\n", "" (the empty
        // tail after the final newline) -- line 3, column 1.
        assert_eq!(span.end, voyager_core::Position::new(3, 1));
    }
}
