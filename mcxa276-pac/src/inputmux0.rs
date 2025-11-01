#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    _reserved0: [u8; 0x20],
    ctimer0cap: [Ctimer0cap; 4],
    timer0trig: Timer0trig,
    _reserved2: [u8; 0x0c],
    ctimer1cap: [Ctimer1cap; 4],
    timer1trig: Timer1trig,
    _reserved4: [u8; 0x0c],
    ctimer2cap: [Ctimer2cap; 4],
    timer2trig: Timer2trig,
    _reserved6: [u8; 0x2c],
    smart_dma_trig: [SmartDmaTrig; 8],
    _reserved7: [u8; 0xc0],
    freqmeas_ref: FreqmeasRef,
    freqmeas_tar: FreqmeasTar,
    _reserved9: [u8; 0x18],
    ctimer3cap: [Ctimer3cap; 4],
    timer3trig: Timer3trig,
    _reserved11: [u8; 0x0c],
    ctimer4cap: [Ctimer4cap; 4],
    timer4trig: Timer4trig,
    _reserved13: [u8; 0x2c],
    aoi1_input: [Aoi1Input; 16],
    _reserved14: [u8; 0x20],
    cmp0_trig: Cmp0Trig,
    _reserved15: [u8; 0x1c],
    adc0_trig: [Adc0Trig; 4],
    _reserved16: [u8; 0x10],
    adc2_trig: [Adc2Trig; 4],
    _reserved17: [u8; 0x10],
    adc1_trig: [Adc1Trig; 4],
    _reserved18: [u8; 0x10],
    adc3_trig: [Adc3Trig; 4],
    _reserved19: [u8; 0x10],
    dac0_trig: Dac0Trig,
    _reserved20: [u8; 0x5c],
    qdc0_trig: Qdc0Trig,
    qdc0_home: Qdc0Home,
    qdc0_index: Qdc0Index,
    qdc0_phaseb: Qdc0Phaseb,
    qdc0_phasea: Qdc0Phasea,
    qdc0_icap1: Qdc0Icap1,
    qdc0_icap2: Qdc0Icap2,
    qdc0_icap3: Qdc0Icap3,
    qdc1_trig: Qdc1Trig,
    qdc1_home: Qdc1Home,
    qdc1_index: Qdc1Index,
    qdc1_phaseb: Qdc1Phaseb,
    qdc1_phasea: Qdc1Phasea,
    qdc1_icap1: Qdc1Icap1,
    qdc1_icap2: Qdc1Icap2,
    qdc1_icap3: Qdc1Icap3,
    flex_pwm0_sm0_exta0: FlexPwm0Sm0Exta0,
    flex_pwm0_sm0_extsync: FlexPwm0Sm0Extsync,
    flex_pwm0_sm1_exta: FlexPwm0Sm1Exta,
    flex_pwm0_sm1_extsync: FlexPwm0Sm1Extsync,
    flex_pwm0_sm2_exta: FlexPwm0Sm2Exta,
    flex_pwm0_sm2_extsync: FlexPwm0Sm2Extsync,
    flex_pwm0_sm3_exta0: FlexPwm0Sm3Exta0,
    flex_pwm0_sm3_extsync: FlexPwm0Sm3Extsync,
    flex_pwm0_fault: [FlexPwm0Fault; 4],
    flex_pwm0_force: FlexPwm0Force,
    _reserved46: [u8; 0x0c],
    flex_pwm1_sm0_exta0: FlexPwm1Sm0Exta0,
    flex_pwm1_sm0_extsync: FlexPwm1Sm0Extsync,
    flex_pwm1_sm1_exta: FlexPwm1Sm1Exta,
    flex_pwm1_sm1_extsync: FlexPwm1Sm1Extsync,
    flex_pwm1_sm2_exta: FlexPwm1Sm2Exta,
    flex_pwm1_sm2_extsync: FlexPwm1Sm2Extsync,
    flex_pwm1_sm3_exta0: FlexPwm1Sm3Exta0,
    flex_pwm1_sm3_extsync: FlexPwm1Sm3Extsync,
    flex_pwm1_fault: [FlexPwm1Fault; 4],
    flex_pwm1_force: FlexPwm1Force,
    _reserved56: [u8; 0x0c],
    pwm0_ext_clk: Pwm0ExtClk,
    pwm1_ext_clk: Pwm1ExtClk,
    _reserved58: [u8; 0x18],
    aoi0_input: [Aoi0Input; 16],
    usbfs_trig: UsbfsTrig,
    _reserved60: [u8; 0x3c],
    ext_trig: [ExtTrig; 8],
    cmp1_trig: Cmp1Trig,
    _reserved62: [u8; 0x1c],
    cmp2_trig: Cmp2Trig,
    _reserved63: [u8; 0x3c],
    lpi2c2_trig: Lpi2c2Trig,
    _reserved64: [u8; 0x1c],
    lpi2c3_trig: Lpi2c3Trig,
    _reserved65: [u8; 0x3c],
    lpi2c0_trig: Lpi2c0Trig,
    _reserved66: [u8; 0x1c],
    lpi2c1_trig: Lpi2c1Trig,
    _reserved67: [u8; 0x1c],
    lpspi0_trig: Lpspi0Trig,
    _reserved68: [u8; 0x1c],
    lpspi1_trig: Lpspi1Trig,
    _reserved69: [u8; 0x1c],
    lpuart0: Lpuart0,
    _reserved70: [u8; 0x1c],
    lpuart1: Lpuart1,
    _reserved71: [u8; 0x1c],
    lpuart2: Lpuart2,
    _reserved72: [u8; 0x1c],
    lpuart3: Lpuart3,
    _reserved73: [u8; 0x1c],
    lpuart4: Lpuart4,
    _reserved74: [u8; 0x1c],
    lpuart5: Lpuart5,
    _reserved75: [u8; 0x1c],
    flexio_trig: [FlexioTrig; 4],
    _reserved76: [u8; 0x0310],
    trigfil_prsc: TrigfilPrsc,
    trigfil_stat0: TrigfilStat0,
    _reserved78: [u8; 0x08],
    trigfil: [Trigfil; 12],
}
impl RegisterBlock {
    #[doc = "0x20..0x30 - Capture select register for CTIMER inputs"]
    #[inline(always)]
    pub const fn ctimer0cap(&self, n: usize) -> &Ctimer0cap {
        &self.ctimer0cap[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x20..0x30 - Capture select register for CTIMER inputs"]
    #[inline(always)]
    pub fn ctimer0cap_iter(&self) -> impl Iterator<Item = &Ctimer0cap> {
        self.ctimer0cap.iter()
    }
    #[doc = "0x30 - Trigger register for TIMER0"]
    #[inline(always)]
    pub const fn timer0trig(&self) -> &Timer0trig {
        &self.timer0trig
    }
    #[doc = "0x40..0x50 - Capture select register for CTIMER inputs"]
    #[inline(always)]
    pub const fn ctimer1cap(&self, n: usize) -> &Ctimer1cap {
        &self.ctimer1cap[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x40..0x50 - Capture select register for CTIMER inputs"]
    #[inline(always)]
    pub fn ctimer1cap_iter(&self) -> impl Iterator<Item = &Ctimer1cap> {
        self.ctimer1cap.iter()
    }
    #[doc = "0x50 - Trigger register for TIMER1"]
    #[inline(always)]
    pub const fn timer1trig(&self) -> &Timer1trig {
        &self.timer1trig
    }
    #[doc = "0x60..0x70 - Capture select register for CTIMER inputs"]
    #[inline(always)]
    pub const fn ctimer2cap(&self, n: usize) -> &Ctimer2cap {
        &self.ctimer2cap[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x60..0x70 - Capture select register for CTIMER inputs"]
    #[inline(always)]
    pub fn ctimer2cap_iter(&self) -> impl Iterator<Item = &Ctimer2cap> {
        self.ctimer2cap.iter()
    }
    #[doc = "0x70 - Trigger register for TIMER2 inputs"]
    #[inline(always)]
    pub const fn timer2trig(&self) -> &Timer2trig {
        &self.timer2trig
    }
    #[doc = "0xa0..0xc0 - SmartDMA Trigger Input Connections"]
    #[inline(always)]
    pub const fn smart_dma_trig(&self, n: usize) -> &SmartDmaTrig {
        &self.smart_dma_trig[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0xa0..0xc0 - SmartDMA Trigger Input Connections"]
    #[inline(always)]
    pub fn smart_dma_trig_iter(&self) -> impl Iterator<Item = &SmartDmaTrig> {
        self.smart_dma_trig.iter()
    }
    #[doc = "0x180 - Selection for frequency measurement reference clock"]
    #[inline(always)]
    pub const fn freqmeas_ref(&self) -> &FreqmeasRef {
        &self.freqmeas_ref
    }
    #[doc = "0x184 - Selection for frequency measurement reference clock"]
    #[inline(always)]
    pub const fn freqmeas_tar(&self) -> &FreqmeasTar {
        &self.freqmeas_tar
    }
    #[doc = "0x1a0..0x1b0 - Capture select register for CTIMER inputs"]
    #[inline(always)]
    pub const fn ctimer3cap(&self, n: usize) -> &Ctimer3cap {
        &self.ctimer3cap[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x1a0..0x1b0 - Capture select register for CTIMER inputs"]
    #[inline(always)]
    pub fn ctimer3cap_iter(&self) -> impl Iterator<Item = &Ctimer3cap> {
        self.ctimer3cap.iter()
    }
    #[doc = "0x1b0 - Trigger register for TIMER3"]
    #[inline(always)]
    pub const fn timer3trig(&self) -> &Timer3trig {
        &self.timer3trig
    }
    #[doc = "0x1c0..0x1d0 - Capture select register for CTIMER inputs"]
    #[inline(always)]
    pub const fn ctimer4cap(&self, n: usize) -> &Ctimer4cap {
        &self.ctimer4cap[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x1c0..0x1d0 - Capture select register for CTIMER inputs"]
    #[inline(always)]
    pub fn ctimer4cap_iter(&self) -> impl Iterator<Item = &Ctimer4cap> {
        self.ctimer4cap.iter()
    }
    #[doc = "0x1d0 - Trigger register for TIMER4"]
    #[inline(always)]
    pub const fn timer4trig(&self) -> &Timer4trig {
        &self.timer4trig
    }
    #[doc = "0x200..0x240 - AOI1 trigger input connections 0"]
    #[inline(always)]
    pub const fn aoi1_input(&self, n: usize) -> &Aoi1Input {
        &self.aoi1_input[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x200..0x240 - AOI1 trigger input connections 0"]
    #[inline(always)]
    pub fn aoi1_input_iter(&self) -> impl Iterator<Item = &Aoi1Input> {
        self.aoi1_input.iter()
    }
    #[doc = "0x260 - CMP0 input connections"]
    #[inline(always)]
    pub const fn cmp0_trig(&self) -> &Cmp0Trig {
        &self.cmp0_trig
    }
    #[doc = "0x280..0x290 - ADC Trigger input connections"]
    #[inline(always)]
    pub const fn adc0_trig(&self, n: usize) -> &Adc0Trig {
        &self.adc0_trig[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x280..0x290 - ADC Trigger input connections"]
    #[inline(always)]
    pub fn adc0_trig_iter(&self) -> impl Iterator<Item = &Adc0Trig> {
        self.adc0_trig.iter()
    }
    #[doc = "0x2a0..0x2b0 - ADC Trigger input connections"]
    #[inline(always)]
    pub const fn adc2_trig(&self, n: usize) -> &Adc2Trig {
        &self.adc2_trig[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x2a0..0x2b0 - ADC Trigger input connections"]
    #[inline(always)]
    pub fn adc2_trig_iter(&self) -> impl Iterator<Item = &Adc2Trig> {
        self.adc2_trig.iter()
    }
    #[doc = "0x2c0..0x2d0 - ADC Trigger input connections"]
    #[inline(always)]
    pub const fn adc1_trig(&self, n: usize) -> &Adc1Trig {
        &self.adc1_trig[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x2c0..0x2d0 - ADC Trigger input connections"]
    #[inline(always)]
    pub fn adc1_trig_iter(&self) -> impl Iterator<Item = &Adc1Trig> {
        self.adc1_trig.iter()
    }
    #[doc = "0x2e0..0x2f0 - ADC Trigger input connections"]
    #[inline(always)]
    pub const fn adc3_trig(&self, n: usize) -> &Adc3Trig {
        &self.adc3_trig[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x2e0..0x2f0 - ADC Trigger input connections"]
    #[inline(always)]
    pub fn adc3_trig_iter(&self) -> impl Iterator<Item = &Adc3Trig> {
        self.adc3_trig.iter()
    }
    #[doc = "0x300 - DAC0 Trigger input connections."]
    #[inline(always)]
    pub const fn dac0_trig(&self) -> &Dac0Trig {
        &self.dac0_trig
    }
    #[doc = "0x360 - QDC0 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc0_trig(&self) -> &Qdc0Trig {
        &self.qdc0_trig
    }
    #[doc = "0x364 - QDC0 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc0_home(&self) -> &Qdc0Home {
        &self.qdc0_home
    }
    #[doc = "0x368 - QDC0 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc0_index(&self) -> &Qdc0Index {
        &self.qdc0_index
    }
    #[doc = "0x36c - QDC0 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc0_phaseb(&self) -> &Qdc0Phaseb {
        &self.qdc0_phaseb
    }
    #[doc = "0x370 - QDC0 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc0_phasea(&self) -> &Qdc0Phasea {
        &self.qdc0_phasea
    }
    #[doc = "0x374 - QDC0 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc0_icap1(&self) -> &Qdc0Icap1 {
        &self.qdc0_icap1
    }
    #[doc = "0x378 - QDC0 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc0_icap2(&self) -> &Qdc0Icap2 {
        &self.qdc0_icap2
    }
    #[doc = "0x37c - QDC0 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc0_icap3(&self) -> &Qdc0Icap3 {
        &self.qdc0_icap3
    }
    #[doc = "0x380 - QDC1 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc1_trig(&self) -> &Qdc1Trig {
        &self.qdc1_trig
    }
    #[doc = "0x384 - QDC1 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc1_home(&self) -> &Qdc1Home {
        &self.qdc1_home
    }
    #[doc = "0x388 - QDC1 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc1_index(&self) -> &Qdc1Index {
        &self.qdc1_index
    }
    #[doc = "0x38c - QDC1 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc1_phaseb(&self) -> &Qdc1Phaseb {
        &self.qdc1_phaseb
    }
    #[doc = "0x390 - QDC1 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc1_phasea(&self) -> &Qdc1Phasea {
        &self.qdc1_phasea
    }
    #[doc = "0x394 - QDC1 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc1_icap1(&self) -> &Qdc1Icap1 {
        &self.qdc1_icap1
    }
    #[doc = "0x398 - QDC1 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc1_icap2(&self) -> &Qdc1Icap2 {
        &self.qdc1_icap2
    }
    #[doc = "0x39c - QDC1 Trigger Input Connections"]
    #[inline(always)]
    pub const fn qdc1_icap3(&self) -> &Qdc1Icap3 {
        &self.qdc1_icap3
    }
    #[doc = "0x3a0 - PWM0 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm0_sm0_exta0(&self) -> &FlexPwm0Sm0Exta0 {
        &self.flex_pwm0_sm0_exta0
    }
    #[doc = "0x3a4 - PWM0 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm0_sm0_extsync(&self) -> &FlexPwm0Sm0Extsync {
        &self.flex_pwm0_sm0_extsync
    }
    #[doc = "0x3a8 - PWM0 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm0_sm1_exta(&self) -> &FlexPwm0Sm1Exta {
        &self.flex_pwm0_sm1_exta
    }
    #[doc = "0x3ac - PWM0 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm0_sm1_extsync(&self) -> &FlexPwm0Sm1Extsync {
        &self.flex_pwm0_sm1_extsync
    }
    #[doc = "0x3b0 - PWM0 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm0_sm2_exta(&self) -> &FlexPwm0Sm2Exta {
        &self.flex_pwm0_sm2_exta
    }
    #[doc = "0x3b4 - PWM0 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm0_sm2_extsync(&self) -> &FlexPwm0Sm2Extsync {
        &self.flex_pwm0_sm2_extsync
    }
    #[doc = "0x3b8 - PWM0 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm0_sm3_exta0(&self) -> &FlexPwm0Sm3Exta0 {
        &self.flex_pwm0_sm3_exta0
    }
    #[doc = "0x3bc - PWM0 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm0_sm3_extsync(&self) -> &FlexPwm0Sm3Extsync {
        &self.flex_pwm0_sm3_extsync
    }
    #[doc = "0x3c0..0x3d0 - PWM0 Fault Input Trigger Connections"]
    #[inline(always)]
    pub const fn flex_pwm0_fault(&self, n: usize) -> &FlexPwm0Fault {
        &self.flex_pwm0_fault[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x3c0..0x3d0 - PWM0 Fault Input Trigger Connections"]
    #[inline(always)]
    pub fn flex_pwm0_fault_iter(&self) -> impl Iterator<Item = &FlexPwm0Fault> {
        self.flex_pwm0_fault.iter()
    }
    #[doc = "0x3d0 - PWM0 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm0_force(&self) -> &FlexPwm0Force {
        &self.flex_pwm0_force
    }
    #[doc = "0x3e0 - PWM1 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm1_sm0_exta0(&self) -> &FlexPwm1Sm0Exta0 {
        &self.flex_pwm1_sm0_exta0
    }
    #[doc = "0x3e4 - PWM1 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm1_sm0_extsync(&self) -> &FlexPwm1Sm0Extsync {
        &self.flex_pwm1_sm0_extsync
    }
    #[doc = "0x3e8 - PWM1 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm1_sm1_exta(&self) -> &FlexPwm1Sm1Exta {
        &self.flex_pwm1_sm1_exta
    }
    #[doc = "0x3ec - PWM1 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm1_sm1_extsync(&self) -> &FlexPwm1Sm1Extsync {
        &self.flex_pwm1_sm1_extsync
    }
    #[doc = "0x3f0 - PWM1 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm1_sm2_exta(&self) -> &FlexPwm1Sm2Exta {
        &self.flex_pwm1_sm2_exta
    }
    #[doc = "0x3f4 - PWM1 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm1_sm2_extsync(&self) -> &FlexPwm1Sm2Extsync {
        &self.flex_pwm1_sm2_extsync
    }
    #[doc = "0x3f8 - PWM1 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm1_sm3_exta0(&self) -> &FlexPwm1Sm3Exta0 {
        &self.flex_pwm1_sm3_exta0
    }
    #[doc = "0x3fc - PWM1 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm1_sm3_extsync(&self) -> &FlexPwm1Sm3Extsync {
        &self.flex_pwm1_sm3_extsync
    }
    #[doc = "0x400..0x410 - PWM1 Fault Input Trigger Connections"]
    #[inline(always)]
    pub const fn flex_pwm1_fault(&self, n: usize) -> &FlexPwm1Fault {
        &self.flex_pwm1_fault[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x400..0x410 - PWM1 Fault Input Trigger Connections"]
    #[inline(always)]
    pub fn flex_pwm1_fault_iter(&self) -> impl Iterator<Item = &FlexPwm1Fault> {
        self.flex_pwm1_fault.iter()
    }
    #[doc = "0x410 - PWM1 input trigger connections"]
    #[inline(always)]
    pub const fn flex_pwm1_force(&self) -> &FlexPwm1Force {
        &self.flex_pwm1_force
    }
    #[doc = "0x420 - PWM0 external clock trigger"]
    #[inline(always)]
    pub const fn pwm0_ext_clk(&self) -> &Pwm0ExtClk {
        &self.pwm0_ext_clk
    }
    #[doc = "0x424 - PWM1 external clock trigger"]
    #[inline(always)]
    pub const fn pwm1_ext_clk(&self) -> &Pwm1ExtClk {
        &self.pwm1_ext_clk
    }
    #[doc = "0x440..0x480 - AOI0 trigger input connections 0"]
    #[inline(always)]
    pub const fn aoi0_input(&self, n: usize) -> &Aoi0Input {
        &self.aoi0_input[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x440..0x480 - AOI0 trigger input connections 0"]
    #[inline(always)]
    pub fn aoi0_input_iter(&self) -> impl Iterator<Item = &Aoi0Input> {
        self.aoi0_input.iter()
    }
    #[doc = "0x480 - USB-FS trigger input connections"]
    #[inline(always)]
    pub const fn usbfs_trig(&self) -> &UsbfsTrig {
        &self.usbfs_trig
    }
    #[doc = "0x4c0..0x4e0 - EXT trigger connections"]
    #[inline(always)]
    pub const fn ext_trig(&self, n: usize) -> &ExtTrig {
        &self.ext_trig[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x4c0..0x4e0 - EXT trigger connections"]
    #[inline(always)]
    pub fn ext_trig_iter(&self) -> impl Iterator<Item = &ExtTrig> {
        self.ext_trig.iter()
    }
    #[doc = "0x4e0 - CMP1 input connections"]
    #[inline(always)]
    pub const fn cmp1_trig(&self) -> &Cmp1Trig {
        &self.cmp1_trig
    }
    #[doc = "0x500 - CMP2 input connections"]
    #[inline(always)]
    pub const fn cmp2_trig(&self) -> &Cmp2Trig {
        &self.cmp2_trig
    }
    #[doc = "0x540 - LPI2C2 trigger input connections"]
    #[inline(always)]
    pub const fn lpi2c2_trig(&self) -> &Lpi2c2Trig {
        &self.lpi2c2_trig
    }
    #[doc = "0x560 - LPI2C3 trigger input connections"]
    #[inline(always)]
    pub const fn lpi2c3_trig(&self) -> &Lpi2c3Trig {
        &self.lpi2c3_trig
    }
    #[doc = "0x5a0 - LPI2C0 trigger input connections"]
    #[inline(always)]
    pub const fn lpi2c0_trig(&self) -> &Lpi2c0Trig {
        &self.lpi2c0_trig
    }
    #[doc = "0x5c0 - LPI2C1 trigger input connections"]
    #[inline(always)]
    pub const fn lpi2c1_trig(&self) -> &Lpi2c1Trig {
        &self.lpi2c1_trig
    }
    #[doc = "0x5e0 - LPSPI0 trigger input connections"]
    #[inline(always)]
    pub const fn lpspi0_trig(&self) -> &Lpspi0Trig {
        &self.lpspi0_trig
    }
    #[doc = "0x600 - LPSPI1 trigger input connections"]
    #[inline(always)]
    pub const fn lpspi1_trig(&self) -> &Lpspi1Trig {
        &self.lpspi1_trig
    }
    #[doc = "0x620 - LPUART0 trigger input connections"]
    #[inline(always)]
    pub const fn lpuart0(&self) -> &Lpuart0 {
        &self.lpuart0
    }
    #[doc = "0x640 - LPUART1 trigger input connections"]
    #[inline(always)]
    pub const fn lpuart1(&self) -> &Lpuart1 {
        &self.lpuart1
    }
    #[doc = "0x660 - LPUART2 trigger input connections"]
    #[inline(always)]
    pub const fn lpuart2(&self) -> &Lpuart2 {
        &self.lpuart2
    }
    #[doc = "0x680 - LPUART3 trigger input connections"]
    #[inline(always)]
    pub const fn lpuart3(&self) -> &Lpuart3 {
        &self.lpuart3
    }
    #[doc = "0x6a0 - LPUART4 trigger input connections"]
    #[inline(always)]
    pub const fn lpuart4(&self) -> &Lpuart4 {
        &self.lpuart4
    }
    #[doc = "0x6c0 - LPUART5 trigger input connections"]
    #[inline(always)]
    pub const fn lpuart5(&self) -> &Lpuart5 {
        &self.lpuart5
    }
    #[doc = "0x6e0..0x6f0 - FlexIO Trigger Input Connections"]
    #[inline(always)]
    pub const fn flexio_trig(&self, n: usize) -> &FlexioTrig {
        &self.flexio_trig[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x6e0..0x6f0 - FlexIO Trigger Input Connections"]
    #[inline(always)]
    pub fn flexio_trig_iter(&self) -> impl Iterator<Item = &FlexioTrig> {
        self.flexio_trig.iter()
    }
    #[doc = "0xa00 - Trigger filter prescaller"]
    #[inline(always)]
    pub const fn trigfil_prsc(&self) -> &TrigfilPrsc {
        &self.trigfil_prsc
    }
    #[doc = "0xa04 - Trigger filter stat"]
    #[inline(always)]
    pub const fn trigfil_stat0(&self) -> &TrigfilStat0 {
        &self.trigfil_stat0
    }
    #[doc = "0xa10..0xa40 - TRIGFIL control"]
    #[inline(always)]
    pub const fn trigfil(&self, n: usize) -> &Trigfil {
        &self.trigfil[n]
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0xa10..0xa40 - TRIGFIL control"]
    #[inline(always)]
    pub fn trigfil_iter(&self) -> impl Iterator<Item = &Trigfil> {
        self.trigfil.iter()
    }
}
#[doc = "CTIMER0CAP (rw) register accessor: Capture select register for CTIMER inputs\n\nYou can [`read`](crate::Reg::read) this register and get [`ctimer0cap::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctimer0cap::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctimer0cap`] module"]
#[doc(alias = "CTIMER0CAP")]
pub type Ctimer0cap = crate::Reg<ctimer0cap::Ctimer0capSpec>;
#[doc = "Capture select register for CTIMER inputs"]
pub mod ctimer0cap;
#[doc = "TIMER0TRIG (rw) register accessor: Trigger register for TIMER0\n\nYou can [`read`](crate::Reg::read) this register and get [`timer0trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timer0trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@timer0trig`] module"]
#[doc(alias = "TIMER0TRIG")]
pub type Timer0trig = crate::Reg<timer0trig::Timer0trigSpec>;
#[doc = "Trigger register for TIMER0"]
pub mod timer0trig;
#[doc = "CTIMER1CAP (rw) register accessor: Capture select register for CTIMER inputs\n\nYou can [`read`](crate::Reg::read) this register and get [`ctimer1cap::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctimer1cap::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctimer1cap`] module"]
#[doc(alias = "CTIMER1CAP")]
pub type Ctimer1cap = crate::Reg<ctimer1cap::Ctimer1capSpec>;
#[doc = "Capture select register for CTIMER inputs"]
pub mod ctimer1cap;
#[doc = "TIMER1TRIG (rw) register accessor: Trigger register for TIMER1\n\nYou can [`read`](crate::Reg::read) this register and get [`timer1trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timer1trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@timer1trig`] module"]
#[doc(alias = "TIMER1TRIG")]
pub type Timer1trig = crate::Reg<timer1trig::Timer1trigSpec>;
#[doc = "Trigger register for TIMER1"]
pub mod timer1trig;
#[doc = "CTIMER2CAP (rw) register accessor: Capture select register for CTIMER inputs\n\nYou can [`read`](crate::Reg::read) this register and get [`ctimer2cap::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctimer2cap::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctimer2cap`] module"]
#[doc(alias = "CTIMER2CAP")]
pub type Ctimer2cap = crate::Reg<ctimer2cap::Ctimer2capSpec>;
#[doc = "Capture select register for CTIMER inputs"]
pub mod ctimer2cap;
#[doc = "TIMER2TRIG (rw) register accessor: Trigger register for TIMER2 inputs\n\nYou can [`read`](crate::Reg::read) this register and get [`timer2trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timer2trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@timer2trig`] module"]
#[doc(alias = "TIMER2TRIG")]
pub type Timer2trig = crate::Reg<timer2trig::Timer2trigSpec>;
#[doc = "Trigger register for TIMER2 inputs"]
pub mod timer2trig;
#[doc = "SmartDMA_TRIG (rw) register accessor: SmartDMA Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`smart_dma_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`smart_dma_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@smart_dma_trig`] module"]
#[doc(alias = "SmartDMA_TRIG")]
pub type SmartDmaTrig = crate::Reg<smart_dma_trig::SmartDmaTrigSpec>;
#[doc = "SmartDMA Trigger Input Connections"]
pub mod smart_dma_trig;
#[doc = "FREQMEAS_REF (rw) register accessor: Selection for frequency measurement reference clock\n\nYou can [`read`](crate::Reg::read) this register and get [`freqmeas_ref::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`freqmeas_ref::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@freqmeas_ref`] module"]
#[doc(alias = "FREQMEAS_REF")]
pub type FreqmeasRef = crate::Reg<freqmeas_ref::FreqmeasRefSpec>;
#[doc = "Selection for frequency measurement reference clock"]
pub mod freqmeas_ref;
#[doc = "FREQMEAS_TAR (rw) register accessor: Selection for frequency measurement reference clock\n\nYou can [`read`](crate::Reg::read) this register and get [`freqmeas_tar::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`freqmeas_tar::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@freqmeas_tar`] module"]
#[doc(alias = "FREQMEAS_TAR")]
pub type FreqmeasTar = crate::Reg<freqmeas_tar::FreqmeasTarSpec>;
#[doc = "Selection for frequency measurement reference clock"]
pub mod freqmeas_tar;
#[doc = "CTIMER3CAP (rw) register accessor: Capture select register for CTIMER inputs\n\nYou can [`read`](crate::Reg::read) this register and get [`ctimer3cap::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctimer3cap::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctimer3cap`] module"]
#[doc(alias = "CTIMER3CAP")]
pub type Ctimer3cap = crate::Reg<ctimer3cap::Ctimer3capSpec>;
#[doc = "Capture select register for CTIMER inputs"]
pub mod ctimer3cap;
#[doc = "TIMER3TRIG (rw) register accessor: Trigger register for TIMER3\n\nYou can [`read`](crate::Reg::read) this register and get [`timer3trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timer3trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@timer3trig`] module"]
#[doc(alias = "TIMER3TRIG")]
pub type Timer3trig = crate::Reg<timer3trig::Timer3trigSpec>;
#[doc = "Trigger register for TIMER3"]
pub mod timer3trig;
#[doc = "CTIMER4CAP (rw) register accessor: Capture select register for CTIMER inputs\n\nYou can [`read`](crate::Reg::read) this register and get [`ctimer4cap::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctimer4cap::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctimer4cap`] module"]
#[doc(alias = "CTIMER4CAP")]
pub type Ctimer4cap = crate::Reg<ctimer4cap::Ctimer4capSpec>;
#[doc = "Capture select register for CTIMER inputs"]
pub mod ctimer4cap;
#[doc = "TIMER4TRIG (rw) register accessor: Trigger register for TIMER4\n\nYou can [`read`](crate::Reg::read) this register and get [`timer4trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timer4trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@timer4trig`] module"]
#[doc(alias = "TIMER4TRIG")]
pub type Timer4trig = crate::Reg<timer4trig::Timer4trigSpec>;
#[doc = "Trigger register for TIMER4"]
pub mod timer4trig;
#[doc = "AOI1_INPUT (rw) register accessor: AOI1 trigger input connections 0\n\nYou can [`read`](crate::Reg::read) this register and get [`aoi1_input::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`aoi1_input::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@aoi1_input`] module"]
#[doc(alias = "AOI1_INPUT")]
pub type Aoi1Input = crate::Reg<aoi1_input::Aoi1InputSpec>;
#[doc = "AOI1 trigger input connections 0"]
pub mod aoi1_input;
#[doc = "CMP0_TRIG (rw) register accessor: CMP0 input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`cmp0_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmp0_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmp0_trig`] module"]
#[doc(alias = "CMP0_TRIG")]
pub type Cmp0Trig = crate::Reg<cmp0_trig::Cmp0TrigSpec>;
#[doc = "CMP0 input connections"]
pub mod cmp0_trig;
#[doc = "ADC0_TRIG (rw) register accessor: ADC Trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`adc0_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`adc0_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@adc0_trig`] module"]
#[doc(alias = "ADC0_TRIG")]
pub type Adc0Trig = crate::Reg<adc0_trig::Adc0TrigSpec>;
#[doc = "ADC Trigger input connections"]
pub mod adc0_trig;
#[doc = "ADC2_TRIG (rw) register accessor: ADC Trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`adc2_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`adc2_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@adc2_trig`] module"]
#[doc(alias = "ADC2_TRIG")]
pub type Adc2Trig = crate::Reg<adc2_trig::Adc2TrigSpec>;
#[doc = "ADC Trigger input connections"]
pub mod adc2_trig;
#[doc = "ADC1_TRIG (rw) register accessor: ADC Trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`adc1_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`adc1_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@adc1_trig`] module"]
#[doc(alias = "ADC1_TRIG")]
pub type Adc1Trig = crate::Reg<adc1_trig::Adc1TrigSpec>;
#[doc = "ADC Trigger input connections"]
pub mod adc1_trig;
#[doc = "ADC3_TRIG (rw) register accessor: ADC Trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`adc3_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`adc3_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@adc3_trig`] module"]
#[doc(alias = "ADC3_TRIG")]
pub type Adc3Trig = crate::Reg<adc3_trig::Adc3TrigSpec>;
#[doc = "ADC Trigger input connections"]
pub mod adc3_trig;
#[doc = "DAC0_TRIG (rw) register accessor: DAC0 Trigger input connections.\n\nYou can [`read`](crate::Reg::read) this register and get [`dac0_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dac0_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@dac0_trig`] module"]
#[doc(alias = "DAC0_TRIG")]
pub type Dac0Trig = crate::Reg<dac0_trig::Dac0TrigSpec>;
#[doc = "DAC0 Trigger input connections."]
pub mod dac0_trig;
#[doc = "QDC0_TRIG (rw) register accessor: QDC0 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc0_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc0_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc0_trig`] module"]
#[doc(alias = "QDC0_TRIG")]
pub type Qdc0Trig = crate::Reg<qdc0_trig::Qdc0TrigSpec>;
#[doc = "QDC0 Trigger Input Connections"]
pub mod qdc0_trig;
#[doc = "QDC0_HOME (rw) register accessor: QDC0 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc0_home::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc0_home::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc0_home`] module"]
#[doc(alias = "QDC0_HOME")]
pub type Qdc0Home = crate::Reg<qdc0_home::Qdc0HomeSpec>;
#[doc = "QDC0 Trigger Input Connections"]
pub mod qdc0_home;
#[doc = "QDC0_INDEX (rw) register accessor: QDC0 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc0_index::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc0_index::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc0_index`] module"]
#[doc(alias = "QDC0_INDEX")]
pub type Qdc0Index = crate::Reg<qdc0_index::Qdc0IndexSpec>;
#[doc = "QDC0 Trigger Input Connections"]
pub mod qdc0_index;
#[doc = "QDC0_PHASEB (rw) register accessor: QDC0 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc0_phaseb::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc0_phaseb::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc0_phaseb`] module"]
#[doc(alias = "QDC0_PHASEB")]
pub type Qdc0Phaseb = crate::Reg<qdc0_phaseb::Qdc0PhasebSpec>;
#[doc = "QDC0 Trigger Input Connections"]
pub mod qdc0_phaseb;
#[doc = "QDC0_PHASEA (rw) register accessor: QDC0 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc0_phasea::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc0_phasea::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc0_phasea`] module"]
#[doc(alias = "QDC0_PHASEA")]
pub type Qdc0Phasea = crate::Reg<qdc0_phasea::Qdc0PhaseaSpec>;
#[doc = "QDC0 Trigger Input Connections"]
pub mod qdc0_phasea;
#[doc = "QDC0_ICAP1 (rw) register accessor: QDC0 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc0_icap1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc0_icap1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc0_icap1`] module"]
#[doc(alias = "QDC0_ICAP1")]
pub type Qdc0Icap1 = crate::Reg<qdc0_icap1::Qdc0Icap1Spec>;
#[doc = "QDC0 Trigger Input Connections"]
pub mod qdc0_icap1;
#[doc = "QDC0_ICAP2 (rw) register accessor: QDC0 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc0_icap2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc0_icap2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc0_icap2`] module"]
#[doc(alias = "QDC0_ICAP2")]
pub type Qdc0Icap2 = crate::Reg<qdc0_icap2::Qdc0Icap2Spec>;
#[doc = "QDC0 Trigger Input Connections"]
pub mod qdc0_icap2;
#[doc = "QDC0_ICAP3 (rw) register accessor: QDC0 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc0_icap3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc0_icap3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc0_icap3`] module"]
#[doc(alias = "QDC0_ICAP3")]
pub type Qdc0Icap3 = crate::Reg<qdc0_icap3::Qdc0Icap3Spec>;
#[doc = "QDC0 Trigger Input Connections"]
pub mod qdc0_icap3;
#[doc = "QDC1_TRIG (rw) register accessor: QDC1 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc1_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc1_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc1_trig`] module"]
#[doc(alias = "QDC1_TRIG")]
pub type Qdc1Trig = crate::Reg<qdc1_trig::Qdc1TrigSpec>;
#[doc = "QDC1 Trigger Input Connections"]
pub mod qdc1_trig;
#[doc = "QDC1_HOME (rw) register accessor: QDC1 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc1_home::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc1_home::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc1_home`] module"]
#[doc(alias = "QDC1_HOME")]
pub type Qdc1Home = crate::Reg<qdc1_home::Qdc1HomeSpec>;
#[doc = "QDC1 Trigger Input Connections"]
pub mod qdc1_home;
#[doc = "QDC1_INDEX (rw) register accessor: QDC1 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc1_index::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc1_index::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc1_index`] module"]
#[doc(alias = "QDC1_INDEX")]
pub type Qdc1Index = crate::Reg<qdc1_index::Qdc1IndexSpec>;
#[doc = "QDC1 Trigger Input Connections"]
pub mod qdc1_index;
#[doc = "QDC1_PHASEB (rw) register accessor: QDC1 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc1_phaseb::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc1_phaseb::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc1_phaseb`] module"]
#[doc(alias = "QDC1_PHASEB")]
pub type Qdc1Phaseb = crate::Reg<qdc1_phaseb::Qdc1PhasebSpec>;
#[doc = "QDC1 Trigger Input Connections"]
pub mod qdc1_phaseb;
#[doc = "QDC1_PHASEA (rw) register accessor: QDC1 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc1_phasea::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc1_phasea::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc1_phasea`] module"]
#[doc(alias = "QDC1_PHASEA")]
pub type Qdc1Phasea = crate::Reg<qdc1_phasea::Qdc1PhaseaSpec>;
#[doc = "QDC1 Trigger Input Connections"]
pub mod qdc1_phasea;
#[doc = "QDC1_ICAP1 (rw) register accessor: QDC1 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc1_icap1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc1_icap1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc1_icap1`] module"]
#[doc(alias = "QDC1_ICAP1")]
pub type Qdc1Icap1 = crate::Reg<qdc1_icap1::Qdc1Icap1Spec>;
#[doc = "QDC1 Trigger Input Connections"]
pub mod qdc1_icap1;
#[doc = "QDC1_ICAP2 (rw) register accessor: QDC1 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc1_icap2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc1_icap2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc1_icap2`] module"]
#[doc(alias = "QDC1_ICAP2")]
pub type Qdc1Icap2 = crate::Reg<qdc1_icap2::Qdc1Icap2Spec>;
#[doc = "QDC1 Trigger Input Connections"]
pub mod qdc1_icap2;
#[doc = "QDC1_ICAP3 (rw) register accessor: QDC1 Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`qdc1_icap3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`qdc1_icap3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@qdc1_icap3`] module"]
#[doc(alias = "QDC1_ICAP3")]
pub type Qdc1Icap3 = crate::Reg<qdc1_icap3::Qdc1Icap3Spec>;
#[doc = "QDC1 Trigger Input Connections"]
pub mod qdc1_icap3;
#[doc = "FlexPWM0_SM0_EXTA0 (rw) register accessor: PWM0 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm0_sm0_exta0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm0_sm0_exta0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm0_sm0_exta0`] module"]
#[doc(alias = "FlexPWM0_SM0_EXTA0")]
pub type FlexPwm0Sm0Exta0 = crate::Reg<flex_pwm0_sm0_exta0::FlexPwm0Sm0Exta0Spec>;
#[doc = "PWM0 input trigger connections"]
pub mod flex_pwm0_sm0_exta0;
#[doc = "FlexPWM0_SM0_EXTSYNC (rw) register accessor: PWM0 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm0_sm0_extsync::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm0_sm0_extsync::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm0_sm0_extsync`] module"]
#[doc(alias = "FlexPWM0_SM0_EXTSYNC")]
pub type FlexPwm0Sm0Extsync = crate::Reg<flex_pwm0_sm0_extsync::FlexPwm0Sm0ExtsyncSpec>;
#[doc = "PWM0 input trigger connections"]
pub mod flex_pwm0_sm0_extsync;
#[doc = "FlexPWM0_SM1_EXTA (rw) register accessor: PWM0 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm0_sm1_exta::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm0_sm1_exta::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm0_sm1_exta`] module"]
#[doc(alias = "FlexPWM0_SM1_EXTA")]
pub type FlexPwm0Sm1Exta = crate::Reg<flex_pwm0_sm1_exta::FlexPwm0Sm1ExtaSpec>;
#[doc = "PWM0 input trigger connections"]
pub mod flex_pwm0_sm1_exta;
#[doc = "FlexPWM0_SM1_EXTSYNC (rw) register accessor: PWM0 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm0_sm1_extsync::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm0_sm1_extsync::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm0_sm1_extsync`] module"]
#[doc(alias = "FlexPWM0_SM1_EXTSYNC")]
pub type FlexPwm0Sm1Extsync = crate::Reg<flex_pwm0_sm1_extsync::FlexPwm0Sm1ExtsyncSpec>;
#[doc = "PWM0 input trigger connections"]
pub mod flex_pwm0_sm1_extsync;
#[doc = "FlexPWM0_SM2_EXTA (rw) register accessor: PWM0 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm0_sm2_exta::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm0_sm2_exta::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm0_sm2_exta`] module"]
#[doc(alias = "FlexPWM0_SM2_EXTA")]
pub type FlexPwm0Sm2Exta = crate::Reg<flex_pwm0_sm2_exta::FlexPwm0Sm2ExtaSpec>;
#[doc = "PWM0 input trigger connections"]
pub mod flex_pwm0_sm2_exta;
#[doc = "FlexPWM0_SM2_EXTSYNC (rw) register accessor: PWM0 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm0_sm2_extsync::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm0_sm2_extsync::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm0_sm2_extsync`] module"]
#[doc(alias = "FlexPWM0_SM2_EXTSYNC")]
pub type FlexPwm0Sm2Extsync = crate::Reg<flex_pwm0_sm2_extsync::FlexPwm0Sm2ExtsyncSpec>;
#[doc = "PWM0 input trigger connections"]
pub mod flex_pwm0_sm2_extsync;
#[doc = "FlexPWM0_SM3_EXTA0 (rw) register accessor: PWM0 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm0_sm3_exta0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm0_sm3_exta0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm0_sm3_exta0`] module"]
#[doc(alias = "FlexPWM0_SM3_EXTA0")]
pub type FlexPwm0Sm3Exta0 = crate::Reg<flex_pwm0_sm3_exta0::FlexPwm0Sm3Exta0Spec>;
#[doc = "PWM0 input trigger connections"]
pub mod flex_pwm0_sm3_exta0;
#[doc = "FlexPWM0_SM3_EXTSYNC (rw) register accessor: PWM0 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm0_sm3_extsync::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm0_sm3_extsync::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm0_sm3_extsync`] module"]
#[doc(alias = "FlexPWM0_SM3_EXTSYNC")]
pub type FlexPwm0Sm3Extsync = crate::Reg<flex_pwm0_sm3_extsync::FlexPwm0Sm3ExtsyncSpec>;
#[doc = "PWM0 input trigger connections"]
pub mod flex_pwm0_sm3_extsync;
#[doc = "FlexPWM0_FAULT (rw) register accessor: PWM0 Fault Input Trigger Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm0_fault::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm0_fault::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm0_fault`] module"]
#[doc(alias = "FlexPWM0_FAULT")]
pub type FlexPwm0Fault = crate::Reg<flex_pwm0_fault::FlexPwm0FaultSpec>;
#[doc = "PWM0 Fault Input Trigger Connections"]
pub mod flex_pwm0_fault;
#[doc = "FlexPWM0_FORCE (rw) register accessor: PWM0 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm0_force::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm0_force::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm0_force`] module"]
#[doc(alias = "FlexPWM0_FORCE")]
pub type FlexPwm0Force = crate::Reg<flex_pwm0_force::FlexPwm0ForceSpec>;
#[doc = "PWM0 input trigger connections"]
pub mod flex_pwm0_force;
#[doc = "FlexPWM1_SM0_EXTA0 (rw) register accessor: PWM1 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm1_sm0_exta0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm1_sm0_exta0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm1_sm0_exta0`] module"]
#[doc(alias = "FlexPWM1_SM0_EXTA0")]
pub type FlexPwm1Sm0Exta0 = crate::Reg<flex_pwm1_sm0_exta0::FlexPwm1Sm0Exta0Spec>;
#[doc = "PWM1 input trigger connections"]
pub mod flex_pwm1_sm0_exta0;
#[doc = "FlexPWM1_SM0_EXTSYNC (rw) register accessor: PWM1 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm1_sm0_extsync::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm1_sm0_extsync::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm1_sm0_extsync`] module"]
#[doc(alias = "FlexPWM1_SM0_EXTSYNC")]
pub type FlexPwm1Sm0Extsync = crate::Reg<flex_pwm1_sm0_extsync::FlexPwm1Sm0ExtsyncSpec>;
#[doc = "PWM1 input trigger connections"]
pub mod flex_pwm1_sm0_extsync;
#[doc = "FlexPWM1_SM1_EXTA (rw) register accessor: PWM1 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm1_sm1_exta::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm1_sm1_exta::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm1_sm1_exta`] module"]
#[doc(alias = "FlexPWM1_SM1_EXTA")]
pub type FlexPwm1Sm1Exta = crate::Reg<flex_pwm1_sm1_exta::FlexPwm1Sm1ExtaSpec>;
#[doc = "PWM1 input trigger connections"]
pub mod flex_pwm1_sm1_exta;
#[doc = "FlexPWM1_SM1_EXTSYNC (rw) register accessor: PWM1 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm1_sm1_extsync::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm1_sm1_extsync::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm1_sm1_extsync`] module"]
#[doc(alias = "FlexPWM1_SM1_EXTSYNC")]
pub type FlexPwm1Sm1Extsync = crate::Reg<flex_pwm1_sm1_extsync::FlexPwm1Sm1ExtsyncSpec>;
#[doc = "PWM1 input trigger connections"]
pub mod flex_pwm1_sm1_extsync;
#[doc = "FlexPWM1_SM2_EXTA (rw) register accessor: PWM1 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm1_sm2_exta::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm1_sm2_exta::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm1_sm2_exta`] module"]
#[doc(alias = "FlexPWM1_SM2_EXTA")]
pub type FlexPwm1Sm2Exta = crate::Reg<flex_pwm1_sm2_exta::FlexPwm1Sm2ExtaSpec>;
#[doc = "PWM1 input trigger connections"]
pub mod flex_pwm1_sm2_exta;
#[doc = "FlexPWM1_SM2_EXTSYNC (rw) register accessor: PWM1 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm1_sm2_extsync::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm1_sm2_extsync::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm1_sm2_extsync`] module"]
#[doc(alias = "FlexPWM1_SM2_EXTSYNC")]
pub type FlexPwm1Sm2Extsync = crate::Reg<flex_pwm1_sm2_extsync::FlexPwm1Sm2ExtsyncSpec>;
#[doc = "PWM1 input trigger connections"]
pub mod flex_pwm1_sm2_extsync;
#[doc = "FlexPWM1_SM3_EXTA0 (rw) register accessor: PWM1 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm1_sm3_exta0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm1_sm3_exta0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm1_sm3_exta0`] module"]
#[doc(alias = "FlexPWM1_SM3_EXTA0")]
pub type FlexPwm1Sm3Exta0 = crate::Reg<flex_pwm1_sm3_exta0::FlexPwm1Sm3Exta0Spec>;
#[doc = "PWM1 input trigger connections"]
pub mod flex_pwm1_sm3_exta0;
#[doc = "FlexPWM1_SM3_EXTSYNC (rw) register accessor: PWM1 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm1_sm3_extsync::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm1_sm3_extsync::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm1_sm3_extsync`] module"]
#[doc(alias = "FlexPWM1_SM3_EXTSYNC")]
pub type FlexPwm1Sm3Extsync = crate::Reg<flex_pwm1_sm3_extsync::FlexPwm1Sm3ExtsyncSpec>;
#[doc = "PWM1 input trigger connections"]
pub mod flex_pwm1_sm3_extsync;
#[doc = "FlexPWM1_FAULT (rw) register accessor: PWM1 Fault Input Trigger Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm1_fault::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm1_fault::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm1_fault`] module"]
#[doc(alias = "FlexPWM1_FAULT")]
pub type FlexPwm1Fault = crate::Reg<flex_pwm1_fault::FlexPwm1FaultSpec>;
#[doc = "PWM1 Fault Input Trigger Connections"]
pub mod flex_pwm1_fault;
#[doc = "FlexPWM1_FORCE (rw) register accessor: PWM1 input trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flex_pwm1_force::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flex_pwm1_force::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flex_pwm1_force`] module"]
#[doc(alias = "FlexPWM1_FORCE")]
pub type FlexPwm1Force = crate::Reg<flex_pwm1_force::FlexPwm1ForceSpec>;
#[doc = "PWM1 input trigger connections"]
pub mod flex_pwm1_force;
#[doc = "PWM0_EXT_CLK (rw) register accessor: PWM0 external clock trigger\n\nYou can [`read`](crate::Reg::read) this register and get [`pwm0_ext_clk::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pwm0_ext_clk::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pwm0_ext_clk`] module"]
#[doc(alias = "PWM0_EXT_CLK")]
pub type Pwm0ExtClk = crate::Reg<pwm0_ext_clk::Pwm0ExtClkSpec>;
#[doc = "PWM0 external clock trigger"]
pub mod pwm0_ext_clk;
#[doc = "PWM1_EXT_CLK (rw) register accessor: PWM1 external clock trigger\n\nYou can [`read`](crate::Reg::read) this register and get [`pwm1_ext_clk::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pwm1_ext_clk::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pwm1_ext_clk`] module"]
#[doc(alias = "PWM1_EXT_CLK")]
pub type Pwm1ExtClk = crate::Reg<pwm1_ext_clk::Pwm1ExtClkSpec>;
#[doc = "PWM1 external clock trigger"]
pub mod pwm1_ext_clk;
#[doc = "AOI0_INPUT (rw) register accessor: AOI0 trigger input connections 0\n\nYou can [`read`](crate::Reg::read) this register and get [`aoi0_input::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`aoi0_input::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@aoi0_input`] module"]
#[doc(alias = "AOI0_INPUT")]
pub type Aoi0Input = crate::Reg<aoi0_input::Aoi0InputSpec>;
#[doc = "AOI0 trigger input connections 0"]
pub mod aoi0_input;
#[doc = "USBFS_TRIG (rw) register accessor: USB-FS trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`usbfs_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`usbfs_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@usbfs_trig`] module"]
#[doc(alias = "USBFS_TRIG")]
pub type UsbfsTrig = crate::Reg<usbfs_trig::UsbfsTrigSpec>;
#[doc = "USB-FS trigger input connections"]
pub mod usbfs_trig;
#[doc = "EXT_TRIG (rw) register accessor: EXT trigger connections\n\nYou can [`read`](crate::Reg::read) this register and get [`ext_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ext_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ext_trig`] module"]
#[doc(alias = "EXT_TRIG")]
pub type ExtTrig = crate::Reg<ext_trig::ExtTrigSpec>;
#[doc = "EXT trigger connections"]
pub mod ext_trig;
#[doc = "CMP1_TRIG (rw) register accessor: CMP1 input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`cmp1_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmp1_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmp1_trig`] module"]
#[doc(alias = "CMP1_TRIG")]
pub type Cmp1Trig = crate::Reg<cmp1_trig::Cmp1TrigSpec>;
#[doc = "CMP1 input connections"]
pub mod cmp1_trig;
#[doc = "CMP2_TRIG (rw) register accessor: CMP2 input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`cmp2_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmp2_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmp2_trig`] module"]
#[doc(alias = "CMP2_TRIG")]
pub type Cmp2Trig = crate::Reg<cmp2_trig::Cmp2TrigSpec>;
#[doc = "CMP2 input connections"]
pub mod cmp2_trig;
#[doc = "LPI2C2_TRIG (rw) register accessor: LPI2C2 trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`lpi2c2_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpi2c2_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpi2c2_trig`] module"]
#[doc(alias = "LPI2C2_TRIG")]
pub type Lpi2c2Trig = crate::Reg<lpi2c2_trig::Lpi2c2TrigSpec>;
#[doc = "LPI2C2 trigger input connections"]
pub mod lpi2c2_trig;
#[doc = "LPI2C3_TRIG (rw) register accessor: LPI2C3 trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`lpi2c3_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpi2c3_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpi2c3_trig`] module"]
#[doc(alias = "LPI2C3_TRIG")]
pub type Lpi2c3Trig = crate::Reg<lpi2c3_trig::Lpi2c3TrigSpec>;
#[doc = "LPI2C3 trigger input connections"]
pub mod lpi2c3_trig;
#[doc = "LPI2C0_TRIG (rw) register accessor: LPI2C0 trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`lpi2c0_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpi2c0_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpi2c0_trig`] module"]
#[doc(alias = "LPI2C0_TRIG")]
pub type Lpi2c0Trig = crate::Reg<lpi2c0_trig::Lpi2c0TrigSpec>;
#[doc = "LPI2C0 trigger input connections"]
pub mod lpi2c0_trig;
#[doc = "LPI2C1_TRIG (rw) register accessor: LPI2C1 trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`lpi2c1_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpi2c1_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpi2c1_trig`] module"]
#[doc(alias = "LPI2C1_TRIG")]
pub type Lpi2c1Trig = crate::Reg<lpi2c1_trig::Lpi2c1TrigSpec>;
#[doc = "LPI2C1 trigger input connections"]
pub mod lpi2c1_trig;
#[doc = "LPSPI0_TRIG (rw) register accessor: LPSPI0 trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`lpspi0_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpspi0_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpspi0_trig`] module"]
#[doc(alias = "LPSPI0_TRIG")]
pub type Lpspi0Trig = crate::Reg<lpspi0_trig::Lpspi0TrigSpec>;
#[doc = "LPSPI0 trigger input connections"]
pub mod lpspi0_trig;
#[doc = "LPSPI1_TRIG (rw) register accessor: LPSPI1 trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`lpspi1_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpspi1_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpspi1_trig`] module"]
#[doc(alias = "LPSPI1_TRIG")]
pub type Lpspi1Trig = crate::Reg<lpspi1_trig::Lpspi1TrigSpec>;
#[doc = "LPSPI1 trigger input connections"]
pub mod lpspi1_trig;
#[doc = "LPUART0 (rw) register accessor: LPUART0 trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`lpuart0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpuart0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpuart0`] module"]
#[doc(alias = "LPUART0")]
pub type Lpuart0 = crate::Reg<lpuart0::Lpuart0Spec>;
#[doc = "LPUART0 trigger input connections"]
pub mod lpuart0;
#[doc = "LPUART1 (rw) register accessor: LPUART1 trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`lpuart1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpuart1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpuart1`] module"]
#[doc(alias = "LPUART1")]
pub type Lpuart1 = crate::Reg<lpuart1::Lpuart1Spec>;
#[doc = "LPUART1 trigger input connections"]
pub mod lpuart1;
#[doc = "LPUART2 (rw) register accessor: LPUART2 trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`lpuart2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpuart2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpuart2`] module"]
#[doc(alias = "LPUART2")]
pub type Lpuart2 = crate::Reg<lpuart2::Lpuart2Spec>;
#[doc = "LPUART2 trigger input connections"]
pub mod lpuart2;
#[doc = "LPUART3 (rw) register accessor: LPUART3 trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`lpuart3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpuart3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpuart3`] module"]
#[doc(alias = "LPUART3")]
pub type Lpuart3 = crate::Reg<lpuart3::Lpuart3Spec>;
#[doc = "LPUART3 trigger input connections"]
pub mod lpuart3;
#[doc = "LPUART4 (rw) register accessor: LPUART4 trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`lpuart4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpuart4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpuart4`] module"]
#[doc(alias = "LPUART4")]
pub type Lpuart4 = crate::Reg<lpuart4::Lpuart4Spec>;
#[doc = "LPUART4 trigger input connections"]
pub mod lpuart4;
#[doc = "LPUART5 (rw) register accessor: LPUART5 trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`lpuart5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpuart5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lpuart5`] module"]
#[doc(alias = "LPUART5")]
pub type Lpuart5 = crate::Reg<lpuart5::Lpuart5Spec>;
#[doc = "LPUART5 trigger input connections"]
pub mod lpuart5;
#[doc = "FLEXIO_TRIG (rw) register accessor: FlexIO Trigger Input Connections\n\nYou can [`read`](crate::Reg::read) this register and get [`flexio_trig::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flexio_trig::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@flexio_trig`] module"]
#[doc(alias = "FLEXIO_TRIG")]
pub type FlexioTrig = crate::Reg<flexio_trig::FlexioTrigSpec>;
#[doc = "FlexIO Trigger Input Connections"]
pub mod flexio_trig;
#[doc = "TRIGFIL_PRSC (rw) register accessor: Trigger filter prescaller\n\nYou can [`read`](crate::Reg::read) this register and get [`trigfil_prsc::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`trigfil_prsc::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@trigfil_prsc`] module"]
#[doc(alias = "TRIGFIL_PRSC")]
pub type TrigfilPrsc = crate::Reg<trigfil_prsc::TrigfilPrscSpec>;
#[doc = "Trigger filter prescaller"]
pub mod trigfil_prsc;
#[doc = "TRIGFIL_STAT0 (r) register accessor: Trigger filter stat\n\nYou can [`read`](crate::Reg::read) this register and get [`trigfil_stat0::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@trigfil_stat0`] module"]
#[doc(alias = "TRIGFIL_STAT0")]
pub type TrigfilStat0 = crate::Reg<trigfil_stat0::TrigfilStat0Spec>;
#[doc = "Trigger filter stat"]
pub mod trigfil_stat0;
#[doc = "TRIGFIL (rw) register accessor: TRIGFIL control\n\nYou can [`read`](crate::Reg::read) this register and get [`trigfil::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`trigfil::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@trigfil`] module"]
#[doc(alias = "TRIGFIL")]
pub type Trigfil = crate::Reg<trigfil::TrigfilSpec>;
#[doc = "TRIGFIL control"]
pub mod trigfil;
