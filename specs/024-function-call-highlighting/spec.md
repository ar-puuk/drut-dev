# Feature Specification: Function-Call Syntax Highlighting

**Feature Branch**: `024-function-call-highlighting`

**Created**: 2026-08-18

**Status**: Draft

**Input**: User description: "Some keywords in Cube Voyager such as REPLACESTR is highlighted in blue, but others equally important ones like RIGHTSTR, TRIM, and STRLEN (and other such functions) are just plain white like normal text — we need to color them accordingly, like a function."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A built-in function reads the same color everywhere it appears (Priority: P1)

A script author opens a real `.s`/`.block` file in the editor. It contains a Cube Voyager
built-in function call — say `RIGHTSTR(TRIM(RouteName),1)` inside an `IF` condition, or
`REPLACESTR(RouteName,'-','',0)` on an assignment's right-hand side. Today, `REPLACESTR`
happens to render in a distinct color while `RIGHTSTR` and `TRIM` — sitting one token
deeper in the same expression — render as plain, unstyled text, for no reason the author
can see (all three are equally real built-in functions). The author needs every recognized
built-in function to render with the same distinct color, regardless of where in the
statement it sits.

**Why this priority**: This is the entire content of the request — the inconsistency is
the bug. Without this story there is no feature.

**Independent Test**: Open a fixture containing `RIGHTSTR(TRIM(RouteName),1)`,
`REPLACESTR(RouteName,'-','',0)`, and `IF (STRLEN(TRIM(@SEGIDExField@))>0)` side by side.
Confirm `RIGHTSTR`, `TRIM`, `REPLACESTR`, and `STRLEN` all render in the same function
color, independent of nesting depth or statement position.

**Acceptance Scenarios**:

1. **Given** a line reading `RouteName = REPLACESTR(RouteName,'-','',0)`, **When** the file
   is opened in the editor, **Then** `REPLACESTR` renders in the function color (as it
   already does today, incidentally).
2. **Given** a line reading `if (RIGHTSTR(TRIM(RouteName),1)='-')`, **When** the file is
   opened in the editor, **Then** both `RIGHTSTR` and `TRIM` render in the same function
   color as `REPLACESTR` does in Scenario 1 — today they render unstyled.
3. **Given** a line reading `if (STRLEN(TRIM(@SEGIDExField@))>0)`, **When** the file is
   opened in the editor, **Then** both `STRLEN` and `TRIM` render in the function color.
4. **Given** a line reading `ANGLE = ROUND(_L.S_Angle * 10) / 10`, **When** the file is
   opened in the editor, **Then** `ROUND` renders in the function color and `_L.S_Angle`
   (a data reference, not a function) does not.

---

### User Story 2 - A function name used as something other than a call keeps its ordinary color (Priority: P2)

A script author has a matrix or array whose name happens to coincide with a recognized
function name in some other, unrelated position (e.g. a `keyword=value` pair name, or a
bareword that isn't immediately followed by `(`). The author needs highlighting to key off
the actual call shape (name immediately followed by `(`), not merely off the word's
spelling, so an unrelated use of the same word never gets miscolored as a function call.

**Why this priority**: Protects against a regression the fix could easily introduce
(over-eager whole-word matching) — secondary to Story 1, but required for the fix to be
trustworthy on real scripts.

**Independent Test**: Confirm a bare occurrence of a recognized function name with no
following `(` — e.g. as a `keyword=value` pair's keyword, or as a plain identifier — never
renders in the function color.

**Acceptance Scenarios**:

1. **Given** a line reading `MAX = 100` (a plain assignment whose left-hand identifier
   happens to spell a recognized function name, immediately followed by whitespace and `=`,
   never by `(`), **When** the file is opened in the editor, **Then** `MAX` does not render
   in the function color.

---

### Edge Cases

- What happens when a recognized function name is immediately followed by `(` but with
  intervening whitespace, e.g. `ROUND (x)`? Both vendor doc mirrors and real corpus usage
  always write the function name directly against its `(` with no space; matching requires
  no intervening whitespace before `(`, keeping the rule unambiguous (a bareword followed
  by whitespace then `(` is far more likely a coincidental adjacency than a call in this
  grammar).
- What happens with a function name written in mixed case, e.g. `CmpNumRetNum(...)` (real
  corpus usage, `crates/voyager-core`'s own case-insensitive keyword convention)? Matching
  is case-insensitive, consistent with every other word-list pattern already in this
  grammar file (`#control-words`, `#statement-words`).
- What happens to a word shaped like a function call but not in the recognized list, e.g. a
  user's own subroutine-like `MYCALC(x)`? It renders unstyled, exactly as before this
  feature existed — this is a non-exhaustive, evidence-only list (see FR-004), the same
  stance `#statement-words` already documents for its own list.
- What happens where a recognized function name is used for genuine array/matrix element
  access rather than a function call, e.g. a user-defined array literally named `MAX`
  indexed as `MAX(1)`? It renders in the function color, indistinguishable from a real call
  — an accepted false positive (Cube Voyager has no user-definable functions, so this
  collision is a naming coincidence, not a structural ambiguity the grammar can resolve;
  see Assumptions).
- What about a vendor-documented function that is conventionally used **without** a
  trailing `(...)` at all (e.g. a skim value referenced as a bare keyword rather than a
  call)? It is excluded from this list entirely — this feature's matching mechanism is
  built entirely around the `(`-immediately-follows trigger (FR-001), so a name that is
  never actually written with a following `(` cannot be reached by it; coloring such a name
  is a `#statement-words`-shaped concern (a bareword-position word list), out of scope for
  this feature (see `research.md`).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The editor grammar MUST render a recognized built-in function name in a
  distinct, consistent color every time it is immediately followed by `(` (no intervening
  whitespace), regardless of where in the statement it appears — nested inside another
  call's arguments, inside an `IF`/short-`IF` condition, on an assignment's right-hand
  side, or anywhere else a value expression is legal.
- **FR-002**: This rendering MUST be visually distinct from `#control-words`
  (`keyword.control.drut`) and MUST NOT depend on the accidental `#pair-values`
  ("bareword immediately after `=`") rule that produces today's inconsistent coloring.
- **FR-003**: Matching MUST be case-insensitive, consistent with every other keyword list
  already in this grammar file.
- **FR-004**: The recognized function name list is a closed, hand-maintained list, not a
  runtime-verified guarantee of every real Cube Voyager built-in function — a genuine
  built-in function not yet in the list simply renders unstyled, exactly as it does today
  before this feature exists (mirrors `#statement-words`' own documented non-exhaustive
  stance). Unlike this grammar's other word lists, this one is deliberately built to be
  broad and agency-agnostic from the start (see FR-005) rather than scoped to one
  organization's own corpus, so that the extension colors built-in functions correctly for
  any Cube Voyager user, not only users whose scripts resemble the corpus this project
  happens to have on hand.
- **FR-005**: The recognized function name list MUST be populated from a complete reading
  of Cube Voyager's own scripting-language function vocabulary across both locally
  available vendor documentation editions (Cube Voyager 6.5.1 and OpenPaths Cube/CUBE
  CONNECT Edition) — the general-purpose Control Language functions (available in any
  `.s`/`.block` script regardless of which program/`PGM=` runs it), the Highway/Matrix
  program functions, the Public Transport skim functions, the CONVERGE-phase
  iteration-statistics function family, and the CUBE Cluster utility functions — identifier
  names only, written in this project's own structure and categorization, never vendor
  documentation prose (constitution Principle II). Deliberately excludes any function or
  method belonging to a separate object-model/scripting-API surface (e.g. a Python/CubePy
  style API) rather than the Voyager control-statement language itself. Real corpus usage
  is cross-checked where available as a secondary confirmation, not as the admission bar.
  See `research.md` for the full source list and methodology.
- **FR-006**: A word that matches a recognized function name but is not immediately
  followed by `(` (e.g. it is a `keyword=value` pair's keyword name, or a bareword with no
  following parenthesis) MUST NOT render in the function color.
- **FR-007**: This feature MUST NOT change `voyager-core`'s tokenizer, parser, or any
  `Diagnostic` category — it is a VS Code editor-highlighting classification only, and every
  `.s`/`.block` file's parse result is unaffected by this change.
- **FR-008**: This feature MUST NOT add a new `drut-config` field, CLI flag, MCP parameter,
  or editor client setting — it corrects highlighting the same shipped `editors/vscode`
  extension already renders under its existing, unversioned grammar.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every one of the 138 function names in the recognized list (`research.md`
  §2) renders in the same distinct color when called, verified directly by an automated,
  data-driven test over the complete list (not a hand-picked sample) — plus targeted
  scenario tests confirming the color is identical regardless of statement position
  (standalone on an assignment right-hand side, nested as another call's argument, or
  inside a condition) for representative names.
- **SC-002**: A script author visually scanning a real `.s`/`.block` file can identify every
  built-in function call from the recognized list at a glance, without needing to check
  whether it happens to sit immediately after an `=`.
- **SC-003**: No existing grammar behavior regresses — every prior grammar test
  (`editors/vscode/test/grammar.test.ts`) continues to pass unchanged.
- **SC-004**: A bareword that spells a recognized function name but is never followed by
  `(` never renders in the function color, in 100% of such occurrences.

## Assumptions

- **Source of the list is deliberately broader than this project's own corpus.** Every
  other word list in this grammar (`#statement-words`, `#pair-keywords`'s structural
  pattern, `voyager-core`'s `PAIR_KEYWORDS`) was built by censusing one organization's real
  scripts, because those positions (a leading statement word, a `keyword=value` pair name)
  are also legitimately occupied by a user's own per-program naming, so frequency across
  independently-authored files was the only available signal separating general vocabulary
  from local convention. A function-call position has no equivalent legitimate-collision
  source — Cube Voyager has no user-definable functions — so this list is instead built
  directly from Cube Voyager's own documented general-purpose scripting vocabulary
  (identifier names only, in this project's own words and categorization — never vendor
  documentation prose, per constitution Principle II), with real corpus usage serving as a
  confirmatory cross-check where available rather than as the admission bar. This keeps the
  extension useful for any Cube Voyager user, not only ones whose scripts happen to
  resemble this project's own reference corpus (the corpus-only, `distinct_files`-threshold
  approach this project's other word lists use was considered and rejected for this
  specific feature for exactly this reason).
- Cube Voyager has no user-definable functions, so a bareword immediately followed by `(`
  with no intervening whitespace is a structurally unambiguous "this position expects a
  function name" slot — the same reasoning already applied to this grammar's `#pair-keywords`
  pattern (a structural shape, not a fixed name list, for the position that follows it).
  A user-chosen array/matrix name that happens to coincide with a recognized function name
  and is also indexed with `(...)` is an accepted, rare false positive (see Edge Cases) —
  the grammar has no way to structurally distinguish the two without semantic analysis,
  which is out of scope for a TextMate grammar (and out of scope for `voyager-core`, which
  performs no semantic validation per the project's grammar model).
- **A complete reading pass, not an infallible one.** Every function-related chapter in
  both vendor doc mirrors was read and either mined or confirmed to contain no additional
  call-shaped functions (`research.md` §2.1) — this is meaningfully stronger than the
  "not exhaustive by construction" stance this grammar's frequency-thresholded word lists
  (`#statement-words`, `PAIR_KEYWORDS`) carry, since those can structurally never see a
  rarely-used function. It is still a manual reading pass over two large documents, not an
  automated, provably-complete extraction, so a genuine miss (a misspelled variant, or a
  function documented in a chapter this pass didn't think to check) remains possible
  (`research.md` §5) — omission from this list is not a claim that a function is
  unsupported or nonstandard, mirroring FR-004's non-exhaustive stance; it is a one-line
  addition to the same flat list when found, the same amendment path `keywords.rs`'s
  `ZONES` addition already established.
- Scope is limited to `editors/vscode/syntaxes/drut.tmLanguage.json` and its
  `editors/vscode/test/grammar.test.ts` coverage. No other adapter (`drut-lsp`, `drut-mcp`,
  `drut-cli`) currently performs syntax-highlighting classification, so none is touched.
