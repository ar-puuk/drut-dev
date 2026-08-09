
;System
    ;file to halt the model run if model crashes
    *(ECHO 'model crashed' > 06_HBW_logsums.txt)



;get start time
ScriptStartTime = currenttime()




    purp = 'HBW' ;HBW only
    prd  = 'Pk'  ;peak only
    n   = 1
    n_1 = 0
    
    purpose = 1
    period = 1 
   
  RUN PGM=MATRIX   MSG='Mode Choice 6: compute logsums & shares - @purp@ - @prd@'
    zones=@Usedzones@
    maxmw=500	;resets the maximimum number of working matrices from 200 to 500
    FILEI ZDATI[1] = '@ParentDir@@ScenarioDir@0_InputProcessing\Urbanization.dbf'  ; square miles, parking, CBD, XY

    ;read in percent of trips by "detailed" market segment (size/income/vehicles/workers)
    FILEI ZDATI[3] = '@ParentDir@@ScenarioDir@Temp\4_ModeChoice\HH1_PercTrips_segment_@purp@.dbf'
    FILEI ZDATI[4] = '@ParentDir@@ScenarioDir@Temp\4_ModeChoice\HH2_PercTrips_segment_@purp@.dbf'
    FILEI ZDATI[5] = '@ParentDir@@ScenarioDir@Temp\4_ModeChoice\HH3_PercTrips_segment_@purp@.dbf'
    FILEI ZDATI[6] = '@ParentDir@@ScenarioDir@Temp\4_ModeChoice\HH4_PercTrips_segment_@purp@.dbf'
    FILEI ZDATI[7] = '@ParentDir@@ScenarioDir@Temp\4_ModeChoice\HH5_PercTrips_segment_@purp@.dbf'
    FILEI ZDATI[8] = '@ParentDir@@ScenarioDir@Temp\4_ModeChoice\HH6_PercTrips_segment_@purp@.dbf'
    
    FILEI MATI[1]  = '@ParentDir@@ScenarioDir@Temp\4_ModeChoice\access_to_transit_markets.mtx' 
    FILEI MATI[2]  = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\skm_w4_@prd@.mtx'
    FILEI MATI[3]  = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\skm_d4_@prd@.mtx'
    FILEI MATI[4]  = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\skm_w6_@prd@.mtx'
    FILEI MATI[5]  = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\skm_d6_@prd@.mtx'
    FILEI MATI[6]  = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\skm_w7_@prd@.mtx'
    FILEI MATI[7]  = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\skm_d7_@prd@.mtx'
    FILEI MATI[8]  = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\skm_w8_@prd@.mtx'
    FILEI MATI[9]  = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\skm_d8_@prd@.mtx'
    FILEI MATI[10] = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\skm_auto_@prd@.mtx'

    FILEI MATI[14]  = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\skm_w9_@prd@.mtx'
    FILEI MATI[15]  = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\skm_d9_@prd@.mtx'
    
    FILEI MATI[18] = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\skm_w5_@prd@.mtx'
    FILEI MATI[19] = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\skm_d5_@prd@.mtx'
    FILEI MATI[20] = '@ParentDir@@ScenarioDir@4_ModeChoice\1a_Skims\direct_walk_premium_@prd@.mtx'
        


    FILEO MATO[1]  = '@ParentDir@@ScenarioDir@6_REMM\_logsums_@purp@_@prd@.mtx', 
        MO=8-10, 
        name=0veh,
             1veh,
             2veh 
    
    FILEO MATO[2]  = '@ParentDir@@ScenarioDir@Temp\4_ModeChoice\@purp@_logsums_@prd@.mtx', 
        MO=33-38, 
        name=0veh_lo,
             0veh_hi,
             1veh_lo,
             1veh_hi,
             2veh_lo,
             2veh_hi
    
    
    ;Cluster: distribute intrastep processing
    DistributeINTRASTEP PROCESSID=ClusterNodeID, PROCESSLIST=2-@CoresAvailable@
    
    
    ZONEMSG = 1
   
    ;assign skims and mode choice data into working matrices
    READ FILE = '@ParentDir@2_ModelScripts\4_ModeChoice\block\HBW_HBO_working_matrices.block'     
    
    ;read mode choice model coefficients and alternative specific constants  
    READ FILE = '@ParentDir@2_ModelScripts\4_ModeChoice\coeffs\@purp@_MC_coefficients.txt'
    READ FILE = '@ParentDir@2_ModelScripts\4_ModeChoice\coeffs\1Nesting_Constants.txt'
    READ FILE = '@ParentDir@2_ModelScripts\4_ModeChoice\coeffs\@purp@_MC_constants_@prd@_@n_1@.txt'      
    
    ;specify arrays for REMM logsum calculations
    READ FILE = '@ParentDir@2_ModelScripts\4_ModeChoice\block\HBW_REMM_arrays.block'
    READ FILE = '@ParentDir@2_ModelScripts\4_ModeChoice\block\HBW_Output_Logsums_arrays.block'
    

  percenttrips_0veh_LowInc = zi.3.P_ILW0V0[z] + zi.3.P_ILW1V0[z] + zi.3.P_ILW2V0[z] + zi.3.P_ILW3V0[z] +
                             zi.4.P_ILW0V0[z] + zi.4.P_ILW1V0[z] + zi.4.P_ILW2V0[z] + zi.4.P_ILW3V0[z] +
                             zi.5.P_ILW0V0[z] + zi.5.P_ILW1V0[z] + zi.5.P_ILW2V0[z] + zi.5.P_ILW3V0[z] +
                             zi.6.P_ILW0V0[z] + zi.6.P_ILW1V0[z] + zi.6.P_ILW2V0[z] + zi.6.P_ILW3V0[z] +
                             zi.7.P_ILW0V0[z] + zi.7.P_ILW1V0[z] + zi.7.P_ILW2V0[z] + zi.7.P_ILW3V0[z] +
                             zi.8.P_ILW0V0[z] + zi.8.P_ILW1V0[z] + zi.8.P_ILW2V0[z] + zi.8.P_ILW3V0[z]
  
  ;3 highest income quartiles                    
  percenttrips_0veh_HighInc = zi.3.P_IHW0V0[z] + zi.3.P_IHW1V0[z] + zi.3.P_IHW2V0[z] + zi.3.P_IHW3V0[z] +
                              zi.4.P_IHW0V0[z] + zi.4.P_IHW1V0[z] + zi.4.P_IHW2V0[z] + zi.4.P_IHW3V0[z] +
                              zi.5.P_IHW0V0[z] + zi.5.P_IHW1V0[z] + zi.5.P_IHW2V0[z] + zi.5.P_IHW3V0[z] +
                              zi.6.P_IHW0V0[z] + zi.6.P_IHW1V0[z] + zi.6.P_IHW2V0[z] + zi.6.P_IHW3V0[z] +
                              zi.7.P_IHW0V0[z] + zi.7.P_IHW1V0[z] + zi.7.P_IHW2V0[z] + zi.7.P_IHW3V0[z] +
                              zi.8.P_IHW0V0[z] + zi.8.P_IHW1V0[z] + zi.8.P_IHW2V0[z] + zi.8.P_IHW3V0[z]
         
  ;1 vehicle segment
  ;Lowest income quartile
  percenttrips_1veh_LowInc = zi.3.P_ILW0V1[z] + zi.3.P_ILW1V1[z] + zi.3.P_ILW2V1[z] + zi.3.P_ILW3V1[z] +
                             zi.4.P_ILW0V1[z] + zi.4.P_ILW1V1[z] + zi.4.P_ILW2V1[z] + zi.4.P_ILW3V1[z] +
                             zi.5.P_ILW0V1[z] + zi.5.P_ILW1V1[z] + zi.5.P_ILW2V1[z] + zi.5.P_ILW3V1[z] +
                             zi.6.P_ILW0V1[z] + zi.6.P_ILW1V1[z] + zi.6.P_ILW2V1[z] + zi.6.P_ILW3V1[z] +
                             zi.7.P_ILW0V1[z] + zi.7.P_ILW1V1[z] + zi.7.P_ILW2V1[z] + zi.7.P_ILW3V1[z] +
                             zi.8.P_ILW0V1[z] + zi.8.P_ILW1V1[z] + zi.8.P_ILW2V1[z] + zi.8.P_ILW3V1[z]

  ;3 highest income quartiles                    
  percenttrips_1veh_HighInc = zi.3.P_IHW0V1[z] + zi.3.P_IHW1V1[z] + zi.3.P_IHW2V1[z] + zi.3.P_IHW3V1[z] +
                              zi.4.P_IHW0V1[z] + zi.4.P_IHW1V1[z] + zi.4.P_IHW2V1[z] + zi.4.P_IHW3V1[z] +
                              zi.5.P_IHW0V1[z] + zi.5.P_IHW1V1[z] + zi.5.P_IHW2V1[z] + zi.5.P_IHW3V1[z] +
                              zi.6.P_IHW0V1[z] + zi.6.P_IHW1V1[z] + zi.6.P_IHW2V1[z] + zi.6.P_IHW3V1[z] +
                              zi.7.P_IHW0V1[z] + zi.7.P_IHW1V1[z] + zi.7.P_IHW2V1[z] + zi.7.P_IHW3V1[z] +
                              zi.8.P_IHW0V1[z] + zi.8.P_IHW1V1[z] + zi.8.P_IHW2V1[z] + zi.8.P_IHW3V1[z]
 
  ;2+ vehicle segment
  ;Lowest income quartile
  percenttrips_2veh_LowInc = zi.3.P_ILW0V2[z] + zi.3.P_ILW1V2[z] + zi.3.P_ILW2V2[z] + zi.3.P_ILW3V2[z] +
                             zi.4.P_ILW0V2[z] + zi.4.P_ILW1V2[z] + zi.4.P_ILW2V2[z] + zi.4.P_ILW3V2[z] +
                             zi.5.P_ILW0V2[z] + zi.5.P_ILW1V2[z] + zi.5.P_ILW2V2[z] + zi.5.P_ILW3V2[z] +
                             zi.6.P_ILW0V2[z] + zi.6.P_ILW1V2[z] + zi.6.P_ILW2V2[z] + zi.6.P_ILW3V2[z] +
                             zi.7.P_ILW0V2[z] + zi.7.P_ILW1V2[z] + zi.7.P_ILW2V2[z] + zi.7.P_ILW3V2[z] +
                             zi.8.P_ILW0V2[z] + zi.8.P_ILW1V2[z] + zi.8.P_ILW2V2[z] + zi.8.P_ILW3V2[z] + 
                             zi.3.P_ILW0V3[z] + zi.3.P_ILW1V3[z] + zi.3.P_ILW2V3[z] + zi.3.P_ILW3V3[z] +
                             zi.4.P_ILW0V3[z] + zi.4.P_ILW1V3[z] + zi.4.P_ILW2V3[z] + zi.4.P_ILW3V3[z] +
                             zi.5.P_ILW0V3[z] + zi.5.P_ILW1V3[z] + zi.5.P_ILW2V3[z] + zi.5.P_ILW3V3[z] +
                             zi.6.P_ILW0V3[z] + zi.6.P_ILW1V3[z] + zi.6.P_ILW2V3[z] + zi.6.P_ILW3V3[z] +
                             zi.7.P_ILW0V3[z] + zi.7.P_ILW1V3[z] + zi.7.P_ILW2V3[z] + zi.7.P_ILW3V3[z] +
                             zi.8.P_ILW0V3[z] + zi.8.P_ILW1V3[z] + zi.8.P_ILW2V3[z] + zi.8.P_ILW3V3[z] 

  ;3 highest income quartiles                    
  percenttrips_2veh_HighInc = zi.3.P_IHW0V2[z] + zi.3.P_IHW1V2[z] + zi.3.P_IHW2V2[z] + zi.3.P_IHW3V2[z] +
                              zi.4.P_IHW0V2[z] + zi.4.P_IHW1V2[z] + zi.4.P_IHW2V2[z] + zi.4.P_IHW3V2[z] +
                              zi.5.P_IHW0V2[z] + zi.5.P_IHW1V2[z] + zi.5.P_IHW2V2[z] + zi.5.P_IHW3V2[z] +
                              zi.6.P_IHW0V2[z] + zi.6.P_IHW1V2[z] + zi.6.P_IHW2V2[z] + zi.6.P_IHW3V2[z] +
                              zi.7.P_IHW0V2[z] + zi.7.P_IHW1V2[z] + zi.7.P_IHW2V2[z] + zi.7.P_IHW3V2[z] +
                              zi.8.P_IHW0V2[z] + zi.8.P_IHW1V2[z] + zi.8.P_IHW2V2[z] + zi.8.P_IHW3V2[z] + 
                              zi.3.P_IHW0V3[z] + zi.3.P_IHW1V3[z] + zi.3.P_IHW2V3[z] + zi.3.P_IHW3V3[z] +
                              zi.4.P_IHW0V3[z] + zi.4.P_IHW1V3[z] + zi.4.P_IHW2V3[z] + zi.4.P_IHW3V3[z] +
                              zi.5.P_IHW0V3[z] + zi.5.P_IHW1V3[z] + zi.5.P_IHW2V3[z] + zi.5.P_IHW3V3[z] +
                              zi.6.P_IHW0V3[z] + zi.6.P_IHW1V3[z] + zi.6.P_IHW2V3[z] + zi.6.P_IHW3V3[z] +
                              zi.7.P_IHW0V3[z] + zi.7.P_IHW1V3[z] + zi.7.P_IHW2V3[z] + zi.7.P_IHW3V3[z] +
                              zi.8.P_IHW0V3[z] + zi.8.P_IHW1V3[z] + zi.8.P_IHW2V3[z] + zi.8.P_IHW3V3[z] 
   
   if (percenttrips_0veh_LowInc + percenttrips_0veh_HighInc > 0)
    perc_low_inc_0 = (percenttrips_0veh_LowInc/(percenttrips_0veh_LowInc + percenttrips_0veh_HighInc))
    perc_high_inc_0 = 1-perc_low_inc_0
   else 
    perc_low_inc_0 = 0
    perc_high_inc_0 = 0
   endif 
   
   if (percenttrips_1veh_LowInc + percenttrips_1veh_HighInc > 0)
    perc_low_inc_1 = (percenttrips_1veh_LowInc/(percenttrips_1veh_LowInc + percenttrips_1veh_HighInc))
    perc_high_inc_1 = 1-perc_low_inc_1
   else 
    perc_low_inc_1 = 0
    perc_high_inc_1 = 0
   endif 
   
   if (percenttrips_2veh_LowInc + percenttrips_2veh_HighInc > 0)
    perc_low_inc_2 = (percenttrips_2veh_LowInc/(percenttrips_2veh_LowInc + percenttrips_2veh_HighInc))
    perc_high_inc_2 = 1-perc_low_inc_2
   else 
    perc_low_inc_2 = 0
    perc_high_inc_2 = 0
   endif 

    loop VEH=1,3,1   ;loop through vehicle ownership segments
       
      ;assign alternative specific constants for each vehicle ownership segment
      READ FILE='@ParentDir@2_ModelScripts\4_ModeChoice\block\HBW_HBO_calculate_asc.block' 
       
      loop INC=1,2,1   ;loop through income segments

          ;based on income - assign a cost coefficient
          IF (INC==1) 
            cost_coef = cost_lowinc_coef_@purp@
            parkcost_coef = parkcost_lowinc_coef_@purp@
          ELSE
            cost_coef = cost_highinc_coef_@purp@
            parkcost_coef = parkcost_highinc_coef_@purp@  
          ENDIF     
 
        loop ACCESS=1,3,1    ;loop through access-to-transit segments

          jloop
            if (i == @dummyzones@ | i == @externalzones@ | j == @dummyzones@ | j == @externalzones@)
                sum_EU = 0
                perc_walk  = 0
                perc_drive = 0
                perc_none  = 0
            else
                ;calculate utilities & relative probabilities, working way up the nesting structure
                READ FILE='@ParentDir@2_ModelScripts\4_ModeChoice\block\HBW_HBO_calculate_utilities.block'              
            endif
                          
            READ FILE='@ParentDir@2_ModelScripts\4_ModeChoice\block\HBW_Output_Logsums.block'
            READ FILE='@ParentDir@2_ModelScripts\4_ModeChoice\block\HBW_REMM.block'
          endjloop  

        endloop   ;ACCESS
      endloop   ;INC
    endloop   ;VEH
     
   ENDRUN




;print timestamp
RUN PGM=MATRIX
    
    ZONES = 1
    
    ScriptEndTime = currenttime()
    ScriptRunTime = ScriptEndTime - @ScriptStartTime@
    
    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_RunTime - @RID@.txt',
        APPEND=T,
        LIST='\n',
             '\n    HBW Logsums                        ', formatdatetime(@ScriptStartTime@, 40, 0, 'yyyy-mm-dd,  hh:nn:ss'), 
                 ',  ', formatdatetime(ScriptRunTime, 40, 0, 'hhh:nn:ss')
    
ENDRUN



        
*(DEL 06_HBW_logsums.txt)  
