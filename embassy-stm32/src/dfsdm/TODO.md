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
* DFSDM_CHnCFGR1.CKABEN     [x] internal [X] pub [X] Config
* DFSDM_CHnCFGR1.SCDEN      [x] internal [X] pub [X] Config
* DFSDM_CHnCFGR2.OFFSET     [x] internal [X] pub [X] Config
* DFSDM_CHnAWSCDR.BKSCD     [x] internal [X] pub [X] Config   Research if we need to connect to official triggers.
* DFSDM_CHnAWSCDR.SCDT      [x] internal [X] pub [X] Config
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
* DFSDM_FLTxCR1.RDMAEN      [X] internal [X] pub    This probably needs a typestate....
* DFSDM_FLTxCR1.RSYNC       [X] internal [ ] pub
* DFSDM_FLTxCR1.JEXTEN      [X] internal [ ] pub
* DFSDM_FLTxCR1.JEXTSEL     [X] internal [ ] pub
* DFSDM_FLTxCR1.JDMAEN      [X] internal [X] pub    This probably needs a typestate....
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
* DFSDM_FLT0CR2.CKABIE  [X] internal [X] pub [ ] When Enabled (ONLY IN 0, GLOBAL)
* DFSDM_FLT0CR2.SCDIE   [X] internal [X] pub [ ] When Enabled (ONLY IN 0, GLOBAL)
* DFSDM_FLT0CR2.AWDIE   [X] internal [X] pub [ ] When Enabled

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
* DFSDM_FLTxAWHTR.AWHT         [X] internal [X] pub   [ ] Config
* DFSDM_FLTxAWHTR.BKAWH        [X] internal [X] pub   [ ] Config
* DFSDM_FLTxAWLTR.AWLT         [X] internal [X] pub   [ ] Config
* DFSDM_FLTxAWLTR.BKAWL        [X] internal [X] pub   [ ] Config
* DFSDM_FLTxEXMAX.EXMAX        [x] internal [x] pub   [ ] Config
* DFSDM_FLTxEXMAX.EXMAXCH      [x] internal [x] pub   [ ] Config
* DFSDM_FLTxEXMIN.EXMIN        [x] internal [x] pub   [ ] Config
* DFSDM_FLTxEXMIN.EXMINCH      [x] internal [x] pub   [ ] Config
* DFSDM_FLTxCNVTIMR.CNVCNT     [X] internal [X] pub

# Flags
## Filter
### Status
* DFSDM_FLT0ISR.SCDF  (ONLY IN 0, GLOBAL)
* DFSDM_FLT0ISR.CKABF  (ONLY IN 0, GLOBAL)
* DFSDM_FLT0ISR.RCIP            [x] internal [x] pub    
* DFSDM_FLT0ISR.JCIP            [x] internal [x] pub    
* DFSDM_FLT0ISR.REOCF           [x] internal [x] pub    Readable/Modifiable also when channel disabled, maybe read res after shutdown?
* DFSDM_FLT0ISR.JEOCF           [x] internal [x] pub    Readable/Modifiable also when channel disabled, maybe read res after shutdown?
* DFSDM_FLT0ISR.AWDF            [X] internal [X] pub
* DFSDM_FLT0ISR.ROVRF           [ ] internal [ ] pub
* DFSDM_FLT0ISR.JOVRF           [ ] internal [ ] pub
* DFSDM_FLTxAWSR.AWHTF          [X] internal [X] pub
* DFSDM_FLTxAWSR.AWLTF          [X] internal [X] pub

### Clear
## Filter
* DFSDM_FLT0ISR.CLRROVRF    [ ] internal [ ] pub    
* DFSDM_FLT0ISR.CLRJOVRF    [ ] internal [ ] pub    
* DFSDM_FLT0ISR.CLRSCDF     [X]
* DFSDM_FLT0ISR.CLRCKABF    [X]
* DFSDM_FLTxAWCFR.CLRAWHTF  [X]   
* DFSDM_FLTxAWCFR.CLRAWLTF  [X]


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
* The enable semantics should really also be linked to channel assignments in filters?
* MAYBE REMOVE ONLINE RECONFIG AND ONLY USE ACCESSORS
* split filters into inj/reg/common, watchdog?
* DFSDMCOMMON has SC and CA as seperate struct items?
* Discuss u32 vs i32 and add signextrension metods publicly?
   

# Interrupts:
## AnalogWatchdog
* Enable: DFSDM_FLTyCR2.AWDIE
* Flag: DFSDM_FLTyISR.AWDF, DFSDM_FLTxAWSR.CLRAWHTF, DFSDM_FLTxAWSR.CLRAWLTF
* Clear: DFSDM_FLTxAWCFR.CLRAWHTF, DFSDM_FLTxAWCFR.CLRAWLTF

## ShortcircuitDetector
* Enable: DFSDM_FLT0CR2.SCDIE
* Flag: DFSDM_FLT0ISR.SCDF
* Clear: DFSDM_FLT0ICR.CLRSCDF

ClockAbsenceDetector
* Enable: DFSDM_FLT0CR2.CKABIE
* Flag: DFSDM_FLT0ISR.CKABF
* Clear: DFSDM_FLT0ICR.CLRCKABF


Overrun:

* Enable: DFSDM_FLTyCR2.ROVRIE
* Flag: DFSDM_FLTyISR.ROVRF

* Enable: DFSDM_FLTyCR2.JOVRIE
* Flag: DFSDM_FLTyISR.JOVRF

``` rust
use static_cell::StaticCell;

static SURV: StaticCell<Surveillance<'static>> = StaticCell::new();

// in main/init:
let surveillance = Surveillance::new(/* ... */);
let surveillance: &'static mut Surveillance<'static> = SURV.init(surveillance);

// now split by move, each half independently 'static-ownable
let Surveillance { scd, ckab } = surveillance; // or a real .split() consuming self

spawner.spawn(scd_task(scd)).unwrap();
spawner.spawn(ckab_task(ckab)).unwrap();

let (reg, inj, mut watchdog, common) = Filter::new(Irqs).split();
```


Filter-Functions:
* common
  * enable
  * configure
  * disable
  * reassign_regular_transceiver
  * reassign_injected_transceivers
  * 
* watchdog
  * wait_for_event
  * set_high_threshold
  * set_low_threshold
  * assign_high_to_break_signals
  * assign_low_to_break_signals
  * assign_transceivers
* regular
  * start_regular_conversion
  * read_regular -> read_regular_it
  * try_get_regular_result
  * get_regular_result_unchecked
  * is_end_of_regular_conversion
  * regular_conversion_in_progress
* injected
  * start_injected_conversion
  * read_injected -> read_injected_it
  * try_get_injected_result
  * try_get_injected_result_unchecked
  * is_end_of_injected_conversion
  * injected_conversion_in_progress

Dfsdm:
* common
  * enable
  * disable
  * 
* monitoring
  * SCD
    * wait_for_event
  * CAD
    * wait_for_event


Features
[X] DFSDMEN:  Enable DFSMD
[X] CHEN: enable transceiver
[X] DFEN: filter enable
[X] CKOUTSRC: CKOUT source
[X] CKOUTDIV[7:0]: ckout divider
[X] DATPACK[1:0]: packing mode
[X] DATMPX[1:0]: parallel data source
[X] CHINSEL: neighbor/own pins
[X] CKABEN: enable CAD
[X] SCDEN: enable SCD
[X] SPICKSEL[1:0]: spi clock select
[X] SITP[1:0]: serial type
[X] OFFSET[23:0]: offset
[X] DTRBS[4:0]: right shift
[X] AWFORD[1:0]: watchog filter order
[X] AWFOSR[4:0]: OSR 
[X] BKSCD[3:0]: SCD break assignment
[X] SCDT[7:0]: SCD threshold
[X] WDATA[15:0]: watchdog data R/O
[X] INDAT1[15:0]: Input data
[X] INDAT0[15:0]: Input data
[X] PLSSKP[5:0]: skip n pulses
[X] AWFSEL: AW fastmode
[X] FAST: fastmode
[X] RCH[2:0]: regular channel selection
[X] RDMAEN: DMA REG enable
[X] JDMAEN: DMA INJ enable
[X] RSYNC: sync reg to ch0
[X] RCONT: continuous reg
[X] RSWSTART: start manually reg
[X] JSWSTART: start manually inj
[X] JEXTEN[1:0]: injected trigger edge/enable
[X] JEXTSEL[4:0]: trigger selection
[X] JSCAN: scnaning injected
[X] JSYNC: sync to ch0
[X] AWDCH[7:0]: AW channel sel
[X] EXCH[7:0]: extremes channel detector
[X] CKABIE: Clock absence interrupt enable
[X] SCDIE: Short-circuit detector interrupt enable
[X] AWDIE: Analog watchdog interrupt enable
[X] REOCIE: Regular end of conversion interrupt enable
[X] JEOCIE: Injected end of conversion interrupt enable
[X] SCDF[7:0]: short-circuit detector flag
[X] CKABF[7:0]: Clock absence flag
[X] RCIP: Regular conversion in progress status
[X] JCIP: Injected conversion in progress status
[X] AWDF: Analog watchdog event occured
[X] REOCF: End of regular conversion flag
[X] JEOCF: End of injected conversion flag
[X] CLRSCDF[7:0]: Clear the short-circuit detector flag
[X] CLRCKABF[7:0]: Clear the clock absence flag
[X] JCHG[7:0]: Injected channel group selection
[X] FORD[2:0]: Sinc filter order
[X] FOSR[9:0]: Sinc filter oversampling ratio (decimation rate)
[X] IOSR[7:0]: Integrator oversampling ratio (averaging length)
[X] JDATA[23:0]: Injected group conversion data
[X] JDATACH[2:0]: Injected channel most recently converted
[X] RDATA[23:0]: Regular channel conversion data
[X] RPEND: Regular channel pending data
[X] RDATACH[2:0]: Regular channel most recently converted
[X] AWHT[23:0]: Analog watchdog high threshold
[X] BKAWH[3:0]: Break signal assignment to analog watchdog high threshold event
[X] AWLT[23:0]: Analog watchdog low threshold
[X] BKAWL[3:0]: Break signal assignment to analog watchdog low threshold event
[X] AWHTF[7:0]: Analog watchdog high threshold flag
[X] AWLTF[7:0]: Analog watchdog low threshold flag
[X] CLRAWHTF[7:0]: Clear the analog watchdog high threshold flag
[X] CLRAWLTF[7:0]: Clear the analog watchdog low threshold flag
[X] EXMAX[23:0]: Extremes detector maximum value
[X] EXMAXCH[2:0]: Extremes detector maximum data channel
[X] EXMIN[23:0]: Extremes detector minimum value
[X] EXMINCH[2:0]: Extremes detector minimum data channel
[X] CNVCNT[27:0]: 28-bit timer counting conversion time t = CNVCNT[27:0] / fDFSDMCLK
[ ] ROVRIE: Regular data overrun interrupt enable
[ ] JOVRIE: Injected data overrun interrupt enable
[ ] ROVRF: Regular conversion overrun flag
[ ] JOVRF: Injected conversion overrun flag
[ ] CLRROVRF: Clear the regular conversion overrun flag
[ ] CLRJOVRF: Clear the injected conversion overrun flag