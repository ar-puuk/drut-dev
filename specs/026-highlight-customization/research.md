# Research: Editor Highlight Color Customization

## 1. `editor.tokenColorCustomizations`'s real shape

VS Code's own setting (not a drut invention) accepts, at minimum:

```json
{
  "textMateRules": [
    { "scope": "keyword.control.drut", "settings": { "foreground": "#RRGGBB" } },
    { "scope": ["comment.line.semicolon.drut", "comment.block.drut"], "settings": { "foreground": "#RRGGBB" } }
  ],
  "[Some Theme Name]": { "textMateRules": [ /* per-theme override, same shape */ ] }
}
```

It also accepts generic shorthand keys (`comments`, `strings`, `numbers`, `functions`,
`variables`, `keywords`, `types`) that recolor that *category across every language*,
not just drut's. **Deliberately not used** — setting the generic `numbers` shorthand
would recolor numeric literals in every other language the user edits, not just
`.s`/`.block` files, which is not what "customize Voyager script highlighting" means.
Every rule this feature writes uses an explicit `scope` (one of drut's own `.drut`-suffixed
scope names) inside the top-level `textMateRules` array — never a shorthand key, never a
per-theme (`"[Theme Name]"`) nested object (Global, theme-independent, matching how a
personal color preference should behave regardless of which theme is active that day).

## 2. Existing precedent: `ensureVariableColorCustomization` / `ensureFormatOnSaveEnabled`

`editors/vscode/src/extension.ts` already has two functions in this same family —
reading/writing a VS Code setting the extension doesn't itself contribute, on
activation, guarded so they never fight a user's own later change:

- `ensureVariableColorCustomization`: injects a `variable:drut` rule into
  `editor.semanticTokenColorCustomizations` (**Workspace** scope,
  `.vscode/settings.json`), **once ever per workspace** (tracked via
  `context.workspaceState`), value hardcoded to `#4EC9B0`. Exists because real manual
  testing found some themes render nothing at all for `variable.other.readwrite.drut`
  (the TextMate scope), a gap only the semantic-token layer's VS-Code-baseline color
  support closes.
- `ensureFormatOnSaveEnabled`: injects a Workspace-scoped, language-overridden
  `editor.formatOnSave = true`, same one-time-ever lifecycle.

**Why this feature doesn't reuse that exact shape**: both existing functions solve "seed
a good default once, then get out of the way forever" — a one-time onboarding nudge, not
an ongoing, live, user-driven preference. This feature is the opposite in kind: a
`drut.highlight.<category>` value is meant to keep applying, continuously, in sync,
across every workspace the user opens (the resolved Scope decision: a personal
preference, not a per-project one) — which means Global scope and live
`onDidChangeConfiguration` reactivity, not Workspace scope and a one-time flag. Emulating
the existing functions' *shape* (a guarded, best-effort, try/catch-wrapped async
function called from `activate()`) is still the right call for consistency; emulating
their *scope and lifecycle* is not.

## 3. Why `variables` (`@name@`) is out of scope for this feature

`ensureVariableColorCustomization`'s `variable:drut` rule lives in
`editor.semanticTokenColorCustomizations`, keyed by *LSP semantic token type*
(`variable`), scoped to the `drut` language. This feature's own mechanism (§1) writes
`editor.tokenColorCustomizations`, keyed by *TextMate scope*
(`variable.other.readwrite.drut`). These are two different, independently-resolved
rendering layers — VS Code applies a semantic-token color over a TextMate-scope color
for the same span whenever a semantic token is present for it. Since `drut-lsp` already
emits a semantic `variable` token for every `@name@` (this is real, shipped, unconditional
— not gated behind any setting), a `drut.highlight.variables` setting built on this
feature's own mechanism would be silently overridden by the existing semantic-token rule
and appear to do nothing. Fixing that would mean either (a) making
`drut.highlight.variables` drive the *existing* `variable:drut` rule's value instead
(a different write-target, different lifecycle migration: one-time/Workspace →
live/Global, a real behavior change to already-shipped code), or (b) suppressing the
existing semantic-token emission for `@name@` so the TextMate layer can win instead
(reintroducing the exact invisibility-under-some-themes bug that rule was added to fix).
Neither is a small addition to this feature — both are their own, separately-scoped
follow-up. `variables` is excluded from `drut.highlight.*`'s initial category list;
`ensureVariableColorCustomization` is untouched (spec.md FR-008).

## 4. Ownership test: exact-scope-match, not substring/prefix match

A `textMateRules` entry's `scope` field may be a single string or an array of strings.
This feature's own rules always use one specific shape per category (a single string for
a single-scope category, e.g. `controlWords`; an array of exactly two strings for
`comments`/`strings`). **Ownership test**: an existing rule is "ours" (safe to
replace/remove on the next sync) exactly when its `scope`, normalized to an array and
compared as a set, equals one of the 9 known category scope-sets exactly — not "contains
one of our scope names," not "overlaps." A user's own hand-written rule combining one of
our scopes with an unrelated scope in the same array (an unusual but possible thing to
do) is therefore never touched — safer than a looser match, at the cost of not cleaning
up that specific edge case, which spec.md's Edge Cases already accepts as a known,
undetected interaction rather than something this feature must solve.

## 5. Grammar split: `support.function.drut` → two scopes

`024-function-call-highlighting` deliberately reused `#statement-words`'s
`support.function.drut` scope for `#function-calls` rather than inventing a fourth
visual tier (that feature's own `research.md` §6/§7) — the right call *for highlighting
alone*, since both categories were meant to render identically. For *this* feature,
identical-by-construction is now the wrong default: a user setting
`drut.highlight.functionCalls` reasonably expects `PRINT`/`FILEI` (statement words) to be
unaffected. Splitting into `support.function.statement.drut` (statement words) and
`support.function.builtin.drut` (function calls) is a pure rename at the grammar level —
neither pattern's match logic changes, only the `"name"` field two patterns already
have. Any theme with no explicit rule for either new name still falls back to its own
`support.function`-prefix rule (if it has one) exactly as before, since TextMate scope
matching is prefix-aware — so this split causes no visible change for any user who
hasn't touched `drut.highlight.*` at all (spec.md FR-006's own "not a behavior
regression" claim, now grounded).

## 6. Color value validation: none added

`drut.highlight.<category>` is a plain `"type": "string"` setting (VS Code's
`package.json` configuration schema has no dedicated "is this a valid CSS color" format
validator for extension-contributed settings). An invalid value is passed through to
`editor.tokenColorCustomizations` verbatim; VS Code's own rendering of that setting
already degrades gracefully for a malformed color (no crash, no override applied) — this
feature adds no separate validation layer, matching spec.md's Edge Cases.
