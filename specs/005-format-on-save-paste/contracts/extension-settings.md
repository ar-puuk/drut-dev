# Contract: `editors/vscode` — format-on-save injection & format-on-paste opt-in

## Format-on-save: `ensureFormatOnSaveEnabled`

New function in `src/extension.ts`, called from `activate()` alongside the
existing `ensureVariableColorCustomization` — both are best-effort,
one-time, workspace-scoped injections that must never block or fail
activation.

```typescript
const FORMAT_ON_SAVE_INJECTED_KEY = "drutFormatOnSaveInjected";

async function ensureFormatOnSaveEnabled(context: vscode.ExtensionContext): Promise<void> {
  if (!vscode.workspace.workspaceFolders || vscode.workspace.workspaceFolders.length === 0) {
    return; // same guard as ensureVariableColorCustomization — nothing to write into
  }
  if (context.workspaceState.get<boolean>(FORMAT_ON_SAVE_INJECTED_KEY)) {
    return;
  }

  try {
    const config = vscode.workspace.getConfiguration(undefined, { languageId: "drut-voyager" });
    const existing = config.inspect<boolean>("editor.formatOnSave");
    if (existing?.workspaceLanguageValue === undefined) {
      await config.update(
        "editor.formatOnSave",
        true,
        vscode.ConfigurationTarget.Workspace,
        /* overrideInLanguage */ true
      );
    }
  } catch {
    // Never let this best-effort convenience fail extension activation.
  }

  await context.workspaceState.update(FORMAT_ON_SAVE_INJECTED_KEY, true);
}
```

| Rule | Enforced by |
|---|---|
| Never overwrites an explicit existing override (user-set, in either direction, or from an earlier extension run) | `existing?.workspaceLanguageValue === undefined` check (research.md §3 — `inspect`, not `get`) |
| Written at workspace scope only, never global | `vscode.ConfigurationTarget.Workspace` |
| Scoped to `.s`/`.block` files only, never every language in the workspace | `getConfiguration(undefined, { languageId: "drut-voyager" })` + `overrideInLanguage: true` (research.md §3) — **not** `ensureVariableColorCustomization`'s object-merge pattern, which doesn't apply here |
| Attempted at most once per workspace, ever | `workspaceState` gate, checked before and set after |
| Never blocks/fails activation | `try`/`catch`, same as the existing color-injection precedent |
| A user who removes/disables it afterward stays that way forever for this workspace | No mechanism in this function ever re-checks after the first activation — verified by User Story 3's acceptance scenario in spec.md |

`activate()` gains one line, directly alongside the existing call:

```typescript
export function activate(context: vscode.ExtensionContext): void {
  void ensureVariableColorCustomization(context);
  void ensureFormatOnSaveEnabled(context);
  ...
```

## Format-on-paste: documentation only, no injection code

Per Clarification Q1 (Option C), `editor.formatOnPaste` is **never**
written by the extension. `README.md` (or `editors/vscode/README.md`, the
extension's own marketplace description — whichever already carries
user-facing usage instructions) gains a short instruction:

```json
{
  "[drut-voyager]": {
    "editor.formatOnPaste": true
  }
}
```

with one sentence of context: pasted Cube Voyager script text will be
reformatted to match its surrounding indentation once this setting is on.
No `package.json` `contributes.configuration` entry is needed —
`editor.formatOnPaste` is a VS Code built-in setting, not one this
extension defines; the extension's only involvement is that the capability
it now serves (`textDocument/rangeFormatting`) is what makes turning the
setting on actually do something for `.s`/`.block` files.
