# Quickstart: Validating Live Diagnostic Updates on Config File Edits

A runnable validation guide, not an implementation walkthrough — proves this
feature against spec.md's Success Criteria. See `contracts/config-watch-api.md` for
the exact mechanism and `research.md` for the full design rationale.

## Prerequisites

- Rust stable toolchain.
- VS Code, for the manual smoke test (step 4).

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. Unit tests — validates FR-001, FR-003, FR-004, FR-007, FR-009

```powershell
cargo test -p drut-lsp --lib
```

Expected: all green, including `ServerState::open_uris()` returning every open
document's URI (and an empty iterator for none open), and the registration-gating
logic itself only constructing a registration request when the capability check
passes (verified directly, not just "the server doesn't crash either way").

## 3. Protocol tests — validates FR-002, FR-005, FR-006, FR-008, FR-010, US1, US2

```powershell
cargo test -p drut-lsp --test protocol_smoke
```

Expected: all green, including — as the **primary, required** test, not one
criterion among several — a real `Connection::memory()` session reproducing
spec.md US1 Acceptance Scenario 1 exactly: a script file open with a diagnostic
naming one invalid `drut.toml` value, the config edited (via a simulated
`workspace/didChangeWatchedFiles` notification) to a *different* invalid value
without any `didChange`/`didClose`/`didOpen` on the script file itself, and the
republished diagnostic naming the *new* bad value. Also covers: multiple open
documents all refreshing on one config change (US1 Scenario 2); a diagnostic
appearing/disappearing on a previously-clean/previously-broken document purely from
a config edit (US1 Scenarios 3-4); an `initialize` handshake with
`didChangeWatchedFiles.dynamicRegistration` absent or `false` never sending a
registration request and the session proceeding normally (US2 Scenarios 1-2); and,
directly proving FR-010/SC-005 (US2 Scenario 3) — a session where
`dynamicRegistration` is `true` (so the registration request *is* sent), but the
test harness never sends any response to it, followed by an ordinary request (e.g.
`textDocument/hover`), confirming that unrelated request still receives its own
response normally — the missing registration response never stalls the loop.

## 4. Manual verification in a real VS Code instance — validates SC-001, SC-003

1. Open a project directory containing a `drut.toml` and a `.s` file with a
   deliberately invalid setting (e.g. `casing = "sideways"`).
2. Launch the extension development host (`F5` in `editors/vscode/`); open the
   `.s` file; confirm the config-warning diagnostic appears.
3. Without closing the `.s` file, edit `drut.toml` to a *different* invalid value
   and save. Confirm the diagnostic updates to name the new value within the same
   session — this is the exact bug being fixed; treat this step as the manual
   confirmation of quickstart step 3's automated proof, not a separate check.
4. Open a second `.s` file governed by the same `drut.toml`; confirm it also shows
   the (correctly updated) diagnostic without being reopened.
5. Fix `drut.toml` to be fully valid; confirm both open files' diagnostics clear
   without either being closed or reopened.

## 5. Full workspace re-proof

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: clean — this feature touches only `drut-lsp`, so this is primarily a
regression check that nothing in `012`'s own test suite (or any earlier feature's)
was disturbed.

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 3, 4 | SC-001 |
| 3 | SC-002 |
| 3, 4 | SC-003 |
| 3 | SC-004 (perceived, not measured — no automated timing assertion is warranted at this scale) |
