# Data Model: Per-Category Casing Configuration and Configurable Indentation Width

## §1. `voyager-core` types

### `CasingSettings` (new)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CasingSettings {
    pub control_words: CasingConvention,
    pub pair_keywords: CasingConvention,
    pub data_references: CasingConvention,
}
```

- Each field independently defaults to `CasingConvention::Preserve` via that enum's own
  existing `#[default]` — `CasingSettings` needs no manual `Default` impl.
- `CasingConvention` itself is **unchanged** (`Preserve`/`Upper`/`Lower`, still the per-category
  value type) — this feature widens how many places it's applied, not what it can express.
- Replaces `FormatOptions.casing`'s current type (`CasingConvention` directly) with
  `CasingSettings` — the same kind of field-type widening `014` already did once
  (`Option<CasingConvention>` → `CasingConvention`), same acceptance criterion: every call
  site is a compile error until updated, not a silent behavior change.

### `FormatOptions` (modified)

```rust
#[derive(Debug, Clone, Copy)]           // Default is now a manual impl, not derived
pub struct FormatOptions {
    pub casing: CasingSettings,
    pub top_level_indent: TopLevelIndentMode,
    pub indent_width: u8,               // new; valid range enforced by drut-config, not here
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            casing: CasingSettings::default(),
            top_level_indent: TopLevelIndentMode::default(),
            indent_width: 4,
        }
    }
}
```

- `indent_width` is intentionally a bare `u8` here, not a bounded/validated type —
  `voyager-core` accepts whatever its caller passes (consistent with this crate never doing
  I/O or policy enforcement; the 1–16 bound is `drut-config`'s job, §3 below).

### `data_reference` module (new)

```rust
/// One recognized data-reference family member (research.md §6).
pub struct DataReferenceEntry {
    pub name: &'static str,   // canonical uppercase spelling, e.g. "MI", "ZONES"
}

/// Every recognized name, case-insensitive match (research.md §6's table).
pub fn data_reference_entries() -> &'static [DataReferenceEntry];

/// Finds every occurrence of a recognized data-reference token in `nodes`,
/// across all three structural shapes (dot-notation read, pair-keyword name,
/// assignment target), returning just the span of the *base name* portion —
/// never including a `[...]` subscript or the text after a `.` — so a caller
/// can rewrite exactly that span's casing without touching anything else.
pub fn data_reference_occurrences(nodes: &[Node]) -> Vec<DataReferenceOccurrence>;

pub struct DataReferenceOccurrence {
    pub name: String,     // the matched entry's canonical name
    pub span: Span,       // exactly the base-name portion
}
```

- Pure functions over already-parsed `Node`/`Token` data, no I/O, no panics on any input —
  same contract shape every other `voyager-core` public function already has.
- `format.rs`'s existing casing-edit collection calls `data_reference_occurrences` alongside
  its existing control-word/pair-keyword collection, applying `options.casing.data_references`
  to each occurrence's span — the same `edit_for_span`-style rewrite the other two categories
  already use, just fed a third source of spans.

## §2. `data_references` recognized-name table

Full family/shape table already in research.md §6 — not duplicated here. Binding fact this
data model depends on: **one canonical name maps to one casing value, applied identically
regardless of which of the three structural shapes a given occurrence takes** (FR-005) — there
is no per-shape variant of `DataReferenceEntry`.

## §3. Configuration precedence matrix

Applies independently per category. "Legacy" = the already-shipped flat `casing`
field/flag/param; "granular" = this feature's three new per-category fields/flags/params.

| Category | Precedence (highest wins) |
|---|---|
| `control_words` | explicit granular flag/param → explicit legacy `--casing`/`casing` → `drut.toml` granular field → `drut.toml` legacy `casing` field → `Preserve` |
| `pair_keywords` | *same as `control_words`* — legacy `casing` has always covered both, so it stays a shared fallback for both, not duplicated per-category logic |
| `data_references` | explicit granular flag/param → `drut.toml` granular field → `Preserve` (legacy `casing` never applies — it structurally never reached this category, so it isn't inserted into this category's fallback chain at all) |
| `indent_width` | explicit `--indent-width`/param → `drut.toml`'s `indent_width` field (validated 1–16, else discarded) → built-in default `4` |

This is `resolve_format_options`'s existing `defaults < drut.toml < explicit` precedence
(unchanged in spirit), applied per-field rather than per-struct, with one added
wrinkle specific to this feature: two config layers (legacy vs. granular) can both be present
at the same precedence tier for `control_words`/`pair_keywords`, resolved by granular-wins
within that tier — this is new, `top_level_indent`/`014`'s casing work never had two
same-tier sources to arbitrate between.

### `drut_config::FormatConfig` / `ExplicitFormatOverride` (modified)

```rust
pub struct FormatConfig {
    pub casing: Option<CasingConvention>,                  // unchanged, legacy
    pub control_words_casing: Option<CasingConvention>,    // new
    pub pair_keywords_casing: Option<CasingConvention>,    // new
    pub data_references_casing: Option<CasingConvention>,  // new
    pub top_level_indent: Option<TopLevelIndentMode>,      // unchanged
    pub indent_width: Option<u8>,                           // new, pre-bound-validation
    // ...
}
// ExplicitFormatOverride mirrors the same shape (same pattern top_level_indent
// already established between these two structs).
```

## §4. `indent_width` validation rule

- Valid range: **1–16 inclusive** (`ROADMAP.md` item 9's carried-forward recommendation).
- A TOML/CLI/MCP value outside that range (including `0`, negative — TOML permits negative
  integers syntactically even though the field is conceptually unsigned — and anything above
  16) is treated exactly like every other malformed `[format]` value in this project: discarded,
  a non-blocking notice is surfaced (CLI stderr / MCP response field / LSP hint diagnostic, the
  same three-surface shape `010-fmt-region-markers`'s unclosed-marker notice already
  established), and resolution falls through to the next precedence tier.
- Enforced once, at `drut-config`'s resolve layer — `voyager-core::FormatOptions.indent_width`
  itself has no validation (§1); a directly-constructed `FormatOptions { indent_width: 0, .. }`
  is a caller error, not something this crate refuses to run (consistent with `voyager-core`
  never refusing to compute a result, per its existing panic-free/no-refusal contract).

## §5. `keywords.rs` table changes

- Removed from `PAIR_KEYWORDS`: `NUMREC`, `CNT`, `ITER`, `LP`, `RECNUM` (all previously
  `observed_with: ["LOOP"]`).
- Added to `PAIR_KEYWORDS`: `ZONES`, `observed_with: ["RUN"]` (its `RUN`/`RUN PGM=MATRIX
  ZONES=...` pair-keyword shape only — see research.md §5 for why its plain-assignment shape
  is `data_reference.rs`'s concern instead).
- `CONTROL_WORDS` unaffected.
