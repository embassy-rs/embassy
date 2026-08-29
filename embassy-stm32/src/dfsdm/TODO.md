Ref: RM0455

[x] Config RCC
[x] (Config CKOUT)

## 33.4.3
[ ] Enable Peripheral                          [x] internal [X] pub MOVE INTO DFSDM OR SO, IT'S CURRENTLY IN COMMON
Set `DFSDMEN` in `DFSDM_CH0CFGR1`.
[X] Enable Transceiver (InputChannel)          [x] internal [X] pub
Set `CHEN` in `DFSDM_CHnCFGR1`.
[X] Enable Filter                              [X] internal [X] pub
Set `DFEN` in `DFSDM_FLTnCR1`.


# Registers sorted by category
## Module config
### Moule off
* DFSDM_CH0CFGR1.CKOUTSRC   [x] internal [X] pub
* DFSDM_CH0CFGR1.CKOUTDIV   [x] internal [X] pub

## Inputchannel config
### Inputchannel off
* DFSDM_CHnCFGR1.DATPACK    [x] internal [X] pub
* DFSDM_CHnCFGR1.DATMPX     [x] internal [X] pub
* DFSDM_CHnCFGR1.CHINSEL    [x] internal [X] pub
* DFSDM_CHnCFGR1.SPICKSEL   [x] internal [X] pub
* DFSDM_CHnCFGR1.SITP       [x] internal [X] pub
* DFSDM_CHnCFGR2.DTRBS      [x] internal [X] pub
* DFSDM_CHnAWSCDR.AWFORD    [x] internal [X] pub
* DFSDM_CHnAWSCDR.AWFOSR    [x] internal [X] pub

### Inputchannel on
* DFSDM_CHnCFGR1.CKABEN     [x] internal [X] pub [X] When Enabled
* DFSDM_CHnCFGR1.SCDEN      [x] internal [X] pub [X] When Enabled
* DFSDM_CHnCFGR2.OFFSET     [x] internal [X] pub [X] When Enabled
* DFSDM_CHnAWSCDR.BKSCD     [x] internal [X] pub [X] When Enabled   Research if we need to connect to official triggers.
* DFSDM_CHnAWSCDR.SCDT      [x] internal [X] pub [X] When Enabled
* DFSDM_CHnDLYR.PLSSKP      [x] internal [X] pub [X] When Enabled   Not in config, implement as method!

## Filter config
### Filter off
* DFSDM_FLTxCR1.RDMAEN      [ ] internal [ ] pub
* DFSDM_FLTxCR1.RSYNC       [ ] internal [ ] pub
* DFSDM_FLTxCR1.RSWSTART    [ ] internal [ ] pub (only really relevant when on tho) 
* DFSDM_FLTxCR1.JEXTEN      [ ] internal [ ] pub
* DFSDM_FLTxCR1.JEXTSEL     [ ] internal [ ] pub
* DFSDM_FLTxCR1.JDMAEN      [ ] internal [ ] pub
* DFSDM_FLTxCR1.JSYNC       [ ] internal [ ] pub
* DFSDM_FLTxCR1.JSWSTART    [ ] internal [ ] pub   (only really relevant when on tho) 
* DFSDM_FLTxFCR.FORD        [ ] internal [ ] pub
* DFSDM_FLTxFCR.FOSR        [ ] internal [ ] pub
* DFSDM_FLTxFCR.IOSR        [ ] internal [ ] pub

### Filter on
* DFSDM_FLTxCR1.AWFSEL  [ ] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR1.FAST    [ ] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR1.RCH     [ ] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR1.RCONT   [ ] internal [ ] pub [ ] When Enabled   
* DFSDM_FLTxCR1.JSCAN   [ ] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR2.AWDCH   [ ] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR2.EXCH    [ ] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR2.ROVRIE  [ ] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR2.JOVRIE  [ ] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR2.REOCIE  [ ] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR2.JEOCIE  [ ] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxJCHGR.JCHG  [ ] internal [ ] pub [ ] When Enabled
* DFSDM_FLT0CR2.CKABIE  [ ] internal [ ] pub [ ] When Enabled (ONLY IN 0, GLOBAL)
* DFSDM_FLT0CR2.SCDIE   [ ] internal [ ] pub [ ] When Enabled (ONLY IN 0, GLOBAL)
* DFSDM_FLT0CR2.AWDIE   [ ] internal [ ] pub [ ] When Enabled (ONLY IN 0, GLOBAL)




# Input data
## Inputchannel
* DFSDM_CHnDATINR.INDAT0
* DFSDM_CHnDATINR.INDAT1
  
# Output data
## Inputchannel
### Watchdog
* DFSDM_CHnWDATR.WDATA

## Filter
* DFSDM_FLTxJDATAR.JDATA
* DFSDM_FLTxJDATAR.JDATACH
* DFSDM_FLTxRDATAR.RDATA
* DFSDM_FLTxRDATAR.RDATACH
* DFSDM_FLTxRDATAR.RPEND
* DFSDM_FLTxAWHTR.AWHT
* DFSDM_FLTxAWHTR.BKAWH
* DFSDM_FLTxAWLTR.AWLT
* DFSDM_FLTxAWLTR.BKAWL
* DFSDM_FLTxEXMAX.EXMAX
* DFSDM_FLTxEXMAX.EXMAXCH
* DFSDM_FLTxEXMIN.EXMIN
* DFSDM_FLTxEXMIN.EXMINCH
* DFSDM_FLTxCNVTIMR.CNVCNT

# Flags
## Filter
### Status
* DFSDM_FLT0ISR.SCDF  (ONLY IN 0, GLOBAL)
* DFSDM_FLT0ISR.CKABF  (ONLY IN 0, GLOBAL)
* DFSDM_FLT0ISR.RCIP
* DFSDM_FLT0ISR.JCIP
* DFSDM_FLT0ISR.AWDF
* DFSDM_FLT0ISR.ROVRF
* DFSDM_FLT0ISR.JOVRF
* DFSDM_FLT0ISR.REOCF
* DFSDM_FLT0ISR.JEOCF
* DFSDM_FLTxAWSR.AWHTF
* DFSDM_FLTxAWSR.AWLTF

### Clear
## Filter
* DFSDM_FLT0ISR.CLRSCDF  (ONLY IN 0, GLOBAL)
* DFSDM_FLT0ISR.CLRCKABF  (ONLY IN 0, GLOBAL)
* DFSDM_FLT0ISR.CLRROVRF
* DFSDM_FLT0ISR.CLRJOVRF
* DFSDM_FLTxAWCFR.CLRAWHTF
* DFSDM_FLTxAWCFR.CLRAWLTF