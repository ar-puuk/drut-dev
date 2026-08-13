# Quickstart: Validating the Extension Binary Bootstrap

A runnable validation guide, not an implementation walkthrough — proves this
feature against spec.md's Success Criteria. See `contracts/
binary-bootstrap-api.md` for the exact algorithm/function signatures and
`research.md` for the technical decisions behind them. More of this
feature's validation is manual than usual, since real download/PATH/
network behavior can't be fully simulated without a real VS Code host, a
real network, and a real published GitHub Release.

## Platform coverage — what's proven vs. assumed (tasks.md T024)

This feature was verified across all three platforms, but not all in the
same way — recorded here explicitly so it's legible to anyone reading this
later, not just stated once in a chat report:

| Platform | Coverage | How |
|---|---|---|
| **Windows** | Real, hands-on manual verification **and** automated, CI-verified, full download-through-**execution** | A human clicking through a real VS Code instance (steps 3-6 below), on this project's own development machine (tasks.md T022) — this is what actually found and drove the fix for the `extractArchive` Windows extraction bug (see `bootstrapIO.ts`'s own comment). Separately, `.github/workflows/release.yml`'s `verify-bootstrap` job also runs the same download/verify/extract/execute sequence for real on a `windows-latest` runner, added after that bug so it can't regress silently again. |
| **Linux** (`x86_64-unknown-linux-gnu`) | Automated, CI-verified, full download-through-**execution** | Same `verify-bootstrap` job, on a real `ubuntu-latest` runner, downloads the real release asset, verifies its checksum, extracts and `chmod`s it, then actually runs it (tasks.md T023). Live-run confirmed (a disposable test release, since fully cleaned up). |
| **macOS, arm64** (`aarch64-apple-darwin`) | Automated, CI-verified, full download-through-**execution** | Same `verify-bootstrap` job, on a real `macos-latest` runner (Apple Silicon-native as of GitHub's current hosted images) — download, checksum, extract, `chmod`, and execute, all for real. |
| **macOS, x64** (`x86_64-apple-darwin`) | Automated, CI-verified, download-through-**extraction only** | Same job, same runner — download/checksum/extract/`chmod` are all real and verified; **execution is deliberately not attempted** (would need Rosetta, not guaranteed present on the runner, and would only prove emulation works, not this feature's own logic). Not silently equated with the other two rows. |

No platform in this table was ever left entirely unverified — the honest
distinction is real-human-in-VS-Code vs. real-CI-on-a-real-runner, not
verified vs. assumed. Windows ended up with both: the manual pass is what
actually surfaces platform-specific bugs a script wouldn't think to check
for, and the CI pass is what keeps that specific bug from coming back
unnoticed on every future release.

## Prerequisites

- Node.js, npm.
- VS Code, for the manual smoke tests (steps 3-6).
- A published GitHub Release with the expected assets (either the real one,
  once one exists, or a disposable test release/tag — same disposable-tag-
  then-fully-clean-up discipline as D2's own live test).

## 1. Build

```powershell
cd editors\vscode
npm install
npm run compile
```

## 2. Unit tests — validates FR-004, FR-005, FR-006, FR-014, FR-015/FR-016, FR-018

```powershell
npm test
```

Expected: all green, including `test/binaryBootstrap.test.ts`'s coverage of
every pure function in `contracts/binary-bootstrap-api.md` — all 4
supported platform/arch combinations plus at least one unsupported one
(e.g. `linux`/`arm64`) for `mapPlatformToTarget`; a real D2-shaped asset
list matching correctly *and* a release missing an expected asset name
correctly returning no match; checksum comparison (match, mismatch,
case-insensitivity); `isUpdateCheckDue`'s boundary (just under vs. just
over the throttle window); `shouldOfferUpdate`'s decline-tracking (same
version not re-offered, a newer version after a decline offered again).

## 3. Manual: first activation, no PATH, no prior install — validates SC-001, US1

1. Ensure `drut` is not on PATH and no prior activation has happened on
   this machine (or run with a fresh/temporary VS Code user-data +
   extensions directory, e.g. `code --user-data-dir <tmp> --extensions-dir <tmp>`).
2. Install the built `.vsix` (`npx @vscode/vsce package`, then "Install
   from VSIX...").
3. Open a `.s` file with a deliberate structural error (e.g. an unmatched
   `IF`).
4. Confirm: within a few seconds, a diagnostic appears — no manual action
   taken beyond opening the file. Confirm (via the existing startup log
   line) the binary path now points inside the extension's global storage
   directory, not PATH.
5. Reload the window and reopen the file. Confirm no re-download happens
   (no new network activity, near-instant startup) — Tier 2 (storage) is
   used this time.

## 4. Manual: PATH still wins — validates SC-002, US2

1. With `drut` genuinely on PATH (e.g. `cargo build --release -p drut-cli`
   and add `target/release` to PATH), and *also* a previously-downloaded
   copy already present in extension storage from step 3 above, activate
   the extension.
2. Confirm via the startup log line that the PATH binary is the one
   actually running, not the stored copy — no download attempted, no
   storage check even relevant to the outcome.

## 5. Manual: graceful degradation — validates SC-003, SC-004, US3

1. **Unsupported platform**: temporarily fake `process.platform`/
   `process.arch` (or run on/emulate an actually-unsupported combination)
   and confirm highlighting still works, exactly one notification names
   the platform/arch as unsupported, and reloading the window doesn't
   repeat it.
2. **Network failure**: disconnect network (or point the GitHub API/
   download URLs at something unreachable for a test build) and confirm
   the same graceful degradation, with the distinct `download-failed`
   notification text instead.
3. **Checksum mismatch**: deliberately corrupt a downloaded binary before
   verification runs (test-only hook, or a modified test release with a
   mismatched sidecar) and confirm the corrupted file is never used to
   start the server — same `download-failed` path, not a crash.

## 6. Manual: update offered, never silent — validates SC-005, US4

1. With a binary already installed via Tier 3 (storage-sourced) at an
   older version than a newer disposable test release, force the
   throttle window (`drutLastUpdateCheckMs`) to be stale and reactivate.
2. Confirm a single dismissible notification offers the update (Update /
   Later) — the running server is *not* replaced before the user chooses.
3. Choose "Later." Reactivate with the same latest release still current.
   Confirm no re-prompt.
4. Publish (or simulate) a newer release still. Reactivate. Confirm a
   *fresh* prompt for the newer version — the earlier decline didn't
   silence this one.
5. Choose "Update" on a fresh prompt. Confirm the new binary installs and
   the running language server ends up using it, with no further manual
   step.

## 7. Full workspace re-proof (unaffected surfaces)

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: clean — this feature touches only `editors/vscode/`, so this is
purely a regression check that nothing in the Rust workspace was disturbed.

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 3 | SC-001 |
| 4 | SC-002 |
| 5.1, 5.2 | SC-003 |
| 5.3 | SC-004 |
| 6 | SC-005, SC-006 |
