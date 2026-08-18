# Phase 0 Research: Undefined `@token@` Diagnostic

Grounded directly in `crates/voyager-core/src/token_resolution.rs`, `crates/drut-lsp/src/
hover.rs`, and `crates/drut-lsp/src/diagnostics.rs` as they exist today — every finding below
was confirmed by reading the actual code before being treated as a planning fact.

## §1: The exact non-`DiagnosticKind` Hint-stream shape already exists, twice

`drut-lsp/src/diagnostics.rs::publish` builds three chained iterators today: `structural_
diagnostics` (the six/seven real `DiagnosticKind` values, `DiagnosticSeverity::ERROR`, source
`"drut"`), `fmt_marker_diagnostics` (`010-fmt-region-markers`'s unclosed `; FMT: OFF` marker,
`DiagnosticSeverity::HINT`, source `"drut-fmt"`, built from the standalone
`voyager_core::unclosed_fmt_off_markers` function — deliberately *not* a `Diagnostic`/
`DiagnosticKind`, per that feature's own spec.md Assumptions), and `config_warnings`
(`012-toml-configuration`'s malformed-`drut.toml` warning, same `HINT` severity, source
`"drut-config"`, built from `drut_config::parse::parse`'s own return value). Neither of the
latter two ever amended `001-voyager-script-parser`'s spec — confirmed by grepping that spec and
its contracts for any reference to either feature; there is none.

**Decision**: this feature is a fourth such stream, `undefined_token_diagnostics`, same shape —
a standalone function outside `Diagnostic`/`DiagnosticKind`, `HINT` severity, a new distinct
source (`"drut-token"`), chained alongside the existing three. No `voyager-core` core-type
change, no spec amendment needed anywhere.

## §2: `variable_ref_at` finds one reference at a position; this feature needs all of them

`token_resolution.rs::variable_ref_at(nodes, pos)` walks `collect_statements` (every real
`Statement`, any nesting depth) plus `collect_if_condition_token_slices` (`IfBranch.condition`
token slices, which aren't wrapped in a `Statement` at all), returning the first `VariableRef`
token whose span contains `pos`. This feature needs every `VariableRef` in the document, not
just the one under a cursor.

**Decision**: add `all_variable_refs(nodes: &[Node]) -> Vec<VariableRefAt>` to
`token_resolution.rs` — identical traversal to `variable_ref_at`'s two collection passes, just
collecting every match instead of stopping at the first one whose span contains a given
position. Mirrors `all_assignments`'s already-established "all instead of one" shape exactly —
not a new traversal pattern, a repeat of one already in this file.

## §3: Every one of FR-003's three "blind spot" exclusions is already free, not new logic

Checked each specific claim against the actual code rather than assumed:

- **Block-opener position** (`RUN PGM=@Prog@`): `token_resolution.rs`'s own module doc comment
  states `Block` (see `block.rs`) discards its opener statement's *value* tokens once matched,
  keeping only `opener_pairs`' keyword-name spans. A `VariableRef` sitting in a discarded value
  token is therefore **not present** in anything `collect_statements`/`collect_if_condition_
  token_slices` walk — `all_variable_refs` (§2) will never even see it. Nothing to detect or
  suppress; it's structurally absent from the input, the same reason `variable_ref_at` already
  can't find it for hover.
- **Multi-level `READ FILE` inclusion**: `hover.rs::collect_included_files`'s own doc comment
  states "one level only, never recursing into an included file's own `READ FILE` statements" —
  already true today, unconditionally, for every caller of that function.
- **Token-built (dynamic) `READ FILE` path**: `token_resolution.rs::read_file_refs` already sets
  `literal_value_span: None` whenever `contains_variable_ref(value)` is true; `collect_included_
  files` already `filter_map`s out any entry whose `literal_value_span` is `None` via `let
  value_span = read_ref.literal_value_span?;`. A dynamic path is already silently skipped by the
  existing function, unconditionally.

**Decision**: FR-003 requires zero new suppression logic. Reusing `all_variable_refs` (§2) for
enumeration and `collect_included_files`/`resolve_token_value` (§4) for resolution, *verbatim,
unchanged*, already satisfies all three exclusions by construction — the same category of
scope-reducing finding `017` made about the lexer needing no dot-boundary change.

## §4: `collect_included_files`/`IncludedFile` need only a visibility change, not a rewrite

`hover.rs::collect_included_files(uri, doc) -> Vec<IncludedFile>` already does exactly the disk
I/O this feature needs (reads every literal-path `READ FILE` target relative to the document's
own directory, parses it, gracefully omits any entry that fails at any step — no real on-disk
location, missing file, unparseable content). It and `struct IncludedFile` are both currently
private to `hover.rs` (no `pub(crate)`).

**Decision**: promote both to `pub(crate)`, reused as-is by the new diagnostic module — not
duplicated. `resolve_token_value(nodes, pos, included, name)` is already `pub` on
`token_resolution.rs` and needs no change at all; called once per `all_variable_refs` entry,
using that reference's own `span.start` as `pos` (the same "what's visible at this exact
position" semantic hover already uses for the cursor position).

## §5: No configuration surface needed

Neither `unclosed_fmt_off_markers` nor the `drut.toml`-malformed-value warning stream has an
on/off toggle — both are unconditional whenever the LSP publishes diagnostics at all. This
feature follows the same shape: no `drut.toml` field, no CLI flag, no MCP param (also directly
required by the "LSP-only" surface-reach decision — there is no CLI/MCP surface for this
capability to be configured through in the first place).
