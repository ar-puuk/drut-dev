// Standalone tests for the pure drut.highlight.* merge/removal logic
// (026-highlight-customization). Runs via plain ts-node -- no `vscode`
// import anywhere in highlightCustomization.ts, mirroring
// formatOnSave.test.ts's/grammar.test.ts's existing standalone convention.

import {
  CATEGORY_SCOPES,
  deepEqual,
  isEmptyTokenColorCustomizations,
  mergeHighlightRules,
  TokenColorCustomizations,
  HighlightCategory,
} from "../src/highlightCustomization";

let failures = 0;
function check(name: string, condition: boolean): void {
  if (condition) {
    console.log(`ok - ${name}`);
  } else {
    failures++;
    console.error(`FAIL - ${name}`);
  }
}

function main(): void {
  // T006: nothing set against an empty current is a strict no-op.
  {
    const result = mergeHighlightRules({}, {});
    check("empty desired against empty current has no textMateRules key", result.textMateRules === undefined);
    check("empty desired against empty current is reported empty", isEmptyTokenColorCustomizations(result));
  }

  // T007: one category set upserts exactly one correct rule.
  {
    const result = mergeHighlightRules({}, { functionCalls: "#FF6B35" });
    check("exactly one rule produced", (result.textMateRules ?? []).length === 1);
    const rule = result.textMateRules![0];
    check("rule scope is support.function.builtin.drut", rule.scope === "support.function.builtin.drut");
    check("rule foreground is #FF6B35", rule.settings?.foreground === "#FF6B35");
  }

  // T008: set then unset removes exactly that rule.
  {
    const afterSet = mergeHighlightRules({}, { functionCalls: "#FF6B35" });
    const afterUnset = mergeHighlightRules(afterSet, {});
    check("unsetting removes the rule", afterUnset.textMateRules === undefined);
    check("unsetting reports empty", isEmptyTokenColorCustomizations(afterUnset));
  }

  // T009: two categories set independently in one pass.
  {
    const result = mergeHighlightRules({}, { controlWords: "#C586C0", functionCalls: "#FF6B35" });
    const rules = result.textMateRules ?? [];
    check("exactly two rules produced", rules.length === 2);
    const controlRule = rules.find((r) => r.scope === "keyword.control.drut");
    const functionRule = rules.find((r) => r.scope === "support.function.builtin.drut");
    check("controlWords rule present with correct color", controlRule?.settings?.foreground === "#C586C0");
    check("functionCalls rule present with correct color", functionRule?.settings?.foreground === "#FF6B35");
  }

  // T010: every one of the 9 categories individually produces a correct
  // rule when set alone -- data-driven, not a hand-picked sample (mirrors
  // 024's/025's own SC-001 remediation).
  {
    const categories = Object.keys(CATEGORY_SCOPES) as HighlightCategory[];
    check("category table has 9 entries", categories.length === 9);
    let allCorrect = true;
    const misses: string[] = [];
    for (const category of categories) {
      const result = mergeHighlightRules({}, { [category]: "#123456" });
      const rules = result.textMateRules ?? [];
      const expectedScope = CATEGORY_SCOPES[category];
      const rule = rules.find((r) => JSON.stringify(r.scope) === JSON.stringify(expectedScope));
      if (rules.length !== 1 || !rule || rule.settings?.foreground !== "#123456") {
        allCorrect = false;
        misses.push(category);
      }
    }
    check(`all 9 categories produce correct rules when set alone${misses.length ? ` (missed: ${misses.join(", ")})` : ""}`, allCorrect);
  }

  // T012: an unrelated rule plus an unrelated top-level key survive a set.
  let currentWithUnrelated: TokenColorCustomizations = {
    textMateRules: [{ scope: "entity.name.tag.python", settings: { foreground: "#EEEEEE" } }],
    "[Some Theme]": { textMateRules: [{ scope: "comment", settings: { foreground: "#000000" } }] },
  };
  {
    const result = mergeHighlightRules(currentWithUnrelated, { controlWords: "#C586C0" });
    const rules = result.textMateRules ?? [];
    check("unrelated rule still present after set", rules.some((r) => r.scope === "entity.name.tag.python"));
    check("new rule also present after set", rules.some((r) => r.scope === "keyword.control.drut"));
    check(
      "unrelated top-level key untouched after set",
      deepEqual(result["[Some Theme]"], currentWithUnrelated["[Some Theme]"])
    );
    currentWithUnrelated = result; // carry forward into the next check
  }

  // T013: unsetting afterward removes only our rule; unrelated content remains.
  {
    const result = mergeHighlightRules(currentWithUnrelated, {});
    const rules = result.textMateRules ?? [];
    check("unrelated rule survives the following unset", rules.some((r) => r.scope === "entity.name.tag.python"));
    check("our rule is gone after unset", !rules.some((r) => r.scope === "keyword.control.drut"));
    check(
      "unrelated top-level key still untouched after unset",
      deepEqual(result["[Some Theme]"], currentWithUnrelated["[Some Theme]"])
    );
  }

  // T014: a scope array only *partially* overlapping one of our known
  // scopes is never touched -- exact-set-match ownership, not
  // substring/overlap (research.md §4).
  {
    const mixedRule: TokenColorCustomizations = {
      textMateRules: [{ scope: ["keyword.control.drut", "keyword.other.foo"], settings: { foreground: "#ABCDEF" } }],
    };
    const afterSet = mergeHighlightRules(mixedRule, { controlWords: "#C586C0" });
    const mixedStillPresent = (afterSet.textMateRules ?? []).some(
      (r) => Array.isArray(r.scope) && r.scope.length === 2 && r.scope.includes("keyword.other.foo")
    );
    check("a scope array only partially matching ours is left untouched", mixedStillPresent);

    const afterUnset = mergeHighlightRules(afterSet, {});
    const mixedStillPresentAfterUnset = (afterUnset.textMateRules ?? []).some(
      (r) => Array.isArray(r.scope) && r.scope.length === 2 && r.scope.includes("keyword.other.foo")
    );
    check("the same partial-match rule survives an unset too", mixedStillPresentAfterUnset);
  }

  // deepEqual sanity checks (backs the no-op write guard in extension.ts).
  {
    check("deepEqual: identical primitives", deepEqual("#FF0000", "#FF0000"));
    check("deepEqual: different primitives", !deepEqual("#FF0000", "#00FF00"));
    check("deepEqual: order-insensitive object keys", deepEqual({ a: 1, b: 2 }, { b: 2, a: 1 }));
    check("deepEqual: order-sensitive arrays", !deepEqual(["a", "b"], ["b", "a"]));
    check("deepEqual: undefined vs undefined", deepEqual(undefined, undefined));
    check("deepEqual: nested structures", deepEqual({ textMateRules: [{ scope: "x" }] }, { textMateRules: [{ scope: "x" }] }));
  }

  if (failures > 0) {
    console.error(`${failures} check(s) failed`);
    process.exit(1);
  }
  console.log("all highlightCustomization checks passed");
}

main();
