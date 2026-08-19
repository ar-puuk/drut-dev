# Quickstart: Validating Function-Call Syntax Highlighting

A runnable validation guide, not an implementation walkthrough — proves this feature against
`spec.md`'s Success Criteria. See `contracts/function-call-highlighting.md` for the exact
behavior contract and `data-model.md`/`research.md` for the full design rationale.

## Prerequisites

- Node.js + npm (matches `editors/vscode`'s existing dev setup).
- `003-lsp-vscode-extension` already shipped (this feature amends its grammar file, not a
  standalone extension).

## 1. Install & compile

```powershell
cd editors/vscode
npm install
npm run compile
```

## 2. Grammar tokenization tests — validates FR-001, FR-002, FR-003, FR-006

```powershell
npm test
```

Expected: `ts-node test/grammar.test.ts` reports all `ok -` lines, zero `FAIL -` lines,
including new checks added for this feature:

- `RIGHTSTR(TRIM(RouteName),1)` — both `RIGHTSTR` and `TRIM` scope as `support.function`.
- `STRLEN(TRIM(@SEGIDExField@))` — both `STRLEN` and `TRIM` scope as `support.function`,
  and the `@SEGIDExField@` substitution still scopes as `variable.other.readwrite`
  (unaffected).
- `RouteName = REPLACESTR(RouteName,'-','',0)` — `REPLACESTR` scopes as `support.function`
  (same visual result as before this feature, now via `#function-calls` rather than the
  `#pair-values` accident — contract table row 1).
- `ANGLE = ROUND(_L.S_Angle * 10) / 10` — `ROUND` scopes as `support.function`;
  `_L.S_Angle` does not.
- `MAX = 100` — `MAX` does NOT scope as `support.function` (no following `(`) — validates
  FR-006 / User Story 2.
- A case-insensitive check, e.g. `CmpNumRetNum(...)`, scopes as `support.function`
  (FR-003).
- A vendor-reference-only function with no `WF-TDM-Official-Releases` corpus occurrence at
  all, e.g. `SUBSTR(street,4,6)` or `ARCSIN(0.5)`, still scopes as `support.function` —
  validates FR-005's broadened, not-corpus-gated scope (this is the check that would have
  failed under this feature's original, corpus-only 21-name draft).
- `BESTJRNY` (a real, vendor-documented Public Transport skim value that is conventionally
  used *without* a trailing `(...)`) does NOT scope as `support.function` when written bare
  — validates the deliberate exclusion in `data-model.md` §1 / `research.md` §2.
- `IF (GAPCHANGEAVE(3) < 0.006 && GAPCHANGEMAX(3) < 0.009) BALANCE = 1` (a real
  CONVERGE-phase usage line from the reference guide) — `GAPCHANGEAVE`/`GAPCHANGEMAX`
  scope as `support.function`; `BALANCE` does not — validates the 42-function
  CONVERGE-phase family (`research.md` §2).
- The data-driven test iterating all 138 recognized names (`tasks.md` T010) reports every
  single one scoping as `support.function` when called — this is the test that makes
  SC-001's "every function name" claim literally true, not just true for a spot-checked
  sample.

## 3. No regression on existing grammar behavior — validates SC-003

The same `npm test` run also re-executes every pre-existing check in `grammar.test.ts`
(control words, comments, strings, `@variable@` substitutions, pair-keywords/pair-values).
Expected: unchanged pass/fail results for all of them.

## 4. Manual spot-check against real corpus excerpts

Open `crates/voyager-core/tests/fixtures/valid/real_corpus/InputProcessing/1_InputSetup.s`
in VS Code with the `drut` extension installed (or `F5` an Extension Development Host from
`editors/vscode`). Confirm line 118, `if (STRLEN(TRIM(@SEGIDExField@))>0)`, now renders
`STRLEN` and `TRIM` in the same color as any `#statement-words` entry (e.g. `PRINT`),
distinct from both `IF`'s control-word color and the surrounding condition's default text
color.

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2 | SC-001, SC-002, SC-004 |
| 3 | SC-003 |
| 4 | SC-001, SC-002 (real-corpus visual confirmation) |
