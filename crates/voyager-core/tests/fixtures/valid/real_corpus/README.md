# Real fixture corpus (T049)

A representative subset of real, production Cube Voyager scripts, sourced
from `WF-TDM-Official-Releases` (not `WF-TDM-Development`, which may contain
in-progress/broken work) — resolving the sourcing/licensing open item noted
in `research.md` §3 for this subset. 9 files, ~5,200 lines total, out of the
161 `.s`/`.block` files in that corpus.

**Redaction**: every file below was checked for absolute filesystem paths,
UNC shares, credentials, and personal/organizational identifiers before being
copied in. None were found — all path-like values use Voyager's own
`@ParentDir@`/`@ScenarioDir@`/`@UserName@`/`@RID@` substitution mechanism
(standard TDM structure, not project-specific secrets), and comments are
ordinary transportation-modeling methodology notes. Nothing was redacted
because nothing sensitive was present; if that changes for a future addition
to this corpus, redact and note it here.

## Coverage

| File | Family | Demonstrates |
|---|---|---|
| `ModelScripts/_TimeStamp_ModelSuccess.block`, `_TimeStamp_ModelCrashed.block` | top-level orchestration | small bare `RUN PGM=MATRIX`/`ENDRUN` blocks, multi-line `PRINT ... LIST=` trailing-comma continuation, `@variable@` refs |
| `InputProcessing/1_InputSetup.s` | InputProcessing | general statement-form diversity, shell-escape crash-trap pattern (`*(ECHO ...)` at top, `*(DEL ...)` at bottom, both outside any `RUN`) |
| `InputProcessing/2_UrbanizationTermTime.s` | InputProcessing | `JLOOP` nested inside an `IF`'s `ELSE` branch, inside `RUN PGM=MATRIX` |
| `Distribute/3_SumToDistricts_GRAVITY.s` | Distribute | `JLOOP` nested inside a bare `IF` — the file named explicitly in the doc-vs-fixture `JLoop` nesting research |
| `Distribute/4pd_mainbody_distribution.block` | Distribute | bare-fragment `.block` shape (no top-level `RUN`), `LINKLOOP`/`ENDLINKLOOP` nested inside `PHASE=ILOOP`, extensive `PHASE=`/`ENDPHASE` pairs |
| `AssignHwy/02_Assign_AM_MD_PM_EV.s` | AssignHwy | a real short-`IF` (`if (RunPM1hr=1)  PM1hY = ' '`, line 50), heavy `RUN`/`PROCESS`/`PHASE` nesting, `IF`/`ELSEIF`/`ELSE` chains |
| `AssignHwy/09_TAZ_Based_Metrics.s` | AssignHwy | multiple sequential `DistributeMULTISTEP`/`EndDistributeMULTISTEP` pairs |
| `ModeChoice/06_HBW_logsums.s` | ModeChoice | `JLOOP` nested inside a `LOOP` (rather than an `IF`) |

## Constructs requested but not present in the real corpus

Two constructs this parser was taught this session have **zero real
instances** anywhere in the full 161-file Official-Releases corpus, not just
this subset — confirmed by scripted search, not sampling:

- **`RUN`/`PROCESS` implicit closing** (by a sibling `RUN`/`!RUN`, a
  shell-escape statement, or a sibling `PROCESS`/`PHASE=`): every `RUN`/
  `ENDRUN` and `PHASE=`/`ENDPHASE` pair in the corpus is explicit. This
  matches spec.md's existing Assumptions note that implicit closing is
  documentation-confirmed only, not fixture-confirmed.
- **Shell-escape statement closing an open `RUN`** specifically (a subset of
  the above): zero instances either.

These remain covered only by the hand-written fixtures already in
`../run_implicit_close.s` and `../process_phase_pairs.s`.
