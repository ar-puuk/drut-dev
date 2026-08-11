# Phase 0 Research: Drut MCP Server

## 1. Which Rust MCP SDK to depend on — RESOLVED: `rmcp`

**Decision**: `rmcp` (crate `rmcp`, published by the `modelcontextprotocol` GitHub
org itself — `github.com/modelcontextprotocol/rust-sdk`), currently at 3.1.2
(published 2026-08-07, three days before this research), with a release cadence
of roughly every few days.

**Rationale**: This is the official Rust SDK for MCP, not a third-party
reimplementation — the same "canonical, actively maintained, typed protocol
structs" bar `lsp-server`/`lsp-types` were chosen against in
`003-lsp-vscode-extension/research.md` §3. Its own ecosystem position confirms
this: `pulseengine-mcp-server`, a competing framework, has explicitly deprecated
itself and now recommends `rmcp` in its own crate description. Provides
`#[tool]`/`#[prompt]` attribute macros (the `macros` feature, enabled by
default) that generate tool listing, JSON-schema advertisement, and dispatch
from ordinary Rust methods — avoiding hand-written JSON-RPC framing for MCP's
`tools/list`/`tools/call` methods the same way `lsp-server` avoided it for LSP.

**Alternatives considered**:
- `rust-mcp-sdk` (223K downloads) — also async/tokio-based, third-party, no
  stronger signal than `rmcp`'s own official backing.
- `pmcp` (`github.com/paiml/pmcp`, 78K downloads) — third-party, also
  tokio-based (depends on `tokio`, `futures-util`, `axum` for its HTTP
  features); no advantage over the official SDK found.
- `mcp-attr` / `rust-mcp-server` / `mcp-gateway` / `klieo-mcp-server` — smaller
  adoption, no indication of being more current or more directly canonical
  than `rmcp`.
- Hand-rolling JSON-RPC over stdio for just the `tools/*` method subset this
  server needs (mirroring how `lsp-server` itself is a fairly thin scaffold) —
  rejected: unlike LSP, where `lsp-server`/`lsp-types` are minimal enough that
  the project already re-implements its own request dispatch loop by hand
  (`drut-lsp/src/lib.rs`'s `handle_request`), MCP's tool-schema advertisement
  and content-block response shapes are meaningfully more involved to get
  right from scratch, and `rmcp`'s `#[tool]` macro removes essentially all of
  that hand-written surface for a real, engineering-time cost savings with no
  corresponding correctness risk — the SDK is official, not a rando crate.

## 2. `rmcp` requires `tokio` — how this coexists with the project's all-synchronous precedent

**Decision**: `tokio` is accepted as a dependency, scoped *only* to a new
`drut-mcp` library crate — `voyager-core`, `drut-cli`'s existing `check`/
`format`/`server` subcommands, and `drut-lsp` remain entirely synchronous and
untouched. `drut-cli` gains a new `mcp` subcommand whose dispatch arm
constructs a `tokio::runtime::Runtime` locally and blocks on it
(`Runtime::new()?.block_on(drut_mcp::run())`) — the same "thin dispatch, zero
protocol logic in `drut-cli` itself" shape `server_cmd.rs` already established
for `drut server`, just with one extra line to enter the async runtime before
handing off. No other subcommand pays any runtime-construction cost; `tokio`
is a transitive dependency of the `drut` binary as a whole (via `drut-mcp`),
but its actual runtime only spins up when `drut mcp` specifically executes.

**Rationale**: Checked every actively-maintained Rust MCP SDK found (§1) —
async/tokio is a consistent, ecosystem-wide characteristic of Rust MCP
tooling, not a `rmcp`-specific choice avoidable by picking a different crate.
Given that, the real question isn't "sync or async" but "how contained is the
blast radius" — and a new adapter crate depending on `tokio` doesn't
duplicate, alter, or depend on any of `voyager-core`'s grammar/parsing logic
(constitution Principle I is about *that* single-source-of-truth guarantee,
not about every adapter sharing one concurrency model), so this is
architecturally equivalent to `drut-lsp` independently choosing `lsp-server`/
`lsp-types` or `drut-cli` independently choosing `clap`/`ignore` — each
adapter's own dependency graph, `voyager-core` untouched (still zero runtime
dependencies, FR-027).

**Alternatives considered**:
- A wholly separate `drut-mcp` *binary* (its own `main.rs`, own
  `#[tokio::main]`), analogous to how `drut-lsp` could theoretically have been
  its own binary instead of a library wired into `drut`. Rejected for the same
  reason `drut-lsp` wasn't: one binary, one install/discovery story for every
  MCP client the same way `drut server`'s callers only ever need to know
  about the single `drut` executable.
- Making `drut-cli`'s whole `main.rs` async (`#[tokio::main]` at the top) so
  every subcommand runs inside a runtime uniformly. Rejected: `check`/
  `format`/`server` have no async need at all, and starting a `tokio` runtime
  unconditionally on every invocation (including a one-shot `drut check` in a
  CI script, where process-startup latency matters) is a real, avoidable cost
  for zero benefit to those subcommands.

## 3. Tool input schemas — RESOLVED: `rmcp`'s `schemars` feature

**Decision**: Enable `rmcp`'s optional `schemars` feature and derive
`schemars::JsonSchema` (alongside `serde::Deserialize`) on each tool's small,
purpose-built input struct (e.g. `DiagnosticsInput { text: Option<String>,
path: Option<String> }`) — the standard, idiomatic way `rmcp`'s `#[tool]`
macro expects to advertise a tool's parameters to an MCP client.

**Rationale**: Avoids hand-writing and hand-maintaining raw JSON Schema
documents for four tools' input shapes; the derived schema and the actual
Rust struct `rmcp` deserializes into can never drift out of sync with each
other, unlike a hand-written schema alongside a hand-written struct.

**Alternatives considered**: Hand-written JSON Schema strings — rejected as
pure, avoidable duplication risk with no offsetting benefit once `schemars`
derivation is available for free via the SDK's own supported feature.

## 4. RUSTSEC advisory check — `tokio`, `rmcp`

**Decision**: Both cleared as of this research pass (2026-08-10) for the
dependency surface this feature actually uses.

**Findings**:
- `tokio`: five historical advisories (RUSTSEC-2021-0072, -0124,
  RUSTSEC-2023-0001, -0005, RUSTSEC-2025-0023), all against versions well
  below whatever current 1.x release gets pinned — none apply to a
  freshly-pinned current version.
- `rmcp`: one advisory, **RUSTSEC-2026-0189** (HIGH, DNS rebinding in the
  Streamable HTTP server transport, unpatched before 1.4.0). Does not apply
  here for two independent reasons: (a) the version being pinned (3.1.2) is
  already well above the 1.4.0 patched floor, and (b) the advisory's own text
  states plainly that "non-HTTP transports such as stdio and child-process
  transports are not affected" — this feature uses stdio exclusively (spec.md
  Assumptions). As defense in depth beyond "already patched," `drut-mcp`'s
  `Cargo.toml` MUST set `default-features = false` and enable only
  `["server", "macros", "transport-io", "schemars"]` — deliberately excluding
  `transport-streamable-http-server` and every other HTTP-transport feature,
  so that vulnerable code path never even compiles into the binary, not just
  goes unused at runtime.

Same caveat this project's every prior dependency audit has carried (README's
own Dependency auditing section): a point-in-time check, not a standing
guarantee — re-run periodically once CI exists, same as the rest.

## 5. Reusing `drut-lsp`'s hover 5-rule derivation — requires extracting it into `voyager-core`

**Decision**: Move the block-kind/matched-counterpart derivation logic
currently living in `drut-lsp/src/hover.rs` (`is_short_if`,
`run_closed_implicitly`, `counterpart_for`, `find_block_at`,
`find_hover_fact`) into `voyager-core` itself, as a new public,
protocol-agnostic entry point (working name: `voyager_core::block_at(nodes:
&[Node], diagnostics: &[Diagnostic], pos: Position) -> Option<BlockInfo>`, or
folded into a small new module). `drut-lsp/src/hover.rs` is refactored to call
this new `voyager-core` function and translate its result into
`lsp_types::Hover` markup — the same derivation, now genuinely one
implementation instead of one implementation plus a planned duplicate.
`drut-mcp`'s structural-query tool calls the identical function directly.

**Rationale**: FR-006 requires the structural-query tool to use "the exact
same 5-rule derivation `drut-lsp`'s hover capability already implements —
never a reimplementation, however similar." The only way to make that a
structural guarantee rather than a hoped-for convention is for both adapters
to call one shared implementation. Today that logic is private to `drut-lsp`
(none of `is_short_if`/`run_closed_investigation`/`counterpart_for`/
`find_block_at`/`find_hover_fact` are `pub`) and tightly coupled to
`lsp_types::Hover`/`HoverParams` and `drut-lsp`'s own `ServerState` — not
reusable by a different adapter without either exposing awkward low-level
pieces across a crate boundary or accepting a real risk of two
independently-maintained copies drifting apart, exactly the failure mode
constitution Principle I exists to prevent. This is worth being explicit
about since it means Phase 1 design and the eventual task list touch
already-shipped, already-tested `drut-lsp` code, not only add new
`drut-mcp` code — a deliberate refactor, not scope creep.

**Alternatives considered**:
- `drut-mcp` depends on `drut-lsp` directly and calls a newly-`pub`-ed version
  of `hover.rs`'s functions. Rejected: those functions are typed against
  `lsp_types`/`ServerState`, so exposing them for reuse would mean either
  leaking LSP-specific types into `drut-mcp`'s own surface (a real,
  unnecessary protocol coupling between two independent adapters) or writing
  a second, thinner reuse-oriented API inside `drut-lsp` anyway — at which
  point that API belongs in `voyager-core`, not `drut-lsp`, since it isn't
  actually LSP-specific logic to begin with.
- Duplicate the derivation logic in `drut-mcp`, accepting the drift risk.
  Rejected outright — this is precisely the outcome Principle I and FR-006
  both forbid.

## 6. Tool result shapes — voyager-core types stay internal; `drut-mcp` defines its own serializable DTOs

**Decision**: `drut-mcp` defines its own small `#[derive(Serialize,
JsonSchema)]` result types (e.g. `DiagnosticDto`, `FormatResultDto`,
`BlockInfoDto`, `KeywordCandidateDto`) and converts from `voyager-core`'s
native types (`Diagnostic`, `FormatResult`, etc., none of which derive
`Serialize`) at the tool boundary — never adding `serde` to `voyager-core`
itself.

**Rationale**: `voyager-core`'s zero-runtime-dependency guarantee (constitution
Principle I, FR-027) is unconditional — confirmed `Diagnostic` derives only
`Debug, Clone, PartialEq, Eq, Hash`, no `Serialize`. This mirrors exactly how
`drut-lsp` already handles the identical situation: `diagnostics.rs`/
`hover.rs` convert `voyager-core` types into `lsp_types` shapes at their own
adapter boundary rather than serializing `voyager-core` types directly — one
more adapter doing the same translation-at-the-boundary pattern, not a new
one.

**Alternatives considered**: Adding `serde`+`Serialize` derives to
`voyager-core`'s own types, gated behind a Cargo feature flag so the default
build stays dependency-free. Rejected as unnecessary complexity — `drut-lsp`
already proves the translate-at-the-boundary pattern works fine without
touching `voyager-core`'s dependency surface at all; no reason for
`drut-mcp` to be the first adapter to need an exception.
