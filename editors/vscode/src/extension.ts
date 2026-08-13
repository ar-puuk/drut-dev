// Drut VS Code extension activation. See
// specs/003-lsp-vscode-extension/contracts/extension-manifest.md for the
// full contract this file implements (FR-021, FR-024-FR-026).
//
// Story 1's static highlighting (FR-021, language registration + grammar)
// needs no code here at all — it's entirely declared in package.json's
// contributes.languages/grammars, functional with zero dependency on
// anything below. Everything in this file is the LanguageClient bootstrap
// (FR-024) and its FR-025/FR-026 degrade-gracefully behavior.

import { spawnSync } from "child_process";
import * as fs from "fs";
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
import {
  archiveExtensionFor,
  findReleaseAssetMatch,
  GitHubRelease,
  isUpdateCheckDue,
  mapPlatformToTarget,
  parseGitHubSlug,
  pickResolutionTier,
  ResolutionSource,
  ResolvedBinary,
  shouldOfferUpdate,
  TargetTriple,
  verifyChecksum,
} from "./binaryBootstrap";
import { downloadToFile, extractArchive, fetchLatestRelease, sha256File } from "./bootstrapIO";
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

// --- 015-extension-binary-bootstrap ("batteries included") ----------------
//
// resolveDrutBinary() replaces the old synchronous, PATH-only
// resolveDrutCommand(). See specs/015-extension-binary-bootstrap/contracts/
// binary-bootstrap-api.md for the normative three-tier algorithm this
// implements. Every pure decision (platform->target-triple mapping,
// asset-list matching, checksum comparison, update-check throttle/offer
// logic) lives in binaryBootstrap.ts, imported below -- this file only
// contains the VS-Code-API-touching orchestration.

const UPDATE_CHECK_THROTTLE_MS = 24 * 60 * 60 * 1000; // spec.md FR-014
const GITHUB_USER_AGENT = "drut-vscode-extension"; // required by GitHub's REST API (research.md §1)

function storedBinaryFileName(): string {
  return process.platform === "win32" ? "drut.exe" : "drut";
}

/// Tier 1 (spec.md FR-002): a genuine pre-flight check using the exact
/// same primitive `vscode-languageclient` itself uses internally to spawn
/// the server (research.md §4) -- never a different resolution mechanism
/// that could disagree with what the real launch would do.
function isOnPath(command: string): boolean {
  // "--help", not "--version" -- confirmed directly against a real build
  // (015-extension-binary-bootstrap's own CI verification run, T023):
  // `drut` is subcommand-only (clap's auto `--version` flag isn't enabled
  // here) and rejects a bare `--version` with a non-zero exit. That
  // doesn't actually break this check (spawnSync only sets `.error` on a
  // genuine ENOENT, never on the child process merely exiting non-zero),
  // but "--help" is the flag that's actually guaranteed to succeed, so
  // it's the clearer probe to use.
  const result = spawnSync(command, ["--help"], { stdio: "ignore" });
  return result.error === undefined || (result.error as NodeJS.ErrnoException).code !== "ENOENT";
}

function readPackageRepositoryUrl(context: vscode.ExtensionContext): string {
  const raw = fs.readFileSync(path.join(context.extensionPath, "package.json"), "utf8");
  const parsed = JSON.parse(raw) as { repository?: { url?: string } };
  return parsed.repository?.url ?? "";
}

/// Tier 3 (spec.md FR-004-FR-010): download, verify, extract, install.
/// Throws on any failure -- callers (resolveDrutBinary, the update-accept
/// path) are responsible for translating that into the right
/// user-observable outcome (FR-011/FR-012, or the update flow's own
/// re-run of this same sequence, FR-017).
async function downloadAndInstall(
  context: vscode.ExtensionContext,
  target: TargetTriple
): Promise<{ command: string; version: string }> {
  const slug = parseGitHubSlug(readPackageRepositoryUrl(context));
  if (!slug) {
    throw new Error("could not determine the GitHub repository from package.json");
  }
  const release = await fetchLatestRelease(slug.owner, slug.repo, GITHUB_USER_AGENT);
  const ext = archiveExtensionFor(target);
  const match = findReleaseAssetMatch(release.assets, target, ext);
  if (!match) {
    throw new Error(`release ${release.tag_name} has no asset for ${target}`);
  }

  const tmpDir = path.join(context.globalStorageUri.fsPath, ".tmp");
  fs.mkdirSync(tmpDir, { recursive: true });
  const archivePath = path.join(tmpDir, match.binary.name);
  const checksumPath = path.join(tmpDir, match.checksum.name);

  await downloadToFile(match.binary.browser_download_url, archivePath, GITHUB_USER_AGENT);
  await downloadToFile(match.checksum.browser_download_url, checksumPath, GITHUB_USER_AGENT);

  const digest = sha256File(archivePath);
  const expected = fs.readFileSync(checksumPath, "utf8");
  if (!verifyChecksum(digest, expected)) {
    throw new Error("checksum verification failed for downloaded binary");
  }

  const extractedTmpPath = path.join(tmpDir, storedBinaryFileName());
  await extractArchive(archivePath, extractedTmpPath, target);

  const finalPath = path.join(context.globalStorageUri.fsPath, storedBinaryFileName());
  fs.renameSync(extractedTmpPath, finalPath); // atomic within globalStorageUri (research.md §6)

  await context.globalState.update("drutInstalledVersion", release.tag_name);
  await context.globalState.update("drutInstalledPlatformArch", `${process.platform}-${process.arch}`);

  return { command: finalPath, version: release.tag_name };
}

/// The full three-tier resolution algorithm (spec.md FR-001). Returns
/// `undefined` when every tier fails or is inapplicable -- the caller's
/// job is then exactly today's degrade-to-highlighting-only path, not a
/// new error shape (data-model.md).
async function resolveDrutBinary(context: vscode.ExtensionContext): Promise<ResolvedBinary | undefined> {
  // Compute every tier's raw input *before* deciding -- the decision
  // itself is delegated to pickResolutionTier (binaryBootstrap.ts), the
  // same pure function User Story 2's own dedicated test exercises. This
  // is deliberate: the tested decision function is the *actual* decision
  // function used at runtime, not a parallel copy that could drift from
  // what's really wired up.
  const pathFound = isOnPath("drut");
  const target = mapPlatformToTarget(process.platform, process.arch);
  const storedPath = path.join(context.globalStorageUri.fsPath, storedBinaryFileName());
  const storedPlatformArch = context.globalState.get<string>("drutInstalledPlatformArch");
  const storedBinaryValid =
    fs.existsSync(storedPath) && storedPlatformArch === `${process.platform}-${process.arch}`;

  const tier = pickResolutionTier(pathFound, storedBinaryValid, target);

  if (tier === "path") {
    return { command: "drut", source: "path" };
  }
  if (tier === "storage") {
    return { command: storedPath, source: "storage" };
  }
  if (tier === "unsupported") {
    notifyOnce(
      "unsupported-platform",
      `Drut doesn't publish a prebuilt language server binary for your platform (${process.platform}/${process.arch}) — syntax highlighting still works, but diagnostics/hover/completion/formatting are unavailable.`
    );
    return undefined;
  }

  // tier === "download". `target` is guaranteed non-null here -- the only
  // way pickResolutionTier returns "download" instead of "unsupported".
  try {
    fs.mkdirSync(context.globalStorageUri.fsPath, { recursive: true });
    const { command } = await downloadAndInstall(context, target as TargetTriple);
    return { command, source: "downloaded" };
  } catch (err) {
    notifyOnce(
      "download-failed",
      `Drut couldn't download the language server binary (${
        err instanceof Error ? err.message : String(err)
      }) — syntax highlighting still works; this will be retried the next time you open VS Code.`
    );
    return undefined;
  }
}

/// User Story 4 (spec.md FR-013-FR-017): throttled, storage-only-scoped,
/// non-blocking background update check. Fire-and-forget -- called after
/// client.start(), never before (FR-014).
async function checkForUpdateInBackground(context: vscode.ExtensionContext, source: ResolutionSource): Promise<void> {
  if (source !== "storage") return; // never second-guesses a PATH install (FR-013)

  const lastChecked = context.globalState.get<number>("drutLastUpdateCheckMs");
  if (!isUpdateCheckDue(lastChecked, Date.now(), UPDATE_CHECK_THROTTLE_MS)) return;
  await context.globalState.update("drutLastUpdateCheckMs", Date.now());

  const slug = parseGitHubSlug(readPackageRepositoryUrl(context));
  if (!slug) return;

  let release: GitHubRelease;
  try {
    release = await fetchLatestRelease(slug.owner, slug.repo, GITHUB_USER_AGENT);
  } catch {
    // Best-effort background check -- a failure here changes nothing
    // about the user's already-working setup, so it's silent by design
    // (spec.md Assumptions), unlike an initial-resolution failure.
    return;
  }

  const installed = context.globalState.get<string>("drutInstalledVersion");
  const declined = context.globalState.get<string>("drutDeclinedUpdateVersion");
  if (!shouldOfferUpdate(installed, release.tag_name, declined)) return;

  const choice = await vscode.window.showInformationMessage(
    `Drut ${release.tag_name} is available (you have ${installed}).`,
    "Update",
    "Later"
  );

  if (choice === "Update") {
    const target = mapPlatformToTarget(process.platform, process.arch);
    if (target === null) return; // shouldn't happen if we got this far, but never trust it silently
    // Stop first: serverOptions is fixed at LanguageClient construction
    // time (no setter, no restart(newOptions) -- verified against the
    // real vscode-languageclient API), and a running process locks its
    // own executable on Windows, so the binary must not be touched while
    // the old server is still alive (spec.md, /speckit-analyze finding).
    await client?.stop();
    try {
      const { command } = await downloadAndInstall(context, target);
      startLanguageClient(command);
    } catch (err) {
      notifyOnce(
        "download-failed",
        `Drut couldn't install the update (${
          err instanceof Error ? err.message : String(err)
        }) — the previous version is no longer running; reload the window to retry.`
      );
    }
  } else {
    await context.globalState.update("drutDeclinedUpdateVersion", release.tag_name);
  }
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
/// offered the auto-injected `variable:drut` color rule — checked
/// so the injection happens at most once ever per workspace, never
/// reapplied on a later activation. This is what makes it safe to remove:
/// a user who deletes the injected setting from `.vscode/settings.json`
/// stays deleted, forever, for that workspace — the extension never fights
/// that choice back.
const VARIABLE_COLOR_INJECTED_KEY = "drutVariableColorInjected";

/// The scoped semantic-token-color rule key this function injects —
/// `variable:drut` colors only `variable`-typed tokens in Drut
/// documents, never touching semantic "variable" coloring in any other
/// language the user might also have open.
const VARIABLE_COLOR_RULE_KEY = "variable:drut";
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
/// overwrites an existing `variable:drut` rule if one is already
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
/// own rationale for why the `"[drut]"`-object-merge trick above
/// doesn't apply here). Deliberately does *not* touch
/// `editor.formatOnPaste` -- that setting stays opt-in/documented-only,
/// per the same resolved clarification (contracts/extension-settings.md).
async function ensureFormatOnSaveEnabled(context: vscode.ExtensionContext): Promise<void> {
  if (!vscode.workspace.workspaceFolders || vscode.workspace.workspaceFolders.length === 0) {
    return; // Same guard as ensureVariableColorCustomization -- nothing to write into.
  }

  const alreadyInjected = context.workspaceState.get<boolean>(FORMAT_ON_SAVE_INJECTED_KEY) ?? false;

  try {
    const config = vscode.workspace.getConfiguration(undefined, { languageId: "drut" });
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

/// Builds and starts the LanguageClient against `command`. Extracted so
/// the update-accept path (checkForUpdateInBackground) can construct a
/// genuinely new client pointed at a new binary — `serverOptions` is fixed
/// at LanguageClient construction time with no setter, so "restarting
/// against a new binary" means calling this again, not mutating anything
/// on the existing instance (015-extension-binary-bootstrap,
/// /speckit-analyze finding).
function startLanguageClient(command: string): void {
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
    documentSelector: [{ scheme: "file", language: "drut" }],
    errorHandler: new OneRestartErrorHandler(),
  };

  client = new LanguageClient("drut", "Drut Language Server", serverOptions, clientOptions);

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

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  void ensureVariableColorCustomization(context);
  void ensureFormatOnSaveEnabled(context);

  const resolved = await resolveDrutBinary(context);
  if (resolved === undefined) {
    // Every tier failed or was inapplicable (spec.md FR-011) — the
    // appropriate notification was already fired inside
    // resolveDrutBinary itself. Static highlighting remains fully
    // functional; there is nothing further to do here.
    return;
  }

  startLanguageClient(resolved.command);

  // Fire-and-forget, after start() — never blocks/delays activation
  // (spec.md FR-014).
  void checkForUpdateInBackground(context, resolved.source);
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
