# Phase 1 Data Model: Published Documentation Site

Not a software data model (no runtime entities/persistence) — this documents the
*content* model the site's chapters and the configuration-reference chapter
specifically are built from, verified directly against current source
(`crates/drut-config/src/lib.rs`, `crates/drut-cli/src/cli.rs`,
`crates/drut-mcp/src/format.rs`, `crates/voyager-core/src/format.rs`) as of
`021-editor-settings-config`.

## Entity: Documentation Site

The published, navigable collection of mdBook chapters at a stable public URL
(research.md §4). One instance; no per-version variants (spec.md Assumptions — no
versioned docs).

**Chapters** (`docs-site/src/SUMMARY.md` order — see contracts/site-structure.md
for the required-content contract per chapter):

1. Introduction
2. Install
3. Getting Started
4. CLI Reference
5. Editor (LSP) Guide
6. MCP Guide
7. Formatter Guide
8. Configuration Reference

## Entity: Configuration Field Entry

One documented `[format]` field. Every field below MUST have a complete entry per
contracts/config-reference-entry.md's contract (FR-003–FR-006). Table verified
directly against source, not transcribed from `CHANGELOG.md`/`ROADMAP.md` prose:

| `drut.toml` key | CLI flag | MCP param | Values | Default | Legacy/granular relationship |
|---|---|---|---|---|---|
| `casing` | `--casing` | `casing` | `preserve` \| `upper` \| `lower` | `preserve` | Legacy flat setting — covers `control_words` + `pair_keywords` together; superseded-but-still-supported by the three granular fields below. |
| `control_words_casing` | `--control-words-casing` | `control_words_casing` | `preserve` \| `upper` \| `lower` | `preserve` | Granular override for the `control_words` category; wins over `casing` for this category specifically when both are set at the same tier. |
| `pair_keywords_casing` | `--pair-keywords-casing` | `pair_keywords_casing` | `preserve` \| `upper` \| `lower` | `preserve` | Granular override for the `pair_keywords` category; wins over `casing` the same way. |
| `data_references_casing` | `--data-references-casing` | `data_references_casing` | `preserve` \| `upper` \| `lower` | `preserve` | Not reachable via the legacy `casing` field at all — the only way to case this category (Matrix/Line/Node/Zone/Database abbreviations, `RO`, link endpoints `A`/`B`, loop indices `I`/`J`). |
| `top_level_indent` | `--top-level-indent` | `top_level_indent` | `preserve` \| `normalize` | `preserve` | N/A |
| `indent_width` | `--indent-width` | `indent_width` | integer `1`–`16` | `4` | N/A |
| `operator_spacing` | `--operator-spacing` | `operator_spacing` | `preserve` \| `fixed` \| `auto` | `preserve` | N/A |
| `blank_lines` | `--blank-lines` | `blank_lines` | `preserve` \| `auto` | `preserve` | N/A |
| `top_level_blank_line_cap` | `--top-level-blank-line-cap` | `top_level_blank_line_cap` | integer `1`–`50` | `2` | Only meaningful when `blank_lines = auto`. |
| `nested_blank_line_cap` | `--nested-blank-line-cap` | `nested_blank_line_cap` | integer `1`–`50` | `1` | Only meaningful when `blank_lines = auto`. |

**Validation rule shared by every field**: an out-of-range or unrecognized value in
`drut.toml` or an editor client setting never blocks formatting — it warns (CLI
stderr / LSP Hint diagnostic, source `drut-config` / MCP `config_warnings` field)
and falls back to the built-in default for just that one field.

## Entity: Precedence Chain

One shared explanation (FR-005), a single ordered list every configuration field
entry points back to rather than restating:

1. Explicit CLI flag / MCP tool parameter (wins always, for that one invocation).
2. `drut.toml` (nearest one found by upward directory walk from the file being
   processed, stopping at a `.git` boundary or filesystem root).
3. Editor client setting (`021-editor-settings-config`; VS Code's `drut.format.*`
   settings, pulled via the standard LSP `workspace/configuration` mechanism).
4. Built-in default (the "Default" column above).

Each tier only fills in a field the tier(s) before it left unset — never merges
partial values within one field.

## Entity: Precedence source note for legacy/granular casing fields

`casing` and its three granular counterparts each get this same two-step fallback
*at every tier* — e.g. at the `drut.toml` tier: a field's own granular key, then
`casing`, before falling through to the next tier's granular key. This is the one
piece of the precedence model that isn't a flat 4-tier list per field and needs its
own explicit callout in the configuration-reference chapter (FR-006) — this is
precisely the CHK001/CHK002 gap `021-editor-settings-config` caught during its own
checklist review, and the entry the reader is least likely to get right by
intuition alone.
