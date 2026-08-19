# Implementation Plan: Data-Reference & User-Variable Highlighting

**Branch**: `028-identifier-highlighting` | **Date**: 2026-08-19 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/028-identifier-highlighting/spec.md`

## Summary

Two identifier classes render with no real highlighting today, only accidental coloring
from position-based `TextMate` rules meant for something else (found via real-world
testing against a production script): the data-reference family (`DBA`/`MI`/`MW`/...,
already recognized by `voyager-core`'s `data_reference.rs` for casing) and generic
user-defined variable identifiers. This plan adds two new `TextMate` scopes
(`variable.language.data-reference.drut`, `variable.other.identifier.drut`), two
`drut.highlight.*` settings (`dataReferences`, `userVariables`) following `026-highlight-
customization`'s existing personal-setting mechanism unchanged, and two small
line-scoped grammar patterns (`#shell-escape`, `#label`) so the new, aggressive
catch-all identifier pattern doesn't reach into non-Voyager-syntax content. Purely
`editors/vscode` — no `voyager-core`, `drut-lsp`, `drut-config`, or CLI change.

## Technical Context

**Language/Version**: TypeScript (VS Code extension host, Node.js) — `editors/vscode`'s
existing stack, unchanged.

**Primary Dependencies**: `vscode` API, `vscode-textmate`/`vscode-oniguruma` (already a
dev dependency, used by `grammar.test.ts`). No new npm dependency.

**Storage**: N/A — VS Code configuration (`drut.highlight.*`, Global scope;
`editor.tokenColorCustomizations`, Global scope) via the existing `mergeHighlightRules`
mechanism.

**Testing**: `editors/vscode/test/grammar.test.ts` (real grammar file through
`vscode-textmate`, no VS Code instance needed) and
`editors/vscode/test/highlightCustomization.test.ts` (pure `CATEGORY_SCOPES`/
`mergeHighlightRules` unit tests via plain `ts-node`) — both already exist, extended with
new cases, no new test harness.

**Target Platform**: VS Code extension (desktop + web, same as today — no platform-
specific API used).

**Project Type**: Editor extension client (`editors/vscode`), single project — same as
`026`/`027`.

**Performance Goals**: Negligible — two more `TextMate` `match` patterns of the same
class as the 10 already in the grammar; no measurable tokenization latency change
expected on the real corpus's largest files.

**Constraints**: Purely additive to the 9 existing `drut.highlight.*` categories and
`drut.highlight.namedVariables` (FR-006); no `voyager-core`/`drut-lsp`/`drut-config`/CLI
change (FR-007); content inside a quoted string or comment never matched (FR-008); Label/
`ShellEscape` content never matched by either new category (FR-004a).

**Scale/Scope**: `editors/vscode` only. 4 new `TextMate` patterns
(`#data-references`, `#user-identifiers`, `#shell-escape`, `#label`), 2 new
`HighlightCategory` entries, 2 new `package.json` settings. No new source files beyond
test-case additions to the two existing test files.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Single Source of Truth)**: This feature adds two hand-authored keyword/
  shape lists to the `TextMate` grammar (the 17-name data-reference family; the generic
  identifier shape) that are *not* generated from or dynamically shared with
  `voyager-core`'s `data_reference.rs`. This looks like duplication on its face, but it
  is the same, already-established pattern the grammar's other 5 keyword-list patterns
  (`#control-words`, `#statement-words`, `#function-calls`, plus the pre-existing
  `#pair-keywords`/`#pair-values` shape rules) already use — a static, declarative
  `TextMate` grammar has no mechanism to import a Rust `const` array at grammar-load
  time, and every prior feature that added a new grammar keyword list
  (`003-lsp-vscode-extension`, `024-function-call-highlighting`, `025-function-casing`)
  accepted the same trade-off. Crucially, this grammar is presentational only — it never
  produces a `Diagnostic`, never feeds a parse decision, and never contradicts
  `voyager-core`'s own authoritative recognition (the two lists just happen to name the
  same 17 identifiers, for a different purpose). The single source of truth for what
  *counts* as a data-reference name for correctness purposes (casing, and any future
  diagnostic) remains `data_reference.rs` alone. **PASS** — consistent with established
  precedent, not a new violation.
- **Principle II (No Verbatim Vendor Documentation)**: The data-reference family name
  list is copied from `voyager-core`'s own `data_reference.rs` (this project's own prior
  work, already Principle-II-compliant when it was written), not from vendor
  documentation. The generic-identifier and `Label`/`ShellEscape` patterns are original,
  shape-based regexes with no vendor text involved. **PASS**.
- **Principle III (Formatter Idempotence)**: Not applicable — no formatter change.
  **PASS** (N/A).
- **Principle IV (False Negatives Over False Positives)**: Not applicable in the strict
  sense (this is highlighting, not linting — no diagnostic is produced), but the same
  spirit applies: an unhighlighted identifier is a cosmetic miss, while a wrongly
  highlighted one (e.g. `ShellEscape`/`Label` content) is the actively-misleading case
  the FR-004a exclusion exists specifically to prevent. **PASS**.
- **Principle V (Vertical, Independently-Usable Increments)**: Two user stories, each
  independently testable/shippable (spec.md) — `dataReferences` (P1) does not depend on
  `userVariables` (P2) landing. **PASS**.
- **Principle VI (LSP-Standard Mechanisms)**: Not applicable — no LSP protocol surface
  touched; this reuses `026`'s existing `TextMate`-scope mechanism (already the
  LSP-agnostic, `TextMate`-standard choice for the reason `research.md` §1 restates).
  **PASS** (N/A).
- **Principle VII (Naming Honesty)**: `dataReferences`/`userVariables` names describe
  exactly what they highlight — no overclaiming. **PASS**.
- **Principle VIII (Public/Private Boundary)**: No vendor-documentation-derived content
  involved. **PASS** (N/A).

No violations requiring Complexity Tracking justification.

## Project Structure

### Documentation (this feature)

```text
specs/028-identifier-highlighting/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md         # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   └── identifier-highlighting.md
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
editors/vscode/
├── syntaxes/
│   └── drut.tmLanguage.json      # +4 patterns: #data-references, #user-identifiers,
│                                  #shell-escape, #label (data-model.md §2)
├── src/
│   └── highlightCustomization.ts # +2 HighlightCategory/CATEGORY_SCOPES entries
│                                  (data-model.md §1) -- extension.ts unchanged
├── package.json                  # +2 drut.highlight.* configuration entries
│                                  (data-model.md §3)
└── test/
    ├── grammar.test.ts                    # + new tokenization spot-checks
    └── highlightCustomization.test.ts     # + new CATEGORY_SCOPES assertions
```

**Structure Decision**: Single project, `editors/vscode` only — identical structure to
`026-highlight-customization`/`027-named-variable-highlight`, no new directories. No
`voyager-core`, `drut-lsp`, `drut-config`, `drut-cli`, or `drut-mcp` file touched
(spec.md FR-007).

## Complexity Tracking

*No violations — table intentionally omitted (Constitution Check above: all gates pass).*
