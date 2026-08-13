# Quickstart: Validating Code Folding Support

A runnable validation guide, not an implementation walkthrough — proves this feature
against spec.md's Success Criteria. See `contracts/folding-range-api.md` for the exact
contract and `research.md` for the full design rationale, including the direct answer
to whether this reaches into `voyager-core` (§1: yes, minimally and additively).

## Prerequisites

- Rust stable toolchain.
- A local checkout of the WF-TDM-Official-Releases corpus (`$CORPUS`).
- VS Code, for the manual smoke test (step 5).

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` unit tests — validates FR-002, FR-003, FR-004, FR-005

```powershell
cargo test -p voyager-core --test block_resolution
```

Expected: all green, including new tests for `all_blocks` covering: every one of the
7 block kinds with an explicit closer, an implicitly-closed `Run`, an implicitly-closed
`Process`, a short-`IF` (`counterpart == None`), a genuinely unmatched `If`/`Loop`/`Run`
(`counterpart == None`), and nested blocks (inner block's `BlockFold` present alongside
its enclosing outer block's).

## 3. `drut-lsp` unit tests — validates FR-002 through FR-008, FR-011

```powershell
cargo test -p drut-lsp --lib folding::
```

Expected: all green (12 tests), covering: every explicitly-closed and
implicitly-closed block kind produces a `Region`-kind range spanning opener→
counterpart line; a well-formed multi-line block comment produces a `Comment`-kind
range; a **single-line** block comment produces no range (FR-008 applied to the
comment stream); a short-`IF` and an unmatched block produce no range; an unclosed
block comment produces no range; nested blocks each get independent, correct
ranges; a document with zero blocks/comments returns `Some(vec![])`, not `None`;
and an unknown/unopened document URI returns `None`.

## 4. `drut-lsp` protocol tests — validates FR-001, FR-009, FR-010, US1, US2, US3

```powershell
cargo test -p drut-lsp --test protocol_smoke -- folding_range
```

Expected: all green (7 tests), including a direct `textDocument/foldingRange`
request/response round trip over `lsp_server::Connection::memory()` (same pattern
`hover.rs`'s and `formatting.rs`'s own protocol tests already use) proving all three
US1 Acceptance Scenarios, both US2 live-edit scenarios (a `didChange` that adds a
line inside a block's body shifts the reported `end_line` accordingly, proving no
stale/cached parse is reused; deleting a block's closer removes its range on the
next request), and US3's Fold-All coverage (the full returned set matches every
foldable construct in a multi-construct fixture exactly, with the single-line
comment and short-`IF` in that same fixture correctly excluded).

## 5. Manual verification in a real VS Code instance — validates SC-001, SC-002

1. Launch the extension development host (`F5` in `editors/vscode/`) against a real
   multi-block `.s`/`.block` script from `$CORPUS` containing at least one `IF`, one
   `LOOP`, one implicitly-closed `RUN`, and one block comment.
2. Confirm a fold control (gutter chevron) appears at each block's opener line and at
   the block comment's opening line; confirm none appears on any short-`IF` line.
3. Collapse a nested block; confirm the outer block's fold control still shows the
   inner block collapsed correctly when the outer fold is later expanded.
4. Run the editor's "Fold All" command; confirm every foldable block/comment
   collapses in one action. Run "Unfold All"; confirm the document returns to its
   original view.
5. Edit inside a currently-open (unfolded) block's body — add a line — and confirm
   re-collapsing the fold now hides the new line too (no reload required).

## 6. Full-corpus revalidation — validates SC-003

```powershell
$env:DRUT_CORPUS_PATH = "$CORPUS"
cargo test --release -p drut-lsp --test diagnostics_corpus -- --ignored
cargo test --release -p drut-lsp --test folding_corpus -- --ignored
```

Expected: still 161/161 clean on existing diagnostics-corpus assertions (this feature
adds no new diagnostic and changes no existing one — a pure regression check).
`folding_corpus` (added per tasks.md T012) proves SC-003 for **both** halves of its
"block or comment" wording, not just blocks (extended during `/speckit-analyze`
remediation — the original design only covered the first of the two below):

1. **Blocks**: for every one of the 161 files, every block with a resolvable
   counterpart in the existing hover/query-structure sense also has a matching
   folding range (cross-checked directly against `block_at` at each block's own
   opener position, i.e. the two `voyager-core` entry points — `all_blocks` and
   `block_at` — agree with each other on every real block in the corpus), and every
   block reported as unmatched by the corpus's existing diagnostics has no folding
   range.
2. **Block comments**: for every one of the 161 files, every terminated
   (`unterminated: false`) block-comment token spanning more than one line has
   exactly one folding range, and every unterminated or single-line block-comment
   token has none.

## 7. Full test suite

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2, 3, 4, 5 | SC-001 |
| 4, 5 | SC-002 |
| 6 | SC-003 |
| 4, 5 | SC-004 |
