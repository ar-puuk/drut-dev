# Quickstart: Validating the Drut MCP Server

A runnable validation guide, not an implementation walkthrough — proves this
feature against spec.md's Success Criteria. See `contracts/mcp-tools.md` for
the full tool surface, `contracts/block-resolution-api.md` for the
`voyager-core` extraction, and `data-model.md` for the types referenced below.

## Prerequisites

- Rust stable toolchain, matching the rest of the workspace.
- The workspace builds: `cargo build --workspace` from repo root.
- A local checkout of the WF-TDM-Official-Releases corpus, available the same
  way it already is for every prior phase's own full-corpus validation
  (`001-voyager-script-parser/research.md` §3), referred to below as
  `$CORPUS`.
- An MCP-capable client for step 7 only (e.g. Claude Desktop's own MCP server
  config, or a generic MCP inspector tool) — everything else validates through
  `cargo test` alone, no client required.

## 1. Build

```powershell
cargo build -p voyager-core -p drut-cli -p drut-mcp
```

Expected: builds cleanly, zero `cargo clippy -p drut-mcp --all-targets`
warnings, matching the zero-warning bar already held for every other crate in
the workspace. Confirms `tokio`/`rmcp` (research.md §1/§2) are pulled in only
by `drut-mcp`, not by `voyager-core` or `drut-lsp`:

```powershell
cargo tree -p voyager-core
cargo tree -p drut-lsp
```

Expected: neither shows `tokio` or `rmcp` anywhere in their dependency trees.

## 2. `block_at` extraction — validates research.md §5, no `drut-lsp` regression

**This step MUST be reported as its own explicit, standalone result at
implementation time — never folded into a general "tests pass" summary.**
Same standard this session already held every other refactor of shipped code
to (the `pair_keyword_boundaries` quote-awareness fix's own dedicated
161-file revalidation, reported on its own rather than absorbed into
`003`'s broader test run). This extraction touches already-shipped,
already-tested `drut-lsp` code as a side effect of adding new `drut-mcp`
functionality — its correctness needs to stand on its own evidence, not
borrow credibility from the rest of the phase's test suite passing.

```powershell
cargo test -p voyager-core block_resolution::
cargo test -p drut-lsp --lib hover::
cargo test -p drut-lsp --test hover
```

Expected, reported explicitly and separately from every other test result
this phase produces:
1. **The new `voyager-core::block_at` unit tests** (`cargo test -p
   voyager-core block_resolution::`) pass — proving the relocated derivation
   is correct in its new home, independent of either caller.
2. **Every pre-existing `drut-lsp` hover test passes unmodified in
   behavior** (`hover_over_if_reports_kind_and_matched_endif`,
   `hover_over_short_if_has_no_separate_closer`,
   `hover_over_implicitly_closed_run_reports_resolved_location`, and the
   rest, both the `--lib hover::` unit tests and the `--test hover`
   protocol-level suite) — same test files, same assertions, only
   `hover.rs`'s own internal call path changes (calls `voyager_core::
   block_at` instead of its own now-removed private functions). A single
   assertion changing to keep a test green does not count as "unmodified
   in behavior" and MUST be called out explicitly if it happens, not
   silently folded in as a pass.
3. **The full 161-file corpus hover-parity check** (step 6 below) — run
   and reported as part of this same explicit result, not deferred or
   assumed from step 2 passing alone.

Only once all three of the above are true, and reported as such, does the
extraction count as verified.

## 3. Each tool's own unit/contract tests

```powershell
cargo test -p drut-mcp
```

Expected: all green — `diagnose`/`format`/`query_structure`/`lookup_keyword`
each covered against `contracts/mcp-tools.md`'s own per-tool behavior
(including the `ScriptSource` both-or-neither error case, FR-002), plus a
no-panic sweep across the same edge-case document shapes
`drut-lsp/tests/no_panic.rs` already exercises (empty document, unterminated
block comment, stray closer with nothing open, etc.) — reused as fixture
content, not re-invented.

## 4. Read-only guarantee — validates FR-010, SC-005

```powershell
cargo test -p drut-mcp --test no_disk_writes
```

Expected: every tool is called (including `format`, the one most tempting to
implement as "format and save") against a fixture directory made read-only for
the duration of the test, and every call still succeeds — proving no tool
attempts a write, not merely that none is documented to.

## 5. Full-corpus diagnostic parity — validates SC-006

```powershell
$env:DRUT_CORPUS_PATH = "$CORPUS"
cargo test -p drut-mcp --test diagnostics_corpus -- --ignored
```

Expected: for all 161 real files, the `diagnose` tool's output is diagnostic-
category-and-location-identical to `drut check`'s own output for the same
file — same corpus, same 100%-clean expectation every prior phase's full-corpus
run has already established.

## 6. `query_structure` matches `drut-lsp` hover on the same real position

```powershell
cargo test -p drut-mcp --test structural_query_parity
```

Expected: for a handful of real corpus files known (from `003`'s own
already-passing hover tests) to contain implicitly-closed `Run`/`Process`
blocks, `query_structure`'s result for the exact same position matches what
`drut-lsp`'s hover already reports for it — both now reading the same
`voyager_core::block_at` (contracts/block-resolution-api.md), so this is a
parity check on the wiring, not a re-verification of the derivation logic
itself (already proven in step 2).

## 7. Manual smoke test — an MCP client actually calls the tools

Not automatable the same way steps 1–6 are (analogous to
`003-lsp-vscode-extension/quickstart.md`'s own steps 7–9, a human-run check of
the thing the protocol-level test harness can't verify: that the packaged
binary actually launches correctly from within a real client).

1. Point an MCP-capable client at `drut mcp` (no flags, stdio — the same
   "point a client at this binary" shape `drut server` already has).
2. Confirm the client's tool list shows all four tools
   (`diagnose`/`format`/`query_structure`/`lookup_keyword`) with real,
   client-rendered parameter descriptions (proving `schemars`-generated
   schemas actually reached the client, not just that they compiled).
3. Call `diagnose` against a real script (inline text is fine) containing a
   deliberate defect and confirm the client shows a correctly-located
   diagnostic.
4. Call `format` against the same script and confirm the returned text is
   different and `changed: true`.
5. Call `query_structure` at a position inside an `IF`/`ENDIF` block and
   confirm the client shows block kind `If` and the `ENDIF` location.
6. Call `lookup_keyword` with `enclosing_control_word: "RUN"` and confirm
   `PGM`, `MSG`, `PRNFILE` appear in the result.

Report what was actually observed at each step, not just that the calls
returned *something* — the same discipline `003`'s own manual verification
pass held itself to.
