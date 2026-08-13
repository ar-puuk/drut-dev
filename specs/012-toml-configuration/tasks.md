---

description: "Task list for TOML-Based Configuration"
---

# Tasks: TOML-Based Configuration

**Input**: Design documents from `/specs/012-toml-configuration/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/,
quickstart.md (all present)

**Tests**: Included — matches this project's established discipline for every
prior cross-crate feature (`004`, `010`, `011`): real unit tests for the new
`drut-config` crate, and real adapter-level tests at every integration point named
in FR-005/FR-009/FR-010, not just manual spot-checks.

**Organization**: Three user stories, matching spec.md exactly — US1 (P1, one
shared config reachable identically on CLI/LSP/MCP — the entire mechanism, MVP),
US2 (P2, an explicit per-run override wins), US3 (P3, a full isolation bypass).
US1 is the foundation everything else builds on; US2 needs no new implementation
code of its own (the precedence logic is already correct once US1's wiring calls
`resolve_format_options` with the right `explicit` value — US2 is dedicated proof);
US3 adds one small, genuinely new piece of surface (`--isolated`/`isolated`) on top
of US1's wiring.

**Everything in this file's scope was measured against the real, current codebase
during planning (research.md §1-§8), not estimated**:

- `voyager-core`'s zero-runtime-dependency rule is confirmed crate-scoped directly
  in `drut-cli`'s and `drut-mcp`'s own `Cargo.toml` comments — parsing/discovery
  logic goes in a **new `drut-config` crate**, never in `voyager-core`, which is
  untouched by this entire feature.
- `--top-level-indent`'s current CLI type (`TopLevelIndentArg` with
  `default_value_t`) cannot distinguish "not passed" from "explicitly `preserve`" —
  it must become `Option<TopLevelIndentArg>` for TOML precedence to work at all
  (research.md §1). This is an internal representation change only; behavior with
  no `drut.toml` anywhere is unchanged.
- `format_cmd::run` resolves `FormatOptions` **once, before** the file-traversal
  loop today — genuine per-file discovery (mirroring Ruff's own documented
  behavior) requires moving that resolution **inside** the loop (research.md §2).
- Per-field fallback on a malformed file (spec.md FR-011, confirmed explicitly with
  the owner before `/speckit-plan`) requires parsing into a `toml::Value` and
  walking it by hand — a single `#[derive(Deserialize)]` would fail the whole file
  on any one field's problem, which does not satisfy FR-011 (research.md §4).
- `drut-lsp` has **zero** URI→filesystem-path conversion today and **never**
  captures the client's workspace root — both are genuinely new, small additions,
  not reused plumbing (research.md §5).
- A malformed config never changes a CLI exit code (confirmed directly against
  `exit.rs`'s three-way convention) — it's informational, exactly matching
  `010`'s `unclosed_fmt_off_files` precedent (research.md §6).
- Scope is `drut format` + the MCP `format` tool **only** — confirmed `check` and
  three of the four MCP tools have zero relationship to `FormatOptions` today
  (research.md §8).

**Post-`/speckit-analyze` remediation (2026-08-12)**: the original draft's adapter
tasks (T011, T015, T016, T017, T018) each depended only on `drut-config`'s
*implementation* tasks (T003–T006), not its *test* tasks (T007–T009) — meaning
adapter integration could, per the stated dependency graph, begin before a single
test proved `discover`/`parse`/`resolve_format_options` actually worked. Flagged
directly against `004`'s own `block_resolution.rs` precedent (extraction trustworthy
specifically because independently tested before being consumed elsewhere) and
fixed: every task that calls into `drut-config` now explicitly depends on T007,
T008, **and** T009, not just their implementation counterparts. T007 also gained a
missing case (a `.git` *file*, not just a directory — the real shape of a git
worktree's own `.git`, which T004's own implementation note already required
handling but no test exercised).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete
  sibling task)
- **[Story]**: US1/US2/US3 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `Cargo.toml` (workspace root), `crates/drut-config/` — the new crate.
- `crates/drut-cli/src/cli.rs`, `format_cmd.rs`, `lib.rs` — CLI wiring.
- `crates/drut-lsp/src/document_store.rs`, `lib.rs`, `workspace.rs` (new),
  `formatting.rs`, `range_formatting.rs`, `diagnostics.rs` — LSP wiring.
- `crates/drut-mcp/src/format.rs` — MCP wiring.
- `README.md` — schema/discovery/precedence documentation (FR-012).

---

## Phase 1: Setup

- [x] T001 Confirm baseline: `cargo build --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings` both clean, on this
      fresh branch before any change.
- [x] T002 Add the new `drut-config` crate: add `"crates/drut-config"` to the
      workspace `Cargo.toml`'s `members` list; create
      `crates/drut-config/Cargo.toml` depending on `voyager-core = { path =
      "../voyager-core" }`, `toml = "1"` (verified current, research.md §3),
      `serde = { version = "1.0.229", features = ["derive"] }` (exact version
      already used identically by `drut-cli`/`drut-mcp`); create an empty
      `crates/drut-config/src/lib.rs`. Confirm `cargo build --workspace` still
      succeeds with the new, empty member.

**Checkpoint**: Baseline confirmed clean; the new crate exists and builds empty.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: `drut-config`'s complete, independently-tested public surface — every
user story's adapter wiring depends on this existing and being correct first.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Define `DrutConfig`, `FormatConfig`, `ConfigWarning`,
      `ExplicitFormatOverride` in `crates/drut-config/src/lib.rs`
      (data-model.md, contracts/toml-config-api.md): `FormatConfig` holds
      `casing: Option<voyager_core::CasingConvention>` and `top_level_indent:
      Option<voyager_core::TopLevelIndentMode>` — **not** derived via
      `#[derive(Deserialize)]` directly onto `voyager-core`'s own enums (they
      deliberately derive no `Deserialize`, research.md §3); define small local
      serde-deserializable enum types and convert via `From`, the same pattern
      `format_cmd.rs`'s `CasingArg -> CasingConvention` already uses.
      `ConfigWarning` has three variants: `ParseError { path, message }`,
      `UnrecognizedKey { path, table, key }`, `InvalidValue { path, table, key,
      message }`. None of these types are ever constructed as a hard error that
      stops a caller — `ConfigWarning` is always accompanied by a best-effort
      fallback value (FR-011).
- [x] T004 Implement `pub fn discover(start: &Path) -> Option<PathBuf>` in
      `crates/drut-config/src/discover.rs` (research.md §7): walk upward from
      `start` (a file's own directory, or `start` itself if already a
      directory) through ancestors; at each directory, check for `drut.toml`
      first (return it immediately if found), then check for a `.git` entry
      (file or directory — presence only, no worktree-redirect parsing needed)
      and stop there if found; otherwise continue to the parent; stop
      unconditionally at the filesystem root. Never panics, including when
      `start` doesn't exist.
- [x] T005 Implement `pub fn parse(path: &Path) -> (DrutConfig, Vec<ConfigWarning>)`
      in `crates/drut-config/src/parse.rs` (research.md §4,
      contracts/toml-config-api.md): read `path`'s content (an I/O failure here
      becomes a `ConfigWarning::ParseError`, not a panic or `Result::Err`); parse
      into a generic `toml::Value` (a real, full TOML-spec parse — not a
      hand-rolled subset); walk the `[format]` table by hand, validating
      `casing`/`top_level_indent` individually — an invalid value for one key
      produces a `ConfigWarning::InvalidValue` for that key alone, every other
      valid key still applies; an unrecognized key inside `[format]` produces
      `ConfigWarning::UnrecognizedKey` (this table is already in active use, so a
      typo here is exactly FR-011's "don't be silently confusing" case); an
      unrecognized *top-level table* other than `[format]` is silently ignored,
      not warned (forward-compatibility — a whole extra bracketed section is a
      much less plausible accidental typo than one key inside a table already in
      use, research.md §4's explicit reasoning for this asymmetry). A file that
      fails to parse as TOML at all produces one `ConfigWarning::ParseError` and
      an empty `FormatConfig` (every field `None`). Depends on T003.
- [x] T006 Implement `pub fn resolve_format_options(file_path: Option<&Path>,
      isolated: bool, explicit: ExplicitFormatOverride) ->
      (voyager_core::FormatOptions, Vec<ConfigWarning>)` in
      `crates/drut-config/src/lib.rs` (data-model.md's resolution algorithm):
      per field, independently — if `explicit`'s field is `Some`, use it;
      else, if not `isolated` and `discover`+`parse` (from `file_path`'s
      directory, if `file_path` is `Some`) produced a `Some` value for that
      field, use it; else use `voyager_core::FormatOptions::default()`'s value.
      `isolated: true` skips `discover`/`parse` entirely — `file_path` is not
      even consulted. `file_path: None` (no real on-disk location) also skips
      discovery entirely, falling straight to `explicit` then default. Depends
      on T003, T004, T005.
- [x] T007 [P] Add unit tests to `crates/drut-config/tests/discover.rs` for
      `discover`: a `drut.toml` in the same directory as the target file wins;
      a `drut.toml` one or more directories above the target (with no closer
      one) wins; a `.git` **directory** between the target and a `drut.toml`
      further up stops the walk before reaching it (returns `None`); **a
      `.git` file (not a directory — the real shape of a git worktree's own
      `.git`, per T004's own "file or directory" requirement) between the
      target and a `drut.toml` further up stops the walk the same way** — this
      case was missing from the original test list (`/speckit-analyze`
      finding); reaching the filesystem root with no `.git` and no `drut.toml`
      anywhere returns `None`; a target file three directories deep finds a
      `drut.toml` at the project root when nothing closer exists (spec.md US1
      Acceptance Scenario 4). Depends on T004.
- [x] T008 [P] Add unit tests to `crates/drut-config/tests/parse.rs` for
      `parse`: a fully valid `[format]` table with both keys set; a file with an
      invalid value for one key (e.g. `casing = "sideways"`) — that field falls
      back to `None` with an `InvalidValue` warning, the other valid key is
      still parsed correctly; a file with an unrecognized key inside `[format]`
      (e.g. `csing = "upper"`) — `UnrecognizedKey` warning, both real keys (if
      present elsewhere in the file) still apply; a file containing an
      unrecognized top-level table (e.g. `[lint]`) alongside a valid `[format]`
      table — zero warnings for the unrecognized table, `[format]` still parses
      normally; a file that isn't valid TOML at all (e.g. unbalanced brackets) —
      one `ParseError` warning, `FormatConfig` fields all `None`. Depends on
      T005.
- [x] T009 [P] Add unit tests to `crates/drut-config/tests/resolve.rs` for
      `resolve_format_options`: an explicit value wins over a present, valid
      `drut.toml` value, independently per field (setting `casing` explicitly
      doesn't affect `top_level_indent`'s own resolution from the file); a
      present, valid `drut.toml` value wins over the built-in default when no
      explicit value is given; `file_path: None` skips discovery, resolving
      straight to explicit-then-default; `isolated: true` skips discovery even
      when a valid `drut.toml` is present and would otherwise apply, for both
      fields independently. Depends on T006.

**Checkpoint**: `drut-config`'s complete public surface exists, is independently
tested, and is ready for every adapter to depend on.

---

## Phase 3: User Story 1 - A team sets one shared project convention everyone gets automatically (Priority: P1) 🎯 MVP

**Goal**: The same file, in the same project, resolves to the same effective
`casing`/`top_level_indent` settings whether processed by the CLI, an LSP-capable
editor, or the MCP `format` tool — with zero configuration on any individual
adapter's part.

**Independent Test**: Add a `drut.toml` setting non-default casing to a directory;
format a file in it via the CLI with no flags, via an editor's Format Document
action, and via the MCP `format` tool with no casing parameter; confirm all three
produce identical, correctly-cased output.

### Implementation for User Story 1

- [x] T010 [US1] In `crates/drut-cli/src/cli.rs`: change `top_level_indent`'s
      type from `TopLevelIndentArg` (with `default_value_t`) to
      `Option<TopLevelIndentArg>` (research.md §1) — matches `casing`'s existing
      `Option<CasingArg>` shape. Update `lib.rs`'s `Command::Format` match arm's
      destructuring accordingly (no behavior change yet, just the type).
- [x] T011 [US1] In `crates/drut-cli/src/format_cmd.rs`: move `FormatOptions`
      construction from once-before-the-loop to **inside** the existing
      `for file in &traversal.matched_files` loop (research.md §2), calling
      `drut_config::resolve_format_options(Some(&file.path), false,
      ExplicitFormatOverride { casing: casing.map(Into::into),
      top_level_indent: top_level_indent.map(Into::into) })` per file (the
      `false` for `isolated` is a placeholder here — T029 in US3 replaces it
      with a real flag; not built here to keep this story's scope to exactly
      what US1 needs). Add `config_warnings: Vec<(PathBuf, Vec<ConfigWarning>)>`
      to `FormatReport`, populated in every mode; add a new `eprintln!` block to
      `print_report` reporting each file and its warnings, in the same
      non-fatal, every-mode style as the existing `recovered_encoding_files`/
      `unsafe_encoding_files`/`unclosed_fmt_off_files` blocks. **No change to
      `derive_exit_outcome`** — confirmed directly against `exit.rs`'s
      three-way convention (research.md §6): a config warning never produces
      `ProblemsFound` or `Fatal`. **Depends on T007, T008, T009 (`drut-config`
      fully verified in isolation first, not just compiling — `004`'s
      `block_resolution.rs` sequencing precedent), T010.**
- [x] T012 [US1] In `crates/drut-lsp/src/document_store.rs`: add
      `workspace_root: Option<PathBuf>` to `ServerState`, with a setter (e.g.
      `pub fn set_workspace_root(&mut self, root: Option<PathBuf>)`) — new
      session-level state, not per-document (research.md §5).
- [x] T013 [US1] In `crates/drut-lsp/src/lib.rs`: capture
      `connection.initialize(caps)`'s `Ok` return value (currently discarded —
      only `.is_err()` is checked today), parse it as
      `lsp_types::InitializeParams`, extract `root_uri` (falling back to the
      first `workspace_folders` entry if `root_uri` is absent, per the LSP
      spec's own deprecation order), convert to a `PathBuf` via the new
      `uri_to_path` helper (T014), and call `state.set_workspace_root(...)`
      before entering the main message loop. A client that sends neither
      `root_uri` nor `workspace_folders` (or params that fail to parse) leaves
      `workspace_root` as `None` — not a startup failure. Depends on T012.
- [x] T014 [US1] Create `crates/drut-lsp/src/workspace.rs` with `pub fn
      uri_to_path(uri: &lsp_types::Uri) -> Option<PathBuf>` (research.md §5):
      returns `None` for any non-`file` scheme; for `file`, percent-decodes the
      path component into a `PathBuf`, correctly stripping the leading `/`
      before a Windows drive letter (`file:///C:/foo` → `C:\foo`, not
      `\C:\foo`) — add a unit test for exactly this case, plus a POSIX-style
      `file:///home/user/a.s` case. Add `pub mod workspace;` to `lib.rs`.
- [x] T015 [US1] In `crates/drut-lsp/src/formatting.rs`: replace the current
      `voyager_core::FormatOptions::default()` call with
      `drut_config::resolve_format_options(resolve_path(doc_uri, state), false,
      ExplicitFormatOverride::default())`, where a small local `resolve_path`
      helper tries `workspace::uri_to_path(uri)` first, then falls back to
      `state.workspace_root.clone()`. `isolated` is always `false` here — no
      per-request LSP isolation mechanism exists in this pass (contracts.md's
      explicit non-goal). **Depends on T007, T008, T009 (`drut-config` verified
      in isolation first), T013, T014.**
- [x] T016 [US1] In `crates/drut-lsp/src/range_formatting.rs`: identical
      change to T015 — same `resolve_path` helper, same call shape. **Depends
      on T007, T008, T009, T013, T014.**
- [x] T017 [US1] In `crates/drut-lsp/src/diagnostics.rs`: `publish()` gains a
      third, independently-sourced diagnostics stream, reusing `010`'s exact
      pattern for `unclosed_fmt_off_markers` — for the current document's
      resolved path (same `resolve_path` logic as T015/T016), call
      `drut_config::discover`+`parse` (or a small `drut_config` helper that
      returns just the warnings for a path) and map each `ConfigWarning` to a
      diagnostic with `severity: Some(DiagnosticSeverity::HINT)`, `source:
      Some("drut-config".to_string())`, a message rendering the specific
      problem — chained onto (not replacing) the existing structural and
      `010`-fmt-marker diagnostics streams. **Depends on T007, T008 (`parse`'s
      own unit tests — this task's entire purpose is surfacing `parse`'s
      warnings correctly, so it especially cannot precede T008), T013, T014.**
- [x] T018 [US1] In `crates/drut-mcp/src/format.rs`: add `top_level_indent:
      Option<String>` to `FormatInput` (same `"preserve"`/`"normalize"`/absent
      shape and validation-error style `casing` already has) — closes the
      CLI/MCP asymmetry (spec.md FR-010). Change `format()`'s options
      construction to call `drut_config::resolve_format_options(
      input.source.path.as_deref().map(Path::new), false, explicit)` where
      `explicit` is built from the now-validated `casing`/`top_level_indent`
      inputs. A `text`-sourced call (`input.source.path` is `None`) passes
      `file_path: None` — no discovery attempted. **Depends on T007, T008,
      T009 (`drut-config` verified in isolation first).**
- [x] T019 [US1] In `crates/drut-mcp/src/format.rs`: add `config_warnings:
      Vec<String>` to `FormatResultDto` (human-readable rendering of each
      `ConfigWarning`, matching `unclosed_fmt_off_lines`'s "simple,
      already-rendered" shape). Depends on T018.

### Tests for User Story 1

- [x] T020 [P] [US1] Add tests to `crates/drut-cli/tests/format_flags.rs`: a
      `drut.toml` in a temp directory setting non-default `casing` governs
      `drut format`'s output on a file in that directory with no `--casing`
      flag passed; a malformed `drut.toml` (e.g. an invalid `casing` value)
      produces a stderr notice, formatting still completes normally for that
      file, and the process exit code is `0` (`ExitOutcome::Clean` — never
      changed by a config warning). Depends on T011.
- [x] T021 [P] [US1] Add tests to `crates/drut-lsp/src/formatting.rs`'s and
      `range_formatting.rs`'s own test modules: a document opened from a real
      on-disk path under a directory containing a `drut.toml` produces output
      matching that file's settings via both `textDocument/formatting` and
      `textDocument/rangeFormatting`; an untitled/unsaved document (no real
      path) with no workspace root configured formats with built-in defaults,
      unchanged from before this feature (regression); an untitled document
      whose `ServerState.workspace_root` does point at a directory containing a
      `drut.toml` picks up that file's settings. Depends on T015, T016.
- [x] T022 [P] [US1] Add a test to `crates/drut-lsp/src/diagnostics.rs`'s own
      test module: a document under a directory with a malformed `drut.toml`
      produces exactly one `HINT`-severity, `"drut-config"`-sourced diagnostic
      on `didOpen`, additive to (not replacing) zero structural diagnostics for
      an otherwise-clean document. Depends on T017.
- [x] T023 [P] [US1] Add tests to `crates/drut-mcp/src/format.rs`'s own test
      module: a `path`-sourced `FormatInput` pointing at a file under a
      directory with a `drut.toml` produces output matching that file's
      settings with no `casing`/`top_level_indent` parameter passed; a
      `text`-sourced call (no `path`) never attempts discovery — confirmed by
      placing a `drut.toml` in the current working directory during the test
      and asserting it has zero effect on a `text`-sourced call's output.
      Depends on T018.
- [x] T024 [US1] Add a three-surface parity test proving spec.md US1's own
      Independent Test directly, not just per-surface behavior in isolation: a
      shared temp directory containing one `drut.toml` and one `.s` file,
      formatted via (a) `drut_cli::format_cmd::run` called directly, (b)
      `drut_lsp::formatting::handle` via a real `ServerState`/`did_open` with a
      `file://` URI pointing at the same on-disk file, and (c)
      `drut_mcp::format::format` via a `path`-sourced `FormatInput` pointing at
      the same file — assert all three produce byte-identical formatted text.
      Place in a new integration-test location that can depend on all three
      crates (e.g. `crates/drut-cli/tests/config_parity.rs`, since `drut-cli`
      already depends on both `drut-lsp` and `drut-mcp`). Depends on T011,
      T015, T018.

**Checkpoint**: A shared `drut.toml` is reachable identically from all three
surfaces, malformed files never block any surface, and the untitled-buffer/
workspace-root fallback behaves correctly.

---

## Phase 4: User Story 2 - A user overrides the project default for one run without editing the shared file (Priority: P2)

**Goal**: An explicit `--casing`/`--top-level-indent` flag (CLI) or `casing`/
`top_level_indent` parameter (MCP) wins over a project's `drut.toml` for that one
call, without changing what the next unflagged call sees.

**Independent Test**: With a `drut.toml` setting one casing convention, run
`drut format --casing <opposite>` on a governed file; confirm the opposite
convention is used, and that a subsequent run with no flag reverts to the file's
own setting.

### Implementation for User Story 2

No new implementation code — `resolve_format_options`'s per-field precedence
(T006) and US1's adapter wiring (T011, T018) already pass the real `explicit`
value through correctly. This story is dedicated proof that the precedence holds
end-to-end, not new plumbing.

### Tests for User Story 2

- [x] T025 [P] [US2] Add tests to `crates/drut-cli/tests/format_flags.rs`: with
      a `drut.toml` setting `casing = "lower"`, running `drut format --casing
      upper` on a governed file produces uppercase output (US2 Acceptance
      Scenario 1); running the same file again with no flag afterward reverts
      to lowercase, proving the override was scoped to one invocation, not a
      persistent change (US2 Acceptance Scenario 2). Depends on T011.
- [x] T026 [P] [US2] Add a test to `crates/drut-cli/tests/format_flags.rs`: a
      `drut.toml` that sets only `casing` (leaving `top_level_indent` absent)
      — formatting a governed file with no indent-related flag behaves exactly
      as today with no configuration file present at all for top-level
      indentation specifically (US2 Acceptance Scenario 3 — an unset field
      falls back to the built-in default independently of whether the file
      exists and sets other fields). Depends on T011.
- [x] T027 [P] [US2] Add a test to `crates/drut-mcp/src/format.rs`'s own test
      module: with a `drut.toml` setting `casing = "lower"`, a `path`-sourced
      `FormatInput` with `casing: Some("upper")` produces uppercase output,
      overriding the file; a subsequent call with `casing: None` on the same
      path reverts to lowercase. Depends on T018.

**Checkpoint**: Explicit overrides are proven to win per-field, scoped to one
call, on both surfaces that support them (CLI, MCP) — LSP has no per-request
override mechanism by design (contracts.md's explicit non-goal), so it's not
tested here.

---

## Phase 5: User Story 3 - A user bypasses project configuration entirely for a single run (Priority: P3)

**Goal**: `--isolated` (CLI) / `isolated: true` (MCP) skips `drut.toml` discovery
entirely for one call, running on built-in defaults (plus any other explicit
overrides for that same call).

**Independent Test**: With a `drut.toml` setting non-default values for both
settings, run with the bypass option enabled; confirm the output matches drut's
built-in defaults exactly.

### Implementation for User Story 3

- [x] T028 [US3] In `crates/drut-cli/src/cli.rs`: add `#[arg(long)] isolated:
      bool` to the `Format` variant (no value, a plain flag) — does not
      conflict with `--casing`/`--top-level-indent` (an explicit flag still
      wins even when isolated; isolation only controls whether `drut.toml` is
      consulted).
- [x] T029 [US3] Thread `isolated` through `crates/drut-cli/src/lib.rs`'s
      `Command::Format` match arm and `format_cmd::run`'s signature, replacing
      T011's placeholder `false` literal with the real flag value passed to
      `drut_config::resolve_format_options`. Depends on T011, T028.
- [x] T030 [US3] In `crates/drut-mcp/src/format.rs`: add `isolated:
      Option<bool>` to `FormatInput` (absent treated as `false`); pass
      `input.isolated.unwrap_or(false)` as `resolve_format_options`'s
      `isolated` argument, replacing T018's hardcoded `false`. Depends on T018.

### Tests for User Story 3

- [x] T031 [P] [US3] Add a test to `crates/drut-cli/tests/format_flags.rs`: a
      `drut.toml` setting non-default `casing` and `top_level_indent`;
      `drut format --isolated` on a governed file produces output matching
      drut's built-in defaults for both settings, as if no `drut.toml` existed
      anywhere. Depends on T029.
- [x] T032 [P] [US3] Add a test to `crates/drut-mcp/src/format.rs`'s own test
      module: same setup, `isolated: true` on a `path`-sourced `FormatInput`
      produces built-in-default output, ignoring the file entirely. Depends on
      T030.

**Checkpoint**: All three user stories complete — shared config, explicit
override, and full isolation all independently proven on every surface that
supports them.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Documentation (FR-012), whole-workspace re-proof, and full-corpus
regression, once all three stories are done.

- [x] T033 Add a new "## Configuration" section to `README.md` (placed after
      "Editor behavior: format-on-save and format-on-paste", before
      "Repository layout") documenting: the `drut.toml` schema (the `[format]`
      table, `casing`/`top_level_indent`), discovery (per-file upward walk-up,
      `.git`-boundary/filesystem-root stop), the precedence order (explicit
      override > `drut.toml` > built-in default), the `--isolated` escape
      hatch, and that a malformed file warns without blocking (FR-012 requires
      this be predictable from documentation alone, not just source code).
- [x] T034 `cargo test --release --workspace` and `cargo clippy --workspace
      --all-targets -- -D warnings`, both clean.
- [x] T035 Full-corpus regression across all three adapter surfaces (SC-003 —
      zero `drut.toml` anywhere in the real corpus, so this proves zero
      behavior change):
      ```powershell
      $env:DRUT_CORPUS_PATH = "path\to\WF-TDM-Official-Releases"
      cargo test --release -p drut-cli --test fixture_corpus_e2e -- --ignored
      cargo test --release -p drut-lsp --test diagnostics_corpus -- --ignored
      cargo test --release -p drut-mcp --test diagnostics_corpus -- --ignored
      ```
- [ ] T036 Run quickstart.md end-to-end (all 7 steps, including the manual VS
      Code step); confirm each step's expected outcome individually before
      reporting the feature done.

**Checkpoint**: Feature-complete against spec.md; documented per FR-012; full
workspace and full corpus re-proven clean.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup. Blocks every user story — none
  of US1/US2/US3 can begin **calling** `drut-config` until its public surface
  exists **and its own tests (T007, T008, T009) pass** — not merely until it
  compiles (`004`'s `block_resolution.rs` sequencing precedent, confirmed via
  `/speckit-analyze` review: every task that actually invokes `discover`/
  `parse`/`resolve_format_options` states T007/T008/T009 as an explicit
  dependency, not just the implementation tasks T003–T006).
- **User Story 1 (Phase 3)**: Depends on Foundational **in full, through
  T009** — not merely through T006. This is the foundation everything else
  builds on — US2 and US3 both build on T011/T018's wiring.
- **User Story 2 (Phase 4)**: Depends on US1's T011 (CLI wiring) and T018 (MCP
  wiring) — no new implementation, only tests against wiring that already exists.
- **User Story 3 (Phase 5)**: Depends on US1's T011/T018 (the `isolated`
  placeholder each replaces).
- **Polish (Phase 6)**: Depends on all three stories being complete.

### Parallel Opportunities

- T007, T008, T009 can proceed in parallel once their respective dependencies
  (T004, T005, T006) land — different test files.
- T012, T013, T014 (LSP session-state/`uri_to_path` groundwork) call **no**
  `drut-config` function themselves — pure LSP-side state/URI-parsing work —
  so they may proceed in parallel with Foundational, starting right after
  Setup, with no dependency on T003–T009 at all.
- T010 (CLI clap-type change) also calls no `drut-config` function and may
  proceed the same way.
- Everything that actually **calls** `drut-config` — T011, T015, T016, T017,
  T018 — must wait for **all of T007, T008, T009**, not just T006, per the
  Foundational dependency correction above. Once that gate clears, T011 (CLI),
  T015/T016/T017 (LSP, additionally needing T013/T014), and T018 (MCP) may all
  proceed in parallel with each other — different crates, no dependency on one
  another.
- T020, T021, T022, T023 can proceed in parallel once their respective
  implementation dependencies land — different files, non-conflicting. T024
  depends on all of T011/T015/T018 landing first (it exercises all three).
- T025, T026, T027 can proceed in parallel once T011/T018 land.
- T031, T032 can proceed in parallel once T029/T030 land.

---

## Implementation Strategy

### Single Pass (foundation-heavy, three thin story layers on top)

1. Setup → baseline confirmed clean, `drut-config` crate exists.
2. Foundational → `drut-config`'s complete, independently-tested public
   surface — the one substantial implementation phase in this feature.
3. User Story 1 → wire all three adapters to call `resolve_format_options`,
   plus the LSP-specific session-state additions (`workspace_root`,
   `uri_to_path`) research.md §5 found were missing entirely. This is where
   almost all the adapter-layer code changes live.
4. User Story 2 → proof only, no new code.
5. User Story 3 → one new flag/param per surface (CLI, MCP), replacing a
   placeholder each of US1's wiring tasks left behind on purpose.
6. Polish → documentation, whole-workspace and full-corpus re-proof.

---

## Notes

- T011 and T018 each leave a hardcoded `isolated: false` in place deliberately —
  this keeps US1's scope to exactly what US1 needs (shared config reachable
  everywhere) without pulling US3's flag/param into an earlier story. T029/T030
  are the only tasks that touch those two lines again.
- T024 (three-surface parity) is this feature's single most direct proof of its
  own stated purpose (spec.md US1's Independent Test, verbatim) — do not treat
  it as redundant with T020/T021/T023's own per-surface coverage.
- Commit after each task or logical group.
