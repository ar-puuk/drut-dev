
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
            
      FF_RAMPPEN = LI.1.FF_RampPen
      AM_RAMPPEN = LI.1.FF_RampPen
      MD_RAMPPEN = LI.1.FF_RampPen
      PM_RAMPPEN = LI.1.FF_RampPen
      EV_RAMPPEN = LI.1.FF_RampPen
      DY_RAMPPEN = LI.1.FF_RampPen
            
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
            
      FF_SPD     = LI.1.FF_SPD
      AM_SPD     = LI.1.FF_SPD
      MD_SPD     = LI.1.FF_SPD
      PM_SPD     = LI.1.FF_SPD
      EV_SPD     = LI.1.FF_SPD
      DY_SPD     = LI.1.FF_SPD
                
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
            
      FF_TIME    = LI.1.FF_TIME
      AM_TIME    = LI.1.FF_TIME
      MD_TIME    = LI.1.FF_TIME
      PM_TIME    = LI.1.FF_TIME
      EV_TIME    = LI.1.FF_TIME
      DY_TIME    = LI.1.FF_TIME
            
      FF_TkTme_M = LI.1.DISTANCE * 60 / FF_TkSpd_M
      AM_TkTme_M = LI.1.DISTANCE * 60 / AM_TkSpd_M
      MD_TkTme_M = LI.1.DISTANCE * 60 / MD_TkSpd_M
      PM_TkTme_M = LI.1.DISTANCE * 60 / PM_TkSpd_M
      EV_TkTme_M = LI.1.DISTANCE * 60 / EV_TkSpd_M
      DY_TkTme_M = LI.1.DISTANCE * 60 / DY_TkSpd_M
            
      FF_TkTme_H = LI.1.DISTANCE * 60 / FF_TkSpd_H
      AM_TkTme_H = LI.1.DISTANCE * 60 / AM_TkSpd_H
      MD_TkTme_H = LI.1.DISTANCE * 60 / MD_TkSpd_H
      PM_TkTme_H = LI.1.DISTANCE * 60 / PM_TkSpd_H
      EV_TkTme_H = LI.1.DISTANCE * 60 / EV_TkSpd_H
      DY_TkTme_H = LI.1.DISTANCE * 60 / DY_TkSpd_H
            
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
        INCLUDE=LW.RPen       , 
                LW.TPen       ,
                LW.Spd_Auto   ,
                LW.TrkSpd_MD  ,
                LW.TrkSpd_HV  ,
                LW.Time_Auto  ,
                LW.TrkTime_MD ,
                LW.TrkTime_HV 
    
    
  FILEO MATO[1] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL1_@PrdTag@.mtx',
           MO=31-60,
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
          MO=61-90,
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
        INCLUDE=LW.RPen       , 
                LW.TPen       ,
                LW.Spd_Auto   ,
                LW.TrkSpd_MD  ,
                LW.TrkSpd_HV  ,
                LW.Time_Auto  ,
                LW.TrkTime_MD ,
                LW.TrkTime_HV 
    
    
  FILEO MATO[1] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL1_@PrdTag@.mtx',
           MO=31-60,
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
          MO=61-90,
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
        INCLUDE=LW.RPen       , 
                LW.TPen       ,
                LW.Spd_Auto   ,
                LW.TrkSpd_MD  ,
                LW.TrkSpd_HV  ,
                LW.Time_Auto  ,
                LW.TrkTime_MD ,
                LW.TrkTime_HV 
    
    
  FILEO MATO[1] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL1_@PrdTag@.mtx',
           MO=31-60,
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
          MO=61-90,
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
        INCLUDE=LW.RPen       , 
                LW.TPen       ,
                LW.Spd_Auto   ,
                LW.TrkSpd_MD  ,
                LW.TrkSpd_HV  ,
                LW.Time_Auto  ,
                LW.TrkTime_MD ,
                LW.TrkTime_HV 
    
    
  FILEO MATO[1] = '@ParentDir@@ScenarioDir@\5_AssignHwy\3_SelectLink\@RID@_SL1_@PrdTag@.mtx',
           MO=31-60,
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
          MO=61-90,
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
            INCLUDE=LW.RPen       , 
                    LW.TPen       ,
                    LW.Spd_Auto   ,
                    LW.TrkSpd_MD  ,
                    LW.TrkSpd_HV  ,
                    LW.Time_Auto  ,
                    LW.TrkTime_MD ,
                    LW.TrkTime_HV 
        
        
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
        
    if (DBA.1.ITERATION[lp]>0)
            
            ;print data assignment convergence summary to csv file
      PRINT FILE = '@ParentDir@@ScenarioDir@5_AssignHwy\0_ConvergeReports\_Stats - Final Assign - @RID@.csv',
                APPEND=T,
                CSV=T,
                FORM=10.7,
                LIST='AM',
                     DBA.1.ITERATION[lp](10.0),
                     DBA.1.LAMBDA[lp](10.4)   ,
                     DBA.1.BALANCE[lp](10.0)  ,
                     DBA.1.RGAP[lp]           ,
                     DBA.1.RGAPCUTOFF[lp]     ,
                     DBA.1.GAP[lp]            ,
                     DBA.1.GAPCUTOFF[lp]      ,
                     DBA.1.RMSE[lp](10.2)     ,
                     DBA.1.RMSECUTOFF[lp]     ,
                     DBA.1.AAD[lp](10.2)      ,
                     DBA.1.AADCUTOFF[lp]      ,
                     DBA.1.RAAD[lp]           ,
                     DBA.1.RAADCUTOFF[lp]     ,
                     DBA.1.PDIFF[lp]          ,
                     DBA.1.PDIFFCUTOFF[lp]    
            
    endif  ;dba.1.ITERATION[lp]>0
        
  ENDLOOP  ;1, DBI.1.NUMRECORDS
    
    
    ;MD
  LOOP lp=1, DBI.2.NUMRECORDS
        
    if (DBA.2.ITERATION[lp]>0)
            
            ;print data assignment convergence summary to csv file
      PRINT FILE = '@ParentDir@@ScenarioDir@5_AssignHwy\0_ConvergeReports\_Stats - Final Assign - @RID@.csv',
                APPEND=T,
                CSV=T,
                FORM=10.7,
                LIST='MD',
                     DBA.2.ITERATION[lp](10.0),
                     DBA.2.LAMBDA[lp](10.4)   ,
                     DBA.2.BALANCE[lp](10.0)  ,
                     DBA.2.RGAP[lp]           ,
                     DBA.2.RGAPCUTOFF[lp]     ,
                     DBA.2.GAP[lp]            ,
                     DBA.2.GAPCUTOFF[lp]      ,
                     DBA.2.RMSE[lp](10.2)     ,
                     DBA.2.RMSECUTOFF[lp]     ,
                     DBA.2.AAD[lp](10.2)      ,
                     DBA.2.AADCUTOFF[lp]      ,
                     DBA.2.RAAD[lp]           ,
                     DBA.2.RAADCUTOFF[lp]     ,
                     DBA.2.PDIFF[lp]          ,
                     DBA.2.PDIFFCUTOFF[lp]    
            
    endif  ;dba.2.ITERATION[lp]>0
        
  ENDLOOP  ;1, DBI.2.NUMRECORDS
    
    
    ;PM
  LOOP lp=1, DBI.3.NUMRECORDS
        
    if (DBA.3.ITERATION[lp]>0)
            
            ;print data assignment convergence summary to csv file
      PRINT FILE = '@ParentDir@@ScenarioDir@5_AssignHwy\0_ConvergeReports\_Stats - Final Assign - @RID@.csv',
                APPEND=T,
                CSV=T,
                FORM=10.7,
                LIST='PM',
                     DBA.3.ITERATION[lp](10.0),
                     DBA.3.LAMBDA[lp](10.4)   ,
                     DBA.3.BALANCE[lp](10.0)  ,
                     DBA.3.RGAP[lp]           ,
                     DBA.3.RGAPCUTOFF[lp]     ,
                     DBA.3.GAP[lp]            ,
                     DBA.3.GAPCUTOFF[lp]      ,
                     DBA.3.RMSE[lp](10.2)     ,
                     DBA.3.RMSECUTOFF[lp]     ,
                     DBA.3.AAD[lp](10.2)      ,
                     DBA.3.AADCUTOFF[lp]      ,
                     DBA.3.RAAD[lp]           ,
                     DBA.3.RAADCUTOFF[lp]     ,
                     DBA.3.PDIFF[lp]          ,
                     DBA.3.PDIFFCUTOFF[lp]    
            
    endif  ;dba.3.ITERATION[lp]>0
        
  ENDLOOP  ;1, DBI.3.NUMRECORDS
    
    
    ;EV
  LOOP lp=1, DBI.4.NUMRECORDS
        
    if (DBA.4.ITERATION[lp]>0)
            
            ;print data assignment convergence summary to csv file
      PRINT FILE = '@ParentDir@@ScenarioDir@5_AssignHwy\0_ConvergeReports\_Stats - Final Assign - @RID@.csv',
                APPEND=T,
                CSV=T,
                FORM=10.7,
                LIST='EV',
                     DBA.4.ITERATION[lp](10.0),
                     DBA.4.LAMBDA[lp](10.4)   ,
                     DBA.4.BALANCE[lp](10.0)  ,
                     DBA.4.RGAP[lp]           ,
                     DBA.4.RGAPCUTOFF[lp]     ,
                     DBA.4.GAP[lp]            ,
                     DBA.4.GAPCUTOFF[lp]      ,
                     DBA.4.RMSE[lp](10.2)     ,
                     DBA.4.RMSECUTOFF[lp]     ,
                     DBA.4.AAD[lp](10.2)      ,
                     DBA.4.AADCUTOFF[lp]      ,
                     DBA.4.RAAD[lp]           ,
                     DBA.4.RAADCUTOFF[lp]     ,
                     DBA.4.PDIFF[lp]          ,
                     DBA.4.PDIFFCUTOFF[lp]    
            
    endif  ;dba.4.ITERATION[lp]>0
        
  ENDLOOP  ;1, DBI.4.NUMRECORDS
    
    
    ;PM1Hr
  @PM1hY@LOOP lp=1, DBI.5.NUMRECORDS
  @PM1hY@    
  @PM1hY@    if (DBA.5.ITERATION[lp]>0)
  @PM1hY@        
  @PM1hY@        ;print data assignment convergence summary to csv file
  @PM1hY@        PRINT FILE = '@ParentDir@@ScenarioDir@5_AssignHwy\0_ConvergeReports\_Stats - Final Assign - @RID@.csv',
    @PM1hY@            APPEND=T,
    @PM1hY@            CSV=T,
    @PM1hY@            FORM=10.7,
    @PM1hY@            LIST='PM1hr',
    @PM1hY@                 DBA.5.ITERATION[lp](10.0),
    @PM1hY@                 DBA.5.LAMBDA[lp](10.4)   ,
    @PM1hY@                 DBA.5.BALANCE[lp](10.0)  ,
    @PM1hY@                 DBA.5.RGAP[lp]           ,
    @PM1hY@                 DBA.5.RGAPCUTOFF[lp]     ,
    @PM1hY@                 DBA.5.GAP[lp]            ,
    @PM1hY@                 DBA.5.GAPCUTOFF[lp]      ,
    @PM1hY@                 DBA.5.RMSE[lp](10.2)     ,
    @PM1hY@                 DBA.5.RMSECUTOFF[lp]     ,
    @PM1hY@                 DBA.5.AAD[lp](10.2)      ,
    @PM1hY@                 DBA.5.AADCUTOFF[lp]      ,
    @PM1hY@                 DBA.5.RAAD[lp]           ,
    @PM1hY@                 DBA.5.RAADCUTOFF[lp]     ,
    @PM1hY@                 DBA.5.PDIFF[lp]          ,
    @PM1hY@                 DBA.5.PDIFFCUTOFF[lp]    
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
               MO=01-30,
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
               MO=31-60,
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
    MW[01] = MI.1.HBW_DA_NON + MI.2.HBW_DA_NON + MI.3.HBW_DA_NON + MI.4.HBW_DA_NON
    MW[02] = MI.1.HBW_SR_NON + MI.2.HBW_SR_NON + MI.3.HBW_SR_NON + MI.4.HBW_SR_NON
    MW[03] = MI.1.HBW_SR_HOV + MI.2.HBW_SR_HOV + MI.3.HBW_SR_HOV + MI.4.HBW_SR_HOV
    MW[04] = MI.1.HBW_DA_TOL + MI.2.HBW_DA_TOL + MI.3.HBW_DA_TOL + MI.4.HBW_DA_TOL
    MW[05] = MI.1.HBW_SR_TOL + MI.2.HBW_SR_TOL + MI.3.HBW_SR_TOL + MI.4.HBW_SR_TOL
        
    MW[06] = MI.1.HBO_DA_NON + MI.2.HBO_DA_NON + MI.3.HBO_DA_NON + MI.4.HBO_DA_NON
    MW[07] = MI.1.HBO_SR_NON + MI.2.HBO_SR_NON + MI.3.HBO_SR_NON + MI.4.HBO_SR_NON
    MW[08] = MI.1.HBO_SR_HOV + MI.2.HBO_SR_HOV + MI.3.HBO_SR_HOV + MI.4.HBO_SR_HOV
    MW[09] = MI.1.HBO_DA_TOL + MI.2.HBO_DA_TOL + MI.3.HBO_DA_TOL + MI.4.HBO_DA_TOL
    MW[10] = MI.1.HBO_SR_TOL + MI.2.HBO_SR_TOL + MI.3.HBO_SR_TOL + MI.4.HBO_SR_TOL
        
    MW[11] = MI.1.NHB_DA_NON + MI.2.NHB_DA_NON + MI.3.NHB_DA_NON + MI.4.NHB_DA_NON
    MW[12] = MI.1.NHB_SR_NON + MI.2.NHB_SR_NON + MI.3.NHB_SR_NON + MI.4.NHB_SR_NON
    MW[13] = MI.1.NHB_SR_HOV + MI.2.NHB_SR_HOV + MI.3.NHB_SR_HOV + MI.4.NHB_SR_HOV
    MW[14] = MI.1.NHB_DA_TOL + MI.2.NHB_DA_TOL + MI.3.NHB_DA_TOL + MI.4.NHB_DA_TOL
    MW[15] = MI.1.NHB_SR_TOL + MI.2.NHB_SR_TOL + MI.3.NHB_SR_TOL + MI.4.NHB_SR_TOL
        
    MW[16] = MI.1.HBC_DA_NON + MI.2.HBC_DA_NON + MI.3.HBC_DA_NON + MI.4.HBC_DA_NON
    MW[17] = MI.1.HBC_SR_NON + MI.2.HBC_SR_NON + MI.3.HBC_SR_NON + MI.4.HBC_SR_NON
    MW[18] = MI.1.HBC_SR_HOV + MI.2.HBC_SR_HOV + MI.3.HBC_SR_HOV + MI.4.HBC_SR_HOV
    MW[19] = MI.1.HBC_DA_TOL + MI.2.HBC_DA_TOL + MI.3.HBC_DA_TOL + MI.4.HBC_DA_TOL
    MW[20] = MI.1.HBC_SR_TOL + MI.2.HBC_SR_TOL + MI.3.HBC_SR_TOL + MI.4.HBC_SR_TOL
        
    MW[21] = MI.1.HBSch_Pr   + MI.2.HBSch_Pr   + MI.3.HBSch_Pr   + MI.4.HBSch_Pr
    MW[22] = MI.1.HBSch_Sc   + MI.2.HBSch_Sc   + MI.3.HBSch_Sc   + MI.4.HBSch_Sc
        
    MW[23] = MI.1.IX         + MI.2.IX         + MI.3.IX         + MI.4.IX
    MW[24] = MI.1.XI         + MI.2.XI         + MI.3.XI         + MI.4.XI
    MW[25] = MI.1.XX         + MI.2.XX         + MI.3.XX         + MI.4.XX
        
    MW[26] = MI.1.SH_LT      + MI.2.SH_LT      + MI.3.SH_LT      + MI.4.SH_LT
    MW[27] = MI.1.SH_MD      + MI.2.SH_MD      + MI.3.SH_MD      + MI.4.SH_MD
    MW[28] = MI.1.SH_HV      + MI.2.SH_HV      + MI.3.SH_HV      + MI.4.SH_HV
    MW[29] = MI.1.Ext_MD     + MI.2.Ext_MD     + MI.3.Ext_MD     + MI.4.Ext_MD
    MW[30] = MI.1.Ext_HV     + MI.2.Ext_HV     + MI.3.Ext_HV     + MI.4.Ext_HV
        
        ;calculate dailys for select link group 2
    MW[31] = MI.5.HBW_DA_NON + MI.6.HBW_DA_NON + MI.7.HBW_DA_NON + MI.8.HBW_DA_NON
    MW[32] = MI.5.HBW_SR_NON + MI.6.HBW_SR_NON + MI.7.HBW_SR_NON + MI.8.HBW_SR_NON
    MW[33] = MI.5.HBW_SR_HOV + MI.6.HBW_SR_HOV + MI.7.HBW_SR_HOV + MI.8.HBW_SR_HOV
    MW[34] = MI.5.HBW_DA_TOL + MI.6.HBW_DA_TOL + MI.7.HBW_DA_TOL + MI.8.HBW_DA_TOL
    MW[35] = MI.5.HBW_SR_TOL + MI.6.HBW_SR_TOL + MI.7.HBW_SR_TOL + MI.8.HBW_SR_TOL
        
    MW[36] = MI.5.HBO_DA_NON + MI.6.HBO_DA_NON + MI.7.HBO_DA_NON + MI.8.HBO_DA_NON
    MW[37] = MI.5.HBO_SR_NON + MI.6.HBO_SR_NON + MI.7.HBO_SR_NON + MI.8.HBO_SR_NON
    MW[38] = MI.5.HBO_SR_HOV + MI.6.HBO_SR_HOV + MI.7.HBO_SR_HOV + MI.8.HBO_SR_HOV
    MW[39] = MI.5.HBO_DA_TOL + MI.6.HBO_DA_TOL + MI.7.HBO_DA_TOL + MI.8.HBO_DA_TOL
    MW[40] = MI.5.HBO_SR_TOL + MI.6.HBO_SR_TOL + MI.7.HBO_SR_TOL + MI.8.HBO_SR_TOL
        
    MW[41] = MI.5.NHB_DA_NON + MI.6.NHB_DA_NON + MI.7.NHB_DA_NON + MI.8.NHB_DA_NON
    MW[42] = MI.5.NHB_SR_NON + MI.6.NHB_SR_NON + MI.7.NHB_SR_NON + MI.8.NHB_SR_NON
    MW[43] = MI.5.NHB_SR_HOV + MI.6.NHB_SR_HOV + MI.7.NHB_SR_HOV + MI.8.NHB_SR_HOV
    MW[44] = MI.5.NHB_DA_TOL + MI.6.NHB_DA_TOL + MI.7.NHB_DA_TOL + MI.8.NHB_DA_TOL
    MW[45] = MI.5.NHB_SR_TOL + MI.6.NHB_SR_TOL + MI.7.NHB_SR_TOL + MI.8.NHB_SR_TOL
        
    MW[46] = MI.5.HBC_DA_NON + MI.6.HBC_DA_NON + MI.7.HBC_DA_NON + MI.8.HBC_DA_NON
    MW[47] = MI.5.HBC_SR_NON + MI.6.HBC_SR_NON + MI.7.HBC_SR_NON + MI.8.HBC_SR_NON
    MW[48] = MI.5.HBC_SR_HOV + MI.6.HBC_SR_HOV + MI.7.HBC_SR_HOV + MI.8.HBC_SR_HOV
    MW[49] = MI.5.HBC_DA_TOL + MI.6.HBC_DA_TOL + MI.7.HBC_DA_TOL + MI.8.HBC_DA_TOL
    MW[50] = MI.5.HBC_SR_TOL + MI.6.HBC_SR_TOL + MI.7.HBC_SR_TOL + MI.8.HBC_SR_TOL
        
    MW[51] = MI.5.HBSch_Pr   + MI.6.HBSch_Pr   + MI.7.HBSch_Pr   + MI.8.HBSch_Pr
    MW[52] = MI.5.HBSch_Sc   + MI.6.HBSch_Sc   + MI.7.HBSch_Sc   + MI.8.HBSch_Sc
        
    MW[53] = MI.5.IX         + MI.6.IX         + MI.7.IX         + MI.8.IX
    MW[54] = MI.5.XI         + MI.6.XI         + MI.7.XI         + MI.8.XI
    MW[55] = MI.5.XX         + MI.6.XX         + MI.7.XX         + MI.8.XX
        
    MW[56] = MI.5.SH_LT      + MI.6.SH_LT      + MI.7.SH_LT      + MI.8.SH_LT
    MW[57] = MI.5.SH_MD      + MI.6.SH_MD      + MI.7.SH_MD      + MI.8.SH_MD
    MW[58] = MI.5.SH_HV      + MI.6.SH_HV      + MI.7.SH_HV      + MI.8.SH_HV
    MW[59] = MI.5.Ext_MD     + MI.6.Ext_MD     + MI.7.Ext_MD     + MI.8.Ext_MD
    MW[60] = MI.5.Ext_HV     + MI.6.Ext_HV     + MI.7.Ext_HV     + MI.8.Ext_HV
        
        
        ;print out sl summary csv headers
    if (I=1)
            
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
        MW[101] = MI.1.HBW_DA_NON
        MW[102] = MI.1.HBW_SR_NON
        MW[103] = MI.1.HBW_SR_HOV
        MW[104] = MI.1.HBW_DA_TOL
        MW[105] = MI.1.HBW_SR_TOL
                
        MW[106] = MI.1.HBO_DA_NON
        MW[107] = MI.1.HBO_SR_NON
        MW[108] = MI.1.HBO_SR_HOV
        MW[109] = MI.1.HBO_DA_TOL
        MW[110] = MI.1.HBO_SR_TOL
                
        MW[111] = MI.1.NHB_DA_NON
        MW[112] = MI.1.NHB_SR_NON
        MW[113] = MI.1.NHB_SR_HOV
        MW[114] = MI.1.NHB_DA_TOL
        MW[115] = MI.1.NHB_SR_TOL
                
        MW[116] = MI.1.HBC_DA_NON
        MW[117] = MI.1.HBC_SR_NON
        MW[118] = MI.1.HBC_SR_HOV
        MW[119] = MI.1.HBC_DA_TOL
        MW[120] = MI.1.HBC_SR_TOL
                
        MW[121] = MI.1.HBSch_Pr
        MW[122] = MI.1.HBSch_Sc
                
        MW[123] = MI.1.IX
        MW[124] = MI.1.XI
        MW[125] = MI.1.XX
                
        MW[126] = MI.1.SH_LT
        MW[127] = MI.1.SH_MD
        MW[128] = MI.1.SH_HV
        MW[129] = MI.1.Ext_MD
        MW[130] = MI.1.Ext_HV
                
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
        MW[201] = MI.1.HBW_DA_NON.T
        MW[202] = MI.1.HBW_SR_NON.T
        MW[203] = MI.1.HBW_SR_HOV.T
        MW[204] = MI.1.HBW_DA_TOL.T
        MW[205] = MI.1.HBW_SR_TOL.T
                
        MW[206] = MI.1.HBO_DA_NON.T
        MW[207] = MI.1.HBO_SR_NON.T
        MW[208] = MI.1.HBO_SR_HOV.T
        MW[209] = MI.1.HBO_DA_TOL.T
        MW[210] = MI.1.HBO_SR_TOL.T
                
        MW[211] = MI.1.NHB_DA_NON.T
        MW[212] = MI.1.NHB_SR_NON.T
        MW[213] = MI.1.NHB_SR_HOV.T
        MW[214] = MI.1.NHB_DA_TOL.T
        MW[215] = MI.1.NHB_SR_TOL.T
                
        MW[216] = MI.1.HBC_DA_NON.T
        MW[217] = MI.1.HBC_SR_NON.T
        MW[218] = MI.1.HBC_SR_HOV.T
        MW[219] = MI.1.HBC_DA_TOL.T
        MW[220] = MI.1.HBC_SR_TOL.T
                
        MW[221] = MI.1.HBSch_Pr.T
        MW[222] = MI.1.HBSch_Sc.T
                
        MW[223] = MI.1.IX.T
        MW[224] = MI.1.XI.T
        MW[225] = MI.1.XX.T
                
        MW[226] = MI.1.SH_LT.T
        MW[227] = MI.1.SH_MD.T
        MW[228] = MI.1.SH_HV.T
        MW[229] = MI.1.Ext_MD.T
        MW[230] = MI.1.Ext_HV.T
                
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
                        LIST=I                 ,
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
