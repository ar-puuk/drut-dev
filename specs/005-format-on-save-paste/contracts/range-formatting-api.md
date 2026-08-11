# Contract: `drut-lsp` — `textDocument/rangeFormatting`

## Capability declaration

`server_capabilities()` (`crates/drut-lsp/src/lib.rs`) gains:

```rust
document_range_formatting_provider: Some(lsp_types::OneOf::Left(true)),
```

alongside the existing `document_formatting_provider` line — same shape,
whole-range-only (no on-type variant), matching this feature's scope
exactly (spec.md explicitly excludes format-on-type/anything beyond
save/paste).

## Request dispatch

`handle_request` (`crates/drut-lsp/src/lib.rs`) gains a
`RangeFormatting::METHOD` arm, structurally identical to the existing
`Formatting::METHOD` arm:

```rust
RangeFormatting::METHOD => match serde_json::from_value::<lsp_types::DocumentRangeFormattingParams>(req.params) {
    Ok(params) => send_ok(connection, id, &range_formatting::handle(state, &params)),
    Err(e) => send_err(connection, id, e.to_string()),
},
```

## Handler contract

```rust
pub fn handle(
    state: &ServerState,
    params: &lsp_types::DocumentRangeFormattingParams,
) -> Option<Vec<lsp_types::TextEdit>>
```

| Input condition | Output |
|---|---|
| `params.text_document.uri` has no open document in `state` | `None` (FR-009 — matches `formatting.rs`'s existing `unopened_document_returns_none` behavior exactly, not a new convention) |
| Document is open; `voyager_core::format` reports `changed: false` for the whole document | `Some(vec![])` — an empty edit list, same "already formatted, nothing to do" convention `formatting.rs` already uses (not `None`, which would mean "no formatter opinion exists at all") |
| Document is open; `changed: true`, but no changed line falls within `params.range` | `Some(vec![])` — a real, structurally-correct empty result: the requested range itself needed no correction, even though the document as a whole did (data-model.md §1's `filter_to_range` returning empty) |
| Document is open; `changed: true`, and at least one changed line falls within `params.range` | `Some(edits)` — one `TextEdit` per changed line within range, each `range` covering that single line (start of line to start of next line, or end-of-document for the last line) and `new_text` set to `LineEdit.new_content` plus its line terminator |

## Algorithm (research.md §2)

1. `let result = voyager_core::format(&doc.text, voyager_core::FormatOptions::default());`
   — identical call `formatting.rs` already makes; casing stays untouched
   for the same reason (spec.md Assumptions: no configuration surface for
   LSP-triggered formatting yet).
2. If `!result.changed`, return `Some(vec![])` immediately — no diffing
   needed.
3. `let line_edits = diff_lines(&doc.text, &result.text);` (data-model.md
   §1) — exact, line-count-preserving comparison, never a generic diff
   algorithm.
4. `let in_range = filter_to_range(line_edits, params.range);`
   (data-model.md §1).
5. Translate each surviving `LineEdit` into an `lsp_types::TextEdit`
   spanning exactly that one line (via `position.rs`'s existing
   `to_lsp_position`/`to_lsp_range` helpers — no new position-translation
   logic).

No step above performs any parsing, block-matching, or indentation
decision itself — every judgment about *what* the correct formatted output
is comes from the single `voyager_core::format` call in step 1 (Principle
I; Constitution Check row I).

## Tests (mirrors `formatting.rs`'s existing test module shape)

- `misindented_line_within_range_is_corrected` — direct analog of
  `formatting.rs`'s `misindented_body_statement_is_corrected_relative_to_its_opener`,
  with a `range` covering the misindented line.
- `already_formatted_document_returns_empty_edit_list` — direct analog of
  `formatting.rs`'s `already_formatted_document_returns_no_edits`.
- `unopened_document_returns_none` — direct analog of `formatting.rs`'s
  identically-named test.
- `change_outside_requested_range_is_not_returned` — a document with two
  *unrelated*, independently-misindented lines, `range` covering only one
  of them; asserts the edit list contains exactly one entry, for the
  in-range line. Deliberately the cheap, unrelated-lines baseline — see
  the next two cases for the causally-connected (block-boundary) shape,
  which this case alone does not exercise.
- `change_at_exact_range_boundary_is_included` — asserts the inclusive-
  boundary behavior (data-model.md §1) at `range.start.line`/`range.end.line`
  exactly, not just clearly-inside/clearly-outside cases.
- `paste_that_opens_a_block_only_returns_the_in_range_edit` — the
  block-boundary case research.md §2's "Edge case resolution" describes,
  verified directly against `voyager_core::format` rather than hand-derived
  (research.md §2's own worked example). Input (7 lines, pasting a lone
  `IF (c=3)` opener — no closer within the paste — between two body
  statements of an already-open block, `range` = line 3 only):
  ```text
  IF (a=1)
      IF (b=2)
          PRINT LIST=1
  IF (c=3)
          PRINT LIST=2
      ENDIF
  ENDIF
  ```
  Whole-document reformat (what `voyager_core::format` actually produces)
  changes **four** lines — 3 (the pasted opener, 0→8 spaces), and three
  lines strictly outside the range: 4 (8→12 spaces), 5 (4→8 spaces), 6
  (0→4 spaces) — plus reports one `UnmatchedIf` diagnostic (on line 0's
  `a=1`, now permanently unclosed: opening a block without its own closer
  inside the pasted range necessarily "steals" a pre-existing closer
  further down to close the *new* block first, innermost-first, leaving
  the outermost opener unmatched — see research.md §2's own note on why
  this is a general structural property, not specific to this example).
  Asserts the range-formatting response contains **exactly one** edit —
  line 3 only — even though the whole-document pass needed four.
- `paste_that_closes_a_block_only_returns_the_in_range_edit` — the mirror
  case: pasting a lone `ENDIF` (no opener within the paste) between the
  same two body statements, closing the inner block early instead of
  opening a new one. Input (`range` = line 3 only):
  ```text
  IF (a=1)
      IF (b=2)
          PRINT LIST=1
  ENDIF
          PRINT LIST=2
      ENDIF
  ENDIF
  ```
  Whole-document reformat changes **three** lines — 3 (the pasted closer,
  0→4 spaces), and two lines strictly outside the range: 4 (8→4 spaces),
  5 (4→0 spaces) — plus one `UnmatchedIf` diagnostic (on the now-stray
  final `ENDIF`, line 6, which needs no reindent itself but has nothing
  left open to close). Asserts the range-formatting response contains
  **exactly one** edit — line 3 only.

These two cases are the same underlying phenomenon viewed from opposite
ends (research.md §2): a single unbalanced paste into an otherwise-valid
document always "steals" a pre-existing opener or closer elsewhere,
producing exactly one transient `UnmatchedIf`-shaped diagnostic alongside
the reindentation ripple — confirmed here empirically, not assumed, and
true in both directions. Both fixtures preserve line count end to end (7
lines in, 7 lines out) — the concrete case research.md §2's line-count-
preservation guarantee is tied to explicitly.
