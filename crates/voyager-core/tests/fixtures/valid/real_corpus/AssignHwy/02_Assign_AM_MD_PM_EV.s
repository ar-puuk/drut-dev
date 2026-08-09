
;System
    ;In case TP+ crashes during batch, this will halt process & help identify error.
    *(ECHO model crashed > 02_Assign_AM_MD_PM_EV.txt)


;get start time
ScriptStartTime = currenttime()



;print timestamp
RUN PGM=MATRIX
    
    ZONES = 1
    
    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_RunTime - @RID@.txt',
        APPEND=T,
        LIST='\n',
             '\n    Begin Period Assignment            ', formatdatetime(@ScriptStartTime@, 40, 0, 'yyyy-mm-dd,  hh:nn:ss')
    
ENDRUN



;set controls for which block file to read
if (Use_SelLinkGrp=1)
    SGRPY = ' '
    SGRPN = ';'
else
    SGRPY = ';'
    SGRPN = ' '
endif


;set controls for assignment starting from scenario network or distribution network
if (FromScenario=1)
    ScenarioY = ' '
    ScenarioN = ';'
    MSGtag    = 'Scenario'
else
    ScenarioY = ';'
    ScenarioN = ' '
    MSGtag    = 'Distribution'
endif


;set controls for running PM 1-hour assignment
PM1hY = ';'
if (RunPM1hr=1)  PM1hY = ' '



RUN PGM=NETWORK   MSG='Final Assign: prepare initial network'
FILEI NETI[1] = '@ParentDir@@ScenarioDir@3_Distribute\Distrib_Network__Summary.net'

FILEO NETO = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\StartNetwork4Assignment_tmp.net'
    
    
    ;parameters
    ZONES = @Usedzones@
    
    
    PHASE = LINKMERGE
        
        ;calculate fields if running from scenario network
        if (@FromScenario@=1)
            
            FF_RAMPPEN = li.1.FF_RampPen
            AM_RAMPPEN = li.1.FF_RampPen
            MD_RAMPPEN = li.1.FF_RampPen
            PM_RAMPPEN = li.1.FF_RampPen
            EV_RAMPPEN = li.1.FF_RampPen
            DY_RAMPPEN = li.1.FF_RampPen
            
            AM_VOL     = 0
            MD_VOL     = 0
            PM_VOL     = 0
            EV_VOL     = 0
            DY_VOL     = 0
            DY_VOL2WY  = 0
            DY_1k      = 0
            DY_1k_2wy  = 0
            
            AM_VC      = 0
            MD_VC      = 0
            PM_VC      = 0
            EV_VC      = 0
            
            FF_SPD     = li.1.FF_SPD
            AM_SPD     = li.1.FF_SPD
            MD_SPD     = li.1.FF_SPD
            PM_SPD     = li.1.FF_SPD
            EV_SPD     = li.1.FF_SPD
            DY_SPD     = li.1.FF_SPD
                
            FF_TkSpd_M = MAX(4, (FF_SPD - 3))
            AM_TkSpd_M = MAX(4, (AM_SPD - 3))
            MD_TkSpd_M = MAX(4, (MD_SPD - 3))
            PM_TkSpd_M = MAX(4, (PM_SPD - 3))
            EV_TkSpd_M = MAX(4, (EV_SPD - 3))
            DY_TkSpd_M = MAX(4, (DY_SPD - 3))
            
            FF_TkSpd_H = MAX(3, (FF_SPD - 5))
            AM_TkSpd_H = MAX(3, (AM_SPD - 5))
            MD_TkSpd_H = MAX(3, (MD_SPD - 5))
            PM_TkSpd_H = MAX(3, (PM_SPD - 5))
            EV_TkSpd_H = MAX(3, (EV_SPD - 5))
            DY_TkSpd_H = MAX(3, (DY_SPD - 5))
            
            FF_TIME    = li.1.FF_TIME
            AM_TIME    = li.1.FF_TIME
            MD_TIME    = li.1.FF_TIME
            PM_TIME    = li.1.FF_TIME
            EV_TIME    = li.1.FF_TIME
            DY_TIME    = li.1.FF_TIME
            
            FF_TkTme_M = li.1.DISTANCE * 60 / FF_TkSpd_M
            AM_TkTme_M = li.1.DISTANCE * 60 / AM_TkSpd_M
            MD_TkTme_M = li.1.DISTANCE * 60 / MD_TkSpd_M
            PM_TkTme_M = li.1.DISTANCE * 60 / PM_TkSpd_M
            EV_TkTme_M = li.1.DISTANCE * 60 / EV_TkSpd_M
            DY_TkTme_M = li.1.DISTANCE * 60 / DY_TkSpd_M
            
            FF_TkTme_H = li.1.DISTANCE * 60 / FF_TkSpd_H
            AM_TkTme_H = li.1.DISTANCE * 60 / AM_TkSpd_H
            MD_TkTme_H = li.1.DISTANCE * 60 / MD_TkSpd_H
            PM_TkTme_H = li.1.DISTANCE * 60 / PM_TkSpd_H
            EV_TkTme_H = li.1.DISTANCE * 60 / EV_TkSpd_H
            DY_TkTme_H = li.1.DISTANCE * 60 / DY_TkSpd_H
            
            AM_VMT     = 0
            MD_VMT     = 0
            PM_VMT     = 0
            EV_VMT     = 0
            DY_VMT     = 0
            
            FF_VHT     = 0
            AM_VHT     = 0
            MD_VHT     = 0
            PM_VHT     = 0
            EV_VHT     = 0
            DY_VHT     = 0
            
            AM_DELAY   = 0
            MD_DELAY   = 0
            PM_DELAY   = 0
            EV_DELAY   = 0
            DY_DELAY   = 0
            
            FF_BTI_TME = 0
            AM_BTI_TME = 0
            MD_BTI_TME = 0
            PM_BTI_TME = 0
            EV_BTI_TME = 0
            DY_BTI_TME = 0
            
        endif  ;(@FromScenario@=1)
        
    ENDPHASE

ENDRUN



;print time stamp
RUN PGM=MATRIX
    
    SubScriptEndTime_IN = currenttime()
    SubScriptRunTime_IN = SubScriptEndTime_IN - @ScriptStartTime@

    ZONES = 1
    
    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_RunTime - @RID@.txt',
        APPEND=T,
        LIST='\n        Initialize highway network  ', formatdatetime(SubScriptRunTime_IN, 40, 0, 'hhh:nn:ss')
    
ENDRUN



;**************************************************************************
;Purpose:    Assign AM trip table to loaded network
;**************************************************************************

;get start time
SubScriptStartTime_AM = currenttime()

;set period tag
PrdTag = 'AM'

;assignment convergence criteria
RelGapCriteria = RGAPCriteria_FinAsgn


RUN PGM=HIGHWAY   MSG='Final Assign: AM period trip assignment'

    FILEI NETI     = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\StartNetwork4Assignment_tmp.net'
          TURNPENI = '@ParentDir@@ScenarioDir@0_InputProcessing\turnpenalties.txt'
          
          MATI[1]  = '@ParentDir@@ScenarioDir@5_AssignHwy\1_ODTables\pa_am3hr_managed.mtx'
          MATI[2]  = '@ParentDir@@ScenarioDir@5_AssignHwy\1_ODTables\ap_am3hr_managed.mtx'
          
          LOOKUPI[1] = '@ParentDir@1_Inputs\0_GlobalData\5_Assignment\MM_Ramp_Penalty_Lookup.csv'
          LOOKUPI[2] = '@ParentDir@@ScenarioDir@0_InputProcessing\RampGP_Connection - @RID@.csv'
          LOOKUPI[3] = '@ParentDir@1_Inputs\0_GlobalData\5_Assignment\MM_Max_Ramp_Penalty.csv'
    
    
    FILEO NETO    = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\@RID@_tmp_@PrdTag@.net',
        INCLUDE=lw.RPen       , 
                lw.TPen       ,
                lw.Spd_Auto   ,
                lw.TrkSpd_MD  ,
                lw.TrkSpd_HV  ,
                lw.Time_Auto  ,
                lw.TrkTime_MD ,
                lw.TrkTime_HV 
    
    
    FILEO MATO[1] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL1_@PrdTag@.mtx',
           mo=31-60,
           name=HBW_DA_NON,
                HBW_SR_NON,
                HBW_SR_HOV,
                HBW_DA_TOL,
                HBW_SR_TOL,
                
                HBO_DA_NON,
                HBO_SR_NON,
                HBO_SR_HOV,
                HBO_DA_TOL,
                HBO_SR_TOL,
                
                NHB_DA_NON,
                NHB_SR_NON,
                NHB_SR_HOV,
                NHB_DA_TOL,
                NHB_SR_TOL,
                
                HBC_DA_NON,
                HBC_SR_NON,
                HBC_SR_HOV,
                HBC_DA_TOL,
                HBC_SR_TOL,
                
                HBSch_Pr  ,
                HBSch_Sc  ,
                
                IX        ,
                XI        ,
                XX        ,
                
                SH_LT     ,
                SH_MD     ,
                SH_HV     ,
                Ext_MD    ,
                Ext_HV    
          
    FILEO MATO[2] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL2_@PrdTag@.mtx',
          mo=61-90,
           name=HBW_DA_NON,
                HBW_SR_NON,
                HBW_SR_HOV,
                HBW_DA_TOL,
                HBW_SR_TOL,
                
                HBO_DA_NON,
                HBO_SR_NON,
                HBO_SR_HOV,
                HBO_DA_TOL,
                HBO_SR_TOL,
                
                NHB_DA_NON,
                NHB_SR_NON,
                NHB_SR_HOV,
                NHB_DA_TOL,
                NHB_SR_TOL,
                
                HBC_DA_NON,
                HBC_SR_NON,
                HBC_SR_HOV,
                HBC_DA_TOL,
                HBC_SR_TOL,
                
                HBSch_Pr  ,
                HBSch_Sc  ,
                
                IX        ,
                XI        ,
                XX        ,
                
                SH_LT     ,
                SH_MD     ,
                SH_HV     ,
                Ext_MD    ,
                Ext_HV    
          
    ;Cluster: distribute intrastep processing
    DistributeINTRASTEP PROCESSID=ClusterNodeID, PROCESSLIST=2-@CoresAvailable@
    
    
    ;parameters
    ZONES   = @UsedZones@
    ZONEMSG = 10
        
        ;period specific variables
        whatperiod = 1
        
        @SGRPN@ READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\4pd_mainbody_managedlanes.block'
        @SGRPY@ READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\4pd_mainbody_managedlanes_selectlink.block'
        
ENDRUN



;print time stamp
RUN PGM=MATRIX

    ZONES = 1
    
    SubScriptEndTime_AM = currenttime()
    SubScriptRunTime_AM = SubScriptEndTime_AM - @SubScriptStartTime_AM@
    
    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_RunTime - @RID@.txt',
        APPEND=T,
        LIST='\n',
             '\n        AM assignment               ', formatdatetime(SubScriptRunTime_AM, 40, 0, 'hhh:nn:ss')
    
ENDRUN



;**************************************************************************
;Purpose:    Assign Mid-day trip table to loaded network
;**************************************************************************

;get start time
SubScriptStartTime_MD = currenttime()

;set period tag
PrdTag = 'MD'

;assignment convergence criteria
RelGapCriteria = RGAPCriteria_FinAsgn


RUN PGM=HIGHWAY   MSG='Final Assign: MD period trip assignment'

    FILEI NETI     = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\StartNetwork4Assignment_tmp.net'     
          TURNPENI = '@ParentDir@@ScenarioDir@0_InputProcessing\turnpenalties.txt'
          
          MATI[1]  = '@ParentDir@@ScenarioDir@5_AssignHwy\1_ODTables\pa_md6hr_managed.mtx'
          MATI[2]  = '@ParentDir@@ScenarioDir@5_AssignHwy\1_ODTables\ap_md6hr_managed.mtx'
          
          LOOKUPI[1] = '@ParentDir@1_Inputs\0_GlobalData\5_Assignment\MM_Ramp_Penalty_Lookup.csv'
          LOOKUPI[2] = '@ParentDir@@ScenarioDir@0_InputProcessing\RampGP_Connection - @RID@.csv'
          LOOKUPI[3] = '@ParentDir@1_Inputs\0_GlobalData\5_Assignment\MM_Max_Ramp_Penalty.csv'
    
    
    FILEO NETO    = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\@RID@_tmp_@PrdTag@.net',
        INCLUDE=lw.RPen       , 
                lw.TPen       ,
                lw.Spd_Auto   ,
                lw.TrkSpd_MD  ,
                lw.TrkSpd_HV  ,
                lw.Time_Auto  ,
                lw.TrkTime_MD ,
                lw.TrkTime_HV 
    
    
    FILEO MATO[1] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL1_@PrdTag@.mtx',
           mo=31-60,
           name=HBW_DA_NON,
                HBW_SR_NON,
                HBW_SR_HOV,
                HBW_DA_TOL,
                HBW_SR_TOL,
                
                HBO_DA_NON,
                HBO_SR_NON,
                HBO_SR_HOV,
                HBO_DA_TOL,
                HBO_SR_TOL,
                
                NHB_DA_NON,
                NHB_SR_NON,
                NHB_SR_HOV,
                NHB_DA_TOL,
                NHB_SR_TOL,
                
                HBC_DA_NON,
                HBC_SR_NON,
                HBC_SR_HOV,
                HBC_DA_TOL,
                HBC_SR_TOL,
                
                HBSch_Pr  ,
                HBSch_Sc  ,
                
                IX        ,
                XI        ,
                XX        ,
                
                SH_LT     ,
                SH_MD     ,
                SH_HV     ,
                Ext_MD    ,
                Ext_HV    
          
    FILEO MATO[2] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL2_@PrdTag@.mtx',
          mo=61-90,
           name=HBW_DA_NON,
                HBW_SR_NON,
                HBW_SR_HOV,
                HBW_DA_TOL,
                HBW_SR_TOL,
                
                HBO_DA_NON,
                HBO_SR_NON,
                HBO_SR_HOV,
                HBO_DA_TOL,
                HBO_SR_TOL,
                
                NHB_DA_NON,
                NHB_SR_NON,
                NHB_SR_HOV,
                NHB_DA_TOL,
                NHB_SR_TOL,
                
                HBC_DA_NON,
                HBC_SR_NON,
                HBC_SR_HOV,
                HBC_DA_TOL,
                HBC_SR_TOL,
                
                HBSch_Pr  ,
                HBSch_Sc  ,
                
                IX        ,
                XI        ,
                XX        ,
                
                SH_LT     ,
                SH_MD     ,
                SH_HV     ,
                Ext_MD    ,
                Ext_HV    

    ;Cluster: distribute intrastep processing
    DistributeINTRASTEP PROCESSID=ClusterNodeID, PROCESSLIST=2-@CoresAvailable@
    
    
    ;parameters
    ZONES   = @UsedZones@
    ZONEMSG = 10
        
        ;period specific variables
        whatperiod = 2
        
        @SGRPN@ READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\4pd_mainbody_managedlanes.block'
        @SGRPY@ READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\4pd_mainbody_managedlanes_selectlink.block'
        
ENDRUN



;print time stamp
RUN PGM=MATRIX

    ZONES = 1
    
    SubScriptEndTime_MD = currenttime()
    SubScriptRunTime_MD = SubScriptEndTime_MD - @SubScriptStartTime_MD@
    
    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_RunTime - @RID@.txt',
        APPEND=T,
        LIST='\n        MD assignment               ', formatdatetime(SubScriptRunTime_MD, 40, 0, 'hhh:nn:ss')
    
ENDRUN



;**************************************************************************
;Purpose:    Assign PM trip table to loaded network
;**************************************************************************

;get start time
SubScriptStartTime_PM = currenttime()

;set period tag
PrdTag = 'PM'

;assignment convergence criteria
RelGapCriteria = RGAPCriteria_FinAsgn


RUN PGM=HIGHWAY   MSG='Final Assign: PM period trip assignment'

    FILEI NETI     = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\StartNetwork4Assignment_tmp.net'     
          TURNPENI = '@ParentDir@@ScenarioDir@0_InputProcessing\turnpenalties.txt'
          
          MATI[1]  = '@ParentDir@@ScenarioDir@5_AssignHwy\1_ODTables\pa_pm3hr_managed.mtx'
          MATI[2]  = '@ParentDir@@ScenarioDir@5_AssignHwy\1_ODTables\ap_pm3hr_managed.mtx'
          
          LOOKUPI[1] = '@ParentDir@1_Inputs\0_GlobalData\5_Assignment\MM_Ramp_Penalty_Lookup.csv'
          LOOKUPI[2] = '@ParentDir@@ScenarioDir@0_InputProcessing\RampGP_Connection - @RID@.csv'
          LOOKUPI[3] = '@ParentDir@1_Inputs\0_GlobalData\5_Assignment\MM_Max_Ramp_Penalty.csv'
    
    
    FILEO NETO    = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\@RID@_tmp_@PrdTag@.net',
        INCLUDE=lw.RPen       , 
                lw.TPen       ,
                lw.Spd_Auto   ,
                lw.TrkSpd_MD  ,
                lw.TrkSpd_HV  ,
                lw.Time_Auto  ,
                lw.TrkTime_MD ,
                lw.TrkTime_HV 
    
    
    FILEO MATO[1] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL1_@PrdTag@.mtx',
           mo=31-60,
           name=HBW_DA_NON,
                HBW_SR_NON,
                HBW_SR_HOV,
                HBW_DA_TOL,
                HBW_SR_TOL,
                
                HBO_DA_NON,
                HBO_SR_NON,
                HBO_SR_HOV,
                HBO_DA_TOL,
                HBO_SR_TOL,
                
                NHB_DA_NON,
                NHB_SR_NON,
                NHB_SR_HOV,
                NHB_DA_TOL,
                NHB_SR_TOL,
                
                HBC_DA_NON,
                HBC_SR_NON,
                HBC_SR_HOV,
                HBC_DA_TOL,
                HBC_SR_TOL,
                
                HBSch_Pr  ,
                HBSch_Sc  ,
                
                IX        ,
                XI        ,
                XX        ,
                
                SH_LT     ,
                SH_MD     ,
                SH_HV     ,
                Ext_MD    ,
                Ext_HV    
          
    FILEO MATO[2] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL2_@PrdTag@.mtx',
          mo=61-90,
           name=HBW_DA_NON,
                HBW_SR_NON,
                HBW_SR_HOV,
                HBW_DA_TOL,
                HBW_SR_TOL,
                
                HBO_DA_NON,
                HBO_SR_NON,
                HBO_SR_HOV,
                HBO_DA_TOL,
                HBO_SR_TOL,
                
                NHB_DA_NON,
                NHB_SR_NON,
                NHB_SR_HOV,
                NHB_DA_TOL,
                NHB_SR_TOL,
                
                HBC_DA_NON,
                HBC_SR_NON,
                HBC_SR_HOV,
                HBC_DA_TOL,
                HBC_SR_TOL,
                
                HBSch_Pr  ,
                HBSch_Sc  ,
                
                IX        ,
                XI        ,
                XX        ,
                
                SH_LT     ,
                SH_MD     ,
                SH_HV     ,
                Ext_MD    ,
                Ext_HV    

    ;Cluster: distribute intrastep processing
    DistributeINTRASTEP PROCESSID=ClusterNodeID, PROCESSLIST=2-@CoresAvailable@
    
    
    ;parameters
    ZONES   = @UsedZones@
    ZONEMSG = 10
        
        ;period specific variables
        whatperiod = 3
        
        @SGRPN@ READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\4pd_mainbody_managedlanes.block'
        @SGRPY@ READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\4pd_mainbody_managedlanes_selectlink.block'
        
ENDRUN



;print time stamp
RUN PGM=MATRIX

    ZONES = 1
    
    SubScriptEndTime_PM = currenttime()
    SubScriptRunTime_PM = SubScriptEndTime_PM - @SubScriptStartTime_PM@
    
    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_RunTime - @RID@.txt',
        APPEND=T,
        LIST='\n        PM assignment               ', formatdatetime(SubScriptRunTime_PM, 40, 0, 'hhh:nn:ss')
    
ENDRUN



;**************************************************************************
;Purpose:    Assign evening trip table loaded network
;**************************************************************************

;get start time
SubScriptStartTime_EV = currenttime()

;set period tag
PrdTag = 'EV'

;assignment convergence criteria
RelGapCriteria = RGAPCriteria_FinAsgn / 10


RUN PGM=HIGHWAY   MSG='Final Assign: EV period trip assignment'

    FILEI NETI     = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\StartNetwork4Assignment_tmp.net'     
          TURNPENI = '@ParentDir@@ScenarioDir@0_InputProcessing\turnpenalties.txt'
          
          MATI[1]  = '@ParentDir@@ScenarioDir@5_AssignHwy\1_ODTables\pa_ev12hr_managed.mtx'
          MATI[2]  = '@ParentDir@@ScenarioDir@5_AssignHwy\1_ODTables\ap_ev12hr_managed.mtx'
          
          LOOKUPI[1] = '@ParentDir@1_Inputs\0_GlobalData\5_Assignment\MM_Ramp_Penalty_Lookup.csv'
          LOOKUPI[2] = '@ParentDir@@ScenarioDir@0_InputProcessing\RampGP_Connection - @RID@.csv'
          LOOKUPI[3] = '@ParentDir@1_Inputs\0_GlobalData\5_Assignment\MM_Max_Ramp_Penalty.csv'
    
    FILEO NETO    = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\@RID@_tmp_@PrdTag@.net',
        INCLUDE=lw.RPen       , 
                lw.TPen       ,
                lw.Spd_Auto   ,
                lw.TrkSpd_MD  ,
                lw.TrkSpd_HV  ,
                lw.Time_Auto  ,
                lw.TrkTime_MD ,
                lw.TrkTime_HV 
    
    
    FILEO MATO[1] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL1_@PrdTag@.mtx',
           mo=31-60,
           name=HBW_DA_NON,
                HBW_SR_NON,
                HBW_SR_HOV,
                HBW_DA_TOL,
                HBW_SR_TOL,
                
                HBO_DA_NON,
                HBO_SR_NON,
                HBO_SR_HOV,
                HBO_DA_TOL,
                HBO_SR_TOL,
                
                NHB_DA_NON,
                NHB_SR_NON,
                NHB_SR_HOV,
                NHB_DA_TOL,
                NHB_SR_TOL,
                
                HBC_DA_NON,
                HBC_SR_NON,
                HBC_SR_HOV,
                HBC_DA_TOL,
                HBC_SR_TOL,
                
                HBSch_Pr  ,
                HBSch_Sc  ,
                
                IX        ,
                XI        ,
                XX        ,
                
                SH_LT     ,
                SH_MD     ,
                SH_HV     ,
                Ext_MD    ,
                Ext_HV    
    
    
    FILEO MATO[2] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL2_@PrdTag@.mtx',
          mo=61-90,
           name=HBW_DA_NON,
                HBW_SR_NON,
                HBW_SR_HOV,
                HBW_DA_TOL,
                HBW_SR_TOL,
                
                HBO_DA_NON,
                HBO_SR_NON,
                HBO_SR_HOV,
                HBO_DA_TOL,
                HBO_SR_TOL,
                
                NHB_DA_NON,
                NHB_SR_NON,
                NHB_SR_HOV,
                NHB_DA_TOL,
                NHB_SR_TOL,
                
                HBC_DA_NON,
                HBC_SR_NON,
                HBC_SR_HOV,
                HBC_DA_TOL,
                HBC_SR_TOL,
                
                HBSch_Pr  ,
                HBSch_Sc  ,
                
                IX        ,
                XI        ,
                XX        ,
                
                SH_LT     ,
                SH_MD     ,
                SH_HV     ,
                Ext_MD    ,
                Ext_HV    
          

    ;Cluster: distribute intrastep processing
    DistributeINTRASTEP PROCESSID=ClusterNodeID, PROCESSLIST=2-@CoresAvailable@
    
    
    ;parameters
    ZONES   = @UsedZones@
    ZONEMSG = 10
        
        ;period specific variables
        whatperiod = 4
        
        @SGRPN@ READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\4pd_mainbody_managedlanes.block'
        @SGRPY@ READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\4pd_mainbody_managedlanes_selectlink.block'
        
ENDRUN



;print time stamp
RUN PGM=MATRIX

    ZONES = 1
    
    SubScriptEndTime_EV = currenttime()
    SubScriptRunTime_EV = SubScriptEndTime_EV - @SubScriptStartTime_EV@
    
    ScriptEndTime = currenttime()
    ScriptRunTime = ScriptEndTime - @ScriptStartTime@
    
    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_RunTime - @RID@.txt',
        APPEND=T,
        LIST='\n        EV assignment               ', formatdatetime(SubScriptRunTime_EV, 40, 0, 'hhh:nn:ss'),
             '\n',
             '\n        Total 4-prd assignment      ', formatdatetime(ScriptRunTime, 40, 0, 'hhh:nn:ss')
    
ENDRUN


    
;**************************************************************************
;Purpose:	Assign PM 1 hour trip table loaded network
;**************************************************************************

;get start time
SubScriptStartTime_PM1hr = currenttime()

;set period tag
PrdTag = 'PM1hr'

;assignment convergence criteria
RelGapCriteria = RGAPCriteria_FinAsgn


if (RunPM1hr=1)
    
    ;assignment convergence criteria
    RelGapCriteria = RGAPCriteria_FinAsgn
    
    
    RUN PGM=HIGHWAY   MSG='Final Assign: PM 1-hour period trip assignment'
    
        FILEI NETI     = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\StartNetwork4Assignment_tmp.net'
              TURNPENI = '@ParentDir@@ScenarioDir@0_InputProcessing\turnpenalties.txt'
              
              MATI[1]  = '@ParentDir@@ScenarioDir@5_AssignHwy\1_ODTables\pa_pm1hr_managed.mtx'
              MATI[2]  = '@ParentDir@@ScenarioDir@5_AssignHwy\1_ODTables\ap_pm1hr_managed.mtx'
              
              LOOKUPI[1] = '@ParentDir@1_Inputs\0_GlobalData\5_Assignment\MM_Ramp_Penalty_Lookup.csv'
              LOOKUPI[2] = '@ParentDir@@ScenarioDir@0_InputProcessing\RampGP_Connection - @RID@.csv'
              LOOKUPI[3] = '@ParentDir@1_Inputs\0_GlobalData\5_Assignment\MM_Max_Ramp_Penalty.csv'
        
        FILEO NETO    = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\@RID@.tmp_@PrdTag@.net', 
            INCLUDE=lw.RPen       , 
                    lw.TPen       ,
                    lw.Spd_Auto   ,
                    lw.TrkSpd_MD  ,
                    lw.TrkSpd_HV  ,
                    lw.Time_Auto  ,
                    lw.TrkTime_MD ,
                    lw.TrkTime_HV 
        
        
        ;Cluster: distribute intrastep processing
        DistributeINTRASTEP PROCESSID=ClusterNodeID, PROCESSLIST=2-@CoresAvailable@
        
        
        ;parameters
        ZONES   = @UsedZones@
        ZONEMSG = 10
            
        ;period specific variables
        whatperiod = 5
        
        READ FILE = '@ParentDir@2_ModelScripts\5_AssignHwy\block\4pd_mainbody_managedlanes.block'
        
    ENDRUN
    
    
    
    ;print time stamp
    RUN PGM=MATRIX
    
        ZONES = 1
        
        SubScriptEndTime_PM1hr = currenttime()
        SubScriptRunTime_PM1hr = SubScriptEndTime_PM1hr - @SubScriptStartTime_PM1hr@
        
        PRINT FILE='@ParentDir@@ScenarioDir@_Log\_RunTime - @RID@.txt',
            APPEND=T,
            LIST='\n',
                 '\n    Assign PM1Hr Trips          ', formatdatetime(@SubScriptRunTime_PM1hr@, 40, 0, 'hh:nn:ss')
        
    ENDRUN
    
endif  ;(RunPM1hr=1)



;**************************************************************************
;compile convergence reports
;**************************************************************************

RUN PGM=MATRIX  MSG='Final Assign: Compiling Assignment Convergence Reports'
    
    FILEI DBI[1] = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\_Convergence - AM.csv',
        DELIMITER =',',
        ITERATION   = #01,
        LAMBDA      =  02,
        BALANCE     =  03,
        RGAP        =  04,
        RGAPCUTOFF  =  05,
        GAP         =  06,
        GAPCUTOFF   =  07,
        RMSE        =  08,
        RMSECUTOFF  =  09,
        AAD         =  10,
        AADCUTOFF   =  11,
        RAAD        =  12,
        RAADCUTOFF  =  13,
        PDIFF       =  14,
        PDIFFCUTOFF =  15,
        AUTOARRAY=ALLFIELDS
    
    FILEI DBI[2] = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\_Convergence - MD.csv',
        DELIMITER =',',
        ITERATION   = #01,
        LAMBDA      =  02,
        BALANCE     =  03,
        RGAP        =  04,
        RGAPCUTOFF  =  05,
        GAP         =  06,
        GAPCUTOFF   =  07,
        RMSE        =  08,
        RMSECUTOFF  =  09,
        AAD         =  10,
        AADCUTOFF   =  11,
        RAAD        =  12,
        RAADCUTOFF  =  13,
        PDIFF       =  14,
        PDIFFCUTOFF =  15,
        AUTOARRAY=ALLFIELDS
        
    FILEI DBI[3] = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\_Convergence - PM.csv',
        DELIMITER =',',
        ITERATION   = #01,
        LAMBDA      =  02,
        BALANCE     =  03,
        RGAP        =  04,
        RGAPCUTOFF  =  05,
        GAP         =  06,
        GAPCUTOFF   =  07,
        RMSE        =  08,
        RMSECUTOFF  =  09,
        AAD         =  10,
        AADCUTOFF   =  11,
        RAAD        =  12,
        RAADCUTOFF  =  13,
        PDIFF       =  14,
        PDIFFCUTOFF =  15,
        AUTOARRAY=ALLFIELDS
        
    FILEI DBI[4] = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\_Convergence - EV.csv',
        DELIMITER =',',
        ITERATION   = #01,
        LAMBDA      =  02,
        BALANCE     =  03,
        RGAP        =  04,
        RGAPCUTOFF  =  05,
        GAP         =  06,
        GAPCUTOFF   =  07,
        RMSE        =  08,
        RMSECUTOFF  =  09,
        AAD         =  10,
        AADCUTOFF   =  11,
        RAAD        =  12,
        RAADCUTOFF  =  13,
        PDIFF       =  14,
        PDIFFCUTOFF =  15,
        AUTOARRAY=ALLFIELDS
        
    @PM1hY@FILEI DBI[5] = '@ParentDir@@ScenarioDir@Temp\5_AssignHwy\_Convergence - PM1hr.csv',
    @PM1hY@    DELIMITER =',',
    @PM1hY@    ITERATION   = #01,
    @PM1hY@    LAMBDA      =  02,
    @PM1hY@    BALANCE     =  03,
    @PM1hY@    RGAP        =  04,
    @PM1hY@    RGAPCUTOFF  =  05,
    @PM1hY@    GAP         =  06,
    @PM1hY@    GAPCUTOFF   =  07,
    @PM1hY@    RMSE        =  08,
    @PM1hY@    RMSECUTOFF  =  09,
    @PM1hY@    AAD         =  10,
    @PM1hY@    AADCUTOFF   =  11,
    @PM1hY@    RAAD        =  12,
    @PM1hY@    RAADCUTOFF  =  13,
    @PM1hY@    PDIFF       =  14,
    @PM1hY@    PDIFFCUTOFF =  15,
    @PM1hY@    AUTOARRAY=ALLFIELDS\
    

    ZONES = 1
    
    
    
    ;print header for assignment convergence report
    PRINT FILE = '@ParentDir@@ScenarioDir@5_AssignHwy\0_ConvergeReports\_Stats - Final Assign - @RID@.csv',
        CSV=T,
        FORM=15.0,
        LIST='PERIOD'     ,
             'ITERATION'  ,
             'LAMBDA'     ,
             'BALANCE'    ,
             'RGAP'       ,
             'RGAPCUTOFF' ,
             'GAP'        ,
             'GAPCUTOFF'  ,
             'RMSE'       ,
             'RMSECUTOFF' ,
             'AAD'        ,
             'AADCUTOFF'  ,
             'RAAD'       ,
             'RAADCUTOFF' ,
             'PDIFF'      ,
             'PDIFFCUTOFF'
    
    
    
    ;get data from temp file convergence reports
    ;AM
    LOOP lp=1, DBI.1.NUMRECORDS
        
        if (dba.1.ITERATION[lp]>0)
            
            ;print data assignment convergence summary to csv file
            PRINT FILE = '@ParentDir@@ScenarioDir@5_AssignHwy\0_ConvergeReports\_Stats - Final Assign - @RID@.csv',
                APPEND=T,
                CSV=T,
                FORM=10.7,
                LIST='AM',
                     dba.1.ITERATION[lp](10.0),
                     dba.1.LAMBDA[lp](10.4)   ,
                     dba.1.BALANCE[lp](10.0)  ,
                     dba.1.RGAP[lp]           ,
                     dba.1.RGAPCUTOFF[lp]     ,
                     dba.1.GAP[lp]            ,
                     dba.1.GAPCUTOFF[lp]      ,
                     dba.1.RMSE[lp](10.2)     ,
                     dba.1.RMSECUTOFF[lp]     ,
                     dba.1.AAD[lp](10.2)      ,
                     dba.1.AADCUTOFF[lp]      ,
                     dba.1.RAAD[lp]           ,
                     dba.1.RAADCUTOFF[lp]     ,
                     dba.1.PDIFF[lp]          ,
                     dba.1.PDIFFCUTOFF[lp]    
            
        endif  ;dba.1.ITERATION[lp]>0
        
    ENDLOOP  ;1, DBI.1.NUMRECORDS
    
    
    ;MD
    LOOP lp=1, DBI.2.NUMRECORDS
        
        if (dba.2.ITERATION[lp]>0)
            
            ;print data assignment convergence summary to csv file
            PRINT FILE = '@ParentDir@@ScenarioDir@5_AssignHwy\0_ConvergeReports\_Stats - Final Assign - @RID@.csv',
                APPEND=T,
                CSV=T,
                FORM=10.7,
                LIST='MD',
                     dba.2.ITERATION[lp](10.0),
                     dba.2.LAMBDA[lp](10.4)   ,
                     dba.2.BALANCE[lp](10.0)  ,
                     dba.2.RGAP[lp]           ,
                     dba.2.RGAPCUTOFF[lp]     ,
                     dba.2.GAP[lp]            ,
                     dba.2.GAPCUTOFF[lp]      ,
                     dba.2.RMSE[lp](10.2)     ,
                     dba.2.RMSECUTOFF[lp]     ,
                     dba.2.AAD[lp](10.2)      ,
                     dba.2.AADCUTOFF[lp]      ,
                     dba.2.RAAD[lp]           ,
                     dba.2.RAADCUTOFF[lp]     ,
                     dba.2.PDIFF[lp]          ,
                     dba.2.PDIFFCUTOFF[lp]    
            
        endif  ;dba.2.ITERATION[lp]>0
        
    ENDLOOP  ;1, DBI.2.NUMRECORDS
    
    
    ;PM
    LOOP lp=1, DBI.3.NUMRECORDS
        
        if (dba.3.ITERATION[lp]>0)
            
            ;print data assignment convergence summary to csv file
            PRINT FILE = '@ParentDir@@ScenarioDir@5_AssignHwy\0_ConvergeReports\_Stats - Final Assign - @RID@.csv',
                APPEND=T,
                CSV=T,
                FORM=10.7,
                LIST='PM',
                     dba.3.ITERATION[lp](10.0),
                     dba.3.LAMBDA[lp](10.4)   ,
                     dba.3.BALANCE[lp](10.0)  ,
                     dba.3.RGAP[lp]           ,
                     dba.3.RGAPCUTOFF[lp]     ,
                     dba.3.GAP[lp]            ,
                     dba.3.GAPCUTOFF[lp]      ,
                     dba.3.RMSE[lp](10.2)     ,
                     dba.3.RMSECUTOFF[lp]     ,
                     dba.3.AAD[lp](10.2)      ,
                     dba.3.AADCUTOFF[lp]      ,
                     dba.3.RAAD[lp]           ,
                     dba.3.RAADCUTOFF[lp]     ,
                     dba.3.PDIFF[lp]          ,
                     dba.3.PDIFFCUTOFF[lp]    
            
        endif  ;dba.3.ITERATION[lp]>0
        
    ENDLOOP  ;1, DBI.3.NUMRECORDS
    
    
    ;EV
    LOOP lp=1, DBI.4.NUMRECORDS
        
        if (dba.4.ITERATION[lp]>0)
            
            ;print data assignment convergence summary to csv file
            PRINT FILE = '@ParentDir@@ScenarioDir@5_AssignHwy\0_ConvergeReports\_Stats - Final Assign - @RID@.csv',
                APPEND=T,
                CSV=T,
                FORM=10.7,
                LIST='EV',
                     dba.4.ITERATION[lp](10.0),
                     dba.4.LAMBDA[lp](10.4)   ,
                     dba.4.BALANCE[lp](10.0)  ,
                     dba.4.RGAP[lp]           ,
                     dba.4.RGAPCUTOFF[lp]     ,
                     dba.4.GAP[lp]            ,
                     dba.4.GAPCUTOFF[lp]      ,
                     dba.4.RMSE[lp](10.2)     ,
                     dba.4.RMSECUTOFF[lp]     ,
                     dba.4.AAD[lp](10.2)      ,
                     dba.4.AADCUTOFF[lp]      ,
                     dba.4.RAAD[lp]           ,
                     dba.4.RAADCUTOFF[lp]     ,
                     dba.4.PDIFF[lp]          ,
                     dba.4.PDIFFCUTOFF[lp]    
            
        endif  ;dba.4.ITERATION[lp]>0
        
    ENDLOOP  ;1, DBI.4.NUMRECORDS
    
    
    ;PM1Hr
    @PM1hY@LOOP lp=1, DBI.5.NUMRECORDS
    @PM1hY@    
    @PM1hY@    if (dba.5.ITERATION[lp]>0)
    @PM1hY@        
    @PM1hY@        ;print data assignment convergence summary to csv file
    @PM1hY@        PRINT FILE = '@ParentDir@@ScenarioDir@5_AssignHwy\0_ConvergeReports\_Stats - Final Assign - @RID@.csv',
    @PM1hY@            APPEND=T,
    @PM1hY@            CSV=T,
    @PM1hY@            FORM=10.7,
    @PM1hY@            LIST='PM1hr',
    @PM1hY@                 dba.5.ITERATION[lp](10.0),
    @PM1hY@                 dba.5.LAMBDA[lp](10.4)   ,
    @PM1hY@                 dba.5.BALANCE[lp](10.0)  ,
    @PM1hY@                 dba.5.RGAP[lp]           ,
    @PM1hY@                 dba.5.RGAPCUTOFF[lp]     ,
    @PM1hY@                 dba.5.GAP[lp]            ,
    @PM1hY@                 dba.5.GAPCUTOFF[lp]      ,
    @PM1hY@                 dba.5.RMSE[lp](10.2)     ,
    @PM1hY@                 dba.5.RMSECUTOFF[lp]     ,
    @PM1hY@                 dba.5.AAD[lp](10.2)      ,
    @PM1hY@                 dba.5.AADCUTOFF[lp]      ,
    @PM1hY@                 dba.5.RAAD[lp]           ,
    @PM1hY@                 dba.5.RAADCUTOFF[lp]     ,
    @PM1hY@                 dba.5.PDIFF[lp]          ,
    @PM1hY@                 dba.5.PDIFFCUTOFF[lp]    
    @PM1hY@        
    @PM1hY@    endif  ;dba.5.ITERATION[lp]>0
    @PM1hY@    
    @PM1hY@ENDLOOP  ;1, DBI.5.NUMRECORDS
    
ENDRUN


;**************************************************************************
;Purpose:    Calculate RowSums & ColSums for Select Link Matrices
;**************************************************************************

if (Use_SelLinkGrp=1)
    
    ;create select link summary csv 
    RUN PGM=MATRIX   MSG='Final Assign: Summarize Select Link Matrices'
        
        FILEI MATI[1] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL1_AM.mtx'
        FILEI MATI[2] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL1_MD.mtx'
        FILEI MATI[3] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL1_PM.mtx'
        FILEI MATI[4] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL1_EV.mtx'
        FILEI MATI[5] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL2_AM.mtx'
        FILEI MATI[6] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL2_MD.mtx'
        FILEI MATI[7] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL2_PM.mtx'
        FILEI MATI[8] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL2_EV.mtx'
        
        FILEO MATO[1] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL1_DY.mtx',
               mo=01-30,
               name=HBW_DA_NON,
                    HBW_SR_NON,
                    HBW_SR_HOV,
                    HBW_DA_TOL,
                    HBW_SR_TOL,
                    
                    HBO_DA_NON,
                    HBO_SR_NON,
                    HBO_SR_HOV,
                    HBO_DA_TOL,
                    HBO_SR_TOL,
                    
                    NHB_DA_NON,
                    NHB_SR_NON,
                    NHB_SR_HOV,
                    NHB_DA_TOL,
                    NHB_SR_TOL,
                    
                    HBC_DA_NON,
                    HBC_SR_NON,
                    HBC_SR_HOV,
                    HBC_DA_TOL,
                    HBC_SR_TOL,
                    
                    HBSch_Pr  ,
                    HBSch_Sc  ,
                    
                    IX        ,
                    XI        ,
                    XX        ,
                    
                    SH_LT     ,
                    SH_MD     ,
                    SH_HV     ,
                    Ext_MD    ,
                    Ext_HV    
        
        FILEO MATO[2] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL2_DY.mtx',
               mo=31-60,
               name=HBW_DA_NON,
                    HBW_SR_NON,
                    HBW_SR_HOV,
                    HBW_DA_TOL,
                    HBW_SR_TOL,
                    
                    HBO_DA_NON,
                    HBO_SR_NON,
                    HBO_SR_HOV,
                    HBO_DA_TOL,
                    HBO_SR_TOL,
                    
                    NHB_DA_NON,
                    NHB_SR_NON,
                    NHB_SR_HOV,
                    NHB_DA_TOL,
                    NHB_SR_TOL,
                    
                    HBC_DA_NON,
                    HBC_SR_NON,
                    HBC_SR_HOV,
                    HBC_DA_TOL,
                    HBC_SR_TOL,
                    
                    HBSch_Pr  ,
                    HBSch_Sc  ,
                    
                    IX        ,
                    XI        ,
                    XX        ,
                    
                    SH_LT     ,
                    SH_MD     ,
                    SH_HV     ,
                    Ext_MD    ,
                    Ext_HV    
        
        
        ZONES = @UsedZones@
        
        
        ;calculate dailys for select link group 1
        mw[01] = mi.1.HBW_DA_NON + mi.2.HBW_DA_NON + mi.3.HBW_DA_NON + mi.4.HBW_DA_NON
        mw[02] = mi.1.HBW_SR_NON + mi.2.HBW_SR_NON + mi.3.HBW_SR_NON + mi.4.HBW_SR_NON
        mw[03] = mi.1.HBW_SR_HOV + mi.2.HBW_SR_HOV + mi.3.HBW_SR_HOV + mi.4.HBW_SR_HOV
        mw[04] = mi.1.HBW_DA_TOL + mi.2.HBW_DA_TOL + mi.3.HBW_DA_TOL + mi.4.HBW_DA_TOL
        mw[05] = mi.1.HBW_SR_TOL + mi.2.HBW_SR_TOL + mi.3.HBW_SR_TOL + mi.4.HBW_SR_TOL
        
        mw[06] = mi.1.HBO_DA_NON + mi.2.HBO_DA_NON + mi.3.HBO_DA_NON + mi.4.HBO_DA_NON
        mw[07] = mi.1.HBO_SR_NON + mi.2.HBO_SR_NON + mi.3.HBO_SR_NON + mi.4.HBO_SR_NON
        mw[08] = mi.1.HBO_SR_HOV + mi.2.HBO_SR_HOV + mi.3.HBO_SR_HOV + mi.4.HBO_SR_HOV
        mw[09] = mi.1.HBO_DA_TOL + mi.2.HBO_DA_TOL + mi.3.HBO_DA_TOL + mi.4.HBO_DA_TOL
        mw[10] = mi.1.HBO_SR_TOL + mi.2.HBO_SR_TOL + mi.3.HBO_SR_TOL + mi.4.HBO_SR_TOL
        
        mw[11] = mi.1.NHB_DA_NON + mi.2.NHB_DA_NON + mi.3.NHB_DA_NON + mi.4.NHB_DA_NON
        mw[12] = mi.1.NHB_SR_NON + mi.2.NHB_SR_NON + mi.3.NHB_SR_NON + mi.4.NHB_SR_NON
        mw[13] = mi.1.NHB_SR_HOV + mi.2.NHB_SR_HOV + mi.3.NHB_SR_HOV + mi.4.NHB_SR_HOV
        mw[14] = mi.1.NHB_DA_TOL + mi.2.NHB_DA_TOL + mi.3.NHB_DA_TOL + mi.4.NHB_DA_TOL
        mw[15] = mi.1.NHB_SR_TOL + mi.2.NHB_SR_TOL + mi.3.NHB_SR_TOL + mi.4.NHB_SR_TOL
        
        mw[16] = mi.1.HBC_DA_NON + mi.2.HBC_DA_NON + mi.3.HBC_DA_NON + mi.4.HBC_DA_NON
        mw[17] = mi.1.HBC_SR_NON + mi.2.HBC_SR_NON + mi.3.HBC_SR_NON + mi.4.HBC_SR_NON
        mw[18] = mi.1.HBC_SR_HOV + mi.2.HBC_SR_HOV + mi.3.HBC_SR_HOV + mi.4.HBC_SR_HOV
        mw[19] = mi.1.HBC_DA_TOL + mi.2.HBC_DA_TOL + mi.3.HBC_DA_TOL + mi.4.HBC_DA_TOL
        mw[20] = mi.1.HBC_SR_TOL + mi.2.HBC_SR_TOL + mi.3.HBC_SR_TOL + mi.4.HBC_SR_TOL
        
        mw[21] = mi.1.HBSch_Pr   + mi.2.HBSch_Pr   + mi.3.HBSch_Pr   + mi.4.HBSch_Pr
        mw[22] = mi.1.HBSch_Sc   + mi.2.HBSch_Sc   + mi.3.HBSch_Sc   + mi.4.HBSch_Sc
        
        mw[23] = mi.1.IX         + mi.2.IX         + mi.3.IX         + mi.4.IX
        mw[24] = mi.1.XI         + mi.2.XI         + mi.3.XI         + mi.4.XI
        mw[25] = mi.1.XX         + mi.2.XX         + mi.3.XX         + mi.4.XX
        
        mw[26] = mi.1.SH_LT      + mi.2.SH_LT      + mi.3.SH_LT      + mi.4.SH_LT
        mw[27] = mi.1.SH_MD      + mi.2.SH_MD      + mi.3.SH_MD      + mi.4.SH_MD
        mw[28] = mi.1.SH_HV      + mi.2.SH_HV      + mi.3.SH_HV      + mi.4.SH_HV
        mw[29] = mi.1.Ext_MD     + mi.2.Ext_MD     + mi.3.Ext_MD     + mi.4.Ext_MD
        mw[30] = mi.1.Ext_HV     + mi.2.Ext_HV     + mi.3.Ext_HV     + mi.4.Ext_HV
        
        ;calculate dailys for select link group 2
        mw[31] = mi.5.HBW_DA_NON + mi.6.HBW_DA_NON + mi.7.HBW_DA_NON + mi.8.HBW_DA_NON
        mw[32] = mi.5.HBW_SR_NON + mi.6.HBW_SR_NON + mi.7.HBW_SR_NON + mi.8.HBW_SR_NON
        mw[33] = mi.5.HBW_SR_HOV + mi.6.HBW_SR_HOV + mi.7.HBW_SR_HOV + mi.8.HBW_SR_HOV
        mw[34] = mi.5.HBW_DA_TOL + mi.6.HBW_DA_TOL + mi.7.HBW_DA_TOL + mi.8.HBW_DA_TOL
        mw[35] = mi.5.HBW_SR_TOL + mi.6.HBW_SR_TOL + mi.7.HBW_SR_TOL + mi.8.HBW_SR_TOL
        
        mw[36] = mi.5.HBO_DA_NON + mi.6.HBO_DA_NON + mi.7.HBO_DA_NON + mi.8.HBO_DA_NON
        mw[37] = mi.5.HBO_SR_NON + mi.6.HBO_SR_NON + mi.7.HBO_SR_NON + mi.8.HBO_SR_NON
        mw[38] = mi.5.HBO_SR_HOV + mi.6.HBO_SR_HOV + mi.7.HBO_SR_HOV + mi.8.HBO_SR_HOV
        mw[39] = mi.5.HBO_DA_TOL + mi.6.HBO_DA_TOL + mi.7.HBO_DA_TOL + mi.8.HBO_DA_TOL
        mw[40] = mi.5.HBO_SR_TOL + mi.6.HBO_SR_TOL + mi.7.HBO_SR_TOL + mi.8.HBO_SR_TOL
        
        mw[41] = mi.5.NHB_DA_NON + mi.6.NHB_DA_NON + mi.7.NHB_DA_NON + mi.8.NHB_DA_NON
        mw[42] = mi.5.NHB_SR_NON + mi.6.NHB_SR_NON + mi.7.NHB_SR_NON + mi.8.NHB_SR_NON
        mw[43] = mi.5.NHB_SR_HOV + mi.6.NHB_SR_HOV + mi.7.NHB_SR_HOV + mi.8.NHB_SR_HOV
        mw[44] = mi.5.NHB_DA_TOL + mi.6.NHB_DA_TOL + mi.7.NHB_DA_TOL + mi.8.NHB_DA_TOL
        mw[45] = mi.5.NHB_SR_TOL + mi.6.NHB_SR_TOL + mi.7.NHB_SR_TOL + mi.8.NHB_SR_TOL
        
        mw[46] = mi.5.HBC_DA_NON + mi.6.HBC_DA_NON + mi.7.HBC_DA_NON + mi.8.HBC_DA_NON
        mw[47] = mi.5.HBC_SR_NON + mi.6.HBC_SR_NON + mi.7.HBC_SR_NON + mi.8.HBC_SR_NON
        mw[48] = mi.5.HBC_SR_HOV + mi.6.HBC_SR_HOV + mi.7.HBC_SR_HOV + mi.8.HBC_SR_HOV
        mw[49] = mi.5.HBC_DA_TOL + mi.6.HBC_DA_TOL + mi.7.HBC_DA_TOL + mi.8.HBC_DA_TOL
        mw[50] = mi.5.HBC_SR_TOL + mi.6.HBC_SR_TOL + mi.7.HBC_SR_TOL + mi.8.HBC_SR_TOL
        
        mw[51] = mi.5.HBSch_Pr   + mi.6.HBSch_Pr   + mi.7.HBSch_Pr   + mi.8.HBSch_Pr
        mw[52] = mi.5.HBSch_Sc   + mi.6.HBSch_Sc   + mi.7.HBSch_Sc   + mi.8.HBSch_Sc
        
        mw[53] = mi.5.IX         + mi.6.IX         + mi.7.IX         + mi.8.IX
        mw[54] = mi.5.XI         + mi.6.XI         + mi.7.XI         + mi.8.XI
        mw[55] = mi.5.XX         + mi.6.XX         + mi.7.XX         + mi.8.XX
        
        mw[56] = mi.5.SH_LT      + mi.6.SH_LT      + mi.7.SH_LT      + mi.8.SH_LT
        mw[57] = mi.5.SH_MD      + mi.6.SH_MD      + mi.7.SH_MD      + mi.8.SH_MD
        mw[58] = mi.5.SH_HV      + mi.6.SH_HV      + mi.7.SH_HV      + mi.8.SH_HV
        mw[59] = mi.5.Ext_MD     + mi.6.Ext_MD     + mi.7.Ext_MD     + mi.8.Ext_MD
        mw[60] = mi.5.Ext_HV     + mi.6.Ext_HV     + mi.7.Ext_HV     + mi.8.Ext_HV
        
        
        ;print out sl summary csv headers
        if (i=1)
            
            PRINT FILE='@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL_Summary.csv',
                    CSV=T,
                    LIST='Tazid'             ,
                         'SelectLinkGroup'   ,
                         'Period'            ,
                         'rowsum_HBW_DA_NON' ,
                         'rowsum_HBW_SR_NON' ,
                         'rowsum_HBW_SR_HOV' ,
                         'rowsum_HBW_DA_TOL' ,
                         'rowsum_HBW_SR_TOL' ,
                         'rowsum_HBO_DA_NON' ,
                         'rowsum_HBO_SR_NON' ,
                         'rowsum_HBO_SR_HOV' ,
                         'rowsum_HBO_DA_TOL' ,
                         'rowsum_HBO_SR_TOL' ,
                         'rowsum_NHB_DA_NON' ,
                         'rowsum_NHB_SR_NON' ,
                         'rowsum_NHB_SR_HOV' ,
                         'rowsum_NHB_DA_TOL' ,
                         'rowsum_NHB_SR_TOL' ,
                         'rowsum_HBC_DA_NON' ,
                         'rowsum_HBC_SR_NON' ,
                         'rowsum_HBC_SR_HOV' ,
                         'rowsum_HBC_DA_TOL' ,
                         'rowsum_HBC_SR_TOL' ,
                         'rowsum_HBSch_Pr'   ,
                         'rowsum_HBSch_Sc'   ,
                         'rowsum_IX'         ,
                         'rowsum_XI'         ,
                         'rowsum_XX'         ,
                         'rowsum_SH_LT'      ,
                         'rowsum_SH_MD'      ,
                         'rowsum_SH_HV'      ,
                         'rowsum_Ext_MD'     ,
                         'rowsum_Ext_HV'     ,
                         'colsum_HBW_DA_NON' ,
                         'colsum_HBW_SR_NON' ,
                         'colsum_HBW_SR_HOV' ,
                         'colsum_HBW_DA_TOL' ,
                         'colsum_HBW_SR_TOL' ,
                         'colsum_HBO_DA_NON' ,
                         'colsum_HBO_SR_NON' ,
                         'colsum_HBO_SR_HOV' ,
                         'colsum_HBO_DA_TOL' ,
                         'colsum_HBO_SR_TOL' ,
                         'colsum_NHB_DA_NON' ,
                         'colsum_NHB_SR_NON' ,
                         'colsum_NHB_SR_HOV' ,
                         'colsum_NHB_DA_TOL' ,
                         'colsum_NHB_SR_TOL' ,
                         'colsum_HBC_DA_NON' ,
                         'colsum_HBC_SR_NON' ,
                         'colsum_HBC_SR_HOV' ,
                         'colsum_HBC_DA_TOL' ,
                         'colsum_HBC_SR_TOL' ,
                         'colsum_HBSch_Pr'   ,
                         'colsum_HBSch_Sc'   ,
                         'colsum_IX'         ,
                         'colsum_XI'         ,
                         'colsum_XX'         ,
                         'colsum_SH_LT'      ,
                         'colsum_SH_MD'      ,
                         'colsum_SH_HV'      ,
                         'colsum_Ext_MD'     ,
                         'colsum_Ext_HV'     
            
        endif  ;i=1
        
    ENDRUN


    ;loop through select link groups
    LOOP lp_SLNUM = 1, 2
        
        if (lp_SLNUM=1) SLNUM = 1
        if (lp_SLNUM=2) SLNUM = 2
        
        ;loop through four periods
        LOOP lp_SLPRD = 1, 5
            
            if (lp_SLPRD=1) SLPRD = 'AM'
            if (lp_SLPRD=2) SLPRD = 'MD'
            if (lp_SLPRD=3) SLPRD = 'PM'
            if (lp_SLPRD=4) SLPRD = 'EV'
            if (lp_SLPRD=5) SLPRD = 'DY'
            
            
            RUN PGM=MATRIX   MSG='Final Assign: Summarize Select Link Matrices'
                
                FILEI MATI[1] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL@SLNUM@_@SLPRD@.mtx'
                
                ZONES = @Usedzones@
                
                
                ;calculate for row sums--------------------------------------------------------------------
                mw[101] = mi.1.HBW_DA_NON
                mw[102] = mi.1.HBW_SR_NON
                mw[103] = mi.1.HBW_SR_HOV
                mw[104] = mi.1.HBW_DA_TOL
                mw[105] = mi.1.HBW_SR_TOL
                
                mw[106] = mi.1.HBO_DA_NON
                mw[107] = mi.1.HBO_SR_NON
                mw[108] = mi.1.HBO_SR_HOV
                mw[109] = mi.1.HBO_DA_TOL
                mw[110] = mi.1.HBO_SR_TOL
                
                mw[111] = mi.1.NHB_DA_NON
                mw[112] = mi.1.NHB_SR_NON
                mw[113] = mi.1.NHB_SR_HOV
                mw[114] = mi.1.NHB_DA_TOL
                mw[115] = mi.1.NHB_SR_TOL
                
                mw[116] = mi.1.HBC_DA_NON
                mw[117] = mi.1.HBC_SR_NON
                mw[118] = mi.1.HBC_SR_HOV
                mw[119] = mi.1.HBC_DA_TOL
                mw[120] = mi.1.HBC_SR_TOL
                
                mw[121] = mi.1.HBSch_Pr
                mw[122] = mi.1.HBSch_Sc
                
                mw[123] = mi.1.IX
                mw[124] = mi.1.XI
                mw[125] = mi.1.XX
                
                mw[126] = mi.1.SH_LT
                mw[127] = mi.1.SH_MD
                mw[128] = mi.1.SH_HV
                mw[129] = mi.1.Ext_MD
                mw[130] = mi.1.Ext_HV
                
                rowsum_HBW_DA_NON = ROWSUM(101)
                rowsum_HBW_SR_NON = ROWSUM(102)
                rowsum_HBW_SR_HOV = ROWSUM(103)
                rowsum_HBW_DA_TOL = ROWSUM(104)
                rowsum_HBW_SR_TOL = ROWSUM(105)
                
                rowsum_HBO_DA_NON = ROWSUM(106)
                rowsum_HBO_SR_NON = ROWSUM(107)
                rowsum_HBO_SR_HOV = ROWSUM(108)
                rowsum_HBO_DA_TOL = ROWSUM(109)
                rowsum_HBO_SR_TOL = ROWSUM(110)
                
                rowsum_NHB_DA_NON = ROWSUM(111)
                rowsum_NHB_SR_NON = ROWSUM(112)
                rowsum_NHB_SR_HOV = ROWSUM(113)
                rowsum_NHB_DA_TOL = ROWSUM(114)
                rowsum_NHB_SR_TOL = ROWSUM(115)
                
                rowsum_HBC_DA_NON = ROWSUM(116)
                rowsum_HBC_SR_NON = ROWSUM(117)
                rowsum_HBC_SR_HOV = ROWSUM(118)
                rowsum_HBC_DA_TOL = ROWSUM(119)
                rowsum_HBC_SR_TOL = ROWSUM(120)
                
                rowsum_HBSch_Pr   = ROWSUM(121)
                rowsum_HBSch_Sc   = ROWSUM(122)
                
                rowsum_IX         = ROWSUM(123)
                rowsum_XI         = ROWSUM(124)
                rowsum_XX         = ROWSUM(125)
                
                rowsum_SH_LT      = ROWSUM(126)
                rowsum_SH_MD      = ROWSUM(127)
                rowsum_SH_HV      = ROWSUM(128)
                rowsum_Ext_MD     = ROWSUM(129)
                rowsum_Ext_HV     = ROWSUM(130)
                
                
                ;calculate for col sums--------------------------------------------------------------------
                mw[201] = mi.1.HBW_DA_NON.T
                mw[202] = mi.1.HBW_SR_NON.T
                mw[203] = mi.1.HBW_SR_HOV.T
                mw[204] = mi.1.HBW_DA_TOL.T
                mw[205] = mi.1.HBW_SR_TOL.T
                
                mw[206] = mi.1.HBO_DA_NON.T
                mw[207] = mi.1.HBO_SR_NON.T
                mw[208] = mi.1.HBO_SR_HOV.T
                mw[209] = mi.1.HBO_DA_TOL.T
                mw[210] = mi.1.HBO_SR_TOL.T
                
                mw[211] = mi.1.NHB_DA_NON.T
                mw[212] = mi.1.NHB_SR_NON.T
                mw[213] = mi.1.NHB_SR_HOV.T
                mw[214] = mi.1.NHB_DA_TOL.T
                mw[215] = mi.1.NHB_SR_TOL.T
                
                mw[216] = mi.1.HBC_DA_NON.T
                mw[217] = mi.1.HBC_SR_NON.T
                mw[218] = mi.1.HBC_SR_HOV.T
                mw[219] = mi.1.HBC_DA_TOL.T
                mw[220] = mi.1.HBC_SR_TOL.T
                
                mw[221] = mi.1.HBSch_Pr.T
                mw[222] = mi.1.HBSch_Sc.T
                
                mw[223] = mi.1.IX.T
                mw[224] = mi.1.XI.T
                mw[225] = mi.1.XX.T
                
                mw[226] = mi.1.SH_LT.T
                mw[227] = mi.1.SH_MD.T
                mw[228] = mi.1.SH_HV.T
                mw[229] = mi.1.Ext_MD.T
                mw[230] = mi.1.Ext_HV.T
                
                colsum_HBW_DA_NON = ROWSUM(201)
                colsum_HBW_SR_NON = ROWSUM(202)
                colsum_HBW_SR_HOV = ROWSUM(203)
                colsum_HBW_DA_TOL = ROWSUM(204)
                colsum_HBW_SR_TOL = ROWSUM(205)
                
                colsum_HBO_DA_NON = ROWSUM(206)
                colsum_HBO_SR_NON = ROWSUM(207)
                colsum_HBO_SR_HOV = ROWSUM(208)
                colsum_HBO_DA_TOL = ROWSUM(209)
                colsum_HBO_SR_TOL = ROWSUM(210)
                
                colsum_NHB_DA_NON = ROWSUM(211)
                colsum_NHB_SR_NON = ROWSUM(212)
                colsum_NHB_SR_HOV = ROWSUM(213)
                colsum_NHB_DA_TOL = ROWSUM(214)
                colsum_NHB_SR_TOL = ROWSUM(215)
                
                colsum_HBC_DA_NON = ROWSUM(216)
                colsum_HBC_SR_NON = ROWSUM(217)
                colsum_HBC_SR_HOV = ROWSUM(218)
                colsum_HBC_DA_TOL = ROWSUM(219)
                colsum_HBC_SR_TOL = ROWSUM(220)
                
                colsum_HBSch_Pr   = ROWSUM(221)
                colsum_HBSch_Sc   = ROWSUM(222)
                
                colsum_IX         = ROWSUM(223)
                colsum_XI         = ROWSUM(224)
                colsum_XX         = ROWSUM(225)
                
                colsum_SH_LT      = ROWSUM(226)
                colsum_SH_MD      = ROWSUM(227)
                colsum_SH_HV      = ROWSUM(228)
                colsum_Ext_MD     = ROWSUM(229)
                colsum_Ext_HV     = ROWSUM(230)
                
                
                ;print results
                PRINT FILE='@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL_Summary.csv',
                        APPEND=T,
                        CSV=T,
                        LIST=i                 ,
                             @SLNUM@           ,
                             '@SLPRD@'         ,
                             rowsum_HBW_DA_NON ,
                             rowsum_HBW_SR_NON ,
                             rowsum_HBW_SR_HOV ,
                             rowsum_HBW_DA_TOL ,
                             rowsum_HBW_SR_TOL ,
                             rowsum_HBO_DA_NON ,
                             rowsum_HBO_SR_NON ,
                             rowsum_HBO_SR_HOV ,
                             rowsum_HBO_DA_TOL ,
                             rowsum_HBO_SR_TOL ,
                             rowsum_NHB_DA_NON ,
                             rowsum_NHB_SR_NON ,
                             rowsum_NHB_SR_HOV ,
                             rowsum_NHB_DA_TOL ,
                             rowsum_NHB_SR_TOL ,
                             rowsum_HBC_DA_NON ,
                             rowsum_HBC_SR_NON ,
                             rowsum_HBC_SR_HOV ,
                             rowsum_HBC_DA_TOL ,
                             rowsum_HBC_SR_TOL ,
                             rowsum_HBSch_Pr   ,
                             rowsum_HBSch_Sc   ,
                             rowsum_IX         ,
                             rowsum_XI         ,
                             rowsum_XX         ,
                             rowsum_SH_LT      ,
                             rowsum_SH_MD      ,
                             rowsum_SH_HV      ,
                             rowsum_Ext_MD     ,
                             rowsum_Ext_HV     ,
                             colsum_HBW_DA_NON ,
                             colsum_HBW_SR_NON ,
                             colsum_HBW_SR_HOV ,
                             colsum_HBW_DA_TOL ,
                             colsum_HBW_SR_TOL ,
                             colsum_HBO_DA_NON ,
                             colsum_HBO_SR_NON ,
                             colsum_HBO_SR_HOV ,
                             colsum_HBO_DA_TOL ,
                             colsum_HBO_SR_TOL ,
                             colsum_NHB_DA_NON ,
                             colsum_NHB_SR_NON ,
                             colsum_NHB_SR_HOV ,
                             colsum_NHB_DA_TOL ,
                             colsum_NHB_SR_TOL ,
                             colsum_HBC_DA_NON ,
                             colsum_HBC_SR_NON ,
                             colsum_HBC_SR_HOV ,
                             colsum_HBC_DA_TOL ,
                             colsum_HBC_SR_TOL ,
                             colsum_HBSch_Pr   ,
                             colsum_HBSch_Sc   ,
                             colsum_IX         ,
                             colsum_XI         ,
                             colsum_XX         ,
                             colsum_SH_LT      ,
                             colsum_SH_MD      ,
                             colsum_SH_HV      ,
                             colsum_Ext_MD     ,
                             colsum_Ext_HV     
                
            ENDRUN
            
        ENDLOOP  ;lp_SLPRD = 1,4
        
    ENDLOOP  ;lp_SLNUM = 1,2
    
endif  ;Use_SelLinkGrp=1



*(del 02_Assign_AM_MD_PM_EV.txt)
