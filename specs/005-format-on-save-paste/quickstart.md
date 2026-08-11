# Quickstart: Validating Format-On-Save and Format-On-Paste

A runnable validation guide, not an implementation walkthrough — proves
this feature against spec.md's Success Criteria. See
`contracts/range-formatting-api.md` for the new LSP capability's exact
behavior and `contracts/extension-settings.md` for the injection/opt-in
mechanism. Steps 1–4 are automated; steps 5–6 are manual VS Code
verification, the same standard 003 and 004 both held themselves to for
anything touching real editor UI/settings behavior.

## Prerequisites

- Rust stable toolchain and Node.js/npm, matching the rest of the
  workspace's existing requirements.
- The workspace builds: `cargo build --workspace` from repo root.
- VS Code (or another Open VSX-compatible editor), for steps 5–6 only.

## 1. Build

```powershell
cargo build -p voyager-core -p drut-cli -p drut-lsp
```

Expected: builds cleanly, zero `cargo clippy -p drut-lsp --all-targets`
warnings.

## 2. `range_formatting` module — validates FR-001–FR-003, FR-008, FR-009

```powershell
cargo test -p drut-lsp range_formatting::
```

Expected: all green — including
`change_outside_requested_range_is_not_returned` and
`change_at_exact_range_boundary_is_included`
(`contracts/range-formatting-api.md`'s two range-boundary-specific cases,
proving FR-003's "only the portion within the range" scope precisely).

## 3. Existing whole-document `formatting` module — validates no regression

```powershell
cargo test -p drut-lsp formatting::
```

Expected: unchanged, all green — this feature does not modify
`formatting.rs`.

## 4. Full-corpus diagnostic parity — validates no regression to anything already covered

```powershell
$env:DRUT_CORPUS_PATH = "path\to\WF-TDM-Official-Releases"
cargo test -p drut-lsp --test diagnostics_corpus -- --ignored
```

Expected: unchanged — still 161/161 clean, same as every prior phase.
This feature adds no new diagnostic behavior; this step exists purely to
confirm the new capability's addition to `lib.rs`'s request-dispatch
`match` didn't disturb anything else routed through it.

## 5. Manual smoke test — format-on-save auto-enables (validates SC-002, SC-004, US1, US3)

1. Package and install the extension in a clean VS Code profile with
   `drut` on `PATH`:
   ```powershell
   cd editors\vscode
   npm install && npm run compile && npx @vscode/vsce package
   ```
   Install the resulting `.vsix`.
2. Open a fresh workspace folder (no prior `.vscode/settings.json`)
   containing a `.s` file with a misindented body statement inside a
   block.
3. Save the file (Ctrl+S) **without** running "Format Document" first.

Expected: the misindentation is corrected automatically on save — no
manual formatting action taken. Inspect `.vscode/settings.json`: it now
contains `"[drut-voyager]": { "editor.formatOnSave": true }` (SC-002).

4. Set `"[drut-voyager]": { "editor.formatOnSave": false }` yourself in
   that same `settings.json`, then close and reopen the workspace.
5. Introduce a new misindentation and save again.

Expected: the file is **not** auto-corrected this time — the extension
respected the explicit override and did not silently re-enable it (SC-004,
US3's acceptance scenario).

## 6. Manual smoke test — format-on-paste stays opt-in until enabled (validates SC-001, SC-003, US2)

1. In a fresh workspace (different from step 5, to avoid its now-disabled
   override), open a `.s` file with an `IF` block.
2. Copy a two-line fragment with indentation that doesn't match the
   target block's depth (e.g. copy from a top-level position, paste one
   level deep).
3. Paste it into the block, **before** turning on `formatOnPaste`.

Expected: the pasted content keeps its original (now-wrong) indentation —
format-on-paste does nothing until the user opts in (Clarification Q1
confirmed at the UI level).

4. Add `"[drut-voyager]": { "editor.formatOnPaste": true }` to
   `.vscode/settings.json` by hand, following this feature's own README
   instruction (`contracts/extension-settings.md`).
5. Paste the same fragment again.

Expected: the pasted lines are reindented to match the block they landed
in, immediately after the paste completes (SC-001). Paste
already-correctly-indented content a third time: expected no further edit
(SC-003 — idempotence).

## 7. Full test suite

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets
```

Expected: all green, zero clippy warnings.

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 6 | SC-001, SC-003 |
| 5 | SC-002, SC-004 |
| 2 | FR-001–FR-003, FR-008, FR-009 |
