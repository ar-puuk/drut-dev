// vscode-independent I/O for the extension binary bootstrap
// (015-extension-binary-bootstrap). Distinct from binaryBootstrap.ts's
// *pure* decision functions (no I/O at all) -- these do real network/fs/
// process work, but never touch the `vscode` module, so they're safely
// importable from a plain Node context: extension.ts (the real extension
// host), test/bootstrapIO.test.ts (a real Windows extraction test, since
// this file can be `require`d outside VS Code entirely), and
// scripts/verify-bootstrap.ts (the CI verification harness, which used to
// duplicate this logic before this split existed -- see that file's own
// updated imports).
//
// Found via T022's real Windows manual verification, not before: this
// split is *why* extractArchive's Windows bug (guessing the extracted
// filename instead of discovering it) had zero automated coverage until a
// human actually ran the download flow for real. Living inside
// extension.ts (which imports `vscode` at module scope) made it
// untestable outside a real extension host process.

import { execFile } from "child_process";
import * as crypto from "crypto";
import * as fs from "fs";
import * as https from "https";
import * as path from "path";
import * as util from "util";
import * as zlib from "zlib";

import { archiveExtensionFor, GitHubRelease, TargetTriple } from "./binaryBootstrap";

const execFileAsync = util.promisify(execFile);

const MAX_REDIRECTS = 5; // GitHub's browser_download_url 302s to a signed S3 URL

/// research.md §5. GitHub rejects unauthenticated requests with no
/// User-Agent header -- easy to miss, set unconditionally here.
///
/// Deliberately no `token` parameter, unlike fetchReleaseByTag below: this
/// is the function the real, shipped extension calls on a real end user's
/// machine, and an end user has no GitHub token to offer. Stays on the
/// unauthenticated 60/hour-per-IP rate limit forever, by design -- one
/// activation-time API call per user is nowhere near that ceiling.
export function fetchLatestRelease(owner: string, repo: string, userAgent: string): Promise<GitHubRelease> {
  return new Promise((resolve, reject) => {
    const req = https.get(
      `https://api.github.com/repos/${owner}/${repo}/releases/latest`,
      { headers: { "User-Agent": userAgent, Accept: "application/vnd.github+json" } },
      (res) => {
        if (res.statusCode !== 200) {
          res.resume();
          reject(new Error(`GitHub API returned ${res.statusCode} for releases/latest`));
          return;
        }
        let body = "";
        res.setEncoding("utf8");
        res.on("data", (chunk) => (body += chunk));
        res.on("end", () => {
          try {
            resolve(JSON.parse(body) as GitHubRelease);
          } catch (e) {
            reject(e);
          }
        });
      }
    );
    req.on("error", reject);
  });
}

/// Fetches a specific tagged release rather than "latest" -- used by the
/// CI verification harness, which needs to verify the exact release its
/// own workflow run just created, not whatever happens to be newest.
///
/// `token` is CI-only plumbing (release.yml passes `github.token` in via the
/// GITHUB_TOKEN env var, scripts/verify-bootstrap.ts reads it and forwards
/// it here) -- found necessary after unauthenticated requests from shared
/// macos-latest runner IPs hit GitHub's 60/hour unauthenticated rate limit
/// mid-run. A real end user's install of this extension never has a GitHub
/// token to offer and never calls this function at all (fetchLatestRelease,
/// below, is what extension.ts actually uses, and it stays unauthenticated
/// deliberately -- see its own comment).
export function fetchReleaseByTag(
  owner: string,
  repo: string,
  tag: string,
  userAgent: string,
  token?: string
): Promise<GitHubRelease> {
  return new Promise((resolve, reject) => {
    const headers: Record<string, string> = { "User-Agent": userAgent, Accept: "application/vnd.github+json" };
    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }
    const req = https.get(
      `https://api.github.com/repos/${owner}/${repo}/releases/tags/${tag}`,
      { headers },
      (res) => {
        if (res.statusCode !== 200) {
          res.resume();
          reject(new Error(`GitHub API returned ${res.statusCode} for releases/tags/${tag}`));
          return;
        }
        let body = "";
        res.setEncoding("utf8");
        res.on("data", (chunk) => (body += chunk));
        res.on("end", () => {
          try {
            resolve(JSON.parse(body) as GitHubRelease);
          } catch (e) {
            reject(e);
          }
        });
      }
    );
    req.on("error", reject);
  });
}

/// research.md §6: writes into `dest` directly -- callers are responsible
/// for using a `.tmp` path inside globalStorageUri and only promoting it
/// to the final location via an atomic rename after full success (FR-009).
/// Follows redirects manually (Node's `https` module doesn't do this on
/// its own, and GitHub's asset URLs 302 to a signed S3 URL).
export function downloadToFile(url: string, dest: string, userAgent: string, redirectsLeft = MAX_REDIRECTS): Promise<void> {
  return new Promise((resolve, reject) => {
    const req = https.get(url, { headers: { "User-Agent": userAgent } }, (res) => {
      if (res.statusCode !== undefined && [301, 302, 307, 308].includes(res.statusCode) && res.headers.location) {
        res.resume();
        if (redirectsLeft <= 0) {
          reject(new Error("too many redirects"));
          return;
        }
        downloadToFile(res.headers.location, dest, userAgent, redirectsLeft - 1).then(resolve, reject);
        return;
      }
      if (res.statusCode !== 200) {
        res.resume();
        reject(new Error(`download failed: HTTP ${res.statusCode}`));
        return;
      }
      const out = fs.createWriteStream(dest);
      res.pipe(out);
      out.on("finish", () => out.close(() => resolve()));
      out.on("error", reject);
    });
    req.on("error", reject);
  });
}

export function sha256File(filePath: string): string {
  const hash = crypto.createHash("sha256");
  hash.update(fs.readFileSync(filePath));
  return hash.digest("hex");
}

/// research.md §2/§3: no new npm dependency for either archive format --
/// each platform only ever needs to handle its own single format.
export async function extractArchive(archivePath: string, destPath: string, target: TargetTriple): Promise<void> {
  if (archiveExtensionFor(target) === "gz") {
    const decompressed = zlib.gunzipSync(fs.readFileSync(archivePath));
    fs.writeFileSync(destPath, decompressed);
    fs.chmodSync(destPath, 0o755); // spec.md FR-008 -- gzip strips the executable bit
    return;
  }
  // Windows: Expand-Archive, built into every supported Windows version
  // (PowerShell 5.0+) -- spawned, not a new npm zip dependency.
  //
  // Real bug found during T022's manual verification (015-extension-
  // binary-bootstrap): this used to *guess* the extracted filename as
  // `<archive-basename>.exe` (e.g. "drut-x86_64-pc-windows-msvc.exe").
  // That file never exists -- confirmed directly against D2's real
  // release.yml (`Package (Windows)` step): `Compress-Archive -Path
  // $binary -DestinationPath $zip` preserves the *source file's own*
  // basename ("drut.exe") inside the archive; only the archive itself
  // gets renamed to drut-<target-triple>.zip. Extracting into a fresh,
  // empty directory and discovering the one real file that resulted --
  // rather than assuming any particular name -- can't drift out of sync
  // with whatever the packaging step actually produces again.
  const extractDir = fs.mkdtempSync(path.join(path.dirname(archivePath), "extract-"));
  try {
    await execFileAsync("powershell", [
      "-NoProfile",
      "-Command",
      `Expand-Archive -Path "${archivePath}" -DestinationPath "${extractDir}" -Force`,
    ]);
    const extracted = fs.readdirSync(extractDir);
    if (extracted.length !== 1) {
      throw new Error(`expected exactly one file after extracting ${archivePath}, found: ${extracted.join(", ") || "(none)"}`);
    }
    fs.renameSync(path.join(extractDir, extracted[0]), destPath);
  } finally {
    fs.rmSync(extractDir, { recursive: true, force: true });
  }
}
