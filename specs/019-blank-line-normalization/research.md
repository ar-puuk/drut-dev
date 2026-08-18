# Phase 0 Research: Blank-Line-Run Normalization

Grounded directly in `crates/voyager-core/src/format.rs` as it exists today (post-`018-operator-
spacing`), not assumed from the spec alone.

## §1: `render()` has no capability to delete a line — a genuinely new one, like `018`'s `SpacingEdit`

Every existing formatting axis (indentation, casing, operator spacing) operates on a strict
1-input-line-to-1-output-line correspondence: `render()`'s main loop iterates `raw_lines` once,
and every iteration unconditionally appends something to `out`. `CasingEdit`/`SpacingEdit` only
ever replace a character range *within* a line; neither can represent "this physical line does
not appear in the output at all."

**Decision**: add a `lines_to_delete: BTreeSet<u32>` computed before the main emission loop, and
one new early-exit check at the top of that loop: `if lines_to_delete.contains(&line_num) {
continue; }`. This is a small, surgical addition — the smallest of the three "new capability"
findings across this project's three formatting-axis features so far (`017` needed none, `018`
needed variable-length edits, this needs line-skipping) — and composes cleanly with everything
else already computed against *original* line numbers (indentation plan, casing edits, spacing
edits, protected regions): none of them need to change at all, since deletion is applied as a
final filter over otherwise-unmodified per-line output, not a renumbering.

## §2: "Blank" matches this codebase's own whitespace convention, not Rust's general one

Every existing whitespace check in `format.rs` (leading-indent detection, trailing-whitespace
handling) is an explicit `c == ' ' || c == '\t'` comparison, never Rust's general
`char::is_whitespace()`. A line is "blank" (empty or whitespace-only) under this feature using
the same explicit convention — `line.chars().all(|c| c == ' ' || c == '\t')` (vacuously true for
a genuinely empty line) — for consistency with how every other line-classification decision in
this module already works, not because of any known real-world case where the distinction
(Unicode whitespace vs. ASCII space/tab) would matter for this corpus.

## §3: A blank-line run can never straddle a block boundary or a protected-region boundary

A block's closer (`ENDIF`/`ENDLOOP`/etc.) and an `; FMT: OFF`/`; FMT: ON` marker are both
non-blank lines (they contain real text — a keyword or a comment). Since a maximal run of blank
lines is, by definition, bounded on both sides by non-blank lines, a run can never contain *part*
of a block interior and *part* of the surrounding top-level content, and can never contain *part*
of a protected region and *part* of unprotected content. This means every blank-line run's
classification (top-level vs. nested) and protection status (protected vs. not) is **uniform
across the whole run** — checking any one line in the run (e.g. its first) is sufficient; there
is no straddling case to handle specially.

## §4: "Any depth" nested classification needs no recursion — a top-level block's own span already covers it

A nested block's `span` is, by construction, always entirely contained within its parent
block's `span` (structural elements cannot extend past their enclosing block's boundaries — this
is guaranteed by `block.rs`'s own matching logic, not assumed). This means marking every line in
`[top_level_block.span.start.line + 1, top_level_block.span.end.line]` as "nested," for every
`Node::Block` in the **top-level** `nodes` slice only (no recursion into `children`/branches
needed at all), already correctly classifies every line at every depth — a line three levels
deep is still within its outermost enclosing top-level block's own span range. This mirrors
`017`'s and `018`'s own recurring finding that this crate's existing tree shape already gives a
feature more "for free" than a first read of the requirement suggests (`017`'s dot-notation
boundaries, `018`'s sibling-adjacency-is-depth). A line not covered by any top-level block's
range is top-level by elimination — no separate "is this top-level" check is needed.

## §5: Contraction keeps the first N lines of a run, never renumbers or reflows anything else

FR-006 requires the surviving lines to be exactly the run's own first N lines, left byte-for-byte
as written — this is a pure deletion of the run's trailing `(length - cap)` lines, never a
rewrite of the survivors' own content (even a whitespace-only survivor keeps its original
whitespace, not trimmed to zero-length) and never a change to anything outside the run itself.
