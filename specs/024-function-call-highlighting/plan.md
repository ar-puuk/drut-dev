# Implementation Plan: Function-Call Syntax Highlighting

**Branch**: `024-function-call-highlighting` | **Date**: 2026-08-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/024-function-call-highlighting/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command; its definition describes the execution workflow.

## Summary

`editors/vscode/syntaxes/drut.tmLanguage.json` currently gives a Cube Voyager built-in
function call a distinct color only by accident, when the call happens to sit immediately
after `=` (the unrelated `#pair-values` rule). The identical function nested one token
deeper — as another call's argument, or inside an `IF` condition — renders unstyled. This
amends `003-lsp-vscode-extension`'s existing two-tier `#control-words`/`#statement-words`
convention with a third pattern, `#function-calls`, matching a closed list of 138 real Cube
Voyager built-in function names — sourced from the language's own general-purpose
scripting vocabulary, not scoped to one organization's corpus (`research.md`) — only when
immediately followed by `(` (no
intervening whitespace — the unambiguous call position, since Voyager has no
user-definable functions), given its own distinct scope so it never depends on
`#pair-values`' positional accident.

## Technical Context

**Language/Version**: JSON (TextMate grammar, `drut.tmLanguage.json`) + TypeScript 5.4 (grammar test harness)

**Primary Dependencies**: `vscode-textmate` ^9.3.2, `vscode-oniguruma` ^2.0.1 (already-shipped
test-only devDependencies of `editors/vscode` — no new dependency)

**Storage**: N/A

**Testing**: `editors/vscode/test/grammar.test.ts`, run via `npm test` (`ts-node
test/grammar.test.ts`) — standalone `vscode-textmate` tokenization spot-checks, no VS Code
instance required

**Target Platform**: VS Code (and any other TextMate-grammar-consuming editor that loads
`drut.tmLanguage.json`)

**Project Type**: Editor extension (thin adapter; see Constitution Principle I) — single
grammar file + its test harness, no other project type applies

**Performance Goals**: N/A (a bounded, ~138-alternative regex added to a `patterns` array
already containing several multi-alternative word-list regexes of comparable or greater
size — no measurable tokenization latency change)

**Constraints**: Zero-runtime-dependency principle (FR-027) applies to `voyager-core`
only, not touched by this feature at all; no `drut-config`/CLI/MCP/editor-setting surface
change (FR-008)

**Scale/Scope**: One new named grammar pattern (`#function-calls`) with one match rule
covering 138 function names, wired into the top-level `patterns` array; grammar-test
additions covering the acceptance scenarios from `spec.md`; no other file touched outside
`editors/vscode/`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Principle I (Single Source of Truth)**: This feature adds no grammar/parsing/lint-rule
  *logic* to `voyager-core` and duplicates none of it — `voyager-core::tokenize`/`parse`
  are untouched (FR-007), confirmed unchanged in Phase 1 re-check. `drut.tmLanguage.json`'s
  word-list patterns (`#control-words`, `#statement-words`, and now `#function-calls`) are
  presentation-only syntax coloring, not structural validation or diagnostics — the same
  category `003-lsp-vscode-extension` already established as compliant (a word absent from
  any of these lists still parses and behaves identically; it just renders unstyled,
  exactly like every prior word-list addition in this file). **PASS.**
- **Principle II (No Verbatim Vendor Docs)**: The 138-name list's *names* were extracted
  from two local vendor mirrors — `_archive/Citilabs Cube 6.5.1/RG_CUBEVOYAGER.md` and
  `_archive/OpenPaths Cube/html/` — permitted, research-only use per `CLAUDE.md`. Every
  description, category grouping, and rationale in `research.md`/`data-model.md`/
  `contracts/` is written fresh, in this project's own structure and wording; no vendor
  prose, table formatting, or example text is reproduced (`research.md` §1). Scope was
  deliberately kept to the Voyager control-language function surface — a separate
  camelCase object-model scripting API found in the same `OpenPaths Cube` docs was
  identified and excluded as out of scope (`research.md` §3). No Bentley/Citilabs
  documentation text or the bhereth extension's grammar file is copied. **PASS.**
- **Principle III (Formatter Idempotence)**: N/A — this feature touches no formatter code
  (`crates/voyager-core/src/format.rs`, `operator_spacing.rs`, etc. are untouched).
- **Principle IV (False Negatives Over False Positives)**: Not a linter rule, but the same
  spirit is honored structurally: FR-004/FR-006 keep the list non-exhaustive and
  position-gated (`(` immediately follows) rather than guessing at coloring a bareword that
  merely resembles a function name. **PASS.**
- **Principle V (Vertical, Independently-Usable Increments)**: Self-contained single-file
  amendment; no prior phase's fixture-corpus tests are affected. **PASS.**
- **Principle VI (LSP-Standard Mechanisms)**: TextMate grammar coloring is the mechanism
  `003-lsp-vscode-extension` already shipped for this exact concern (static word-list
  coloring, `#control-words`/`#statement-words`); this feature extends that existing,
  already-accepted mechanism rather than introducing a new VS Code-proprietary API. Moving
  the whole static-coloring scheme to LSP semantic tokens is out of scope for this
  amendment (it would be a `003`-level architectural change, not this feature's concern).
  **PASS** (no new deviation introduced).
- **Principle VII (Naming Honesty)**: The new pattern is named for what it does
  (`#function-calls`, matching call-shaped occurrences) — no overclaiming. **PASS.**
- **Principle VIII (Public/Private Boundary)**: No vendor-documentation-derived content is
  imported; the corpus census in `research.md` records only word spellings and file counts,
  in this project's own words, matching how `keywords.rs`'s own module docs already
  document a prior corpus census. **PASS.**

No violations — Complexity Tracking table is not needed.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
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
│   └── drut.tmLanguage.json   # add #function-calls pattern + repository entry
└── test/
    └── grammar.test.ts        # add scenario checks (spec.md Acceptance Scenarios 1-4)
```

**Structure Decision**: Single existing project (`editors/vscode`, the VS Code extension
established by `003-lsp-vscode-extension`). No new directory, crate, or package — this
feature edits exactly the two files above. `voyager-core` and every other crate/adapter is
untouched (Constitution Principle I; FR-007/FR-008).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
