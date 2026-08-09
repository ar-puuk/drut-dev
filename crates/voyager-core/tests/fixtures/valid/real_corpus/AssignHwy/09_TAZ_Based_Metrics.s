;========================================================================================================================
;    create taz based metrics output csv for vizTool
;========================================================================================================================

;System
    ;In case TP+ crashes during batch, this will halt process & help identify error.
    *(ECHO model crashed > 09_TAZ_Based_Metrics.txt)



;create header files ---------------------------------------------------------------------------------------------------
;get start time
ScriptStartTime = currenttime()

;print taz based metrics header
RUN PGM=MATRIX  MSG='TAZ Based Metrics: print header file'
    
    ZONES = 1
    
    PRINT FILE='@ParentDir@@ScenarioDir@\5_AssignHwy\4_Summaries\@RID@ - TAZ-Based Metrics.csv',
        CSV=T,
        LIST='TAZID'             ,
             'Metric'            ,
             'Purpose'           ,
             'Period'            ,
             'PA'                ,
             'Total'             ,
             'DriveAlone'        ,
             'SharedRide'        ,
             'DA_NON'            ,
             'DA_TOL'            ,
             'SR_NON'            ,
             'SR_TOL'            ,
             'SR_HOV'            ,
             'DriveSelf'         ,
             'DropOff'           ,
             'SchoolBus'         ,
             'Truck'             ,
             'External'          
    
ENDRUN


;print runtime header
RUN PGM=MATRIX  MSG='TAZ Based Metrics: print header file'
    ZONES = 1
    PRINT FILE='@ParentDir@@ScenarioDir@\_Log\_RunTime - @RID@ - TAZ-Based Metrics.csv',
        CSV=T,
        LIST='Part'      ,
             'Step'      ,
             'NumMetrics',
             'numprd'    ,
             'StartTime' ,
             'EndTime'   ,
             'RunTime'   
ENDRUN


;print runtime report
printEndTime = currenttime()

RUN PGM=MATRIX
    
    ZONES=1
    
    printRunTime = @printEndTime@ - @ScriptStartTime@
    
    PRINT FILE='@ParentDir@@ScenarioDir@\_Log\_RunTime - @RID@ - TAZ-Based Metrics.csv',
        APPEND=T,
        CSV=T,
        LIST='headers'        ,
             ''               ,
             ''               ,
             ''               ,
             @ScriptStartTime@,
             @printEndTime@   ,
             printRunTime
    
ENDRUN



;create temp matrices --------------------------------------------------------------------------------------------------
;  note: temp matrices use Cluster intrastep processing whereas final matrices do not
;        combine temp matrix loops into 1 group to use Cluster intrastep processing
;        combine final matrix loops into a different group to use Cluster multi-step processing

;loop through metric types
LOOP NumMetrics=1,3
    
    if (NumMetrics=1)  Metric = 'PMT'
    if (NumMetrics=2)  Metric = 'PHT'
    if (NumMetrics=3)  Metric = 'PHT_FF'
    
    if (NumMetrics=1)  SkimMat = 'Dist'
    if (NumMetrics=2)  SkimMat = 'TotTime'
    if (NumMetrics=3)  SkimMat = 'TotTime'
    
    if (NumMetrics=1) SkimMatIdx = 11
    if (NumMetrics=2) SkimMatIdx = 11
    if (NumMetrics=3) SkimMatIdx = 12
    
    
    ;loop through different periods
    LOOP numprd=1,4
        
        if (numprd=1)  PeriodLp1 = 'AM'
        if (numprd=2)  PeriodLp1 = 'MD'
        if (numprd=3)  PeriodLp1 = 'PM'
        if (numprd=4)  PeriodLp1 = 'EV'
        
        if (numprd=1)  PeriodHr  = 'am3hr'
        if (numprd=2)  PeriodHr  = 'md6hr'
        if (numprd=3)  PeriodHr  = 'pm3hr'
        if (numprd=4)  PeriodHr  = 'ev12hr'
        
        tempStartTime = currenttime()
        RUN PGM=MATRIX   MSG='TAZ Based Metrics: Create Temp Matrices - @Metric@ @PeriodLp1@'
            
            READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\TAZ_Based_Metrics - create temp matrices.block'
            
        ENDRUN
        
    ENDLOOP ;numprd=1,4
    
ENDLOOP ;metric=1,3



;calculate TAZ based metrics -------------------------------------------------------------------------------------------
;  note: final metrics reports use print statements that prevent the use of Cluster intrastep processing
;        use Cluster multi-step processing to speed up this part of the code

paStartTime = currenttime()

;PMT -------------------------------------------------------------------------------------
Metric     = 'PMT'
SkimMat    = 'Dist'
SkimMatIdx = 11
NumMetrics = 1
    ;AM --------------------------------------------------------------
    ;assign Cluster multi-step processing group
    DistributeMULTISTEP PROCESSID=ClusterNodeID PROCESSNUM=2
        
        PeriodLp1     = 'AM'
        PeriodHr      = 'am3hr'
        PeriodMetrics = PeriodLp1
        
        RUN PGM=MATRIX   MSG='TAZ Based Metrics: Calculate Metrics - @Metric@ @PeriodLp1@'
            
            READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\TAZ_Based_Metrics - metric calculation.block'
            
        ENDRUN
        
    EndDistributeMULTISTEP
    
    
    ;MD --------------------------------------------------------------
    ;assign Cluster multi-step processing group
    DistributeMULTISTEP PROCESSID=ClusterNodeID PROCESSNUM=3
        
        PeriodLp1     = 'MD'
        PeriodHr      = 'md6hr'
        PeriodMetrics = PeriodLp1
        
        RUN PGM=MATRIX   MSG='TAZ Based Metrics: Calculate Metrics - @Metric@ @PeriodLp1@'
            
            READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\TAZ_Based_Metrics - metric calculation.block'
            
        ENDRUN
        
    EndDistributeMULTISTEP
    
    
    ;PM --------------------------------------------------------------
    ;assign Cluster multi-step processing group
    DistributeMULTISTEP PROCESSID=ClusterNodeID PROCESSNUM=4
        
        PeriodLp1     = 'PM'
        PeriodHr      = 'pm3hr'
        PeriodMetrics = PeriodLp1
        
        RUN PGM=MATRIX   MSG='TAZ Based Metrics: Calculate Metrics - @Metric@ @PeriodLp1@'
            
            READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\TAZ_Based_Metrics - metric calculation.block'
            
        ENDRUN
        
    EndDistributeMULTISTEP
    
    
    ;EV --------------------------------------------------------------
    ;assign Cluster multi-step processing group
    DistributeMULTISTEP PROCESSID=ClusterNodeID PROCESSNUM=5
        
        PeriodLp1     = 'EV'
        PeriodHr      = 'ev12hr'
        PeriodMetrics = PeriodLp1
        
        RUN PGM=MATRIX   MSG='TAZ Based Metrics: Calculate Metrics - @Metric@ @PeriodLp1@'
            
            READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\TAZ_Based_Metrics - metric calculation.block'
            
        ENDRUN
        
    EndDistributeMULTISTEP


;PHT -------------------------------------------------------------------------------------
Metric     = 'PHT'
SkimMat    = 'TotTime'
SkimMatIdx = 11
NumMetrics = 2

    ;AM --------------------------------------------------------------
    ;assign Cluster multi-step processing group
    DistributeMULTISTEP PROCESSID=ClusterNodeID PROCESSNUM=6
        
        PeriodLp1     = 'AM'
        PeriodHr      = 'am3hr'
        PeriodMetrics = PeriodLp1
        
        RUN PGM=MATRIX   MSG='TAZ Based Metrics: Calculate Metrics - @Metric@ @PeriodLp1@'
            
            READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\TAZ_Based_Metrics - metric calculation.block'
            
        ENDRUN
        
    EndDistributeMULTISTEP
    
    
    ;MD --------------------------------------------------------------
    ;assign Cluster multi-step processing group
    DistributeMULTISTEP PROCESSID=ClusterNodeID PROCESSNUM=7
        
        PeriodLp1     = 'MD'
        PeriodHr      = 'md6hr'
        PeriodMetrics = PeriodLp1
        
        RUN PGM=MATRIX   MSG='TAZ Based Metrics: Calculate Metrics - @Metric@ @PeriodLp1@'
            
            READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\TAZ_Based_Metrics - metric calculation.block'
            
        ENDRUN
        
    EndDistributeMULTISTEP
    
    
    ;PM --------------------------------------------------------------
    ;assign Cluster multi-step processing group
    DistributeMULTISTEP PROCESSID=ClusterNodeID PROCESSNUM=8
        
        PeriodLp1     = 'PM'
        PeriodHr      = 'pm3hr'
        PeriodMetrics = PeriodLp1
        
        RUN PGM=MATRIX   MSG='TAZ Based Metrics: Calculate Metrics - @Metric@ @PeriodLp1@'
            
            READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\TAZ_Based_Metrics - metric calculation.block'
            
        ENDRUN
        
    EndDistributeMULTISTEP
    
    
    ;EV --------------------------------------------------------------
    ;assign Cluster multi-step processing group
    DistributeMULTISTEP PROCESSID=ClusterNodeID PROCESSNUM=9
        
        PeriodLp1     = 'EV'
        PeriodHr      = 'ev12hr'
        PeriodMetrics = PeriodLp1
        
        RUN PGM=MATRIX   MSG='TAZ Based Metrics: Calculate Metrics - @Metric@ @PeriodLp1@'
            
            READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\TAZ_Based_Metrics - metric calculation.block'
            
        ENDRUN
        
    EndDistributeMULTISTEP


;PHT_FF ----------------------------------------------------------------------------------
Metric     = 'PHT_FF'
SkimMat    = 'TotTime'
SkimMatIdx = 12
NumMetrics = 3

    ;AM --------------------------------------------------------------
    ;assign Cluster multi-step processing group
    DistributeMULTISTEP PROCESSID=ClusterNodeID PROCESSNUM=10
        
        PeriodLp1     = 'AM'
        PeriodHr      = 'am3hr'
        PeriodMetrics = PeriodLp1
        
        RUN PGM=MATRIX   MSG='TAZ Based Metrics: Calculate Metrics - @Metric@ @PeriodLp1@'
            
            READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\TAZ_Based_Metrics - metric calculation.block'
            
        ENDRUN
        
    EndDistributeMULTISTEP
    
    
    ;MD --------------------------------------------------------------
    ;assign Cluster multi-step processing group
    DistributeMULTISTEP PROCESSID=ClusterNodeID PROCESSNUM=11
        
        PeriodLp1     = 'MD'
        PeriodHr      = 'md6hr'
        PeriodMetrics = PeriodLp1
        
        RUN PGM=MATRIX   MSG='TAZ Based Metrics: Calculate Metrics - @Metric@ @PeriodLp1@'
            
            READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\TAZ_Based_Metrics - metric calculation.block'
            
        ENDRUN
        
    EndDistributeMULTISTEP
    
    
    ;PM --------------------------------------------------------------
    ;assign Cluster multi-step processing group
    DistributeMULTISTEP PROCESSID=ClusterNodeID PROCESSNUM=12
        
        PeriodLp1     = 'PM'
        PeriodHr      = 'pm3hr'
        PeriodMetrics = PeriodLp1
        
        RUN PGM=MATRIX   MSG='TAZ Based Metrics: Calculate Metrics - @Metric@ @PeriodLp1@'
            
            READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\TAZ_Based_Metrics - metric calculation.block'
            
        ENDRUN
        
    EndDistributeMULTISTEP
    
    
    ;EV --------------------------------------------------------------
    ;assign Cluster multi-step processing group - keep on processor 1 (Main)
        
        PeriodLp1     = 'EV'
        PeriodHr      = 'ev12hr'
        PeriodMetrics = PeriodLp1
        
        RUN PGM=MATRIX   MSG='TAZ Based Metrics: Calculate Metrics - @Metric@ @PeriodLp1@'
            
            READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\TAZ_Based_Metrics - metric calculation.block'
            
        ENDRUN
        

;Cluster: wait for all multi-step processing to finish before continuing
WAIT4FILES, 
    FILES="ClusterNodeID2.Script.End" ,
    FILES="ClusterNodeID3.Script.End" ,
    FILES="ClusterNodeID4.Script.End" ,
    FILES="ClusterNodeID5.Script.End" ,
    FILES="ClusterNodeID6.Script.End" ,
    FILES="ClusterNodeID7.Script.End" ,
    FILES="ClusterNodeID8.Script.End" ,
    FILES="ClusterNodeID9.Script.End" ,
    FILES="ClusterNodeID10.Script.End",
    FILES="ClusterNodeID11.Script.End",
    FILES="ClusterNodeID12.Script.End",
    CheckReturnCode=T
 

;create final metric file ----------------------------------------------------------------------------------------------

;loop through metric types
LOOP NumMetrics=1,3
    
    if (NumMetrics=1)  Metric = 'PMT'
    if (NumMetrics=2)  Metric = 'PHT'
    if (NumMetrics=3)  Metric = 'PHT_FF'
    
    
    ;loop through different periods
    LOOP numprd=1,4
        
        if (numprd=1)  PeriodLp1 = 'AM'
        if (numprd=2)  PeriodLp1 = 'MD'
        if (numprd=3)  PeriodLp1 = 'PM'
        if (numprd=4)  PeriodLp1 = 'EV'
        
        RUN PGM=MATRIX  MSG='TAZ Based Metrics: Create Final Metric File - appending @Metric@-@PeriodLp1@'
            
            FILEI DBI[1] = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\TAZ-Based Metrics - @Metric@-@PeriodLp1@.csv',
                DELIMITER =',',
                TAZID       = #01,
                Metric(C)   =  02,
                Purpose(C)  =  03,
                Period(C)   =  04,
                PA(C)       =  05,
                Total       =  06,
                DriveAlone  =  07,
                SharedRide  =  08,
                DA_NON      =  09,
                DA_TOL      =  10,
                SR_NON      =  11,
                SR_TOL      =  12,
                SR_HOV      =  13,
                DriveSelf   =  14,
                DropOff     =  15,
                SchoolBus   =  16,
                Truck       =  17,
                External    =  18,
                AUTOARRAY=ALLFIELDS,
                SORT=Metric
            
            
            ZONES = 1
            
            
            LOOP numrec=1, dbi.1.NUMRECORDS
                
                PRINT FILE='@ParentDir@@ScenarioDir@\5_AssignHwy\4_Summaries\@RID@ - TAZ-Based Metrics.csv',
                    APPEND=T,
                    CSV=T,
                    LIST=dba.1.TAZID[numrec]      ,
                         dba.1.Metric[numrec]     ,
                         dba.1.Purpose[numrec]    ,
                         dba.1.Period[numrec]     ,
                         dba.1.PA[numrec]         ,
                         dba.1.Total[numrec]      ,
                         dba.1.DriveAlone[numrec] ,
                         dba.1.SharedRide[numrec] ,
                         dba.1.DA_NON[numrec]     ,
                         dba.1.DA_TOL[numrec]     ,
                         dba.1.SR_NON[numrec]     ,
                         dba.1.SR_TOL[numrec]     ,
                         dba.1.SR_HOV[numrec]     ,
                         dba.1.DriveSelf[numrec]  ,
                         dba.1.DropOff[numrec]    ,
                         dba.1.SchoolBus[numrec]  ,
                         dba.1.Truck[numrec]      ,
                         dba.1.External[numrec]   
                
            ENDLOOP  ;numrec=1, dbi.1.NUMRECORDS
            
        ENDRUN
        
    ENDLOOP ;numprd=1,4
    
ENDLOOP ;metric=1,3


;calculate run time for metrics calculations
    RUN PGM=MATRIX   MSG='TAZ Based Metrics: Calculate Metrics - Runtime'
        
        ZONES = 1
        
        ;calculate time
        paEndTime = currenttime()
        paRunTime = paEndTime - @paStartTime@
        
        PRINT FILE='@ParentDir@@ScenarioDir@\_Log\_RunTime - @RID@ - TAZ-Based Metrics.csv',
            APPEND=T,
            CSV=T,
            LIST='metrics'        ,
                 'pa'             ,
                 'All'            ,
                 'All'            ,
                 @paStartTime@    ,
                 paEndTime        ,
                 paRunTime        
        
    ENDRUN




if (Run_vizTool=1)

RUN PGM=MATRIX MSG='Run Python Script to Create JSON for vizTool'

    ZONES = 1
    
    ;create control input file for this Python script
    PRINT FILE = '@ParentDir@@ScenarioDir@_Log\py_Variables - vt_JsonVars.txt',
        LIST='#Python input file variables and paths',
             '\njsonId          = "zonemetrics"',
             '\n',
             '\nParentDir       = r"@ParentDir@\"',     ;note: \ added to prevent python from crashing
             '\nScenarioDir     = r"@ScenarioDir@\"',   ;note: \ added to prevent python from crashing
             '\n',
             '\n',
             '\n# input files ------------------------------------------------------------------',
             '\nModelVersion    = "@ModelVersion@"',
             '\nScenarioGroup   = "@ScenarioGroup@"',
             '\nRunYear         =  @RunYear@',
             '\n',
             '\ninputFile       = r"@ParentDir@@ScenarioDir@5_AssignHwy\4_Summaries\@RID@ - TAZ-Based Metrics.csv"',
             '\n',
             '\n'

ENDRUN

;Python script: create json for transit node boardings/alightings
;  note using single asterix minimizes the command window when executed, double asterix executes the command window non-minimized
;  note: the 1>&2 echos the python window output to the one started by Cube
**"@ParentDir@2_ModelScripts\_Python\py-tdm-env\python.exe" "@ParentDir@2_ModelScripts\_Python\py-vizTool\vt_CompileJson.py" 1>&2


;handle python script errors
if (ReturnCode<>0)
    
    PROMPT QUESTION='Python failed to run correctly',
        ANSWER="Please check the py log file in '@ParentDir@@ScenarioDir@_Log' for error messages."
    
    GOTO :ONERROR
    
    ABORT
    
endif  ;ReturnCode<>0


;DOS command to delete '__pycache__' folder
;  note: '/s' removes folder & contents of folder includling any subfolders
;  note: '/q' denotes quite mode, meaning doesn't ask for confirmation to delete
*(rmdir /s /q "_Log\__pycache__")
*(rmdir /s /q "@ParentDir@2_ModelScripts\_Python\py-vizTool\__pycache__")


endif  ;Run_vizTool=1


;print timestamp
RUN PGM=MATRIX
    
    ZONES = 1
    
    ScriptEndTime = currenttime()
    ScriptRunTime = ScriptEndTime - @ScriptStartTime@
    
    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_RunTime - @RID@.txt',
        APPEND=T,
        LIST='\n    TAZ-Based Metrics                  ', formatdatetime(@ScriptStartTime@, 40, 0, 'yyyy-mm-dd,  hh:nn:ss'), 
                 ',  ', formatdatetime(ScriptRunTime, 40, 0, 'hhh:nn:ss')
    
ENDRUN




*(DEL 09_TAZ_Based_Metrics.txt)