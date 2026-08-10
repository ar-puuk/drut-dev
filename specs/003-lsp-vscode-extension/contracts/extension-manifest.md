# Contract: `editors/vscode/` Extension Manifest

The extension's contract with VS Code/Open VSX itself — the `package.json`
contribution points and companion static files this feature's `editors/vscode/`
package must define. Not Rust; documented here in the same
decision-plus-rationale spirit as the other contracts.

## `contributes.languages`

Registers a language ID (e.g. `drut-voyager`) for `.s`/`.block` file extensions,
plus `language-configuration.json`'s path (brackets, comment-toggling — FR-022).
Structural shape may reference `bhereth.language-citilabscubevoyager`'s language
registration under the constitution's granted permission (research.md §8); the
language ID, display name, and all wording are Drut's own.

## `contributes.grammars`

Points at `syntaxes/drut.tmLanguage.json` (FR-021) — a static TextMate grammar
covering control words, comments (including nested block comments, per the
Phase 1 lexer fix), strings, and `@variable@` substitutions (Story 1's four
highlighted categories). Scoped to the language ID above; functions with zero
dependency on `drut server` being installed or running (FR-021's own wording).

## `contributes.semanticTokenTypes` / `contributes.semanticTokenModifiers`

Declares the two custom names `drut-lsp` uses beyond `lsp-types`' standard set
(`contracts/lsp-capabilities.md`'s `semantic_tokens_provider` row): `shortIf`
(type) and `unreachable` (modifier) — required so VS Code's generic
semantic-highlighting infrastructure recognizes tokens tagged with these names
at all, before any theme color is applied.

## `contributes.semanticTokenScopes`

Maps `shortIf`/`unreachable` to TextMate-style scopes so the active color theme
can style them distinctly (Story 6) without the extension hard-coding colors
itself — themes remain in control of actual appearance, only scope naming is
this extension's responsibility.

## `activationEvents`

Activates on opening a document with the registered language ID — not
`onStartupFinished` or similarly broad, so the extension has no footprint in
workspaces that never open a `.s`/`.block` file.

## `main` / client bootstrap (`src/extension.ts`)

On activation:
1. Resolve the `drut` binary (bundled, on `PATH`, or a user-configurable
   setting — implementation detail, not fixed by this contract).
2. If unresolvable, or if spawning it fails: skip steps 3–4 entirely, leaving
   Story 1's grammar-only highlighting fully functional, and show a single
   non-repeating notification (FR-025) — never block extension activation on
   the server being available.
3. Otherwise, start a `vscode-languageclient` `LanguageClient` with
   `serverOptions` spawning `drut server` over stdio, and `clientOptions`
   scoped to the registered language ID.
4. Register a crash handler: on unexpected server exit, notify once and
   attempt one restart (FR-026), without tearing down or re-registering the
   static grammar/language configuration from step 1–2, which never depended
   on the server in the first place.

## `publisher` / marketplace metadata

`publisher` is Drut's own registered VS Code Marketplace and Open VSX publisher
ID (FR-027) — never a fork, rename, or reuse of `bhereth`'s or any other
third-party publisher identity. Packaging via `@vscode/vsce package`, publishing
via `vsce publish` (Marketplace) and `ovsx publish` (Open VSX), both under that
same identity (research.md §7).

## `ensureVariableColorCustomization` (added 2026-08-10, outside original scope)

On activation, if a workspace folder is open and this workspace hasn't already
been offered it (tracked via `ExtensionContext.workspaceState`, once ever),
writes a `variable:drut-voyager` rule into the *workspace's* (never the user's
global) `editor.semanticTokenColorCustomizations` setting, merging into any
existing customization rather than replacing it, and never overwriting an
existing rule under that same key — whether the user wrote it or a prior
activation did. Exists because no TextMate scope or standard LSP semantic
token type is colored by every VS Code color theme (coloring is opt-in per
theme author); this is the only mechanism that guarantees `@variable@`
references get a visible color out of the box regardless of the active theme,
found necessary via real manual verification (spec.md's dated Assumptions
entry has the full incident). See that same entry for why this is a narrower
thing than the "no per-workspace settings" exclusion below rules out — a
color-rendering tweak to a pre-existing generic VS Code setting, not a new,
Drut-invented functional configuration surface. Best-effort: a write failure
(e.g. a read-only workspace) is swallowed silently, never blocking activation
or any other capability.

## Explicitly out of scope

- A configuration UI for per-workspace settings — spec.md Assumptions rule out
  a config file/settings surface this phase, still true for anything that
  would change the server's or extension's own *functional* behavior (see the
  narrower carve-out above, which is color-rendering only).
- Bundling the `drut` binary inside the `.vsix` package itself (vs. relying on
  `PATH`/a path setting) — an implementation choice left open by step 1 above,
  not fixed by this contract; either satisfies FR-024 as written.
