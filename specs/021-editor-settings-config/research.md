# Phase 0 Research: Editor-Settings Exposure for `[format]` Config Fields

Grounded directly in `crates/drut-config/src/lib.rs`, `crates/drut-lsp/src/{lib,formatting,
range_formatting}.rs`, and `editors/vscode/`'s existing structure as they exist today — every
finding below was confirmed by reading the actual code before being treated as a planning fact.

## §1: `resolve_format_options`'s existing per-field `.or()` chain is where the new tier slots in

`drut_config::resolve_casing_and_indent` resolves each of the 10 fields with a chain shaped like
`explicit.field.or(config.format.field).unwrap_or_default()` (two fields — `indent_width`,
`top_level_blank_line_cap`/`nested_blank_line_cap` — go through a small validating helper
instead, for their range checks). Confirmed directly: there is currently no third source in this
chain at all.

**Decision**: add one more `.or(client_defaults.field)` immediately before `.unwrap_or_default()`
in every chain, and thread the same value through the two numeric-range-validating helpers as a
third fallback source. `ExplicitFormatOverride`'s existing 10-field shape is reused (not
duplicated) for the new parameter's type — this project already has precedent for two
identically-shaped structs with different precedence semantics (`FormatConfig` from TOML,
`ExplicitFormatOverride` from CLI/MCP); a third value of the same shape, from client settings, is
a continuation of that pattern, not a new one. The new parameter is named for its own role
(`client_defaults`), not reused as another `ExplicitFormatOverride` *instance* mixed into the
existing `explicit` value — mixing would incorrectly give it CLI/MCP-tier precedence.

**Precision caught during checklist review (CHK001/CHK002)**: `control_words`/`pair_keywords`
are not "one more `.or()` each" — both fields already have their *own* internal legacy-`casing`
fallback within the `explicit` tier (`explicit.control_words_casing.or(explicit.casing)`) and
again within the `config`/`drut.toml` tier (`config.format.control_words_casing.or(config.
format.casing)`). A `client_defaults` tier that only added `.or(client_defaults.
control_words_casing)` without a matching `.or(client_defaults.casing)` afterward would silently
make the client-settings tier's legacy `casing` field a no-op for these two fields specifically —
inconsistent with how the other two tiers already treat it, and a real behavior gap a user
setting `drut.format.casing` (without the granular one) would hit. **Decision, corrected**: these
two fields get *two* additional fallback steps at the client-settings tier
(`.or(client_defaults.control_words_casing).or(client_defaults.casing)`), mirroring the existing
two-tier pattern exactly; every other field (no legacy counterpart exists) gets exactly one.

## §2: The main loop's *only* existing server-initiated request is fire-and-forget — the precedent to follow, not a blocking wait

`drut-lsp/src/lib.rs::register_drut_toml_watcher` (`013-lsp-config-file-watch`) is, per its own
doc comment, "the one and only request this server ever initiates" — it sends a
`client/registerCapability` request and explicitly does **not** wait for the response; the main
loop (`for msg in &connection.receiver { match msg { ... } }`) never blocks on any single message,
and an unconfirmed/never-arriving response is documented as a non-issue, handled generically
whenever (if ever) it shows up.

**Decision**: `workspace/configuration` follows the identical shape, not a new blocking-request-
response pattern this codebase has never used. Fire the request once client support is confirmed
at `initialize` time (and again on every `workspace/didChangeConfiguration` notification); cache
whatever comes back in `ServerState`; `formatting.rs`/`range_formatting.rs` read whatever is
currently cached (possibly "nothing yet," on the very first request before the initial pull
response arrives — which just means the client-setting tier is empty for that one request,
self-correcting the moment the cache populates). No new synchronous-wait machinery is added to
the main loop.

## §3: `workspace/didChangeConfiguration`'s payload is not the data source — it's a re-pull trigger

Confirmed via `vscode-languageclient`'s own bundled implementation
(`editors/vscode/node_modules/vscode-languageclient/lib/common/configuration.js`): the modern
client sends this notification with `settings: null` (the deprecated "push the actual values"
shape is explicitly noted as superseded by "the new pull model (`workspace/configuration`
request)" in `vscode-languageserver-protocol`'s own type comments).

**Decision**: this server never reads `DidChangeConfigurationParams.settings` for real values —
receiving the notification at all is treated purely as "something changed, re-fire
`workspace/configuration` and refresh the cache," identical treatment regardless of whatever the
notification's own payload happens to contain.

## §4: One `workspace/configuration` request, one section, not ten

`workspace/configuration`'s `items` array can request an arbitrary set of dotted section strings
in one round trip; a client (VS Code included) resolves a section like `"drut.format"` to the
merged object of every setting declared under that `contributes.configuration` prefix, not one
request per leaf setting.

**Decision**: request exactly one section, `"drut.format"`, once per pull — not 10 separate
per-field requests. `package.json`'s `contributes.configuration` properties are declared as
`drut.format.<camelCaseFieldName>` (e.g. `drut.format.controlWordsCasing`), the same dotted
naming VS Code's own built-in settings use.

## §5: No per-document/per-workspace-folder scoping — a deliberate, not overlooked, simplification

`workspace/configuration`'s `items` also accept a `scopeUri` per item, letting a server ask for a
value resolved specifically for one folder (respecting a multi-root workspace's per-folder
overrides). This project has no existing multi-root-workspace handling anywhere else to extend.

**Decision**: pull one single, global `"drut.format"` value — not scoped per document or per
workspace folder. This is coherent with, not just simpler than, the feature's own precedence
design (spec.md FR-003): `drut.toml` is already the per-project, scope-aware, team-shared
config layer (discovered per-document by walking up from its own path); client settings are
explicitly the *personal, single global fallback* layer one level below it. Giving client
settings their own per-scope granularity would blur that distinction and duplicate what
`drut.toml` already does better, not add real value for the motivating scenario (spec.md User
Story 1 — one personal preference, applied uniformly across every project without its own
`drut.toml`).

## §6: `formatting.rs`/`range_formatting.rs`/`diagnostics.rs`'s `config_warnings` stream — reach of the change

`formatting.rs::handle` and `range_formatting.rs::handle` both call `resolve_format_options`
directly with `ExplicitFormatOverride::default()` (confirmed: LSP requests carry no explicit
override today at all) — both gain the new `client_defaults` argument, sourced from
`ServerState`'s cache. `diagnostics.rs`'s `config_warnings` stream (the malformed-`drut.toml`
Hint) is unrelated — it reports on `drut.toml` parse problems specifically, not on the resolved
`FormatOptions` value, and needs no change.
