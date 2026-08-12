// Pure decision logic for whether to auto-inject the workspace-scoped
// editor.formatOnSave override (contracts/extension-settings.md,
// specs/005-format-on-save-paste). Deliberately kept in its own module with
// zero dependency on the `vscode` package -- the real "vscode" module only
// resolves inside a running extension host, so a function meant to be
// unit-testable via plain ts-node (test/formatOnSave.test.ts, mirroring
// test/grammar.test.ts's existing standalone convention) cannot import
// anything that pulls it in, even transitively. This is a refinement made
// during implementation, not part of the original plan.md/tasks.md file
// layout (which described this predicate as living directly in
// extension.ts) -- extension.ts's ensureFormatOnSaveEnabled is the
// effectful wrapper that imports this function and calls it with real
// values read from the VS Code configuration API.

/**
 * Decides whether `ensureFormatOnSaveEnabled` should write the
 * `editor.formatOnSave` workspace-language override.
 *
 * - `alreadyInjected`: this workspace's `drutFormatOnSaveInjected`
 *   `workspaceState` flag -- `true` once this extension has ever attempted
 *   the injection in this workspace, regardless of outcome.
 * - `existingWorkspaceLanguageValue`: the current
 *   `config.inspect("editor.formatOnSave").workspaceLanguageValue` for the
 *   `drut-voyager` language override -- `undefined` iff no such override
 *   exists yet, at any value (research.md §3: `inspect`, not `get`, since
 *   `get` would report the merged effective value from some unrelated
 *   scope the extension has no business overriding).
 *
 * Returns `true` only when neither guard has already been tripped --
 * never injected before in this workspace, and no explicit language-scoped
 * override (of either polarity) already exists. This single predicate is
 * what makes FR-004 (auto-enable) and FR-006 (never fight a user's
 * override) the same mechanism rather than two separately-coded rules.
 */
export function shouldInjectFormatOnSave(
  alreadyInjected: boolean,
  existingWorkspaceLanguageValue: unknown
): boolean {
  return !alreadyInjected && existingWorkspaceLanguageValue === undefined;
}
