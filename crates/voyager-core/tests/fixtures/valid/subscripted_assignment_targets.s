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
