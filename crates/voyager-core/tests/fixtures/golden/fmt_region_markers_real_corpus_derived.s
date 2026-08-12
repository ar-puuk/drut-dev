; Derived from real WF-TDM-Official-Releases AssignHwy/Distribute shapes,
; with synthetic FMT markers inserted around a hand-aligned block
; (010-fmt-region-markers T011).
RUN PGM=MATRIX
    DistributeMULTISTEP PROCESSID=ClusterNodeID PROCESSNUM=2
; FMT: OFF
      PeriodLp1     = 'AM'
      PeriodHr      = 'am3hr'
      PeriodMetrics = PeriodLp1
; FMT: ON
        RUN PGM=MATRIX   MSG='Calculate Metrics'
            ZONES = 1
        ENDRUN
    EndDistributeMULTISTEP
ENDRUN
