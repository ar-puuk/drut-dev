# Phase 1 Data Model: Format-On-Save and Format-On-Paste

No persistent storage, no new `voyager-core` entity, no protocol-facing DTO
crate — this feature is small enough that its "data model" is really just
the shapes involved in two request/response cycles plus one piece of
extension-side state. Documented here for the same reason 003/004's
data-models were: so `/speckit-tasks` has concrete field-level detail to
generate tasks against.

## 1. `drut-lsp` — range-formatting request/response

Reuses `lsp_types::DocumentRangeFormattingParams`/`Vec<TextEdit>` directly
(research.md §1) — no new type defined in `drut-lsp` for the wire shape
itself, same as `formatting.rs`'s existing `DocumentFormattingParams`/
`Vec<TextEdit>` usage.

### `LineEdit` (internal, `range_formatting.rs`-local)

The one new internal type this feature adds — not exposed outside the
module, exists purely to make the line-diff step (research.md §2)
independently testable from the LSP-shape translation around it.

| Field | Type | Notes |
|---|---|---|
| `line_index` | `u32` | 0-based line number (LSP convention — matches `Range.start.line`/`end.line` directly, no translation needed for the comparison step) |
| `new_content` | `String` | The formatted line's full content, replacing the original line's content at `line_index` |

Derivation: `diff_lines(original: &str, formatted: &str) -> Vec<LineEdit>` —
walks both texts' `.lines()` in lockstep (safe per research.md §2's
line-count-preservation guarantee), emitting a `LineEdit` for every index
where the two disagree. A second, separate step,
`filter_to_range(edits: Vec<LineEdit>, range: lsp_types::Range) -> Vec<LineEdit>`,
keeps only entries whose `line_index` falls within
`[range.start.line, range.end.line]` — kept as its own function specifically
so it has its own direct unit test coverage of the boundary condition
(spec.md's Edge Case: a change just outside the requested range must never
leak through), independent of the diffing logic above it.

## 2. `editors/vscode` — format-on-save injection state

### `workspaceState` key: `drutFormatOnSaveInjected`

| Field | Type | Notes |
|---|---|---|
| (the key itself) | `boolean` | Mirrors `VARIABLE_COLOR_INJECTED_KEY`'s existing shape/lifecycle exactly — `true` once this workspace has been offered the injection (regardless of whether the write actually changed anything), never re-checked or re-cleared automatically |

### Injection decision inputs (not stored — computed fresh each activation)

| Source | Field | Notes |
|---|---|---|
| `vscode.workspace.getConfiguration(undefined, { languageId: "drut-voyager" }).inspect("editor.formatOnSave")` | `.workspaceLanguageValue` | `undefined` iff no language-scoped workspace override exists yet — the actual gate for whether to write (research.md §3), distinct from and more precise than the `workspaceState` key above, which only gates re-attempting on a later activation |

Two gates exist deliberately, not redundantly: `workspaceState` answers
"has this extension already tried, ever" (cheap, synchronous, no
config read needed on the common "already handled" path); `inspect()`
answers "does an explicit override exist right now" (the actual
correctness-critical check, consulted only on the first-ever activation
per workspace, before writing).
