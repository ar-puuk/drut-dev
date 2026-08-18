# Configuration Reference

Every `[format]` field Drut currently understands, in one place. All of them are
set the same way, in a `drut.toml` file at (or above) the file you're formatting:

```toml
[format]
control_words_casing = "lower"
indent_width = 2
```

Drut discovers the nearest `drut.toml` by walking up from the file being
processed, stopping at the first `drut.toml` found, a `.git` boundary, or the
filesystem root — whichever comes first. A project with no `drut.toml` anywhere
behaves exactly like a project with an empty one: every field uses its built-in
default. Every field is optional; omitting a key is identical to writing its
default value explicitly.

## Starter `drut.toml`

Every field, commented out at its built-in default. Copy this into a `drut.toml`
at your project root and uncomment (then change) only the fields you want to
override — a commented-out line changes nothing, so you never need to remember
the full field list or its defaults from scratch:

```toml
[format]
# control_words_casing = "preserve"      # preserve | upper | lower
# pair_keywords_casing = "preserve"      # preserve | upper | lower
# data_references_casing = "preserve"    # preserve | upper | lower
# top_level_indent = "preserve"          # preserve | auto
# indent_width = 4                       # 1-16
# operator_spacing = "preserve"          # preserve | fixed | auto
# blank_lines = "preserve"               # preserve | auto
# top_level_blank_line_cap = 2           # 1-50, only used when blank_lines = "auto"
# nested_blank_line_cap = 1              # 1-50, only used when blank_lines = "auto"
```

See [Fields](#fields) below for what each one actually does, and
[Precedence](#precedence) for how a set value interacts with CLI flags, MCP
parameters, and editor settings.

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
never assembled from pieces at different tiers. Every field below resolves
this same plain four-tier chain — no field has any extra fallback wrinkle.

> A flat `casing` field once existed, covering `control_words`+`pair_keywords`
> together — removed once the three granular fields below fully superseded
> it. A `drut.toml`/CLI/MCP/editor-setting still using `casing` no longer does
> anything; it degrades exactly like any other unrecognized key (a warning,
> falling back to each field's own built-in default), never a hard failure.

## Fields

### `control_words_casing`

Casing convention for the `control_words` category (things like `IF`,
`ENDIF`, `LOOP`, `ENDLOOP`).

**Values**: `preserve` **← default**, `upper`, `lower`.

**Default**: `preserve`.

**Also known as**: CLI flag `--control-words-casing`; MCP `format` tool parameter
`control_words_casing`.

**Example**:

```toml
[format]
control_words_casing = "upper"
```

**Precedence**: follows the [four-tier chain](#precedence) above.

### `pair_keywords_casing`

Casing convention for the `pair_keywords` category (keyword names inside
a `Control` statement's `keyword=value` pairs, e.g. `PATHLOAD`, `MATI`), same
shape as [`control_words_casing`](#control_words_casing) above.

**Values**: `preserve` **← default**, `upper`, `lower`.

**Default**: `preserve`.

**Also known as**: CLI flag `--pair-keywords-casing`; MCP `format` tool parameter
`pair_keywords_casing`.

**Example**:

```toml
[format]
pair_keywords_casing = "lower"
```

**Precedence**: follows the [four-tier chain](#precedence) above.

### `data_references_casing`

Casing for the data-reference category: Matrix/Line/Node/Zone/Database
abbreviations (`MI`/`MO`/`MW`, `LI`/`LW`, `NI`/`NW`, `ZI`/`ZONES`/`Z`,
`DBI`/`DBA`), `RO`, the link-endpoint fields `A`/`B`, and the reserved loop-index
identifiers `I`/`J`.

**Values**: `preserve` **← default**, `upper`, `lower`.

**Default**: `preserve`.

**Also known as**: CLI flag `--data-references-casing`; MCP `format` tool
parameter `data_references_casing`.

**Example**:

```toml
[format]
data_references_casing = "lower"
```

**Precedence**: follows the [four-tier chain](#precedence) above.

### `top_level_indent`

Whether top-level (depth-0, not inside any block) statement indentation is left
exactly as written, or normalized to column 0.

**Values**:
- `preserve` **← default** — leave top-level indentation exactly as written.
- `auto` — force every top-level line to column 0.

**Default**: `preserve`.

**Also known as**: CLI flag `--top-level-indent`; MCP `format` tool parameter
`top_level_indent`.

**Example**:

```toml
[format]
top_level_indent = "auto"
```

**Precedence**: follows the [four-tier chain](#precedence) above.

### `indent_width`

Spaces per nesting level of block indentation, relative to the enclosing block's
own opening-statement column.

**Values**: any integer from `1` to `16` — **default `4`**.

**Default**: `4`.

**Also known as**: CLI flag `--indent-width`; MCP `format` tool parameter
`indent_width`.

**Example**:

```toml
[format]
indent_width = 2
```

**Precedence**: follows the [four-tier chain](#precedence) above. An
out-of-range value (`0`, `500`, ...) at any tier is
treated as unset for that tier — resolution falls through to the next tier
exactly as if the field had been omitted there.

### `operator_spacing`

Whitespace normalization around `=`, comparison operators (`==`, `<>`, `>=`,
`<=`, `<`, `>`), binary arithmetic (`+`, `-`, `*`, `/`), comma spacing between
multiple `keyword=value` pairs, and interior padding inside `[...]`/`(...)`.

**Values**:
- `preserve` **← default** — leave existing spacing exactly as written.
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

**Precedence**: follows the [four-tier chain](#precedence) above.

### `blank_lines`

Whether runs of consecutive blank lines (including whitespace-only lines) are
left as written or contracted down to a configured cap.

**Values**:
- `preserve` **← default** — leave every blank-line run exactly as written,
  however long.
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

**Precedence**: follows the [four-tier chain](#precedence) above.

### `top_level_blank_line_cap`

The maximum number of consecutive blank lines `blank_lines = "auto"` allows
between top-level statements/blocks before contracting the run. Only meaningful
when [`blank_lines`](#blank_lines) is `"auto"`.

**Values**: any integer from `1` to `50` — **default `2`**.

**Default**: `2`.

**Also known as**: CLI flag `--top-level-blank-line-cap`; MCP `format` tool
parameter `top_level_blank_line_cap`.

**Example**:

```toml
[format]
blank_lines = "auto"
top_level_blank_line_cap = 1
```

**Precedence**: follows the [four-tier chain](#precedence) above. An
out-of-range value at any tier is treated as unset for
that tier, same as [`indent_width`](#indent_width).

### `nested_blank_line_cap`

The maximum number of consecutive blank lines `blank_lines = "auto"` allows
inside any block's own body, uniformly regardless of nesting depth, before
contracting the run. Only meaningful when [`blank_lines`](#blank_lines) is
`"auto"`.

**Values**: any integer from `1` to `50` — **default `1`**.

**Default**: `1`.

**Also known as**: CLI flag `--nested-blank-line-cap`; MCP `format` tool
parameter `nested_blank_line_cap`.

**Example**:

```toml
[format]
blank_lines = "auto"
nested_blank_line_cap = 2
```

**Precedence**: follows the [four-tier chain](#precedence) above. Same
out-of-range handling as
[`top_level_blank_line_cap`](#top_level_blank_line_cap).
