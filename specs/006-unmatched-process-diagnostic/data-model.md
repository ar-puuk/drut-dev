# Phase 1 Data Model: UnmatchedProcess Diagnostic

One new enum variant, zero new types, zero changes to any existing type's
shape. Documented here for consistency with every prior feature's
data-model.md, not because this feature has meaningful entity complexity.

## `DiagnosticKind::UnmatchedProcess` (new variant)

Added to the existing `voyager-core::DiagnosticKind` enum
(`crates/voyager-core/src/diagnostic.rs`), alongside the six existing
variants (`UnmatchedIf`, `UnmatchedLoop`, `UnclosedBlockComment`,
`InvalidContinuation`, `UnmatchedRun`, `MisplacedBreak`) and
`InvalidEncoding` — no field, no associated data, matching every existing
variant's own shape (`DiagnosticKind` is a bare, data-less enum;
`Diagnostic` itself, not the kind, carries `span`/`message`).

| Property | Value |
|---|---|
| Carries data? | No — bare enum variant, same as every sibling |
| `Diagnostic.span` when this kind fires | The `PROCESS`/`PHASE=` opener statement's own span (research.md §4) |
| `Diagnostic.message` when this kind fires | Fixed original-wording string (research.md §4) |

## Explicitly unchanged

- `Block` (`crates/voyager-core/src/block.rs`) — `closer: Option<Span>`,
  `BlockKind::Process { name: Option<String> }`: no field added, no
  variant added. This feature adds a diagnostic *signal* derived from
  information `parse_process` already computes; it does not add new
  information to the structural tree itself (spec.md Assumptions).
- `DiagnosticDto` (`crates/drut-mcp/src/diagnose.rs`) — same four fields
  (`category`, `message`, `start_line`/`start_column`/`end_line`/
  `end_column`); `category` simply gains one more possible string value.
- SARIF output shape (`crates/drut-cli/src/report/sarif.rs`) — same
  `SarifLog`/rule/result structs; the rule catalog gains one more entry,
  same shape as the other seven.
