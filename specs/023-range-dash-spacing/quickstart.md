# Quickstart: Validating Range-Dash Spacing Exemption

A runnable validation guide, not an implementation walkthrough — proves this feature against
spec.md's Success Criteria. See `contracts/range-dash-spacing.md` for the exact behavior contract
and `data-model.md`/`research.md` for the full design rationale.

## Prerequisites

- Rust stable toolchain.
- `018-operator-spacing` already merged (this feature amends it, not a standalone module).

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` `operator_spacing` unit tests — validates FR-001–FR-006

```powershell
cargo test -p voyager-core operator_spacing
```

Expected: all green, including — note every `FILEO ...=` example's own `=` also gets `018`'s
ordinary, unrelated one-space treatment (pre-existing `018` behavior, not part of this feature):
- `FILEO SELECTLINK=1-50,75,90-100` renders `FILEO SELECTLINK = 1-50,75,90-100` (`fixed`) — every
  range in the list stays tight; its three commas are same-pair-internal and were never touched
  by `018`'s comma rule (FR-004) in the first place, the same as `LOOP i=1,5,1`'s own internal
  commas today.
- `FILEO NODES=1-50 ,SELECTLINK=75 - 100` renders `FILEO NODES = 1-50, SELECTLINK = 75-100` — the
  pair-boundary comma (`018`'s existing rule) and both range dashes (this feature) each normalize
  their own disjoint gap in the same pass.
- `FILEO NODES=200 - 300`, `FILEO NODES=200- 300`, and `FILEO NODES=200 -300` all render
  `FILEO NODES = 200-300` — the exemption actively strips existing spacing, it does not merely
  preserve it.
- `X = 100-1` (an `Assignment`, not a pair-keyword value) renders `X = 100 - 1` — unchanged `018`
  binary-arithmetic spacing.
- `IF (COUNT-1 == 0)` renders `IF(COUNT - 1 == 0)` — a condition is never a pair-keyword value.
- `FILEO SELECTLINK=@START@-50` renders `FILEO SELECTLINK = @START@ - 50` — a non-integer operand
  falls back to ordinary spacing, confirming FR-002's bare-integer-only scope.
- `FILEO OFFSET=-100,50` renders `FILEO OFFSET = -100,50` — the leading `-` is unary, never
  evaluated against this feature's condition at all.
- `FILEO THRESHOLD=1.5-2.5` renders `FILEO THRESHOLD = 1.5 - 2.5` — a decimal number is one
  `Word` token containing a `.`, never a bare integer literal (research.md §3).

## 3. Idempotence — validates SC-004

```powershell
cargo test -p voyager-core operator_spacing -- idempotent
```

Expected: formatting `FILEO SELECTLINK = 1-50,75,90-100` (already `fixed`-formatted — ranges
tight, pair `=` spaced) a second time produces zero edits and byte-identical output —
`push_gap_edit`'s existing "already correct" no-op guard applies to the new `want_spaces = 0`
target the same way it already does for `1`.

## 4. `preserve` mode — validates SC-003

```powershell
cargo test -p voyager-core operator_spacing -- preserve
```

Expected: every fixture from Step 2, formatted with `operator_spacing` unset or `preserve`,
renders byte-identical to its input — this feature changes nothing when `018`'s own default
applies.

## 5. Full workspace re-proof + real-corpus revalidation

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Run the full real fixture corpus through `drut format` with `operator_spacing=fixed` and
`operator_spacing=auto` — hand-verify any fixture containing a real Cube Voyager range-list value
(e.g. a `SELECTLINK`/`NODES`-shaped pair-keyword) now renders its ranges tight, with no other
diff introduced; promote any such diff to a new golden fixture, the same discipline `018` and
`019` already established for their own new golden variants. Confirm idempotence on the new
golden variant (the `check_idempotent` harness `format_corpus.rs` already runs for every other
configured variant).

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2 | SC-001, SC-002 |
| 3 | SC-004 |
| 4 | SC-003 |
| 5 | All (integration re-proof, real-corpus evidence for SC-001) |
