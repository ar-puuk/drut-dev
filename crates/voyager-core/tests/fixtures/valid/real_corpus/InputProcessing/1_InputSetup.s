
;System
;In case TP+ crashes during batch, this will halt process & help identify error.
*(ECHO model crashed > 1_InputSetup.txt)
    

    

;if not using speed and capacity override, supply a valid/dummy field name to prevent
;script from crashing
if (SpeedOverride<>1)     SpeedOverrideField    = 'A'
if (CapacityOverride<>1)  CapacityOverrideField = 'A'


;check that added node and link fields in control center are not blank to prevent script
;from crashing
if (''=AddNodeFields)  AddNodeFields = ' '
if (''=AddLinkFields)  AddLinkFields = ' '


;get start time
ScriptStartTime = currenttime()



;check if added node or link variables present on master network
RUN PGM=NETWORK  MSG='Set Up: check links and nodes'
    FILEI NETI = '@ParentDir@1_Inputs\3_Highway\@MasterPrefix@.net'
    
    FILEO NODEO = '@ParentDir@@ScenarioDir@Temp\0_InputProcessing\0a_Tmp_checkAddNodeFields.dbf', 
        FORMAT=DBF, 
        INCLUDE=N, X, Y @AddNodeFields@
    
    FILEO LINKO = '@ParentDir@@ScenarioDir@Temp\0_InputProcessing\0a_Tmp_checkAddLinkFields.dbf', 
        FORMAT=DBF, 
        INCLUDE=A, B @AddLinkFields@
ENDRUN



;Check if TAZ, SE, & Transit Walk Buffer files and fields exist and begin the LOG file
RUN PGM=NETWORK  MSG='Set Up: check TAZ, SE, and Transit Walk Buffer, begin LOG'
    FILEI  NETI[1] = '@ParentDir@1_Inputs\3_Highway\@MasterPrefix@.net'
          NODEI[2] = '@ParentDir@1_Inputs\1_TAZ\@TAZ_DBF@', 
              RENAME=TAZID-N
    
    ZONES = @UsedZones@
    
    
    PHASE=NODEMERGE
        ;check if requred highway network node fields are present
        check = ni.1.N            + 
                ni.1.X            + 
                ni.1.Y            + 
                ni.1.TAZID        + 
                ni.1.EXTERNAL     + 
                ni.1.@pnr_field@  +
                ni.1.@CRT_Fare_Zone@
        
        checkstring = ni.1.NODENAME
        
        
        ;check if required TAZ fields are present
        check = ni.2.ACRES      +
                ni.2.DEVACRES   + 
                ni.2.CBD        + 
                ni.2.PRKCSTPERM + 
                ni.2.PRKCSTTEMP + 
                ni.2.TERMTIME   + 
                ni.2.CO_FIPS    + 
                ni.2.CITY_FIPS  + 
                ni.2.DISTLRG    + 
                ni.2.DISTMED    + 
                ni.2.DISTSML    + 
                ni.2.@EcoEdPassZones@  + 
                ni.2.@FreeFareZones@
                
        checkstring = ni.2.CO_NAME
        checkstring = ni.2.CITY_NAME
        checkstring = ni.2.DLRG_NAME
        checkstring = ni.2.DMED_NAME
        checkstring = ni.2.DSML_NAME
        
    ENDPHASE
    
    
    PHASE=LINKMERGE
        ;check if required highway network link fields are present
        check = li.1.A          + 
                li.1.B          + 
                li.1.DISTANCE   + 
                li.1.ONEWAY     + 
                li.1.TAZID      + 
                li.1.@TruckRestrict@ 
                
        check = li.1.@LNfield@        + 
                li.1.@FTfield@        + 
                li.1.@HOVmarker@      + 
                li.1.@SpdFactor@      + 
                li.1.@CapFactor@      + 
                li.1.@TranSpeedField@ + 
                li.1.@Rel_LN@
        
        checkstring = li.1.STREET
        checkstring = li.1.@SEGIDField@
        
        ;check for required field if SpeedOverride=1
        if (@SpeedOverride@=1)
            check = li.1.@SpeedOverrideField@
        endif
        
        ;check for required field if CapacityOverride=1
        if (@CapacityOverride@=1)
            check = li.1.@CapacityOverrideField@
        endif
        
        ;check for required field if SegIdExField isn't empty
        if (STRLEN(TRIM(@SEGIDExField@))>0)
            checkstring = li.1.@SEGIDExField@
        endif

    ENDPHASE
    
    
    PHASE=SUMMARY
        ;print start of LOG file
        PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', 
            APPEND=F, 
            LIST=';=============================================================================\n',
                 '\n',
                 '                               MODEL RUN LOG\n',
                 '\n',
                 '  @Description@\n',
                 ';=============================================================================\n',
                 '\n',
                 'General Identification\n',
                 '  Model Version                      @ModelVersion@\n',
                 '  User Name                          @UserName@\n',
                 '  User Company                       @UserCompany@\n',
                 '\n',
                 'General Variables for This Run\n',
                 '  Scenario Group                     @ScenarioGroup@\n',
                 '  Run Year                           @RunYear@\n',
                 '  Run ID                             @RID@\n',
                 '  Path to Model Scenario             @ParentDir@@ScenarioDir@\n',
                 '\n',
                 '  Run vizTool                        @Run_vizTool@\n',
                 '  Run TAZ Based Metrics              @Run_TAZBasedMetrics@\n',
                 '\n',
                 '\n',
                 'TAZ Variables\n',
                 '   TAZ Area File                     @TAZ_DBF@\n',
                 '      Large District Field           DISTLRG\n',
                 '      Medium District Field          DISTMED\n',
                 '      Small District Field           DISTSML\n',
                 '      Eco-Ed Transit pass Field      @EcoEdPassZones@\n',
                 '      Free Fare Transit Field        @FreeFareZones@\n',
                 '  \n',
                 '  \n',
                 'Demographic Variables\n',
                 '   Demographic Year                  @demographicyear@\n',
                 '   Box Elder SE File                 @BE_SEFile@\n',
                 '   WFRC SE File                      @WFRC_SEFile@\n',
                 '   MAG SE File                       @MAG_SEFile@\n',
                 '  \n',
                 '  \n',
                 'Highway Network Variables\n',
                 '   Network Year                      @networkyear@\n',
                 '   Scenario Network                  @RID@.net\n',
                 '   Master Network File               @MasterPrefix@.net\n',
                 '     HOT Zone Update Polygon File    @tollz_shp@\n',
                 '     Node Fields\n',
                 '       Park & Ride Field             @pnr_field@\n',
                 '       CRT Fare Zone Field           @CRT_Fare_Zone@\n',
                 '       Added Node Fields             @AddNodeFields@\n',
                 '     Link Fields\n',
                 '       Lane Field                    @LNfield@\n',
                 '       FT Field                      @FTfield@\n',
                 '       HOV Field                     @HOVmarker@\n',
                 '       Speed Factor Field            @SpdFactor@\n',
                 '       Capacity Factor Field         @CapFactor@\n',
                 '       Transit Speed Field           @TranSpeedField@\n',
                 '       Reliability Lanes Field       @Rel_LN@\n',
                 '       Min HOT Toll                  @HOT_Toll_Min@\n',
                 '       Max HOT Toll                  @HOT_Toll_Max@\n',
                 '       Added Link Fields             @AddLinkFields@\n',
                 '  \n',
                 'Transit Network Variables\n',
                 '   Transit Files Dir                 @Mlin@\n',
                 '\n',
                 '\n',
                 'External Trip Variables\n',
                 '   External Volume Count             @Ext_Vol_Count@\n',
                 '   External Trip End Pattern         @Ext_TripEndPattern@\n',
                 '   External Trip Table               @Ext_TripTable@\n',
                 '\n',
                 '\n',
                 'Speed & Capacity\n',
                 '   Spd-Cap Lookup File               1_Inputs\0_GlobalData\0_SpeedCap\@SpeedCapLookupFile@\n',
                 '\n'
      
        ;print speed override fields to LOG file
        if (@SpeedOverride@=1)
            PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', LIST=
            '   Speed Override Function Is Turned On\n',
            '      Speed Override Field           @SpeedOverrideField@\n',
            '\n'
        endif
        
        ;print capacity override fields to LOG file
        if (@CapacityOverride@=1)
            PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', LIST=
            '   Capacity Override Function Is Turned On\n',
            '      Capacity Override Field        @SpeedOverrideField@\n',
            '\n'
        endif
        
        
        ;print visual break for start of next section
        PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', LIST=
        ';*********************************************************************\n'
    ENDPHASE
    
ENDRUN



;check @TAZ_DBF@ input file
RUN PGM=MATRIX
    FILEI DBI[1] = '@ParentDir@1_Inputs\1_TAZ\@TAZ_DBF@', 
        SORT=TAZID, 
        AUTOARRAY=ALLFIELDS
    
    ;set MATRIX parameters
    ZONES = 1
    
    
    ;print checking @TAZ_DBF@ header to LOG file
    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', APPEND=T, LIST=
    'Checking @TAZ_DBF@ for inconsistencies:'
    
    
    ;check for 0 TAZID's
    LOOP RecLp=1,DBI.1.NUMRECORDS
        ;set variables
        TAZ_NUM  = dba.1.TAZID[RecLp]        
        
        ;check if DBF TAZ start with 0
        if (TAZ_NUM=0)
            ;count nubmer of TAZ with 0 for SHPTAZID
            CountZeroTAZID = CountZeroTAZID + 1
            
            ;toggle check_TAZ0 flag
            check_TAZ0 = 1
        endif
        
        ;print errors to LOG
        if (check_TAZ0=1 & RecLp=DBI.1.NUMRECORDS)
            ;print error message to LOG
            PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', FORM=1.0, LIST=
            '  TAZ with 0 ID:' CountZeroTAZID(5.0) ' records have 0 for a TAZID\n'
        endif
    ENDLOOP
    
    
    ;check gaps in TAZ numbering are dummy zones
    LOOP RecLp=1,DBI.1.NUMRECORDS
        ;set variables
        TAZ_Cnt  = TAZ_Cnt + 1
        TAZ_NUM  = dba.1.TAZID[RecLp]
        TAZ_Prev = dba.1.TAZID[RecLp-1]
        
        ;check gaps in records
        if (TAZ_Cnt<>TAZ_NUM)
            ;decide how to handle gaps found
            if (TAZ_NUM=TAZ_Prev & RecLp>1)
                ;skip duplicate record, will be checked in next section
                TAZ_Cnt = TAZ_Cnt - 1
            else
                ;determine number of zones missing from check file
                GapVal = TAZ_NUM - TAZ_Cnt
                
                ;loop through missing zones
                LOOP lpGap=1,GapVal
                    ;check if missing TAZ is a dummy or unused zone
                    if (TAZ_Cnt>@UsedZones@ | TAZ_Cnt=@externalzones@)
                        ;gap is unused zone, do not report in log
                    
                    else
                        ;if first error print header
                        if (check_Gap=0)
                            PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', FORM=1.0, LIST=
                            '  The following Used Zones are missing in @TAZ_DBF@:'
                            
                            ;toggle check variable
                            check_Gap = 1
                        endif
                            
                        ;print error to LOG
                        PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', FORM=10.0, LIST=TAZ_Cnt
                    endif
                    
                    ;increment count variable
                    TAZ_Cnt = TAZ_Cnt + 1
                ENDLOOP
            endif
        endif
    ENDLOOP
    
    
    ;check for duplicate SHPTAZID
    LOOP RecLp=1,DBI.1.NUMRECORDS
        ;set variables
        TAZ_NUM  = dba.1.TAZID[RecLp]
        
        ;check for duplicate records
        if (RecLp<DBI.1.NUMRECORDS)
            TAZ_Next = dba.1.TAZID[RecLp + 1]
            
            if (TAZ_NUM=TAZ_Next & TAZ_NUM>0)
                ;if first error print header
                if(check_Duplicate=0)
                    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', LIST=
                    '\n',
                    '\n',
                    '  Duplicate SHPTAZID found:'
                    
                    ;toggle check variable
                    check_Duplicate = 1
                endif
                
                PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', FORM=10.0, LIST=TAZ_NUM
            endif
        endif
    ENDLOOP
    
    
    ;check if used zone has total 0 for Total Area
    LOOP RecLp=1,DBI.1.NUMRECORDS
        ;set variables
        TAZ_NUM  = dba.1.TAZID[RecLp]
        
        ;check gaps in records
        if (dba.1.ACRES[RecLp]=0)
            if (TAZ_NUM=@externalzones@)
                ;not an error
            else
                ;if first error print header
                if (check_TotArea=0)
                    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', FORM=1.0, LIST=
                    '\n',
                    '\n',
                    '  The following Used Zones have TotalArea=0:'
                    
                    ;toggle check variable
                    check_TotArea = 1
                endif
                
                ;print error message to LOG
                PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', FORM=10.0, LIST=TAZ_NUM
            endif
        endif
    ENDLOOP
    
    
    ;check TAZ Area
    LOOP RecLp=1,DBI.1.NUMRECORDS
        ;set variables
        TAZ_NUM  = dba.1.TAZID[RecLp]
        
        ;check if Developable Acres > Total Acres
        if (dba.1.DEVACRES[RecLp]>dba.1.ACRES[RecLp])
            if(check_DevAcre=0)
                ;print error message to LOG
                PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', LIST=
                '\n',
                '\n',
                '  Developable Acres > Total Acres for the following SHPTAZID:'
                
                ;toggle check_Acre variable
                check_DevAcre = 1
            endif
            
            PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', FORM=10.0, LIST=TAZ_NUM
        endif
    ENDLOOP
    
    
    ;if no errors, print message in log
    if (check_TAZ0=0 & check_Gap=0 & check_Duplicate=0 & check_TotArea=0 & check_DevAcre=0)
        PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', LIST=
            '  No inconsistencies found'
    endif
    
    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_LogFile - @RID@.txt', LIST='\n'

ENDRUN



;start model runtime report
RUN PGM=MATRIX
    
    ZONES = 1
    
    ScriptEndTime = currenttime()
    ScriptRunTime = ScriptEndTime - @ScriptStartTime@
    
    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_RunTime - @RID@.txt',
        REWIND=T,
        LIST='=======================================================================',
             '\n                           MODEL RUNTIME REPORT',
             '\n=======================================================================',
             '\n',
             '\nScenario:  @RID@',
             '\n',
             '\n',
             '\n----------------------------------------------------------------------',
             '\nINPUT PROCESSING',
             '\n    Setup                              ', formatdatetime(@ScriptStartTime@, 40, 0, 'yyyy-mm-dd,  hh:nn:ss'), 
                 ',  ', formatdatetime(ScriptRunTime, 40, 0, 'hhh:nn:ss')
    
ENDRUN



;System cleanup
    *(DEL 1_InputSetup.txt)
