# Contract: Casing Categories + Indent Width (addition)

Extends `001-voyager-script-parser/contracts/public-api.md` and
`002-cli-check-format/contracts/formatting-api.md`. A conceptual signature contract, not final
Rust source, but the shapes and guarantees below are binding — same convention every prior
contract doc in this repo follows.

## `voyager-core` additions

```text
pub struct CasingSettings {
    pub control_words: CasingConvention,
    pub pair_keywords: CasingConvention,
    pub data_references: CasingConvention,
}

pub struct FormatOptions {
    pub casing: CasingSettings,          // was: CasingConvention
    pub top_level_indent: TopLevelIndentMode,
    pub indent_width: u8,                // new
}

pub struct DataReferenceEntry { pub name: &'static str }
pub struct DataReferenceOccurrence { pub name: String, pub span: Span }

pub fn data_reference_entries() -> &'static [DataReferenceEntry]
pub fn data_reference_occurrences(nodes: &[Node]) -> Vec<DataReferenceOccurrence>
```

- **`CasingConvention` itself is unchanged** — still `Preserve`/`Upper`/`Lower`, `Preserve` the
  `#[default]`. This contract widens where it's applied (three fields instead of one), not
  what it can express.
- **`format`/`format_bytes` signatures are unchanged** — still `fn format(source: &str,
  options: FormatOptions) -> FormatResult`. Only `FormatOptions`'s own shape changes.
- **No panics, determinism, idempotency, behavior preservation**: every guarantee
  `002-cli-check-format/contracts/formatting-api.md` already makes for `format`/`format_bytes`
  holds unchanged, re-verified (not assumed) for both new axes specifically — data-references
  casing and non-default `indent_width` are new *inputs* to the same functions, not new
  functions with their own guarantees to define from scratch.
- **`data_reference_occurrences`**: pure, no I/O, never panics on any input including
  structurally broken `nodes` (same contract shape `token_resolution.rs`'s existing public
  functions already have). A returned `span` covers exactly the base-name portion of a match —
  never a `[...]` subscript, never text after a `.` — so a caller can safely rewrite only that
  span without corrupting a subscript index, a computed value, or an unrelated token.
- **One name, one casing value, regardless of shape** (FR-005): if `MW` matches in its
  pair-keyword-shaped form in one place and its assignment-target-shaped form in another, both
  occurrences carry `name: "MW"` and both are rewritten identically by whatever
  `options.casing.data_references` specifies — `data_reference_occurrences` itself has no
  concept of "shape" in its return type, by design, so no caller can accidentally apply
  different casing per shape.

## `drut-config` additions

```text
pub struct FormatConfig {
    pub casing: Option<CasingConvention>,                   // unchanged
    pub control_words_casing: Option<CasingConvention>,     // new
    pub pair_keywords_casing: Option<CasingConvention>,     // new
    pub data_references_casing: Option<CasingConvention>,   // new
    pub top_level_indent: Option<TopLevelIndentMode>,        // unchanged
    pub indent_width: Option<u8>,                            // new, unvalidated at this layer
    // ...
}
// ExplicitFormatOverride: identical new fields, same existing pattern as top_level_indent.

pub fn resolve_format_options(
    config: &FormatConfig,
    explicit: &ExplicitFormatOverride,
) -> FormatOptions
```

- Implements the full precedence matrix in `data-model.md` §3, including the
  legacy/granular same-tier arbitration for `control_words`/`pair_keywords` (granular wins
  when both are present at the same tier).
- `indent_width` resolution applies the 1–16 bound (`data-model.md` §4) at this layer, not
  inside `voyager-core` — an out-of-range value is treated exactly like a malformed TOML
  string value elsewhere in this crate: discarded, non-blocking notice surfaced, falls through
  to the next precedence tier.
- TOML parsing: `casing`, `control_words_casing`, `pair_keywords_casing`,
  `data_references_casing` each accept `"upper"`/`"lower"`/`"preserve"` (unchanged accepted
  values, just three more fields accepting them). `indent_width` accepts a TOML integer.

## `drut-cli` additions

- `--casing=<upper|lower|preserve>` — **unchanged**, still applies to `control_words` +
  `pair_keywords`.
- `--control-words-casing`, `--pair-keywords-casing`, `--data-references-casing` — new, same
  `ValueEnum` shape as `--casing`, each independently overriding one category.
- `--indent-width=<N>` — new, same "requires an explicit value, no bare flag" rule
  `002-cli-check-format` FR-015 already established for `--casing`.

## `drut-mcp` additions

- `casing` string parameter — **unchanged**, still applies to `control_words` + `pair_keywords`.
- `control_words_casing`, `pair_keywords_casing`, `data_references_casing` string parameters —
  new, same accepted-value shape as `casing`.
- `indent_width` integer parameter — new.

## What this contract does *not* promise (by design, this phase)

- No built-in `auto`/preset value at any surface (FR-003) — every new field accepts only
  `upper`/`lower`/`preserve`, same closed set `casing` already has.
- No split of `data_references` by structural shape (`ROADMAP.md` item 11, Bill's evidenced
  preference) — deliberately deferred, a purely additive follow-on if ever built.
- No `=`/operator spacing normalization (`ROADMAP.md` item 12) — unrelated axis, separate
  future feature.
- No completion/spell-check coverage for `data_references` tokens — `keywords.rs`'s dictionary
  gains only the `NUMREC`-family removal and `ZONES` addition (§5 of research.md); the
  `data_references` family list lives in its own new module, not `keywords.rs`, and isn't
  wired into `completion_candidates`/`did_you_mean` by this feature.
