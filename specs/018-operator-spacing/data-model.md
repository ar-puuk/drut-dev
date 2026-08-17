# Data Model: Operator Spacing Normalization

## §1. `voyager-core` types

### `OperatorSpacing` (new)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OperatorSpacing {
    #[default]
    Preserve,
    Fixed,
    Auto,
}
```

- Same three-value, `Preserve`-default shape as `CasingConvention`/`TopLevelIndentMode` — no
  new pattern, a repeat of an established one.
- `Auto` is a strict superset of `Fixed`'s behavior (FR-006) — the renderer implements this as
  "do everything `Fixed` does, then additionally compute alignment padding," never two
  independent code paths that could drift apart.

### `FormatOptions` (modified)

```rust
#[derive(Debug, Clone, Copy)]
pub struct FormatOptions {
    pub casing: CasingSettings,
    pub top_level_indent: TopLevelIndentMode,
    pub indent_width: u8,
    pub operator_spacing: OperatorSpacing,   // new
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            casing: CasingSettings::default(),
            top_level_indent: TopLevelIndentMode::default(),
            indent_width: 4,
            operator_spacing: OperatorSpacing::default(),   // Preserve
        }
    }
}
```

### `operator_spacing` module (new)

```rust
/// A recognized operator occurrence within one statement's already-tokenized
/// value/pair token list (research.md §1, §2, §5).
struct OperatorOccurrence {
    kind: OperatorKind,
    span: Span,              // the operator's own span (merged, for multi-char ops)
    is_continuation: bool,   // research.md §3 — suppresses the trailing-side edit
}

enum OperatorKind {
    Assignment,                       // =
    Comparison(ComparisonOp),         // ==, <>, >=, <=, <, >
    Arithmetic(ArithmeticOp),         // + - * / (binary only, research.md §5)
    Comma,                            // between Control pairs
}

/// Tracks which token indices in `tokens` fall inside an open string/quoted
/// literal — an independent local pass, since `TokenKind` itself doesn't
/// expose this (research.md §9). Odd running count of `'`/`"` Punctuation
/// tokens seen so far == inside a string; unmatched trailing quote treats
/// everything after it as inside a string too (fail toward exclusion, never
/// toward false recognition).
fn quoted_token_mask(tokens: &[Token]) -> Vec<bool>;

/// Merges adjacent single-char `=`/`<`/`>` Punctuation tokens into one
/// logical multi-char comparison operator when zero-gap-adjacent
/// (research.md §2); passes every other operator token through as-is.
/// Consults `quoted_token_mask` first — a token inside a string is never
/// recognized as an operator at all (research.md §9).
fn recognize_operators(tokens: &[Token]) -> Vec<OperatorOccurrence>;

/// Distinguishes a unary +/- from a binary one by looking at the previous
/// token in the same value/pair token list (research.md §5); binary +/- and
/// every other operator kind always participates.
fn is_binary_arithmetic(tokens: &[Token], index: usize) -> bool;

/// Bracket/paren interior-padding and control-word-paren adjacency rules
/// (research.md §7) — same token-pair-scan shape as `recognize_operators`,
/// kept in its own function only because its "zero space" rule differs from
/// operators' "one space" rule, not because it needs different input data.
/// Also consults `quoted_token_mask` first, same as `recognize_operators`.
fn collect_bracket_paren_edits(tokens: &[Token], edits: &mut Vec<SpacingEdit>);
```

- **A token inside an open string/quoted literal is never recognized by any rule in this
  module** (FR-010a) — `quoted_token_mask` is consulted before every other check, not layered
  on afterward as a filter that could be forgotten at a new call site. Confirmed necessary by
  direct testing: `tokenize("LIST='a+b'\n")` emits a standalone `Punctuation("+")` token for the
  `+` inside the quotes, indistinguishable from a real operator at the `TokenKind` level
  (research.md §9) — this is not a hypothetical edge case, it's the tokenizer's actual, verified
  behavior today.
- Pure functions over already-parsed `Statement`/`Token` data, no I/O, never panics — same
  contract shape every other `voyager-core` public/internal function already has.
- Lives in its own module (`src/operator_spacing.rs`), not inside `format.rs` directly — same
  separation `data_reference.rs` already established for `017`, keeping `format.rs`'s existing
  casing/indentation logic undisturbed by this feature's token-adjacency scanning.

## §2. Edit application (research.md §4)

### `SpacingEdit` (new)

```rust
/// (line, 0-based char start, 0-based char end (exclusive), replacement text)
/// Same shape as the existing `CasingEdit` alias, but — unlike `CasingEdit` —
/// `replacement.len()` is NOT required to equal `end - start`: this is the
/// type that actually needs insertion/removal, which `CasingEdit`'s
/// same-length contract never had to support.
type SpacingEdit = (u32, usize, usize, String);
```

### `render()` line-application change

Today, `render()` applies `CasingEdit`s via a same-length in-place column splice
(`chars[start..end].clone_from_slice(&repl_chars)`, guarded by `repl_chars.len() == end -
start`). This guard is *why* a variable-length edit silently no-ops today — not a bug to fix in
the existing casing path (casing edits are always same-length, the guard is correct for them),
but a capability that has to be added net-new for spacing.

New per-line rebuild, used only when `spacing_edits` is non-empty for that line:

1. Collect that line's `CasingEdit`s and `SpacingEdit`s into one list, sorted by `start` column
   (both kinds operate on disjoint spans — token text vs. surrounding whitespace — so a single
   merged, sorted pass is safe; no span from one kind ever overlaps a span from the other).
2. Walk the line left-to-right: copy each untouched gap between the previous edit's `end` and
   the next edit's `start` verbatim, then splice in that edit's `replacement` (whatever its
   length).
3. Copy the remaining tail after the last edit verbatim.

Indentation continues to apply afterward, exactly as today (leading-whitespace-only, computed
against the now-rebuilt line's own current leading whitespace) — unaffected by this change since
it never looks past the first non-whitespace character.

### Performance short-circuit

Mirrors the existing `options.casing.* != Preserve` short-circuit in `render()`: the whole
`operator_spacing` collection pass (and the merged-rebuild line path) is skipped entirely when
`options.operator_spacing == OperatorSpacing::Preserve` — a `Preserve`-configured or
unconfigured call does exactly the same work it did before this feature existed (FR-009/SC-003).

## §3. Alignment-run detection (`Auto` only, research.md §6)

```rust
/// One maximal run of consecutive `Node::Statement(Assignment)` entries
/// within a single `Vec<Node>` slice (a block's children, or the top-level
/// node list) — "same nesting depth" is free, since sibling adjacency in
/// that slice already implies it (research.md §6).
struct AlignmentRun {
    /// Index range within the enclosing `Vec<Node>` slice.
    members: Vec<AssignmentMember>,
    target_column: usize,   // longest left-hand side + 1 space, across the run
}

struct AssignmentMember {
    /// The statement's own `=` operator span (post-Fixed-normalization
    /// position, i.e. already exactly one space after the left-hand side
    /// before alignment padding is added).
    equals_span: Span,
    lhs_width: usize,
}
```

- A run ends (and the next one starts fresh) at: a non-`Assignment` sibling node (including a
  pair-keyword-shaped `Control` statement, FR-007), a blank source line between two siblings'
  spans, a comment-only source line between them (FR-008), or an `Assignment` statement whose
  own line falls inside a `; FMT: OFF`/`; FMT: ON` protected region (FR-008) — a protected
  member is excluded from the run entirely (never a member, never padded, never counted toward
  a neighbor's `target_column`), the same as any other non-participating statement, not
  silently skipped-while-still-counted. All four checked once per adjacent sibling pair via the
  same between-statement line scan `protected_regions` already uses to classify a line as
  comment-only/protected-or-not, not a new classification mechanism.
- `target_column` is computed once per run, after every member's `Fixed`-shaped single-space
  edit is already known — alignment padding is *additional* spaces inserted before the `=`,
  never a change to what `Fixed` already decided the minimum correct spacing is.
- A run of length 1 produces no alignment edit at all — its member is already correctly spaced
  by the `Fixed`-equivalent pass alone (spec.md US2 Acceptance Scenario 4).

## §4. Configuration precedence

Single new setting, so no multi-tier arbitration like `017`'s legacy-vs-granular casing case —
this mirrors `top_level_indent`'s simpler one-setting precedence exactly:

| Setting | Precedence (highest wins) |
|---|---|
| `operator_spacing` | explicit `--operator-spacing`/MCP param → `drut.toml`'s `operator_spacing` field → built-in default `preserve` |

### `drut_config::FormatConfig` / `ExplicitFormatOverride` (modified)

```rust
pub struct FormatConfig {
    // ...existing fields unchanged...
    pub operator_spacing: Option<OperatorSpacing>,   // new
}
// ExplicitFormatOverride mirrors the same shape, same pattern as every
// other [format] field already established.
```

An unrecognized/invalid TOML value (anything other than `"preserve"`/`"fixed"`/`"auto"`,
case-insensitive — matching `casing`'s existing case-insensitivity) is discarded with a
non-blocking notice and falls through to the built-in default, identical to every other
malformed `[format]` field (FR-011).

## §5. `format.rs` module doc comment (research.md §8)

The "Scope, precisely" doc comment's claim that "intra-line spacing between tokens... is copied
through unchanged" is reworded to state that plainly for `Preserve` (still the default and
still what every existing caller gets), while pointing to this feature's module for what
`Fixed`/`Auto` additionally do — not deleted, since it remains exactly true for the default and
every existing configuration.
