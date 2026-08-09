; A RUN block with no explicit ENDRUN, closed implicitly by the next RUN
; statement, and a separate RUN closed implicitly by a shell-escape.
RUN PGM=NETWORK
    X = 1
RUN PGM=MATRIX
    Y = 2
*(ECHO done)
