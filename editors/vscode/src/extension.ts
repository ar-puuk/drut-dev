// Drut VS Code extension activation. See
// specs/003-lsp-vscode-extension/contracts/extension-manifest.md for the
// full contract this file implements (FR-021, FR-024-FR-026).
//
// Story 1's static highlighting (FR-021, language registration + grammar)
// needs no code here at all — it's entirely declared in package.json's
// contributes.languages/grammars, functional with zero dependency on
// anything below. Everything in this file is the LanguageClient bootstrap
// (FR-024) and its FR-025/FR-026 degrade-gracefully behavior.

import * as path from "path";
import * as vscode from "vscode";
import {
  CloseAction,
  CloseHandlerResult,
  ErrorAction,
  ErrorHandlerResult,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";
import { shouldInjectFormatOnSave } from "./formatOnSaveDecision";

let client: LanguageClient | undefined;

/// FR-025's "single, non-repeating notification per distinct failure" —
/// tracked per failure kind so a missing binary and a later crash are each
/// reported once, but the same ongoing cause is never re-notified.
const notifiedFailureKinds = new Set<string>();

function notifyOnce(kind: string, message: string): void {
  if (notifiedFailureKinds.has(kind)) {
    return;
  }
  notifiedFailureKinds.add(kind);
  vscode.window.showWarningMessage(message);
}

/// Resolves the `drut` binary via `PATH` — relying on Node's own
/// `child_process` PATH search, which already accounts for platform
/// PATH/`.exe` conventions correctly (spec.md Edge Cases), no bespoke
/// per-platform logic needed here. No settings-based override: spec.md's
/// Assumptions rule out any configuration surface this phase ("the server
/// and extension behave the same way across every workspace").
function resolveDrutCommand(): string {
  return "drut";
}

/// FR-026: allow exactly one automatic restart per crash occurrence, then
/// stop — a second, still-failing attempt is a distinct, separately-
/// notified failure (FR-025), not a further automatic retry.
class OneRestartErrorHandler {
  private restarted = false;

  error(_error: Error, _message: unknown, _count: number | undefined): ErrorHandlerResult {
    return { action: ErrorAction.Continue };
  }

  closed(): CloseHandlerResult {
    if (!this.restarted) {
      this.restarted = true;
      notifyOnce("crash", "Drut language server crashed — attempting to restart it.");
      return { action: CloseAction.Restart };
    }
    notifyOnce(
      "crash-restart-failed",
      "Drut language server crashed again after restarting — it will not be restarted automatically. Static highlighting is unaffected."
    );
    return { action: CloseAction.DoNotRestart };
  }
}

/// Workspace-state key tracking whether this workspace has already been
/// offered the auto-injected `variable:drut-voyager` color rule — checked
/// so the injection happens at most once ever per workspace, never
/// reapplied on a later activation. This is what makes it safe to remove:
/// a user who deletes the injected setting from `.vscode/settings.json`
/// stays deleted, forever, for that workspace — the extension never fights
/// that choice back.
const VARIABLE_COLOR_INJECTED_KEY = "drutVariableColorInjected";

/// The scoped semantic-token-color rule key this function injects —
/// `variable:drut-voyager` colors only `variable`-typed tokens in Drut
/// documents, never touching semantic "variable" coloring in any other
/// language the user might also have open.
const VARIABLE_COLOR_RULE_KEY = "variable:drut-voyager";
const VARIABLE_COLOR_VALUE = "#4EC9B0";

/// Guarantees `@name@` references get a visible color the first time this
/// extension activates in a workspace, regardless of the active color
/// theme's own rules — added 2026-08-10, found necessary via real manual
/// VS Code testing (see spec.md's dated Assumptions entry): no TextMate
/// scope or standard LSP semantic token type is colored by every theme —
/// coloring is opt-in per theme author, a structural property of VS Code's
/// theming model, not something a "better" scope name can fix. Injects
/// into this *workspace's* `editor.semanticTokenColorCustomizations`
/// setting (`.vscode/settings.json`), never the user's global settings —
/// scoped to the project asking for it, visible/inspectable, and trivial
/// to remove or override by hand. Merges into any existing customization
/// object rather than overwriting it (so a user's own unrelated semantic
/// color rules for other languages are never clobbered), and never
/// overwrites an existing `variable:drut-voyager` rule if one is already
/// present — whether the user set it themselves or this function did on
/// an earlier activation.
async function ensureVariableColorCustomization(context: vscode.ExtensionContext): Promise<void> {
  if (!vscode.workspace.workspaceFolders || vscode.workspace.workspaceFolders.length === 0) {
    return; // A single loose file with no workspace folder open — nothing to write into.
  }
  if (context.workspaceState.get<boolean>(VARIABLE_COLOR_INJECTED_KEY)) {
    return;
  }

  try {
    const config = vscode.workspace.getConfiguration();
    const current =
      config.get<{ rules?: Record<string, unknown> }>("editor.semanticTokenColorCustomizations") ?? {};
    const rules = current.rules ?? {};

    if (!(VARIABLE_COLOR_RULE_KEY in rules)) {
      await config.update(
        "editor.semanticTokenColorCustomizations",
        { ...current, rules: { ...rules, [VARIABLE_COLOR_RULE_KEY]: VARIABLE_COLOR_VALUE } },
        vscode.ConfigurationTarget.Workspace
      );
    }
  } catch {
    // Never let this best-effort convenience fail extension activation —
    // static highlighting, diagnostics, hover, completion, and formatting
    // are all fully independent of whether this write succeeds.
  }

  await context.workspaceState.update(VARIABLE_COLOR_INJECTED_KEY, true);
}

/// Workspace-state key tracking whether this workspace has already been
/// offered the auto-enabled `editor.formatOnSave` override -- same
/// one-time-ever, never-reapplied lifecycle as
/// `VARIABLE_COLOR_INJECTED_KEY` above (specs/005-format-on-save-paste
/// FR-006, contracts/extension-settings.md).
const FORMAT_ON_SAVE_INJECTED_KEY = "drutFormatOnSaveInjected";

/// Auto-enables `editor.formatOnSave` for `.s`/`.block` files the first
/// time this extension activates in a workspace (specs/005-format-on-save-paste
/// FR-004, Clarification Q1 Option C) -- unlike the color customization
/// above, this writes a genuine VS Code language-scoped setting override
/// (`getConfiguration(undefined, { languageId }).update(..., /*
/// overrideInLanguage */ true)`, research.md §3), not a value nested inside
/// one particular setting's own rule-map convention -- the two mechanisms
/// solve different problems and are not interchangeable (research.md §3's
/// own rationale for why the `"[drut-voyager]"`-object-merge trick above
/// doesn't apply here). Deliberately does *not* touch
/// `editor.formatOnPaste` -- that setting stays opt-in/documented-only,
/// per the same resolved clarification (contracts/extension-settings.md).
async function ensureFormatOnSaveEnabled(context: vscode.ExtensionContext): Promise<void> {
  if (!vscode.workspace.workspaceFolders || vscode.workspace.workspaceFolders.length === 0) {
    return; // Same guard as ensureVariableColorCustomization -- nothing to write into.
  }

  const alreadyInjected = context.workspaceState.get<boolean>(FORMAT_ON_SAVE_INJECTED_KEY) ?? false;

  try {
    const config = vscode.workspace.getConfiguration(undefined, { languageId: "drut-voyager" });
    const existing = config.inspect<boolean>("editor.formatOnSave");
    // shouldInjectFormatOnSave is the single source of truth for this
    // decision (T002's own unit tests) -- no duplicated guard here.
    if (shouldInjectFormatOnSave(alreadyInjected, existing?.workspaceLanguageValue)) {
      await config.update(
        "editor.formatOnSave",
        true,
        vscode.ConfigurationTarget.Workspace,
        /* overrideInLanguage */ true
      );
    }
  } catch {
    // Never let this best-effort convenience fail extension activation --
    // same discipline as ensureVariableColorCustomization above.
  }

  await context.workspaceState.update(FORMAT_ON_SAVE_INJECTED_KEY, true);
}

export function activate(context: vscode.ExtensionContext): void {
  void ensureVariableColorCustomization(context);
  void ensureFormatOnSaveEnabled(context);

  const command = resolveDrutCommand();

  // No `transport` field here (deliberately, found the hard way
  // 2026-08-10 during manual VS Code verification): `vscode-languageclient`
  // treats `TransportKind.stdio` as a signal to append a `--stdio` flag to
  // `args` before spawning — a convention plenty of language servers
  // (rust-analyzer, clangd, ...) opt into, but `drut server` never asked
  // for or accepts (`cli.rs`'s `Server` variant is a bare, flagless
  // subcommand). With that field set, every launch failed immediately —
  // `clap` rejected the injected `--stdio` and the process exited before
  // the LSP handshake even began, which surfaced as a same-line crash-and-
  // permanent-give-up in the client log, not a working connection. A plain
  // `command`+`args` `ServerOptions` with no `transport` field already
  // communicates over stdio by default — this is that default, not a
  // withheld capability.
  const serverOptions: ServerOptions = {
    command,
    args: ["server"],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "drut-voyager" }],
    errorHandler: new OneRestartErrorHandler(),
  };

  client = new LanguageClient("drut-voyager", "Drut Language Server", serverOptions, clientOptions);

  // FR-025: if the binary can't be found or fails to start, `start()`
  // rejects — static highlighting (package.json's grammar contribution)
  // is entirely independent of this and stays fully functional either way.
  client.start().then(undefined, (err: unknown) => {
    notifyOnce(
      "missing-binary",
      `Could not start the Drut language server (${command} server) — syntax highlighting still works, but diagnostics/hover/completion are unavailable. ${
        err instanceof Error ? err.message : String(err)
      }`
    );
  });
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
