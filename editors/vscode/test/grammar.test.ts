// Tokenization spot-checks for the static TextMate grammar (Story 1,
// FR-021) — control words, comments (including the nested block-comment
// case), strings, and @variable@ substitutions each get a distinct scope.
// Runs standalone via vscode-textmate + vscode-oniguruma; no VS Code
// instance needed (unlike a full extension-host test).

import * as fs from "fs";
import * as path from "path";
import * as oniguruma from "vscode-oniguruma";
import * as vsctm from "vscode-textmate";

const SCOPE_NAME = "source.drut";
const GRAMMAR_PATH = path.join(__dirname, "..", "syntaxes", "drut.tmLanguage.json");

async function loadGrammar(): Promise<vsctm.IGrammar> {
  const wasmPath = require.resolve("vscode-oniguruma/release/onig.wasm");
  const wasmBin = fs.readFileSync(wasmPath).buffer;
  await oniguruma.loadWASM(wasmBin);

  const onigLib = Promise.resolve({
    createOnigScanner: (patterns: string[]) => new oniguruma.OnigScanner(patterns),
    createOnigString: (s: string) => new oniguruma.OnigString(s),
  });

  const registry = new vsctm.Registry({
    onigLib,
    loadGrammar: async (scopeName: string) => {
      if (scopeName === SCOPE_NAME) {
        return JSON.parse(fs.readFileSync(GRAMMAR_PATH, "utf8"));
      }
      return null;
    },
  });

  const grammar = await registry.loadGrammar(SCOPE_NAME);
  if (!grammar) {
    throw new Error("failed to load grammar");
  }
  return grammar;
}

function tokenizeAll(grammar: vsctm.IGrammar, lines: string[]): vsctm.IToken[][] {
  let ruleStack: vsctm.StateStack | null = vsctm.INITIAL;
  const result: vsctm.IToken[][] = [];
  for (const line of lines) {
    const lineResult = grammar.tokenizeLine(line, ruleStack);
    result.push(lineResult.tokens);
    ruleStack = lineResult.ruleStack;
  }
  return result;
}

function scopesAt(tokens: vsctm.IToken[], charIndex: number): string[] {
  const token = tokens.find((t) => t.startIndex <= charIndex && charIndex < t.endIndex);
  if (!token) {
    throw new Error(`no token covers char index ${charIndex}`);
  }
  return token.scopes;
}

let failures = 0;
function check(name: string, condition: boolean): void {
  if (condition) {
    console.log(`ok - ${name}`);
  } else {
    failures++;
    console.error(`FAIL - ${name}`);
  }
}

async function main(): Promise<void> {
  const grammar = await loadGrammar();

  // Control word gets keyword.control scope.
  {
    const [tokens] = tokenizeAll(grammar, ["IF (a=b)"]);
    const scopes = scopesAt(tokens, 0);
    check("control word IF scoped as keyword.control", scopes.some((s) => s.includes("keyword.control")));
  }

  // Line comment gets comment.line scope.
  {
    const [tokens] = tokenizeAll(grammar, ["; a comment"]);
    const scopes = scopesAt(tokens, 2);
    check("line comment scoped as comment.line", scopes.some((s) => s.includes("comment.line")));
  }

  // Nested block comment: the whole region, including the inner /* */,
  // stays inside comment.block across all three lines.
  {
    const lines = ["/* outer", "/* inner */ still-comment", "*/ done"];
    const tokenized = tokenizeAll(grammar, lines);
    const line0 = scopesAt(tokenized[0], 3); // inside "outer"
    const line1 = scopesAt(tokenized[1], 15); // inside "still-comment"
    const line2 = scopesAt(tokenized[2], 0); // the closing "*/"
    check("nested block comment: outer line is comment.block", line0.some((s) => s.includes("comment.block")));
    check(
      "nested block comment: content after inner close still comment.block",
      line1.some((s) => s.includes("comment.block"))
    );
    check("nested block comment: closing line is comment.block", line2.some((s) => s.includes("comment.block")));
  }

  // Single-quoted string gets string.quoted scope.
  {
    const [tokens] = tokenizeAll(grammar, ["PRINT LIST='hello'"]);
    const scopes = scopesAt(tokens, 13);
    check("quoted string scoped as string.quoted", scopes.some((s) => s.includes("string.quoted")));
  }

  // @variable@ substitution gets a distinct, theme-recognized scope (the
  // same variable.other.readwrite convention shell-script grammars use for
  // $VAR, chosen 2026-08-10 after the original variable.other.substitution
  // leaf scope was found -- via real manual VS Code testing -- to render
  // with no color under a stock dark theme).
  {
    const [tokens] = tokenizeAll(grammar, ["RUN PGM=@MY_VAR@"]);
    const scopes = scopesAt(tokens, 10);
    check(
      "@variable@ name scoped as variable.other.readwrite",
      scopes.some((s) => s.includes("variable.other.readwrite"))
    );
    const delimScopes = scopesAt(tokens, 8);
    check(
      "@variable@ delimiter scoped as punctuation.definition.variable",
      delimScopes.some((s) => s.includes("punctuation.definition.variable"))
    );
  }

  // A general (non-block-structural) statement word like PRINT gets its own
  // distinct scope, separate from true control-flow keywords -- added
  // 2026-08-10, real-usage-evidenced (see the grammar's own "statement-words"
  // comment for the census this is sourced from).
  {
    const [tokens] = tokenizeAll(grammar, ["PRINT LIST='hello'"]);
    const scopes = scopesAt(tokens, 0);
    check("PRINT scoped as support.function", scopes.some((s) => s.includes("support.function")));
    check("PRINT is NOT scoped as keyword.control", !scopes.some((s) => s.includes("keyword.control")));
  }

  // Richer, Python-inspired highlighting tier, added 2026-08-10.

  // A pair keyword's own name is scoped like a Python keyword argument.
  {
    const [tokens] = tokenizeAll(grammar, ["RUN PGM=MATRIX"]);
    const scopes = scopesAt(tokens, 4); // "P" of PGM.
    check("PGM (a pair keyword name) scoped as variable.parameter", scopes.some((s) => s.includes("variable.parameter")));
  }

  // A subscripted pair keyword's whole name, brackets included, still counts.
  {
    const [tokens] = tokenizeAll(grammar, ["PATHLOAD VOL[01]=mw[01]"]);
    const scopes = scopesAt(tokens, 9); // "V" of VOL[01].
    check("VOL[01] scoped as variable.parameter", scopes.some((s) => s.includes("variable.parameter")));
  }

  // A control word that's also immediately followed by = (the PHASE=
  // shortcut) keeps its more specific keyword.control scope, not
  // variable.parameter -- array order in the top-level patterns list
  // resolves the tie.
  {
    const [tokens] = tokenizeAll(grammar, ["PHASE=INPUT"]);
    const scopes = scopesAt(tokens, 0);
    check("PHASE= keeps keyword.control over variable.parameter", scopes.some((s) => s.includes("keyword.control")));
    check("PHASE= is NOT scoped as variable.parameter", !scopes.some((s) => s.includes("variable.parameter")));
  }

  // Numeric literals get their own scope.
  {
    const [tokens] = tokenizeAll(grammar, ["ZONES = 5"]);
    const scopes = scopesAt(tokens, 8); // "5".
    check("5 scoped as constant.numeric", scopes.some((s) => s.includes("constant.numeric")));
  }

  // A negative number literal is one numeric token, sign included.
  {
    const [tokens] = tokenizeAll(grammar, ["ZONES=-5"]);
    const scopes = scopesAt(tokens, 6); // the "-" of -5.
    check("the sign of -5 is part of the numeric token", scopes.some((s) => s.includes("constant.numeric")));
  }

  // Subtraction: 2-2 is number, operator, number -- not one negative number.
  {
    const [tokens] = tokenizeAll(grammar, ["MW[1] = 2-2"]);
    const minusScopes = scopesAt(tokens, 9); // the "-" between the two 2s.
    check("the - in 2-2 is scoped as keyword.operator, not part of a number", minusScopes.some((s) => s.includes("keyword.operator")));
  }

  // Arithmetic/comparison operators get their own scope.
  {
    const [tokens] = tokenizeAll(grammar, ["MW[1] = 2 + 2"]);
    const scopes = scopesAt(tokens, 10); // "+".
    check("+ scoped as keyword.operator", scopes.some((s) => s.includes("keyword.operator")));
  }

  // Brackets on a pair's *value* side (not immediately followed by =, so not
  // absorbed whole into a #pair-keywords match the way VOL[01]= itself is)
  // get their own punctuation scope.
  {
    const [tokens] = tokenizeAll(grammar, ["PATHLOAD VOL[01]=mw[01]"]);
    const openBracket = scopesAt(tokens, 19); // the "[" of mw[01].
    check("[ scoped as punctuation.definition.array.begin", openBracket.some((s) => s.includes("punctuation.definition.array.begin")));
  }

  // Commas get their own punctuation scope.
  {
    const [tokens] = tokenizeAll(grammar, ["FILEI DBI[1] = 'x.dbf', SORT=TAZID"]);
    const comma = scopesAt(tokens, 22); // ",".
    check(", scoped as punctuation.separator.comma", comma.some((s) => s.includes("punctuation.separator.comma")));
  }

  // A pair's bareword VALUE (not just its keyword name) gets its own scope
  // too -- added 2026-08-10, e.g. PGM=MATRIX's MATRIX (a real Cube Voyager
  // program name, but not a closed vocabulary this grammar hand-lists).
  {
    const [tokens] = tokenizeAll(grammar, ["RUN PGM=MATRIX"]);
    const scopes = scopesAt(tokens, 8); // "M" of MATRIX.
    check("MATRIX (a pair's value) scoped as constant.other", scopes.some((s) => s.includes("constant.other")));
  }

  // A string's own \n escape gets a distinct scope, inside the string.
  {
    const [tokens] = tokenizeAll(grammar, ["PRINT LIST='line one\\nline two'"]);
    const scopes = scopesAt(tokens, 21); // the "\" of \n.
    check("\\n inside a string scoped as constant.character.escape", scopes.some((s) => s.includes("constant.character.escape")));
    check("\\n's scope is still nested inside string.quoted", scopes.some((s) => s.includes("string.quoted")));
  }

  if (failures > 0) {
    console.error(`${failures} check(s) failed`);
    process.exit(1);
  }
  console.log("all grammar checks passed");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
