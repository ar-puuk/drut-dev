# Feature Specification: `@name@` Variable Highlight Color Customization

**Feature Branch**: `027-named-variable-highlight`

**Created**: 2026-08-18

**Status**: Draft

**Input**: User description: "Add `drut.highlight.namedVariables` for `@name@`
substitution — the one category `026-highlight-customization` deliberately excluded
because it's governed by a separate, already-shipped mechanism
(`ensureVariableColorCustomization`, semantic-token-based, workspace-scoped, one-time,
hardcoded `#4EC9B0` default) rather than the TextMate-scope mechanism the other 9
categories use."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A script author recolors `@name@` substitutions to their own preference (Priority: P1)

A script author wants `@name@` references to render in a specific color — matching
their theme better, or standing out more for readability. They set
`drut.highlight.namedVariables` in Settings UI, exactly the same way they'd set any of
`026`'s 9 other `drut.highlight.*` settings, and every `@name@` reference immediately
renders in that color, in every workspace, without a reload.

**Why this priority**: This is the entire content of the request.

**Independent Test**: Set `drut.highlight.namedVariables` to a color, open a `.s` file
containing `@name@`, confirm it renders in that color.

**Acceptance Scenarios**:

1. **Given** `drut.highlight.namedVariables` is unset, **When** a `.s` file containing
   `@name@` is open in a workspace that has never activated this extension before,
   **Then** it renders in today's existing default (`#4EC9B0`) — byte-identical to
   `026`'s own shipped behavior, unaffected by this feature's presence.
2. **Given** `drut.highlight.namedVariables` is set to `#FF0000`, **When** a `.s` file
   containing `@name@` is open, **Then** it renders in `#FF0000`, without a window
   reload.
3. **Given** `drut.highlight.namedVariables` was set and is now cleared, **When** the
   file is viewed again, **Then** `@name@` reverts to the documented default
   (`#4EC9B0`) — not stuck at the last-configured color, and not left with no override
   at all (Assumptions: a full "no override" state would reintroduce the
   invisible-under-some-themes bug the original mechanism exists to fix).
4. **Given** a workspace where a user manually deleted the auto-seeded `variable:drut`
   rule from `.vscode/settings.json` (today's documented "sticks deleted forever"
   escape hatch) and has never touched `drut.highlight.namedVariables`, **When** the
   extension activates again, **Then** the rule is NOT re-added — this feature does not
   regress that existing guarantee for anyone who doesn't use the new setting.

---

### Edge Cases

- What happens if the user hand-edits the `variable:drut` rule's value directly in
  `.vscode/settings.json` (not via `drut.highlight.namedVariables`) while
  `drut.highlight.namedVariables` is unset? Left alone, exactly as today — this
  feature's live-sync only activates once `drut.highlight.namedVariables` has an
  explicit value; while unset, behavior is byte-identical to `026`'s shipped
  `ensureVariableColorCustomization` (Assumptions).
- What happens on the very first activation ever, in a brand-new workspace, with
  `drut.highlight.namedVariables` already set beforehand (e.g. synced from another
  machine)? The very first seed uses the configured color directly, not the
  `#4EC9B0` default followed by a second corrective write.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The extension MUST expose `drut.highlight.namedVariables`, following the
  exact same personal-setting shape (`026`'s FR-001/FR-009/FR-010: optional string,
  default unset, Global scope read only).
- **FR-002**: When `drut.highlight.namedVariables` has an explicit Global value, the
  extension MUST keep the current workspace's `variable:drut` rule in
  `editor.semanticTokenColorCustomizations` continuously synced to that value —
  live, on activation and on every relevant settings change, the same reactivity
  `026`'s other 9 categories already have.
- **FR-003**: `editor.semanticTokenColorCustomizations`'s `variable:drut` rule MUST be
  written at **Workspace** scope specifically — a deliberate, documented exception to
  `026`'s FR-010 "Global only" rule (see Assumptions: VS Code resolves an
  object-valued setting per-scope, not as a cross-scope deep merge, and the
  pre-existing default seed already lives at Workspace scope in any previously-
  activated workspace; writing at Global would be silently masked there).
- **FR-004**: When `drut.highlight.namedVariables` is unset, this feature MUST NOT
  change `026`'s/the original `ensureVariableColorCustomization` behavior in any way
  for a workspace that has never had `drut.highlight.namedVariables` set — the
  existing one-time-seed-then-never-touch-again lifecycle (including "a manual
  deletion sticks forever") is preserved exactly.
- **FR-005**: When `drut.highlight.namedVariables` transitions from set to unset (in a
  workspace where this feature's live-sync was previously active), the extension MUST
  perform exactly one corrective write reverting the rule to the documented default
  (`#4EC9B0`) — not remove the rule outright (Assumptions: a fully theme-driven state
  is not safe for this specific category) and not leave it stuck at the last
  configured color.
- **FR-006**: This feature MUST NOT modify `026`'s other 9 `drut.highlight.*`
  categories or their `editor.tokenColorCustomizations` mechanism in any way — purely
  additive, a 10th category using a different (but still VS-Code-native) mechanism.
- **FR-007**: This feature MUST NOT change `voyager-core`'s tokenizer, parser,
  formatter, `drut-lsp`'s semantic-tokens *emission* logic (still unconditionally
  emitting a standard `variable` token for every `@name@`), or any `Diagnostic`
  category — purely an `editors/vscode` client-side concern, same as `026`.

### Key Entities

- **`drut.highlight.namedVariables` → `variable:drut`**: the 10th
  `drut.highlight.<category>` entry, using `editor.semanticTokenColorCustomizations`
  (Workspace scope) instead of `026`'s `editor.tokenColorCustomizations` (Global
  scope) — the one category where the underlying VS Code mechanism differs, because
  the underlying rendering layer (`drut-lsp`'s semantic tokens) differs.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Setting `drut.highlight.namedVariables` visibly recolors `@name@`
  references without a window reload, in every tested scenario.
- **SC-002**: A workspace that never sets `drut.highlight.namedVariables` sees
  zero behavior change from `026`'s already-shipped `ensureVariableColorCustomization`
  — including the "manual deletion sticks forever" guarantee, verified explicitly
  (Acceptance Scenario 4).
- **SC-003**: Unsetting `drut.highlight.namedVariables` after it was set reverts to
  the documented default color, never leaves the rule stuck at a stale custom color,
  and never removes the override outright.

## Assumptions

- **Why Workspace scope, not Global**: VS Code resolves `editor.
  semanticTokenColorCustomizations` (and `editor.tokenColorCustomizations`) per-scope
  as a whole value, not as a cross-scope deep merge — Workspace, when set, wins over
  Global entirely for that setting. `026`'s already-shipped
  `ensureVariableColorCustomization` seeds its default at Workspace scope in every
  workspace this extension has ever activated in. Writing this feature's live-synced
  override at Global scope would therefore be silently invisible in any such
  already-seeded workspace — Workspace scope is not a stylistic choice here, it's
  required for the feature to actually work in the common case.
- **Why unsetting reverts to a default rather than removing the rule**: `026`'s
  research.md §3 already recorded why the `variable:drut` rule exists at all — real
  manual testing found some themes render `variable.other.readwrite.drut` (the
  TextMate scope) invisibly, a gap only the semantic-token layer's rule closes. A
  fully theme-driven state (no rule at all) would reintroduce that invisibility for
  those themes. This is a deliberate, evidenced exception to the "unset means no
  override at all" rule `026`'s other 9 categories follow.
- **Why the original one-time-seed lifecycle is preserved for untouched workspaces**:
  changing it (e.g. making every workspace live-synced unconditionally) would mean a
  user who manually deleted the seeded rule — an explicitly documented, intentional
  escape hatch in `026`'s own research — would find it silently reappear on the next
  activation, a real regression. This feature only enters "live" behavior once the
  user has explicitly set `drut.highlight.namedVariables` at least once.
- Scope is `editors/vscode` only — same as `026`. No `drut.toml`, CLI, or MCP surface.
