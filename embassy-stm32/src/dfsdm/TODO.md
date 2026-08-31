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
* DFSDM_CHnDLYR.PLSSKP      [x] internal [X] pub [X] When Enabled   Not in config, implemented as method.

## TIMERS
* Implement set_break_dfsdm_enable, set_break2_dfsdm_enable reference: set_break_comparator_enable
* TIM1_AF1
* BKDF1BK0E
* BK2DF1BK1E
* BKDF1BK2E
* BK2DF1BK3E

## Filter config
### Filter off
* DFSDM_FLTxCR1.RDMAEN      [X] internal [ ] pub    This probably needs a typestate....
* DFSDM_FLTxCR1.RSYNC       [X] internal [ ] pub
* DFSDM_FLTxCR1.JEXTEN      [X] internal [ ] pub
* DFSDM_FLTxCR1.JEXTSEL     [X] internal [ ] pub
* DFSDM_FLTxCR1.JDMAEN      [X] internal [ ] pub
* DFSDM_FLTxCR1.JSYNC       [X] internal [ ] pub
* DFSDM_FLTxFCR.FORD        [X] internal [X] pub
* DFSDM_FLTxFCR.FOSR        [X] internal [X] pub
* DFSDM_FLTxFCR.IOSR        [X] internal [X] pub

### Filter on
* DFSDM_FLTxCR1.JSWSTART    [x] internal [x] pub   (only really relevant when on tho)  Not in config, implemented as method.
* DFSDM_FLTxCR1.RSWSTART    [x] internal [x] pub (only really relevant when on tho)  Not in config, implemented as method.
* DFSDM_FLTxCR1.AWFSEL  [X] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR1.FAST    [X] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR1.RCH     [X] internal [X] pub [ ] When Enabled
* DFSDM_FLTxCR1.RCONT   [X] internal [ ] pub [ ] When Enabled   
* DFSDM_FLTxCR1.JSCAN   [X] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR2.AWDCH   [X] internal [X] pub [ ] When Enabled
* DFSDM_FLTxCR2.EXCH    [X] internal [X] pub [ ] When Enabled
* DFSDM_FLTxCR2.ROVRIE  [X] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR2.JOVRIE  [X] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR2.REOCIE  [X] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxCR2.JEOCIE  [X] internal [ ] pub [ ] When Enabled
* DFSDM_FLTxJCHGR.JCHG  [X] internal [X] pub [ ] When Enabled
* DFSDM_FLT0CR2.CKABIE  [X] internal [ ] pub [ ] When Enabled (ONLY IN 0, GLOBAL)
* DFSDM_FLT0CR2.SCDIE   [X] internal [ ] pub [ ] When Enabled (ONLY IN 0, GLOBAL)
* DFSDM_FLT0CR2.AWDIE   [X] internal [ ] pub [ ] When Enabled (ONLY IN 0, GLOBAL)

NOTE:
Due to interrupts we should really put the IRQ requirement into the Common? And then the filters? global vs local interrupts etc idk



# Input data
## Inputchannel
* DFSDM_CHnDATINR.INDAT0    [x] internal [x] pub    [ ] dma     
* DFSDM_CHnDATINR.INDAT1    [x] internal [x] pub    [ ] dma
* Typemark interleaved/standard and even dual for proper DMA call restriction. [ ] TODO
  
# Output data
## Inputchannel
### Watchdog
* DFSDM_CHnWDATR.WDATA

## Filter
* DFSDM_FLTxJDATAR.JDATA       [x] internal [x] pub     Readable/Modifiable also when channel disabled, maybe read res after shutdown?
* DFSDM_FLTxJDATAR.JDATACH     [x] internal [x] pub     Readable/Modifiable also when channel disabled, maybe read res after shutdown?
* DFSDM_FLTxRDATAR.RDATA       [x] internal [x] pub     Readable/Modifiable also when channel disabled, maybe read res after shutdown?
* DFSDM_FLTxRDATAR.RDATACH     [x] internal [x] pub     Readable/Modifiable also when channel disabled, maybe read res after shutdown?
* DFSDM_FLTxRDATAR.RPEND       [x] internal [x] pub     Readable/Modifiable also when channel disabled, maybe read res after shutdown?
* DFSDM_FLTxAWHTR.AWHT         [ ] internal [ ] pub
* DFSDM_FLTxAWHTR.BKAWH        [ ] internal [ ] pub
* DFSDM_FLTxAWLTR.AWLT         [ ] internal [ ] pub
* DFSDM_FLTxAWLTR.BKAWL        [ ] internal [ ] pub
* DFSDM_FLTxEXMAX.EXMAX        [ ] internal [ ] pub
* DFSDM_FLTxEXMAX.EXMAXCH      [ ] internal [ ] pub
* DFSDM_FLTxEXMIN.EXMIN        [ ] internal [ ] pub
* DFSDM_FLTxEXMIN.EXMINCH      [ ] internal [ ] pub
* DFSDM_FLTxCNVTIMR.CNVCNT     [ ] internal [ ] pub

# Flags
## Filter
### Status
* DFSDM_FLT0ISR.SCDF  (ONLY IN 0, GLOBAL)
* DFSDM_FLT0ISR.CKABF  (ONLY IN 0, GLOBAL)
* DFSDM_FLT0ISR.RCIP            [x] internal [x] pub    
* DFSDM_FLT0ISR.JCIP            [x] internal [x] pub    
* DFSDM_FLT0ISR.AWDF            [ ] internal [ ] pub
* DFSDM_FLT0ISR.ROVRF           [ ] internal [ ] pub
* DFSDM_FLT0ISR.JOVRF           [ ] internal [ ] pub
* DFSDM_FLT0ISR.REOCF           [x] internal [x] pub    Readable/Modifiable also when channel disabled, maybe read res after shutdown?
* DFSDM_FLT0ISR.JEOCF           [x] internal [x] pub    Readable/Modifiable also when channel disabled, maybe read res after shutdown?
* DFSDM_FLTxAWSR.AWHTF          [ ] internal [ ] pub
* DFSDM_FLTxAWSR.AWLTF          [ ] internal [ ] pub

### Clear
## Filter
* DFSDM_FLT0ISR.CLRSCDF     [ ] internal [ ] pub    (ONLY IN 0, GLOBAL)
* DFSDM_FLT0ISR.CLRCKABF    [ ] internal [ ] pub    (ONLY IN 0, GLOBAL)
* DFSDM_FLT0ISR.CLRROVRF    [ ] internal [ ] pub    
* DFSDM_FLT0ISR.CLRJOVRF    [ ] internal [ ] pub    
* DFSDM_FLTxAWCFR.CLRAWHTF  [ ] internal [ ] pub    
* DFSDM_FLTxAWCFR.CLRAWLTF  [ ] internal [ ] pub    


# General ToDos:
* Polling conversion            [X]
* Async polling conversion      [ ]
* Async interrupt conversion    [ ]
* Async dma conversion          [ ]
* ADC to DFSDM conversions?     [ ]
  * Also do the same with DMA for non-adc channels  [ ]
* Break signals                 [ ]
* Timer-triggering              [ ]     I tihnk this works as soon as we assign triggers properly, but who knows...
* Buttload of examples using combined PWM and other stuff   [ ]

# Notes
* When enabled vs pub: COnstructor/config vs accessor I guess?
* Handle overrun etc in normal handler? Or external, registerable?