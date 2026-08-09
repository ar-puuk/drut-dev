; Nested IF/LOOP/RUN, mixed-case control words, and both continuation forms.
Run PGM=MATRIX
    ZONES = 100
    if (ZONES=100)
        loop I=1,10
            X = I + 1,
                2 + 3
            IF (X>5)
                Y = 1
            ELSEIF (X>2)
                Y = 2
            ELSE
                Y = 3
            ENDIF
        ENDLOOP
    endif
EndRun

FILEI {
    NETI = mynet.net
    ZDATI = zonal.dat
}
