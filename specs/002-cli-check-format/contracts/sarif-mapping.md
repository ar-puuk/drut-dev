# Contract: `DiagnosticKind` → SARIF Mapping

This is the presentation-layer mapping `drut check --format=sarif` uses to turn
`001-voyager-script-parser/contracts/diagnostics.md`'s `DiagnosticKind` values into
a SARIF 2.1.0 log (FR-009). It introduces no new diagnostic semantics — every row
below is a rendering decision, not a grammar/parsing rule, and lives in `drut-cli`
(specifically `src/report/sarif.rs`), not `voyager-core`.

## `run.tool.driver`

| Field | Value |
|---|---|
| `name` | `"drut"` |
| `informationUri` | The project's repository URL (left to implementation; not a schema-required field for validity) |
| `version` | The `drut-cli` crate's own version at build time |
| `rules` | One `reportingDescriptor` per `DiagnosticKind` variant below, declared once per SARIF run regardless of whether that kind fired in this particular run — so a SARIF viewer can show a rule's description even for a clean run |

## `ruleId` / `level` mapping

| `DiagnosticKind` | `ruleId` | SARIF `level` | Notes |
|---|---|---|---|
| `UnmatchedIf` | `unmatched-if` | `error` | |
| `UnmatchedLoop` | `unmatched-loop` | `error` | |
| `UnclosedBlockComment` | `unclosed-block-comment` | `error` | |
| `InvalidContinuation` | `invalid-continuation` | `error` | |
| `UnmatchedRun` | `unmatched-run` | `error` | |
| `MisplacedBreak` | `misplaced-break` | `error` | |
| `InvalidEncoding` | `invalid-encoding` | `error` | Only ever reachable via `parse_bytes`/`format_bytes`, same as its source contract (FR-034 in `001-voyager-script-parser`) |

**Rationale for uniform `level: "error"`**: `001-voyager-script-parser/contracts/
diagnostics.md` deliberately defines no severity levels at the `Diagnostic` layer —
every kind is a structural syntax defect or a decoding fallback of last resort, not
a heuristic lint finding. `error` is the closest SARIF level to "a structural defect
was found," and reserves SARIF's `warning`/`note` levels for a future phase's
heuristic lint rules, which per constitution Principle IV ship as warnings until
validated against the corpus with zero false positives — a distinction this phase's
diagnostics don't need, since they aren't heuristic.

**Validation rule**: `ruleId` values are kebab-case, English-language slugs of the
`DiagnosticKind` variant name — original wording, not copied from any vendor source
(constitution Principle II) — and MUST stay stable across releases, since SARIF
consumers (e.g. GitHub code scanning) key suppression/tracking state off `ruleId`.

## `result` mapping (per diagnostic)

| SARIF field | Source |
|---|---|
| `ruleId` | Table above, from the `Diagnostic.kind` |
| `level` | Table above |
| `message.text` | `Diagnostic.message` (already original wording per FR-024 in `001-voyager-script-parser`) |
| `locations[0].physicalLocation.artifactLocation.uri` | The matched file's path, relative to the invocation's target path where possible |
| `locations[0].physicalLocation.region.startLine` / `startColumn` | `Diagnostic.span.start.line` / `.column` (1-based, matching `Position`'s own convention) |
| `locations[0].physicalLocation.region.endLine` / `endColumn` | `Diagnostic.span.end.line` / `.column` |

## Consumers must not assume this mapping is closed

`001-voyager-script-parser/contracts/diagnostics.md` explicitly reserves the right
to add new `DiagnosticKind` variants later within the same non-semantic scope. This
mapping table MUST be extended with a corresponding row whenever that happens —
an unmapped `DiagnosticKind` reaching the SARIF renderer is a bug in this crate, not
a case the renderer should silently drop or panic on (FR-023).
