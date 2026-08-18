# Contract: Editor-Settings Exposure for `[format]` Config Fields (addition)

A conceptual signature contract, not final Rust source, but the shapes and guarantees below are
binding — same convention every prior contract doc in this repo follows.

## `drut-config` additions

```text
pub fn resolve_format_options(
    file_path: Option<&Path>,
    isolated: bool,
    explicit: ExplicitFormatOverride,
    client_defaults: ExplicitFormatOverride,   // new, 4th positional argument
) -> (voyager_core::FormatOptions, Vec<ConfigWarning>)
```

- **Precedence, exactly**: `explicit.field.or(config.format.field).or(client_defaults.field)
  .unwrap_or_default()` — `client_defaults` is consulted only after both `explicit` and
  `drut.toml` have had a chance to set the field, never before either (spec.md FR-003).
  `control_words`/`pair_keywords` are the two exceptions: each tier (`explicit`, `config`, and
  now `client_defaults`) applies its *own* granular-then-legacy-`casing` fallback internally, so
  the full chain for `control_words` is `explicit.control_words_casing.or(explicit.casing)
  .or(config.format.control_words_casing).or(config.format.casing)
  .or(client_defaults.control_words_casing).or(client_defaults.casing).unwrap_or_default()`
  (research.md §1) — a client setting only `drut.format.casing` (not the granular equivalent)
  still reaches these two fields, matching how the legacy field already behaves at every other
  tier.
- **Existing callers unaffected**: every CLI/MCP call site passes `ExplicitFormatOverride::
  default()` for the new parameter — behavior for those two surfaces is byte-for-byte identical
  to before this feature (spec.md FR-007).
- **Invalid client-setting values degrade, never fail**: the same non-blocking-notice-and-
  fallback pattern every other malformed config value in this project already uses (spec.md
  FR-005).

## `drut-lsp` additions

- `ServerState::{set_client_format_defaults, client_format_defaults}` — a single cached
  `ExplicitFormatOverride` value, `Default` until the first successful pull.
- A second server-initiated request (`workspace/configuration`, section `"drut.format"`),
  fire-and-forget, same shape as the existing `client/registerCapability` request — never blocks
  the main loop, a missing/unparseable response simply leaves the cache at its previous value.
- `workspace/didChangeConfiguration` triggers a re-pull; its own notification payload is never
  read as a data source (research.md §3).
- `formatting.rs`/`range_formatting.rs`: both now pass `state.client_format_defaults()` instead
  of `ExplicitFormatOverride::default()` — no other change to either handler's own logic.

## `editors/vscode` additions

- `package.json`'s `contributes.configuration` declares all 10 fields under `drut.format.*`
  (camelCase), no `"default"` on any property (data-model.md §3) — an unset VS Code setting
  correctly means "not present," not a hidden second source of the built-in default.

## What this contract does *not* promise (by design, this phase)

- No per-workspace-folder/multi-root scoping for client settings — one single global pull
  (research.md §5), a deliberate simplification, not an oversight.
- No CLI/MCP reach — `drut-cli`/`drut-mcp` gain nothing from this feature (spec.md FR-007).
- No new formatting axis, no new accepted values for any existing field — this is purely a new
  *source* for the same 10 fields that already exist.
