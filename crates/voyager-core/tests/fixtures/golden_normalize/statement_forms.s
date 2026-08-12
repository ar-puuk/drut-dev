; Label, shell-escape, and assignment statement forms at top level and
; nested inside a RUN block, including placement around IF/ELSEIF.
ScriptStartTime = currenttime()

:STEP0
*(ECHO starting step 0)
*DEL tempfile.tmp

RUN PGM=MATRIX
    :STEP1
    EndTime_IP = currenttime()
    IF (EndTime_IP>0)
        A = 1
    ELSEIF (EndTime_IP=0)
        :BETWEEN_BRANCHES
        B = 2
    ELSE
        C = 3
    ENDIF
ENDRUN

*(ECHO step complete)
ScriptEndTime = currenttime()
