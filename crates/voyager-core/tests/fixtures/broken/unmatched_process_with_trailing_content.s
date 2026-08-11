; EXPECT: UnmatchedProcess
RUN PGM=MATRIX
    ZONES=1454
    PROCESS PHASE=INPUT
        FILEI MATI[1]=base_matrix.mat
        FILEO MATO[1]=output_matrix.mat
    MW[1]=MI.1.1*1.05
    MW[2]=MI.1.2+100
ENDRUN
