
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
    if (i=1)
        JLOOP
            max_DISTLRG = MAX(max_DISTLRG, zi.1.DISTLRG[j])
            max_DISTMED = MAX(max_DISTMED, zi.1.DISTMED[j])
        ENDJLOOP
    endif
    
    
    
    ;check DISTLRG values
    if (zi.1.DISTLRG<=0)
        out_DistLrg = INT(max_DISTLRG) + 1
    else
        out_DistLrg = INT(zi.1.DISTLRG[i])
    endif
    
    
    
    ;check DISTMED values
    if (zi.1.DISTMED<=0)
        out_DistMed = INT(max_DISTMED) + 1
    else
        out_DistMed = INT(zi.1.DISTMED[i])
    endif
    
    
    
    ;write output file
    RO.Z       = i
    RO.DISTLRG = out_DistLrg
    RO.DISTMED = out_DistMed
    
    WRITE RECO=1
    
ENDRUN



;summarize to LARGE Districts
RUN PGM=MATRIX  MSG='Distribution 2: District Summary - DISTLRG'
FILEI ZDATI[1] = '@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\_lookup_DISTLRG_DISTMED.dbf'
FILEI MATI[1] = '@ParentDir@@ScenarioDir@3_Distribute\PA_AllPurp_Gravity.mtx'

FILEO MATO[1] = '@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\DISTLRG_PA_Gravity_AllPurp.mtx',
    mo=100-116, 120, 130-131, 140-141,
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
    mw[100] = mi.1.TOT     
    mw[101] = mi.1.HBW     
    mw[102] = mi.1.HBShp   
    mw[103] = mi.1.HBOth   
    mw[104] = mi.1.HBSch_Pr
    mw[105] = mi.1.HBSch_Sc
    mw[106] = mi.1.HBC     
    mw[107] = mi.1.NHBW    
    mw[108] = mi.1.NHBNW   
    mw[109] = mi.1.IX      
    mw[110] = mi.1.XI      
    mw[111] = mi.1.XX      
    mw[112] = mi.1.SH_LT   
    mw[113] = mi.1.SH_MD   
    mw[114] = mi.1.SH_HV   
    mw[115] = mi.1.Ext_MD  
    mw[116] = mi.1.Ext_HV  
    
    mw[120] = mi.1.HBSch   
    mw[130] = mi.1.Tot_HBW 
    mw[131] = mi.1.Tel_HBW 
    mw[140] = mi.1.Tot_NHBW
    mw[141] = mi.1.Tel_NHBW
    
    
    ;summarize to districts
    RENUMBER ZONEO=zi.1.DISTLRG, missingzi=m, missingzo=w
    
ENDRUN




;summarize to MEDIUM Districts
RUN PGM=MATRIX  MSG='Distribution 2: District Summary - DISTMED'
FILEI ZDATI[1] = '@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\_lookup_DISTLRG_DISTMED.dbf'
FILEI MATI[1] = '@ParentDir@@ScenarioDir@3_Distribute\PA_AllPurp_Gravity.mtx'

FILEO MATO[1] = '@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\DISTMED_PA_Gravity_AllPurp.mtx',
    mo=100-116, 120, 130-131, 140-141,
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
    mw[100] = mi.1.TOT     
    mw[101] = mi.1.HBW     
    mw[102] = mi.1.HBShp   
    mw[103] = mi.1.HBOth   
    mw[104] = mi.1.HBSch_Pr
    mw[105] = mi.1.HBSch_Sc
    mw[106] = mi.1.HBC     
    mw[107] = mi.1.NHBW    
    mw[108] = mi.1.NHBNW   
    mw[109] = mi.1.IX      
    mw[110] = mi.1.XI      
    mw[111] = mi.1.XX      
    mw[112] = mi.1.SH_LT   
    mw[113] = mi.1.SH_MD   
    mw[114] = mi.1.SH_HV   
    mw[115] = mi.1.Ext_MD  
    mw[116] = mi.1.Ext_HV  
    
    mw[120] = mi.1.HBSch   
    mw[130] = mi.1.Tot_HBW 
    mw[131] = mi.1.Tel_HBW 
    mw[140] = mi.1.Tot_NHBW
    mw[141] = mi.1.Tel_NHBW
    
    
    ;summarize to districts
    RENUMBER ZONEO=zi.1.DISTMED, missingzi=m, missingzo=w
ENDRUN




;Convert to LARGE District matrix to CSV format
RUN PGM=MATRIX  MSG='Distribution 2: District Summary - DISTLRG - CSV'
FILEI MATI[1] = '@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\DISTLRG_PA_Gravity_AllPurp.mtx'
    
    
    ;set MATRIX parameters
    ZONEMSG = 10
    
    
    ;assign work matrices
    mw[100] = mi.1.TOT     
    mw[101] = mi.1.HBW     
    mw[102] = mi.1.HBShp   
    mw[103] = mi.1.HBOth   
    mw[104] = mi.1.HBSch_Pr
    mw[105] = mi.1.HBSch_Sc
    mw[106] = mi.1.HBC     
    mw[107] = mi.1.NHBW    
    mw[108] = mi.1.NHBNW   
    mw[109] = mi.1.IX      
    mw[110] = mi.1.XI      
    mw[111] = mi.1.XX      
    mw[112] = mi.1.SH_LT   
    mw[113] = mi.1.SH_MD   
    mw[114] = mi.1.SH_HV   
    mw[115] = mi.1.Ext_MD  
    mw[116] = mi.1.Ext_HV  
    
    mw[120] = mi.1.HBSch   
    mw[130] = mi.1.Tot_HBW 
    mw[131] = mi.1.Tel_HBW 
    mw[140] = mi.1.Tot_NHBW
    mw[141] = mi.1.Tel_NHBW
    
    
    ;print header to output file
    if (i=1)
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
          LIST=i(10.0),
               j(10.0),
               
               mw[100],          ;TOT     
               mw[101],          ;HBW     
               mw[102],          ;HBShp   
               mw[103],          ;HBOth   
               mw[104],          ;HBSch_Pr
               mw[105],          ;HBSch_Sc
               mw[106],          ;HBC     
               mw[107],          ;NHBW    
               mw[108],          ;NHBNW   
               mw[109],          ;IX      
               mw[110],          ;XI      
               mw[111],          ;XX      
               mw[112],          ;SH_LT   
               mw[113],          ;SH_MD   
               mw[114],          ;SH_HV   
               mw[115],          ;Ext_MD  
               mw[116],          ;Ext_HV  
               
               mw[120],          ;HBSch   
               mw[130],          ;Tot_HBW 
               mw[131],          ;Tel_HBW 
               mw[140],          ;Tot_NHBW
               mw[141]           ;Tel_NHBW
        
    ENDJLOOP
    
ENDRUN




;Convert to MEDIUM District matrix to CSV format
RUN PGM=MATRIX  MSG='Distribution 2: District Summary - DISTMED - CSV'
FILEI MATI[1] = '@ParentDir@@ScenarioDir@3_Distribute\SumToDistricts\DISTMED_PA_Gravity_AllPurp.mtx'
    
    
    ;set MATRIX parameters
    ZONEMSG = 10
    
    
    ;assign work matrices
    mw[100] = mi.1.TOT     
    mw[101] = mi.1.HBW     
    mw[102] = mi.1.HBShp   
    mw[103] = mi.1.HBOth   
    mw[104] = mi.1.HBSch_Pr
    mw[105] = mi.1.HBSch_Sc
    mw[106] = mi.1.HBC     
    mw[107] = mi.1.NHBW    
    mw[108] = mi.1.NHBNW   
    mw[109] = mi.1.IX      
    mw[110] = mi.1.XI      
    mw[111] = mi.1.XX      
    mw[112] = mi.1.SH_LT   
    mw[113] = mi.1.SH_MD   
    mw[114] = mi.1.SH_HV   
    mw[115] = mi.1.Ext_MD  
    mw[116] = mi.1.Ext_HV  
    
    mw[120] = mi.1.HBSch   
    mw[130] = mi.1.Tot_HBW 
    mw[131] = mi.1.Tel_HBW 
    mw[140] = mi.1.Tot_NHBW
    mw[141] = mi.1.Tel_NHBW
    
    
    ;print header to output file
    if (i=1)
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
          LIST=i(10.0),
               j(10.0),
               
               mw[100],          ;TOT     
               mw[101],          ;HBW     
               mw[102],          ;HBShp   
               mw[103],          ;HBOth   
               mw[104],          ;HBSch_Pr
               mw[105],          ;HBSch_Sc
               mw[106],          ;HBC     
               mw[107],          ;NHBW    
               mw[108],          ;NHBNW   
               mw[109],          ;IX      
               mw[110],          ;XI      
               mw[111],          ;XX      
               mw[112],          ;SH_LT   
               mw[113],          ;SH_MD   
               mw[114],          ;SH_HV   
               mw[115],          ;Ext_MD  
               mw[116],          ;Ext_HV  
               
               mw[120],          ;HBSch   
               mw[130],          ;Tot_HBW 
               mw[131],          ;Tel_HBW 
               mw[140],          ;Tot_NHBW
               mw[141]           ;Tel_NHBW
        
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
