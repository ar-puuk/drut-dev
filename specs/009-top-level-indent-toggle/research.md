# Phase 0 Research: Top-Level Indent Default Revert

## §1 — Does `plan_block`/`plan_children` need any change?

No. Traced the full call chain in `crates/voyager-core/src/format.rs`:

- `plan_indentation` (the top-level entry point, iterating `ParseResult.nodes`)
  is the *only* place that ever force-inserts column 0 for a line — the
  single line `008` added: `plan.insert(node.span().start.line, 0)`, run
  once per top-level node before recursing into `plan_block` for a
  top-level `Block`.
- `plan_block`'s `base` is `computed_indent(plan, lines, opener_line)` —
  which reads whatever was already inserted into `plan` for that line, and
  **falls back to the line's actual on-disk indentation
  (`original_indent_width`) if nothing was inserted**. This fallback
  already existed before `008` and is exactly what `008`'s own plan.md
  observed makes that feature's one-line change sufficient.
- `plan_block`/`plan_children` are called identically for a top-level
  block and a nested block — neither knows or cares whether the node
  they're planning is top-level. All top-level-specific behavior lives
  entirely in `plan_indentation`'s own loop, nowhere else.

Consequence: making `plan_indentation`'s single insert conditional on
`mode == Normalize` is suffient by itself. Under `Preserve`, no plan entry
is ever inserted for a top-level line, so `computed_indent` falls back to
that line's real on-disk column when `plan_block` computes `base` for a
top-level block — reproducing `007`-era behavior exactly, with zero
changes to `plan_block`/`plan_children`/closer-alignment/branch-alignment.

`007`'s diagnosed-block-children skip (`diagnosed_openers.contains(&block
.span.start) { return; }`) is untouched code and needs no re-evaluation
this time (unlike `008`, which had to re-derive its own rationale) —
under `Preserve` it behaves exactly as it did pre-`008` (protects a
diagnosed top-level block's children, opener also untouched since nothing
forces it); under `Normalize` it behaves exactly as `008` already
verified (protects children only, opener forced independently). Nothing
about this feature changes what the skip protects or why.

## §2 — `FormatOptions` call-site inventory (FR-004)

Every construction site in the workspace, and what each needs:

| Call site | Shape | Risk class | Required treatment |
|---|---|---|---|
| `voyager-core/src/format.rs` — `FormatOptions` struct definition | N/A | N/A | Add field with `#[default]`-derived `TopLevelIndentMode::Preserve`. |
| `drut-cli/src/format_cmd.rs:61` — `FormatOptions { casing: ... }` | Full struct literal, no `..Default::default()` | **Compiler-forced** — will not build until this field is set | Set explicitly from the new `--top-level-indent` flag's parsed value (the one call site that legitimately *should* pass a non-default value). |
| `drut-mcp/src/format.rs:42` — `FormatOptions { casing: convention }` | Full struct literal, no `..Default::default()` | **Compiler-forced** | Set explicitly to `TopLevelIndentMode::default()` — no MCP-side toggle in scope (spec Assumptions), but the value must still be written, not spread from `Default`, so the choice is visible in the diff rather than implicit. |
| `drut-lsp/src/formatting.rs:35` — `voyager_core::FormatOptions::default()` | `::default()` call | **Not compiler-forced** — silently resolves to whatever the derived default is | No code change. New dedicated test proving the resolved default is `Preserve` (User Story 3, Acceptance Scenario 2). |
| `drut-lsp/src/range_formatting.rs:106` — same | `::default()` call | **Not compiler-forced** | Same: no code change, new dedicated test (Acceptance Scenario 2, the range-formatting path specifically, since it's a separate handler from whole-document formatting). |
| `voyager-core/src/format.rs`'s own test module — `upper()` helper (`FormatOptions { casing: Some(...) }`) and one inline literal (casing-lower test) | Full struct literal | **Compiler-forced** | Both are casing-only tests with top-level content already at column 0 in their fixtures (§3) — mode-independent either way; set the new field to `TopLevelIndentMode::default()` explicitly (matches real production default, keeps the test's intent — "casing behavior" — undiluted by an unrelated field). |
| Every other `FormatOptions::default()` call site (all of `voyager-core`'s own tests, `format_sequence.rs`, `format_corpus.rs`, `drut-mcp`'s own test module) | `::default()` call | Not compiler-forced, but these are tests, audited individually in §3 | No change needed for tests whose fixture is already mode-independent (§3); explicit `Normalize` for tests that specifically assert `008`-era top-level forcing (§3). |

Confirms FR-004's three-part check is exhaustive: (a) the CLI flag's own
`clap` default, (b) `FormatOptions::default()`'s derived value, (c) every
non-`::default()`-based construction (`drut-cli`, `drut-mcp`) — nothing
else constructs a `FormatOptions` anywhere in the workspace.

## §3 — Existing test audit: which tests assume `008`-era default behavior?

Every `voyager-core` test using `FormatOptions::default()` was individually
checked against its own fixture text — many already place their top-level
content at column 0 in the source itself, which makes them **mode-
independent** (identical output whether `Preserve` or `Normalize`, since
there's no non-zero top-level indentation for either mode to act on or
leave alone). Only tests whose fixture *starts* with genuine non-zero
top-level indentation, and whose assertion specifically checks that it
gets corrected, actually exercise the `008`-era default and need
retargeting.

**`crates/voyager-core/src/format.rs`'s own test module** — 3 of ~34 tests
need retargeting to explicit `TopLevelIndentMode::Normalize` (their
fixtures have non-zero top-level indentation and their assertions require
column-0 correction):
- `top_level_baseline_is_always_normalized_to_zero`
- `bare_top_level_statement_is_normalized_to_zero`
- `diagnosed_block_opener_is_normalized_but_children_stay_untouched`

Each gets a new `Preserve`-mode sibling test (reviving the pre-`008`
assertion the first test's own comment says it replaced —
`top_level_baseline_is_left_untouched` — plus two new siblings for the
other two, all three proving the corresponding non-zero top-level
indentation is left completely untouched under the new default).

`top_level_block_with_stale_children_corrects_both_together` and
`already_column_zero_top_level_is_idempotent` both use fixtures whose
top-level opener is *already* at column 0 — mode-independent, confirmed
by inspection, no change needed (only the block's *children* get
corrected, which is identical under both modes per §1).

`behavior_preservation_reparses_to_the_same_structure` has a 2-space
top-level indent in its fixture, but only asserts structural/diagnostic
round-trip equivalence, not exact formatted text — passes unchanged under
either mode (leading whitespace is not structurally significant to the
parser).

Every remaining test (casing tests, `format_bytes`/`EncodingFidelity`
tests, CRLF/idempotency/continuation/comment tests, etc.) has its
top-level content already at column 0 in the fixture text itself —
inspected individually, none need retargeting.

**`crates/voyager-core/tests/format_sequence.rs`** — all 5 tests
retargeted to explicit `Normalize`. Every one of them specifically exists
to prove the `008`-era PROCESS/RUN residue guarantee (a top-level `RUN`
block left at stale non-zero indentation gets corrected to column 0) —
under `Preserve`, none of these fixtures would change at all (by
`Preserve`'s own definition), so leaving them on the default would either
fail outright or silently stop testing what they were written to prove.
This is exactly FR-006's requirement.

**`crates/voyager-core/tests/format_corpus.rs`** — the three shared
helpers (`check_golden`, `check_idempotent`,
`check_structure_and_diagnostics_preserved`) all hardcode
`FormatOptions::default()` internally. Parameterizing each with an
explicit `FormatOptions` argument (call sites updated to pass it) is the
smallest change that supports both: the existing `#[test]` functions keep
calling with `FormatOptions::default()` (now `Preserve`, driving golden
regeneration per FR-005), and new `#[test]` functions call with explicit
`Normalize` against a **new, separate fixture directory**
(`tests/fixtures/golden_normalize/`) populated by copying `008`'s
already-committed, already-human-reviewed golden output verbatim before
any regeneration touches `tests/fixtures/golden/` — proving `Normalize`
mode is byte-identical to `008`'s shipped behavior (FR-006/SC-002)
without needing a second human-review pass (the content is unchanged from
what was already reviewed under `008`).

**`drut-mcp`'s own test module** (`format.rs`'s 3 tests) — all use
top-level-at-column-0 fixtures (`"IF (a=b)\n..."`), mode-independent, no
change needed for existing tests. A new dedicated test is added (FR-004(c)
for this specific adapter) with a genuinely non-zero top-level fixture,
confirming the MCP `format` tool's default output is `Preserve`.

## §4 — CLI flag shape: `--casing`'s vs. `OutputFormat`'s

Two existing `clap` patterns in `crates/drut-cli/src/cli.rs` were compared:

- `--casing`: `#[arg(long, value_enum)] casing: Option<CasingArg>` — no
  `default_value_t`, `None` is a real, meaningful third state ("casing
  normalization off entirely").
- `format`'s own output-format flag: `#[arg(long, value_enum,
  default_value_t = OutputFormat::Text)] format: OutputFormat` — always
  has a value; omitting the flag resolves to an explicit default, not an
  "off" state.

`--top-level-indent` matches the second shape, not the first: per spec
FR-002, this is a genuinely two-valued setting with no "off" state —
`format` always either preserves or normalizes top-level indentation,
there's no third "don't decide" behavior distinct from `preserve`. Using
`Option<TopLevelIndentArg>` would introduce a meaningless third state
(`None`) that would need its own resolution logic identical to
`Some(TopLevelIndentArg::Preserve)`, adding complexity `--casing`'s own
shape doesn't need because `--casing` really does have three meaningfully
different states. `default_value_t = TopLevelIndentArg::Preserve` is the
correct, simpler precedent to follow.
