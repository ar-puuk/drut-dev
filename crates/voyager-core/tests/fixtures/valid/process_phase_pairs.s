; A PROCESS PHASE=.../ENDPROCESS pair and a bare PHASE=.../ENDPHASE pair,
; plus one PHASE= block closed implicitly by the next PHASE= statement.
RUN PGM=HIGHWAY
    PROCESS PHASE=LINKREAD
        C = LI.CAP
    ENDPROCESS

    PHASE=ILOOP
        PATHLOAD PATH=TIME, VOL[1]=MI.1.1
    ENDPHASE

    PHASE=LINKREAD
        T0 = LI.DISTANCE
    PHASE=ADJUST
        LW.COST = T0
    ENDPROCESS
ENDRUN
