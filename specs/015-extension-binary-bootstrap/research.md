# Phase 0 Research: Extension Binary Bootstrap ("Batteries Included")

## §1 — HTTP mechanism: Node's `https` module, not global `fetch()`

Modern Node ships a global `fetch()`, but *which* Node version a given VS
Code release's extension host actually bundles is not something this repo
has ever verified or pinned (`engines.vscode: ^1.85.0` constrains the VS
Code API surface, not the Node runtime version underneath it). Rather than
add an unverified assumption ("the extension host's Node is new enough for
global `fetch`"), this feature uses Node's `https` module directly — it has
existed, unchanged in the relevant respects, since every Node version this
project could plausibly run under. Zero ambiguity, zero new research
surface to get wrong later. Both requests this feature ever makes — the
GitHub Releases API call and the two file downloads (binary + `.sha256`) —
are simple GETs; `https.get` plus manual response-stream handling is not
meaningfully more code than `fetch` would be for this case.

**Required header, easy to miss**: GitHub's REST API rejects unauthenticated
requests with no `User-Agent` header. Every request this feature makes must
set one (e.g. `"drut-vscode-extension"`).

## §2/§3 — No new npm dependency for archive extraction

Confirmed directly against D2's real, shipped output (`.github/workflows/
release.yml`): `.gz` (plain gzip, not `.tar.gz`) on macOS/Linux, `.zip` on
Windows. The key structural fact that makes zero-dependency extraction
possible: **a given user's machine only ever needs to handle its own
platform's single archive format** — a Windows user's extension downloads
and extracts a `.zip`, never a `.gz`; a macOS/Linux user's extension
downloads and extracts a `.gz`, never a `.zip`. There is no code path that
needs both formats on the same machine.

- **macOS/Linux (`.gz`)**: Node's built-in `zlib.gunzipSync()` — no
  dependency, no child process.
- **Windows (`.zip`)**: `Expand-Archive` (PowerShell), built into every
  supported Windows version (PowerShell 5.0+, present since Windows 10 and
  Windows Server 2016) — spawned as a child process
  (`powershell -NoProfile -Command "Expand-Archive -Path ... -DestinationPath ... -Force"`).
  No npm zip library needed.

## §4 — PATH pre-flight check: the same primitive `vscode-languageclient` already relies on

`extension.ts`'s own existing doc comment states `vscode-languageclient`'s
spawn "already accounts for platform PATH/`.exe` conventions correctly" —
this is Node `child_process`'s own behavior: even without `shell: true`,
Node's `spawn`/`spawnSync` on Windows performs its own `PATHEXT`-aware PATH
resolution internally (documented Node behavior, not shell-dependent), and
on POSIX platforms relies on the OS's own `execvp`-style PATH search.

This feature's pre-flight check uses `child_process.spawnSync` with the
bare command name and inspects `result.error?.code === 'ENOENT'` for "not
found." Using the **exact same primitive** the actual server-launching spawn
already uses (not a different resolution mechanism, e.g. manually walking
`process.env.PATH`) guarantees the pre-flight check can never disagree with
what the real launch would have done — the one property that matters most
here, since a false "found" would break User Story 1's fallback chain and a
false "not found" would break User Story 2's non-regression guarantee.

## §5 — GitHub Releases API shape (confirmed against D2's real output, `013`/`014`-session live-test history)

`GET https://api.github.com/repos/<owner>/<repo>/releases/latest` (owner/
repo parsed from `package.json`'s `repository.url`, FR-018 — already
`"https://github.com/ar-puuk/drut-dev.git"`, parsed with a plain regex/split,
no dependency needed). Returns `404` if no release has ever been published
— this is not a special case to handle differently from any other failure;
it's just another "couldn't get a release" outcome, same `download-failed`
path as a network error (spec.md Edge Cases). Response body's `assets`
array entries have `name` and `browser_download_url` — matched by exact
`name` equality against the 8 real names D2 produces (4 binaries + 4
`.sha256` sidecars), never by constructing an assumed name independently.

## §6 — Atomic install: temp files must share a filesystem with the final path

`fs.renameSync` is atomic only within the same filesystem/volume. Using
Node's `os.tmpdir()` for intermediate download/extraction files risks
landing on a different drive than `context.globalStorageUri` (especially on
Windows, where the system temp drive and the user-profile-rooted extension
storage drive aren't guaranteed to match), which would silently degrade the
"impossible to mistake a partial file for a valid install" guarantee (FR-009)
into a non-atomic copy. **Decision**: all intermediate files (raw download,
post-verification, post-extraction) live in a `.tmp` subdirectory *inside*
`context.globalStorageUri` itself, guaranteeing the final install step's
rename is a same-filesystem, atomic operation.

## §7 — Storage layout

One resident binary at a time (spec.md Scale/Scope — an update replaces the
prior one, versions never accumulate):

- `<globalStorageUri>/drut` (POSIX) or `<globalStorageUri>/drut.exe`
  (Windows) — the installed binary itself, fixed filename (not
  version-stamped in the filename; version is tracked separately, see
  below) so "is something already installed" is a single, simple
  existence check and an update is a same-name atomic replace.
- `<globalStorageUri>/.tmp/` — scratch space for in-progress
  download/verify/extract, per §6.
- `context.globalState` keys: `drutInstalledVersion` (string, the release
  tag last successfully installed — recorded directly from what was
  downloaded, FR-010, never learned by executing the binary),
  `drutInstalledPlatformArch` (string, e.g. `"win32-x64"` — read back on a
  later activation to catch the stale/mismatched-storage edge case from
  spec.md's Edge Cases; a mismatch means "treat as absent," not "trust
  anyway"), `drutLastUpdateCheckMs` (number, epoch ms, FR-014's throttle),
  `drutDeclinedUpdateVersion` (string, optional — FR-016's "don't re-offer
  this exact version" tracking, distinct from `drutInstalledVersion`).

## §8 — Version comparison: a minimal dependency-free comparator, not the `semver` package

Lockstep versioning (`CONTRIBUTING.md`'s "Versioning" section) commits every
real release to a plain `X.Y.Z` tag going forward — the `0.0.1-test`
pre-release suffix seen during D2's own disposable live-test tag was
explicitly a throwaway, never a real release shape. A minimal
major/minor/patch numeric-triple comparator (split on `.`, compare
numerically component-by-component) is sufficient and keeps the
zero-new-dependency constraint intact; a version string that doesn't parse
as three numeric components falls back to a direct string-inequality check
(`latest !== installed` ⇒ treat as "newer," matching the conservative "offer
rather than silently skip" bias — never silently missing a real update over
an unexpected format, per SC-005's "never surprised" framing being about
*unannounced changes*, not about occasionally being offered an update for a
version that turns out to already match).
