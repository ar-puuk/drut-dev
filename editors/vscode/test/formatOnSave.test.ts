// Unit tests for the format-on-save injection-decision predicate
// (specs/005-format-on-save-paste, FR-004/FR-006/US1/US3). Runs standalone
// via ts-node, mirroring test/grammar.test.ts's existing convention -- no
// VS Code instance needed, since shouldInjectFormatOnSave has zero
// dependency on the `vscode` module (formatOnSaveDecision.ts's own module
// doc explains why that separation exists).

import { shouldInjectFormatOnSave } from "../src/formatOnSaveDecision";

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
  // Not yet injected, no existing override -> inject (US1's normal,
  // first-activation case).
  check(
    "not yet injected, no existing override: should inject",
    shouldInjectFormatOnSave(false, undefined) === true
  );

  // Already injected (workspaceState gate) -> never re-attempt, regardless
  // of the current override state.
  check(
    "already injected, no existing override: should not inject",
    shouldInjectFormatOnSave(true, undefined) === false
  );
  check(
    "already injected, existing override present: should not inject",
    shouldInjectFormatOnSave(true, true) === false
  );

  // Not yet injected, but an explicit override already exists -- the
  // FR-006/US3 guarantee: never overwrite a user's (or an earlier run's)
  // existing choice, regardless of its value.
  check(
    "not yet injected, existing override is true: should not inject",
    shouldInjectFormatOnSave(false, true) === false
  );
  check(
    "not yet injected, existing override is false: should not inject",
    shouldInjectFormatOnSave(false, false) === false
  );

  if (failures > 0) {
    console.error(`${failures} check(s) failed`);
    process.exit(1);
  }
  console.log("all format-on-save injection-decision checks passed");
}

main();
