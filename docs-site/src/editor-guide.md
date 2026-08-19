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

## Syntax highlighting

Static TextMate-grammar highlighting (works immediately, before the language
server even attaches) recognizes these categories:

| Category | Covers | Scope |
|---|---|---|
| Control words | `IF`, `LOOP`, `RUN`, `ENDIF`, ... | `keyword.control.drut` |
| Statement words | `PRINT`, `FILEI`, `FILEO`, `ARRAY`, ... | `support.function.statement.drut` |
| Function calls | A recognized Cube Voyager built-in function name immediately followed by `(` — `REPLACESTR(...)`, `ROUND(...)`, and 136 others (see the [Formatter Guide](formatter-guide.md#function-call-casing) for the full list) | `support.function.builtin.drut` |
| Pair-keyword names | A `keyword=value` pair's keyword, e.g. `PATHLOAD`'s `PATH` | `variable.parameter.drut` |
| Values | A pair's bareword value, e.g. `PGM=MATRIX`'s `MATRIX` | `constant.other.drut` |
| Data references | The Matrix/Line/Node/Zone/Database family (`MI`, `MW`, `DBA`, `ZONES`, ...), by name, regardless of position | `variable.language.data-reference.drut` |
| User variables | Any other bareword identifier not covered by a category above | `variable.other.identifier.drut` |
| `@name@` substitution | Variable references | `variable.other.readwrite.drut`, plus a semantic-token `variable` override (below) |
| Numbers | Numeric literals | `constant.numeric.drut` |
| Operators | `=`, `+`, `-`, `<>`, ... | `keyword.operator.drut` |
| Comments | `; ...` and `/* ... */` | `comment.line.semicolon.drut` / `comment.block.drut` |
| Strings | Quoted string literals | `string.quoted.single.drut` / `string.quoted.double.drut` |

Function calls and statement words render in the same color by default (both
use the generic "built-in procedure" convention most themes already style),
but are independently recognized and independently colorable — see Highlight
color customization below.

`@name@` references also always get a real color, not just whatever a theme
happens to assign — the extension auto-seeds a `#4EC9B0` semantic-token
override the first time it activates in a workspace, since some themes render
that TextMate scope with no color at all. This seed is workspace-scoped and
one-time only: deleting it from `.vscode/settings.json` by hand keeps it
deleted, forever, for that workspace (the extension never fights that choice
back) — unless you configure `drut.highlight.namedVariables` (below).

## Editor client settings

All 10 `[format]` fields (see the [Configuration Reference](configuration-reference.md))
are also available as personal VS Code settings, not only via a project's
committed `drut.toml`:

| Setting | `drut.toml` field |
|---|---|
| `drut.format.casingControlWords` | `casing_control_words` |
| `drut.format.casingPairKeywords` | `casing_pair_keywords` |
| `drut.format.casingDataReferences` | `casing_data_references` |
| `drut.format.casingFunctionCalls` | `casing_function_calls` |
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

## Highlight color customization

Unlike the `[format]` fields above, `drut.highlight.*` settings are **VS Code
personal settings only** — there is no `drut.toml` equivalent, no CLI flag, no
MCP parameter. Color is a personal/accessibility preference (theme,
colorblindness, monitor), not a shared file-content convention the way casing
or indentation is, so there's nothing to put in a committed project file.

Eleven settings, one per category from the Syntax highlighting table above
(`@name@` excepted — see below), each an optional CSS color
(e.g. `#RRGGBB`):

| Setting | Colors |
|---|---|
| `drut.highlight.controlWords` | Control words |
| `drut.highlight.statementWords` | Statement words |
| `drut.highlight.functionCalls` | Function calls |
| `drut.highlight.pairKeywords` | Pair-keyword names |
| `drut.highlight.values` | Values |
| `drut.highlight.dataReferences` | Data references |
| `drut.highlight.userVariables` | User variables |
| `drut.highlight.numbers` | Numbers |
| `drut.highlight.operators` | Operators |
| `drut.highlight.comments` | Comments |
| `drut.highlight.strings` | Strings |

Leaving any of these unset keeps your color theme's own choice for that
category — setting one takes effect immediately (no window reload), and
clearing it afterward reverts to the theme's color, not a stuck last value.
None of these ever touch a rule they didn't add themselves — another
extension's customizations, or your own hand-written ones, always survive
untouched.

A bareword immediately before `=` always renders under `pairKeywords`, even
if it's also a `userVariables`-shaped identifier (`LINKID` in
`LINKID = _ANode`) — this grammar has no real parse tree to tell a
keyword-pair's own name apart from an ordinary assignment's target variable.
The bareword
immediately *after* `=` renders under `pairValues` only when it's the
entire right-hand side, with nothing else following (`X = _ANode` alone) —
that shape is genuinely indistinguishable from a keyword-pair's own value
(`PGM=MATRIX`'s `MATRIX`) without a real parse tree. As soon as anything
else follows on the same right-hand side — another operand, an operator, a
string — the whole expression falls to `userVariables` instead, so
`LINKID = _ANode + '_' + _BNode`'s `_ANode` and `_BNode` render identically
(neither is a real keyword-pair value). `dataReferences` is the one
exception to the adjacency rule entirely:
a recognized data-reference name always wins that category even when it's
also pair-keyword-shaped (`ZONES` in `RUN PGM=MATRIX ZONES=5` renders under
`dataReferences`, not `pairKeywords`).

**`drut.highlight.namedVariables`** (`@name@` substitution) works the same way
from a user's perspective, but is written into the current *workspace's*
settings (`.vscode/settings.json`), not your personal/global settings — VS
Code resolves this particular setting per-scope rather than merging across
scopes, and the auto-seeded default described above already lives at
workspace scope, so a global-scope write would be silently invisible.
Leaving it unset preserves the auto-seed behavior exactly (including "a
manual deletion sticks forever"); setting it takes over live; clearing it
afterward reverts to the `#4EC9B0` default specifically, not to no color at
all (a fully theme-driven state would reintroduce the invisibility problem
this default exists to prevent).
