# Quickstart: Validating Editor-Settings Exposure for `[format]` Config Fields

A runnable validation guide, not an implementation walkthrough — proves this feature against
spec.md's Success Criteria. See `contracts/editor-settings-config.md` for the exact API/protocol
shape and `data-model.md`/`research.md` for the full design rationale.

## Prerequisites

- Rust stable toolchain; Node/npm for the VS Code extension side.

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `drut-config` precedence tests — validates FR-003, FR-005

```powershell
cargo test -p drut-config
```

Expected: all green, including —
- A `client_defaults` value applies when neither `explicit` nor `drut.toml` set a field.
- A `drut.toml` value wins over a `client_defaults` value for the same field (US2 AS1).
- A `client_defaults` value wins for a *different* field `drut.toml` doesn't set at all, in the
  same resolution call (US2 AS2).
- An out-of-range `client_defaults` numeric value (`indent_width`, either blank-line cap) falls
  back to the built-in default with a non-blocking notice.
- Every existing CLI/MCP-shaped test (passing `ExplicitFormatOverride::default()` for the new
  parameter) still passes unmodified — confirms the new parameter is additive, not disruptive.

## 3. `drut-lsp` pull/cache/fallback tests — validates FR-002, FR-004, FR-006

```powershell
cargo test -p drut-lsp
```

Expected: all green, including —
- A client that advertises `workspace/configuration` support receives the request at startup.
- A client that doesn't advertise support never receives it, and formatting behaves exactly as
  before this feature (FR-004, SC-005).
- A `workspace/didChangeConfiguration` notification triggers a re-pull, and the *next* format
  request against an already-open document reflects the refreshed value (FR-006, SC-004) — no
  document close/reopen.
- A malformed/unparseable pulled value for one field leaves that field's cache entry `None`
  (falls through to the built-in default) without affecting any other field.

## 4. Protocol-level round trip — validates SC-001, SC-004

```powershell
cargo test -p drut-lsp --test protocol_smoke
```

Expected: all green, including a real `initialize`/`workspace/configuration`/
`textDocument/formatting` round trip over `Connection::memory()` proving a client-set value for
at least one field per config category (casing, indentation, operator spacing, blank lines)
actually changes formatted output with no `drut.toml` present.

## 5. VS Code extension — validates FR-008, SC-006

```powershell
cd editors/vscode
npm run compile
```

Then inspect `package.json`'s `contributes.configuration.properties` directly: confirm all 10
`drut.format.*` entries are present, each with the correct type/enum/range matching its
`drut-config` counterpart, and none declares a `"default"` (data-model.md §3).

## 6. Full workspace re-proof + real-corpus revalidation

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Then confirm SC-003 directly: run the full 161-file real corpus through the CLI with no
`drut.toml` and no client settings — expected zero diagnostic/output change from before this
feature (the new fourth parameter is always `ExplicitFormatOverride::default()` on the CLI
surface).

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2 | SC-002, SC-003 |
| 3, 4 | SC-001, SC-004, SC-005 |
| 5 | SC-006 |
| 6 | SC-003, all others (integration re-proof) |
