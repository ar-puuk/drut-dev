# Phase 0 Research: TOML-Based Configuration

All findings below are measured against the real, current codebase
(`crates/drut-cli/src/cli.rs`, `format_cmd.rs`, `traverse.rs`, `lib.rs`, `exit.rs`;
`crates/drut-lsp/src/formatting.rs`, `range_formatting.rs`, `document_store.rs`,
`lib.rs`; `crates/drut-mcp/src/format.rs`, `source.rs`; `crates/voyager-core/src/
format.rs`), plus the vendored `lsp-server 0.10.0` and `lsp-types 0.97.0` crate
source, and `toml`'s current crates.io listing — not estimated. A prior conversation
turn already established the high-level design (schema, `drut.toml` naming,
`drut-config` crate) and Ruff-convention research (file discovery, table nesting,
precedence, `--isolated`); this document supplements that with the concrete
code-level facts needed to actually implement it.

## §1. `--top-level-indent`'s current CLI type cannot distinguish "not passed" from "explicitly Preserve"

**Finding**: `cli.rs`'s `Format` variant declares:

```rust
#[arg(long, value_enum, default_value_t = TopLevelIndentArg::Preserve)]
top_level_indent: TopLevelIndentArg,
```

`default_value_t` means clap *always* produces a concrete `TopLevelIndentArg` value —
there is no way, today, for `format_cmd::run` to tell "the user didn't pass this
flag" apart from "the user explicitly passed `--top-level-indent=preserve`". `casing`
has no such problem (`Option<CasingArg>`, already `None` when absent).

**Decision**: Change `top_level_indent`'s CLI type to `Option<TopLevelIndentArg>`
(drop `default_value_t`) — required, not optional, for TOML precedence (FR-006) to
work correctly. Without this change, an explicit `Preserve` flag and "no flag, consult
TOML" would be indistinguishable, silently making TOML's `top_level_indent` setting
unreachable from the CLI whenever a user's shell alias or script habitually passes
`--top-level-indent=preserve` explicitly. This is purely an internal representation
change — observable behavior when no `drut.toml` exists anywhere is unchanged
(`None` still resolves to `Preserve`, spec.md FR-007).

## §2. `format_cmd::run` resolves `FormatOptions` once, before the traversal loop — must move inside it

**Finding**: `format_cmd.rs`'s `run` builds a single `FormatOptions` value from the
CLI's flags *before* `traverse(path)` even runs, then reuses that one value for every
matched file in the loop (`for file in &traversal.matched_files { let result =
format_bytes(&file.bytes, options); ... }`).

**Decision**: Per-file resolution (spec.md FR-003's "genuine per-file walk-up," not
once-per-invocation) requires moving `FormatOptions` construction *inside* the loop,
resolved fresh from each `file.path` — `traverse.rs`'s `MatchedFile.path` is already
a real, usable on-disk path for this purpose, no traversal change needed. This
directly mirrors Ruff's own documented behavior (the closest config file is used "for
every individual file," even within one multi-file invocation) rather than the
CLI-batch simplification originally sketched before that research.

## §3. `voyager-core`'s zero-dependency rule is crate-scoped, confirmed directly — new crate is required, not just recommended

**Finding**: Both `drut-cli/Cargo.toml` and `drut-mcp/Cargo.toml` carry an identical
comment confirming this explicitly: *"Not bound by voyager-core's zero-dependency
rule (that constraint is scoped to the core crate specifically...)."* `voyager-core`
itself has zero `[dependencies]` beyond nothing (`std` only). A `toml`+`serde`-based
parser therefore cannot live in `voyager-core` without violating FR-027, confirmed
directly rather than inferred.

**Decision**: `drut-config`, a new crate depending on `toml = "1"` (verified current
on crates.io: 1.1.2+spec-1.1.0, no dependency-bump risk) and `serde = "1.0.229"`
(exact version already used identically by both `drut-cli` and `drut-mcp` — no
version-skew risk), plus a path dependency on `voyager-core` for the `FormatOptions`/
`CasingConvention`/`TopLevelIndentMode` types it ultimately produces. `voyager-core`
gains no dependency and no awareness of `drut-config` — a one-directional
dependency, matching Principle I's existing flow (adapters depend on core, never the
reverse).

`drut-config` cannot deserialize directly into `voyager_core::CasingConvention`/
`TopLevelIndentMode`, since those types deliberately derive no `serde::Deserialize`
(that would itself add a dependency to `voyager-core`). `drut-config` therefore
defines its own small serde-deserializable schema types and converts them via `From`,
the identical pattern `format_cmd.rs` already uses for `CasingArg -> CasingConvention`.

## §4. Per-field fallback on a malformed file requires manual `toml::Value` walking, not a single struct-level `Deserialize`

**Finding**: spec.md's FR-011/Assumptions (confirmed explicitly with the owner before
`/speckit-plan`) commit to *per-setting* fallback — one bad key doesn't invalidate
every other valid key in the same file. A single `#[derive(Deserialize)]` onto a
strict schema struct fails the whole file on any one field's problem (wrong type,
unrecognized key with `deny_unknown_fields`), which does not satisfy that
requirement.

**Decision**: `drut-config::parse` parses into a generic `toml::Value` first (a full,
real TOML-spec parse — this is exactly what avoids the "hand-rolled parser silently
mishandles valid syntax" risk flagged before `drut-config` was chosen), then walks the
`[format]` table by hand, validating each of the two known keys (`casing`,
`top_level_indent`) individually. Any problem with one key — wrong type, an
unrecognized string value — produces one `ConfigWarning` for that key and that key
alone falls back to the built-in default; every other valid key still applies. A file
that fails to parse as TOML at all produces one `ConfigWarning::ParseError` and every
setting it would have provided falls back to default (spec.md's Edge Cases already
draws exactly this line: whole-file failure only for total syntax failure).

**A further distinction, worth recording explicitly**: an unrecognized *key inside a
known table* (e.g. `[format]` with `csing = "upper"`, a plausible typo of an already-
in-use table) warns — this is exactly FR-011's "don't be silently confusing" case. An
unrecognized *top-level table* (e.g. a hypothetical future `[lint]` written by a newer
`drut-config` and then read by an older one) is silently ignored, not warned — a whole
extra bracketed section is a much less plausible accidental typo than one key inside a
table already in active use, and warning on it would make every schema addition a
breaking change for anyone still on an older `drut` version. This asymmetry is
deliberate, not an oversight.

## §5. `drut-lsp` has no URI→filesystem-path conversion today, and doesn't capture the client's workspace root at all

**Finding**: `lsp_types::Uri` (from `lsp-types 0.97.0`) is a newtype around
`fluent_uri::Uri<String>` with no `to_file_path()`-style method (confirmed by reading
`uri.rs` directly) — every existing `drut-lsp` handler only ever compares/echoes
`Uri` values as opaque keys (`ServerState`'s `HashMap<Uri, OpenDocument>`), never
converts one to a real on-disk path. Separately, `lib.rs::run` calls
`connection.initialize(caps)` (from `lsp-server 0.10.0`) and discards its `Ok` value
— which is, per that function's own source, the raw client `InitializeParams` JSON,
including `root_uri`/`workspace_folders`. Neither piece of information is captured
anywhere in `ServerState` today.

**Decision**: Two small, genuinely new (not just "reuse existing plumbing") additions
to `drut-lsp`:
1. A `uri_to_path(uri: &lsp_types::Uri) -> Option<PathBuf>` helper (new, `drut-lsp`-
   local — this is LSP-wire-format-specific translation, not a `drut-config` concern,
   matching `position.rs`'s existing "translation lives at the LSP boundary" pattern)
   handling the `file://` scheme and the well-known Windows drive-letter leading-slash
   gotcha (`file:///C:/foo` → `C:\foo`, not `\C:\foo`).
2. `ServerState` gains `workspace_root: Option<PathBuf>`, populated once by parsing
   `connection.initialize(caps)`'s returned `Ok(params)` into `lsp_types::
   InitializeParams` and extracting `root_uri` (falling back to the first
   `workspace_folders` entry if `root_uri` is absent, per the LSP spec's own
   documented deprecation order) — used only for spec.md's untitled-buffer fallback
   case (a document with no real path falls back to the workspace root's own
   configuration, if any, rather than skipping straight to built-in defaults).

## §6. Malformed-config surfacing per adapter — reusing each surface's own already-established idiom, not inventing a fourth

**Decision**, directly mirroring `010-fmt-region-markers`'s own multi-surface
precedent for "a non-fatal, non-`Diagnostic` notice":
- **CLI**: a new, `unclosed_fmt_off_files`-shaped report field
  (`config_warnings: Vec<(PathBuf, Vec<ConfigWarning>)>` or equivalent), printed via
  a dedicated `eprintln!` block in `print_report`, same non-fatal treatment — per
  `exit.rs`'s own three-way convention, this **never changes the exit code**
  (`ExitOutcome::Clean` stays `Clean`), exactly matching `unclosed_fmt_off_files`'s
  own explicit "informational only; never affects the exit code" comment. Confirmed
  directly against `exit.rs` rather than assumed.
- **LSP**: a diagnostic on the affected document(s), reusing the exact pattern
  `010` established in `diagnostics.rs` — `DiagnosticSeverity::HINT` (or `WARNING`,
  given a config problem is arguably more actionable than an unclosed marker; Phase 1
  contract decides), a distinct `source` tag (e.g. `"drut-config"`, parallel to
  `010`'s `"drut-fmt"`), never a `voyager_core::DiagnosticKind` variant.
- **MCP**: a new response field on `FormatResultDto`, mirroring
  `unclosed_fmt_off_lines`'s own shape (a `Vec<String>` of human-readable warning
  messages, or a small structured DTO — Phase 1 contract decides the exact shape).

## §7. `.git`-boundary detection and Windows path semantics

**Decision**: At each ancestor directory during upward walk-up, check for `drut.toml`
first; if absent, check for a `.git` entry (a directory in the common case, but a
worktree's `.git` is a *file* containing a `gitdir:` pointer — the boundary check only
needs to detect *presence*, not parse worktree redirection, so `Path::exists()`
against `dir.join(".git")` is sufficient regardless of which it is) and stop there if
found; otherwise continue to the parent, stopping unconditionally at the filesystem
root. Every path operation goes through `std::path::Path`/`PathBuf`, which already
handles Windows drive-letter and separator semantics correctly — no OS-specific
branching needed beyond what `std` already provides, confirmed by this being the same
approach `traverse.rs`'s own directory walking already relies on (via the `ignore`
crate, itself built on `std::path`).

## §8. Scope confirmation: `check` subcommand and three of four MCP tools are unaffected

**Finding**: `check_cmd.rs` and `main.rs` have zero references to `FormatOptions`/
`casing`/`top_level_indent` (confirmed via direct grep) — `check` reports structural
diagnostics only, entirely unrelated to formatting settings. Of the four MCP tools,
only `format` uses `FormatOptions` at all; `diagnose`/`query_structure` take a
`ScriptSource` but never touch casing/indent; `lookup_keyword` takes no source at
all (confirmed via `source.rs`'s own doc comment: "the shared text-or-path input
shape every tool but `lookup_keyword` accepts").

**Decision**: This feature's CLI surface is `drut format` only (no `--isolated` on
`check`); its MCP surface is the `format` tool only. No other subcommand or tool
changes.
