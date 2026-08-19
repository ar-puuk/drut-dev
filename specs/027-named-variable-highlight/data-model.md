# Data Model: `@name@` Variable Highlight Color Customization

## 1. `decideVariableColorSync` (pure function, `highlightCustomization.ts`)

```typescript
export interface VariableColorSyncState {
  /** Legacy one-time-seed flag -- true once ensureVariableColorCustomization
   *  has ever written the rule for this workspace, by any path. */
  alreadySeeded: boolean;
  /** New flag -- true once drut.highlight.namedVariables has taken over
   *  live-sync duty for this workspace (research.md §3). */
  liveSyncActive: boolean;
}

export interface VariableColorDecision {
  shouldWrite: boolean;
  value?: string;              // the color to write, when shouldWrite is true
  nextState: VariableColorSyncState;
}

export const DEFAULT_VARIABLE_COLOR = "#4EC9B0"; // unchanged from today's shipped value

/**
 * research.md §3's truth table, as code. `existingRuleValue` is the current
 * `rules["variable:drut"]` value already present in
 * `editor.semanticTokenColorCustomizations` (Workspace scope), if any.
 * `configuredColor` is `drut.highlight.namedVariables`'s Global value, if any.
 */
export function decideVariableColorSync(
  state: VariableColorSyncState,
  existingRuleValue: string | undefined,
  configuredColor: string | undefined
): VariableColorDecision {
  if (configuredColor !== undefined) {
    const shouldWrite = existingRuleValue !== configuredColor;
    return {
      shouldWrite,
      value: configuredColor,
      nextState: { alreadySeeded: true, liveSyncActive: true },
    };
  }
  if (state.liveSyncActive) {
    // Just turned the override off -- one corrective revert to the default.
    return {
      shouldWrite: true,
      value: DEFAULT_VARIABLE_COLOR,
      nextState: { alreadySeeded: true, liveSyncActive: false },
    };
  }
  if (!state.alreadySeeded && existingRuleValue === undefined) {
    // Original one-time seed, unchanged from today.
    return {
      shouldWrite: true,
      value: DEFAULT_VARIABLE_COLOR,
      nextState: { alreadySeeded: true, liveSyncActive: false },
    };
  }
  // Already seeded (or user removed it) and no override configured -- never
  // fight a manual choice (spec.md FR-004).
  return { shouldWrite: false, nextState: { ...state, liveSyncActive: false } };
}
```

## 2. Effectful wrapper (`extension.ts`, refactor of `ensureVariableColorCustomization`)

```typescript
const VARIABLE_COLOR_LIVE_SYNC_KEY = "drutVariableColorLiveSyncActive"; // NEW

async function ensureVariableColorCustomization(context: vscode.ExtensionContext): Promise<void> {
  if (!vscode.workspace.workspaceFolders || vscode.workspace.workspaceFolders.length === 0) {
    return; // unchanged guard
  }

  try {
    const config = vscode.workspace.getConfiguration();
    const current = config.get<{ rules?: Record<string, unknown> }>("editor.semanticTokenColorCustomizations") ?? {};
    const rules = current.rules ?? {};
    const configuredColor = config.inspect<string>("drut.highlight.namedVariables")?.globalValue;

    const decision = decideVariableColorSync(
      {
        alreadySeeded: context.workspaceState.get<boolean>(VARIABLE_COLOR_INJECTED_KEY) ?? false,
        liveSyncActive: context.workspaceState.get<boolean>(VARIABLE_COLOR_LIVE_SYNC_KEY) ?? false,
      },
      rules[VARIABLE_COLOR_RULE_KEY] as string | undefined,
      configuredColor
    );

    if (decision.shouldWrite) {
      await config.update(
        "editor.semanticTokenColorCustomizations",
        { ...current, rules: { ...rules, [VARIABLE_COLOR_RULE_KEY]: decision.value } },
        vscode.ConfigurationTarget.Workspace
      );
    }
    await context.workspaceState.update(VARIABLE_COLOR_INJECTED_KEY, decision.nextState.alreadySeeded);
    await context.workspaceState.update(VARIABLE_COLOR_LIVE_SYNC_KEY, decision.nextState.liveSyncActive);
  } catch {
    // Never let this best-effort convenience fail extension activation --
    // unchanged discipline.
  }
}
```

Called from `activate()` exactly as today, **plus** from the existing
`onDidChangeConfiguration` handler (already gated on
`e.affectsConfiguration("drut.highlight")`, which already covers
`drut.highlight.namedVariables` — no new listener needed, research.md §4).
