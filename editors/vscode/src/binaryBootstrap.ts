// Pure, VS-Code-API-independent decision logic for the extension binary
// bootstrap ("batteries included", 015-extension-binary-bootstrap). Every
// function here is directly unit-testable via `ts-node test/
// binaryBootstrap.test.ts` -- the same pure/impure split
// formatOnSaveDecision.ts already established, applied to a much larger
// feature. See specs/015-extension-binary-bootstrap/contracts/
// binary-bootstrap-api.md for the normative signatures this file
// implements.

/// The 4 target triples D2's release.yml matrix actually publishes --
/// confirmed by reading that file directly, not guessed (research.md §5).
export type TargetTriple =
  | "x86_64-pc-windows-msvc"
  | "aarch64-apple-darwin"
  | "x86_64-apple-darwin"
  | "x86_64-unknown-linux-gnu";

export type ResolutionSource = "path" | "storage" | "downloaded";

export interface ResolvedBinary {
  command: string;
  source: ResolutionSource;
}

export type ResolutionTier = "path" | "storage" | "unsupported" | "download";

/// spec.md FR-001/FR-002 (User Story 2's own priority-order guarantee).
/// Pure: takes only the already-computed inputs, decides the tier, and
/// does none of the (impure) I/O itself. Extracted specifically so User
/// Story 2's "Tier 1 short-circuits past Tier 2/3" claim is genuinely
/// testable, not just structurally true by inspection of a `return`
/// statement -- `pathFound: true` must yield `"path"` regardless of what
/// `storedBinaryValid`/`target` are, proving the decision doesn't even
/// need Tier 2/3's own inputs to be meaningful once Tier 1 has already
/// succeeded.
export function pickResolutionTier(
  pathFound: boolean,
  storedBinaryValid: boolean,
  target: TargetTriple | null
): ResolutionTier {
  if (pathFound) return "path";
  if (storedBinaryValid) return "storage";
  if (target === null) return "unsupported";
  return "download";
}

export interface ReleaseAsset {
  name: string;
  browser_download_url: string;
}

export interface GitHubRelease {
  tag_name: string;
  assets: ReleaseAsset[];
}

export interface ReleaseAssetMatch {
  binary: ReleaseAsset;
  checksum: ReleaseAsset;
}

/// spec.md FR-004. Returns null for anything outside D2's real 4-target
/// matrix -- never a guessed/best-effort triple. Exhaustive table, not a
/// heuristic.
export function mapPlatformToTarget(platform: string, arch: string): TargetTriple | null {
  if (platform === "win32" && arch === "x64") return "x86_64-pc-windows-msvc";
  if (platform === "darwin" && arch === "arm64") return "aarch64-apple-darwin";
  if (platform === "darwin" && arch === "x64") return "x86_64-apple-darwin";
  if (platform === "linux" && arch === "x64") return "x86_64-unknown-linux-gnu";
  return null;
}

/// spec.md FR-004/FR-007. ".zip" for the Windows target, ".gz" for the
/// other three -- matches D2's real per-target extension exactly.
export function archiveExtensionFor(target: TargetTriple): "zip" | "gz" {
  return target === "x86_64-pc-windows-msvc" ? "zip" : "gz";
}

/// spec.md FR-005. Exact-name match against a real release's asset list.
/// A partial match (only one of the two names present) is still "no
/// match" -- never a half-verified install.
export function findReleaseAssetMatch(
  assets: ReleaseAsset[],
  target: TargetTriple,
  ext: "zip" | "gz"
): ReleaseAssetMatch | null {
  const binaryName = `drut-${target}.${ext}`;
  const checksumName = `${binaryName}.sha256`;
  const binary = assets.find((a) => a.name === binaryName);
  const checksum = assets.find((a) => a.name === checksumName);
  if (!binary || !checksum) return null;
  return { binary, checksum };
}

/// spec.md FR-006. Case-insensitive hex comparison. `expectedFileContent`
/// is the raw .sha256 sidecar's content, e.g.
/// "<hex>  drut-x86_64-unknown-linux-gnu.gz\n" -- extracts just the hex
/// portion (first whitespace-delimited token) before comparing.
export function verifyChecksum(computedHexDigest: string, expectedFileContent: string): boolean {
  const expectedHex = expectedFileContent.trim().split(/\s+/)[0] ?? "";
  return computedHexDigest.trim().toLowerCase() === expectedHex.toLowerCase();
}

/// spec.md FR-018. Parses "https://github.com/<owner>/<repo>.git" (or
/// without the trailing ".git") out of package.json's own repository.url
/// -- never a second hardcoded owner/repo string living independently of
/// that field.
export function parseGitHubSlug(repositoryUrl: string): { owner: string; repo: string } | null {
  const match = repositoryUrl.match(/github\.com[/:]([^/]+)\/([^/.]+?)(?:\.git)?\/?$/);
  if (!match) return null;
  return { owner: match[1], repo: match[2] };
}

/// spec.md FR-014. Pure boundary check -- `now` passed in explicitly for
/// testability, no Date.now() call inside. undefined lastCheckedMs =>
/// always due (never checked before).
export function isUpdateCheckDue(lastCheckedMs: number | undefined, nowMs: number, throttleMs: number): boolean {
  if (lastCheckedMs === undefined) return true;
  return nowMs - lastCheckedMs >= throttleMs;
}

/// spec.md FR-015/FR-016. "Is this version newer, and have we already been
/// told no for exactly this version." A version string that doesn't parse
/// as three numeric components falls back to a direct inequality check
/// (research.md §8) -- never silently missing a real update over an
/// unexpected format.
export function shouldOfferUpdate(
  installedVersion: string | undefined,
  latestVersion: string,
  declinedVersion: string | undefined
): boolean {
  if (installedVersion === undefined) return false; // nothing installed via this mechanism yet -- storage-only scope (FR-013)
  if (latestVersion === declinedVersion) return false;
  if (latestVersion === installedVersion) return false;

  const parse = (v: string): [number, number, number] | null => {
    const m = v.replace(/^v/, "").match(/^(\d+)\.(\d+)\.(\d+)$/);
    if (!m) return null;
    return [Number(m[1]), Number(m[2]), Number(m[3])];
  };
  const a = parse(latestVersion);
  const b = parse(installedVersion);
  if (a && b) {
    if (a[0] !== b[0]) return a[0] > b[0];
    if (a[1] !== b[1]) return a[1] > b[1];
    return a[2] > b[2];
  }
  // Unparseable shape (e.g. a disposable test tag's pre-release suffix) --
  // conservative fallback: any difference is treated as "offer it."
  return latestVersion !== installedVersion;
}
