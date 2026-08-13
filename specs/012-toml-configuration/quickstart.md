# Quickstart: Validating TOML-Based Configuration

A runnable validation guide, not an implementation walkthrough — proves this feature
against spec.md's Success Criteria. See `contracts/toml-config-api.md` for the exact
schema/API and `research.md` for the full design rationale.

## Prerequisites

- Rust stable toolchain.
- A local checkout of the WF-TDM-Official-Releases corpus (`$CORPUS`), for the
  full-corpus regression step.
- VS Code, for the manual LSP smoke test (step 6).

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `drut-config` unit tests — validates FR-001 through FR-008, FR-011

```powershell
cargo test -p drut-config
```

Expected: all green, including discovery (nearest file wins; `.git` boundary stops
the walk; filesystem root stops it too; a file three directories deep finds a config
at the project root), parsing (all three `ConfigWarning` categories — parse error,
unrecognized key, invalid value — each falling back per-field, not per-file, except
the total-parse-failure case), and `resolve_format_options`'s precedence (explicit
beats file beats default, independently per field; `isolated: true` skips discovery
entirely even with a valid nearby file).

## 3. CLI integration — validates FR-001, FR-003, FR-006, FR-008, US1/US2/US3

```powershell
cargo test -p drut-cli --test format_flags
```

Expected: all green, including a `drut.toml` in a temp directory producing the
configured casing/indent with no flags passed; an explicit `--casing`/
`--top-level-indent` flag overriding it for one invocation only; `--isolated`
ignoring a present, valid `drut.toml` entirely; a malformed `drut.toml` producing a
stderr notice while formatting still completes and the exit code stays `0`
(`ExitOutcome::Clean` — never changed by a config warning, contracts.md).

## 4. LSP integration — validates FR-005, FR-009, US1

```powershell
cargo test -p drut-lsp --lib -- formatting:: range_formatting::
cargo test -p drut-lsp --test protocol_smoke -- config
```

(Multiple module-path filters go after a single `--`, as separate arguments —
`cargo test -p drut-lsp --lib formatting:: range_formatting::` is invalid syntax,
caught while running this exact step.)

Expected: all green, including a real `textDocument/formatting` request over
`Connection::memory()` against a document whose real on-disk path sits under a
`drut.toml`, producing output matching that file's settings with zero client-side
configuration; the same document formatted via `textDocument/rangeFormatting`
matches; an unsaved/untitled document (no real path, workspace root has no
`drut.toml`) formats with built-in defaults exactly as before this feature; a
malformed `drut.toml` produces a `HINT`-severity, `"drut-config"`-sourced diagnostic
distinct from every existing diagnostics stream, while formatting still completes.

## 5. MCP integration — validates FR-010, US1/US2/US3

```powershell
cargo test -p drut-mcp --lib format::
```

Expected: all green, including the same `drut.toml`-in-a-temp-directory scenario as
step 3, driven through the `format` tool via a `path`-sourced `FormatInput`; an
explicit `casing`/`top_level_indent` parameter overriding it; `isolated: true`
ignoring it; a `text`-sourced call (no `path`) never attempting discovery at all,
matching today's behavior exactly.

## 6. Manual verification in a real VS Code instance — validates SC-001, SC-002

1. Create a small project directory containing a `drut.toml` (`[format]
   casing = "lower"`) and a `.s` file with uppercase keywords.
2. Launch the extension development host (`F5` in `editors/vscode/`) against that
   directory.
3. Run Format Document. Confirm keywords are lowercased — with zero VS Code
   settings changed by hand, matching the CLI's own output on the same file with no
   flags passed.
4. Introduce a deliberate typo into `drut.toml` (e.g. `csing = "lower"`). Re-run
   Format Document. Confirm formatting still completes (using built-in defaults for
   the broken field) and a visible diagnostic appears on the open document
   identifying the problem.

## 7. Full-workspace and full-corpus regression — validates SC-003

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
$env:DRUT_CORPUS_PATH = "$CORPUS"
cargo test --release -p drut-cli --test fixture_corpus_e2e -- --ignored
cargo test --release -p drut-lsp --test diagnostics_corpus -- --ignored
cargo test --release -p drut-mcp --test diagnostics_corpus -- --ignored
```

Expected: still clean across the board — none of the real corpus files sit near a
`drut.toml`, so this is a pure regression check proving SC-003 (zero `drut.toml`
anywhere behaves identically to before this feature).

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2, 3, 4, 5 | SC-001 |
| 4, 6 | SC-002 |
| 7 | SC-003 |
| 3, 5 | SC-004 |
| 2, 3, 4, 5, 6 | SC-005 |
