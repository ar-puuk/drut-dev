# Data Model: Range-Dash Spacing Exemption

No new `voyager-core` public types — `FormatOptions`/`OperatorSpacing` are unchanged (FR-008: no
new configuration surface). This amends `operator_spacing.rs`'s existing internal recognition
logic only.

## §1. `operator_spacing` module (modified)

```rust
/// Whether a binary `-` occurrence gets Fixed/Auto's ordinary one-space
/// arithmetic spacing, or the range-dash exemption's zero-space spacing
/// (research.md §1). Decided once per occurrence, at recognition time —
/// never re-derived at edit-application time.
enum DashSpacing {
    Arithmetic,   // existing behavior: 1 space on each side
    Range,        // this feature: 0 spaces on each side
}
```

- `OperatorOccurrence` (existing) gains no new field — instead, `recognize_operators` computes
  each binary `-` occurrence's `DashSpacing` inline and threads it through to
  `collect_operator_edits`'s `push_gap_edit` calls as the `want_spaces` argument, replacing the
  hard-coded `1` those calls use for every other operator kind today. Every non-`-` operator kind,
  and every `-` that doesn't qualify as `Range`, keeps calling `push_gap_edit` with `1`, unchanged.
- A binary `-` occurrence qualifies as `Range` (research.md §1–§3) when **all** of:
  1. The enclosing statement's `StatementKind` is `Control` (never `Assignment`, never inside an
     `IF`/short-`IF` condition, `LOOP` bound, or any other expression context — those are never
     pair-keyword values).
  2. The occurrence's own token index falls inside some pair's value range, as derived from
     `pair_keyword_boundaries(&stmt.tokens)` (research.md §2) — the same boundary data
     `collect_comma_edits` already derives for the identical statement.
  3. The token immediately before the occurrence (`tokens[start_index - 1]`) and the token
     immediately after it (`tokens[end_index]`) are each a bare integer literal (research.md §3):
     `TokenKind::Word` with `text` non-empty and every character an ASCII digit.
- A `-` that is already unary (per `018`'s existing `is_binary_arithmetic`) never becomes an
  `OperatorOccurrence` in the first place, so it is never evaluated against the `Range` condition
  at all — FR-005's ordering requirement is satisfied by the existing control flow, not a new
  check this feature adds.

## §2. Edit application

No change to `SpacingEdit`'s shape or to `render()`'s line-application logic (both from `018`'s
own data-model.md §2) — a `Range`-classified occurrence produces exactly the same *kind* of edit
(a `push_gap_edit` call) as an `Arithmetic` one, just with `want_spaces = 0` instead of `1`. The
existing "no-op when the gap is already the target width" guard inside `push_gap_edit` (data
already correct → no edit queued) applies identically, which is what makes a range already
written tight (`1-50`) produce zero edits and an already-formatted script byte-identical on a
second pass (SC-004's idempotence requirement).

## §3. Interaction with comma spacing and alignment

- **Comma spacing** (`collect_comma_edits`, unchanged): operates on pair-*boundary* comma gaps
  only (research.md §4) — disjoint from the `-` gaps this feature touches, even within the same
  pair-keyword value (`1-50,75,90-100`'s three list-item commas are never pair-boundary commas,
  since there is only one `kw_start` on that statement).
- **Alignment** (`Auto`'s `collect_alignment_edits`, unchanged): only ever considers
  `Node::Statement(Assignment)` siblings' own `=` positions (`018` FR-007) — a `Control`
  statement's pair-keyword value, and therefore every range dash inside it, is never a candidate
  for alignment padding regardless of this feature. No interaction to reconcile.
