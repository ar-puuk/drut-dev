
;System
    ;In case TP+ crashes during batch, this will halt process & help identify error.
    *(ECHO model crashed > 3_SumToDistricts_GRAVITY.txt)



;get start time
ScriptStartTime = currenttime()




;verify DISTLRG & DISTMED values
RUN PGM=MATRIX  MSG='Distribution 2: District Summary - DISTLRG'
  FILEI ZDATI[1] = '@ParentDir@1_Inputs\1_TAZ\@TAZ_DBF@',
    Z=TAZID

  FILEO RECO[1] = '@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\_lookup_DISTLRG_DISTMED.dbf',
    FORM=10.0, 
    FIELDS=Z, 
           DISTLRG,
           DISTMED
    
    
    
    ;set MATRIX parameters
  ZONES   = @UsedZones@
  ZONEMSG = 50
    
    
    
    ;find max DISTLRG & DISTMED
  if (I=1)
    JLOOP
      max_DISTLRG = MAX(max_DISTLRG, ZI.1.DISTLRG[J])
      max_DISTMED = MAX(max_DISTMED, ZI.1.DISTMED[J])
    ENDJLOOP
  endif
    
    
    
    ;check DISTLRG values
  if (ZI.1.DISTLRG<=0)
    out_DistLrg = INT(max_DISTLRG) + 1
  else
    out_DistLrg = INT(ZI.1.DISTLRG[I])
  endif
    
    
    
    ;check DISTMED values
  if (ZI.1.DISTMED<=0)
    out_DistMed = INT(max_DISTMED) + 1
  else
    out_DistMed = INT(ZI.1.DISTMED[I])
  endif
    
    
    
    ;write output file
  RO.Z       = I
  RO.DISTLRG = out_DistLrg
  RO.DISTMED = out_DistMed
    
  WRITE RECO=1
    
ENDRUN



;summarize to LARGE Districts
RUN PGM=MATRIX  MSG='Distribution 2: District Summary - DISTLRG'
  FILEI ZDATI[1] = '@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\_lookup_DISTLRG_DISTMED.dbf'
  FILEI MATI[1] = '@ParentDir@@ScenarioDir@3_Distribute\PA_AllPurp_Gravity.mtx'

  FILEO MATO[1] = '@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\DISTLRG_PA_Gravity_AllPurp.mtx',
    MO=100-116, 120, 130-131, 140-141,
    name=TOT      ,
         HBW      ,
         HBShp    ,
         HBOth    ,
         HBSch_Pr ,
         HBSch_Sc ,
         HBC      ,
         NHBW     ,
         NHBNW    ,
         IX       ,
         XI       ,
         XX       ,
         SH_LT    ,
         SH_MD    ,
         SH_HV    ,
         Ext_MD   ,
         Ext_HV   ,
         HBSch    ,
         Tot_HBW  ,
         Tel_HBW  ,
         Tot_NHBW ,
         Tel_NHBW 
    
    
    
    ;set MATRIX parameters
  ZONES   = @UsedZones@
  ZONEMSG = 50
    
    
    
    ;assign work matrices
  MW[100] = MI.1.TOT     
  MW[101] = MI.1.HBW     
  MW[102] = MI.1.HBShp   
  MW[103] = MI.1.HBOth   
  MW[104] = MI.1.HBSch_Pr
  MW[105] = MI.1.HBSch_Sc
  MW[106] = MI.1.HBC     
  MW[107] = MI.1.NHBW    
  MW[108] = MI.1.NHBNW   
  MW[109] = MI.1.IX      
  MW[110] = MI.1.XI      
  MW[111] = MI.1.XX      
  MW[112] = MI.1.SH_LT   
  MW[113] = MI.1.SH_MD   
  MW[114] = MI.1.SH_HV   
  MW[115] = MI.1.Ext_MD  
  MW[116] = MI.1.Ext_HV  
    
  MW[120] = MI.1.HBSch   
  MW[130] = MI.1.Tot_HBW 
  MW[131] = MI.1.Tel_HBW 
  MW[140] = MI.1.Tot_NHBW
  MW[141] = MI.1.Tel_NHBW
    
    
    ;summarize to districts
  RENUMBER ZONEO=ZI.1.DISTLRG, missingzi=m, missingzo=w
    
ENDRUN




;summarize to MEDIUM Districts
RUN PGM=MATRIX  MSG='Distribution 2: District Summary - DISTMED'
  FILEI ZDATI[1] = '@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\_lookup_DISTLRG_DISTMED.dbf'
  FILEI MATI[1] = '@ParentDir@@ScenarioDir@3_Distribute\PA_AllPurp_Gravity.mtx'

  FILEO MATO[1] = '@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\DISTMED_PA_Gravity_AllPurp.mtx',
    MO=100-116, 120, 130-131, 140-141,
    name=TOT      ,
         HBW      ,
         HBShp    ,
         HBOth    ,
         HBSch_Pr ,
         HBSch_Sc ,
         HBC      ,
         NHBW     ,
         NHBNW    ,
         IX       ,
         XI       ,
         XX       ,
         SH_LT    ,
         SH_MD    ,
         SH_HV    ,
         Ext_MD   ,
         Ext_HV   ,
         HBSch    ,
         Tot_HBW  ,
         Tel_HBW  ,
         Tot_NHBW ,
         Tel_NHBW 
    
    
    
    ;set MATRIX parameters
  ZONES   = @UsedZones@
  ZONEMSG = 50
    
    
    
    ;assign work matrices
  MW[100] = MI.1.TOT     
  MW[101] = MI.1.HBW     
  MW[102] = MI.1.HBShp   
  MW[103] = MI.1.HBOth   
  MW[104] = MI.1.HBSch_Pr
  MW[105] = MI.1.HBSch_Sc
  MW[106] = MI.1.HBC     
  MW[107] = MI.1.NHBW    
  MW[108] = MI.1.NHBNW   
  MW[109] = MI.1.IX      
  MW[110] = MI.1.XI      
  MW[111] = MI.1.XX      
  MW[112] = MI.1.SH_LT   
  MW[113] = MI.1.SH_MD   
  MW[114] = MI.1.SH_HV   
  MW[115] = MI.1.Ext_MD  
  MW[116] = MI.1.Ext_HV  
    
  MW[120] = MI.1.HBSch   
  MW[130] = MI.1.Tot_HBW 
  MW[131] = MI.1.Tel_HBW 
  MW[140] = MI.1.Tot_NHBW
  MW[141] = MI.1.Tel_NHBW
    
    
    ;summarize to districts
  RENUMBER ZONEO=ZI.1.DISTMED, missingzi=m, missingzo=w
ENDRUN




;Convert to LARGE District matrix to CSV format
RUN PGM=MATRIX  MSG='Distribution 2: District Summary - DISTLRG - CSV'
  FILEI MATI[1] = '@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\DISTLRG_PA_Gravity_AllPurp.mtx'
    
    
    ;set MATRIX parameters
  ZONEMSG = 10
    
    
    ;assign work matrices
  MW[100] = MI.1.TOT     
  MW[101] = MI.1.HBW     
  MW[102] = MI.1.HBShp   
  MW[103] = MI.1.HBOth   
  MW[104] = MI.1.HBSch_Pr
  MW[105] = MI.1.HBSch_Sc
  MW[106] = MI.1.HBC     
  MW[107] = MI.1.NHBW    
  MW[108] = MI.1.NHBNW   
  MW[109] = MI.1.IX      
  MW[110] = MI.1.XI      
  MW[111] = MI.1.XX      
  MW[112] = MI.1.SH_LT   
  MW[113] = MI.1.SH_MD   
  MW[114] = MI.1.SH_HV   
  MW[115] = MI.1.Ext_MD  
  MW[116] = MI.1.Ext_HV  
    
  MW[120] = MI.1.HBSch   
  MW[130] = MI.1.Tot_HBW 
  MW[131] = MI.1.Tel_HBW 
  MW[140] = MI.1.Tot_NHBW
  MW[141] = MI.1.Tel_NHBW
    
    
    ;print header to output file
  if (I=1)
    PRINT FILE='@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\DISTLRG_PA_Gravity_AllPurp.csv',
            CSV=T, 
            FORM=12.2, 
            LIST='I'        ,
                 'J'        ,
                 
                 'TOT'      ,
                 'HBW'      ,
                 'HBShp'    ,
                 'HBOth'    ,
                 'HBSch_Pr' ,
                 'HBSch_Sc' ,
                 'HBC'      ,
                 'NHBW'     ,
                 'NHBNW'    ,
                 'IX'       ,
                 'XI'       ,
                 'XX'       ,
                 'SH_LT'    ,
                 'SH_MD'    ,
                 'SH_HV'    ,
                 'Ext_MD'   ,
                 'Ext_HV'   ,
                 
                 'HBSch'    ,
                 'Tot_HBW'  ,
                 'Tel_HBW'  ,
                 'Tot_NHBW' ,
                 'Tel_NHBW' 
        
  endif  ;i=1
    
    
  JLOOP
      ;print matrix data to a linear csv file
    PRINT FILE='@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\DISTLRG_PA_Gravity_AllPurp.csv',
          CSV=T, 
          FORM=12.2, 
          LIST=I(10.0),
               J(10.0),
               
               MW[100],          ;TOT     
               MW[101],          ;HBW     
               MW[102],          ;HBShp   
               MW[103],          ;HBOth   
               MW[104],          ;HBSch_Pr
               MW[105],          ;HBSch_Sc
               MW[106],          ;HBC     
               MW[107],          ;NHBW    
               MW[108],          ;NHBNW   
               MW[109],          ;IX      
               MW[110],          ;XI      
               MW[111],          ;XX      
               MW[112],          ;SH_LT   
               MW[113],          ;SH_MD   
               MW[114],          ;SH_HV   
               MW[115],          ;Ext_MD  
               MW[116],          ;Ext_HV  
               
               MW[120],          ;HBSch   
               MW[130],          ;Tot_HBW 
               MW[131],          ;Tel_HBW 
               MW[140],          ;Tot_NHBW
               MW[141]           ;Tel_NHBW
        
  ENDJLOOP
    
ENDRUN




;Convert to MEDIUM District matrix to CSV format
RUN PGM=MATRIX  MSG='Distribution 2: District Summary - DISTMED - CSV'
  FILEI MATI[1] = '@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\DISTMED_PA_Gravity_AllPurp.mtx'
    
    
    ;set MATRIX parameters
  ZONEMSG = 10
    
    
    ;assign work matrices
  MW[100] = MI.1.TOT     
  MW[101] = MI.1.HBW     
  MW[102] = MI.1.HBShp   
  MW[103] = MI.1.HBOth   
  MW[104] = MI.1.HBSch_Pr
  MW[105] = MI.1.HBSch_Sc
  MW[106] = MI.1.HBC     
  MW[107] = MI.1.NHBW    
  MW[108] = MI.1.NHBNW   
  MW[109] = MI.1.IX      
  MW[110] = MI.1.XI      
  MW[111] = MI.1.XX      
  MW[112] = MI.1.SH_LT   
  MW[113] = MI.1.SH_MD   
  MW[114] = MI.1.SH_HV   
  MW[115] = MI.1.Ext_MD  
  MW[116] = MI.1.Ext_HV  
    
  MW[120] = MI.1.HBSch   
  MW[130] = MI.1.Tot_HBW 
  MW[131] = MI.1.Tel_HBW 
  MW[140] = MI.1.Tot_NHBW
  MW[141] = MI.1.Tel_NHBW
    
    
    ;print header to output file
  if (I=1)
    PRINT FILE='@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\DISTMED_PA_Gravity_AllPurp.csv',
            CSV=T, 
            FORM=12.2, 
            LIST='I'        ,
                 'J'        ,
                 
                 'TOT'      ,
                 'HBW'      ,
                 'HBShp'    ,
                 'HBOth'    ,
                 'HBSch_Pr' ,
                 'HBSch_Sc' ,
                 'HBC'      ,
                 'NHBW'     ,
                 'NHBNW'    ,
                 'IX'       ,
                 'XI'       ,
                 'XX'       ,
                 'SH_LT'    ,
                 'SH_MD'    ,
                 'SH_HV'    ,
                 'Ext_MD'   ,
                 'Ext_HV'   ,
                 
                 'HBSch'    ,
                 'Tot_HBW'  ,
                 'Tel_HBW'  ,
                 'Tot_NHBW' ,
                 'Tel_NHBW' 
        
  endif  ;i=1
    
    
  JLOOP
        ;print matrix data to a linear csv file
    PRINT FILE='@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\DISTMED_PA_Gravity_AllPurp.csv',
            CSV=T, 
            FORM=12.2, 
          LIST=I(10.0),
               J(10.0),
               
               MW[100],          ;TOT     
               MW[101],          ;HBW     
               MW[102],          ;HBShp   
               MW[103],          ;HBOth   
               MW[104],          ;HBSch_Pr
               MW[105],          ;HBSch_Sc
               MW[106],          ;HBC     
               MW[107],          ;NHBW    
               MW[108],          ;NHBNW   
               MW[109],          ;IX      
               MW[110],          ;XI      
               MW[111],          ;XX      
               MW[112],          ;SH_LT   
               MW[113],          ;SH_MD   
               MW[114],          ;SH_HV   
               MW[115],          ;Ext_MD  
               MW[116],          ;Ext_HV  
               
               MW[120],          ;HBSch   
               MW[130],          ;Tot_HBW 
               MW[131],          ;Tel_HBW 
               MW[140],          ;Tot_NHBW
               MW[141]           ;Tel_NHBW
        
  ENDJLOOP
    
ENDRUN




;print timestamp
RUN PGM=MATRIX
    
  ZONES = 1
    
  ScriptEndTime = currenttime()
  ScriptRunTime = ScriptEndTime - @ScriptStartTime@
    
  PRINT FILE='@ParentDir@@ScenarioDir@_Log\_RunTime - @RID@.txt',
        APPEND=T,
        LIST='\n    Sum Distrib TT to Districts        ', formatdatetime(@ScriptStartTime@, 40, 0, 'yyyy-mm-dd,  hh:nn:ss'), 
                 ',  ', formatdatetime(ScriptRunTime, 40, 0, 'hhh:nn:ss')
    
ENDRUN




;System cleanup
  *(DEL 3_SumToDistricts_GRAVITY.txt)
