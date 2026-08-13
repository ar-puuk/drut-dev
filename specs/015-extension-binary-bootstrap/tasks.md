---

description: "Task list for Extension Binary Bootstrap (\"Batteries Included\")"
---

# Tasks: Extension Binary Bootstrap ("Batteries Included")

**Input**: Design documents from `/specs/015-extension-binary-bootstrap/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/,
quickstart.md (all present)

**Tests**: Included — every pure function gets direct unit coverage
(`ts-node test/binaryBootstrap.test.ts`, mirroring `formatOnSaveDecision.ts`'s
existing precedent); the impure orchestration (real network/fs/spawn) is
proven via `quickstart.md`'s manual steps instead, per plan.md's own stated
testing posture — no VS Code extension-host test harness exists in this repo
today, and building one is out of scope for this feature.

**Organization**: Foundational phase carries what every story needs (the
pure decision functions, and Tier 1's PATH pre-flight check, which nothing
downstream can meaningfully build without). Then four user stories matching
spec.md exactly: US1 (P1, works out of the box — Tiers 2+3 and the full
resolution wiring), US2 (P1, PATH is never second-guessed — verification,
not new construction, since Tier 1 already exists from Foundational), US3
(P2, graceful degradation — the two new notification kinds), US4 (P2, update
offered, never silent — the background check, wholly separate from
`resolveDrutBinary` itself).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an
  incomplete sibling task)
- **[Story]**: US1/US2/US3/US4 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `editors/vscode/src/binaryBootstrap.ts` — every pure function (Foundational
  + US4's `shouldOfferUpdate`/`isUpdateCheckDue`, already homed here from
  Foundational).
- `editors/vscode/src/extension.ts` — `resolveDrutBinary`, `activate()`'s
  async rewrite, the two new `notifyOnce` call sites, and
  `checkForUpdateInBackground`.
- `editors/vscode/test/binaryBootstrap.test.ts` — new, all pure-function
  coverage.
- `editors/vscode/package.json` — `"test"` script only.

---

## Phase 1: Setup

- [x] T001 Confirm baseline: `cd editors\vscode && npm install && npm run
      compile && npm test` all clean, on this fresh branch before any
      change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The pure decision logic and the PATH pre-flight check every
story's resolution path depends on.

- [x] T002 Create `editors/vscode/src/binaryBootstrap.ts`. Add
      `TargetTriple` type and `mapPlatformToTarget(platform, arch):
      TargetTriple | null` per `contracts/binary-bootstrap-api.md`'s
      exhaustive 4-entry table (cross-checked directly against
      `release.yml`'s matrix). Add `archiveExtensionFor(target): "zip" |
      "gz"`.
- [x] T003 [P] In `binaryBootstrap.ts`: add `ReleaseAsset`,
      `ReleaseAssetMatch`, `GitHubRelease` types and
      `findReleaseAssetMatch(assets, target, ext): ReleaseAssetMatch |
      null` (exact-name match against a real asset list; a partial match
      — one of the two names present, not both — is still `null`).
      Depends on T002 (uses `TargetTriple`).
- [x] T004 [P] In `binaryBootstrap.ts`: add `verifyChecksum
      (computedHexDigest, expectedFileContent): boolean` — case-insensitive,
      extracts the hex token from a real `.sha256` sidecar's content
      (`"<hex>  filename\n"` shape).
- [x] T005 [P] In `binaryBootstrap.ts`: add `parseGitHubSlug(repositoryUrl):
      {owner, repo} | null` — parses `package.json`'s own `repository.url`
      (FR-018), not a second hardcoded string.
- [x] T006 In `editors/vscode/src/extension.ts`: replace synchronous
      `resolveDrutCommand()` with the start of async `resolveDrutBinary
      (context)` — Tier 1 only for now (the `spawnSync("drut", ["--version"],
      {stdio: "ignore"})` pre-flight check per `contracts/
      binary-bootstrap-api.md`'s algorithm, returning `{command: "drut",
      source: "path"}` on success). `activate()` becomes `async`, `await`s
      this, unchanged otherwise. Depends on T002 (uses `ResolvedBinary`/
      `ResolutionSource` types, added here or in T002 — caller's choice
      which file houses the type aliases; `data-model.md` doesn't mandate
      one over the other).
- [x] T007 [P] Add `editors/vscode/test/binaryBootstrap.test.ts`
      (`ts-node`-run, mirrors `test/formatOnSave.test.ts`'s shape): cover
      `mapPlatformToTarget` (all 4 supported combinations + at least one
      unsupported, e.g. `linux`/`arm64`), `archiveExtensionFor`,
      `findReleaseAssetMatch` (real match; missing-binary-name case;
      missing-checksum-name case), `verifyChecksum` (match; mismatch;
      case-insensitivity), `parseGitHubSlug` (the real
      `https://github.com/ar-puuk/drut-dev.git` value from this repo's own
      `package.json`, plus a without-`.git` variant). Depends on T002-T005.
- [x] T008 [P] Update `editors/vscode/package.json`'s `"test"` script to
      add `&& ts-node test/binaryBootstrap.test.ts`.

**Checkpoint**: Pure decision logic fully tested; Tier 1 (PATH) is real,
wired, and behaviorally unchanged from today (`npm test` and `npm run
compile` both clean).

---

## Phase 3: User Story 1 - The extension works immediately after installing from the Marketplace (Priority: P1)

**Goal**: With no PATH binary and no prior install, activation results in a
fully working language server with no manual step.

**Independent Test**: Fresh machine state (or fresh temp VS Code profile),
no `drut` on PATH, open a `.s` file with a deliberate error — a diagnostic
appears within seconds, no user action beyond opening the file.

### Implementation for User Story 1

- [x] T009 [US1] In `binaryBootstrap.ts`: add the HTTP helpers —
      `fetchLatestRelease(owner, repo): Promise<GitHubRelease>` (Node
      `https`, `GET /repos/{owner}/{repo}/releases/latest`, sets a
      `User-Agent` header per research.md §1/§5) and `downloadTo(url,
      destPath): Promise<void>` (streams a response body to a file).
- [x] T010 [US1] In `extension.ts`: extend `resolveDrutBinary` with Tier 2
      — check `context.globalStorageUri`'s fixed-name binary
      (`drut`/`drut.exe`) exists *and* `globalState.
      drutInstalledPlatformArch` matches the current
      `${process.platform}-${process.arch}` (research.md §7/data-model.md;
      a mismatch or absence falls through, never trusted as-is — spec.md
      Edge Cases). Depends on T006.
- [x] T011 [US1] In `extension.ts`: extend `resolveDrutBinary` with Tier 3
      — `mapPlatformToTarget`; if `null`, return `undefined` (US3 handles
      the notification, T015); else `fetchLatestRelease` +
      `findReleaseAssetMatch` + download both files into a `.tmp`
      subdirectory *inside* `globalStorageUri` (research.md §6, same-
      filesystem requirement for the atomic rename step) +
      `verifyChecksum` (mismatch => treat as failure) + decompress
      (`zlib.gunzipSync` on non-Windows; spawn `Expand-Archive` via
      PowerShell on Windows, per research.md §2/§3 — no new npm
      dependency) + `chmod 0o755` on non-Windows + atomic rename into the
      final fixed-name path + record `drutInstalledVersion`/
      `drutInstalledPlatformArch` in `globalState`. Depends on T009, T010.
- [x] T012 [US1] Wire `activate()`'s async flow completely: `await
      resolveDrutBinary(context)`; if it returns a `ResolvedBinary`, build
      `serverOptions`/start the client exactly as today; if `undefined`,
      skip straight to the existing highlighting-only state (the
      appropriate notification was already fired inside
      `resolveDrutBinary` itself per US3's tasks). Depends on T011.

**Checkpoint**: User Story 1 independently proven — `quickstart.md` step 3
(manual, first real activation with no PATH/no prior install).

---

## Phase 4: User Story 2 - An existing PATH-based install is never second-guessed (Priority: P1)

**Goal**: Confirm, not build further — Tier 1 (already implemented in
Foundational) genuinely short-circuits before Tiers 2/3 are ever consulted.

**Independent Test**: With `drut` on PATH *and* a stored copy already
present from an earlier activation, the PATH binary is the one that runs.

### Verification for User Story 2

- [x] T013 [US2] Add a unit test to `binaryBootstrap.test.ts` (or a small,
      focused test directly against `resolveDrutBinary`'s own tier-order
      logic, with Tier 2/3's underlying calls mocked/stubbed) proving Tier
      1 succeeding means Tier 2's storage check and Tier 3's download logic
      are never invoked at all — not just that the final `command` happens
      to be `"drut"`, but that the *later tiers' own code paths are
      provably unreached* when Tier 1 already succeeded. Depends on T006,
      T010, T011 (all three tiers must exist to prove the short-circuit is
      real, not accidental).

**Checkpoint**: User Story 2 independently proven — `quickstart.md` step 4
(manual, PATH present + stored copy present, PATH still wins).

---

## Phase 5: User Story 3 - Unsupported platforms and failures degrade gracefully (Priority: P2)

**Goal**: Every distinct failure kind gets its own single, correctly-
attributed, non-repeating notification; highlighting is never affected.

**Independent Test**: An unsupported platform/arch, and separately a
download/verification failure, both result in working highlighting plus
exactly one correctly-worded notification each — no crash, no repeat.

### Implementation for User Story 3

- [x] T014 [US3] In `extension.ts`: add the `"unsupported-platform"`
      `notifyOnce` call site, firing exactly where `resolveDrutBinary`'s
      Tier 3 finds `mapPlatformToTarget` returned `null` (T011's own
      early-return point) — message names the actual
      `process.platform`/`process.arch`, distinct wording from the
      existing generic `"missing-binary"` message. Depends on T011.
- [x] T015 [US3] In `extension.ts`: add the `"download-failed"`
      `notifyOnce` call site, wrapping Tier 3's fetch/download/verify/
      extract/install sequence (T011) in a single try/catch so *any*
      failure in that sequence (network error, 404, asset not found in the
      release, checksum mismatch, extraction failure) reaches this one
      call site — message distinct from both `"unsupported-platform"` and
      `"missing-binary"`. Depends on T011.

**Checkpoint**: User Story 3 independently proven — `quickstart.md` step 5
(manual: unsupported platform, network failure, checksum mismatch, each
producing exactly one correctly-attributed notification that doesn't
repeat).

---

## Phase 6: User Story 4 - A newer release is offered, never silently installed (Priority: P2)

**Goal**: A storage-sourced install gets a throttled, dismissible,
per-version update offer — never a silent replacement, never second-
guessing a PATH install.

**Independent Test**: A stale storage-sourced install gets one dismissible
offer per newer version; declining doesn't block normal use or suppress a
later, genuinely newer offer; accepting results in the new binary running
with no further manual step.

### Implementation for User Story 4

- [x] T016 [US4] In `binaryBootstrap.ts`: add `isUpdateCheckDue
      (lastCheckedMs, nowMs, throttleMs): boolean` (pure, `now` passed in
      explicitly for testability — no `Date.now()` call inside) and
      `shouldOfferUpdate(installedVersion, latestVersion,
      declinedVersion): boolean` (research.md §8's minimal numeric
      comparator, string-inequality fallback for anything that doesn't
      parse as three numeric components). Add both to
      `binaryBootstrap.test.ts`: `isUpdateCheckDue`'s boundary (just under
      vs. just over the throttle window, and the `undefined`
      -lastCheckedMs-always-due case); `shouldOfferUpdate`'s decline-
      tracking (same version not re-offered; a newer version after a
      decline offered again; an older/equal "latest" never offered).
- [x] T017 [US4] In `extension.ts`: add `checkForUpdateInBackground
      (context, source)` — returns immediately (no-op) unless `source ===
      "storage"` (FR-013); checks `isUpdateCheckDue` against
      `globalState.drutLastUpdateCheckMs` (FR-014); on a due check, calls
      `fetchLatestRelease` (a failure here is swallowed silently — this is
      a best-effort background check, not a path that fires a user-facing
      notification, per `contracts/binary-bootstrap-api.md`'s explicit
      note); if `shouldOfferUpdate` is true, `showInformationMessage` with
      Update/Later actions. Depends on T009 (reuses `fetchLatestRelease`),
      T016.
- [x] T018 [US4] In `extension.ts`: wire the "Update" action, **in this
      exact order** — (1) `await client.stop()` on the existing client
      *first*, before touching the binary file at all (verified directly
      against `vscode-languageclient`'s real API,
      `node_modules/vscode-languageclient/lib/common/client.d.ts`:
      `serverOptions` is fixed at construction time with no public setter
      and no `restart(newOptions)` method — reusing
      `OneRestartErrorHandler`'s `CloseAction.Restart` is **not** viable
      here, since it only relaunches the *same*, already-fixed
      `serverOptions` after an unexpected crash; it cannot point at a
      different binary. This is genuinely new code, not a reused
      mechanism — corrected after `/speckit-analyze` found the original
      task description's "same restart primitive" claim was factually
      wrong. Stopping first also avoids a real Windows file-lock failure:
      a running process holds its own executable file locked, so
      overwriting it while the old server is still alive is unsafe); (2)
      re-run Tier 3's download/verify/extract/install sequence (T011) for
      the newer release; (3) construct a **new** `LanguageClient` instance
      with `serverOptions.command` pointing at the newly-installed binary,
      assign it to the module-level `client` variable, and call
      `.start()` on it. Wire "Later" to record
      `globalState.drutDeclinedUpdateVersion` (FR-016). Depends on T017.
- [x] T019 [US4] Call `checkForUpdateInBackground` from `activate()`,
      fire-and-forget (`void checkForUpdateInBackground(...)`), placed
      *after* `client.start()` — never before, never awaited (FR-014's
      non-blocking requirement). Depends on T012, T017.

**Checkpoint**: User Story 4 independently proven — `quickstart.md` step 6
(manual: offer appears, decline doesn't block or permanently silence, a
newer version still gets a fresh offer, accepting installs and runs the
new binary).

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Whole-workspace re-proof, once all four stories are done.

- [x] T020 `cd editors\vscode && npm run compile && npm test`, both clean.
- [x] T021 [P] `cargo test --release --workspace` and `cargo clippy
      --workspace --all-targets -- -D warnings`, both clean — this feature
      touches only `editors/vscode/`, so this is a pure regression check
      that the Rust workspace is undisturbed.
- [x] T022 **Windows — real hands-on manual verification.** Run
      `quickstart.md`'s manual steps 3-6 end-to-end against a real VS Code
      instance on Windows (this session's own machine) and a disposable
      test GitHub Release (same disposable-tag-then-fully-clean-up
      discipline as D2's own live test), reporting each step's outcome
      individually. This is the *only* platform genuinely clicked through
      by a human in this cycle.
- [x] T023 **macOS/Linux — automated CI verification, not manual.** Add a
      `verify-bootstrap` job to `.github/workflows/release.yml`
      (`needs: release`, matrix `macos-latest`/`ubuntu-latest`), running a
      small Node script that exercises the *real* `binaryBootstrap.ts`
      download/verify/extract/chmod sequence against that run's own
      just-published release assets on real GitHub-hosted runners: on
      `ubuntu-latest`, download+verify+extract+chmod the
      `x86_64-unknown-linux-gnu` asset and execute it (e.g. `--version`) to
      prove full functional correctness, not just extraction; on
      `macos-latest` (arm64-native), do the same for the
      `aarch64-apple-darwin` asset (native, executable on that runner) —
      the `x86_64-apple-darwin` asset is downloaded/verified/extracted/
      chmod'd on the same runner but deliberately **not executed** (would
      need Rosetta, not guaranteed present, and would only be testing
      emulation, not this feature's own logic) — noted explicitly as
      extraction-verified-but-not-executed, not silently equated with the
      other two. This becomes a permanent regression guard for every real
      future release, not a one-time check.
- [x] T024 Record the resulting platform-coverage split explicitly in
      `quickstart.md` (a short new note, not a rewrite): Windows =
      real hands-on manual verification (T022); Linux = automated,
      CI-verified, full download-through-execution; macOS = automated,
      CI-verified for arm64 (native, full download-through-execution) and
      for x64 (download-through-extraction only, execution not attempted)
      — so what's proven vs. assumed is legible to anyone reading this
      later, not just stated once in a chat report.

**Checkpoint**: Feature-complete against spec.md; all four user stories
independently proven; full workspace re-proven clean; cross-platform
coverage explicit and honestly represented, not overclaimed.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational.
- **User Story 2 (Phase 4)**: Depends on Foundational (T006) *and* US1's
  T010/T011 — its own independent test requires Tiers 2/3 to genuinely
  exist to prove Tier 1 short-circuits *past* them, not merely that Tier 1
  alone works.
- **User Story 3 (Phase 5)**: Depends on US1's T011 (both new notification
  call sites hook into Tier 3's existing structure).
- **User Story 4 (Phase 6)**: Depends on Foundational (T009 for
  `fetchLatestRelease`) and US1's T011/T012 (the install sequence it
  re-runs, and `activate()`'s flow it hooks into after `client.start()`).
- **Polish (Phase 7)**: Depends on all four stories being complete.

### Parallel Opportunities

- T003, T004, T005 can run in parallel once T002 lands (different
  functions in the same new file — sequential edits to the same file in
  practice, but no *logical* dependency between them).
- T007, T008 can run in parallel once T002-T006 land.
- T021 is independent of T020/T022.

---

## Implementation Strategy

### Single Pass (all four stories share one Foundational base and one file)

1. Setup → baseline confirmed clean.
2. Foundational → every pure function tested; Tier 1 (PATH) real and
   unchanged in behavior from today.
3. User Story 1 → Tiers 2+3, the full resolution chain, `activate()`'s
   async rewrite — the "batteries included" promise itself.
4. User Story 2 → proof, not construction — Tier 1 genuinely short-
   circuits past the Tiers US1 just built.
5. User Story 3 → the two new failure-specific notifications, hooked into
   Tier 3's structure US1 already built.
6. User Story 4 → the wholly-separate background update-check path.
7. Polish → whole-workspace re-proof, full manual quickstart pass.

---

## Notes

- Every task naming a real file path was checked against the actual current
  `editors/vscode/` layout before this file was written (plan.md's Project
  Structure), not assumed.
- Commit after each task or logical group.
