# Implementation Plan: Editor-Settings Exposure for `[format]` Config Fields

**Branch**: `021-editor-settings-config` | **Date**: 2026-08-17 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/021-editor-settings-config/spec.md`

## Summary

A new, lowest-precedence-but-one config tier — client (editor) settings — inserted into
`drut-config::resolve_format_options` between `drut.toml` and the built-in default, reachable
only through `drut-lsp` and populated via the standard LSP `workspace/configuration`/
`workspace/didChangeConfiguration` mechanism. `resolve_format_options` gains one new parameter
(`client_defaults: ExplicitFormatOverride`, reusing the existing 10-field type — research.md §1);
every existing CLI/MCP call site passes `ExplicitFormatOverride::default()` for it, so those two
surfaces are completely unaffected (spec.md FR-007).

`drut-lsp` gains its *second* ever server-initiated request (the first, from
`013-lsp-config-file-watch`, already established the exact pattern to follow: fire-and-forget,
never blocks the main loop, generically handled by the existing `handle_response` dispatch —
research.md §2). The pulled value is cached in `ServerState` and re-pulled on
`workspace/didChangeConfiguration` (whose own payload is never read as a data source — research.md
§3, since the modern LSP client convention sends it as a bare re-pull trigger). One request per
pull, for the merged `"drut.format"` section (research.md §4) — not one round trip per field, and
not scoped per document/workspace-folder (research.md §5, a deliberate simplification: client
settings are a personal, single global fallback, distinct from `drut.toml`'s already-existing
per-project, scope-aware role).

`editors/vscode/package.json` declares all 10 fields under `drut.format.*` in
`contributes.configuration`, with no `"default"` on any property (an unset VS Code setting must
mean "not present," not a hidden second built-in-default source).

## Technical Context

**Language/Version**: Rust, stable toolchain, 2021 edition (server-side); TypeScript/JSON
(`package.json`, extension side) — both already in use, no new language/toolchain.

**Primary Dependencies**: None new — `drut-lsp` already depends on `lsp-types`/`lsp-server` and
already sends/handles at least one server-initiated request; no new crate.

**Storage**: N/A.

**Testing**:
- `crates/drut-config/tests/resolve.rs` (or equivalent) — new precedence cases for the fourth
  parameter (client-defaults-only resolution; `drut.toml`-wins-over-client-defaults; per-field
  independence; invalid client-defaults value degrades correctly); every existing test updated
  to pass `ExplicitFormatOverride::default()` as the new fourth argument (a compile-time-forced,
  mechanical change, not new logic).
- `crates/drut-lsp/src/lib.rs`/`document_store.rs` test modules — `workspace_configuration_
  supported` capability detection (both true/false client-capability cases); `ServerState`
  cache get/set; a malformed pulled value leaving only that one field `None`.
- `crates/drut-lsp/tests/protocol_smoke.rs` — a real `initialize` (advertising `workspace.
  configuration` support) → `workspace/configuration` request/response → `textDocument/
  formatting` round trip proving a client-set value changes output with no `drut.toml` present;
  a `drut.toml`-present case proving it still wins over a conflicting client setting; a
  `workspace/didChangeConfiguration` → re-pull → next-format-reflects-new-value live-update case;
  a client with no `workspace.configuration` support case proving zero behavior change (no
  request ever sent, existing tests continue passing unmodified).
- `editors/vscode/package.json` — no automated test framework exists for this file today; validated
  by direct inspection (quickstart.md step 5) against the exact field list, matching how prior
  `package.json`-only changes in this project were verified.
- Full real-corpus revalidation — expected zero diagnostic/output change on the CLI surface
  (SC-003), since the CLI always passes `ExplicitFormatOverride::default()` for the new parameter.

**Target Platform**: Cross-platform, unchanged.

**Project Type**: `drut-config` core addition (one new parameter, additive); `drut-lsp` new
capability (second server-initiated request, new `ServerState` cache field); `editors/vscode`
`package.json`-only change (no new extension source file needed — the extension itself doesn't
need to *read* these settings, only declare them; `vscode-languageclient`'s already-bundled
`SyncConfigurationFeature` handles the client side of `workspace/configuration` automatically
once the section is requested by the server, confirmed in `research.md`'s own grep of that
library). `drut-cli`, `drut-mcp`: unaffected beyond the one new always-default call argument.

**Performance Goals**: One additional request/response round trip per pull (at startup, and on
each `workspace/didChangeConfiguration`) — not per format request; every format request itself
reads an already-cached, synchronously-available value, no new per-request latency.

**Constraints**:
- MUST NOT change CLI or MCP behavior in any way (FR-007) — confirmed by construction (both
  always pass `ExplicitFormatOverride::default()` for the new parameter) and by the existing
  test suites for both passing unmodified.
- MUST NOT introduce a blocking wait anywhere in `drut-lsp`'s main loop (research.md §2) — this
  project has an established, deliberate fire-and-forget precedent for server-initiated requests;
  a blocking pattern here would be a new, inconsistent architecture, not a repeat of one that
  already exists.
- MUST degrade gracefully (never error, never regress existing behavior) when the connected
  client doesn't support `workspace/configuration` (FR-004) — same "advertised-capability-gated,
  silent skip otherwise" shape `did_change_watched_files_supported` already established.
- MUST NOT let a client setting override a value `drut.toml` itself sets for the same field
  (FR-003) — enforced by the `.or()` chain's fixed ordering, not a runtime check that could be
  bypassed.
- Invalid client-setting values MUST degrade like any other malformed config value (FR-005) —
  reuses the exact existing validation helpers (`resolve_indent_width`/`resolve_blank_line_cap`),
  not new validation logic.

**Scale/Scope**: 10 fields, one new `resolve_format_options` parameter, one new `ServerState`
field, one new request/response pair in `drut-lsp`, one `package.json` section. No real-corpus
golden-fixture work needed (unlike `017`-`019`) — this feature changes no formatting *logic* at
all, only which config values feed into logic that already exists and is already fully tested.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|---|---|---|
| I. Single Source of Truth | **PASS** | All real precedence logic lives in `drut-config::resolve_format_options`, the same single function every other config surface already funnels through — `drut-lsp` only supplies one more input value, it implements no resolution logic of its own. |
| II. No Verbatim Vendor Doc Redistribution | **PASS** | No Voyager vendor documentation is involved at all — this is pure tooling/protocol plumbing. |
| III. Formatter Idempotence & Behavior Preservation | **PASS, N/A for new behavior, re-verified for old** | No formatting *logic* changes — `voyager_core::format` and every `FormatOptions` value it can already receive are untouched. Re-verified (not assumed) that CLI/MCP output is byte-identical to before this feature (SC-003), since both always supply an empty fourth argument. |
| IV. False Negatives Over False Positives | **N/A** | No diagnostic category is added, changed, or affected. |
| V. Vertical, Independently-Usable Increments | **PASS** | Single user-story pair (US1/US2 are two acceptance angles on the same one capability, not separable) — independently valuable and independently testable as one unit. |
| VI. LSP-Standard Mechanisms Over Editor-Proprietary APIs | **PASS, directly load-bearing** | This principle is the reason `workspace/configuration` (standard LSP) was chosen over a VS Code-proprietary mechanism (spec.md Assumptions, `ROADMAP.md` item 15) — re-confirmed in Constitution Check, not just stated once at scoping time. |
| VII. Naming Honesty | **PASS** | `client_defaults` names exactly what it is (a fallback default, not an override); `drut.format.*` VS Code setting names mirror their `drut-config`/CLI counterparts' own meaning, no renaming for its own sake. |
| VIII. Public/Private Boundary | **PASS** | All touched crates are already public; no vendor-doc-derived material anywhere in this feature. |

No unjustified violations. No Complexity Tracking entries.

**Post-Design Re-check** (after Phase 1 data-model.md/contracts/quickstart.md):
`contracts/editor-settings-config.md`'s exact precedence/signature inventory confirms the
Principle I/VI framing above holds precisely — no row's status changed. The one genuinely new
architectural piece (a second server-initiated request) stays within the exact shape the first
one already established (research.md §2), so it doesn't introduce a new pattern this table would
need to re-justify.

## Project Structure

### Documentation (this feature)

```text
specs/021-editor-settings-config/
├── plan.md                        # This file (/speckit-plan command output)
├── research.md                    # Phase 0 output
├── data-model.md                  # Phase 1 output
├── quickstart.md                  # Phase 1 output
├── contracts/
│   └── editor-settings-config.md  # exact signature/protocol shapes, precedence, guarantees
├── checklists/
│   └── requirements.md            # already created by /speckit-specify
└── tasks.md                       # Phase 2 output (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
crates/drut-config/
└── src/lib.rs                       # resolve_format_options gains
                                     #   client_defaults: ExplicitFormatOverride
                                     #   (4th param); resolve_casing_and_indent's
                                     #   per-field chains and the two
                                     #   range-validating helpers each gain
                                     #   one more fallback source
                                     #   (data-model.md §1)

crates/drut-lsp/
├── src/document_store.rs            # ServerState gains
│                                    #   client_format_defaults field +
│                                    #   get/set (data-model.md §2)
├── src/lib.rs                       # workspace_configuration_supported,
│                                    #   request_client_format_defaults
│                                    #   (2nd-ever server-initiated
│                                    #   request, fire-and-forget);
│                                    #   handle_response gains a match arm;
│                                    #   handle_notification gains a
│                                    #   workspace/didChangeConfiguration
│                                    #   arm (re-pull trigger only)
├── src/formatting.rs                # resolve_format_options call site
│                                    #   passes state.client_format_defaults()
│                                    #   instead of ::default()
└── src/range_formatting.rs          # same change, mirrored

editors/vscode/
└── package.json                     # contributes.configuration: 10 new
                                     #   drut.format.* properties
                                     #   (data-model.md §3), no source
                                     #   changes needed (vscode-languageclient's
                                     #   bundled SyncConfigurationFeature
                                     #   already handles the client side)

crates/drut-cli/, crates/drut-mcp/   # no changes (FR-007) — existing call
                                     #   sites gain a mechanical
                                     #   ExplicitFormatOverride::default()
                                     #   4th argument only, forced by the
                                     #   signature change, not new logic

ROADMAP.md                           # item 15 marked done on completion
```

**Structure Decision**: No new crate, no new extension source file. One new parameter on an
existing, single-source-of-truth function; one new cache field plus one new fire-and-forget
request/response pair in `drut-lsp`, following an already-established pattern exactly; one
`package.json`-only declaration on the extension side. Every piece of real precedence logic
funnels through `drut-config::resolve_format_options`, unchanged in shape from every other
config surface this project already has.

## Complexity Tracking

*No entries — no unjustified Constitution Check violations, no new dependencies, no new crates.
The one genuinely new piece (a second server-initiated LSP request) is justified directly by
FR-002 (the standard-mechanism requirement) and follows an existing, already-reviewed pattern
rather than inventing a new one.*
