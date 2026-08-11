# Contract: `UnmatchedProcess` Diagnostic

Amends `specs/001-voyager-script-parser/contracts/diagnostics.md` (the
single authoritative diagnostics contract) rather than superseding it
(research.md §5). This file specifies exactly what that amendment is and
gives every adapter its exact required change (research.md §1).

## `contracts/diagnostics.md` table addition

New row, directly under the existing `UnmatchedRun` row:

| Kind | Span | Condition | Traces to |
|---|---|---|---|
| `UnmatchedProcess` | The `PROCESS`/`PHASE=` statement's own location | A `PROCESS`/`PHASE=` block has no matching `ENDPROCESS`/`ENDPHASE` **and** no implicit closer (a following `PROCESS`/`PHASE=` statement) before end-of-input or before the enclosing block's own closer forces an early stop | FR-002 (this feature) |

## `contracts/diagnostics.md`'s "Note on block kinds without a diagnostic category" — rewritten

**Before** (current text): names `Process`/`JLoop`/`LinkLoop`/
`DistributeMultistep` together as all four lacking a category, with
coverage "explicitly left to a later phase."

**After**: `Process` is removed from that list (it now has
`UnmatchedProcess`, resolved by this feature, `006-unmatched-process-
diagnostic`) — the note is rewritten to name only `JLoop`/`LinkLoop`/
`DistributeMultistep` as still deferred, with an explicit pointer to this
feature as precedent for how that future decision could be made (real
corpus investigation first, diagnostic added only once empirically
zero-false-positive) — not silently dropped from the note as if it had
never applied to `Process`.

## Firing condition (normative, mirrors `UnmatchedRun` exactly)

```text
UnmatchedProcess fires iff:
  - the PROCESS/PHASE= block has no explicit ENDPROCESS/ENDPHASE, AND
  - the block has no following PROCESS/PHASE= statement (the legitimate
    implicit-close pattern) immediately after its body, AND
  - body-parsing stopped either at true end-of-input, or because the
    enclosing block's own closer (ENDIF/ENDLOOP/ENDRUN/etc.) was
    encountered first
```

Never fires when either the explicit or implicit closer condition holds —
both remain completely silent, exactly as today (spec.md FR-003/FR-004).

## Message and SARIF text (research.md §4)

- `Diagnostic.message`: `"this PROCESS/PHASE= has no matching
  ENDPROCESS/ENDPHASE and no following PROCESS/PHASE= statement before the
  end of the file"`
- SARIF `ruleId`: `"unmatched-process"`
- SARIF `shortDescription`: `"A PROCESS/PHASE= has no matching
  ENDPROCESS/ENDPHASE and no implicit closer."`

## Required adapter changes (research.md §1 — repeated here as the binding checklist)

| File | Change |
|---|---|
| `crates/voyager-core/src/diagnostic.rs` | Add `UnmatchedProcess` variant + doc comment |
| `crates/voyager-core/src/block.rs` | `parse_process` gains the firing logic (research.md §3) |
| `crates/voyager-core/tests/fixture_corpus.rs` | `parse_diagnostic_kind` match + `every_diagnostic_category_has_at_least_one_broken_fixture`'s array, one entry each |
| `crates/voyager-core/tests/fixtures/broken/` | New fixture, `unmatched_process_with_trailing_content.s` (FR-009) |
| `crates/drut-cli/src/report/sarif.rs` | `ALL_KINDS` (→ 8), `rule_id`, `short_description` — one entry each |
| `crates/drut-lsp/src/diagnostics.rs` | `kind_name` — one entry; module doc's kind-count updated |
| `crates/drut-mcp/src/diagnose.rs` | `category_name` — one entry; `DiagnosticDto` doc comment's kind-count updated |
| `crates/drut-cli/src/report/text.rs` | **No change** — Debug-formats `diag.kind` directly |
| `editors/vscode/src/*` | **No change** — no hand-listed diagnostic kinds anywhere |

Every "exhaustive match" file above will fail to compile if its entry is
missed — this is a completeness *guarantee* from Rust's own exhaustiveness
checking, not something that needs separate verification once the crate
builds clean.
