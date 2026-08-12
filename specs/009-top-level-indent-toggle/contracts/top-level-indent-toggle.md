# Contract: Top-Level Indent Default Revert

Amends `002-cli-check-format/spec.md`'s FR-012 and `contracts/
formatting-api.md` a second time in place (matching how `008` itself
amended `007`'s version) — not a new, competing contract file. Also adds
a new FR (mirroring FR-015's `--casing` FR) for the CLI flag itself.

## `spec.md` FR-012 bullet — amended a second time

The existing bullet (already carrying `008`'s dated amendment) gets a
second dated entry appended, preserving both prior entries:

> **Top-level (depth-0) statement indentation defaults to left untouched
> (`preserve`)**, on every format pass, unless `--top-level-indent=
> normalize` is explicitly requested, in which case it is always
> normalized to column 0, unconditionally — regardless of the statement's
> current indentation or formatting history. **Amended 2026-08-11
> (`008-top-level-indentation-normalization`)**: [008's existing text,
> unchanged] ... **Amended again 2026-08-11
> (`009-top-level-indent-toggle`)**: this reverts `008`'s *default* back
> to the original `007`-era `preserve` behavior — `008`'s corpus-evidence
> framing (only 20.4% of real top-level statements at column 0, modal
> value at column 8) was never in question; the project has simply
> decided predictability-by-default was the wrong trade for users who
> never asked for it. `008`'s `normalize` behavior is fully retained,
> unchanged, as an explicit opt-in (`--top-level-indent=normalize`) for
> users who do want it.

## New FR (mirrors FR-015)

> **FR-026**: `format` MUST support a `--top-level-indent` flag accepting
> `preserve` (default) or `normalize`, selecting between the two FR-012
> top-level indentation behaviors. Unlike FR-015's `--casing` flag, this
> setting has no "off" state — omitting the flag resolves to the explicit
> `preserve` default, not an unset/`None` value (research.md §4).

## `contracts/formatting-api.md` — amended

**Before** (post-`008`): `"... top-level baseline always normalized to
column 0, continuation-line indentation left untouched ..."`

**After**: `"... top-level baseline defaults to left untouched
(preserve), or always normalized to column 0 when --top-level-indent=
normalize is explicitly requested, continuation-line indentation left
untouched ..."`

## Algorithm (normative, research.md §1)

```text
plan_indentation(nodes, lines, diagnosed_openers, mode, plan):
  for each top-level node in nodes:
    if mode == Normalize:
      plan[node's own line] = 0        # unchanged from 008, now conditional
    if node is a Block:
      plan_block(node, lines, diagnosed_openers, plan)   # unchanged
```

`plan_block`, `plan_children`, `computed_indent` are **not modified** —
same as `008`'s own contract already established; `computed_indent`'s
"prefer a planned value over the original" fallback is what makes
`Preserve` mode work for free once the seed is skipped.

## `007`'s skip — unaffected, no re-evaluation needed this time

Unlike `008` (which had to re-derive `007`'s skip's rationale because the
opener-line protection it used to provide became redundant),
`diagnosed_block_openers`/`plan_block`'s skip needs **no rationale
change** here: under `Preserve`, the skip's behavior is identical to
pre-`008` (protects a diagnosed top-level block's children; the opener is
also untouched, but because nothing forces it under `Preserve`, not
because the skip itself protects it — same non-overlapping-responsibility
split `008` already established). Under `Normalize`, behavior is
identical to `008`'s already-verified behavior. No code or doc-comment
change to `diagnosed_block_openers`/`plan_block` is needed beyond
`plan_indentation`'s own conditional (research.md §1).

## `FormatOptions` call-site treatment (research.md §2, normative)

| Call site | Required change |
|---|---|
| `drut-cli/src/format_cmd.rs` | Explicit, from the new flag's parsed `TopLevelIndentArg` value. |
| `drut-mcp/src/format.rs` | Explicit `TopLevelIndentMode::default()` — no MCP-facing toggle. |
| `drut-lsp/src/formatting.rs`, `range_formatting.rs` | No code change; new dedicated test per file proving the resolved default is `Preserve`. |
| Everywhere else (`FormatOptions::default()`) | No change required by this contract — inherits `Preserve` automatically; tests audited per research.md §3. |
