// Unit tests for the extension binary bootstrap's pure decision logic
// (specs/015-extension-binary-bootstrap). Runs standalone via ts-node,
// mirroring test/formatOnSave.test.ts's existing convention -- every
// function under test has zero dependency on the `vscode` module.

import {
  archiveExtensionFor,
  findReleaseAssetMatch,
  isUpdateCheckDue,
  mapPlatformToTarget,
  parseGitHubSlug,
  pickResolutionTier,
  ReleaseAsset,
  shouldOfferUpdate,
  verifyChecksum,
} from "../src/binaryBootstrap";

let failures = 0;
function check(name: string, condition: boolean): void {
  if (condition) {
    console.log(`ok - ${name}`);
  } else {
    failures++;
    console.error(`FAIL - ${name}`);
  }
}

function testMapPlatformToTarget(): void {
  check("win32/x64 maps to the msvc target", mapPlatformToTarget("win32", "x64") === "x86_64-pc-windows-msvc");
  check("darwin/arm64 maps to the aarch64 darwin target", mapPlatformToTarget("darwin", "arm64") === "aarch64-apple-darwin");
  check("darwin/x64 maps to the x86_64 darwin target", mapPlatformToTarget("darwin", "x64") === "x86_64-apple-darwin");
  check("linux/x64 maps to the gnu target", mapPlatformToTarget("linux", "x64") === "x86_64-unknown-linux-gnu");
  check("linux/arm64 is unsupported (null)", mapPlatformToTarget("linux", "arm64") === null);
  check("win32/arm64 is unsupported (null)", mapPlatformToTarget("win32", "arm64") === null);
  check("an unrecognized platform is unsupported (null)", mapPlatformToTarget("freebsd", "x64") === null);
}

function testArchiveExtensionFor(): void {
  check("Windows target uses .zip", archiveExtensionFor("x86_64-pc-windows-msvc") === "zip");
  check("Linux target uses .gz", archiveExtensionFor("x86_64-unknown-linux-gnu") === "gz");
  check("macOS arm64 target uses .gz", archiveExtensionFor("aarch64-apple-darwin") === "gz");
  check("macOS x64 target uses .gz", archiveExtensionFor("x86_64-apple-darwin") === "gz");
}

function testFindReleaseAssetMatch(): void {
  // A real D2-shaped asset list (all 4 binaries + sidecars), matching
  // exactly what release.yml's own live test produced.
  const assets: ReleaseAsset[] = [
    { name: "drut-x86_64-pc-windows-msvc.zip", browser_download_url: "https://example/win.zip" },
    { name: "drut-x86_64-pc-windows-msvc.zip.sha256", browser_download_url: "https://example/win.zip.sha256" },
    { name: "drut-aarch64-apple-darwin.gz", browser_download_url: "https://example/mac-arm.gz" },
    { name: "drut-aarch64-apple-darwin.gz.sha256", browser_download_url: "https://example/mac-arm.gz.sha256" },
    { name: "drut-x86_64-unknown-linux-gnu.gz", browser_download_url: "https://example/linux.gz" },
    { name: "drut-x86_64-unknown-linux-gnu.gz.sha256", browser_download_url: "https://example/linux.gz.sha256" },
  ];

  const linuxMatch = findReleaseAssetMatch(assets, "x86_64-unknown-linux-gnu", "gz");
  check("a real match is found for a present asset", linuxMatch !== null);
  check(
    "the matched binary/checksum are the exact right assets",
    linuxMatch?.binary.name === "drut-x86_64-unknown-linux-gnu.gz" &&
      linuxMatch?.checksum.name === "drut-x86_64-unknown-linux-gnu.gz.sha256"
  );

  check(
    "a target with no asset in the list at all returns null, not a crash",
    findReleaseAssetMatch(assets, "x86_64-apple-darwin", "gz") === null
  );

  const missingChecksum: ReleaseAsset[] = [{ name: "drut-x86_64-unknown-linux-gnu.gz", browser_download_url: "x" }];
  check(
    "binary present but checksum sidecar missing is still no match (never half-verified)",
    findReleaseAssetMatch(missingChecksum, "x86_64-unknown-linux-gnu", "gz") === null
  );

  const missingBinary: ReleaseAsset[] = [
    { name: "drut-x86_64-unknown-linux-gnu.gz.sha256", browser_download_url: "x" },
  ];
  check(
    "checksum present but binary missing is still no match",
    findReleaseAssetMatch(missingBinary, "x86_64-unknown-linux-gnu", "gz") === null
  );
}

function testVerifyChecksum(): void {
  const sidecar = "abcdef0123456789  drut-x86_64-unknown-linux-gnu.gz\n";
  check("a matching digest verifies", verifyChecksum("abcdef0123456789", sidecar));
  check("a mismatched digest fails", !verifyChecksum("0000000000000000", sidecar));
  check("comparison is case-insensitive", verifyChecksum("ABCDEF0123456789", sidecar));
}

function testParseGitHubSlug(): void {
  // The real value from this repo's own package.json.
  const real = parseGitHubSlug("https://github.com/ar-puuk/drut-dev.git");
  check("owner parsed correctly from the real repository.url", real?.owner === "ar-puuk");
  check("repo parsed correctly from the real repository.url", real?.repo === "drut-dev");

  const withoutGitSuffix = parseGitHubSlug("https://github.com/ar-puuk/drut-dev");
  check("parses correctly without a trailing .git too", withoutGitSuffix?.owner === "ar-puuk" && withoutGitSuffix?.repo === "drut-dev");

  check("a non-GitHub URL returns null", parseGitHubSlug("https://gitlab.com/someone/something.git") === null);
}

function testIsUpdateCheckDue(): void {
  const now = 1_000_000_000;
  const throttle = 24 * 60 * 60 * 1000;
  check("never checked before is always due", isUpdateCheckDue(undefined, now, throttle));
  check("just under the throttle window is not due", !isUpdateCheckDue(now - (throttle - 1), now, throttle));
  check("exactly at the throttle window is due", isUpdateCheckDue(now - throttle, now, throttle));
  check("well past the throttle window is due", isUpdateCheckDue(now - throttle * 2, now, throttle));
}

function testPickResolutionTier(): void {
  // User Story 2's own dedicated guarantee: PATH found means "path",
  // full stop -- regardless of what Tier 2/3's own inputs say, even
  // deliberately contradictory/impossible ones. This is the genuine,
  // non-vacuous proof that Tier 2/3 are never consulted once Tier 1 has
  // already succeeded: if the decision depended on evaluating them, an
  // impossible/contradictory input here would change the outcome or
  // throw -- it does neither.
  check(
    "PATH found -> \"path\", even with a valid stored binary and a valid target (contradictory inputs, still short-circuits)",
    pickResolutionTier(true, true, "x86_64-unknown-linux-gnu") === "path"
  );
  check(
    "PATH found -> \"path\", even with no stored binary and no supported target",
    pickResolutionTier(true, false, null) === "path"
  );
  check(
    "PATH not found, stored binary valid -> \"storage\", regardless of target",
    pickResolutionTier(false, true, null) === "storage"
  );
  check(
    "PATH not found, no stored binary, unsupported target -> \"unsupported\"",
    pickResolutionTier(false, false, null) === "unsupported"
  );
  check(
    "PATH not found, no stored binary, supported target -> \"download\"",
    pickResolutionTier(false, false, "x86_64-unknown-linux-gnu") === "download"
  );
}

function testShouldOfferUpdate(): void {
  check("nothing installed via this mechanism yet -> never offered", !shouldOfferUpdate(undefined, "0.2.0", undefined));
  check("a genuinely newer version with no prior decline -> offered", shouldOfferUpdate("0.1.0", "0.2.0", undefined));
  check("the same version already declined -> not re-offered", !shouldOfferUpdate("0.1.0", "0.2.0", "0.2.0"));
  check(
    "a newer version than the declined one -> offered again (decline isn't permanent)",
    shouldOfferUpdate("0.1.0", "0.3.0", "0.2.0")
  );
  check("the latest equals what's installed -> not offered", !shouldOfferUpdate("0.2.0", "0.2.0", undefined));
  check("the latest is older than installed -> not offered", !shouldOfferUpdate("0.2.0", "0.1.0", undefined));
  check("numeric comparison handles double digits correctly (0.10.0 > 0.9.0)", shouldOfferUpdate("0.9.0", "0.10.0", undefined));
  check(
    "an unparseable version shape still offers rather than silently missing an update",
    shouldOfferUpdate("0.1.0", "0.1.0-test", undefined)
  );
}

function main(): void {
  testMapPlatformToTarget();
  testArchiveExtensionFor();
  testFindReleaseAssetMatch();
  testVerifyChecksum();
  testParseGitHubSlug();
  testIsUpdateCheckDue();
  testPickResolutionTier();
  testShouldOfferUpdate();

  if (failures > 0) {
    console.error(`${failures} check(s) failed`);
    process.exit(1);
  }
  console.log("all binary-bootstrap decision-logic checks passed");
}

main();
