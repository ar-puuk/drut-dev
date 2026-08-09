; BREAK nested only inside a bare IF (no enclosing LOOP/RUN/PROCESS) and
; BREAK nested inside a PROCESS/PHASE stack — both are structurally accepted
; (FR-026's narrowed condition only fires with no enclosing block at all).
IF (X=1)
    BREAK
ENDIF

RUN PGM=HIGHWAY
    PROCESS PHASE=ADJUST
        IF (Y=1)
            BREAK
        ENDIF
    ENDPROCESS
ENDRUN
