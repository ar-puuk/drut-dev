# Contract: Automatic Line-Width Wrapping (addition)

A conceptual signature contract, not final Rust source, but the shapes and guarantees below are
binding — same convention every prior contract doc in this repo follows.

## `voyager-core` additions

```text
pub enum LineWrapMode { Preserve (default), Auto }
pub enum LineWrapStyle { Fill (default), OnePerLine }
pub struct FormatOptions {
    // ...existing fields...
    pub line_wrap: LineWrapMode,
    pub line_wrap_width: u16,       // default 120
    pub line_wrap_style: LineWrapStyle,
}
```

- Pure, no I/O, never panics on any input, including structurally broken/diagnosed statements —
  same contract shape every other public `voyager-core` type/function already has.
- `line_wrap.rs`'s internal functions (`top_level_split_points`, `already_continued`,
  `plan_wrap`, `wrap_edit`) are module-private — no new public API surface beyond the two enums
  and the three `FormatOptions` fields above.
- Every existing `voyager-core` public type/function (`CasingConvention`, `OperatorSpacing`,
  `BlankLineMode`, `format`/`format_bytes`, everything in `token_resolution.rs`): **unchanged**.

## `drut-config` additions

```text
pub struct FormatConfig {
    // ...existing fields...
    pub line_wrap: Option<LineWrapMode>,
    pub line_wrap_width: Option<u16>,
    pub line_wrap_style: Option<LineWrapStyle>,
}
```

- Same `explicit > drut.toml > personal-setting > built-in-default` precedence chain (data-model.md
  §3) every existing `[format]` field already has, resolved through the same single shared
  function every CLI/LSP/MCP adapter already calls.

## `drut-cli`/`drut-mcp`/`editors/vscode` additions

- CLI: `--line-wrap=<preserve|auto>`, `--line-wrap-width=<u16>`, `--line-wrap-style=<fill|one-per-line>`.
- MCP `format` tool: matching `line_wrap`/`line_wrap_width`/`line_wrap_style` params, same
  threading shape as every existing multi-field `[format]` option.
- VS Code extension: `drut.format.lineWrap`/`drut.format.lineWrapWidth`/`drut.format.lineWrapStyle`
  personal settings, same shape as every existing `drut.format.*` entry — a project's committed
  `drut.toml` still wins over these for the same field.

## Guarantees

- **Byte-identical when unconfigured** (FR-007/SC-003): a project with no `line_wrap`
  configuration anywhere produces output identical to before this feature existed, verified
  against the full existing golden-fixture set and real corpus.
- **Opt-in only** (FR-001): `line_wrap` defaults to `Preserve`; wrapping is never active unless
  explicitly enabled somewhere in the resolved configuration chain.
- **Sensible width default once opted in** (FR-002): `Auto` with no explicit width anywhere
  resolves to `120`, not an error and not silent no-op.
- **Configurable wrap style, `Fill` default** (FR-002a): `Auto` with no explicit style resolves
  to `Fill` (as many pairs as fit per continuation line); `OnePerLine` is available as an
  explicit opt-in alternative.
- **Only `Control`-statement top-level pair-list commas are eligible split points** (FR-003,
  FR-004): never inside a function call's parentheses, a bracketed subscript, or a quoted
  string; never on an `Assignment`/`Label`/`ShellEscape` statement; never on a `Control`
  statement with no eligible comma.
- **Never re-flows an already-continued statement** (FR-005): a statement containing any
  `ContinuationMarker` token is left completely untouched by this feature, regardless of its
  width — the mechanism idempotence (SC-004) relies on structurally, not incidentally.
- **Correct continuation-line indentation** (FR-006): one level deeper than the statement's own
  opening line, computed independently of `indent_plan` (which has no entry for a synthetic
  line).
- **Terminator-correct** (data-model.md §2): an inserted line break uses the specific original
  line's own captured CRLF/LF style, never a hardcoded `\n` — verified with a dedicated
  CRLF-file test, not assumed.
- **Idempotent** (SC-004): running `Auto` twice in a row produces no further change on the
  second pass, verified with a dedicated second-pass fixture (not merely a generic re-run-diff
  check).
- **Invalid configuration never silently misbehaves** (FR-009/SC-005): a malformed `drut.toml`
  value degrades to that field's own built-in default with a non-blocking notice, every time; a
  command-line or MCP value outside the accepted shape is rejected with a clear usage/tool error
  at that surface's own input point, every time.
- **Every configuration surface exposes this identically** (FR-010): CLI, `drut.toml`, MCP, and
  the VS Code personal-setting mechanism all expose the same three settings, with the same
  resolved precedence — no surface silently lagging or disagreeing with another.

## What this contract does *not* promise (by design, this phase)

- No wrapping of `Assignment`/`Label`/`ShellEscape` statements, or of an arithmetic/string
  expression's own `+ - / * ^ & |` continuation characters — `Control`-statement pair-list
  commas only, this increment.
- No splitting inside a function call's parentheses or a bracketed subscript, regardless of
  width.
- No re-flowing/re-wrapping of a statement that already contains any author-written
  continuation, regardless of that statement's current width.
- No dead-store-style "was this wrap actually necessary" analysis beyond the width comparison —
  purely mechanical once a statement is identified as an eligible, over-width, not-already-
  continued `Control` statement.
