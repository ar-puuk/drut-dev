# Quickstart: Validating Data-Reference & User-Variable Highlighting

A runnable validation guide, not an implementation walkthrough — proves this feature
against `spec.md`'s Success Criteria. See `contracts/identifier-highlighting.md` for the
exact behavior contract and `data-model.md`/`research.md` for the full design rationale.

## Prerequisites

- Node.js + npm (matches `editors/vscode`'s existing dev setup).
- `026-highlight-customization` already shipped (this feature adds two more entries to
  its `HighlightCategory`/`CATEGORY_SCOPES` machinery).

## 1. Install & compile

```powershell
cd editors/vscode
npm install
npm run compile
```

## 2. Grammar tokenization spot-checks — validates FR-001, FR-003, FR-004, FR-004a

```powershell
npm test
```

Expected: `grammar.test.ts` reports new checks confirming —

- `DBA` scopes as `variable.language.data-reference.drut` both immediately after `=`
  (`X = DBA.2.field`) and inside a function-call argument
  (`ROUND(DBA.2.VOL[numrec])`) — the exact reported gap (User Story 1).
- `DBI` on a `LOOP` opener line's bound expression (`LOOP NUMREC = counter,
  DBI.2.NUMRECORDS`) also scopes as `variable.language.data-reference.drut`.
- `ZONES` in `RUN PGM=MATRIX ZONES=5` scopes as `variable.language.data-reference.drut`,
  not `variable.parameter.drut` (FR-003 precedence).
- In `LINKID = _ANode + '_' + _BNode`, `_BNode` scopes as
  `variable.other.identifier.drut` — the exact reported gap (User Story 2).
- A recognized control word, statement word, function-call name, pair-keyword name, or
  pair value never scopes as `variable.other.identifier.drut` (FR-004's exclusion list,
  regression check).
- A `ShellEscape` line (`*copy A B`) scopes entirely as
  `meta.embedded.shell-escape.drut` — `A`/`B` do not scope as `dataReferences` inside it
  (FR-004a).
- A `Label` declaration (`:STEP0`) scopes as `entity.name.label.drut`, not
  `variable.other.identifier.drut` (FR-004a).
- Every pre-existing `grammar.test.ts` check continues to pass unchanged (no regression
  to any of the 9 existing categories).

## 3. Pure category/scope-table unit tests — validates FR-002, FR-005, FR-006

```powershell
npx ts-node test/highlightCustomization.test.ts
```

Expected: all green, including two new cases confirming `dataReferences`/
`userVariables` are present in `CATEGORY_SCOPES` with the correct scope strings, and
that `mergeHighlightRules`'s existing ownership/merge behavior (unchanged code) already
generalizes correctly to the two new keys with zero special-casing.

## 4. Manual verification in a real Extension Development Host

`F5` from `editors/vscode` to launch a Development Host, open the exact production
script excerpt that surfaced this feature (`LOOP NUMREC = counter,
DBI.2.NUMRECORDS` ... `VOL_COR = ROUND(DBA.2.VOL[numrec]) / 100` ... `LINKID = _ANode +
'_' + _BNode`), then:

1. Confirm `drut.highlight.dataReferences` and `drut.highlight.userVariables` are both
   unset; confirm `DBA`/`DBI` render consistently everywhere they appear (not just after
   `=`), and `_ANode`/`_BNode` both render as recognizable identifiers (SC-001, SC-002).
2. Set `drut.highlight.dataReferences` to a distinct color in Settings UI (search "drut
   highlight"). Confirm every `DBA`/`DBI`/`ZONES`/etc. occurrence recolors immediately,
   with no reload (SC-003).
3. Set `drut.highlight.userVariables` to a distinct color. Confirm `_BNode` (and every
   other bareword not otherwise categorized) recolors immediately (SC-003).
4. Clear both settings. Confirm both categories revert to the theme's own color, and
   (if nothing else was in `editor.tokenColorCustomizations`) the setting is removed
   from `settings.json` entirely rather than left as `{}` (inherited from `026`'s
   existing `mergeHighlightRules` behavior).
5. Confirm none of `drut.highlight.controlWords`, `.statementWords`, `.functionCalls`,
   `.pairKeywords`, `.values`, `.numbers`, `.operators`, `.comments`, `.strings`, or
   `.namedVariables` changed behavior at any point above (SC-004).

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2 | SC-001 |
| 3 | SC-002 |
| 2, 3, 4 | SC-003 |
| 5 | SC-004 |
| 1 | SC-001, SC-002 (mechanism-level proof, before any setting is touched) |
