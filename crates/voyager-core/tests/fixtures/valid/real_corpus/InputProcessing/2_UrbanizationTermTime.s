
;System
    ;In case TP+ crashes during batch, this will halt process & help identify error.
    *(ECHO model crashed > 2_UrbanizationTermTime.txt)



;get start time
ScriptStartTime = currenttime()




;output a temp dbf file with node, X coord, Y coord
RUN PGM=NETWORK  MSG='SE Processing 2: urbanization - get node X & Y coordinates'
    FILEI NETI     = '@ParentDir@1_Inputs\3_Highway\@MasterPrefix@.net'
    
    FILEO NODEO    = '@ParentDir@@ScenarioDir@Temp\0_InputProcessing\0b_StartUrbanTermTimeFile.dbf', FORMAT=DBF,
                     INCLUDE=N, X, Y
    
    ZONES = @Usedzones@
ENDRUN




;find nearest zones to current zone, calculate urbanization value, area type and terminal times

;write out urbanization file
RUN PGM=MATRIX  MSG='SE Processing 2: urbanization - calc urbanization-terminal times'
    FILEI ZDATI[1] = '@ParentDir@@ScenarioDir@Temp\0_InputProcessing\0b_StartUrbanTermTimeFile.dbf', Z=N
          ZDATI[2] = '@ParentDir@1_Inputs\1_TAZ\@TAZ_DBF@', Z=TAZID
          ZDATI[3] = '@ParentDir@@ScenarioDir@0_InputProcessing\SE_File_@RID@.dbf'
    
    FILEO RECO[1]  = '@ParentDir@@ScenarioDir@0_InputProcessing\Urbanization.dbf', 
        FORM=11.0,
        FIELDS=Z, 
        X(13.5), 
        Y(13.5), 
        ACRES(11.2), 
        DEVACRES(11.2),
            SQMILE(11.5), 
            CBD, 
            PRKCSTPERM(10.2), 
            PRKCSTTEMP(10.2), 
            POP5, 
            EMP5,
            DEVACRE5, 
            URBANVAL(11.2), 
            ATYPE(5.0), 
            TERMTIME(5.0)
    
    
    ;set MATRIX parameters
    ZONES   = @UsedZones@
    ZONEMSG = @ZoneMsgRate@
    
    
    ;define arrays
    ARRAY neighbors   = 9,
             HHPOP    = 9,
             totemp   = 9,
             retemp   = 9,
             distance = 9,
             devacres = 9
    
    ARRAY popninezone      = ZONES,
          empninezone      = ZONES,
          retninezone      = ZONES,
          devacresninezone = ZONES,
          urban            = ZONES
    
    
    
    ;initialize variables
    startj      = 1
    endj        = @Usedzones@
    maxmindist  = 9999999999
    distance[1] = 9999999999
    distance[2] = 9999999999
    distance[3] = 9999999999
    distance[4] = 9999999999
    distance[5] = 9999999999
    distance[6] = 9999999999
    distance[7] = 9999999999
    distance[8] = 9999999999
    distance[9] = 9999999999
    
    
    
    ;exclude dummy zones and externals
    if (i=@dummyzones@ | i=@externalzones@)
        ;skip procssing record
    
    else
        ;loop through zones tallying stats on nearest 8 zones to zone I
        JLOOP J=startj, endj, 1 exclude=@dummyzones@, @externalzones@
            
            ;calculate distance from zone I to zone J
            dist = SQRT((zi.1.X[I] - zi.1.X[J])^2 + (zi.1.Y[I] - zi.1.Y[J])^2)
            
            ;check if distance is less than maximum distance then determine its proximity
            ;and assign new data to arrays if one of 4 closest zones
            if ((dist < maxmindist) & (dist > 0))
                ;check if zone J is closest zone update arrays if yes
                if (dist < distance[1])
                
                    neighbors[9] = neighbors[8]  ;9th closest
                    neighbors[8] = neighbors[7]  ;8th closest                
                    neighbors[7] = neighbors[6]  ;7th closest
                    neighbors[6] = neighbors[5]  ;6th closest
                    neighbors[5] = neighbors[4]  ;5th closest
                    neighbors[4] = neighbors[3]  ;4th closest
                    neighbors[3] = neighbors[2]  ;3rd closest
                    neighbors[2] = neighbors[1]  ;2nd closest
                    neighbors[1] = J             ;1st closest
                    
                    HHPOP[9] = HHPOP[8]
                    HHPOP[8] = HHPOP[7]
                    HHPOP[7] = HHPOP[6]
                    HHPOP[6] = HHPOP[5]
                    HHPOP[5] = HHPOP[4]
                    HHPOP[4] = HHPOP[3]
                    HHPOP[3] = HHPOP[2]
                    HHPOP[2] = HHPOP[1]
                    HHPOP[1] = zi.3.HHPOP[J]
                    
                    totemp[9] = totemp[8]
                    totemp[8] = totemp[7]
                    totemp[7] = totemp[6]
                    totemp[6] = totemp[5]
                    totemp[5] = totemp[4]
                    totemp[4] = totemp[3]
                    totemp[3] = totemp[2]
                    totemp[2] = totemp[1]
                    totemp[1] = zi.3.TOTEMP[J]
                    
                    retemp[9] = retemp[8]
                    retemp[8] = retemp[7]
                    retemp[7] = retemp[6]
                    retemp[6] = retemp[5]
                    retemp[5] = retemp[4]
                    retemp[4] = retemp[3]
                    retemp[3] = retemp[2]
                    retemp[2] = retemp[1]
                    retemp[1] = zi.3.RETEMP[J]
                    
                    devacres[9] = devacres[8]
                    devacres[8] = devacres[7]
                    devacres[7] = devacres[6]
                    devacres[6] = devacres[5]
                    devacres[5] = devacres[4]
                    devacres[4] = devacres[3]
                    devacres[3] = devacres[2]
                    devacres[2] = devacres[1]
                    devacres[1] = zi.2.DEVACRES[J]
                    
                    maxmindist  = distance[8]
                    distance[9] = distance[8]
                    distance[8] = distance[7]
                    distance[7] = distance[6]
                    distance[6] = distance[5]
                    distance[5] = distance[4]
                    distance[4] = distance[3]
                    distance[3] = distance[2]
                    distance[2] = distance[1]
                    distance[1] = dist
                    
                ;check if zone J is 2nd closest zone and update arrays if yes
                elseif (dist < distance[2])
                    neighbors[9] = neighbors[8]
                    neighbors[8] = neighbors[7]                
                    neighbors[7] = neighbors[6]
                    neighbors[6] = neighbors[5]
                    neighbors[5] = neighbors[4]
                    neighbors[4] = neighbors[3]
                    neighbors[3] = neighbors[2]
                    neighbors[2] = J
                    
                    HHPOP[9] = HHPOP[8]
                    HHPOP[8] = HHPOP[7]
                    HHPOP[7] = HHPOP[6]
                    HHPOP[6] = HHPOP[5]
                    HHPOP[5] = HHPOP[4]
                    HHPOP[4] = HHPOP[3]
                    HHPOP[3] = HHPOP[2]
                    HHPOP[2] = zi.3.HHPOP[J]
                    
                    totemp[9] = totemp[8]
                    totemp[8] = totemp[7]
                    totemp[7] = totemp[6]
                    totemp[6] = totemp[5]
                    totemp[5] = totemp[4]
                    totemp[4] = totemp[3]
                    totemp[3] = totemp[2]
                    totemp[2] = zi.3.TOTEMP[J]
                    
                    retemp[9] = retemp[8]
                    retemp[8] = retemp[7]
                    retemp[7] = retemp[6]
                    retemp[6] = retemp[5]
                    retemp[5] = retemp[4]
                    retemp[4] = retemp[3]
                    retemp[3] = retemp[2]
                    retemp[2] = zi.3.RETEMP[J]
                    
                    devacres[9] = devacres[8]
                    devacres[8] = devacres[7]
                    devacres[7] = devacres[6]
                    devacres[6] = devacres[5]
                    devacres[5] = devacres[4]
                    devacres[4] = devacres[3]
                    devacres[3] = devacres[2]
                    devacres[2] = zi.2.DEVACRES[J]
                    
                    maxmindist  = distance[8]
                    distance[9] = distance[8]
                    distance[8] = distance[7]
                    distance[7] = distance[6]
                    distance[6] = distance[5]
                    distance[5] = distance[4]
                    distance[4] = distance[3]
                    distance[3] = distance[2]
                    distance[2] = dist
                    
                ;check if zone J is 3rd closest zone and update arrays if yes
                elseif (dist < distance[3])
                    
                    neighbors[9] = neighbors[8]
                    neighbors[8] = neighbors[7]
                    neighbors[7] = neighbors[6]
                    neighbors[6] = neighbors[5]
                    neighbors[5] = neighbors[4]
                    neighbors[4] = neighbors[3]
                    neighbors[3] = J
                    
                    HHPOP[9] = HHPOP[8]
                    HHPOP[8] = HHPOP[7]
                    HHPOP[7] = HHPOP[6]
                    HHPOP[6] = HHPOP[5]
                    HHPOP[5] = HHPOP[4]
                    HHPOP[4] = HHPOP[3]
                    HHPOP[3] = zi.3.HHPOP[J]
                    
                    totemp[9] = totemp[8]
                    totemp[8] = totemp[7]
                    totemp[7] = totemp[6]
                    totemp[6] = totemp[5]
                    totemp[5] = totemp[4]
                    totemp[4] = totemp[3]
                    totemp[3] = zi.3.TOTEMP[J]
                    
                    retemp[9] = retemp[8]
                    retemp[8] = retemp[7]
                    retemp[7] = retemp[6]
                    retemp[6] = retemp[5]
                    retemp[5] = retemp[4]
                    retemp[4] = retemp[3]
                    retemp[3] = zi.3.RETEMP[J]
                    
                    devacres[9] = devacres[8]
                    devacres[8] = devacres[7]
                    devacres[7] = devacres[6]
                    devacres[6] = devacres[5]
                    devacres[5] = devacres[4]
                    devacres[4] = devacres[3]
                    devacres[3] = zi.2.DEVACRES[J]
                    
                    maxmindist  = distance[8]
                    distance[9] = distance[8]
                    distance[8] = distance[7]
                    distance[7] = distance[6]
                    distance[6] = distance[5]
                    distance[5] = distance[4]
                    distance[4] = distance[3]
                    distance[3] = dist
                    
                ;check if zone J is 4th closest zone and update arrays if yes
                elseif (dist < distance[4])
                    
                    neighbors[9] = neighbors[8]
                    neighbors[8] = neighbors[7]
                    neighbors[7] = neighbors[6]
                    neighbors[6] = neighbors[5]
                    neighbors[5] = neighbors[4]
                    neighbors[4] = J
                    
                    HHPOP[9] = HHPOP[8]
                    HHPOP[8] = HHPOP[7]
                    HHPOP[7] = HHPOP[6]
                    HHPOP[6] = HHPOP[5]
                    HHPOP[5] = HHPOP[4]
                    HHPOP[4] = zi.3.HHPOP[J]
                    
                    totemp[9] = totemp[8]
                    totemp[8] = totemp[7]
                    totemp[7] = totemp[6]
                    totemp[6] = totemp[5]
                    totemp[5] = totemp[4]
                    totemp[4] = zi.3.TOTEMP[J]
                    
                    retemp[9] = retemp[8]
                    retemp[8] = retemp[7]
                    retemp[7] = retemp[6]
                    retemp[6] = retemp[5]
                    retemp[5] = retemp[4]
                    retemp[4] = zi.3.RETEMP[J]
                    
                    devacres[9] = devacres[8]
                    devacres[8] = devacres[7]
                    devacres[7] = devacres[6]
                    devacres[6] = devacres[5]
                    devacres[5] = devacres[4]
                    devacres[4] = zi.2.DEVACRES[J]
                    
                    maxmindist  = distance[8]
                    distance[9] = distance[8]
                    distance[8] = distance[7]
                    distance[7] = distance[6]
                    distance[6] = distance[5]
                    distance[5] = distance[4]
                    distance[4] = dist            
                    
                ;check if zone J is 5th closest zone and update arrays if yes
                elseif (dist < distance[5])
                    neighbors[9] = neighbors[8]
                    neighbors[8] = neighbors[7]
                    neighbors[7] = neighbors[6]
                    neighbors[6] = neighbors[5]
                    neighbors[5] = J
                    
                    HHPOP[9] = HHPOP[8]
                    HHPOP[8] = HHPOP[7]
                    HHPOP[7] = HHPOP[6]
                    HHPOP[6] = HHPOP[5]
                    HHPOP[5] = zi.3.HHPOP[J]
                    
                    totemp[9] = totemp[8]
                    totemp[8] = totemp[7]
                    totemp[7] = totemp[6]
                    totemp[6] = totemp[5]
                    totemp[5] = zi.3.TOTEMP[J]
                    
                    retemp[9] = retemp[8]
                    retemp[8] = retemp[7]
                    retemp[7] = retemp[6]
                    retemp[6] = retemp[5]
                    retemp[5] = zi.3.RETEMP[J]
                    
                    devacres[9] = devacres[8]
                    devacres[8] = devacres[7]
                    devacres[7] = devacres[6]
                    devacres[6] = devacres[5]
                    devacres[5] = zi.2.DEVACRES[J]
                    
                    maxmindist  = distance[8]
                    distance[9] = distance[8]
                    distance[8] = distance[7]
                    distance[7] = distance[6]
                    distance[6] = distance[5]
                    distance[5] = dist            
                    
                ;check if zone J is 6th closest zone and update arrays if yes
                elseif (dist < distance[6])
                    neighbors[9] = neighbors[8]
                    neighbors[8] = neighbors[7]
                    neighbors[7] = neighbors[6]
                    neighbors[6] = J
                    
                    HHPOP[9] = HHPOP[8]
                    HHPOP[8] = HHPOP[7]
                    HHPOP[7] = HHPOP[6]
                    HHPOP[6] = zi.3.HHPOP[J]
                    
                    totemp[9] = totemp[8]
                    totemp[8] = totemp[7]
                    totemp[7] = totemp[6]
                    totemp[6] = zi.3.TOTEMP[J]
                    
                    retemp[9] = retemp[8]
                    retemp[8] = retemp[7]
                    retemp[7] = retemp[6]
                    retemp[6] = zi.3.RETEMP[J]
                    
                    devacres[9] = devacres[8]
                    devacres[8] = devacres[7]
                    devacres[7] = devacres[6]
                    devacres[6] = zi.2.DEVACRES[J]
                    
                    maxmindist  = distance[8]
                    distance[9] = distance[8]
                    distance[8] = distance[7]
                    distance[7] = distance[6]
                    distance[6] = dist
                    
                ;check if zone J is 7th closest zone and update arrays if yes
                elseif (dist < distance[7])
                    neighbors[9] = neighbors[8]
                    neighbors[8] = neighbors[7]
                    neighbors[7] = J
                    
                    HHPOP[9] = HHPOP[8]
                    HHPOP[8] = HHPOP[7]
                    HHPOP[7] = zi.3.HHPOP[J]
                    
                    totemp[9] = totemp[8]
                    totemp[8] = totemp[7]
                    totemp[7] = zi.3.TOTEMP[J]
                    
                    retemp[9] = retemp[8]
                    retemp[8] = retemp[7]
                    retemp[7] = zi.3.RETEMP[J]
                    
                    devacres[9] = devacres[8]
                    devacres[8] = devacres[7]
                    devacres[7] = zi.2.DEVACRES[J]
                    
                    maxmindist  = distance[8]
                    distance[9] = distance[8]
                    distance[8] = distance[7]
                    distance[7] = dist
                
                ;check if zone J is 8th closest zone and update arrays if yes
                elseif (dist < distance[8])
                    neighbors[9] = neighbors[8]
                    neighbors[8] = J
                    
                    HHPOP[9] = HHPOP[8]
                    HHPOP[8] = zi.3.HHPOP[J]
                    
                    totemp[9] = totemp[8]
                    totemp[8] = zi.3.TOTEMP[J]
                    
                    retemp[9] = retemp[8]
                    retemp[8] = zi.3.RETEMP[J]
                    
                    devacres[9] = devacres[8]
                    devacres[8] = zi.2.DEVACRES[J]
                    
                    maxmindist  = distance[8]
                    distance[9] = distance[8]
                    distance[8] = dist
                                        
                ;check if zone J is 9th closest zone and update arrays if yes (note, position 9 not currently used)
                elseif (dist < distance[9])
                    neighbors[9] = J
                    HHPOP[9]    = zi.3.HHPOP[J]
                    totemp[9]    = zi.3.TOTEMP[J]
                    retemp[9]    = zi.3.RETEMP[J]
                    devacres[9]  = zi.2.DEVACRES[J]
                    maxmindist   = dist
                    distance[9]  = dist
                endif
            endif
        ENDJLOOP
                
        ;print neighbors temp file
        if (I=1)
            PRINT FILE='@ParentDir@@ScenarioDir@Temp\0_InputProcessing\_Neighbors.csv', 
                CSV=T,
                FORM=10.0, 
                LIST='TAZ', 
                     'Neighbor1', 'Distance1', 
                     'Neighbor2', 'Distance2', 
                     'Neighbor3', 'Distance3', 
                     'Neighbor4', 'Distance4', 
                     'Neighbor5', 'Distance5', 
                     'Neighbor6', 'Distance6', 
                     'Neighbor7', 'Distance7', 
                     'Neighbor8', 'Distance8'
        endif
        
        PRINT FILE='@ParentDir@@ScenarioDir@Temp\0_InputProcessing\_Neighbors.csv', 
            CSV=T,
            FORM=10.0, 
            LIST=I, 
                 neighbors[1], distance[1](10.1), 
                 neighbors[2], distance[2](10.1), 
                 neighbors[3], distance[3](10.1), 
                 neighbors[4], distance[4](10.1), 
                 neighbors[5], distance[5](10.1), 
                 neighbors[6], distance[6](10.1), 
                 neighbors[7], distance[7](10.1), 
                 neighbors[8], distance[8](10.1)
        
        
        ;initialize 9-zone totals with data in zone I
        popninezone[I]      = zi.3.HHPOP[I]             ;population of zone
        empninezone[I]      = zi.3.TOTEMP[I]            ;employment of zone
        retninezone[I]      = zi.3.RETEMP[I]            ;employment of zone
        devacresninezone[I] = zi.2.DEVACRES[I]          ;area of zone
        
        
        ;add to zone I data from the 8 nearest zones
        loop k=1,8
            popninezone[I]      = popninezone[I]      + HHPOP[k]
            empninezone[I]      = empninezone[I]      + totemp[k]
            retninezone[I]      = retninezone[I]      + retemp[k]
            devacresninezone[I] = devacresninezone[I] + devacres[k]
        endloop
        
        
        ;calculate urbanization variable
        if (devacresninezone[I]=0)
            urban[I] = 0
        else
            urban[I] = (popninezone[I] + 2.07*empninezone[I]) / devacresninezone[I]
        endif
        
        
        ;assign terminal time based on urbanization value
        if (urban[I] <=1)
            termij = 1  ;Terminal Time associated with Rural area type
            AType  = 1
        elseif (urban[I] > 1 && urban[I] <=5)
            termij = 1  ;Terminal Time associated with Transistion area type
            AType  = 2
        elseif (urban[I] > 5 && urban[I] <=15)
            termij = 1  ;Terminal Time associated with Suburban area type
            AType  = 3
        elseif (urban[I] > 15 && urban[I] <=100)
            termij = 2  ;Terminal Time associated with Urban area type
            AType  = 4
        else
            termij = 3  ;Terminal Time associated with CBD-like area type
            AType  = 5
        endif
        
        
        ;overwrite terminal time if TERMTIME from TAZ file > 0
        if (zi.2.TERMTIME[I]>0)  termij = zi.2.TERMTIME[I]
        
        
        
        ;assign output variables and write to output file
        RO.Z             = Z
        RO.X             = zi.1.X
        RO.Y             = zi.1.Y
        RO.ACRES         = zi.2.ACRES
        RO.DEVACRES      = zi.2.DEVACRES
        RO.SQMILE        = zi.2.ACRES * 0.0015625  ;1 acre = 0.0015625 square miles
        RO.CBD           = zi.2.CBD
        RO.PRKCSTPERM    = zi.2.PRKCSTPERM
        RO.PRKCSTTEMP    = zi.2.PRKCSTTEMP
        
        RO.POP5          = popninezone[I]
        RO.EMP5          = empninezone[I]
        RO.DEVACRE5      = devacresninezone[I]
        RO.URBANVAL      = urban[I]
        RO.TERMTIME      = termij
        RO.ATYPE         = AType
        
        WRITE  RECO=1
        
    endif  ;exclude dummy zones and externals
    
ENDRUN




;print timestamp
RUN PGM=MATRIX
    
    ZONES = 1
    
    ScriptEndTime = currenttime()
    ScriptRunTime = ScriptEndTime - @ScriptStartTime@
    
    PRINT FILE='@ParentDir@@ScenarioDir@_Log\_RunTime - @RID@.txt',
        APPEND=T,
        LIST='\n    Urbanization                       ', formatdatetime(@ScriptStartTime@, 40, 0, 'yyyy-mm-dd,  hh:nn:ss'), 
                 ',  ', formatdatetime(ScriptRunTime, 40, 0, 'hhh:nn:ss')
    
ENDRUN




;System cleanup
    *(DEL 2_UrbanizationTermTime.txt)
