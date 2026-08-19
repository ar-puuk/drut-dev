# Contract: Editor Highlight Color Customization (amends `003-lsp-vscode-extension` / `024-function-call-highlighting`)

Extends `editors/vscode`'s settings surface and grammar. A conceptual signature
contract, not final JSON/TypeScript source — same convention every prior contract doc in
this repo follows.

## No `voyager-core`/adapter-crate change

- No `voyager-core` function signature, type, or `Diagnostic` category changes.
- No `drut-config`/`drut-cli`/`drut-mcp` field, flag, or parameter is added.
- `editors/vscode/syntaxes/drut.tmLanguage.json`'s only change is renaming one shared
  `"name"` field into two (§ below) — no match-pattern logic changes.

## Behavior contract

- **9 new settings**: `drut.highlight.controlWords`, `.statementWords`,
  `.functionCalls`, `.pairKeywords`, `.values`, `.numbers`, `.operators`, `.comments`,
  `.strings` — each an optional string (a CSS color), default unset.
- **Unset is a strict no-op**: with every `drut.highlight.*` setting unset,
  `editor.tokenColorCustomizations` is never written to by this feature at all — not
  written-then-cleared, never touched in the first place (spec.md SC-002).
- **Set applies globally, immediately**: setting `drut.highlight.<category>` writes (or
  updates) a `textMateRules` rule for that category's scope(s) into
  `editor.tokenColorCustomizations` at Global (User) scope, and every open `.s`/`.block`
  document re-renders with the new color without a window reload (FR-005).
- **Unset reverts, doesn't strand**: clearing a previously-set `drut.highlight.<category>`
  removes exactly that category's rule, reverting to the active theme's own color for
  that scope — never leaves a stale color behind (spec.md Acceptance Scenario 3).
- **Never touches a rule it doesn't own**: any other content in
  `editor.tokenColorCustomizations` — another extension's rules, the user's own hand-
  written rules, per-theme override objects, shorthand keys — survives every
  `drut.highlight.*` set/unset cycle byte-for-byte (FR-004, ownership test in
  `research.md` §4).
- **Workspace/Folder-scoped `drut.highlight.*` values are ignored**: only the Global
  value is read; only Global `editor.tokenColorCustomizations` is written (FR-010).
- **`functionCalls` and `statementWords` are now independently colorable**: setting one
  does not affect the other — proof that the FR-006 grammar scope split actually
  decoupled them (spec.md SC-005).
- **`variables` (`@name@`) is unaffected by this feature entirely** — its existing
  `ensureVariableColorCustomization` behavior (workspace-scoped, one-time, `#4EC9B0`
  default) continues exactly as before, untouched (research.md §3).

## Illustrative examples

| `drut.highlight.*` state | `editor.tokenColorCustomizations` after sync |
|---|---|
| All unset | Untouched — this feature writes nothing |
| `functionCalls = "#FF6B35"` only | `{"textMateRules":[{"scope":"support.function.builtin.drut","settings":{"foreground":"#FF6B35"}}]}` merged into whatever else was already present |
| `comments = "#6A9955"` only | `{"textMateRules":[{"scope":["comment.line.semicolon.drut","comment.block.drut"],"settings":{"foreground":"#6A9955"}}]}` merged in |
| `functionCalls` set, then unset again | The `support.function.builtin.drut` rule is removed; if it was the only rule this feature had added and `editor.tokenColorCustomizations` had no other content at all, the whole setting is cleared rather than left as `{}` |
| User's own unrelated rule for `entity.name.tag.python` already present | Present, unmodified, at every step of every scenario above |
