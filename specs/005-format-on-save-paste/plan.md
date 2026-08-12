# Implementation Plan: Format-On-Save and Format-On-Paste

**Branch**: `005-format-on-save-paste` | **Date**: 2026-08-11 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/005-format-on-save-paste/spec.md`

## Summary

Two independent adapter-only additions, both reusing `voyager_core::format`
unchanged (constitution Principle I — no new formatting/grammar logic
anywhere in this feature).

**Format-on-save** (US1, P1): pure client-side wiring. `drut-lsp` already
declares `document_formatting_provider` and answers
`textDocument/formatting` (`crates/drut-lsp/src/formatting.rs`, shipped in
003). This feature adds a one-time, workspace-scoped auto-injection of
`editor.formatOnSave` for `.s`/`.block` files in `editors/vscode`, using VS
Code's genuine language-override configuration API (research.md §3) — not
the ad hoc `"[languageId]"`-object-merge trick 003's semantic-token-color
injection used, which doesn't apply to a plain boolean setting the way it
does to a rule-keyed customization map.

**Format-on-paste** (US2, P2): a real new `drut-lsp` capability —
`textDocument/rangeFormatting`, backed by the same `voyager_core::format`
call as whole-document formatting. Since `voyager-core`'s formatter only
ever rewrites a line's *leading* whitespace and never inserts, removes, or
reorders lines (`format.rs`'s own documented scope), the requested range is
served by running a full-document format internally, diffing line-by-line
against the original, and returning only the edits whose line falls within
the requested range (research.md §2 — the strategy spec.md's FR-003 left
open for this plan to resolve). Ships opt-in only, documented in the
README, per Clarification Q1's Option C — the extension never auto-enables
it.

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition (`drut-lsp`) +
TypeScript (`editors/vscode`) — both already-established, unchanged by this
feature.

**Primary Dependencies**:
- `lsp-types` (existing `drut-lsp` dependency, already pinned) — already
  defines `lsp_types::request::RangeFormatting`,
  `DocumentRangeFormattingParams`, and
  `ServerCapabilities::document_range_formatting_provider` (confirmed
  directly against the vendored `lsp-types-0.97.0` source, research.md §1).
  **Zero new crate dependency** for the range-formatting capability.
- `vscode` extension API (existing `editors/vscode` dependency, already
  pinned) — `WorkspaceConfiguration.update`'s fourth parameter
  (`overrideInLanguage: boolean`), used together with
  `getConfiguration(undefined, { languageId })`, is the confirmed-working
  mechanism for writing a genuine language-scoped setting override
  (research.md §3). **Zero new npm dependency.**
- `voyager_core::format`/`FormatOptions` (existing, unchanged) — the only
  formatting logic either new entry point calls.

**Storage**: N/A for `drut-lsp` (stateless per request, same as every other
capability). One new `ExtensionContext.workspaceState` boolean key in
`editors/vscode` (`drutFormatOnSaveInjected`), mirroring 003's existing
`drutVariableColorInjected` key exactly in shape and lifecycle.

**Testing**:
- `cargo test -p drut-lsp` — new `range_formatting.rs` module tests
  (mirroring `formatting.rs`'s existing test shape: misindented-body case,
  already-formatted-returns-no-edits case, unopened-document-returns-none
  case) plus range-boundary cases specific to this capability (edit outside
  the requested range is never returned; a paste that straddles a block
  boundary only gets the in-range portion corrected, per FR-003).
- `cargo test -p drut-lsp --test diagnostics_corpus -- --ignored` (existing,
  re-run unchanged) — proves this feature didn't regress anything already
  covered.
- `editors/vscode`'s existing `npm test` (grammar tokenization) is
  unaffected — this feature touches `extension.ts` and `package.json`
  only, no grammar file.
- Manual VS Code verification (quickstart.md steps 5–6) — same standard
  003 and 004 both held themselves to for anything touching real editor
  UI/settings behavior, since no automated harness drives the actual
  paste/save UI gesture.

**Target Platform**: Cross-platform (Windows/macOS/Linux), unchanged.

**Project Type**: Adapter-only additions to two existing components
(`drut-lsp` library, `editors/vscode` extension) — no new crate, no new
`voyager-core` entry point, no new npm package.

**Performance Goals**: Same "perceptibly-immediate" bar
`003-lsp-vscode-extension/plan.md` set for every other LSP response — a
range-formatting call does one whole-document `voyager_core::format` call
plus one linear line-by-line diff, the same order of magnitude as the
already-shipped whole-document formatting call.

**Constraints**:
- MUST NOT duplicate any formatting/grammar logic outside `voyager-core`
  (Principle I) — both new/reused entry points call `voyager_core::format`
  exactly as `formatting.rs` already does.
- MUST NOT write to disk from `drut-lsp` under any circumstance — both
  capabilities return `TextEdit`s for the client to apply, same as the
  existing whole-document formatting handler.
- MUST NOT alter program meaning (Principle III) — range-formatting reuses
  the whole-document formatter's own idempotence/behavior-preservation
  guarantee rather than re-deriving it (FR-008).
- MUST NOT silently re-enable a setting the user has explicitly turned off
  (FR-006) — the workspace-state one-time gate is the enforcement
  mechanism, same shape as 003's existing precedent.
- MUST NOT auto-enable format-on-paste (FR-005, Clarification Q1) — this is
  a hard behavioral difference from format-on-save, not a shared code path.

**Scale/Scope**: Single-document, single-request scope for
range-formatting, matching `formatting.rs`'s own whole-document scope — no
multi-file or workspace-wide behavior introduced.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | Both new/reused LSP entry points call `voyager_core::format` unchanged — no formatting/grammar logic is added to `drut-lsp` or `editors/vscode`. The line-diffing needed to serve a *range* result (research.md §2) is presentation/adapter logic (which of an already-correct whole-document format's line changes to report back), not a second formatting decision — it makes no judgment `voyager-core` didn't already make. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | No new keyword lists, grammar rules, or hover/help text — this feature adds no user-facing text beyond a README opt-in instruction and code comments, both original wording. |
| III. Formatter Idempotence & Behavior Preservation | **PASS, unchanged** | No formatter logic changes; both entry points inherit `voyager_core::format`'s existing, already-proven idempotence guarantee (FR-008) rather than re-implementing or re-verifying it independently. |
| IV. False Negatives Over False Positives | **PASS** | The edge case where a paste's structural context extends outside the requested range (spec.md Edge Cases) is handled by scoping the returned edit set to the requested range rather than guessing at a wider correction — consistent with this principle's preference for doing less over doing something wrong. |
| V. Vertical, Independently-Usable Increments | **PASS** | US1 (format-on-save) is fully independently valuable and testable with zero dependency on US2; US2 (format-on-paste) is additive on top. This phase does not start until `004-mcp-server`'s fixture-corpus tests pass cleanly (already true, merged to `main`). |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **PASS** | `textDocument/rangeFormatting` is the LSP-standard mechanism VS Code's own `editor.formatOnPaste` is built on (research.md §1) — this feature explicitly avoids any editor-proprietary paste-hook API (e.g. a custom `DocumentPasteEditProvider`) in favor of the standard one, the same principle this project already held itself to in 003. |
| VII. Naming Honesty | **PASS** | No new named capability that overclaims — "format-on-save"/"format-on-paste" describe exactly what each does, nothing more. |
| VIII. Public/Private Boundary | **PASS** | Both changes land in already-public components (`drut-lsp`, `editors/vscode`); no vendor-documentation-corpus content involved. |

No unjustified violations. No Complexity Tracking entries — this feature
adds zero new dependencies (crate or npm) and zero new architectural
components.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/
quickstart.md): `contracts/range-formatting-api.md`'s line-diff algorithm
and `contracts/extension-settings.md`'s injection mechanism both confirm
the Principle I/VI rows above hold precisely as described — no row's status
changed from the pre-design check.

## Project Structure

### Documentation (this feature)

```text
specs/005-format-on-save-paste/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output (/speckit-plan command)
├── data-model.md                  # Phase 1 output (/speckit-plan command)
├── quickstart.md                  # Phase 1 output (/speckit-plan command)
├── contracts/                     # Phase 1 output (/speckit-plan command)
│   ├── range-formatting-api.md      # new drut-lsp textDocument/rangeFormatting
│   └── extension-settings.md        # format-on-save injection + format-on-paste opt-in doc
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/
└── drut-lsp/                       # existing crate; this feature ADDS to it
    └── src/
        ├── lib.rs                     # add: document_range_formatting_provider
        │                                # capability + RangeFormatting request
        │                                # dispatch, alongside the existing
        │                                # Formatting arm
        ├── formatting.rs               # unchanged — whole-document formatting
        │                                # stays exactly as 003 shipped it
        └── range_formatting.rs         # NEW: textDocument/rangeFormatting
                                         # handler — runs voyager_core::format,
                                         # line-diffs against the original,
                                         # filters to the requested range
                                         # (contracts/range-formatting-api.md)

editors/
└── vscode/                         # existing extension; this feature ADDS to it
    ├── package.json                  # add: a documented, off-by-default
    │                                # `editor.formatOnPaste` mention in
    │                                # README only (no package.json
    │                                # configuration contribution needed —
    │                                # formatOnPaste is a built-in VS Code
    │                                # setting, not one this extension
    │                                # defines)
    └── src/
        └── extension.ts               # add: ensureFormatOnSaveEnabled,
                                         # same shape as the existing
                                         # ensureVariableColorCustomization,
                                         # called from activate()
```

**Structure Decision**: No new crate, no new npm package, no new workspace
member. `drut-lsp` gains one new module (`range_formatting.rs`), mirroring
how `formatting.rs` itself was added directly to `drut-lsp` rather than
anywhere else when it shipped mid-003. `editors/vscode` gains one new
function in its existing single-file `extension.ts`, mirroring
`ensureVariableColorCustomization`'s own placement and shape exactly — no
new file needed for one function of comparable size.

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new
dependencies.*
