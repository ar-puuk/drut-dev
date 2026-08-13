# Contract: Extension Binary Bootstrap ("Batteries Included")

Normative for `editors/vscode/src/binaryBootstrap.ts` (pure functions,
independently testable) and `editors/vscode/src/extension.ts`'s
orchestration (impure, VS Code API-dependent). New file, not an amendment —
`003-lsp-vscode-extension/contracts/extension-manifest.md` remains the base
contract for everything else in `extension.ts`.

## Pure functions (`binaryBootstrap.ts`) — normative signatures

```typescript
/// spec.md FR-004. Returns null for anything outside D2's real 4-target
/// matrix -- never a guessed/best-effort triple.
function mapPlatformToTarget(platform: NodeJS.Platform, arch: string): TargetTriple | null;
// Exhaustive table (research.md §5, cross-checked directly against
// release.yml's matrix, not re-derived from memory):
//   ("win32",  "x64")   -> "x86_64-pc-windows-msvc"
//   ("darwin", "arm64") -> "aarch64-apple-darwin"
//   ("darwin", "x64")   -> "x86_64-apple-darwin"
//   ("linux",  "x64")   -> "x86_64-unknown-linux-gnu"
//   everything else     -> null

/// spec.md FR-004. ".zip" for the Windows target, ".gz" for the other
/// three -- matches D2's real per-target extension exactly, not a
/// per-OS-family guess independent of it.
function archiveExtensionFor(target: TargetTriple): "zip" | "gz";

/// spec.md FR-005. Exact-name match against a real release's asset list.
/// Returns null if either the binary or its .sha256 sidecar is missing --
/// a partial match (one found, one not) is still "no match," never a
/// half-verified install.
function findReleaseAssetMatch(
  assets: ReleaseAsset[],
  target: TargetTriple,
  ext: "zip" | "gz"
): ReleaseAssetMatch | null;
// Expected names: `drut-${target}.${ext}` and `drut-${target}.${ext}.sha256`
// -- matched by exact equality against `assets[].name`, never constructed
// and trusted without confirming presence in the real list first.

/// spec.md FR-006. Case-insensitive hex comparison (sha256sum/
/// Get-FileHash's own output casing conventions differ across platforms --
/// D2's own Unix step lowercases, Windows step also lowercases explicitly,
/// but this function does its own case-insensitive compare regardless of
/// what either side happens to produce).
function verifyChecksum(computedHexDigest: string, expectedFileContent: string): boolean;
// expectedFileContent is the raw .sha256 sidecar's content, e.g.
// "<hex>  drut-x86_64-unknown-linux-gnu.gz\n" -- this function extracts
// just the hex portion (first whitespace-delimited token) before comparing.

/// spec.md FR-014. Pure boundary check, no Date.now() call inside --
/// caller passes `now` explicitly for testability.
function isUpdateCheckDue(lastCheckedMs: number | undefined, nowMs: number, throttleMs: number): boolean;
// undefined lastCheckedMs => always due (never checked before).

/// spec.md FR-015/FR-016. Encapsulates "is this version newer, and have we
/// already been told no for exactly this version."
function shouldOfferUpdate(
  installedVersion: string,
  latestVersion: string,
  declinedVersion: string | undefined
): boolean;
// latestVersion === declinedVersion => false (already declined, don't
// re-nag). latestVersion is newer than installedVersion (research.md §8's
// comparator) AND latestVersion !== declinedVersion => true.

/// spec.md FR-018. Parses "https://github.com/<owner>/<repo>.git" (or
/// without the trailing ".git") out of package.json's own repository.url
/// -- never a second hardcoded "ar-puuk/drut-dev" string living
/// independently of that field.
function parseGitHubSlug(repositoryUrl: string): { owner: string; repo: string } | null;
```

## Orchestration (`extension.ts`) — normative algorithm

```text
resolveDrutBinary(context):
  # Tier 1: PATH (spec.md FR-002 -- unchanged behavior, never overridden)
  if spawnSync("drut", ["--version"], {stdio: "ignore"}).error?.code != "ENOENT":
    return { command: "drut", source: "path" }

  # Tier 2: extension storage (spec.md FR-003)
  target = mapPlatformToTarget(process.platform, process.arch)
  storedPath = globalStorageUri / (win32 ? "drut.exe" : "drut")
  if target != null
     and fileExists(storedPath)
     and globalState.drutInstalledPlatformArch == `${process.platform}-${process.arch}`:
    return { command: storedPath, source: "storage" }
  # (mismatch or missing => falls through, treated as absent -- spec.md Edge Cases)

  # Tier 3: download (spec.md FR-004-FR-010)
  if target == null:
    notifyOnce("unsupported-platform", ...)  # spec.md FR-012, US3 Scenario 1
    return undefined

  try:
    release = fetchLatestRelease(owner, repo)          # research.md §5
    match = findReleaseAssetMatch(release.assets, target, archiveExtensionFor(target))
    if match == null: raise
    downloadTo(tmpDir/binary, match.binary.browser_download_url)
    downloadTo(tmpDir/checksum, match.checksum.browser_download_url)
    if not verifyChecksum(sha256(tmpDir/binary), read(tmpDir/checksum)): raise   # FR-006
    extractedPath = decompress(tmpDir/binary, target)   # research.md §2/§3
    if not windows: chmod(extractedPath, 0o755)          # FR-008
    atomicRename(extractedPath, storedPath)               # FR-009, research.md §6
    globalState.drutInstalledVersion = release.tag_name
    globalState.drutInstalledPlatformArch = `${process.platform}-${process.arch}`
    return { command: storedPath, source: "downloaded" }
  except:
    notifyOnce("download-failed", ...)  # spec.md FR-012, US3 Scenarios 2-3
    return undefined

checkForUpdateInBackground(context, source):   # fire-and-forget, called after client.start()
  if source != "storage": return                                    # FR-013
  if not isUpdateCheckDue(globalState.drutLastUpdateCheckMs, Date.now(), 24h): return  # FR-014
  globalState.drutLastUpdateCheckMs = Date.now()
  release = fetchLatestRelease(owner, repo)   # failure here is silent -- this
                                               # is a best-effort background
                                               # check, not a user-facing
                                               # failure path (spec.md doesn't
                                               # require a notification for a
                                               # failed *update check*, only
                                               # for a failed *initial* resolve)
  if shouldOfferUpdate(globalState.drutInstalledVersion, release.tag_name, globalState.drutDeclinedUpdateVersion):
    choice = showInformationMessage(`Drut ${release.tag_name} is available...`, "Update", "Later")
    if choice == "Update":
      await client.stop()                                                # MUST happen first --
                                                                           # serverOptions is fixed at
                                                                           # LanguageClient construction
                                                                           # time (no setter, no
                                                                           # restart(newOptions) --
                                                                           # verified against the real
                                                                           # vscode-languageclient API),
                                                                           # and a running process locks
                                                                           # its own executable on
                                                                           # Windows, so the binary must
                                                                           # not be touched while the old
                                                                           # server is still alive
      re-run the download/verify/extract/install steps for `release`      # FR-017
      client = new LanguageClient(..., { command: newBinaryPath, args: ["server"] }, ...)
      await client.start()
    else:
      globalState.drutDeclinedUpdateVersion = release.tag_name             # FR-016
```

## Notification kinds (extends `notifyOnce`, no mechanism change)

| Kind | Fires when | Message covers |
|---|---|---|
| `missing-binary` | *(existing, unchanged)* a resolved binary (any tier) still fails to launch | generic launch failure |
| `unsupported-platform` | *(new)* `mapPlatformToTarget` returns `null` | names the actual platform/arch, explains no prebuilt binary exists |
| `download-failed` | *(new)* any failure in Tier 3 (network, 404, asset not found, checksum mismatch) | explains the download/verification couldn't complete, highlighting still works |

## Explicitly out of scope (spec.md Assumptions)

- No settings surface (custom binary path override, disable-auto-download,
  configurable update-check interval).
- No accumulation of multiple installed versions side-by-side.
- No authentication/token use against the GitHub API.
