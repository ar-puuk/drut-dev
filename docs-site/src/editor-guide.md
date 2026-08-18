# Editor (LSP) Guide

Everything here comes from `drut server`, launched automatically by the VS
Code/Open VSX extension — and, since it's a standard Language Server Protocol
implementation, usable from any LSP-capable editor, not only VS Code.

## Diagnostics

Two kinds of diagnostics are published, at different severities.

**Structural diagnostics** (real problems, `Error` severity) — seven categories,
covering unmatched blocks and a few other real structural defects:

| Diagnostic | Fires on |
|---|---|
| `UnmatchedIf` | An `IF` with no matching `ENDIF`, or a dangling `ENDIF`/`ELSEIF`/`ELSE`. |
| `UnmatchedLoop` | A `LOOP` with no matching `ENDLOOP`, or a dangling `ENDLOOP`. |
| `UnclosedBlockComment` | A block comment with no matching `*/` before end of file. |
| `InvalidContinuation` | A continuation character with no valid following line. |
| `UnmatchedRun` | A non-disabled `RUN` with no `ENDRUN` and no implicit closer (a following `RUN` or shell-escape statement), a disabled `!RUN` missing its required explicit `ENDRUN`, or a dangling `ENDRUN`. |
| `UnmatchedProcess` | A `PROCESS`/`PHASE=` with no matching `ENDPROCESS`/`ENDPHASE` and no following `PROCESS`/`PHASE=` (the legitimate implicit-close pattern). |
| `MisplacedBreak` | A `BREAK` with no enclosing block of any kind. |

(An eighth category, `InvalidEncoding`, exists in `voyager-core` for raw-byte
input but is unreachable through live editing — the LSP transport only ever
delivers already-decoded text.)

**Hint-level diagnostics** (best-effort signals, not hard errors, `Hint`
severity) — three of these, each its own distinct source so they're visually
and programmatically distinguishable from the structural set above:

| Diagnostic | Source | Fires on |
|---|---|---|
| Unclosed `; FMT: OFF` | `drut-fmt` | A `; FMT: OFF` marker with no matching `; FMT: ON` before end of file — the rest of the file stays unformatted, and this tells you why. |
| Malformed `drut.toml` | `drut-config` | An unrecognized key or an out-of-range value in the resolved `drut.toml` — formatting still completes using the built-in default for just that field. |
| Undefined `@token@` | `drut-token` | An `@token@` reference with no assignment findable in the same file or a directly included one. **Never a hard error** — a resolver blind spot (a reference on a block-opener line, more than one level of `READ FILE` inclusion, or a token-built inclusion path) is never itself treated as evidence the token doesn't exist; it may still be defined somewhere Drut can't see. |

## Hover

Hovering a block keyword (`IF`, `LOOP`, `RUN`, ...) shows its kind and where its
matched counterpart is — correctly resolved even through `RUN`/`PROCESS`'s
implicit-close quirk. Hovering an `@token@` reference shows the value it
currently resolves to and where that value was assigned (the most recent
same-file assignment before the reference, or one found via a directly-included
`READ FILE`).

## Completion and spell-check

Autocomplete for control words and `keyword=value` pair names is scoped to the
enclosing control word (e.g. completing inside `RUN PGM=...` only offers
`PGM`-relevant pair keywords). A misspelled keyword gets a "did you mean"
suggestion riding on the same hover mechanism.

## Folding

Every block kind (`IF`/`LOOP`/`RUN`/`PROCESS`/`JLOOP`/`LINKLOOP`/
`DISTRIBUTEMULTISTEP`) and block comment can be collapsed/expanded like any
other language.

## Format-on-save and format-on-paste

**Format-on-save** is auto-enabled the first time the extension activates in a
workspace (workspace-scoped, one-time — it won't silently turn itself back on
if you disable it afterward). Saving a `.s`/`.block` file reformats it
automatically.

**Format-on-paste** stays off by default. Turn it on with:

```json
{
  "[drut]": {
    "editor.formatOnPaste": true
  }
}
```

in your workspace's `.vscode/settings.json`. Once enabled, pasting Cube Voyager
script text into a `.s`/`.block` file reindents it to match its new surrounding
structure immediately — correctly handling a paste that opens or closes a
block.

## Editor client settings

All 9 `[format]` fields (see the [Configuration Reference](configuration-reference.md))
are also available as personal VS Code settings, not only via a project's
committed `drut.toml`:

| Setting | `drut.toml` field |
|---|---|
| `drut.format.casingControlWords` | `casing_control_words` |
| `drut.format.casingPairKeywords` | `casing_pair_keywords` |
| `drut.format.casingDataReferences` | `casing_data_references` |
| `drut.format.indentTopLevel` | `indent_top_level` |
| `drut.format.indentWidth` | `indent_width` |
| `drut.format.operatorSpacing` | `operator_spacing` |
| `drut.format.blankLines` | `blank_lines` |
| `drut.format.blankLinesTopCap` | `blank_lines_top_cap` |
| `drut.format.blankLinesNestedCap` | `blank_lines_nested_cap` |

Set these through VS Code's built-in Settings UI (search for "drut"), or
directly in `settings.json`. **A `drut.toml` value always wins** over a
conflicting client setting for the same field — a client setting is a personal
fallback default, never a way to override a project's own committed
configuration. See the Configuration Reference's
[Precedence](configuration-reference.md#precedence) section for the full
four-tier chain. A changed setting takes effect on the very next format request
against an already-open document — no reopen or editor restart needed.
