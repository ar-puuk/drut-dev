# Research: `@name@` Variable Highlight Color Customization

## 1. Why this can't just be a 10th entry in `CATEGORY_SCOPES`

`026`'s `mergeHighlightRules`/`CATEGORY_SCOPES` mechanism targets
`editor.tokenColorCustomizations` (TextMate-scope-keyed). `@name@` is additionally
colored by a semantic token (`drut-lsp` emits a standard `variable` type for it,
unconditionally), and VS Code visually layers a semantic-token color over a
TextMate-scope color for the same span. Because of that, whatever `026`'s own
mechanism wrote for `variable.other.readwrite.drut` would be invisible wherever the
pre-existing `variable:drut` semantic rule is present — which is every workspace this
extension has ever activated in, per `ensureVariableColorCustomization`'s one-time
seed. This was `026`'s own research.md §3 finding; this feature is the follow-up it
predicted.

## 2. Scope resolution: why Workspace, not Global

VS Code does not deep-merge an object-valued setting across scopes — whichever scope
has an explicit value for the *whole setting* wins, most-specific-scope-first
(Workspace beats Global). `ensureVariableColorCustomization` writes
`editor.semanticTokenColorCustomizations` at Workspace scope
(`.vscode/settings.json`) on first activation in every workspace. If this feature
wrote its live-synced override at Global scope instead, any workspace that has ever
activated this extension before already has a Workspace-scoped value for that setting
— which would silently win over the new Global write, making
`drut.highlight.namedVariables` appear to do nothing in exactly the situations where a
user would actually test it (an existing project they already work in). Workspace
scope is therefore required for correctness, not a stylistic deviation from `026`'s
Global-only convention for its other 9 categories.

## 3. The decision function: reconciling "live sync" with "never fight a manual deletion"

Two behaviors need to coexist without regressing either:

- **Existing guarantee** (`026` research.md §2, quoting the original feature's own
  intent): a user who manually deletes the seeded `variable:drut` rule keeps it
  deleted forever, for that workspace, as long as they never touch
  `drut.highlight.namedVariables`.
- **New guarantee**: once a user *does* set `drut.highlight.namedVariables`, it should
  behave like every other `drut.highlight.*` category — live, reactive, and reverting
  cleanly on unset.

These are reconciled with one additional `workspaceState` boolean,
`drutVariableColorLiveSyncActive` (alongside the pre-existing
`drutVariableColorInjected`), tracking whether this feature's live-sync has ever taken
over for this specific workspace:

| `configuredColor` (Global `drut.highlight.namedVariables`) | `liveSyncActive` (before) | Action | `liveSyncActive` (after) |
|---|---|---|---|
| set | any | write `configuredColor` if the current rule doesn't already match it | `true` |
| unset | `true` (was live-synced, just turned off) | write the documented default (`#4EC9B0`) once | `false` |
| unset | `false`, never seeded before, no existing rule | write the documented default once (today's exact original seed behavior) | `false` |
| unset | `false`, already seeded (or user removed it) before | do nothing — never fight a manual choice | `false` |

This table is `decideVariableColorSync`'s exact truth table (`data-model.md` §1) — a
pure function, unit-tested directly, not re-derived from prose at implementation time.

## 4. Reuse of the existing `onDidChangeConfiguration` listener

`026`'s `extension.ts` already registers a listener on
`e.affectsConfiguration("drut.highlight")` — a prefix match that already covers
`drut.highlight.namedVariables` as a new sub-key, with zero listener-registration
changes needed. This feature adds a call to the refactored
`ensureVariableColorCustomization` inside that same handler (alongside the existing
`applyHighlightCustomizations` call), and keeps the existing call from `activate()`
(for the "brand-new workspace" seed case, spec.md Edge Cases).
