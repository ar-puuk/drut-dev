# Implementation Plan: Extension Binary Bootstrap ("Batteries Included")

**Branch**: `015-extension-binary-bootstrap` | **Date**: 2026-08-13 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/015-extension-binary-bootstrap/spec.md`

## Summary

Replace `extension.ts`'s synchronous `resolveDrutCommand()` (bare `"drut"`,
PATH-only) with an async `resolveDrutBinary(context)` that tries, in order:
(1) a real pre-flight PATH check (`spawnSync`, inspect for `ENOENT` — not
attempt-and-catch), (2) a platform/arch-matched binary already installed in
`context.globalStorageUri`, (3) download-verify-extract-install from the
latest GitHub Release, using D2's actual asset list (`drut-<target-triple>.
<ext>` + `.sha256`, fetched live via the public GitHub REST API, never
constructed from an assumed template). All impure, VS-Code-API-touching
orchestration lives in `extension.ts`; every pure decision (platform→target-
triple mapping, asset-list matching, checksum comparison, update-check
throttling/offer logic) is extracted into a new `src/binaryBootstrap.ts`,
mirroring `formatOnSaveDecision.ts`'s existing pure/impure split exactly —
testable via the same `ts-node test/*.test.ts` pattern, no VS Code test host
needed. No new npm dependency: gzip via Node's built-in `zlib`, Windows zip
extraction via a spawned `Expand-Archive` (PowerShell, built into every
supported Windows version), HTTP via Node's built-in `https` module (decided
over the newer global `fetch()` specifically to avoid any uncertainty about
which Node version a given VS Code release's extension host guarantees —
`https` has been available in every Node version ever, zero ambiguity).
Version-staleness handling (spec.md User Story 4, already resolved as an
Assumption, not left to this phase): a throttled (≤ once/24h), storage-only-
scoped, non-blocking background check; a newer version gets a single
dismissible `showInformationMessage` offer (Update/Later), never a silent
replacement.

## Technical Context

**Language/Version**: TypeScript (this repo's existing `editors/vscode/tsconfig.json` target), Node.js (VS Code extension host — version guaranteed by `engines.vscode: ^1.85.0`, treated as unknown/unverified rather than assumed, which is exactly why `https`/`zlib`/`child_process` — present in every Node version this project could plausibly run under — were chosen over anything requiring a specific newer Node baseline like global `fetch()`).

**Primary Dependencies**: None new. `vscode-languageclient` (existing) for the LSP client itself; Node built-ins only for everything this feature adds (`https`, `zlib`, `crypto` for SHA-256, `child_process` for the Windows `Expand-Archive` spawn, `fs`/`fs/promises` for storage I/O).

**Storage**: `context.globalStorageUri` (VS Code's per-extension, per-machine persistent storage directory) for the downloaded binary; `context.globalState` for the installed-version record and update-check throttle/decline state. Neither is workspace-scoped — a binary downloaded once serves every workspace on that machine.

**Testing**:
- `editors/vscode/test/binaryBootstrap.test.ts` (new, `ts-node`-run, mirrors `test/formatOnSave.test.ts`'s existing shape): every pure function in `src/binaryBootstrap.ts` — platform/arch→target-triple mapping (all 4 supported combinations plus at least one unsupported one, e.g. `linux`/`arm64`), asset-list matching (exact-name match succeeds; a release missing the expected asset name is treated as no-match, not a crash), checksum comparison (match/mismatch/case-insensitivity of hex digest), update-check throttle (`isUpdateCheckDue` at exactly the boundary, just under, just over), and the decline-tracking offer logic (same version not re-offered; a newer version after a decline is offered again).
- No VS Code extension-host integration test is added for the impure orchestration in `extension.ts` itself (matches this project's existing testing posture for this file — `formatOnSaveDecision.ts`'s extraction exists specifically so the decision logic can be tested without one).
- Manual verification (quickstart.md) covers what only a real VS Code + real network + real GitHub Release can prove: an actual first-activation download-and-run, PATH still winning when present, and the graceful-degradation notification text.

**Target Platform**: Windows x64, macOS x64/arm64, Linux x64 (D2's actual matrix) get the download path; every other platform/arch (notably Linux/Windows arm64) gets the unsupported-platform degradation path, never an attempted download.

**Performance Goals**: Activation must not be perceptibly slower when a binary is already resolved from PATH or extension storage (the common case after the first run) — the download path only runs once per machine under normal circumstances. The background update check (FR-014) must add zero perceptible delay to activation, since it never blocks `client.start()`.

**Constraints**:
- MUST NOT add a new npm dependency (FR-007's zero-new-dependency constraint, research.md §2/§3).
- MUST NOT let a PATH-resolved binary ever be second-guessed by storage or a fresh download (FR-002, User Story 2 — the one true regression risk this whole feature carries).
- MUST NOT trust a downloaded binary before its checksum is verified (FR-006).
- MUST NOT let an interrupted download/extraction be mistaken for a valid install later (FR-009) — a temp-path-then-atomic-rename pattern, not a direct write to the final path.
- MUST NOT block or delay language-server startup for the background update check (FR-014).
- MUST derive the GitHub repository slug from `package.json`'s own `repository.url` (FR-018), not a second hardcoded string.

**Scale/Scope**: Single extension, single machine per install, at most 4 possible downloaded-binary variants (never more than one resident at a time — an update replaces the prior stored binary, doesn't accumulate versions). Entirely `editors/vscode/`-scoped; zero Rust-crate changes.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS, with one noted, unavoidable coupling** | No grammar/parsing/formatting logic duplicated — this feature is pure binary-distribution bootstrap, entirely outside `voyager-core`'s domain. The one real coupling: D2's release.yml (GitHub Actions YAML) and this feature's platform→target-triple/asset-naming knowledge (TypeScript) cannot literally share a single source file across those two systems — both were read directly from D2's actual shipped file at spec time (not guessed), and the mapping is a small, explicit 4-entry table, so drift if D2's matrix ever changes is easy to notice and fix by hand, not silently divergent. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | No new text derived from Cube Voyager vendor documentation. |
| III. Formatter Idempotence & Behavior Preservation | **N/A** | No formatter change. |
| IV. False Negatives Over False Positives | **N/A** | No diagnostic category involved. |
| V. Vertical, Independently-Usable Increments | **PASS** | Single, atomic, independently valuable and testable change; does not depend on any other pending pre-publish item and blocks item 8 (actual publish) per ROADMAP.md's own stated ordering. |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **N/A** | Binary distribution/bootstrapping is not an LSP protocol concern at all — LSP defines no mechanism for "how did the server binary get onto this machine." Using VS Code's own extension-host storage APIs (`globalStorageUri`/`globalState`) here isn't competing with an LSP-standard alternative, since none exists for this problem — the same reasoning applies to every other LSP client that does this (rust-analyzer, ruff), each per-client, not via LSP itself. |
| VII. Naming Honesty | **PASS** | "Binary bootstrap"/"batteries included" describes exactly what this does — no overclaiming. |
| VIII. Public/Private Boundary | **PASS** | No vendor-documentation-corpus content involved; `editors/vscode/` is already public. |

No unjustified violations. No Complexity Tracking entries.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/
quickstart.md): `contracts/binary-bootstrap-api.md`'s exact function
signatures and resolution algorithm confirm the Principle I/VI framing above
holds precisely — no row's status changed.

## Project Structure

### Documentation (this feature)

```text
specs/015-extension-binary-bootstrap/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output (/speckit-plan command)
├── data-model.md                  # Phase 1 output (/speckit-plan command)
├── quickstart.md                  # Phase 1 output (/speckit-plan command)
├── contracts/                     # Phase 1 output (/speckit-plan command)
│   └── binary-bootstrap-api.md      # resolution algorithm, notification
│                                    # kinds, storage layout, pure-function
│                                    # signatures
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
editors/vscode/
├── src/
│   ├── extension.ts                 # resolveDrutCommand() replaced by an
│   │                                #   async resolveDrutBinary(context);
│   │                                #   activate() becomes async, awaits
│   │                                #   it before building serverOptions;
│   │                                #   new notifyOnce kinds
│   │                                #   ("unsupported-platform",
│   │                                #   "download-failed"); fires the
│   │                                #   background update check
│   │                                #   (fire-and-forget) after
│   │                                #   client.start(), never before/
│   │                                #   blocking it
│   │   └── binaryBootstrap.ts       # NEW -- every pure decision function:
│   │                                #   platform/arch -> target triple;
│   │                                #   asset-list matching; checksum
│   │                                #   comparison; update-check
│   │                                #   throttle/offer logic; GitHub repo
│   │                                #   slug parsed from package.json
│   └── formatOnSaveDecision.ts      # unchanged -- the existing precedent
│                                    #   this feature's pure/impure split
│                                    #   mirrors
├── test/
│   ├── binaryBootstrap.test.ts      # NEW -- covers every pure function
│   │                                #   above
│   ├── formatOnSave.test.ts         # unchanged
│   └── grammar.test.ts              # unchanged
└── package.json                     # "test" script gains
                                     #   `&& ts-node test/binaryBootstrap.test.ts`;
                                     #   no new dependencies section changes
```

**Structure Decision**: No new crate, no Rust change anywhere. One new
TypeScript module (`src/binaryBootstrap.ts`) holding every pure function,
one new test file exercising it, and `extension.ts`'s existing
`resolveDrutCommand`/`activate` functions extended in place — the same
shape `formatOnSaveDecision.ts` already established for this exact kind of
split, not a new pattern being invented.

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new
dependencies, no new architectural components beyond one new module
following an already-established pattern.*
