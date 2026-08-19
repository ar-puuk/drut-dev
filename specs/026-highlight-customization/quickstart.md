# Quickstart: Validating Editor Highlight Color Customization

A runnable validation guide, not an implementation walkthrough — proves this feature
against `spec.md`'s Success Criteria. See `contracts/highlight-customization.md` for the
exact behavior contract and `data-model.md`/`research.md` for the full design rationale.

## Prerequisites

- Node.js + npm (matches `editors/vscode`'s existing dev setup).
- `024-function-call-highlighting` already shipped (this feature splits its
  `support.function.drut` scope).

## 1. Install & compile

```powershell
cd editors/vscode
npm install
npm run compile
```

## 2. Grammar scope-split regression — validates FR-006, SC-005

```powershell
npm test
```

Expected: `grammar.test.ts` reports new checks confirming `PRINT`/`FILEI` (statement
words) scope as `support.function.statement`, a recognized function call scopes as
`support.function.builtin`, and both are still distinct from `keyword.control` — plus
every pre-existing check continues to pass unchanged (no visible behavior regression
from the rename alone).

## 3. Pure merge-logic unit tests — validates FR-002, FR-003, FR-004, FR-010

```powershell
npx ts-node test/highlightCustomization.test.ts
```

Expected: all green, including —

- An empty `desired` map against a populated `current` object with unrelated content
  leaves that content untouched and adds nothing.
- Setting one category upserts exactly one rule with the correct scope(s) and color.
- Unsetting a previously-set category removes exactly that rule and no other.
- A rule with a scope that only *partially* overlaps one of drut's known category scope
  sets (not an exact match) is never touched (research.md §4's exact-match ownership
  test).
- When the result would leave `textMateRules` empty and there's no other content,
  `isEmptyTokenColorCustomizations` reports true (User Story 2 Acceptance Scenario 2).

## 4. Manual verification in a real Extension Development Host

`F5` from `editors/vscode` to launch a Development Host, open a `.s` fixture, then:

1. Confirm `drut.highlight.functionCalls` and `drut.highlight.statementWords` are both
   unset today's theme colors both `PRINT` and a recognized function call the same way
   they always have (SC-002).
2. Set `drut.highlight.functionCalls` to a distinct color in Settings UI (search "drut
   highlight"). Confirm the function call recolors immediately, with no reload, and
   `PRINT` is unaffected (SC-001, SC-004, SC-005).
3. Hand-add an unrelated rule to `editor.tokenColorCustomizations` in `settings.json`
   (e.g. for `entity.name.tag.python`). Repeat step 2's set/unset cycle; confirm the
   hand-added rule is present and unmodified throughout (SC-003).
4. Clear `drut.highlight.functionCalls`. Confirm the function call reverts to the
   theme's own color, and (if nothing else was in `editor.tokenColorCustomizations`)
   the setting is removed from `settings.json` entirely rather than left as `{}`.

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2 | SC-005 |
| 3 | SC-001, SC-002, SC-003, SC-004 (mechanism-level proof) |
| 4 | SC-001, SC-002, SC-003, SC-004, SC-005 (real-editor confirmation) |
