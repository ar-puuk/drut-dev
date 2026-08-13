# Contract: TOML-Based Configuration

Covers the new `drut-config` crate's public surface and each adapter's integration
point.

## Schema (`drut.toml`)

```toml
[format]
casing = "lower"                 # "upper" | "lower"; any other string -> ConfigWarning::InvalidValue, field falls back to None
top_level_indent = "normalize"   # "preserve" | "normalize"; any other string -> ConfigWarning::InvalidValue, field falls back to None
```

- Any key inside `[format]` other than `casing`/`top_level_indent` ->
  `ConfigWarning::UnrecognizedKey`, ignored, every other valid key in the file still
  applies (research.md §4).
- Any top-level table other than `[format]` is silently ignored (forward-compat,
  research.md §4) — not a warning.
- A file that fails to parse as TOML at all -> one `ConfigWarning::ParseError`, every
  field falls back to the built-in default, exactly as if no file existed.

## `drut-config` public API

```rust
pub struct DrutConfig { pub format: FormatConfig }
pub struct FormatConfig {
    pub casing: Option<voyager_core::CasingConvention>,
    pub top_level_indent: Option<voyager_core::TopLevelIndentMode>,
}
pub enum ConfigWarning {
    ParseError { path: PathBuf, message: String },
    UnrecognizedKey { path: PathBuf, table: String, key: String },
    InvalidValue { path: PathBuf, table: String, key: String, message: String },
}
pub struct ExplicitFormatOverride {
    pub casing: Option<voyager_core::CasingConvention>,
    pub top_level_indent: Option<voyager_core::TopLevelIndentMode>,
}

/// Walk upward from `start` (a file or directory) for the nearest `drut.toml`.
/// Stops at the first file found, a `.git` boundary, or the filesystem root.
/// Never panics; `start` not existing is treated the same as "nothing found."
pub fn discover(start: &Path) -> Option<PathBuf>;

/// Parse `path`'s content. Never returns `Err` for a content problem (syntax,
/// unrecognized/invalid keys) -- those become `ConfigWarning`s alongside a
/// best-effort `DrutConfig` (fields that couldn't be resolved are `None`).
/// Only an I/O failure to even read `path` is a hard error (the file existing,
/// per `discover`, but becoming unreadable between discovery and parse --
/// treated as a `ConfigWarning::ParseError` too, not a panic or `Result::Err`,
/// since callers must never be blocked, FR-011).
pub fn parse(path: &Path) -> (DrutConfig, Vec<ConfigWarning>);

/// The one entry point every adapter calls.
pub fn resolve_format_options(
    file_path: Option<&Path>,
    isolated: bool,
    explicit: ExplicitFormatOverride,
) -> (voyager_core::FormatOptions, Vec<ConfigWarning>);
```

**Never panics** on any input, including a `file_path` that doesn't exist, a
`drut.toml` with arbitrary/adversarial content, or `None` for `file_path` — matches
every other public entry point's guarantee across this workspace (`voyager-core`'s
own `tokenize`/`parse`, `all_blocks`, etc.).

**Deterministic**: identical `(file_path, isolated, explicit)` and identical
filesystem content produce an identical result every call — no caching, no ambient
state beyond the filesystem itself (spec.md's Key Entities: "read fresh at resolution
time").

## CLI (`drut format`)

- `--isolated` (new bool flag, no value): when set, `resolve_format_options` is
  called with `isolated: true` for every file. Does not conflict with `--casing`/
  `--top-level-indent` — an explicit flag still wins even when isolated; isolation
  only controls whether `drut.toml` is *consulted* at all.
- `top_level_indent`'s CLI type changes from `TopLevelIndentArg` (always has a
  value) to `Option<TopLevelIndentArg>` (research.md §1) — `None` when the flag
  isn't passed, matching `casing`'s existing shape. Behavior when no `drut.toml`
  exists anywhere is unchanged (`None` -> `Preserve`, same as today).
- `FormatOptions` construction moves from once-before-the-loop to once-per-matched-
  file, inside `format_cmd::run`'s existing loop (research.md §2), resolved from
  each `MatchedFile.path`.
- `FormatReport` gains `config_warnings: Vec<(PathBuf, Vec<ConfigWarning>)>`,
  populated in every mode (same treatment as `unclosed_fmt_off_files`), printed via
  a new `eprintln!` block in `print_report`. **Never changes `derive_exit_outcome`'s
  result** — confirmed against `exit.rs`'s three-way convention directly
  (research.md §6); a config warning is informational, matching
  `unclosed_fmt_off_files`'s own explicit precedent, not a `ProblemsFound`/`Fatal`
  condition.

## LSP (`textDocument/formatting`, `textDocument/rangeFormatting`)

- `ServerState` gains `workspace_root: Option<PathBuf>`, set once from
  `InitializeParams.root_uri` (falling back to the first `workspace_folders` entry)
  at `initialize` time (research.md §5).
- A new `uri_to_path(uri: &lsp_types::Uri) -> Option<PathBuf>` helper (`drut-lsp`-
  local, not `drut-config`) converts a `file://` URI to a real path, correctly
  handling the Windows drive-letter leading-slash case.
- Both `formatting.rs::handle` and `range_formatting.rs::handle` replace their
  current `voyager_core::FormatOptions::default()` call with:
  `drut_config::resolve_format_options(path_for(&params.text_document.uri, state), false, ExplicitFormatOverride::default())`
  where `path_for` tries `uri_to_path` first, then falls back to
  `state.workspace_root.clone()`, then `None`. `isolated` is always `false` here —
  no per-request LSP mechanism to request isolation exists in this pass
  (deliberately out of scope, matching spec.md's Assumptions).
- Malformed-config warnings surface as a new, additive diagnostics stream in
  `diagnostics.rs`, reusing `010`'s exact pattern: `DiagnosticSeverity::HINT`,
  `source: "drut-config"`, one diagnostic per `ConfigWarning` on the affected
  document — chained onto, not replacing, the existing structural- and
  `010`-fmt-marker diagnostics streams.

## MCP (`format` tool)

- `FormatInput` gains two new optional fields:
  `top_level_indent: Option<String>` (`"preserve"`/`"normalize"`/absent, same
  validation-error shape `casing` already has for an invalid string) and
  `isolated: Option<bool>` (absent treated as `false`).
- `format()`'s options-construction resolves via
  `drut_config::resolve_format_options(input.source.path.as_deref().map(Path::new), input.isolated.unwrap_or(false), explicit)`
  where `explicit` is built from `input.casing`/`input.top_level_indent` exactly as
  `casing_option` already validates `casing` today. A `text`-sourced call (no
  `path`) passes `file_path: None` — no discovery attempted, matching the LSP
  untitled-buffer case's own "no real location, no lookup" rule.
- `FormatResultDto` gains `config_warnings: Vec<String>` (human-readable rendering
  of each `ConfigWarning`, matching `unclosed_fmt_off_lines`'s "simple, already-
  rendered" shape rather than exposing `ConfigWarning`'s own Rust enum shape over
  the wire).

## Non-goals (explicitly out of contract)

- No `extend`/config-inheritance mechanism (spec.md Assumptions).
- No dotted `.drut.toml` higher-precedence variant (spec.md Assumptions).
- No per-request LSP isolation mechanism (`initializationOptions`-based override is a
  possible future refinement, not built here).
- No new `voyager_core::DiagnosticKind` variant of any kind.
