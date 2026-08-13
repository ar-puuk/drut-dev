# Phase 1 Data Model: Extension Binary Bootstrap ("Batteries Included")

## New: `TargetTriple` (the 4 values D2 actually publishes)

```typescript
// One of exactly these four string literals -- matches D2's release.yml
// matrix verbatim (research.md §5). Not an open string type: a
// platform/arch pair that doesn't map to one of these four is unsupported,
// full stop (spec.md FR-004).
type TargetTriple =
  | "x86_64-pc-windows-msvc"
  | "aarch64-apple-darwin"
  | "x86_64-apple-darwin"
  | "x86_64-unknown-linux-gnu";
```

## New: `ResolutionSource`

```typescript
// Which of the three priority-ordered mechanisms supplied the binary for
// this activation (spec.md Key Entities). Drives whether the background
// update check runs at all (only "storage" -- FR-013).
type ResolutionSource = "path" | "storage" | "downloaded";

interface ResolvedBinary {
  command: string; // bare "drut" for "path"; an absolute path for the other two
  source: ResolutionSource;
}
```

## New: `ReleaseAsset` / `ReleaseAssetMatch`

```typescript
// The shape actually returned by GET /repos/{owner}/{repo}/releases/latest
// (research.md §5) -- only the two fields this feature reads.
interface ReleaseAsset {
  name: string;
  browser_download_url: string;
}

interface GitHubRelease {
  tag_name: string; // e.g. "v0.1.0"
  assets: ReleaseAsset[];
}

// The result of matching a release's real asset list against a target
// triple + expected extension -- both the binary and its checksum sidecar
// must be found by exact name, or this is null (spec.md FR-005).
interface ReleaseAssetMatch {
  binary: ReleaseAsset;
  checksum: ReleaseAsset;
}
```

## New: extension-storage-persisted state (`context.globalState`)

```typescript
// research.md §7. All optional/undefined on a machine that has never
// downloaded a binary -- absence is a valid, common state, not an error.
interface BootstrapState {
  drutInstalledVersion?: string;        // e.g. "0.1.0" -- the tag last installed (FR-010)
  drutInstalledPlatformArch?: string;   // e.g. "win32-x64" -- mismatch => treat as absent
  drutLastUpdateCheckMs?: number;       // epoch ms; throttles FR-014's background check
  drutDeclinedUpdateVersion?: string;   // FR-016's "don't re-offer this exact version"
}
```

## Changed: `extension.ts`'s activation flow

```typescript
// Before: synchronous, PATH-only.
function resolveDrutCommand(): string {
  return "drut";
}

// After: async, three-tier (spec.md FR-001).
async function resolveDrutBinary(context: vscode.ExtensionContext): Promise<ResolvedBinary | undefined> {
  // undefined -- rather than throwing -- represents "every tier failed or
  // was inapplicable" (spec.md FR-011): the caller's job is then exactly
  // today's degrade-to-highlighting-only path, not a new error shape.
}
```

`activate()` becomes `async`, `await`s `resolveDrutBinary`, and only builds
`serverOptions`/starts the client when a `ResolvedBinary` came back —
`undefined` skips straight to the existing highlighting-only state, with the
specific notification already fired by `resolveDrutBinary` itself
(`unsupported-platform` or `download-failed`, per which tier actually
failed — spec.md FR-012). The background update check
(`checkForUpdateInBackground`, fire-and-forget, only when `source ===
"storage"`) is kicked off *after* `client.start()`, never before.

## Explicitly unchanged

- `notifyOnce`, `OneRestartErrorHandler`, `ensureVariableColorCustomization`,
  `ensureFormatOnSaveEnabled`, `shouldInjectFormatOnSave` — untouched.
  `notifyOnce` gains two new call sites (new kind strings), not a changed
  signature or mechanism.
- `serverOptions`'s shape (`{ command, args: ["server"] }`) — unchanged;
  only how `command` is *obtained* changes, not what's done with it.
- `package.json`'s `dependencies`/`devDependencies` — unchanged (research.md
  §1-§3's zero-new-dependency decisions).
