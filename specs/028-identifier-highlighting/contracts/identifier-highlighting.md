# Contract: Data-Reference & User-Variable Highlighting (amends `026-highlight-customization`)

Extends `editors/vscode`'s grammar and settings surface. A conceptual signature
contract, not final JSON/TypeScript source — same convention every prior contract doc in
this repo follows.

## No `voyager-core`/adapter-crate change

- No `voyager-core` function signature, type, or `Diagnostic` category changes.
- No `drut-config`/`drut-cli`/`drut-mcp` field, flag, or parameter is added.
- No `drut-lsp` semantic-token emission change.

## Grammar contract (`drut.tmLanguage.json`)

- **`#data-references`** (new): matches `MI`, `MO`, `MW`, `LI`, `LW`, `NI`, `NW`, `ZI`,
  `ZONES`, `Z`, `DBI`, `DBA`, `RO`, `A`, `B`, `I`, `J` — case-insensitively, by exact
  name or dot-notation prefix (`dba.2.field` → just `dba`) — regardless of surrounding
  syntax position. Scope: `variable.language.data-reference.drut`.
- **`#user-identifiers`** (new): matches any bareword identifier not already claimed by
  an earlier, more specific pattern (control word, statement word, function-call name,
  `#data-references`, pair-keyword-shaped name, pair-value-shaped value). Scope:
  `variable.other.identifier.drut`. Placed last in the top-level `patterns` array —
  ordering *is* the filtering mechanism (research.md §4).
- **`#shell-escape`** (new): consumes an entire `ShellEscape` statement's physical line
  as one opaque region — no other pattern, old or new, matches inside it. Scope:
  `meta.embedded.shell-escape.drut`.
- **`#label`** (new): consumes a `Label` statement's `:name` shape. Scope:
  `entity.name.label.drut`.
- **Precedence**: a name that is both a recognized data-reference name and shape-eligible
  for `pairKeywords`/`pairValues` (e.g. `ZONES` in `RUN PGM=MATRIX ZONES=5`) is claimed by
  `#data-references` — array order, not a regex change to either existing pattern
  (research.md §3).
- **No existing scope, match pattern, or match precedence for any of the 9 pre-existing
  categories changes** — the 4 new patterns are additive; the only observable side effect
  on old behavior is that a `ShellEscape` line's leading `*` (previously matched
  incidentally by `#operators`) is now part of `#shell-escape`'s own scope instead
  (research.md §5 — a small, deliberate correctness fix, not a regression).

## Settings contract (`package.json` / `drut.highlight.*`)

- **2 new settings**: `drut.highlight.dataReferences`, `drut.highlight.userVariables` —
  each an optional string (a CSS color), default unset, Global scope only — same
  personal-setting shape every one of `026`'s 9 categories already uses.
- **Unset is a strict no-op**: with both new settings unset, behavior is byte-identical
  to a build where this feature's code never ran (same `026` SC-002 guarantee, inherited
  for free — `applyHighlightCustomizations` already treats every `CATEGORY_SCOPES` key
  uniformly).
- **Set applies globally, immediately**: setting either recolors every matching token,
  in every open `.s`/`.block` document, without a window reload — same reactivity
  `026`'s 9 categories already have (no new listener needed, `e.affectsConfiguration
  ("drut.highlight")` already covers any new sub-key).
- **Independent of every other `drut.highlight.*` category**, including each other.
- **Never touches a rule it doesn't own**: `mergeHighlightRules`'s existing
  exact-scope-set ownership check (`isOwnedRule`) already generalizes to the two new
  scopes with zero code change (it iterates `ALL_CATEGORIES`, not a hardcoded 9).

## Illustrative examples

| Scenario | Result |
|---|---|
| `drut.highlight.dataReferences` unset | `DBA` in `ROUND(DBA.2.VOL[numrec])` renders in the active theme's own color for `variable.language.data-reference.drut` (nothing, under a theme with no rule for it — same as any other unset category today) |
| `drut.highlight.dataReferences = "#4FC1FF"` | Every `MI`/`MW`/`DBA`/`ZONES`/... occurrence in the file, in any position, renders `#4FC1FF` |
| `drut.highlight.userVariables = "#9CDCFE"` | `_BNode` in `_ANode + '_' + _BNode` renders `#9CDCFE`; `_ANode` (immediately after `=`) keeps rendering under `drut.highlight.values`'s own color, unless that's also set (spec.md Assumptions — documented `=`-adjacency trade-off) |
| A script contains `*copy A B` (`ShellEscape`) | The whole line — `*`, `copy`, `A`, `B` — renders under `meta.embedded.shell-escape.drut`; `A`/`B` do **not** render as `dataReferences` even though those are recognized family names |
| A script contains `:STEP0` (`Label`) | `STEP0` renders under `entity.name.label.drut`, not `userVariables` |
| `RUN PGM=MATRIX ZONES=5` | `ZONES` renders under `dataReferences`'s color (or theme default), not `pairKeywords`'s |
