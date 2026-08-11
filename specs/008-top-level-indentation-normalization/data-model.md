# Phase 1 Data Model: Top-Level Indentation Normalization

No new types, no changed type shapes. This is a pure behavioral change to
one existing function (`plan_indentation`, `crates/voyager-core/src/
format.rs`) — documented here for consistency with every prior feature's
data-model.md, not because this feature has entity complexity.

## Explicitly unchanged

- `IndentPlan` (`BTreeMap<u32, usize>`) — same type; this feature only
  changes *which* entries get inserted into it (every top-level line now
  gets one, unconditionally), not its shape.
- `Block`, `Node`, `Diagnostic`, `DiagnosticKind` — untouched.
- `diagnosed_block_openers`'s own return type (`BTreeSet<Position>`) and
  logic — unchanged; only its consumer's (`plan_block`'s) role narrows in
  meaning, not in code (research.md §1).
- `FormatResult`, `FormatOptions` — unchanged; this feature changes what
  `FormatResult.text`/`changed` compute to for top-level content, not
  either type's shape.
