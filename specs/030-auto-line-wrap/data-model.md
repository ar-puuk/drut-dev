# Data Model: Automatic Line-Width Wrapping

## §1. `voyager-core` types

### `LineWrapMode` (new)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineWrapMode {
    #[default]
    Preserve,
    Auto,
}
```

- Two-value, `Preserve`-default shape, same pattern as `BlankLineMode`.

### `LineWrapStyle` (new)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineWrapStyle {
    #[default]
    Fill,
    OnePerLine,
}
```

- `Fill` is the default, not `OnePerLine` — a spec.md-resolved decision (Clarification Q2),
  driven directly by FR-005's "never re-flow an already-continued statement" rule: whichever
  style wraps a statement first is effectively permanent for it, and manually further-splitting
  an already-packed `Fill` line later is a smaller, safer, always-valid edit than manually
  un-packing many `OnePerLine` continuations back into `Fill` form — so the default is the
  direction that's cheaper to manually diverge from.
- Has no effect at all unless `line_wrap == Auto` (FR-002a) — reading this field while
  `line_wrap == Preserve` never happens, mirroring how `blank_lines_top_cap`/
  `blank_lines_nested_cap` are likewise inert under `BlankLineMode::Preserve`.

### `FormatOptions` (modified)

```rust
#[derive(Debug, Clone, Copy)]
pub struct FormatOptions {
    // ...existing fields unchanged...
    pub line_wrap: LineWrapMode,           // new, default Preserve
    pub line_wrap_width: u16,              // new, default 120
    pub line_wrap_style: LineWrapStyle,    // new, default Fill
}
```

### `line_wrap` module (new)

```rust
/// One eligible split point within a Control statement's flat token list --
/// a top-level comma (research.md §4), identified by its own token index
/// and character span.
struct SplitPoint {
    token_index: usize,
    span: Span,
}

/// Walks `tokens` (a Control statement's own flat token list, from
/// `build_statements`) consulting `operator_spacing::quoted_token_mask`
/// first (reused, not duplicated -- research.md §4 corrected during
/// implementation: a quoted value is NOT one atomic token in this grammar,
/// `'a, b'` lexes as separate tokens, the same discovery
/// `operator_spacing.rs` already made for its own operator characters). A
/// masked (inside-a-string) token is skipped entirely, including for
/// paren/bracket depth-tracking. Among unmasked tokens, tracks paren `(`/`)`
/// and bracket `[`/`]` depth, collecting every `,` Punctuation token seen at
/// depth zero -- a comma nested inside a function call's parentheses or a
/// bracketed subscript is never collected.
fn top_level_split_points(tokens: &[Token]) -> Vec<SplitPoint>;

/// `true` if `tokens` contains any `TokenKind::ContinuationMarker` --
/// FR-005's "already continued" check. A statement for which this is true
/// is never touched by this module at all, checked before any other work
/// in this module runs (research.md §1, the mechanism idempotence relies
/// on).
fn already_continued(tokens: &[Token]) -> bool;

/// Given a Control statement's rendered single-line length, its own split
/// points, and the configured width/style, decides which split points
/// actually become line breaks (research.md §4-model):
/// - `Fill`: walk split points left to right, breaking at the last split
///   point still within budget before the next pair would exceed it --
///   never breaks at every available point, only where needed.
/// - `OnePerLine`: every split point becomes a break, unconditionally.
/// Returns `None` (no wrap edits at all) when the statement's own rendered
/// length is already at or under the configured width, or when there are
/// zero split points -- both are "leave untouched," not "wrap with zero
/// breaks."
fn plan_wrap(
    statement_text: &str,
    split_points: &[SplitPoint],
    width: u16,
    style: LineWrapStyle,
) -> Option<Vec<SplitPoint>>;

/// Builds the actual SpacingEdit for one chosen split point: a zero-width
/// insertion immediately after the comma, whose replacement is `terminator`
/// (the specific original line's own captured CRLF/LF style, never a
/// hardcoded '\n' -- research.md §1) followed by the continuation line's
/// own indentation (one level deeper than the statement's opening line's
/// resolved indent, computed independently of `indent_plan`, which has no
/// entry for a line that didn't exist in the original source).
fn wrap_edit(split: &SplitPoint, terminator: &str, continuation_indent: &str) -> SpacingEdit;
```

- Pure functions over already-parsed `Statement`/`Token` data (`top_level_split_points`,
  `already_continued`, `plan_wrap`) plus one function (`wrap_edit`) that takes the caller's
  already-resolved terminator/indentation strings as plain data — no I/O anywhere in this
  module, never panics, same contract shape every other `voyager-core` internal module already
  has.
- Lives in its own module (`src/line_wrap.rs`), not inside `format.rs` directly — same
  separation `operator_spacing.rs` already established, keeping `format.rs`'s existing casing/
  indentation/spacing logic undisturbed by this feature's own comma-adjacency scanning.
- `already_continued` is checked first, before `top_level_split_points`/`plan_wrap` are even
  called, for every candidate `Control` statement — not a filter applied after the fact that a
  future change could accidentally bypass.

## §2. Edit application (research.md §1)

### `SpacingEdit` (existing type, semantics extended)

```rust
// Unchanged shape: (line, 0-based char start, 0-based char end (exclusive), replacement text).
// This feature is the first to put a literal line-terminator character
// inside `replacement` -- the existing per-line rebuild loop already
// accepts this mechanically (it just extends the output buffer with
// `replacement`'s chars), so no change to the edit *type* itself, only to
// what a caller is now allowed to put inside one.
type SpacingEdit = (u32, usize, usize, String);
```

### `render()` change

`render()`'s existing per-line rebuild loop (`018-operator-spacing`'s own addition) already
walks a line's merged, sorted edit list and splices in each edit's `replacement`, whatever its
length. This feature adds no new loop shape — it adds a new *source* of `SpacingEdit`s
(`line_wrap::plan_wrap` + `wrap_edit`, collected once per `Control` statement, short-circuited
entirely when `options.line_wrap == LineWrapMode::Preserve`, mirroring `casing`'s/
`operator_spacing`'s existing short-circuit) whose replacement strings happen to contain an
embedded terminator character.

**The one real change to the loop itself**: after a wrap edit's replacement (containing a
terminator) has been spliced in, the loop's own line-level `out.push_str(terminator)` call
(applied once, at the very end of the *original* line's own content) still fires exactly once,
appending that original line's terminator after the *last* physical line the original line was
split into — correct by construction, since the split-off continuation lines' own terminators
are the ones embedded inside each wrap edit's replacement, not the original line's trailing
terminator (which still belongs to whatever content remains after the final split point).

### Performance short-circuit

Mirrors the existing `options.casing.* != Preserve`/`options.operator_spacing != Preserve`
short-circuits in `render()`: the whole `line_wrap` collection pass is skipped entirely when
`options.line_wrap == LineWrapMode::Preserve` — a `Preserve`-configured or unconfigured call
does exactly the same work it did before this feature existed (FR-007/SC-003).

## §3. Configuration precedence

Three new settings, each following the same single-tier precedence every existing `[format]`
field already has (no multi-tier legacy-vs-granular arbitration, matching `top_level_indent`'s
simple shape):

| Setting | Precedence (highest wins) |
|---|---|
| `line_wrap` | explicit `--line-wrap`/MCP param → `drut.toml`'s `line_wrap` field → personal `drut.format.lineWrap` setting → built-in default `preserve` |
| `line_wrap_width` | explicit `--line-wrap-width`/MCP param → `drut.toml`'s `line_wrap_width` field → personal `drut.format.lineWrapWidth` setting → built-in default `120` |
| `line_wrap_style` | explicit `--line-wrap-style`/MCP param → `drut.toml`'s `line_wrap_style` field → personal `drut.format.lineWrapStyle` setting → built-in default `fill` |

### `drut_config::FormatConfig` / `ExplicitFormatOverride` (modified)

```rust
pub struct FormatConfig {
    // ...existing fields unchanged...
    pub line_wrap: Option<LineWrapMode>,           // new
    pub line_wrap_width: Option<u16>,              // new
    pub line_wrap_style: Option<LineWrapStyle>,    // new
}
// ExplicitFormatOverride mirrors the same shape, same pattern as every
// other [format] field already established.
```

An unrecognized/invalid `line_wrap`/`line_wrap_style` TOML value (anything other than their
accepted values, case-insensitive, matching every other enum-shaped `[format]` field's existing
case-insensitivity) or an out-of-range `line_wrap_width` (e.g. `0`, or an unreasonably large
value — exact bound a planning-phase-confirmed range, mirroring `resolve_blank_line_cap`'s own
validated-range shape) is discarded with a non-blocking notice and falls through to that field's
own built-in default, identical to every other malformed `[format]` field (FR-009).

## §4. What this feature does *not* touch

- `voyager_core::Diagnostic`/`DiagnosticKind`: unchanged — this feature produces no new
  diagnostic of any kind.
- The `018-operator-spacing`/`019-blank-line-normalization`/`017-casing-categories-indent-width`
  formatting axes: unchanged, unaffected — this feature's edits and theirs coexist in the same
  per-line merged/sorted edit list `render()` already builds, the same way every pair of
  existing axes already coexists.
- `Assignment`/`Label`/`ShellEscape` statement forms: never wrapping candidates in this
  increment (spec.md Assumptions) — only `Control` statements are.
- A function call's parentheses, a bracketed subscript, or a quoted string's interior: never an
  eligible split point, regardless of width (FR-003).
