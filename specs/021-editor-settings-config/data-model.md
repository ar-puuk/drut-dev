# Data Model: Editor-Settings Exposure for `[format]` Config Fields

## §1. `drut-config` additions

### `resolve_format_options` (modified)

```rust
pub fn resolve_format_options(
    file_path: Option<&Path>,
    isolated: bool,
    explicit: ExplicitFormatOverride,
    client_defaults: ExplicitFormatOverride,   // new — research.md §1
) -> (voyager_core::FormatOptions, Vec<ConfigWarning>)
```

- `client_defaults` reuses `ExplicitFormatOverride`'s existing 10-field shape (research.md §1) —
  not a new struct, but a distinct *parameter*, never merged into `explicit` itself (that would
  incorrectly promote it to CLI/MCP-tier precedence).
- Every existing caller (CLI's `format`/`check` commands, MCP's `format` tool) passes
  `ExplicitFormatOverride::default()` for this new parameter — client settings only ever have a
  real value at the LSP call sites (spec.md FR-007: CLI/MCP behavior is completely unchanged).
- `resolve_casing_and_indent`'s per-field chains each gain one more `.or(client_defaults.field)`
  immediately before `.unwrap_or_default()` — **except** `control_words`/`pair_keywords`, which
  each gain *two* (`.or(client_defaults.control_words_casing).or(client_defaults.casing)`,
  respectively), mirroring the legacy-then-granular fallback both the `explicit` and `config`
  tiers already apply for these two fields specifically (research.md §1's precision correction —
  a real gap caught during checklist review, not a stylistic choice). `resolve_indent_width`/
  `resolve_blank_line_cap` gain a third fallback argument the same way, validated identically to
  how a `drut.toml` value already is (an out-of-range client-setting value degrades to the
  built-in default with a non-blocking notice — spec.md FR-005).

## §2. `drut-lsp` additions

### `ServerState` (modified)

```rust
pub struct ServerState {
    // ...existing fields unchanged...
    client_format_defaults: drut_config::ExplicitFormatOverride,   // new
}

impl ServerState {
    pub fn set_client_format_defaults(&mut self, defaults: drut_config::ExplicitFormatOverride);
    pub fn client_format_defaults(&self) -> drut_config::ExplicitFormatOverride;   // Copy-able
}
```

- Same shape as the existing `workspace_root` field/getter/setter pair — a single cached value,
  `Default` (all `None`) until the first successful pull completes (research.md §2).

### `lib.rs` additions

```rust
/// Whether the client advertised support for `workspace/configuration`
/// (research.md §2) — same "no static-capability alternative, confirmed
/// against lsp-types' own capability doc comments" shape
/// `did_change_watched_files_supported` already established for the file
/// watcher.
fn workspace_configuration_supported(params: Option<&lsp_types::InitializeParams>) -> bool;

/// Sends the (now second) request this server ever initiates: asking for
/// the merged "drut.format" section (research.md §4). Fire-and-forget, same
/// shape as `register_drut_toml_watcher` — does not block the main loop.
fn request_client_format_defaults(connection: &Connection);
```

- `handle_response` (already generic over every response this server might receive) gains one
  more match arm: a response whose ID matches this request parses the returned JSON object's 10
  known field names into an `ExplicitFormatOverride`, calling `ServerState::
  set_client_format_defaults`. An unparseable/missing field is simply left `None` for that field
  (same "malformed value degrades to the next tier" contract, not a hard failure).
- `handle_notification` gains a `workspace/didChangeConfiguration` arm that calls
  `request_client_format_defaults` again — the payload itself is never read (research.md §3).
- `formatting.rs::handle`/`range_formatting.rs::handle` each change their
  `resolve_format_options` call from `ExplicitFormatOverride::default()` (the fourth positional
  argument) to `state.client_format_defaults()`.
- `initialize`'s capability response is unaffected — `workspace/configuration` requires no new
  *client*-facing capability declaration from this server (it's a request the server *sends*, not
  one the client sends to it); only the request registration itself is conditioned on
  `workspace_configuration_supported`.

## §3. `editors/vscode/package.json` additions

```jsonc
"contributes": {
  "configuration": {
    "title": "Drut",
    "properties": {
      "drut.format.casing": { "type": "string", "enum": ["preserve", "upper", "lower"] },
      "drut.format.controlWordsCasing": { "type": "string", "enum": ["preserve", "upper", "lower"] },
      "drut.format.pairKeywordsCasing": { "type": "string", "enum": ["preserve", "upper", "lower"] },
      "drut.format.dataReferencesCasing": { "type": "string", "enum": ["preserve", "upper", "lower"] },
      "drut.format.topLevelIndent": { "type": "string", "enum": ["preserve", "normalize"] },
      "drut.format.indentWidth": { "type": "integer", "minimum": 1, "maximum": 16 },
      "drut.format.operatorSpacing": { "type": "string", "enum": ["preserve", "fixed", "auto"] },
      "drut.format.blankLines": { "type": "string", "enum": ["preserve", "auto"] },
      "drut.format.topLevelBlankLineCap": { "type": "integer", "minimum": 0 },
      "drut.format.nestedBlankLineCap": { "type": "integer", "minimum": 0 }
    }
  }
}
```

- Every property omits a `"default"` — a VS Code setting with no declared default resolves to
  `undefined` until a user actually sets it, which is exactly "not present in the pulled
  `workspace/configuration` object" from the server's own point of view (`serde_json`'s standard
  "absent key" handling), preserving the "no client setting configured" case correctly (spec.md
  US1 AS3) without this project inventing its own sentinel value for "unset."
- Enum/range values mirror each field's own accepted-value set exactly (data-model.md §1/§4 in
  `017`/`018`/`019`'s own docs) — no new value vocabulary invented here.

## §4. What this feature does *not* touch

- `drut-cli`, `drut-mcp`: zero source changes (spec.md FR-007) — both keep passing
  `ExplicitFormatOverride::default()` for the new parameter, unconditionally.
- `voyager_core::FormatOptions`/every formatting axis's own enum types: unchanged — this feature
  adds a new *source* for existing values, never a new value or a new axis.
- `diagnostics.rs`'s `config_warnings` stream: unchanged (research.md §6).
- `drut-config`'s TOML parsing (`parse.rs`): unchanged — client settings never come from
  `drut.toml` parsing, only from the LSP `workspace/configuration` response.
