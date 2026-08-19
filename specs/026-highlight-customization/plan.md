# Implementation Plan: Editor Highlight Color Customization

**Branch**: `026-highlight-customization` | **Date**: 2026-08-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/026-highlight-customization/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Adds 9 `drut.highlight.<category>` VS Code settings (`controlWords`, `statementWords`,
`functionCalls`, `pairKeywords`, `values`, `numbers`, `operators`, `comments`,
`strings`), each an optional color string. The extension keeps VS Code's native
`editor.tokenColorCustomizations` (Global/User scope, `textMateRules`) in sync with
these settings on activation and on every `drut.highlight.*` change — writing a rule for
each configured category, removing any rule for a category left unset, and never
touching any rule the extension didn't itself add. A prerequisite grammar change splits
`support.function.drut` (currently shared by `#statement-words` and `#function-calls`)
into two distinct scopes so those two categories are independently colorable. `@name@`
substitution (`variables`) is explicitly out of scope — an existing, separately-shipped
mechanism (`ensureVariableColorCustomization`, semantic-token-based, workspace-scoped,
one-time) already governs it, and this feature's TextMate-scope-based mechanism would
not visibly win against it.

## Technical Context

**Language/Version**: TypeScript (`editors/vscode`, matching the existing extension's
toolchain) + JSON (`drut.tmLanguage.json` grammar split, `package.json` settings
contributions)

**Primary Dependencies**: None new — uses only the `vscode` extension API
(`workspace.getConfiguration`, `ConfigurationTarget.Global`,
`onDidChangeConfiguration`), already used by `ensureVariableColorCustomization`/
`ensureFormatOnSaveEnabled` for the same class of operation.

**Storage**: N/A (reads/writes VS Code's own User-scope `settings.json` via the
extension API — no drut-owned storage)

**Testing**: `editors/vscode/test/grammar.test.ts` (scope-split regression coverage,
mirrors `024`'s own convention) + a new `test/highlightCustomization.test.ts`
(standalone `ts-node`, zero `vscode` import, mirrors `formatOnSave.test.ts`'s existing
convention for testing pure decision/merge logic without an Extension Development Host)

**Target Platform**: VS Code (this feature is explicitly VS-Code-specific per the
resolved Scope decision — no LSP-standard mechanism applies, since
`editor.tokenColorCustomizations` itself is a VS-Code-proprietary setting, not part of
the LSP spec; Constitution Principle VI's preference for LSP-standard mechanisms was
explicitly weighed and set aside during scoping, since the alternative — semantic tokens
— was rejected for being substantial, duplicate-logic work with no portability benefit
given this is a personal-setting-only feature to begin with)

**Project Type**: Editor extension (thin adapter) — `editors/vscode` only; no
`voyager-core`/`drut-config`/`drut-cli`/`drut-mcp`/`drut-lsp` change (spec.md FR-007/
FR-008)

**Performance Goals**: N/A — one settings read + one conditional settings write, only on
activation and on a `drut.highlight.*` change (not per-keystroke, not per-document-open)

**Constraints**: MUST NOT modify any pre-existing `editor.tokenColorCustomizations`
content it doesn't own (FR-004); MUST NOT touch the existing
`ensureVariableColorCustomization`/`ensureFormatOnSaveEnabled` functions or their
injected settings (FR-008)

**Scale/Scope**: One grammar-file edit (split one scope into two), one new pure-logic
TS module + its test file, one small wiring addition in `extension.ts`, 9 new
`package.json` setting contributions, `grammar.test.ts` additions confirming the split

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Single Source of Truth)**: No grammar/parsing/lint-rule logic is added
  or duplicated — this feature reads/writes VS Code's own settings using already-shipped
  scope names; the category→scope mapping is presentation wiring, not grammar logic.
  **PASS.**
- **Principle II (No Verbatim Vendor Docs)**: N/A — no vendor documentation involved;
  this is pure VS Code extension-API usage.
- **Principle III (Formatter Idempotence & Behavior Preservation)**: N/A — no formatter
  code touched.
- **Principle IV (False Negatives Over False Positives)**: N/A — not a diagnostic/lint
  rule. The closest analogue (never touching a rule this feature doesn't own, FR-004) is
  honored by construction (exact-scope-match ownership test, research.md §3).
- **Principle V (Vertical, Independently-Usable Increments)**: Self-contained; every
  `drut.highlight.*` setting defaults to unset, so a project that doesn't use this
  feature sees zero behavior change (spec.md SC-002).
- **Principle VI (LSP-Standard Mechanisms Over Editor-Proprietary APIs)**: Explicitly
  weighed and set aside (Technical Context above) — the LSP-standard alternative
  (semantic tokens) was rejected during scoping as disproportionate, duplicate-logic
  work for a feature explicitly scoped to VS-Code-personal-settings only. Documented
  here as a deliberate, reasoned exception, not an oversight — the constitution's own
  wording is "wherever ... equivalent," and a live personal color-preference feature
  with no cross-editor portability goal is not a case where the two mechanisms are
  equivalent in cost.
- **Principle VII (Naming Honesty)**: `drut.highlight.*` is named for exactly what it
  does (per-category highlight color). **PASS.**
- **Principle VIII (Public/Private Boundary)**: N/A — no vendor-documentation-derived
  content.

**One violation, justified above (Principle VI)** — recorded in Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/026-highlight-customization/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
editors/vscode/
├── syntaxes/
│   └── drut.tmLanguage.json      # split support.function.drut into
│                                  # support.function.statement.drut /
│                                  # support.function.builtin.drut
├── src/
│   ├── highlightCustomization.ts # NEW -- pure category<->scope table + the
│   │                              # textMateRules merge/removal logic, zero
│   │                              # `vscode` import (mirrors
│   │                              # formatOnSaveDecision.ts's convention)
│   └── extension.ts               # NEW ensureHighlightCustomization() effectful
│                                   # wrapper (mirrors ensureVariableColorCustomization's
│                                   # shape) + onDidChangeConfiguration registration;
│                                   # called from activate()
├── test/
│   ├── grammar.test.ts            # add scope-split regression checks
│   └── highlightCustomization.test.ts  # NEW -- standalone ts-node tests for the
│                                        # pure merge/removal logic
└── package.json                   # 9 new drut.highlight.* setting contributions
```

**Structure Decision**: Single existing project (`editors/vscode`). No new crate, no
`voyager-core`/adapter-crate change (Constitution Principle I; FR-007/FR-008).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|---------------------------------------|
| Principle VI: uses a VS-Code-proprietary setting (`editor.tokenColorCustomizations`) rather than an LSP-standard mechanism | The feature is explicitly scoped to a personal VS Code setting (resolved Scope decision) — there is no cross-editor portability goal to justify the LSP-standard alternative's cost | Semantic tokens (the LSP-standard alternative) would require expanding `drut-lsp`'s semantic-tokens implementation from 3 narrow, special-purpose types to a full per-category system duplicating the grammar's own classification logic — substantial new Rust/LSP work for a feature whose only target client is VS Code |
