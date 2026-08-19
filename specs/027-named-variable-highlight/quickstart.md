# Quickstart: `@name@` Variable Highlight Color Customization

## 1. Build

```powershell
cd editors/vscode
npm install
npm run compile
```

## 2. Pure decision-function tests

```powershell
npx ts-node test/highlightCustomization.test.ts
```

Expected: new `decideVariableColorSync` checks pass, including —

- Never-seeded workspace, no configured color → seeds the default, marks seeded.
- Configured color set → writes it, marks live-sync active, regardless of prior state.
- Live-sync active, configured color cleared → one write reverting to the default,
  clears live-sync-active.
- Never live-synced, already seeded, no configured color → no write (never fights a
  manual deletion) — the `026`-preserving regression case (spec.md SC-002).

## 3. Manual verification (Extension Development Host)

1. In a fresh workspace, confirm `@name@` renders `#4EC9B0` on first open (today's
   existing default, unaffected).
2. Set `drut.highlight.namedVariables` to a distinct color; confirm immediate
   recoloring, no reload.
3. Clear it; confirm reversion to `#4EC9B0`, not a stuck custom color.
4. In a separate, previously-activated workspace, manually delete the `variable:drut`
   rule from `.vscode/settings.json`; reload; confirm it stays deleted (never touched
   `drut.highlight.namedVariables` in that workspace).
