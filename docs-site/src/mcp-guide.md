# MCP Guide

`drut mcp` speaks the [Model Context Protocol](https://modelcontextprotocol.io/)
over stdio — a way for an AI coding assistant to query Drut's understanding of a
Cube Voyager script directly, instead of guessing from raw text. All four tools
are **read-only**: none of them write to disk.

Launch it the same way any MCP client launches a stdio server: point your
client's MCP configuration at the `drut` binary with the `mcp` argument.

## `diagnose`

Reports every structural diagnostic `voyager-core` can find for a script (given
as inline text or a file path). Returns an empty list for a structurally valid
script. Same diagnostic categories as the [Editor Guide](editor-guide.md#diagnostics)'s
structural set — the Hint-level streams (unclosed `; FMT: OFF`, malformed
`drut.toml`, undefined `@token@`) are LSP-only and never appear here.

**Use it when**: an assistant needs to check whether a script (or a change it's
about to propose) is structurally valid before suggesting it.

## `format`

Reformats a script's whitespace/indentation (and, opt-in via the same
parameters `drut format`/`drut.toml` accept — `casing_control_words`,
`operator_spacing`, `blank_lines`, and the rest) and returns the result plus
whether anything changed. Idempotent: formatting an already-formatted script
reports `changed=false`.

**Use it when**: an assistant is about to write or hand back Cube Voyager script
text and wants it consistently formatted first.

## `query_structure`

Reports which of the seven block kinds (`If`/`Loop`/`Run`/`Process`/`JLoop`/
`LinkLoop`/`DistributeMultistep`), if any, encloses a given 1-based line/column
position in a script, and where its matched counterpart is — correctly resolved
even through `Run`/`Process`'s implicit-close quirk. Reports `kind: null`, not
an error, when no block encloses the position.

**Use it when**: an assistant needs to understand the block structure around a
specific position — e.g. "is this line inside an `IF`, and if so, where does
that `IF` end?"

## `lookup_keyword`

Looks up real, corpus-evidenced `keyword=value` pair-name candidates for a given
enclosing control word (e.g. `RUN`), falling back to the general control-word
list when none is given. Optionally also runs a "did you mean" spell-check
against a supplied token.

**Use it when**: an assistant is generating or validating a `keyword=value` pair
and wants to confirm the keyword name is real (or find the likely intended one
for a typo).
