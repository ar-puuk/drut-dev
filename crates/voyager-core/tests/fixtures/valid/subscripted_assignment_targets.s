; Assignment statements whose target carries a bracketed subscript (FR-023),
; the real shape found in 08_TripTablesByPeriod.s (single-subscript, 6,000+
; occurrences in that one file alone) and 5_SegmentSummary_Dist.s
; (double-subscript). Structurally representative, not copied verbatim.
RUN PGM=MATRIX
    ZONES = 100

    MW[1] = mi.2.hbw0
    MW[2] = mi.2.hbw1
    MW[3] = mi.2.hbw2

    SUBAREAID[Seg_Idx][idx_SUBAREAID] = SUBAREAID[Seg_Idx][idx_SUBAREAID] + 1
ENDRUN

; A Control statement's own keyword=value pairs can carry the same subscript
; shape (same fix, same underlying assignment_equals_index logic) — the real
; shape found in 4pd_mainbody_distribution.block:780-781.
RUN PGM=HIGHWAY
    PROCESS PHASE=ILOOP
        PATHLOAD PATH=lw.COST_Auto_Wrk, DEC=F2,
            CONSOLIDATE=T,
            EXCLUDEGROUP=1-2,7,
            VOL[01]=mw[01],
            VOL[31]=mw[31]
    ENDPROCESS
ENDRUN
