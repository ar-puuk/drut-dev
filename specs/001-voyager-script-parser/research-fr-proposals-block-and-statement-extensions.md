# Proposed FR Additions: Block Types & Statement Shapes Found in Real Fixtures

**Status as of 2026-08-08**: FR-028, FR-029, and FR-030 have been **applied** to
spec.md as proposed. FR-031, FR-032, and the LINKLOOP finding below remain
**deferred** (recorded in spec.md's Assumptions section as explicitly out of scope
for this phase, not silently dropped).

**Superseded 2026-08-08 (same day, after a documentation verification pass)**: the
recommendation below to defer LINKLOOP no longer holds — a vendor-documentation
cross-check found it's general-purpose, cross-program syntax, not the narrow 3-file
finding this doc treats it as, and it's now **FR-033** in spec.md. The FUNCTION `{
... }` finding was also resolved, but differently than proposed here: rather than
becoming its own FR-032, the `{...}` mechanism it uses turned out to be documented
as general-purpose (usable after any control word, not `FUNCTION`-specific) and was
folded into FR-006 instead. FR-031 (the hybrid `WORD=value keyword=value...` shape)
remains deferred as originally recommended. This file is kept as-is below for its
historical evidence trail; spec.md is the current source of truth.

These extend FR-007–FR-009 (block matching) and FR-001–FR-006 (statement parsing).
Each entry follows the same evidence style as the U1–U3 findings already applied.

## Block types (extend FR-007–FR-009)

### FR-028 (proposed) — `PHASE=value ... ENDPHASE`

> The parser MUST recognize and structurally match `PHASE=value ... ENDPHASE` blocks.
> `PHASE=` is itself a `keyword=value` pair optionally followed by further
> space-separated `keyword=value` pairs on the same statement (e.g.
> `PHASE=INPUT, FILEI=li.1`) — the same shape `RUN PGM=...` uses (FR-009). `ENDPHASE`
> takes no arguments.

**Evidence**: Confirmed in `2_ModelScripts/5_AssignHwy/block/
4pd_mainbody_managedlanes.block` (`PHASE=` at lines 66, 306, 925, 1199; `ENDPHASE` at
corresponding closes) and `1_Inputs/3_Highway/_Network Processing Tools/Copy SEGID
field from One Master Network to Another/
CopySEGIDfieldfromOneMasterNetworktoAnother.s:15` (open) / `:131` (close). Distinct
`PHASE=` values observed across the corpus: `ADJUST`, `CONVERGE`, `DATAPREP`, `ILOOP`,
`LINKMERGE`, `LINKREAD`, `MATO`, `NODEMERGE`, `SKIMIJ`, `INPUT`. Census: 35 distinct
files, 138 occurrences each of `PHASE`/`ENDPHASE`. Checked every file containing
either keyword for an open/close count mismatch — **zero mismatches found**, i.e.
every real-world `PHASE`/`ENDPHASE` pair in the corpus is balanced.

### FR-029 (proposed) — `JLOOP ... ENDJLOOP`

> The parser MUST recognize and structurally match `JLOOP ... ENDJLOOP` blocks as a
> loop-block type distinct from `LOOP`/`ENDLOOP` (FR-008), opened by `JLOOP` followed
> by space-separated `keyword=value` pairs.

**Evidence**: `2_ModelScripts/0_InputProcessing/b_SEProcessing/
2_UrbanizationTermTime.s:97`: `JLOOP J=startj, endj, 1 exclude=@dummyzones@,
@externalzones@` ... closed at `:455`: `ENDJLOOP`. Census: 30 distinct files, 88
occurrences each of `JLOOP`/`ENDJLOOP`. Observed nested inside an outer `IF`/`ELSE`
(itself presumably inside an outer per-zone loop) but never inside another `JLOOP` in
the sampled context.

### FR-030 (proposed) — `DistributeMULTISTEP ... EndDistributeMULTISTEP`

> The parser MUST recognize and structurally match `DistributeMULTISTEP
> PROCESSID=... PROCESSNUM=... ... EndDistributeMULTISTEP` blocks — a
> parallel-processing sub-block construct distinct from `RUN`/`ENDRUN`,
> `LOOP`/`ENDLOOP`, and `PHASE`/`ENDPHASE`. Note the literal keyword is exactly
> `DistributeMULTISTEP` / `EndDistributeMULTISTEP` (case-insensitive per FR-011) — not
> a generic `MULTISTEP` suffix pattern.

**Evidence**: `2_ModelScripts/3_Distribute/1_Distribution.s`: three consecutive,
non-nested pairs — `:1714`/`:1795` (`PROCESSNUM=2`), `:1800`/`:1905`
(`PROCESSNUM=3`), `:1910`/`:2022` (`PROCESSNUM=4`), plus a fourth pair later in the
same file at `:6839`/`:7134`. Also confirmed in `4_ModeChoice/
11_MC_HBW_HBO_NHB_HBC.s` (7 consecutive pairs, `PROCESSNUM=2` through `8`),
`5_AssignHwy/09_TAZ_Based_Metrics.s` (11 consecutive pairs, `PROCESSNUM=2` through
`12`), and 6 more files. Census: 8 distinct files. Pairs are always sequential
(`PROCESSNUM` incrementing), **never nested** — one pair fully closes before the next
opens.

## Statement shapes (extend FR-001–FR-006, specifically the FR-003/FR-023 boundary)

### FR-031 (proposed) — hybrid `WORD=value keyword=value...` statement

> The parser MUST recognize a statement shape where the first token takes a value
> directly via `=` (like `Assignment`, FR-023) and is then followed by further
> space-separated `keyword=value` pairs (like a `Control` statement's tail, FR-003) —
> e.g. `COMBINE = EQUI ENHANCE=2, SMOOTH=1, MULTITHREAD=@CoresAvailable@, MEMORY=T`.
> This is neither a pure `Control` statement (no separate leading control word before
> the first `=`) nor a pure `Assignment` (more than one `keyword=value` pair follows).

**Evidence**: Byte-for-byte identical statement found in `2_ModelScripts/
3_Distribute/block/4pd_mainbody_distribution.block:10`, `5_AssignHwy/block/
4pd_mainbody_managedlanes.block:10`, and `5_AssignHwy/block/
4pd_mainbody_managedlanes_SelectLink.block:10` — 3 distinct files, all within a
`RUN PGM=HWYASSIGN` box's `ADJUST` phase. Narrow (single PGM-box context), but
completely unambiguous in shape.

### FR-032 (proposed) — brace-delimited `FUNCTION { ... }` block

> The parser MUST recognize `FUNCTION { ... }` as a block delimited by `{`/`}` rather
> than by a paired control word — structurally unlike every other block type
> (FR-007–FR-009, FR-028–FR-030), all of which use word pairs. Its contents are an
> ordinary nested statement sequence (e.g. `Assignment` statements).

**Evidence**: `4pd_mainbody_distribution.block:842` (open) / `:900` (close, with a
trailing `;end functions` comment); `4pd_mainbody_managedlanes.block:927`/`:999`;
`4pd_mainbody_managedlanes_SelectLink.block:1024`/`:1096`. Same 3 files as FR-031,
always inside the same `PHASE=ADJUST` block. Contents are ordinary (if
domain-specific) `Assignment` statements, e.g. `V = VOL[01] + VOL[02] + ...` and
`COST = TIME`, joined by ordinary FR-006 continuation.

### LINKLOOP investigation (requested follow-up — does NOT meet the FR-028–030 bar)

> Candidate: `LINKLOOP ... ENDLINKLOOP`, a bare block (no arguments on either the
> opener or closer).

**Evidence**: `2_ModelScripts/3_Distribute/block/4pd_mainbody_distribution.block`:
`:416`/`:734` (1 pair). `5_AssignHwy/block/4pd_mainbody_managedlanes.block`:
`:313`/`:320` and `:418`/`:750` (2 pairs). `5_AssignHwy/block/
4pd_mainbody_managedlanes_SelectLink.block`: `:326`/`:333` and `:431`/`:763` (2
pairs). **5 balanced pairs total, but only 3 distinct files — the same 3 files as
FR-031/FR-032**, not new/independent evidence. Every `LINKLOOP`/`ENDLINKLOOP` pair
found is balanced (0 mismatches), same clean-evidence quality as FR-028–030, but the
breadth (3 files, one PGM box) matches the narrow FR-031/FR-032 tier, not the broad
30+/30+/8-file tier FR-028–030 met.

**Recommendation**: Group with FR-031/FR-032 as deferred, not promoted to FR-033 —
it doesn't meet the "broad, multi-file, general-purpose" bar the other three did.
Not applied; noted in spec.md's Assumptions section alongside the other two deferred
findings, in case you want to override this call.

## Summary for scope decision

| Proposed FR | Confidence | Breadth | Extends |
|---|---|---|---|
| FR-028 `PHASE`/`ENDPHASE` | High | 35 files, general-purpose (`NETWORK`, `HWYASSIGN` PGM boxes) | FR-007–FR-009 |
| FR-029 `JLOOP`/`ENDJLOOP` | High | 30 files, general-purpose | FR-008 |
| FR-030 `DistributeMULTISTEP`/`EndDistributeMULTISTEP` | High | 8 files, `DISTRIBUTE`/cluster-processing specific | FR-007–FR-009 |
| FR-031 hybrid `WORD=value keyword=value...` | Moderate | 3 files, single PGM-box (`HWYASSIGN` ADJUST phase) only | FR-003/FR-023 boundary |
| FR-032 `FUNCTION { ... }` brace block | Moderate | 3 files, same narrow context as FR-031 | FR-007–FR-009 (new mechanism) |

FR-028–FR-030 are backed by broad, general-purpose evidence (30+ files each, multiple
unrelated script families) — the same evidentiary bar U1–U3 met. FR-031/FR-032 are
real and unambiguous but narrow (3 files, one PGM box) — worth flagging as lower
priority / plausible Phase-2 candidates if you want to keep Phase 1's grammar surface
smaller.
