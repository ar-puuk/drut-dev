// CI-only verification harness for the extension binary bootstrap
// (015-extension-binary-bootstrap, tasks.md T023). Run on real
// macos-latest/ubuntu-latest/windows-latest GitHub-hosted runners, after a
// real release exists, to prove the download/verify/extract/chmod
// sequence works for real, not simulated.
//
// Deliberately standalone -- does not import extension.ts (which imports
// the `vscode` module, unavailable outside a real extension host process).
// Reuses binaryBootstrap.ts's pure functions *and* bootstrapIO.ts's real
// I/O functions directly -- both are vscode-independent by construction,
// safe to import from a plain Node script. No duplicated download/extract
// logic anymore (there used to be a hand-rolled, gz-only copy here; see
// git history) -- reusing the exact same extractArchive() extension.ts
// itself calls means this harness tests the real code path, including the
// Windows Expand-Archive branch, not a parallel copy that could drift.
//
// Usage: ts-node scripts/verify-bootstrap.ts <target-triple> <tag> [--execute]

import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import { execFileSync } from "child_process";

import { archiveExtensionFor, findReleaseAssetMatch, TargetTriple, verifyChecksum } from "../src/binaryBootstrap";
import { downloadToFile, extractArchive, fetchReleaseByTag, sha256File } from "../src/bootstrapIO";

const USER_AGENT = "drut-vscode-extension-ci-verify";

async function main(): Promise<void> {
  const [targetArg, tag, executeFlag] = process.argv.slice(2);
  const target = targetArg as TargetTriple;
  const shouldExecute = executeFlag === "--execute";

  if (!target || !tag) {
    console.error("usage: ts-node scripts/verify-bootstrap.ts <target-triple> <tag> [--execute]");
    process.exit(2);
  }

  const packageJson = JSON.parse(fs.readFileSync(path.join(__dirname, "..", "package.json"), "utf8")) as {
    repository: { url: string };
  };
  const slugMatch = packageJson.repository.url.match(/github\.com[/:]([^/]+)\/([^/.]+?)(?:\.git)?\/?$/);
  if (!slugMatch) throw new Error("could not parse repository slug from package.json");
  const [, owner, repo] = slugMatch;

  console.log(`Verifying bootstrap for target=${target} tag=${tag} (execute=${shouldExecute})`);

  // release.yml passes github.token in via GITHUB_TOKEN when running in CI
  // -- authenticated requests get a 5000/hour rate limit instead of the
  // 60/hour-per-IP unauthenticated one, which shared macos-latest runner
  // IPs were observed hitting mid-run. Undefined when run locally by hand;
  // fetchReleaseByTag falls back to an unauthenticated request in that case.
  const token = process.env.GITHUB_TOKEN;
  const release = await fetchReleaseByTag(owner, repo, tag, USER_AGENT, token);
  const ext = archiveExtensionFor(target);
  const match = findReleaseAssetMatch(release.assets, target, ext);
  if (!match) {
    throw new Error(`release ${tag} has no asset for ${target} (${ext})`);
  }
  console.log(`Found asset: ${match.binary.name} + ${match.checksum.name}`);

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "drut-verify-"));
  const archivePath = path.join(tmpDir, match.binary.name);
  const checksumPath = path.join(tmpDir, match.checksum.name);

  await downloadToFile(match.binary.browser_download_url, archivePath, USER_AGENT);
  await downloadToFile(match.checksum.browser_download_url, checksumPath, USER_AGENT);
  console.log("Downloaded binary + checksum sidecar.");

  const digest = sha256File(archivePath);
  const expected = fs.readFileSync(checksumPath, "utf8");
  if (!verifyChecksum(digest, expected)) {
    throw new Error(`checksum mismatch: computed ${digest}, expected file said ${expected.trim()}`);
  }
  console.log("Checksum verified.");

  const extractedPath = path.join(tmpDir, ext === "zip" ? "drut.exe" : "drut");
  await extractArchive(archivePath, extractedPath, target);
  console.log(`Extracted (and chmod'd, on non-Windows): ${extractedPath}`);

  if (shouldExecute) {
    // "--help", not "--version" -- confirmed directly against a real
    // local build: `drut` is subcommand-only (clap's auto `--version`
    // flag isn't enabled) and rejects a bare `--version` with a non-zero
    // exit; `--help` is the flag actually guaranteed to succeed at the
    // top level regardless of which subcommand a real invocation would
    // use.
    const output = execFileSync(extractedPath, ["--help"], { encoding: "utf8" });
    console.log(`Executed successfully: ${output.split("\n")[0]}`);
  } else {
    console.log("Execution skipped for this target (deliberately not attempted -- see release.yml's own comment).");
  }

  fs.rmSync(tmpDir, { recursive: true, force: true });
  console.log(`PASS: ${target}`);
}

main().catch((err) => {
  console.error(`FAIL: ${err instanceof Error ? err.message : String(err)}`);
  process.exit(1);
});
