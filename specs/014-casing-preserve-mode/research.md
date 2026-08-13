# Phase 0 Research: Casing Gains an Explicit `Preserve` Mode

## §1 — Every place `voyager-core` reads or matches on `CasingConvention`/`FormatOptions.casing`

Traced directly against `crates/voyager-core/src/format.rs` (not assumed
from memory):

| Site | Today's shape | Required change |
|---|---|---|
| `CasingConvention` enum definition (`format.rs:44-48`) | Two variants (`Upper`, `Lower`), no `Default` impl | Add `Preserve` as `#[default]`, alongside `#[derive(..., Default)]` |
| `FormatOptions.casing` field (`format.rs:70`) | `Option<CasingConvention>` | Bare `CasingConvention` |
| `render()`'s casing-edit gate (`format.rs:200-203`) | `if let Some(convention) = options.casing { collect_casing_edits(..., convention, ...) }` | `if options.casing != CasingConvention::Preserve { collect_casing_edits(..., options.casing, ...) }` — the sole place in the whole crate that decides *whether* to run casing edits at all |
| `edit_for_span`'s match (`format.rs:582-585`) | `match convention { Upper => ..., Lower => ... }` — genuinely exhaustive today over two variants | **A second real call site, not just `render`'s guard.** Once `Preserve` exists, this match becomes non-exhaustive and is a compile error until a third arm is added. In practice this arm is unreachable — `render`'s guard means `collect_casing_edits`/`edit_for_span` are never invoked at all when `options.casing == Preserve` — but the compiler has no way to know that, so the arm still has to exist and do something sane (a no-op is the obvious choice, consistent with `edit_for_span`'s own "returns `None` for a no-op" contract). |
| Every function between `render`'s guard and `edit_for_span` (`collect_casing_edits`, `collect_block_casing_edits`, `collect_statement_casing_edits`, `push_if_present`) | All already take a bare `convention: CasingConvention` parameter, never `Option` | **No change** — these were already unwrapped once at `render`'s single gate; nothing downstream of that gate has ever been `Option`-typed. |

Consequence: this feature's entire `voyager-core` surface is exactly two
call sites (`render`'s guard, `edit_for_span`'s match) plus the two type
definitions — confirmed exhaustively, not assumed from the feature
description's own (correct, but not exhaustively-verified-at-spec-time)
summary.

## §2 — `FormatOptions.casing`/`CasingConvention` construction-site inventory across the workspace

Every place that builds a `FormatOptions` struct literal or otherwise
produces a `CasingConvention` value, confirmed via direct `grep`:

| Call site | Shape today | Required treatment |
|---|---|---|
| `drut_config::lib.rs:95` — `let casing = explicit.casing.or(config.format.casing);` | Yields `Option<CasingConvention>`, assigned straight into `FormatOptions.casing` | Add `.unwrap_or_default()`, matching line 96-99's already-existing `top_level_indent` pattern in the same function exactly |
| `drut_config::lib.rs:104-108` — `default_options`'s `casing: explicit.casing` | Same | Same: `.unwrap_or_default()` |
| `drut-cli/src/format_cmd.rs:91-94` — `ExplicitFormatOverride { casing: casing.map(CasingConvention::from), .. }` | Builds the *override* layer, not `FormatOptions` directly — stays `Option` | **No change** — `ExplicitFormatOverride.casing` remains `Option<CasingConvention>` per FR-004; only `impl From<CasingArg> for CasingConvention` (same file, lines 15-21) needs a new `CasingArg::Preserve => CasingConvention::Preserve` arm |
| `drut-mcp/src/format.rs:60-77` — `explicit_override`'s `casing` match | Builds the override layer, same as above | **No change to the Option-ness** — only the match itself gains a `Some("preserve") => Some(CasingConvention::Preserve)` arm |
| `drut-lsp/src/formatting.rs`, `src/range_formatting.rs` | Both call `resolve_format_options(..., ExplicitFormatOverride::default())` — never construct `FormatOptions` or `CasingConvention` directly | **No change** — confirmed by direct read of both files; the only way either could be affected is if `resolve_format_options`'s return type changed shape, and it hasn't (`FormatOptions` itself just has one field's type change, still one struct) |
| `voyager-core`'s own test module — `upper()`/`normalize()` builders (`format.rs:695-707`) and one inline literal (`format.rs:984`, the `casing_lower_...` test) | `FormatOptions { casing: Some(...) / None, .. }` | **Compiler-forced** — `casing: Some(CasingConvention::Upper)` → `casing: CasingConvention::Upper`; `casing: None` → `casing: CasingConvention::Preserve` |

Confirms the feature's own stated design symmetry holds exactly: only
`voyager-core`'s two definitions plus the two `resolve_format_options`
lines plus three string/enum-value match arms (TOML, CLI, MCP) need any
change at all. `drut-lsp` is untouched, exactly as spec.md's Assumptions
state.

## §3 — A pre-existing doc-comment/spec claim this feature makes stale

`002-cli-check-format/spec.md`'s **FR-026** (added by `009-top-level-indent-
toggle`, defining `--top-level-indent`) contains this sentence, contrasting
itself against `--casing`:

> Unlike FR-015's `--casing` flag, this setting has no "off" state —
> omitting the flag resolves to the explicit `preserve` default, not an
> unset/`None` value.

This was accurate when written: `top_level_indent`'s *resolved* value
(`voyager_core::FormatOptions.top_level_indent`) was already non-`Option`
(a real `TopLevelIndentMode` with `#[default]`), while `casing`'s resolved
value genuinely could be `None` — a real third state distinct from both
`Upper` and `Lower`. After this feature, that distinction no longer exists:
`FormatOptions.casing` becomes non-`Option` too, resolving to `Preserve` by
default exactly like `top_level_indent` resolves to `Preserve` — the two
settings now have the *same* shape at the resolved-value layer. FR-026's
contrast is therefore inaccurate the moment this feature ships, not merely
outdated in spirit. Folded into FR-011's amendment scope (spec.md) — the
FR-026 sentence itself needs correcting, not just a new FR-015 entry added
alongside it.

`voyager-core/src/format.rs`'s own doc comment on `CasingConvention`
(`format.rs:41-43`, "no hardcoded default; `FormatOptions.casing` being
`None` is how 'off' is represented, not a third variant here") and on
`FormatOptions.casing` (`format.rs:68-69`, "`None` (default) leaves all
keyword/control-word casing untouched") make the same now-superseded claim
directly in code — both covered by FR-010.

No other stale references to the old shape were found in
`README.md`/`CLAUDE.md`/`ROADMAP.md` (checked directly — none of the three
describe `CasingConvention`'s internal shape at this level of detail).

## §4 — Confirming zero golden-fixture/behavior change (FR-003, User Story 2)

`format_corpus.rs`'s golden fixtures and `format_sequence.rs`'s regression
tests were not re-inspected fixture-by-fixture (unlike `009`'s research.md
§3) because none of them can be affected *by construction*: every one calls
`FormatOptions::default()` or an explicit `FormatOptions { casing: None,
.. }`/`Some(Upper)`/`Some(Lower)` literal, and this feature's own FR-003
requirement is that `Preserve`'s formatter behavior is byte-identical to
today's `None` for every input — the render-time control flow
(`render`'s guard) is unchanged in what it *does*, only in how the
"don't touch casing" state is spelled (`options.casing != Preserve` is
true/false in exactly the same cases `options.casing.is_some()` was
true/false before, since `Preserve` is the only new state and it maps
exactly onto the old `None`). This is confirmed as a *result* in Phase 2
(the full existing suite passing with zero modification is the actual
proof), not asserted here as a substitute for running it.

## §5 — TOML/CLI/MCP explicit-`preserve` string-value shape: no new design decision

`top_level_indent` already established, in this exact codebase, the full
pattern this feature's FR-005/FR-006/FR-007 replicate for `casing`:

- **TOML** (`drut-config/src/parse.rs`'s `parse_top_level_indent`,
  lines 119-144): a `match value.as_str() { Some("preserve") => ..., Some
  ("normalize") => ..., Some(other) => <InvalidValue warning>, None =>
  <InvalidValue warning> }` shape. `parse_casing` (lines 92-117) is
  structurally identical, just two arms instead of three — adding a third
  `Some("preserve")` arm is copying an established pattern, not inventing
  one.
- **CLI** (`drut-cli/src/cli.rs`): `TopLevelIndentArg` (lines 88-92) is a
  three-line `ValueEnum` derive with `Preserve`/`Normalize` variants,
  already `Option`-wrapped on the `Format` subcommand
  (`Option<TopLevelIndentArg>`, changed from a `default_value_t` shape to
  `Option` by `012-toml-configuration` specifically so "flag omitted" and
  "flag explicitly says preserve" are distinguishable once a `drut.toml`
  layer exists to fall back to). `CasingArg` (lines 82-86) is already
  `Option`-wrapped the same way — it only needs the third variant added to
  its own `ValueEnum` derive, no change to how it's wrapped.
- **MCP** (`drut-mcp/src/format.rs`'s `explicit_override`, lines 60-77):
  `top_level_indent`'s match (lines 67-76) already has the exact
  three-arm-plus-error shape `casing`'s match (lines 61-66) needs to grow
  into.

No new design decision is required at any of these three surfaces — this
section exists to record that the shape was verified against the real,
current code (not assumed from `009`'s spec/plan, which predates
`012`'s later change of `top_level_indent` to `Option`-wrapped at the CLI
layer) before treating it as a safe pattern to replicate.
