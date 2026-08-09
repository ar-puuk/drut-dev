; JLOOP nested inside IF, LINKLOOP nested inside LOOP, and a sequential
; (non-nested) DistributeMULTISTEP pair.
RUN PGM=HIGHWAY
    IF (I=1)
        JLOOP
            X = MAX(X, ZI.1.DIST[J])
        ENDJLOOP
    ENDIF

    LOOP K=1,10
        LINKLOOP
            Y = LI.CAP
        ENDLINKLOOP
    ENDLOOP

    DistributeMULTISTEP
        Z = 1
    EndDistributeMULTISTEP
ENDRUN
