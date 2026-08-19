# Data Model: Editor Highlight Color Customization

## 1. `HighlightCategory` (new, `editors/vscode/src/highlightCustomization.ts`)

```typescript
export type HighlightCategory =
  | "controlWords"
  | "statementWords"
  | "functionCalls"
  | "pairKeywords"
  | "values"
  | "numbers"
  | "operators"
  | "comments"
  | "strings";

/** Single source of truth for category -> TextMate scope(s) (research.md §4/§5). */
export const CATEGORY_SCOPES: Record<HighlightCategory, string | string[]> = {
  controlWords: "keyword.control.drut",
  statementWords: "support.function.statement.drut",
  functionCalls: "support.function.builtin.drut",
  pairKeywords: "variable.parameter.drut",
  values: "constant.other.drut",
  numbers: "constant.numeric.drut",
  operators: "keyword.operator.drut",
  comments: ["comment.line.semicolon.drut", "comment.block.drut"],
  strings: ["string.quoted.single.drut", "string.quoted.double.drut"],
};
```

`drut.highlight.<category>` in `package.json` uses this exact camelCase category name
for each of the 9 keys (`drut.highlight.controlWords`, ..., `drut.highlight.strings`) —
same naming convention `drut.format.*` already established.

## 2. `TokenColorCustomizations` (structural type, not owned by drut — VS Code's own setting)

```typescript
interface TextMateRule {
  scope?: string | string[];
  settings?: { foreground?: string; [key: string]: unknown };
  [key: string]: unknown; // tolerate/preserve unknown fields on a rule we don't own
}

interface TokenColorCustomizations {
  textMateRules?: TextMateRule[];
  [key: string]: unknown; // preserve every other top-level key untouched (shorthand
                           // keys like "comments"/"numbers", per-theme "[Name]" objects)
}
```

## 3. `mergeHighlightRules` (pure function, `highlightCustomization.ts`)

```typescript
/**
 * Returns the new TokenColorCustomizations value to write, given the
 * current value (as read from config) and the desired category->color
 * mapping (a category absent or mapped to undefined means "unset").
 *
 * - Removes every existing rule this feature owns (exact scope-set match
 *   against CATEGORY_SCOPES, research.md §4) whose category is now unset.
 * - Upserts a rule for every category with a defined color.
 * - Every other key/rule in `current` is preserved unchanged (FR-004).
 * - Omits the `textMateRules` key entirely from the result when it would
 *   otherwise be an empty array (User Story 2 Acceptance Scenario 2 --
 *   "no empty leftover structure").
 */
export function mergeHighlightRules(
  current: TokenColorCustomizations,
  desired: Partial<Record<HighlightCategory, string | undefined>>
): TokenColorCustomizations;

/** True iff `result` has zero own keys -- caller should clear the whole
 *  `editor.tokenColorCustomizations` setting (config.update(key, undefined,
 *  Global)) rather than write an empty object. */
export function isEmptyTokenColorCustomizations(result: TokenColorCustomizations): boolean;
```

## 4. Effectful wrapper (`extension.ts`, mirrors `ensureVariableColorCustomization`'s shape)

```typescript
async function applyHighlightCustomizations(): Promise<void> {
  try {
    const config = vscode.workspace.getConfiguration();
    const desired: Partial<Record<HighlightCategory, string | undefined>> = {};
    for (const category of Object.keys(CATEGORY_SCOPES) as HighlightCategory[]) {
      // .inspect().globalValue only -- FR-010, never the workspace-merged
      // effective value, never a workspace-scoped write target.
      desired[category] = config.inspect<string>(`drut.highlight.${category}`)?.globalValue;
    }
    const currentGlobalRaw = config.inspect<TokenColorCustomizations>("editor.tokenColorCustomizations")?.globalValue;
    const next = mergeHighlightRules(currentGlobalRaw ?? {}, desired);
    const nextOrUndefined = isEmptyTokenColorCustomizations(next) ? undefined : next;
    // No-op guard: skip the write entirely when nothing would actually
    // change -- otherwise every single activation (including for a user who
    // never touches drut.highlight.* at all) would unconditionally rewrite
    // editor.tokenColorCustomizations, in tension with spec.md SC-002's
    // "byte-for-byte identical to a state where this feature's code never
    // ran" even though the *value* would be correct either way
    // (/speckit-analyze finding, 026-highlight-customization).
    if (!deepEqual(nextOrUndefined, currentGlobalRaw)) {
      await config.update("editor.tokenColorCustomizations", nextOrUndefined, vscode.ConfigurationTarget.Global);
    }
  } catch {
    // Never let this best-effort convenience fail extension activation --
    // same discipline as ensureVariableColorCustomization/
    // ensureFormatOnSaveEnabled.
  }
}
```

Called once from `activate()`, and again on every
`workspace.onDidChangeConfiguration` event where `e.affectsConfiguration("drut.highlight")`
is true. No loop risk: the handler is scoped to `drut.highlight` changes only, and this
function's own write targets a different setting (`editor.tokenColorCustomizations`), so
it never re-triggers itself.
