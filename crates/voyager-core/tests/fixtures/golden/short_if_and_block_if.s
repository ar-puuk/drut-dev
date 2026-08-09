; A short-IF (single trailing statement, no ENDIF) alongside an ordinary
; block-form IF in the same file.
IF (X=1) Y = 2

IF (X=2)
    Y = 3
ELSEIF (X=3)
    Y = 4
ELSE
    Y = 5
ENDIF
