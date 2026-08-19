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
  // with no color under a stock dark theme). The whole match, delimiters
  // included, shares this one scope (issue #2) rather than splitting the
  // @ delimiters into their own punctuation.definition.variable scope --
  // themes commonly color that differently from the name, which is exactly
  // the "only the token part is highlighted" bug issue #2 reported.
  {
    const [tokens] = tokenizeAll(grammar, ["RUN PGM=@MY_VAR@"]);
    const nameScopes = scopesAt(tokens, 10);
    check(
      "@variable@ name scoped as variable.other.readwrite",
      nameScopes.some((s) => s.includes("variable.other.readwrite"))
    );
    const openDelimScopes = scopesAt(tokens, 8);
    check(
      "@variable@ opening delimiter scoped as variable.other.readwrite too",
      openDelimScopes.some((s) => s.includes("variable.other.readwrite"))
    );
    const closeDelimScopes = scopesAt(tokens, 15);
    check(
      "@variable@ closing delimiter scoped as variable.other.readwrite too",
      closeDelimScopes.some((s) => s.includes("variable.other.readwrite"))
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

  // -- 024-function-call-highlighting --
  //
  // A built-in function call now gets support.function regardless of where
  // in the statement it sits (fixing the bug this feature exists for: only
  // REPLACESTR used to render colored, and only by accident, when it
  // happened to sit immediately after "=" and got caught by #pair-values).

  // A function nested inside a condition, itself nested inside another call.
  {
    const line = "if (RIGHTSTR(TRIM(RouteName),1)='-')";
    const [tokens] = tokenizeAll(grammar, [line]);
    const outer = scopesAt(tokens, line.indexOf("RIGHTSTR"));
    const inner = scopesAt(tokens, line.indexOf("TRIM"));
    check("RIGHTSTR (nested in a condition) scoped as support.function", outer.some((s) => s.includes("support.function")));
    check("TRIM (nested inside RIGHTSTR's call) scoped as support.function", inner.some((s) => s.includes("support.function")));
  }

  // Two functions nested inside one condition; the @variable@ substitution
  // inside stays unaffected.
  {
    const line = "if (STRLEN(TRIM(@SEGIDExField@))>0)";
    const [tokens] = tokenizeAll(grammar, [line]);
    const strlen = scopesAt(tokens, line.indexOf("STRLEN"));
    const trim = scopesAt(tokens, line.indexOf("TRIM"));
    const variable = scopesAt(tokens, line.indexOf("@SEGIDExField@"));
    check("STRLEN scoped as support.function", strlen.some((s) => s.includes("support.function")));
    check("TRIM (nested inside STRLEN's call) scoped as support.function", trim.some((s) => s.includes("support.function")));
    check("@SEGIDExField@ still scoped as variable.other.readwrite", variable.some((s) => s.includes("variable.other.readwrite")));
  }

  // A function call on an assignment's right-hand side -- REPLACESTR already
  // rendered colored before this feature (via the unrelated #pair-values
  // accident); confirm it still does, now via #function-calls.
  {
    const line = "RouteName = REPLACESTR(RouteName,'-','',0)";
    const [tokens] = tokenizeAll(grammar, [line]);
    const scopes = scopesAt(tokens, line.indexOf("REPLACESTR"));
    check("REPLACESTR (assignment RHS) scoped as support.function", scopes.some((s) => s.includes("support.function")));
  }

  // A function call whose argument is a data reference, not another call --
  // the data reference itself is untouched.
  {
    const line = "ANGLE = ROUND(_L.S_Angle * 10) / 10";
    const [tokens] = tokenizeAll(grammar, [line]);
    const roundScopes = scopesAt(tokens, line.indexOf("ROUND"));
    const dataRefScopes = scopesAt(tokens, line.indexOf("_L.S_Angle"));
    check("ROUND scoped as support.function", roundScopes.some((s) => s.includes("support.function")));
    check("_L.S_Angle (a data reference, not a function) is NOT scoped as support.function", !dataRefScopes.some((s) => s.includes("support.function")));
  }

  // Matching is case-insensitive, same as every other word list in this
  // grammar -- real corpus usage writes this one in mixed case.
  {
    const line = "X = CmpNumRetNum(V,'=',0,1,V)";
    const [tokens] = tokenizeAll(grammar, [line]);
    const scopes = scopesAt(tokens, line.indexOf("CmpNumRetNum"));
    check("CmpNumRetNum (mixed case) scoped as support.function", scopes.some((s) => s.includes("support.function")));
  }

  // Vendor-reference-only functions with no WF-TDM-Official-Releases corpus
  // occurrence at all still scope correctly -- this is the check that would
  // have failed under this feature's original, corpus-only 21-name draft
  // (research.md Sec 1).
  {
    const line1 = "Y = SUBSTR(street,4,6)";
    const [tokens1] = tokenizeAll(grammar, [line1]);
    const substrScopes = scopesAt(tokens1, line1.indexOf("SUBSTR"));
    check("SUBSTR (no corpus evidence) scoped as support.function", substrScopes.some((s) => s.includes("support.function")));

    const line2 = "Z = ARCSIN(0.5)";
    const [tokens2] = tokenizeAll(grammar, [line2]);
    const arcsinScopes = scopesAt(tokens2, line2.indexOf("ARCSIN"));
    check("ARCSIN (no corpus evidence) scoped as support.function", arcsinScopes.some((s) => s.includes("support.function")));
  }

  // A real CONVERGE-phase usage line from the reference guide (research.md
  // Sec 2, CONVERGE-phase family) -- BALANCE is not one of the 138
  // recognized names, so it is NOT scoped as support.function.
  {
    const line = "IF (GAPCHANGEAVE(3) < 0.006 && GAPCHANGEMAX(3) < 0.009) BALANCE = 1";
    const [tokens] = tokenizeAll(grammar, [line]);
    const ave = scopesAt(tokens, line.indexOf("GAPCHANGEAVE"));
    const max = scopesAt(tokens, line.indexOf("GAPCHANGEMAX"));
    const balance = scopesAt(tokens, line.indexOf("BALANCE"));
    check("GAPCHANGEAVE(3) scoped as support.function", ave.some((s) => s.includes("support.function")));
    check("GAPCHANGEMAX(3) scoped as support.function", max.some((s) => s.includes("support.function")));
    check("BALANCE is NOT scoped as support.function", !balance.some((s) => s.includes("support.function")));
  }

  // Data-driven: every one of the 138 recognized function names (data-model.md
  // Sec 1 / research.md Sec 2 -- keep this array in sync with the grammar's
  // own #function-calls alternation, the same manual-sync convention already
  // accepted for #control-words mirroring voyager-core's FIXED_KEYWORDS)
  // scopes as support.function when called. This is what makes spec.md's
  // SC-001 ("every function name... verified") literally true, not just true
  // for the hand-picked scenarios above.
  {
    const allFunctionNames = [
      "AADAVE", "AADCHANGE", "AADCHANGEAVE", "AADCHANGEMAX", "AADCHANGEMIN", "AADMAX", "AADMIN",
      "ABS", "ARCCOS", "ARCSIN", "ARCTAN", "ARRAYSUM", "BRDINGS", "BRDPEN", "CAPACITYFOR",
      "CHECKNAME", "CMPNUMRETNUM", "COMPCOST", "COS", "CURRENTTIME", "CWDCOSTP", "CWDWAITA",
      "CWDWAITP", "DELETESTR", "DIST", "DUPSTR", "EXP", "EXPDIST", "EXPINV", "FAREA", "FAREP",
      "FILESEXIST", "FIRSTREADYNODE", "FORMAT", "FORMATDATETIME", "GAMMADIST", "GAMMAINV",
      "GAPAVE", "GAPCHANGE", "GAPCHANGEAVE", "GAPCHANGEMAX", "GAPCHANGEMIN", "GAPMAX", "GAPMIN",
      "GCOST", "GETMATRIXROW", "GETVALUE", "INLIST", "INSERTSTR", "INT", "IWAITA", "IWAITP",
      "LEFTSTR", "LINKNUM", "LN", "LOG", "LOGNORMDIST", "LOGNORMINV", "LOWEST", "LTRIM",
      "MATVAL", "MAX", "MIN", "NORMDIST", "NORMINV", "NUMREADYNODES", "PATHTRACE", "PDIFFAVE",
      "PDIFFCHANGE", "PDIFFCHANGEAVE", "PDIFFCHANGEMAX", "PDIFFCHANGEMIN", "PDIFFMAX",
      "PDIFFMIN", "POISSONDIST", "POISSONINV", "POW", "PRINTPROGRESS", "RAADAVE", "RAADCHANGE",
      "RAADCHANGEAVE", "RAADCHANGEMAX", "RAADCHANGEMIN", "RAADMAX", "RAADMIN", "RAND", "RANDOM",
      "RANDSEED", "REPLACESTR", "REPLACESTRIC", "REVERSESTR", "RGAPAVE", "RGAPCHANGE",
      "RGAPCHANGEAVE", "RGAPCHANGEMAX", "RGAPCHANGEMIN", "RGAPMAX", "RGAPMIN", "RIGHTSTR",
      "RMSEAVE", "RMSECHANGE", "RMSECHANGEAVE", "RMSECHANGEMAX", "RMSECHANGEMIN", "RMSEMAX",
      "RMSEMIN", "ROUND", "ROWADD", "ROWAVE", "ROWCNT", "ROWDIV", "ROWFAC", "ROWFIX", "ROWMAX",
      "ROWMIN", "ROWMPY", "ROWREAD", "ROWSUM", "SIN", "SPEEDFOR", "SQRT", "STR", "STRLEN",
      "STRLOWER", "STRPOS", "STRPOSEX", "STRUPPER", "SUBSTR", "TAN", "TIMEA", "TIMEP", "TRIM",
      "VAL", "VALOFCHOICE", "XFERPENA", "XFERPENP", "XWAITA", "XWAITP",
    ];
    check("recognized function name list has 138 entries", allFunctionNames.length === 138);

    let allScoped = true;
    const misses: string[] = [];
    for (const name of allFunctionNames) {
      const line = `X = ${name}(1,2,3)`;
      const [tokens] = tokenizeAll(grammar, [line]);
      const scopes = scopesAt(tokens, line.indexOf(name));
      if (!scopes.some((s) => s.includes("support.function"))) {
        allScoped = false;
        misses.push(name);
      }
    }
    check(
      `all ${allFunctionNames.length} recognized function names scope as support.function when called${
        misses.length ? ` (missed: ${misses.join(", ")})` : ""
      }`,
      allScoped
    );
  }

  // A recognized function name with no following "(" is never miscolored --
  // e.g. a keyword=value pair literally named MAX, with no call present.
  {
    const line = "MAX = 100";
    const [tokens] = tokenizeAll(grammar, [line]);
    const scopes = scopesAt(tokens, line.indexOf("MAX"));
    check("MAX with no following ( is NOT scoped as support.function", !scopes.some((s) => s.includes("support.function")));
  }

  // BESTJRNY is a real, vendor-documented Public Transport skim value that
  // is conventionally used *without* a trailing "(...)" -- deliberately
  // excluded from the 138-name list (data-model.md Sec 1; research.md Sec 2)
  // since this pattern's entire mechanism keys off the "(" lookahead.
  {
    const line = "MW[5] = BESTJRNY";
    const [tokens] = tokenizeAll(grammar, [line]);
    const scopes = scopesAt(tokens, line.indexOf("BESTJRNY"));
    check("bare BESTJRNY (no call) is NOT scoped as support.function", !scopes.some((s) => s.includes("support.function")));
  }

  // A function-shaped substring inside a quoted string is never reachable --
  // #function-calls is top-level-only, the same string-safety guarantee
  // #pair-values already documents for itself.
  {
    const line = "PRINT LIST='calling REPLACESTR(x) here'";
    const [tokens] = tokenizeAll(grammar, [line]);
    const scopes = scopesAt(tokens, line.indexOf("REPLACESTR"));
    check("REPLACESTR inside a quoted string is NOT scoped as support.function", !scopes.some((s) => s.includes("support.function")));
    check("REPLACESTR inside a quoted string is still inside string.quoted", scopes.some((s) => s.includes("string.quoted")));
  }

  // -- 026-highlight-customization --
  //
  // #statement-words and #function-calls now use two distinct scopes
  // (support.function.statement.drut / support.function.builtin.drut)
  // instead of one shared support.function.drut, so drut.highlight.
  // statementWords and drut.highlight.functionCalls can color them
  // independently.
  {
    const line = "PRINT LIST='x'";
    const [tokens] = tokenizeAll(grammar, [line]);
    const scopes = scopesAt(tokens, line.indexOf("PRINT"));
    check("PRINT (statement word) scoped as support.function.statement", scopes.some((s) => s.includes("support.function.statement")));
    check("PRINT (statement word) is NOT scoped as support.function.builtin", !scopes.some((s) => s.includes("support.function.builtin")));
  }
  {
    const line = "X = REPLACESTR(y,'-','',0)";
    const [tokens] = tokenizeAll(grammar, [line]);
    const scopes = scopesAt(tokens, line.indexOf("REPLACESTR"));
    check("REPLACESTR (function call) scoped as support.function.builtin", scopes.some((s) => s.includes("support.function.builtin")));
    check("REPLACESTR (function call) is NOT scoped as support.function.statement", !scopes.some((s) => s.includes("support.function.statement")));
  }

  // -- 028-identifier-highlighting --
  //
  // The data-reference family (MI/MO/MW/LI/LW/NI/NW/ZI/ZONES/Z/DBI/DBA/RO/
  // A/B/I/J) now gets its own variable.language.data-reference scope,
  // regardless of position -- fixing the reported bug where DBA only
  // rendered by accident when immediately after "=".

  // DBA scopes the same whether it's a pair value or a function-call argument.
  {
    const line1 = "X = DBA.2.field";
    const [tokens1] = tokenizeAll(grammar, [line1]);
    const afterEquals = scopesAt(tokens1, line1.indexOf("DBA"));
    check("DBA after = scoped as variable.language.data-reference", afterEquals.some((s) => s.includes("variable.language.data-reference")));

    const line2 = "VOL_COR = ROUND(DBA.2.VOL[numrec]) / 100";
    const [tokens2] = tokenizeAll(grammar, [line2]);
    const insideCall = scopesAt(tokens2, line2.indexOf("DBA"));
    check(
      "DBA inside a function-call argument ALSO scoped as variable.language.data-reference (the reported gap)",
      insideCall.some((s) => s.includes("variable.language.data-reference"))
    );
  }

  // DBI on a LOOP opener's own bound expression -- not a keyword=value pair
  // shape, so #pair-keywords/#pair-values could never reach it either.
  {
    const line = "LOOP NUMREC = counter, DBI.2.NUMRECORDS";
    const [tokens] = tokenizeAll(grammar, [line]);
    const scopes = scopesAt(tokens, line.indexOf("DBI"));
    check("DBI on a LOOP opener's bound expression scoped as variable.language.data-reference", scopes.some((s) => s.includes("variable.language.data-reference")));
  }

  // ZONES is both a recognized data-reference name and pair-keyword-shaped
  // (immediately followed by =) -- #data-references, listed first, wins.
  {
    const line = "RUN PGM=MATRIX ZONES=5";
    const [tokens] = tokenizeAll(grammar, [line]);
    const scopes = scopesAt(tokens, line.indexOf("ZONES"));
    check("ZONES scoped as variable.language.data-reference, not variable.parameter", scopes.some((s) => s.includes("variable.language.data-reference")));
    check("ZONES is NOT scoped as variable.parameter", !scopes.some((s) => s.includes("variable.parameter")));
  }

  // A ShellEscape line (leading "*") is raw OS shell text -- the whole line
  // scopes as meta.embedded.shell-escape, and A/B (recognized data-reference
  // link-endpoint names) do NOT scope as variable.language.data-reference
  // inside it.
  {
    const line = "*copy A B";
    const [tokens] = tokenizeAll(grammar, [line]);
    const wholeLine = scopesAt(tokens, 0);
    const aScopes = scopesAt(tokens, line.indexOf("A"));
    check("ShellEscape line scoped as meta.embedded.shell-escape", wholeLine.some((s) => s.includes("meta.embedded.shell-escape")));
    check("A inside a ShellEscape line is NOT scoped as variable.language.data-reference", !aScopes.some((s) => s.includes("variable.language.data-reference")));
  }

  // A Label declaration scopes as entity.name.label, not as a data reference
  // or (once #user-identifiers exists below) a user variable.
  {
    const line = ":STEP0";
    const [tokens] = tokenizeAll(grammar, [line]);
    const scopes = scopesAt(tokens, line.indexOf("STEP0"));
    check("Label name STEP0 scoped as entity.name.label", scopes.some((s) => s.includes("entity.name.label")));
  }

  // A bareword identifier not claimed by any earlier, more specific pattern
  // now gets variable.other.identifier -- fixing the reported inconsistency
  // where _ANode (right after "=") rendered as a pair-value but _BNode (an
  // operand two tokens later) fell through to variable.other.identifier.
  // #pair-values now excludes a bareword that's itself followed by more
  // expression (an operator/quote/paren/bracket), so both operands of the
  // same expression render identically -- neither is a real keyword=value
  // pair.
  {
    const line = "LINKID = _ANode + '_' + _BNode";
    const [tokens] = tokenizeAll(grammar, [line]);
    const aNodeScopes = scopesAt(tokens, line.indexOf("_ANode"));
    const bNodeScopes = scopesAt(tokens, line.indexOf("_BNode"));
    check("_ANode (followed by more expression) scoped as variable.other.identifier", aNodeScopes.some((s) => s.includes("variable.other.identifier")));
    check("_ANode is NOT scoped as constant.other", !aNodeScopes.some((s) => s.includes("constant.other")));
    check("_BNode (an expression operand) scoped as variable.other.identifier", bNodeScopes.some((s) => s.includes("variable.other.identifier")));
  }

  // A bareword that IS the entire assignment right-hand side (nothing
  // follows) still can't be told apart from a genuine keyword=value pair
  // value by a grammar with no real parse tree -- documented trade-off,
  // unchanged by the fix above.
  {
    const line = "X = _ANode";
    const [tokens] = tokenizeAll(grammar, [line]);
    const scopes = scopesAt(tokens, line.indexOf("_ANode"));
    check("_ANode as a whole-RHS copy still scoped as constant.other (documented trade-off)", scopes.some((s) => s.includes("constant.other")));
  }

  // A name already owned by a more specific category never falls through to
  // variable.other.identifier -- comprehensive negative check across every
  // other category (FR-004's full exclusion list).
  {
    const line = "IF (X=1) PRINT LIST=ROUND(DBA.1.field)";
    const [tokens] = tokenizeAll(grammar, [line]);
    const ifScopes = scopesAt(tokens, line.indexOf("IF"));
    const printScopes = scopesAt(tokens, line.indexOf("PRINT"));
    const roundScopes = scopesAt(tokens, line.indexOf("ROUND"));
    const listScopes = scopesAt(tokens, line.indexOf("LIST"));
    const dbaScopes = scopesAt(tokens, line.indexOf("DBA"));
    check("IF (control word) is NOT scoped as variable.other.identifier", !ifScopes.some((s) => s.includes("variable.other.identifier")));
    check("PRINT (statement word) is NOT scoped as variable.other.identifier", !printScopes.some((s) => s.includes("variable.other.identifier")));
    check("ROUND (function call) is NOT scoped as variable.other.identifier", !roundScopes.some((s) => s.includes("variable.other.identifier")));
    check("LIST (pair-keyword name) is NOT scoped as variable.other.identifier", !listScopes.some((s) => s.includes("variable.other.identifier")));
    check("DBA (data-reference name) is NOT scoped as variable.other.identifier", !dbaScopes.some((s) => s.includes("variable.other.identifier")));
  }

  // Neither new category reaches inside a quoted string (FR-008;
  // /speckit-analyze finding E1) -- inherited from #strings' existing
  // begin/end nesting, but verified directly for both new categories here.
  {
    const line = "PRINT LIST='DBA and _BNode'";
    const [tokens] = tokenizeAll(grammar, [line]);
    const dbaScopes = scopesAt(tokens, line.indexOf("DBA"));
    const bNodeScopes = scopesAt(tokens, line.indexOf("_BNode"));
    check("DBA inside a quoted string is NOT scoped as variable.language.data-reference", !dbaScopes.some((s) => s.includes("variable.language.data-reference")));
    check("_BNode inside a quoted string is NOT scoped as variable.other.identifier", !bNodeScopes.some((s) => s.includes("variable.other.identifier")));
    check("both stay inside string.quoted", dbaScopes.some((s) => s.includes("string.quoted")) && bNodeScopes.some((s) => s.includes("string.quoted")));
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
