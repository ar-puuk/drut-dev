; Demonstrates ; FMT: OFF / ; FMT: ON region markers (010-fmt-region-markers).
IF (X=1)
    Y = 1
; FMT: OFF
      Z = 2
; FMT: ON
    W = 3
    IF (A=B)
; FMT: OFF
          C = 4
; FMT: ON
        D = 5
    ENDIF
; FMT: OFF
      IF (P=Q)
; FMT: ON
          R = 6
      ENDIF
ENDIF
