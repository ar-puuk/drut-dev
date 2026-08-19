# CLI Reference

`drut` has four subcommands: `check`, `format`, `server`, and `mcp`.

## `check`

```powershell
drut check <PATH>
```

Reports every structural diagnostic found for each `.s`/`.block` file under
`<PATH>` (a single file or a directory, scanned recursively). Prints nothing and
exits `0` for a clean run.

| Flag | Values | Default | Effect |
|---|---|---|---|
| `--format` | `text`, `sarif` | `text` | Output format. `sarif` emits [SARIF 2.1.0](https://sarifweb.azurewebsites.net/), for CI/tooling integration. |

## `format`

```powershell
drut format <PATH> [flags]
```

Normalizes whitespace (and, opt-in, keyword casing/operator spacing/blank-line
runs) for each `.s`/`.block` file under `<PATH>`. With none of `--write`,
`--check`, or `--diff`, defaults to printing the reformatted result to stdout.

**Disposition flags** (mutually exclusive):

| Flag | Effect |
|---|---|
| `--write` | Overwrite each matched file in place. |
| `--check` | Report which files would change; write nothing. |
| `--diff` | Print a unified diff per changed file; write nothing. |

**Formatting-axis flags** — every one of these mirrors a `drut.toml` `[format]`
field one-to-one; see the [Configuration Reference](configuration-reference.md)
for accepted values, defaults, and what each actually does:

| Flag | `drut.toml` field |
|---|---|
| `--casing-control-words` | [`casing_control_words`](configuration-reference.md#casing_control_words) |
| `--casing-pair-keywords` | [`casing_pair_keywords`](configuration-reference.md#casing_pair_keywords) |
| `--casing-data-references` | [`casing_data_references`](configuration-reference.md#casing_data_references) |
| `--casing-function-calls` | [`casing_function_calls`](configuration-reference.md#casing_function_calls) |
| `--indent-top-level` | [`indent_top_level`](configuration-reference.md#indent_top_level) |
| `--indent-width` | [`indent_width`](configuration-reference.md#indent_width) |
| `--operator-spacing` | [`operator_spacing`](configuration-reference.md#operator_spacing) |
| `--blank-lines` | [`blank_lines`](configuration-reference.md#blank_lines) |
| `--blank-lines-top-cap` | [`blank_lines_top_cap`](configuration-reference.md#blank_lines_top_cap) |
| `--blank-lines-nested-cap` | [`blank_lines_nested_cap`](configuration-reference.md#blank_lines_nested_cap) |
| `--line-wrap` | [`line_wrap`](configuration-reference.md#line_wrap) |
| `--line-wrap-width` | [`line_wrap_width`](configuration-reference.md#line_wrap_width) |
| `--line-wrap-style` | [`line_wrap_style`](configuration-reference.md#line_wrap_style) |

An explicit flag here always wins over `drut.toml` and any editor setting for
that one run — see the Configuration Reference's
[Precedence](configuration-reference.md#precedence) section.

**Other flags**:

| Flag | Effect |
|---|---|
| `--isolated` | Skip `drut.toml` discovery entirely for this run — built-in defaults plus whatever other flags you passed. Useful for CI reproducibility or a one-off sanity check. |

## `server`

```powershell
drut server
```

Speaks the Language Server Protocol over stdio. No flags — launched by an LSP
client (like the VS Code extension), not run interactively. See the
[Editor Guide](editor-guide.md).

## `mcp`

```powershell
drut mcp
```

Speaks the Model Context Protocol over stdio. No flags — launched by an
MCP-capable client (an AI coding assistant), not run interactively. Exposes four
read-only tools, entirely independent of `server` above (no shared state). See
the [MCP Guide](mcp-guide.md).
