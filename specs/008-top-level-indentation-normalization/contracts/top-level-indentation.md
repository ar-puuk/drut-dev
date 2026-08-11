# Contract: Top-Level Indentation Normalization

Amends `002-cli-check-format/spec.md`'s FR-012 and
`002-cli-check-format/contracts/formatting-api.md` in place (matching how
`006` amended `001`'s diagnostics contract, and `007` amended `002`'s
FR-012 for the diagnosed-block-skip behavior) — not a new, competing
contract file.

## `spec.md` FR-012 bullet — replaced

**Before**:
> **Top-level (depth-0) statement indentation is left untouched.** The
> corpus shows no dominant convention here (best single value only 26.9%,
> at column 8; only 20.4% sit at column 0) — `format` normalizes the
> increment added per nested level, never a file's own top-level
> baseline.

**After**:
> **Top-level (depth-0) statement indentation is always normalized to
> column 0**, on every format pass, unconditionally — regardless of the
> statement's current indentation or formatting history. **Amended
> 2026-08-11 (`008-top-level-indentation-normalization`)**: this reverses
> the original policy above. The original corpus finding (only 20.4% of
> real top-level statements at column 0, modal value at column 8) is
> historical record, not disproven — the project has deliberately traded
> preserving that real-author diversity for predictability (the same
> stance most languages with enforced structure take), knowing this
> reformats a majority of real top-level statements in the reference
> corpus on first run.

## `formatting-api.md` — replaced

**Before**: `"... top-level baseline and continuation-line indentation
left untouched, comments left entirely untouched) ..."`

**After**: `"... top-level baseline always normalized to column 0,
continuation-line indentation left untouched, comments left entirely
untouched) ..."` — continuation-line handling is unaffected by this
feature (spec.md's Edge Cases don't touch it; no corpus-evidence question
was reopened for that dimension).

## Algorithm (normative, research.md §1)

```text
plan_indentation(nodes, lines, diagnosed_openers, plan):
  for each top-level node in nodes:
    plan[node's own line] = 0        # NEW — unconditional, every node
    if node is a Block:
      plan_block(node, lines, diagnosed_openers, plan)   # unchanged
```

`plan_block`, `plan_children`, `computed_indent` are **not modified** —
`computed_indent`'s existing "prefer a planned value over the original"
fallback is what makes the single new line above sufficient.

## `007`'s skip — role narrowed, not removed (research.md §1, FR-004's resolution)

`diagnosed_block_openers`/`plan_block`'s skip-a-diagnosed-block's-children
logic is **retained, unchanged in code**. Its own doc comments are updated
to state the narrowed rationale: it never protected the opener line (the
new unconditional top-level rule now owns that, independently and more
robustly — proven against a stale-indentation case `007` alone never
would have corrected, research.md §1's table); it only ever protects a
diagnosed block's *children*, whose structural relationship to that block
remains genuinely uncertain regardless of what column the opener sits at.
