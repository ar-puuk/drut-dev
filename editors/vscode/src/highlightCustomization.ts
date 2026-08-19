// Pure category<->scope table and editor.tokenColorCustomizations merge/removal
// logic for drut.highlight.* (026-highlight-customization, data-model.md
// §1-§3). Deliberately kept in its own module with zero dependency on the
// `vscode` package -- the real "vscode" module only resolves inside a
// running extension host, so this needs to stay unit-testable via plain
// ts-node (test/highlightCustomization.test.ts, mirroring
// formatOnSaveDecision.ts's/test/formatOnSave.test.ts's existing standalone
// convention). extension.ts's applyHighlightCustomizations is the effectful
// wrapper that imports this module and calls it with real values read from
// the VS Code configuration API.

/** One customizable highlight category recognized via `editor.
 * tokenColorCustomizations` (026's own mechanism; spec.md Key Entities).
 * `@name@` substitution is deliberately not a member here: it's governed by
 * a different, pre-existing mechanism (extension.ts's
 * ensureVariableColorCustomization, semantic-token based via `editor.
 * semanticTokenColorCustomizations`) that this module's TextMate-scope-based
 * mechanism would not visibly win against (026 research.md §3). It's
 * reachable instead through `drut.highlight.namedVariables`, backed by
 * `decideVariableColorSync` below (027-named-variable-highlight). */
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

/** Single source of truth for category -> TextMate scope(s) (data-model.md
 * §1). A category's value is a single scope string, or an array of exactly
 * the scopes that together make up that category (comments/strings) --
 * always written and matched as one rule per category, never split further. */
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

const ALL_CATEGORIES = Object.keys(CATEGORY_SCOPES) as HighlightCategory[];

/** One entry of VS Code's own `editor.tokenColorCustomizations.textMateRules`
 * array -- a shape we don't own, so every field beyond `scope`/`settings`
 * (which we read/write) is preserved untouched via the index signature. */
export interface TextMateRule {
  scope?: string | string[];
  settings?: { foreground?: string; [key: string]: unknown };
  [key: string]: unknown;
}

/** VS Code's own `editor.tokenColorCustomizations` setting shape -- also not
 * ours; every key besides `textMateRules` (per-theme "[Theme Name]" override
 * objects, the generic shorthand keys like "numbers"/"comments") is
 * preserved untouched via the index signature (FR-004). */
export interface TokenColorCustomizations {
  textMateRules?: TextMateRule[];
  [key: string]: unknown;
}

/** Normalizes a rule's `scope` field to a sorted array of strings, for
 * order-independent set comparison. */
function normalizeScope(scope: string | string[] | undefined): string[] {
  const arr = scope === undefined ? [] : Array.isArray(scope) ? scope : [scope];
  return [...arr].sort();
}

function scopesEqual(a: string[], b: string[]): boolean {
  return a.length === b.length && a.every((s, i) => s === b[i]);
}

/** Whether `rule` is exactly one of this module's own category rules --
 * its `scope`, normalized and compared as a set, exactly equals one of the
 * 9 known category scope-sets (research.md §4: exact match, not
 * substring/overlap, so a user's own rule that merely happens to reference
 * one of our scopes alongside something else is never touched). */
function isOwnedRule(rule: TextMateRule): boolean {
  const ruleScopes = normalizeScope(rule.scope);
  return ALL_CATEGORIES.some((category) => scopesEqual(ruleScopes, normalizeScope(CATEGORY_SCOPES[category])));
}

/**
 * Returns the new `TokenColorCustomizations` value to write, given the
 * current value (as read from config) and the desired category->color
 * mapping (a category absent from `desired`, or mapped to `undefined`,
 * means "unset").
 *
 * - Removes every existing rule this module owns (`isOwnedRule`) -- they are
 *   about to be fully recomputed from `desired` below.
 * - Upserts one rule per category with a defined color in `desired`.
 * - Every other key/rule in `current` is preserved unchanged (FR-004) --
 *   `current`'s own key order (aside from `textMateRules`) and every
 *   unrelated `textMateRules` entry's relative order are both preserved.
 * - Omits the `textMateRules` key entirely from the result when it would
 *   otherwise be an empty array (User Story 2 Acceptance Scenario 2 -- "no
 *   empty leftover structure").
 */
export function mergeHighlightRules(
  current: TokenColorCustomizations,
  desired: Partial<Record<HighlightCategory, string | undefined>>
): TokenColorCustomizations {
  const survivingRules = (current.textMateRules ?? []).filter((rule) => !isOwnedRule(rule));

  const newRules: TextMateRule[] = ALL_CATEGORIES.filter((category) => desired[category] !== undefined).map(
    (category) => ({
      scope: CATEGORY_SCOPES[category],
      settings: { foreground: desired[category] as string },
    })
  );

  const rules = [...survivingRules, ...newRules];
  const result: TokenColorCustomizations = { ...current };
  if (rules.length > 0) {
    result.textMateRules = rules;
  } else {
    delete result.textMateRules;
  }
  return result;
}

/** True iff `result` has zero own keys -- caller should clear the whole
 * `editor.tokenColorCustomizations` setting (`config.update(key, undefined,
 * Global)`) rather than write an empty object. */
export function isEmptyTokenColorCustomizations(result: TokenColorCustomizations): boolean {
  return Object.keys(result).length === 0;
}

/** Structural deep-equality -- order-insensitive for plain-object keys,
 * order-sensitive for arrays (matches JSON semantics for the shapes this
 * module actually produces/compares). Backs `extension.ts`'s no-op write
 * guard (`/speckit-analyze` finding: skip the `editor.
 * tokenColorCustomizations` write entirely when nothing would actually
 * change, so an activation for a user who never touches `drut.highlight.*`
 * never rewrites `settings.json` at all). */
export function deepEqual(a: unknown, b: unknown): boolean {
  if (a === b) {
    return true;
  }
  if (typeof a !== "object" || typeof b !== "object" || a === null || b === null) {
    return false;
  }
  if (Array.isArray(a) || Array.isArray(b)) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) {
      return false;
    }
    return a.every((item, i) => deepEqual(item, b[i]));
  }
  const aKeys = Object.keys(a as Record<string, unknown>);
  const bKeys = Object.keys(b as Record<string, unknown>);
  if (aKeys.length !== bKeys.length) {
    return false;
  }
  return aKeys.every(
    (key) => key in (b as Record<string, unknown>) && deepEqual((a as Record<string, unknown>)[key], (b as Record<string, unknown>)[key])
  );
}

// -- 027-named-variable-highlight -------------------------------------------
//
// drut.highlight.namedVariables (@name@ substitution) reconciles a live,
// user-driven preference with the pre-existing ensureVariableColorCustomization
// mechanism's own guarantee ("a manual deletion of the seeded rule sticks
// forever, for a workspace that never touches this new setting") -- see
// 027's research.md §3 for the full truth table this function encodes.

/** Tracked, per-workspace state this decision needs across calls (backed by
 * two `context.workspaceState` keys in extension.ts). */
export interface VariableColorSyncState {
  /** True once the rule has ever been written for this workspace, by either
   * the original one-time seed or a later live sync. */
  alreadySeeded: boolean;
  /** True once `drut.highlight.namedVariables` has taken over live-sync
   * duty for this workspace (data-model.md §1). */
  liveSyncActive: boolean;
}

export interface VariableColorDecision {
  shouldWrite: boolean;
  /** The color to write into the `variable:drut` rule -- present iff `shouldWrite`. */
  value?: string;
  nextState: VariableColorSyncState;
}

/** Unchanged from `026`'s originally-shipped hardcoded seed value. */
export const DEFAULT_VARIABLE_COLOR = "#4EC9B0";

/**
 * Decides whether/how to update the workspace's `variable:drut` rule in
 * `editor.semanticTokenColorCustomizations`.
 *
 * - `configuredColor` set: keep the rule live-synced to it (spec.md FR-002).
 * - `configuredColor` unset, but live-sync was just active: one corrective
 *   revert to `DEFAULT_VARIABLE_COLOR` (spec.md FR-005) -- never leaves the
 *   rule stuck at a stale custom color, never removes it outright (a fully
 *   theme-driven state would reintroduce the invisible-under-some-themes
 *   bug this whole mechanism exists to fix).
 * - `configuredColor` unset, never live-synced, never seeded, no existing
 *   rule: the original one-time seed, byte-identical to `026`'s shipped
 *   behavior.
 * - `configuredColor` unset, already seeded (or the user removed the rule
 *   by hand) and never live-synced: no write -- never fights a manual
 *   choice (spec.md FR-004, the regression this feature must not cause).
 */
export function decideVariableColorSync(
  state: VariableColorSyncState,
  existingRuleValue: string | undefined,
  configuredColor: string | undefined
): VariableColorDecision {
  if (configuredColor !== undefined) {
    return {
      shouldWrite: existingRuleValue !== configuredColor,
      value: configuredColor,
      nextState: { alreadySeeded: true, liveSyncActive: true },
    };
  }
  if (state.liveSyncActive) {
    return {
      shouldWrite: true,
      value: DEFAULT_VARIABLE_COLOR,
      nextState: { alreadySeeded: true, liveSyncActive: false },
    };
  }
  if (!state.alreadySeeded && existingRuleValue === undefined) {
    return {
      shouldWrite: true,
      value: DEFAULT_VARIABLE_COLOR,
      nextState: { alreadySeeded: true, liveSyncActive: false },
    };
  }
  return { shouldWrite: false, nextState: { alreadySeeded: state.alreadySeeded, liveSyncActive: false } };
}
