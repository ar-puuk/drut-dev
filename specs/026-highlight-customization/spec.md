# Feature Specification: Editor Highlight Color Customization

**Feature Branch**: `026-highlight-customization`

**Created**: 2026-08-18

**Status**: Draft

**Input**: User description: "In addition to the [format] toml option or drut.format
vscode option, offer a drut.highlight.* VS Code setting where users can specify the
color for syntax highlighting for different categories of text, while keeping today's
theme-driven colors as the default when a category is left unset. Clarified during
discussion: (1) mechanism is the extension translating drut.highlight.* into VS Code's
own native `editor.tokenColorCustomizations` setting, reusing the TextMate scopes
`024-function-call-highlighting` already ships, not a new coloring engine and not an
LSP semantic-tokens rebuild; (2) scope is a personal VS Code setting only — no
`drut.toml [highlight]` section, no `drut-config`/CLI/MCP wiring — since color is a
personal/accessibility preference, not a shared file-content convention the way casing
or indentation is."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A script author recolors one category of text to their own preference (Priority: P1)

A script author finds their current color theme doesn't distinguish built-in function
calls clearly enough from control words, or wants a specific category to stand out for
accessibility reasons. They open VS Code's Settings UI, find `Drut > Highlight`, and set
a color for the category they care about (e.g. `drut.highlight.functionCalls`). Without
reloading the window or restarting the extension, every `.s`/`.block` file's matching
text immediately renders in that color, and every category they left unset keeps
rendering exactly as their color theme already colors it today.

**Why this priority**: This is the entire content of the request — the whole feature is
this one interaction, personalizing colors without losing today's theme-driven behavior
for anything not explicitly touched.

**Independent Test**: Set `drut.highlight.functionCalls` to a color, open a `.s` file
containing a recognized function call, confirm it renders in that color while an
untouched category (e.g. control words) still renders however the active theme colors
`keyword.control.drut`.

**Acceptance Scenarios**:

1. **Given** `drut.highlight.functionCalls` is unset, **When** a `.s` file containing
   `REPLACESTR(...)` is open, **Then** it renders exactly as it does today — whatever
   color the active theme gives `support.function.builtin.drut` — no different from
   before this feature existed.
2. **Given** `drut.highlight.functionCalls` is set to `#FF6B35`, **When** the same file
   is open (already open, or newly opened), **Then** `REPLACESTR(...)` renders in
   `#FF6B35`, and every other category (control words, pair-keywords, strings, ...)
   still renders exactly as the active theme colors them.
3. **Given** `drut.highlight.functionCalls` was set and is now cleared (setting removed
   or set back to its default/empty), **When** the file is viewed again, **Then**
   `REPLACESTR(...)` reverts to the active theme's own color for
   `support.function.builtin.drut` — not stuck at the last-configured color.
4. **Given** two categories are set to two different colors in the same session
   (e.g. `drut.highlight.controlWords` and `drut.highlight.functionCalls`), **When** a
   file containing both is open, **Then** each category renders in its own configured
   color, independently.

---

### User Story 2 - A script author's own unrelated `editor.tokenColorCustomizations` rules are preserved (Priority: P2)

A script author already has their own `editor.tokenColorCustomizations` rules configured
— for a different language, or a manual tweak they made to some other scope, possibly
even a manual rule they wrote for one of drut's own scopes before this feature existed.
Turning `drut.highlight.*` settings on or off must never silently discard any rule this
feature didn't itself create.

**Why this priority**: `editor.tokenColorCustomizations` is a single, shared, user-owned
setting — every other extension and every hand-written customization lives in the same
JSON value. Getting this wrong means data loss in the user's own settings file, a much
worse failure than the feature simply not working.

**Independent Test**: Hand-write an unrelated `editor.tokenColorCustomizations` rule (a
different scope entirely), then set and later unset a `drut.highlight.*` value; confirm
the unrelated rule is present, unmodified, at every step.

**Acceptance Scenarios**:

1. **Given** an existing `editor.tokenColorCustomizations` setting containing a rule for
   an unrelated scope (e.g. a Python-specific scope from another extension), **When**
   `drut.highlight.controlWords` is set to a color, **Then** the unrelated rule is still
   present in `editor.tokenColorCustomizations`, byte-for-byte unchanged.
2. **Given** the state from Scenario 1, **When** `drut.highlight.controlWords` is later
   unset, **Then** only the rule this feature added for `keyword.control.drut` is
   removed — the unrelated rule remains, and no empty leftover structure is left behind
   that would visibly clutter the user's settings JSON.

---

### Edge Cases

- What happens if the user manually edits `editor.tokenColorCustomizations` to add their
  own rule for one of drut's own scopes (e.g. hand-writes a rule for
  `keyword.control.drut`) while a `drut.highlight.controlWords` value is also set? The
  two are the same underlying mechanism targeting the same scope — last write wins, the
  same as any other case of two sources targeting one TextMate scope in this setting.
  This feature does not attempt to detect or warn about the collision; documented as a
  known interaction, not a bug.
- What happens to a category whose current scope is shared with another category (before
  this feature, `support.function.drut` is used by both general statement words and
  function calls — see Assumptions)? Resolved as a prerequisite: the grammar's shared
  scope is split into two distinct scopes so the two categories can be colored
  independently, matching what a user reasonably expects from two separately-named
  `drut.highlight.*` settings.
- What happens when a configured value isn't a valid color? VS Code's own settings
  schema validates the shape at the UI/settings-file level (a plain string field with a
  documented expected format); an invalid string is passed through to
  `editor.tokenColorCustomizations` as-is, the same graceful-degradation VS Code itself
  already applies to a malformed color value anywhere in that setting (renders as no
  override, not a hard failure) — this feature does not add its own separate validation
  layer.
- What happens across multiple open windows/workspaces? `editor.tokenColorCustomizations`
  at the User (Global) scope is inherently a per-installation setting, so a change is
  already visible everywhere VS Code's own settings-sync/multi-window behavior already
  makes any Global setting visible — no drut-specific multi-window logic needed.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The extension MUST expose one `drut.highlight.<category>` VS Code setting
  per customizable category (Key Entities), each accepting a color value or being left
  unset.
- **FR-002**: When a `drut.highlight.<category>` setting is set, the extension MUST
  ensure `editor.tokenColorCustomizations`'s (User/Global-scope) `textMateRules` array
  contains a rule mapping that category's TextMate scope to the configured color.
- **FR-003**: When a `drut.highlight.<category>` setting is unset (absent or cleared),
  the extension MUST ensure no rule for that category's scope remains in
  `editor.tokenColorCustomizations` — reverting that category to whatever color the
  user's active theme provides, matching this feature's own pre-existing (theme-driven)
  behavior exactly.
- **FR-004**: The extension MUST NOT modify any `editor.tokenColorCustomizations` rule
  it did not itself add — every rule for a scope outside this feature's own known
  category-to-scope table (Key Entities) is left byte-for-byte untouched, regardless of
  how many times `drut.highlight.*` settings change.
- **FR-005**: A change to any `drut.highlight.*` setting MUST take visible effect without
  requiring a window reload or extension restart.
- **FR-006**: `editors/vscode/syntaxes/drut.tmLanguage.json`'s shared
  `support.function.drut` scope (used by both `#statement-words` and `#function-calls`
  since `024-function-call-highlighting`) MUST be split into two distinct scopes so
  general statement words and built-in function calls are independently colorable —
  a prerequisite grammar change, not a behavior regression (both continue to render
  identically under any theme that has no rule of its own for either new scope name,
  since TextMate scope-selector fallback matches on the shared `support.function.*`
  prefix either way).
- **FR-007**: This feature MUST NOT add a `drut.toml [highlight]` section, a CLI flag, or
  an MCP tool parameter — VS Code personal setting only (per the resolved Scope
  question).
- **FR-008**: This feature MUST NOT change `voyager-core`'s tokenizer, parser, formatter,
  or any `Diagnostic` category, and MUST NOT change `drut-lsp`'s semantic-tokens
  implementation or `editors/vscode`'s existing `ensureVariableColorCustomization`/
  `ensureFormatOnSaveEnabled` injection functions — purely an `editors/vscode`
  client-side concern, additive alongside that existing code, not a modification of it.
- **FR-009**: Every `drut.highlight.<category>` setting MUST default to unset (no
  built-in opinionated color), matching this project's existing "no built-in preset"
  stance for every other customizable `[format]`/`drut.format.*` field.
- **FR-010**: The extension MUST read and write only the Global (User) scope of both
  `drut.highlight.*` and `editor.tokenColorCustomizations` — a Workspace- or
  Folder-scoped `drut.highlight.*` value (e.g. set in a repo's `.vscode/settings.json`)
  MUST NOT be applied, and MUST NOT cause a write to Workspace-scoped
  `editor.tokenColorCustomizations`. This follows directly from the resolved Scope
  decision (personal, not project-shared) — a workspace-level color choice silently
  leaking into `editor.tokenColorCustomizations` at Global scope (and thus into every
  other project the user opens) would be a worse outcome than simply not supporting
  workspace-level overrides in this initial pass.

### Key Entities

- **Highlight category**: one customizable text role, each mapping to exactly one
  TextMate scope in `drut.tmLanguage.json` (post-FR-006 split):

  | `drut.highlight.<category>` | TextMate scope | Today's text |
  |---|---|---|
  | `controlWords` | `keyword.control.drut` | `IF`, `LOOP`, `RUN`, `ENDIF`, ... |
  | `statementWords` | `support.function.statement.drut` (new, split from `support.function.drut`) | `PRINT`, `FILEI`, `FILEO`, `ARRAY`, ... |
  | `functionCalls` | `support.function.builtin.drut` (new, split from `support.function.drut`) | `REPLACESTR(...)`, `ROUND(...)`, ... (`024`) |
  | `pairKeywords` | `variable.parameter.drut` | a `keyword=value` pair's keyword name |
  | `values` | `constant.other.drut` | a pair's bareword value (e.g. `PGM=MATRIX`'s `MATRIX`) |
  | `numbers` | `constant.numeric.drut` | numeric literals |
  | `operators` | `keyword.operator.drut` | `=`, `+`, `-`, `<>`, ... |
  | `comments` | `comment.line.semicolon.drut` + `comment.block.drut` | `; ...` and `/* ... */` |
  | `strings` | `string.quoted.single.drut` + `string.quoted.double.drut` | quoted string literals |

  String-escape (`constant.character.escape.drut`) and punctuation
  (`punctuation.*.drut`) scopes are not exposed as customizable categories in this
  initial pass — low-value to color independently of their surrounding string/structural
  context; can be added the same incremental way any other category could be, later.

  **`@name@` substitution (`variable.other.readwrite.drut`) is deliberately excluded**
  from this feature's category list — see Assumptions for why (an existing, separate,
  already-shipped color mechanism already governs it, and this feature's own
  TextMate-scope-based mechanism would not visibly win against it).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every one of the 9 categories, individually set to a distinct color,
  visibly renders in that color, verified for each category independently.
- **SC-002**: With every `drut.highlight.*` setting unset, `editor.
  tokenColorCustomizations` is unaffected by this feature's presence — byte-for-byte
  identical to a state where this feature's code never ran.
- **SC-003**: A pre-existing, unrelated `editor.tokenColorCustomizations` rule survives
  every `drut.highlight.*` set/unset cycle unchanged, in 100% of tested cases.
- **SC-004**: Setting or clearing any `drut.highlight.*` value takes effect without a
  window reload, in 100% of tested cases.
- **SC-005**: `statementWords` and `functionCalls` render independently once each is set
  to a distinct color — confirming the FR-006 scope split actually decouples them,
  not just that the setting exists.

## Assumptions

- **Mechanism**: resolved during scoping discussion — the extension owns writing into
  VS Code's own native `editor.tokenColorCustomizations` setting (Global/User scope,
  theme-independent `textMateRules`, not a per-theme override), rather than building a
  new coloring surface or expanding `drut-lsp`'s semantic-tokens implementation
  (currently narrow — 3 special-purpose token types, not a general per-category system;
  expanding it to cover every category would be substantial, duplicate-logic work with
  no benefit here since the personal-setting-only scope decision means portability to
  non-VS-Code LSP clients isn't a goal for this feature).
- **Scope**: resolved during scoping discussion — personal VS Code setting only. No
  `drut.toml` section, so no `drut-config`/`drut-cli`/`drut-mcp` involvement at all.
- **Ownership detection**: a rule is recognized as "ours" by its `scope` value matching
  one of this feature's own known category-to-scope table entries (Key Entities) — every
  drut scope name is suffixed `.drut`/ends in a `.drut`-namespaced segment, which no
  other extension or built-in VS Code scope would plausibly collide with, so this is a
  safe, sufficient ownership test without needing a separate marker field.
- `editor.tokenColorCustomizations` is written at Global (User) scope, matching
  `drut.highlight.*` itself being a personal, not project-shared, setting — a
  workspace-scoped write is out of scope for this feature. This is a deliberate,
  reasoned *departure* from the existing `ensureVariableColorCustomization` precedent
  (`editors/vscode/src/extension.ts`), which writes Workspace-scoped, one-time-only, on
  first activation per workspace — that mechanism solves a different problem (seed a
  visible default once, for themes that render nothing at all for a given scope/type,
  then get out of the way so a user's manual removal sticks forever) than this feature's
  (an ongoing, live, personal preference that should already apply the same way in every
  project the user opens, not be re-configured per workspace).
- **`@name@` substitution (the `variables` category) is excluded from this feature's
  scope entirely, discovered while auditing existing color-injection code.**
  `editors/vscode/src/extension.ts`'s `ensureVariableColorCustomization` already
  auto-injects a hardcoded color (`#4EC9B0`) for `@name@` references, once, into the
  *workspace's* `editor.semanticTokenColorCustomizations` (a `variable:drut` rule — the
  standard LSP `variable` semantic token type, scoped to the `drut` language only) —
  added because real manual testing found some themes render `variable.other.
  readwrite.drut` (the TextMate scope this feature's own mechanism would target)
  invisibly under their own rules, a gap only the semantic-token layer's baseline color
  support could close. VS Code layers semantic-token colors over TextMate-scope colors
  for the same span, so a `drut.highlight.variables` setting built on this feature's
  `editor.tokenColorCustomizations` mechanism would silently lose to the existing
  semantic-token rule and appear to do nothing — not a bug in this feature, but a
  real mechanism mismatch that would need its own dedicated design (making
  `drut.highlight.variables` drive the *existing* `variable:drut` semantic-token rule's
  value instead, and migrating that rule's lifecycle from one-time/Workspace to
  live/Global) rather than being folded into this feature's uniform, single-mechanism
  approach. Left as a clearly-scoped future addition, not attempted here.
