# Data Model: Function-Call Syntax Highlighting

This feature has no runtime data model (no `voyager-core` types, no persisted state) — its
only "data" is the static word list embedded in the TextMate grammar file itself. Documented
here for parity with this project's other spec-kit features.

## 1. `FUNCTION_CALL_NAMES` (conceptual — lives as one regex alternation in
   `drut.tmLanguage.json`, not a Rust/TypeScript type)

| Field | Type | Description |
|---|---|---|
| `name` | string | Canonical uppercase spelling (matching is case-insensitive; see `research.md` §2) |
| `category` | enum | Which reference-guide chapter the name was sourced from (`research.md` §2) — documentation grouping only, not encoded in the grammar itself (the grammar has one flat alternation, no sub-scoping by category) |

**Membership** (138 entries — `research.md` §2, full source list and rationale):

- **Numeric** (26): `ABS`, `CMPNUMRETNUM`, `CURRENTTIME`, `EXP`, `EXPDIST`, `EXPINV`,
  `GAMMADIST`, `GAMMAINV`, `INLIST`, `INT`, `LN`, `LOG`, `LOGNORMDIST`, `LOGNORMINV`, `MAX`,
  `MIN`, `NORMDIST`, `NORMINV`, `POISSONDIST`, `POISSONINV`, `POW`, `RAND`, `RANDOM`,
  `RANDSEED`, `ROUND`, `SQRT`
- **Trigonometric** (6): `ARCCOS`, `ARCSIN`, `ARCTAN`, `COS`, `SIN`, `TAN`
- **Character/String** (20): `DELETESTR`, `DUPSTR`, `FORMAT`, `FORMATDATETIME`,
  `INSERTSTR`, `LEFTSTR`, `LTRIM`, `REPLACESTR`, `REPLACESTRIC`, `REVERSESTR`, `RIGHTSTR`,
  `STR`, `STRLEN`, `STRLOWER`, `STRPOS`, `STRPOSEX`, `STRUPPER`, `SUBSTR`, `TRIM`, `VAL`
- **Highway/Matrix** (21): `ARRAYSUM`, `CAPACITYFOR`, `CHECKNAME`, `GETMATRIXROW`,
  `GETVALUE`, `LINKNUM`, `LOWEST`, `MATVAL`, `PATHTRACE`, `ROWADD`, `ROWAVE`, `ROWCNT`,
  `ROWDIV`, `ROWFAC`, `ROWFIX`, `ROWMAX`, `ROWMIN`, `ROWMPY`, `ROWREAD`, `ROWSUM`,
  `SPEEDFOR`
- **Public Transport skims** (19): `BRDINGS`, `BRDPEN`, `COMPCOST`, `CWDCOSTP`,
  `CWDWAITA`, `CWDWAITP`, `DIST`, `FAREA`, `FAREP`, `GCOST`, `IWAITA`, `IWAITP`, `TIMEA`,
  `TIMEP`, `VALOFCHOICE`, `XFERPENA`, `XFERPENP`, `XWAITA`, `XWAITP`
- **CONVERGE-phase iteration statistics** (42): `GAPCHANGE`, `RGAPCHANGE`, `AADCHANGE`,
  `RAADCHANGE`, `PDIFFCHANGE`, `RMSECHANGE`, `GAPMIN`, `GAPMAX`, `GAPAVE`, `GAPCHANGEMIN`,
  `GAPCHANGEMAX`, `GAPCHANGEAVE`, `RGAPMIN`, `RGAPMAX`, `RGAPAVE`, `RGAPCHANGEMIN`,
  `RGAPCHANGEMAX`, `RGAPCHANGEAVE`, `AADMIN`, `AADMAX`, `AADAVE`, `AADCHANGEMIN`,
  `AADCHANGEMAX`, `AADCHANGEAVE`, `RAADMIN`, `RAADMAX`, `RAADAVE`, `RAADCHANGEMIN`,
  `RAADCHANGEMAX`, `RAADCHANGEAVE`, `PDIFFMIN`, `PDIFFMAX`, `PDIFFAVE`, `PDIFFCHANGEMIN`,
  `PDIFFCHANGEMAX`, `PDIFFCHANGEAVE`, `RMSEMIN`, `RMSEMAX`, `RMSEAVE`, `RMSECHANGEMIN`,
  `RMSECHANGEMAX`, `RMSECHANGEAVE`
- **CUBE Cluster utility** (3): `FILESEXIST`, `FIRSTREADYNODE`, `NUMREADYNODES`
- **Corpus-confirmed, not in either vendor doc mirror** (1): `PRINTPROGRESS`

**Explicitly not a member despite being a plausible-sounding sibling**:

- `RTRIM` (`LTRIM`'s natural counterpart): checked against both the vendor reference guide
  and the real corpus — genuinely absent from both, not merely unlisted.
- `BESTJRNY`, `EXCESSDEMAND`, `EXCESSPROP`: real, vendor-documented Public Transport skim
  values, but conventionally used as bare keywords with no trailing `(...)` — excluded
  because this feature's entire matching mechanism requires an immediately-following `(`
  (`research.md` §2).

**Comprehensive by direct document reading, not by construction**: unlike
`#statement-words` (whose non-exhaustiveness is structural — a frequency threshold cannot
see a rare function), this list was built by reading every function-related chapter in
both vendor doc mirrors (`research.md` §2.1), not by sampling. It is still a manually
curated list, not a provably complete extraction, so a genuine miss remains possible
(`research.md` §5) — a real Cube Voyager built-in function absent from this list is
unaffected structurally; it renders unstyled, exactly as before this feature, until added.

## 2. `#function-calls` grammar pattern (conceptual shape)

| Field | Value |
|---|---|
| Match trigger | One of the 138 names in §1, case-insensitive, immediately followed by `(` with zero intervening whitespace |
| Scope applied | `support.function.drut` (same scope `#statement-words` already uses — see `research.md` §7) |
| Position in `patterns` array | After `#control-words` and `#statement-words`, before `#pair-keywords` — a function-call-shaped word must not be reachable by `#pair-keywords`' `word(?=\s*=)` lookahead (functions are never immediately followed by `=`), so ordering relative to `#pair-keywords`/`#pair-values` does not affect correctness, but placement alongside the other two word-list patterns keeps the grammar's tiering readable |
| Interaction with `#pair-values` | None by construction — `#pair-values` only fires immediately after `=`; `#function-calls` only fires immediately before `(`; a single bareword token cannot satisfy both lookaheads at once, so the two patterns never compete for the same token (FR-002) |
