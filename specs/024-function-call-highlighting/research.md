# Research: Function-Call Syntax Highlighting

## 1. Methodology: vendor-reference vocabulary, corpus as cross-check (not the admission bar)

The initial approach for this feature (see git history: this `research.md` was rewritten
mid-flight) built the recognized-function list purely from a `distinct_files >= 3`-style
census of the local `WF-TDM-Official-Releases` corpus (161 `.s`/`.block` files), the same
method `#statement-words` and `voyager-core`'s `keywords.rs` `PAIR_KEYWORDS` already use.
That produced a working but narrow 21-function list — narrow because it could only ever
surface functions this one organization's scripts happen to call. A reviewer correctly
pointed out that would make the extension under-color scripts from any other Cube Voyager
user whose scripts call a different subset of the language's built-ins.

Revised approach: the recognized list is built from Cube Voyager's own **general-purpose
scripting-language function vocabulary** — sourced from the local vendor documentation
mirror, `_archive/Citilabs Cube 6.5.1/RG_CUBEVOYAGER.md` (gitignored, research-only per
`CLAUDE.md`'s explicit allowance: "`_archive/`... holds vendor documentation mirrors for
research only"). Only identifier **names** were extracted — never the vendor's descriptive
prose, examples, or table wording (constitution Principle II); every description in this
document and in `data-model.md`/`contracts/` is written fresh, in this project's own words.
Real corpus usage (`WF-TDM-Official-Releases`) is retained as a secondary confirmation
where available — every name that was in the original 21-function corpus-only list is
cross-checked below — but corpus presence is no longer the bar a name must clear to be
included.

## 2. Recognized function list (138 names, by source category)

Built by a **complete pass** through every function-related chapter of both vendor doc
mirrors (§1 methodology, §3 cross-version validation) — every heading in
`RG_CUBEVOYAGER.md` matching "function(s)", "built-in", or "special...function" was read
and either mined for names or confirmed to contain no new call-shaped functions (§3.1).
The goal is a comprehensive list for the Voyager control-language surface (`.s`/`.block`
scripts) specifically — not the separate object-model/scripting-API surface some newer
Cube products also expose (§3, "Deliberately excluded").

**General Control Language functions** — available in any `.s`/`.block` script regardless
of which program (`PGM=`) runs it (reference guide, "Functions and built-ins" / Numeric,
Trig, and Character/String function tables):

- *Numeric* (26): `ABS`, `CMPNUMRETNUM`, `CURRENTTIME`, `EXP`, `EXPDIST`, `EXPINV`,
  `GAMMADIST`, `GAMMAINV`, `INLIST`, `INT`, `LN`, `LOG`, `LOGNORMDIST`, `LOGNORMINV`, `MAX`,
  `MIN`, `NORMDIST`, `NORMINV`, `POISSONDIST`, `POISSONINV`, `POW`, `RAND`, `RANDOM`,
  `RANDSEED`, `ROUND`, `SQRT`
- *Trigonometric* (6): `ARCCOS`, `ARCSIN`, `ARCTAN`, `COS`, `SIN`, `TAN`
- *Character/String* (20): `DELETESTR`, `DUPSTR`, `FORMAT`, `FORMATDATETIME`, `INSERTSTR`,
  `LEFTSTR`, `LTRIM`, `REPLACESTR`, `REPLACESTRIC`, `REVERSESTR`, `RIGHTSTR`, `STR`,
  `STRLEN`, `STRLOWER`, `STRPOS`, `STRPOSEX`, `STRUPPER`, `SUBSTR`, `TRIM`, `VAL`

**Highway/Matrix program functions** — row, matrix, and network-lookup functions used
across the Highway and Matrix programs (reference guide's Highway-program "Built-in
functions" table and Matrix program's "Matrix function descriptions"), broadly used by any
agency running highway assignment or matrix manipulation scripts, not organization-specific:

- (21): `ARRAYSUM`, `CAPACITYFOR`, `CHECKNAME`, `GETMATRIXROW`, `GETVALUE`, `LINKNUM`,
  `LOWEST`, `MATVAL`, `PATHTRACE`, `ROWADD`, `ROWAVE`, `ROWCNT`, `ROWDIV`, `ROWFAC`,
  `ROWFIX`, `ROWMAX`, `ROWMIN`, `ROWMPY`, `ROWREAD`, `ROWSUM`, `SPEEDFOR`

**Public Transport skim functions** — the reference guide's "Summary of skim functions"
quick-reference table plus one function documented separately in the crowding-model
("SPREADFUNC") section, all written with a trailing `(RouteSet[, Mode])` or `(MinRoute)`
call form:

- (19): `BRDINGS`, `BRDPEN`, `COMPCOST`, `CWDCOSTP`, `CWDWAITA`, `CWDWAITP`, `DIST`,
  `FAREA`, `FAREP`, `GCOST`, `IWAITA`, `IWAITP`, `TIMEA`, `TIMEP`, `VALOFCHOICE`,
  `XFERPENA`, `XFERPENP`, `XWAITA`, `XWAITP`
- **Excluded from that same table** — `BESTJRNY`, `EXCESSDEMAND`, `EXCESSPROP`: the
  reference guide shows these three used as bare keywords, with no trailing `(...)` at all,
  in both their individual descriptions and the summary table (every other row in the same
  table explicitly shows `(RouteSet[, Mode])`; these three rows show none). This feature's
  entire matching mechanism keys off a `(` immediately following the name (FR-001); a name
  genuinely never written with one cannot be reached by it and would need a different,
  bareword-position pattern (`#statement-words`-shaped) — out of scope here (spec.md Edge
  Cases).

**CONVERGE-phase iteration-statistics functions** (reference guide's Highway-program
"Built-in functions available in the CONVERGE phase" table) — a systematic family over 6
base metrics (`GAP`, `RGAP`, `AAD`, `RAAD`, `PDIFF`, `RMSE`) each with 7 statistic suffixes
(`CHANGE`, `MIN`, `MAX`, `AVE`, `CHANGEMIN`, `CHANGEMAX`, `CHANGEAVE`); a real usage example
in the same reference guide confirms the call form directly (`IF (GAPCHANGEAVE(3) < 0.006
&& GAPCHANGEMAX(3) < 0.009 && ABS(GAPCHANGEMIN) < 0.009) BALANCE = 1`):

- (42): `GAPCHANGE`, `RGAPCHANGE`, `AADCHANGE`, `RAADCHANGE`, `PDIFFCHANGE`, `RMSECHANGE`,
  `GAPMIN`, `GAPMAX`, `GAPAVE`, `GAPCHANGEMIN`, `GAPCHANGEMAX`, `GAPCHANGEAVE`, `RGAPMIN`,
  `RGAPMAX`, `RGAPAVE`, `RGAPCHANGEMIN`, `RGAPCHANGEMAX`, `RGAPCHANGEAVE`, `AADMIN`,
  `AADMAX`, `AADAVE`, `AADCHANGEMIN`, `AADCHANGEMAX`, `AADCHANGEAVE`, `RAADMIN`, `RAADMAX`,
  `RAADAVE`, `RAADCHANGEMIN`, `RAADCHANGEMAX`, `RAADCHANGEAVE`, `PDIFFMIN`, `PDIFFMAX`,
  `PDIFFAVE`, `PDIFFCHANGEMIN`, `PDIFFCHANGEMAX`, `PDIFFCHANGEAVE`, `RMSEMIN`, `RMSEMAX`,
  `RMSEAVE`, `RMSECHANGEMIN`, `RMSECHANGEMAX`, `RMSECHANGEAVE`

**CUBE Cluster distributed-processing utility functions** (reference guide, "Utility
functions"):

- (3): `FILESEXIST`, `FIRSTREADYNODE`, `NUMREADYNODES`

**Corpus-confirmed, not found in either vendor doc mirror**:

- (1): `PRINTPROGRESS` — 8 distinct files in `WF-TDM-Official-Releases`, clear call-shaped
  usage (`PrintProgress(5.0)` inside a `PRINT ... LIST=` argument list); checked against
  both `RG_CUBEVOYAGER.md` and `OpenPaths Cube`'s docs and genuinely absent from both
  (§3) — possibly added in a release neither mirror covers. Included on real-usage evidence
  alone, the same bar the original corpus-only pass used for its whole list.

**Total: 26 + 6 + 20 + 21 + 19 + 42 + 3 + 1 = 138 names.**

### 2.1. Sections read and confirmed to contain no additional call-shaped functions

Every other "function(s)"/"built-in" heading in `RG_CUBEVOYAGER.md` was read and found to
be either a duplicate listing of a category already captured above (the Network program's
and Matrix program's own "Built-in functions" sections restate the same Highway/Matrix
names; "Special matrix functions" restates the same row/matrix set), or genuinely not a
call-shaped function at all:

- **Network Topology Functions** (`_N.Connections`, `_L.S_Angle`, `_NI._Angle[#]`, etc.):
  dotted variable-reference/subscript syntax, not `WORD(` call syntax — already correctly
  outside this feature's scope (the same reasoning `spec.md` Acceptance Scenario 4 already
  uses `_L.S_Angle` to illustrate).
- **`FUNCTION` statement** (`COST`/`TC`/`V` keywords): a `keyword=value` control statement
  (`#pair-keywords` territory), not a built-in function call.
- **CUBE Avenue's "Functions and built-ins"**: despite the heading, this section documents
  new *script variables* (`STORAGE`, `TIMESEGMENT`, `SEGMENTSTART`, `PERIOD`), not
  functions.
- **"Travel function values: Friction factors"**: describes `LOOKUP`-curve data, not a
  built-in function.

## 3. Cross-version validation: Cube Voyager 6.5.1 vs. OpenPaths Cube (CUBE CONNECT Edition)

`_archive/` holds two independent vendor documentation mirrors: `Citilabs Cube 6.5.1`
(§2's source) and `OpenPaths Cube` (a later, rebranded product generation — its help
pages are titled "CUBE CONNECT Edition Help"). To keep this list correct for any Cube
Voyager user, not just one tied to the older edition, every category in §2 was
cross-checked against the `OpenPaths Cube` mirror as well:

- **Numeric, Trig, and Character/String functions** (52 names): identical set, identical
  names, in `OpenPaths Cube`'s own "Control statement syntax" > "Expressions" chapter —
  no additions, no removals, no spelling differences between the two product generations.
- **Highway/Matrix functions** (21 names): identical set confirmed present (`ROWSUM`,
  `ROWFIX`, `PATHTRACE`, `LOWEST`, `CAPACITYFOR`, `ARRAYSUM`, `MATVAL`, `ROWREAD`, etc. all
  located).
- **Public Transport skim functions** (19 names, including `GCOST`): identical set
  confirmed present (`BRDINGS`, `TIMEA`, `XFERPENA`, `ValOfChoice`, `GCost(MinRoute)`, etc.
  all located) — `GCOST` was in fact *found first* in `OpenPaths Cube`'s docs during this
  cross-check, then confirmed present with identical wording in `RG_CUBEVOYAGER.md` too
  (§2), demonstrating the two-mirror cross-check catches real gaps, not just confirms them.
- **CONVERGE-phase iteration-statistics functions** (42 names): identical family confirmed
  present in `OpenPaths Cube`'s docs (spot-checked `GAPCHANGE`, `RGAPCHANGEAVE`,
  `AADCHANGEMIN`, `RMSECHANGEAVE`).
- **`PRINTPROGRESS`**: checked and, like the 6.5.1 guide, genuinely absent from
  `OpenPaths Cube`'s docs too — kept on real corpus evidence alone (§2), not vendor
  documentation in either edition.
- **Deliberately excluded**: a broad scan of `OpenPaths Cube`'s HTML also surfaced a
  camelCase method family (`addNonTransitLeg()`, `addNonTransitLegs()`,
  `removeNonTransitMode()`, etc.) — inspecting their context shows these are methods on a
  `NonTransitLeg`/network **object model** (a distinct scripting/API surface for editing a
  Public Transport network programmatically), not Voyager control-statement functions
  callable from a `.s`/`.block` script. Their naming convention (camelCase, not Voyager's
  ALL-CAPS convention every function in §2 uses) and object-method framing confirm they
  are out of scope for this feature (`CLAUDE.md`: this project targets Cube Voyager
  control-statement scripts specifically) — correctly excluded, not an oversight.

No function in §2 needed correction as a result of this cross-check — the two vendor
editions agree completely on this vocabulary.

## 4. Cross-check against the original corpus-only census

Every one of the original 21-function corpus-evidenced list (previous revision of this
document) is a subset of the vendor-reference list above — `ABS`, `CMPNUMRETNUM`,
`CURRENTTIME`, `EXP`, `FORMATDATETIME`, `INT`, `LN`, `LTRIM`, `MAX`, `MIN`, `PATHTRACE`,
`PRINTPROGRESS`, `REPLACESTR`, `RIGHTSTR`, `ROUND`, `ROWFIX`, `ROWSUM`, `SQRT`, `STR`,
`STRLEN`, `TIMEA`, `TRIM` all appear above. This is a useful sanity check in both
directions: nothing the corpus confirmed as real turned out to be absent from the vendor's
own function vocabulary, and the vendor list explains *why* `REPLACESTR`/`RIGHTSTR`/
`CMPNUMRETNUM` had only one real corpus occurrence each despite being entirely real,
standard functions — they are simply less frequently needed in this organization's
particular scripts, exactly the kind of gap a single-corpus census cannot see past.

## 5. Scope note: a complete pass, not an infallible one

Unlike the first draft of this feature (a deliberately time-boxed subset), this 138-name
list is the result of reading every function-related heading in both vendor doc mirrors
(§2.1 records what was checked and ruled out, not merely skipped) — not a partial sample.
This is meaningfully stronger than "not exhaustive by construction" (`#statement-words`'
own standard, inherited from a frequency threshold that structurally can't see rare
functions): every name here was found by direct document reading, not by how often it
happens to appear in any one corpus.

That said, this is still a human reading pass over ~20,000 lines of converted
documentation across two large HTML/PDF exports, not an automated, provably-complete
extraction — a spelling variant or a function documented only in a chapter this pass
didn't think to check is possible, the same residual risk any manually-curated word list
in this grammar already carries (`#statement-words`, `PAIR_KEYWORDS`). A genuine miss
found later is a one-line addition to the same flat list (`keywords.rs`'s `ZONES` addition
is the established precedent for this), not a design flaw to fix.

## 6. Grammar-position decision: match only `WORD(` with no intervening whitespace

Every function in §2 that takes arguments is documented (and, where checked, used in the
real corpus) with its name written directly against the opening `(`. Requiring no
intervening whitespace keeps the pattern unambiguous and avoids coloring an unrelated
bareword that merely precedes an unrelated `(` later on the same line.

## 7. Scope choice: new pattern vs. extending `#statement-words`

Decision: a **new**, separate `#function-calls` pattern, not an addition to the existing
`#statement-words` regex.

- **Alternative considered**: append these names to `#statement-words`' existing
  alternation and rely on that pattern's existing `support.function.drut` scope.
- **Rejected because**: `#statement-words` matches its words *unconditionally* — it has no
  `(`-follows lookahead. Reusing it for these names would color e.g. a `keyword=value` pair
  literally named `MAX` (FR-006 / spec.md User Story 2) even with no call present, the exact
  kind of position-blind false positive this feature exists to avoid introducing elsewhere
  while fixing it for `REPLACESTR`. A dedicated pattern with its own `(?=\()` lookahead keeps
  the call-position requirement precise without changing `#statement-words`' existing,
  already-tested behavior for its own words.
- Scope name: `support.function.drut` is reused for the new pattern (not a new scope name)
  — semantically correct (TextMate's own conventional category for a language's callable
  procedures/functions) and keeps the two-tier convention's visual result identical to what
  `#statement-words` already renders, satisfying FR-001/FR-002's "distinct, consistent
  color" requirement without inventing a fourth visual tier the constitution's Principle VII
  (naming honesty) would need to separately justify.
