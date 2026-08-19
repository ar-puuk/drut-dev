# Data Model: Data-Reference & User-Variable Highlighting

## 1. `HighlightCategory`/`CATEGORY_SCOPES` (extended, `editors/vscode/src/highlightCustomization.ts`)

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
  | "strings"
  | "dataReferences"    // NEW
  | "userVariables";    // NEW

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
  dataReferences: "variable.language.data-reference.drut",  // NEW
  userVariables: "variable.other.identifier.drut",          // NEW
};
```

No other change to `highlightCustomization.ts`, `extension.ts`, or `package.json`'s
`contributes.configuration` wiring beyond two more `drut.highlight.<category>` entries
mirroring the existing 9 exactly (`applyHighlightCustomizations` already iterates
`Object.keys(CATEGORY_SCOPES)` generically — research.md §1).

## 2. `drut.tmLanguage.json` repository additions

Two content-matching patterns (data-model, not code — see `contracts/` for the literal
regex source strings, research.md §2/§4 for derivation):

- **`#data-references`**: one `match` rule, scope `variable.language.data-reference.drut`
  — matches the 17-name data-reference family (`MI`/`MO`/`MW`/`LI`/`LW`/`NI`/`NW`/`ZI`/
  `ZONES`/`Z`/`DBI`/`DBA`/`RO`/`A`/`B`/`I`/`J`), case-insensitively, by exact name or
  dot-notation prefix.
- **`#user-identifiers`**: one `match` rule, scope `variable.other.identifier.drut` —
  matches any remaining bareword identifier shape, placed last in the top-level
  `patterns` array so it only ever sees what every earlier, more specific pattern left
  unclaimed.

Two more supporting patterns, needed so the above two (in particular the aggressive
catch-all) don't reach into non-Voyager-syntax content (research.md §5):

- **`#shell-escape`**: one `match` rule, scope `meta.embedded.shell-escape.drut` —
  consumes an entire `ShellEscape` statement's physical line (leading `*` included) as
  one opaque region.
- **`#label`**: one `match` rule, scope `entity.name.label.drut` — consumes a `Label`
  statement's `:name` shape.

Top-level `patterns` array order (only the 4 new entries are additions; every existing
entry keeps its current relative order except where a new entry is interleaved):

```text
#comments
#shell-escape        (NEW)
#label                (NEW)
#strings
#variable-ref
#control-words
#statement-words
#function-calls
#data-references       (NEW)
#pair-keywords
#pair-values
#numbers
#operators
#punctuation
#user-identifiers        (NEW, last)
```

## 3. `package.json` `contributes.configuration` additions

Two entries, same shape as the existing 9 `drut.highlight.*` string settings:

```jsonc
"drut.highlight.dataReferences": {
  "type": "string",
  "markdownDescription": "Color for the Matrix/Line/Node/Zone/Database data-reference family (`MI`, `MW`, `DBA`, `ZONES`, ...) — a CSS color such as `#RRGGBB`. Leave unset to keep your color theme's own color for this category (scope `variable.language.data-reference.drut`). Personal setting only — there is no `drut.toml` equivalent."
},
"drut.highlight.userVariables": {
  "type": "string",
  "markdownDescription": "Color for a user-defined identifier that isn't any recognized keyword, function call, pair-keyword, or data-reference name — a CSS color such as `#RRGGBB`. Leave unset to keep your color theme's own color for this category (scope `variable.other.identifier.drut`). Personal setting only — there is no `drut.toml` equivalent."
}
```
