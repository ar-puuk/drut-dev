# Known Environment Quirks

Machine- or environment-specific behavior that has been mistaken for (or could be
mistaken for) a code regression at least once already. Recorded here so the next
person — or the next session — recognizes it on sight instead of re-diagnosing it
from scratch. This is explicitly **not** a place for project decisions or rationale
(that's `research.md` per feature); it's a log of "this failure is your machine, not
your code."

## Application Control blocks build-script execution outside the trusted repo path

**Symptom**: `cargo build`/`cargo test` fails with an error shaped like:

```
error: failed to run custom build command for `<some-crate> vX.Y.Z`
Caused by:
  could not execute process `...\target\debug\build\<crate>-<hash>\build-script-build` (never executed)
Caused by:
  An Application Control policy has blocked this file. (os error 4551)
```

**Root cause**: the implementation machine's OS-level Application Control policy
allow-lists `D:\GitHub\drut-dev` (or, more precisely, whatever path the repo was
cloned to for day-to-day work), but does **not** allow-list other paths — most
notably temp directories (`%TEMP%`/`AppData\Local\Temp`). Any crate whose build
finishes with a compiled `build-script-build.exe` gets that executable blocked
from running when the workspace lives outside the trusted path. This is a
**location property of the machine, not a property of the crate, its build
script's content, or the project's code** — the same dependency graph builds and
runs cleanly from the trusted path.

**How to recognize it's this, not a regression**:
- The error names a `build-script-build` executable specifically, not a compile
  error in any `.rs` file.
- `os error 4551` is the Application Control signature — a Windows/security-agent
  code, not a Cargo or Rust toolchain error.
- The same dependency, same lockfile-pinned version, builds fine from
  `D:\GitHub\drut-dev` but fails from anywhere else (a fresh clone under `%TEMP%`,
  a CI runner using a different path, etc.). If you can reproduce success by
  rebuilding from the trusted path, that confirms it's this, not a real break.
- It has now fired for two *different* crates with unrelated build scripts (see
  below) — ruling out "something wrong with this one crate" as the explanation.

**What *not* to do**: don't chase this as a dependency bug, don't pin/unpin/replace
the crate to "fix" it, and don't treat a fresh-clone or CI failure shaped like this
as proof the committed code or fixtures are broken — check whether the build
directory is inside the trusted path first.

**Instances observed so far**:

| Date | Crate whose build script was blocked | Where it fired | Notes |
|---|---|---|---|
| 2026-08-09 | `serde-sarif` (`schemafy`-based code generation) | `D:\GitHub\drut-dev` during `002-cli-check-format` implementation, T001–T003 | Led to superseding `serde-sarif` with a hand-written `#[derive(Serialize)]` struct set — see `specs/002-cli-check-format/research.md` §4. In hindsight the dependency swap wasn't required to fix the *block itself* (the block is path-based, not crate-based); it was kept anyway because the hand-written structs are just as correct for this project's narrow SARIF subset and removed the dependency. |
| 2026-08-09 | `getrandom v0.3.4` (transitive, via `jsonschema` → `ahash` → `getrandom`, the SC-003 SARIF-schema-validation dev-dependency) | A fresh `git clone` into `%TEMP%\fresh_clone_audit`, done as part of verifying the `.gitattributes` byte-exactness audit | The identical build-script fingerprint had already succeeded in `D:\GitHub\drut-dev`'s own `target/` moments earlier in the same session — confirming the block is about the *directory `cargo` runs from*, not the crate. No code or dependency change was needed; the fresh-clone check's actual purpose (byte-exactness verification) had already completed and passed before this fired. |
| 2026-08-10 | The compiled *test binary* itself (`position_encoding-<hash>.exe`, `drut-lsp`'s own test crate — no third-party crate involved at all) | `D:\GitHub\drut-dev` (the trusted path), during `003-lsp-vscode-extension` implementation | A variant of the same symptom, not just a `build-script-build.exe` case: the freshly-linked `cargo test` binary itself got blocked immediately after being written, even inside the trusted path. Deleting the stale `.exe`/`.d` files under `target/debug/deps/` and letting `cargo test` relink a fresh copy resolved it immediately, same binary hash and all — consistent with a scan race on the just-written file rather than a durable block on that specific path or hash. No code change involved; confirms the quirk isn't strictly scoped to build scripts. |
| 2026-08-10 | `target/release/drut.exe` (`drut-cli`'s own release binary, rebuilt mid-session for real manual VS Code testing) | `D:\GitHub\drut-dev` (the trusted path), rebuilding after wiring `textDocument/formatting` and the `variable` semantic token into `drut-lsp` | Same shape as the row above (a freshly-relinked binary, not a build script), but this time on a **release**, not test, artifact — and notably, an earlier `cargo build` had already produced and successfully run this exact binary (VS Code's language client had it open as a live child process) moments before the rebuild that got blocked. Deleting the stale `target/release/drut.exe`/`drut.d` and letting `cargo build --release` relink resolved it immediately, same as before. Confirms this quirk isn't specific to `cargo test` output either — any freshly-written executable under `target/` is fair game. |

If this fires for a third, unrelated crate, that's expected — don't add a new
research.md entry for it; just add a row to the table above and move on.
