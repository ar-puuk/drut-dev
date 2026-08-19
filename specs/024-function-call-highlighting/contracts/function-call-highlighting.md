# Contract: Function-Call Syntax Highlighting (amendment to `003-lsp-vscode-extension`)

Extends `editors/vscode/syntaxes/drut.tmLanguage.json`'s existing two-tier
`#control-words`/`#statement-words` convention. A conceptual signature contract, not final
JSON source — same convention every prior contract doc in this repo follows.

## No public API / protocol change

- No `voyager-core` function signature, `Token`, `Diagnostic`, or `ParseResult` field is
  added, renamed, or removed (FR-007).
- No `drut-config` field, CLI flag, MCP parameter, or VS Code client setting is added,
  renamed, or removed (FR-008).
- This is a change to `source.drut`'s TextMate scope assignments only.

## Behavior contract

- **Scope of the new pattern**: a word from the 138-name list (`data-model.md`
  §1), matched case-insensitively, immediately followed by `(` with **zero** intervening
  whitespace, renders with `support.function.drut` — the same scope `#statement-words`
  already uses, so it is visually indistinguishable from that tier under any theme
  (deliberate: both are "the language's built-in procedures," just populated by different
  methods and gated by a different position — `#statement-words` unconditionally, this
  pattern only when `(` immediately follows).
- **Position-independent**: this rendering applies wherever the call-shaped occurrence
  appears in a statement — nested inside another call's arguments, inside an `IF`/short-`IF`
  condition, on an `Assignment`'s right-hand side, inside a `Control` statement's
  pair-keyword value, or anywhere else a value expression is legal. Unlike `#pair-values`,
  this pattern's trigger has nothing to do with proximity to `=`.
- **Never fires without the trailing `(`**: a bareword that spells a recognized function
  name but is not immediately followed by `(` (a `keyword=value` pair's keyword name, a
  plain identifier, whitespace before a later unrelated `(`) is untouched by this pattern
  and keeps whatever scope it already had (FR-006).
- **Does not touch `#pair-values`' existing behavior**: `#pair-values` still colors any
  bareword immediately after `=`, including — coincidentally, same as before this feature —
  one of these 138 names when it happens to sit there (e.g. `RouteName = REPLACESTR(...)`:
  `REPLACESTR` now matches `#function-calls` first by array order, so it renders
  `support.function.drut` either way; the visual result is unchanged from today for that
  specific case, only *how* it gets there changes).
- **Non-exhaustive by construction**: a genuine Cube Voyager built-in function not in the
  138-name list renders unstyled, exactly as before this feature (FR-004; `research.md` §5).
- **No grammar ordering hazard**: `#function-calls`' `(`-lookahead and `#pair-keywords`'/
  `#pair-values`' `=`-lookahead/lookbehind are mutually exclusive triggers on the same
  token position — a single bareword cannot simultaneously be immediately followed by `(`
  and immediately followed (or preceded, for `#pair-values`) by `=`, so pattern array order
  between these three does not create a correctness dependency (`data-model.md` §2).

## Illustrative examples (not exhaustive — see `spec.md`'s Acceptance Scenarios)

| Input | `RIGHTSTR`/`TRIM`/etc. scope before this feature | Scope after this feature |
|---|---|---|
| `RouteName = REPLACESTR(RouteName,'-','',0)` | `constant.other.drut` (accidental, via `#pair-values`) | `support.function.drut` (via `#function-calls`) |
| `if (RIGHTSTR(TRIM(RouteName),1)='-')` | unstyled (no pattern claims this position) | `support.function.drut` for both `RIGHTSTR` and `TRIM` |
| `if (STRLEN(TRIM(@SEGIDExField@))>0)` | unstyled | `support.function.drut` for both `STRLEN` and `TRIM` |
| `ANGLE = ROUND(_L.S_Angle * 10) / 10` | unstyled | `support.function.drut` for `ROUND`; `_L.S_Angle` unaffected |
| `MAX = 100` (bareword, no following `(`) | `variable.parameter.drut` (via `#pair-keywords`) | unchanged — `#function-calls` never fires (no `(`) |
| `MYCALC(x)` (not in the 138-name list) | unstyled | unchanged — unstyled (FR-004, non-exhaustive) |
