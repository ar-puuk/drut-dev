# Contract: `@name@` Variable Highlight Color Customization (amends `026-highlight-customization`)

## No `voyager-core`/adapter-crate change

Same as `026` — purely `editors/vscode`. `drut-lsp`'s semantic-token emission for
`@name@` is unchanged (still unconditional, still the standard `variable` type).

## Behavior contract

- **New setting**: `drut.highlight.namedVariables`, same shape as `026`'s 9 settings
  (optional string, default unset, Global scope read only).
- **Untouched-by-default guarantee**: a workspace that never sets
  `drut.highlight.namedVariables` sees `ensureVariableColorCustomization` behave
  byte-identically to its pre-`027` shipped behavior — including a manual deletion of
  the seeded rule sticking forever.
- **Live sync, once opted in**: setting `drut.highlight.namedVariables` keeps the
  current workspace's `variable:drut` rule (in `editor.semanticTokenColorCustomizations`,
  Workspace scope) matching that value, immediately, on every relevant change.
- **Clean revert on unset**: clearing `drut.highlight.namedVariables` after it was set
  performs exactly one write reverting the rule to `#4EC9B0` (the documented default)
  — never removes the rule outright, never leaves it stuck at the last custom color.
- **Workspace scope, not Global** — required for correctness (research.md §2), a
  documented exception to `026`'s Global-only rule for its other 9 categories.

## Illustrative examples

| Scenario | Result |
|---|---|
| Brand-new workspace, `drut.highlight.namedVariables` never set | `variable:drut = #4EC9B0` seeded once, exactly as `026` already shipped |
| Existing workspace (already seeded), user sets `drut.highlight.namedVariables = "#FF0000"` | `variable:drut` updated to `#FF0000` immediately |
| Same workspace, user later clears `drut.highlight.namedVariables` | `variable:drut` reverts to `#4EC9B0` (one corrective write) |
| A different, never-configured workspace where the user long ago manually deleted the seeded rule | Stays deleted — this feature never re-adds it unless `drut.highlight.namedVariables` is explicitly set in that workspace |
