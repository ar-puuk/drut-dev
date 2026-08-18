# Contract: Range-Dash Spacing Exemption (amendment to `018-operator-spacing`)

Extends `018-operator-spacing/contracts/operator-spacing.md`, which itself extends
`001-voyager-script-parser/contracts/public-api.md` and
`002-cli-check-format/contracts/formatting-api.md`. A conceptual signature contract, not final
Rust source — same convention every prior contract doc in this repo follows.

## No public API change

- `voyager-core::FormatOptions`, `OperatorSpacing`, and the `format`/`format_bytes` function
  signatures are **entirely unchanged** — this is a corrected behavior under the already-shipped
  `operator_spacing: Fixed | Auto` values, not a new setting (FR-008). No `drut-config` field, no
  CLI flag, no MCP parameter, no VS Code setting is added, renamed, or removed by this feature.

## Behavior contract

- **Scope of the exemption**: a binary `-` inside a `Control` statement's pair-keyword value,
  with a bare integer literal (a token of only ASCII digits — no `@token@` reference, no decimal
  point, no other identifier) directly adjacent on both sides, renders with **zero** surrounding
  whitespace under `Fixed`/`Auto` — stripping any existing spacing, not merely leaving it as
  written.
- **Scope boundary — everywhere else unchanged**: a `-` that is unary (per `018`'s existing
  rule), that sits outside a pair-keyword's value (an `Assignment` RHS, an `IF`/short-`IF`
  condition, a `LOOP` bound, or any other expression context), or that has a non-integer token on
  either side, keeps `018`'s existing binary-arithmetic spacing (exactly one space each side)
  entirely unchanged.
- **`Preserve` is untouched**: a project with `operator_spacing` unset or `preserve` sees
  byte-identical output to before this feature existed, exactly as `018`'s own `Preserve`
  guarantee already states.
- **Composes with existing `018` rules, never overrides them**: `018`'s comma-spacing rule keeps
  its existing scope exactly — a comma *inside* a single pair's own value list (e.g. the three
  commas in `1-50,75,90-100`) was never touched by that rule before this feature and still isn't
  (the same existing behavior that already leaves `LOOP i=1,5,1`'s internal commas alone); a
  pair-*boundary* comma sitting immediately next to a range-dash value (e.g.
  `FILEO NODES=1-50 ,SELECTLINK=75 - 100`) is normalized by the comma rule and has its adjacent range(s)
  tightened by this feature, independently, in the same pass. `Auto`'s alignment pass never
  considers pair-keyword values at all (`018` FR-007), so there is no interaction to reconcile
  there either.
- **Idempotent**: a range already written tight (`1-50`) produces no edit and is byte-identical
  on a second formatting pass, the same guarantee every other `018` rule already provides.
- **No panics, determinism, behavior preservation**: every guarantee
  `002-cli-check-format/contracts/formatting-api.md` and `018`'s own contract already make for
  `format`/`format_bytes` holds unchanged, re-verified (not assumed) for this amendment
  specifically.

## Illustrative examples (not exhaustive — see spec.md's Acceptance Scenarios)

Every `FILEO ...=` example below also shows the pair's own `=` getting `018`'s ordinary,
unrelated one-space treatment — that's pre-existing `018` behavior, not part of this feature; only
the `-` handling is new here.

| Input (`operator_spacing = fixed`) | Output | Why |
|---|---|---|
| `FILEO SELECTLINK=1-50,75,90-100` | `FILEO SELECTLINK = 1-50,75,90-100` | Both `-`s: pair-keyword value, integer both sides → tight; commas are same-pair-internal, outside both rules' scope |
| `FILEO NODES=200 - 300` | `FILEO NODES = 200-300` | Same rule; existing spacing is stripped, not preserved |
| `X = 100-1` | `X = 100 - 1` | `Assignment`, not a pair-keyword value → unchanged `018` spacing |
| `IF (COUNT-1 == 0)` | `IF(COUNT - 1 == 0)` | Condition, not a pair-keyword value → unchanged `018` spacing |
| `FILEO SELECTLINK=@START@-50` | `FILEO SELECTLINK = @START@ - 50` | Non-integer operand → unchanged `018` spacing |
| `FILEO OFFSET=-100,50` | `FILEO OFFSET = -100,50` | Unary `-`, never a binary occurrence at all |
| `FILEO NODES=1-50 ,SELECTLINK=75 - 100` | `FILEO NODES = 1-50, SELECTLINK = 75-100` | Pair-boundary comma (`018`'s rule) and two range dashes (this feature) apply independently in one pass |
