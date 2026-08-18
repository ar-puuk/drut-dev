# Configuration Reference

Every `[format]` field Drut currently understands, in one place. All of them are
set the same way, in a `drut.toml` file at (or above) the file you're formatting:

```toml
[format]
casing = "lower"
indent_width = 2
```

Drut discovers the nearest `drut.toml` by walking up from the file being
processed, stopping at the first `drut.toml` found, a `.git` boundary, or the
filesystem root — whichever comes first. A project with no `drut.toml` anywhere
behaves exactly like a project with an empty one: every field uses its built-in
default. Every field is optional; omitting a key is identical to writing its
default value explicitly.

**A malformed value never blocks formatting.** An unrecognized key or an
out-of-range value only affects that one field — it warns (CLI stderr, an LSP
Hint diagnostic, or the MCP `format` tool's `config_warnings` field) and falls
back to that field's built-in default. Every other valid setting in the same file
still applies.

## Precedence

Every field below resolves the same four-tier way, checked in this order — the
first tier that sets a value wins:

1. **An explicit CLI flag or MCP tool parameter**, passed for one specific
   invocation — always wins when given.
2. **`drut.toml`** — the nearest one found by the discovery walk above.
3. **An editor client setting** — for VS Code, one of the `drut.format.*`
   settings under Settings; delivered to `drut-lsp` via the standard LSP
   `workspace/configuration` mechanism. A personal editor preference never
   overrides a project's own committed `drut.toml` — it only fills in a field
   `drut.toml` leaves unset.
4. **The built-in default** — used only if none of the above set the field.

Each tier only fills in a field the tier(s) before it left unset; a field is
never assembled from pieces at different tiers.

Four fields (`casing`, `control_words_casing`, `pair_keywords_casing`,
`data_references_casing`) have one extra wrinkle on top of the four tiers above —
see [`casing`](#casing)'s entry for the full explanation.

## Fields

### `casing`

The legacy, all-in-one keyword-casing setting. Cases every `control_words` and
`pair_keywords` token together — it cannot reach `data_references` tokens at all
(use [`data_references_casing`](#data_references_casing) for those).

**Values**:
- `preserve` — leave existing casing exactly as written.
- `upper` — uppercase every control word and pair-keyword name.
- `lower` — lowercase every control word and pair-keyword name.

**Default**: `preserve`.

**Also known as**: CLI flag `--casing`; MCP `format` tool parameter `casing`.

**Example**:

```toml
[format]
casing = "upper"
```

**Precedence**: follows the [four-tier chain](#precedence) above, tier by tier —
but *within* each tier, drut checks that tier's granular field
(`control_words_casing`/`pair_keywords_casing`) first, and only falls back to
that same tier's `casing` value if the granular field is unset there. Concretely,
for `control_words_casing`: explicit `--control-words-casing`, then explicit
`--casing`, then `drut.toml`'s `control_words_casing`, then `drut.toml`'s
`casing`, then the editor setting's granular field, then the editor setting's
`casing`, then the built-in default — in that exact order. A `casing` value set
at a *higher*-precedence tier (say, an explicit `--casing` flag) still wins over
a granular field set only at a *lower* tier (say, `control_words_casing` in
`drut.toml`) — tier order is checked before either field's own fallback within a
tier. If you only ever set `casing`, both `control_words` and `pair_keywords`
follow it exactly as before `control_words_casing`/`pair_keywords_casing`
existed.

### `control_words_casing`

Independent override for the `control_words` category alone (things like `IF`,
`ENDIF`, `LOOP`, `ENDLOOP`) — wins over `casing` for this category specifically,
per the two-step fallback explained in [`casing`](#casing)'s entry above.

**Values**: `preserve` | `upper` | `lower`.

**Default**: `preserve` (falls back to `casing` first if `casing` is set and this
field isn't — see [`casing`](#casing)).

**Also known as**: CLI flag `--control-words-casing`; MCP `format` tool parameter
`control_words_casing`.

**Example**:

```toml
[format]
casing = "upper"               # control_words + pair_keywords default to upper...
control_words_casing = "lower" # ...except control_words specifically, forced lower
```

**Precedence**: see [`casing`](#casing) — same four-tier chain, with the
two-step legacy/granular fallback applied at every tier.

### `pair_keywords_casing`

Independent override for the `pair_keywords` category alone (keyword names inside
a `Control` statement's `keyword=value` pairs, e.g. `PATHLOAD`, `MATI`) — wins
over `casing` for this category specifically, same shape as
[`control_words_casing`](#control_words_casing) above.

**Values**: `preserve` | `upper` | `lower`.

**Default**: `preserve` (falls back to `casing` first — see [`casing`](#casing)).

**Also known as**: CLI flag `--pair-keywords-casing`; MCP `format` tool parameter
`pair_keywords_casing`.

**Example**:

```toml
[format]
pair_keywords_casing = "lower"
```

**Precedence**: see [`casing`](#casing) — same four-tier chain, with the
two-step legacy/granular fallback applied at every tier.

### `data_references_casing`

Casing for the data-reference category: Matrix/Line/Node/Zone/Database
abbreviations (`MI`/`MO`/`MW`, `LI`/`LW`, `NI`/`NW`, `ZI`/`ZONES`/`Z`,
`DBI`/`DBA`), `RO`, the link-endpoint fields `A`/`B`, and the reserved loop-index
identifiers `I`/`J`. **Not reachable by `casing` at all** — this is the only way
to case this category from configuration.

**Values**: `preserve` | `upper` | `lower`.

**Default**: `preserve`.

**Also known as**: CLI flag `--data-references-casing`; MCP `format` tool
parameter `data_references_casing`.

**Example**:

```toml
[format]
data_references_casing = "lower"
```

**Precedence**: follows the plain [four-tier chain](#precedence) — no
legacy-field fallback, since `casing` never reaches this category.

### `top_level_indent`

Whether top-level (depth-0, not inside any block) statement indentation is left
exactly as written, or normalized to column 0.

**Values**:
- `preserve` — leave top-level indentation exactly as written.
- `normalize` — force every top-level line to column 0.

**Default**: `preserve`.

**Also known as**: CLI flag `--top-level-indent`; MCP `format` tool parameter
`top_level_indent`.

**Example**:

```toml
[format]
top_level_indent = "normalize"
```

**Precedence**: follows the [four-tier chain](#precedence) above; no
legacy-field fallback.

### `indent_width`

Spaces per nesting level of block indentation, relative to the enclosing block's
own opening-statement column.

**Values**: any integer from `1` to `16`.

**Default**: `4`.

**Also known as**: CLI flag `--indent-width`; MCP `format` tool parameter
`indent_width`.

**Example**:

```toml
[format]
indent_width = 2
```

**Precedence**: follows the [four-tier chain](#precedence) above; no
legacy-field fallback. An out-of-range value (`0`, `500`, ...) at any tier is
treated as unset for that tier — resolution falls through to the next tier
exactly as if the field had been omitted there.

### `operator_spacing`

Whitespace normalization around `=`, comparison operators (`==`, `<>`, `>=`,
`<=`, `<`, `>`), binary arithmetic (`+`, `-`, `*`, `/`), comma spacing between
multiple `keyword=value` pairs, and interior padding inside `[...]`/`(...)`.

**Values**:
- `preserve` — leave existing spacing exactly as written.
- `fixed` — normalize every occurrence to exactly one space on each side (and
  zero interior padding inside brackets/parens), independent of neighboring
  lines.
- `auto` — everything `fixed` does, plus vertically aligns the `=` of
  consecutive `Assignment` statements at the same nesting depth to the column of
  the longest left-hand side in the run. A run resets at a blank line, a
  comment-only line, a nesting-depth change, or a non-`Assignment` statement.

**Default**: `preserve`.

**Also known as**: CLI flag `--operator-spacing`; MCP `format` tool parameter
`operator_spacing`.

**Example**:

```toml
[format]
operator_spacing = "auto"
```

See the [Formatter Guide](formatter-guide.md#operator-spacing) for full
before/after examples of `fixed` vs. `auto`.

**Precedence**: follows the [four-tier chain](#precedence) above; no
legacy-field fallback.

### `blank_lines`

Whether runs of consecutive blank lines (including whitespace-only lines) are
left as written or contracted down to a configured cap.

**Values**:
- `preserve` — leave every blank-line run exactly as written, however long.
- `auto` — contract a run down to the applicable cap ([`top_level_blank_line_cap`](#top_level_blank_line_cap)
  or [`nested_blank_line_cap`](#nested_blank_line_cap)) only when the run
  exceeds that cap — never pads a shorter run up.

**Default**: `preserve`.

**Also known as**: CLI flag `--blank-lines`; MCP `format` tool parameter
`blank_lines`.

**Example**:

```toml
[format]
blank_lines = "auto"
```

**Precedence**: follows the [four-tier chain](#precedence) above; no
legacy-field fallback.

### `top_level_blank_line_cap`

The maximum number of consecutive blank lines `blank_lines = "auto"` allows
between top-level statements/blocks before contracting the run. Only meaningful
when [`blank_lines`](#blank_lines) is `"auto"`.

**Values**: any integer from `1` to `50`.

**Default**: `2`.

**Also known as**: CLI flag `--top-level-blank-line-cap`; MCP `format` tool
parameter `top_level_blank_line_cap`.

**Example**:

```toml
[format]
blank_lines = "auto"
top_level_blank_line_cap = 1
```

**Precedence**: follows the [four-tier chain](#precedence) above; no
legacy-field fallback. An out-of-range value at any tier is treated as unset for
that tier, same as [`indent_width`](#indent_width).

### `nested_blank_line_cap`

The maximum number of consecutive blank lines `blank_lines = "auto"` allows
inside any block's own body, uniformly regardless of nesting depth, before
contracting the run. Only meaningful when [`blank_lines`](#blank_lines) is
`"auto"`.

**Values**: any integer from `1` to `50`.

**Default**: `1`.

**Also known as**: CLI flag `--nested-blank-line-cap`; MCP `format` tool
parameter `nested_blank_line_cap`.

**Example**:

```toml
[format]
blank_lines = "auto"
nested_blank_line_cap = 2
```

**Precedence**: follows the [four-tier chain](#precedence) above; no
legacy-field fallback. Same out-of-range handling as
[`top_level_blank_line_cap`](#top_level_blank_line_cap).
