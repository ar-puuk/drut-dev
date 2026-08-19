# Implementation Plan: `@name@` Variable Highlight Color Customization

**Branch**: `027-named-variable-highlight` | **Date**: 2026-08-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/027-named-variable-highlight/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

Adds `drut.highlight.namedVariables`, the 10th `drut.highlight.*` category and the one
`026-highlight-customization` deliberately deferred. Refactors `extension.ts`'s
existing `ensureVariableColorCustomization` (unchanged behavior when the new setting
is untouched) into a pure-logic-backed function that, once the user has explicitly set
`drut.highlight.namedVariables`, keeps the current workspace's `variable:drut` rule in
`editor.semanticTokenColorCustomizations` continuously synced to that value — reverting
to the documented default (`#4EC9B0`) on unset rather than removing the rule outright.

## Technical Context

**Language/Version**: TypeScript (`editors/vscode`, same toolchain as `026`)

**Primary Dependencies**: None new — `vscode` extension API only, same as `026`/the
pre-existing `ensureVariableColorCustomization`.

**Storage**: `context.workspaceState` (already used by the pre-existing mechanism for
its one-time-seed flag; this feature adds one more boolean flag alongside it)

**Testing**: New pure-logic cases in `test/highlightCustomization.test.ts` (the new
decision function has zero `vscode` dependency, same as everything else in that file)

**Target Platform**: VS Code (same rationale as `026` — this setting maps to a
VS-Code-proprietary customization surface; Constitution Principle VI already weighed
and set aside in `026`'s plan.md for the same class of feature)

**Project Type**: Editor extension (thin adapter) — `editors/vscode` only

**Performance Goals**: N/A — same cost class as `026`'s other 9 categories

**Constraints**: MUST NOT change `ensureVariableColorCustomization`'s behavior at all
for any workspace that never sets `drut.highlight.namedVariables` (spec.md FR-004,
SC-002) — the refactor must be behavior-preserving for the untouched case, verified by
a dedicated test, not just asserted.

**Scale/Scope**: One pure decision function + its tests, one refactor of an existing
function, one new workspaceState key, one new `package.json` setting, one addition to
the existing `onDidChangeConfiguration` handling (already fires for any
`drut.highlight.*` change, including this new key, so no new listener needed)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Single Source of Truth)**: No grammar/parsing logic touched;
  `drut-lsp`'s semantic-token emission for `@name@` is unchanged (FR-007). **PASS.**
- **Principle II–V, VII, VIII**: Same analysis as `026`'s plan.md — N/A or PASS,
  nothing new introduced by this feature's nature.
- **Principle VI (LSP-Standard Mechanisms)**: Same exception already recorded and
  justified in `026`'s plan.md Complexity Tracking — this feature extends that same,
  already-accepted exception to one more category, not a new deviation.

No new violations — Complexity Tracking table is not needed (references `026`'s
existing entry).

## Project Structure

### Documentation (this feature)

```text
specs/027-named-variable-highlight/
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
├── src/
│   ├── highlightCustomization.ts  # add decideVariableColorSync (pure, new)
│   └── extension.ts                # refactor ensureVariableColorCustomization to
│                                    # call it; add namedVariables to
│                                    # applyHighlightCustomizations' Global-read loop
│                                    # is NOT needed (this category isn't in
│                                    # CATEGORY_SCOPES -- separate code path)
├── test/
│   └── highlightCustomization.test.ts  # add decideVariableColorSync cases
└── package.json                    # 1 new drut.highlight.namedVariables setting
```

**Structure Decision**: Single existing project (`editors/vscode`), same as `026`. No
new crate, no other adapter touched.
