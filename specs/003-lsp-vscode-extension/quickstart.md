# Quickstart: Validating the Drut LSP Server & VS Code/Open VSX Extension

A runnable validation guide, not an implementation walkthrough — proves this
feature against spec.md's Success Criteria. See `contracts/lsp-capabilities.md`
for the full protocol surface, `contracts/position-encoding.md` for the UTF-16
translation contract, and `data-model.md` for the types referenced below.
Deep automated VS Code UI testing (driving the actual editor end-to-end) is out
of scope this phase — the LSP-level test harness (steps 2–6) is the authoritative
correctness proof (FR-028), and steps 7–9 are a human-run smoke test of the
packaged extension itself, the one thing the LSP-level harness can't verify
(that `drut server` actually launches correctly from within VS Code).

## Prerequisites

- Rust stable toolchain and Node.js/npm (current stable), matching
  `voyager-core`'s and `vscode-languageclient`'s own requirements.
- The workspace builds: `cargo build --workspace` from repo root.
- A local checkout of the WF-TDM-Official-Releases corpus, available the same
  way it already is for `voyager-core`'s and `drut-cli`'s own full-corpus
  validation (`001-voyager-script-parser/research.md` §3), referred to below as
  `$CORPUS`.
- VS Code (or another Open VSX-compatible editor) installed, for steps 7–9 only.

## 1. Build

```powershell
cargo build -p voyager-core -p drut-cli -p drut-lsp
```

Expected: builds cleanly, zero `cargo clippy -p drut-lsp --all-targets`
warnings, matching the zero-warning bar already held for `voyager-core`/
`drut-cli`.

## 2. `keywords` module — dictionary and fuzzy match (`voyager-core::keywords`)

```powershell
cargo test -p voyager-core keywords::
```

Expected: all green — dictionary lookup, `completion_candidates`' general vs.
context-scoped behavior (`contracts/keyword-dictionary-api.md`), and
`did_you_mean`'s unique-minimum-within-2 rule (research.md §5) all pass.

## 3. LSP protocol smoke test — validates the server actually speaks LSP

```powershell
cargo test -p drut-lsp --test protocol_smoke
```

Expected: a real `initialize`/`initialized`/`textDocument/didOpen` round trip
over `lsp_server::Connection::memory()` (research.md §9) succeeds, and the
`initialize` response's `capabilities.position_encoding` is `"utf-16"`
(`contracts/position-encoding.md`'s fixed-constant guarantee).

## 4. Full-corpus diagnostic parity — validates SC-002, FR-028 (Definition of Done)

```powershell
$env:DRUT_CORPUS_PATH = "$CORPUS"
cargo test -p drut-lsp --test diagnostics_corpus -- --ignored
```

Expected: every valid corpus file, opened via `textDocument/didOpen` over the
in-memory connection, publishes zero diagnostics; every deliberately-broken
fixture publishes a diagnostic correctly identifying its injected defect —
reproducing, at the LSP protocol layer, the same 161/161-clean result already
proven for `voyager-core` (library) and `drut-cli` (CLI). This is the LSP-level
equivalent of `002-cli-check-format/quickstart.md` step 2.

## 5. Hover, completion, spell-check, semantic tokens — validates SC-004, SC-005 (partially), SC-006

```powershell
cargo test -p drut-lsp --test hover
cargo test -p drut-lsp --test completion
cargo test -p drut-lsp --test spellcheck
cargo test -p drut-lsp --test semantic_tokens
```

Expected: all green — each suite exercises its user story's Acceptance
Scenarios directly (hover's implicit-Run/Process-close case, completion's
context-scoped vs. general-fallback split from research.md §2, spell-check's
unique-match/no-match/exact-match cases, semantic tokens' short-IF and
unreachable-after-`BREAK` cases).

## 6. Position-encoding correctness on supplementary-plane characters — validates SC-005

```powershell
cargo test -p drut-lsp --test position_encoding
```

Expected: a fixture containing a supplementary-plane character (e.g. inside a
comment) produces diagnostic/hover/semantic-token positions that land on the
correct character when interpreted as UTF-16 code units by a real LSP-position
consumer — not shifted by the char-count-vs-UTF-16 discrepancy this feature was
required to resolve (`contracts/position-encoding.md`).

## 7. Extension packaging — validates FR-021, FR-027 (build side)

```powershell
cd editors\vscode
npm install
npm run compile
npx @vscode/vsce package
```

Expected: a `.vsix` package builds with no errors; `npx @vscode/vsce ls`
(or equivalent) shows `syntaxes/drut.tmLanguage.json` and
`language-configuration.json` included.

## 8. Manual smoke test — static highlighting with no server (validates SC-001, Story 1)

1. Install the packaged `.vsix` in a clean VS Code profile with no `drut`
   binary on `PATH`.
2. Open a `.s` file from `$CORPUS` containing a nested block comment.

Expected: control words, comments (including the nested region), strings, and
`@variable@` substitutions render in visually distinct colors — matching
spec.md's own Story 1 Independent Test exactly. This step alone proves
nothing about FR-025's missing-server notification (see step 9) — that
behavior's implementing code (the extension's `LanguageClient` bootstrap)
isn't exercised by this step, only the static grammar/language-configuration
(corrected 2026-08-09, `/speckit-analyze` finding F1).

## 9. Manual smoke test — live diagnostics with the server running, and the missing-server notice (validates SC-003, FR-025)

1. With `drut` on `PATH` (from step 1's build), reopen VS Code so the
   extension can find and launch `drut server`.
2. Open a valid `.s` file from `$CORPUS`; confirm no diagnostics appear.
3. Introduce an unmatched `IF`; confirm a diagnostic appears without saving.
4. Undo the change; confirm the diagnostic disappears without reopening the
   file.
5. Remove `drut` from `PATH` and reload the window; confirm a single,
   non-repeating notification appears about the missing server (FR-025) — no
   repeated popups, and highlighting from step 8 stays intact. Restore
   `PATH` afterward.

Expected: diagnostics track the live buffer, appearing/disappearing with a
perceptibly-immediate delay, matching spec.md SC-003's bar; the missing-binary
case degrades exactly as FR-025 requires.

## 10. Full test suite

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets
```

Expected: all green, zero clippy warnings across `voyager-core`, `drut-cli`, and
`drut-lsp` — the actual CI gate steps 1–6's manual runs are a human-readable
proxy for.

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 8 | SC-001 |
| 4 | SC-002 |
| 9 | SC-003 |
| 5 | SC-004, SC-006 |
| 5, 6 | SC-005 |
| 7, 8 | SC-007 |
| 3, 4 | SC-008 |
