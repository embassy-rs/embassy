#[allow(unused_imports)]
#[rustfmt::skip]
pub mod pac {
    // The nRF9160 has a secure and non-secure (NS) mode.
    // To avoid cfg spam, we remove _ns or _s suffixes here.

    #[doc(no_inline)]
    pub use nrf9160_pac::{
        interrupt,
        Interrupt,

        cc_host_rgf_s as cc_host_rgf,
        clock_ns as clock,
        cryptocell_s as cryptocell,
        ctrl_ap_peri_s as ctrl_ap_peri,
        dppic_ns as dppic,
        egu0_ns as egu0,
        ficr_s as ficr,
        fpu_ns as fpu,
        gpiote0_s as gpiote,
        i2s_ns as i2s,
        ipc_ns as ipc,
        kmu_ns as kmu,
        nvmc_ns as nvmc,
        p0_ns as p0,
        pdm_ns as pdm,
        power_ns as power,
        pwm0_ns as pwm0,
        regulators_ns as regulators,
        rtc0_ns as rtc0,
        saadc_ns as saadc,
        spim0_ns as spim0,
        spis0_ns as spis0,
        spu_s as spu,
        tad_s as tad,
        timer0_ns as timer0,
        twim0_ns as twim0,
        twis0_ns as twis0,
        uarte0_ns as uarte0,
        uicr_s as uicr,
        vmc_ns as vmc,
        wdt_ns as wdt,
    };
    
    #[cfg(feature = "nrf9160-ns")]
    #[doc(no_inline)]
    pub use nrf9160_pac::{
        CLOCK_NS as CLOCK,
        DPPIC_NS as DPPIC,
        EGU0_NS as EGU0,
        EGU1_NS as EGU1,
        EGU2_NS as EGU2,
        EGU3_NS as EGU3,
        EGU4_NS as EGU4,
        EGU5_NS as EGU5,
        FPU_NS as FPU,
        GPIOTE1_NS as GPIOTE1,
        I2S_NS as I2S,
        IPC_NS as IPC,
        KMU_NS as KMU,
        NVMC_NS as NVMC,
        P0_NS as P0,
        PDM_NS as PDM,
        POWER_NS as POWER,
        PWM0_NS as PWM0,
        PWM1_NS as PWM1,
        PWM2_NS as PWM2,
        PWM3_NS as PWM3,
        REGULATORS_NS as REGULATORS,
        RTC0_NS as RTC0,
        RTC1_NS as RTC1,
        SAADC_NS as SAADC,
        SPIM0_NS as SPIM0,
        SPIM1_NS as SPIM1,
        SPIM2_NS as SPIM2,
        SPIM3_NS as SPIM3,
        SPIS0_NS as SPIS0,
        SPIS1_NS as SPIS1,
        SPIS2_NS as SPIS2,
        SPIS3_NS as SPIS3,
        TIMER0_NS as TIMER0,
        TIMER1_NS as TIMER1,
        TIMER2_NS as TIMER2,
        TWIM0_NS as TWIM0,
        TWIM1_NS as TWIM1,
        TWIM2_NS as TWIM2,
        TWIM3_NS as TWIM3,
        TWIS0_NS as TWIS0,
        TWIS1_NS as TWIS1,
        TWIS2_NS as TWIS2,
        TWIS3_NS as TWIS3,
        UARTE0_NS as UARTE0,
        UARTE1_NS as UARTE1,
        UARTE2_NS as UARTE2,
        UARTE3_NS as UARTE3,
        VMC_NS as VMC,
        WDT_NS as WDT,
    };

    #[cfg(feature = "nrf9160-s")]
    #[doc(no_inline)]
    pub use nrf9160_pac::{
        CC_HOST_RGF_S as CC_HOST_RGF,
        CLOCK_S as CLOCK,
        CRYPTOCELL_S as CRYPTOCELL,
        CTRL_AP_PERI_S as CTRL_AP_PERI,
        DPPIC_S as DPPIC,
        EGU0_S as EGU0,
        EGU1_S as EGU1,
        EGU2_S as EGU2,
        EGU3_S as EGU3,
        EGU4_S as EGU4,
        EGU5_S as EGU5,
        FICR_S as FICR,
        FPU_S as FPU,
        GPIOTE0_S as GPIOTE0,
        I2S_S as I2S,
        IPC_S as IPC,
        KMU_S as KMU,
        NVMC_S as NVMC,
        P0_S as P0,
        PDM_S as PDM,
        POWER_S as POWER,
        PWM0_S as PWM0,
        PWM1_S as PWM1,
        PWM2_S as PWM2,
        PWM3_S as PWM3,
        REGULATORS_S as REGULATORS,
        RTC0_S as RTC0,
        RTC1_S as RTC1,
        SAADC_S as SAADC,
        SPIM0_S as SPIM0,
        SPIM1_S as SPIM1,
        SPIM2_S as SPIM2,
        SPIM3_S as SPIM3,
        SPIS0_S as SPIS0,
        SPIS1_S as SPIS1,
        SPIS2_S as SPIS2,
        SPIS3_S as SPIS3,
        SPU_S as SPU,
        TAD_S as TAD,
        TIMER0_S as TIMER0,
        TIMER1_S as TIMER1,
        TIMER2_S as TIMER2,
        TWIM0_S as TWIM0,
        TWIM1_S as TWIM1,
        TWIM2_S as TWIM2,
        TWIM3_S as TWIM3,
        TWIS0_S as TWIS0,
        TWIS1_S as TWIS1,
        TWIS2_S as TWIS2,
        TWIS3_S as TWIS3,
        UARTE0_S as UARTE0,
        UARTE1_S as UARTE1,
        UARTE2_S as UARTE2,
        UARTE3_S as UARTE3,
        UICR_S as UICR,
        VMC_S as VMC,
        WDT_S as WDT,
    };
}

/// The maximum buffer size that the EasyDMA can send/recv in one operation.
pub const EASY_DMA_SIZE: usize = (1 << 13) - 1;
pub const FORCE_COPY_BUFFER_SIZE: usize = 1024;

embassy_hal_common::peripherals! {
    // RTC
    RTC0,
    RTC1,

    // WDT
    WDT,

    // UARTE, TWI & SPI
    UARTETWISPI0,
    UARTETWISPI1,
    UARTETWISPI2,
    UARTETWISPI3,

    // SAADC
    SAADC,

    // PWM
    PWM0,
    PWM1,
    PWM2,
    PWM3,

    // TIMER
    TIMER0,
    TIMER1,
    TIMER2,

    // GPIOTE
    GPIOTE_CH0,
    GPIOTE_CH1,
    GPIOTE_CH2,
    GPIOTE_CH3,
    GPIOTE_CH4,
    GPIOTE_CH5,
    GPIOTE_CH6,
    GPIOTE_CH7,

    // PPI
    PPI_CH0,
    PPI_CH1,
    PPI_CH2,
    PPI_CH3,
    PPI_CH4,
    PPI_CH5,
    PPI_CH6,
    PPI_CH7,
    PPI_CH8,
    PPI_CH9,
    PPI_CH10,
    PPI_CH11,
    PPI_CH12,
    PPI_CH13,
    PPI_CH14,
    PPI_CH15,

    PPI_GROUP0,
    PPI_GROUP1,
    PPI_GROUP2,
    PPI_GROUP3,
    PPI_GROUP4,
    PPI_GROUP5,

    // GPIO port 0
    P0_00,
    P0_01,
    P0_02,
    P0_03,
    P0_04,
    P0_05,
    P0_06,
    P0_07,
    P0_08,
    P0_09,
    P0_10,
    P0_11,
    P0_12,
    P0_13,
    P0_14,
    P0_15,
    P0_16,
    P0_17,
    P0_18,
    P0_19,
    P0_20,
    P0_21,
    P0_22,
    P0_23,
    P0_24,
    P0_25,
    P0_26,
    P0_27,
    P0_28,
    P0_29,
    P0_30,
    P0_31,
}

impl_uarte!(UARTETWISPI0, UARTE0, UARTE0_SPIM0_SPIS0_TWIM0_TWIS0);
impl_uarte!(UARTETWISPI1, UARTE1, UARTE1_SPIM1_SPIS1_TWIM1_TWIS1);
impl_uarte!(UARTETWISPI2, UARTE2, UARTE2_SPIM2_SPIS2_TWIM2_TWIS2);
impl_uarte!(UARTETWISPI3, UARTE3, UARTE3_SPIM3_SPIS3_TWIM3_TWIS3);

impl_spim!(UARTETWISPI0, SPIM0, UARTE0_SPIM0_SPIS0_TWIM0_TWIS0);
impl_spim!(UARTETWISPI1, SPIM1, UARTE1_SPIM1_SPIS1_TWIM1_TWIS1);
impl_spim!(UARTETWISPI2, SPIM2, UARTE2_SPIM2_SPIS2_TWIM2_TWIS2);
impl_spim!(UARTETWISPI3, SPIM3, UARTE3_SPIM3_SPIS3_TWIM3_TWIS3);

impl_twim!(UARTETWISPI0, TWIM0, UARTE0_SPIM0_SPIS0_TWIM0_TWIS0);
impl_twim!(UARTETWISPI1, TWIM1, UARTE1_SPIM1_SPIS1_TWIM1_TWIS1);
impl_twim!(UARTETWISPI2, TWIM2, UARTE2_SPIM2_SPIS2_TWIM2_TWIS2);
impl_twim!(UARTETWISPI3, TWIM3, UARTE3_SPIM3_SPIS3_TWIM3_TWIS3);

impl_pwm!(PWM0, PWM0, PWM0);
impl_pwm!(PWM1, PWM1, PWM1);
impl_pwm!(PWM2, PWM2, PWM2);
impl_pwm!(PWM3, PWM3, PWM3);

impl_timer!(TIMER0, TIMER0, TIMER0);
impl_timer!(TIMER1, TIMER1, TIMER1);
impl_timer!(TIMER2, TIMER2, TIMER2);

impl_pin!(P0_00, 0, 0);
impl_pin!(P0_01, 0, 1);
impl_pin!(P0_02, 0, 2);
impl_pin!(P0_03, 0, 3);
impl_pin!(P0_04, 0, 4);
impl_pin!(P0_05, 0, 5);
impl_pin!(P0_06, 0, 6);
impl_pin!(P0_07, 0, 7);
impl_pin!(P0_08, 0, 8);
impl_pin!(P0_09, 0, 9);
impl_pin!(P0_10, 0, 10);
impl_pin!(P0_11, 0, 11);
impl_pin!(P0_12, 0, 12);
impl_pin!(P0_13, 0, 13);
impl_pin!(P0_14, 0, 14);
impl_pin!(P0_15, 0, 15);
impl_pin!(P0_16, 0, 16);
impl_pin!(P0_17, 0, 17);
impl_pin!(P0_18, 0, 18);
impl_pin!(P0_19, 0, 19);
impl_pin!(P0_20, 0, 20);
impl_pin!(P0_21, 0, 21);
impl_pin!(P0_22, 0, 22);
impl_pin!(P0_23, 0, 23);
impl_pin!(P0_24, 0, 24);
impl_pin!(P0_25, 0, 25);
impl_pin!(P0_26, 0, 26);
impl_pin!(P0_27, 0, 27);
impl_pin!(P0_28, 0, 28);
impl_pin!(P0_29, 0, 29);
impl_pin!(P0_30, 0, 30);
impl_pin!(P0_31, 0, 31);

impl_ppi_channel!(PPI_CH0, 0 => configurable);
impl_ppi_channel!(PPI_CH1, 1 => configurable);
impl_ppi_channel!(PPI_CH2, 2 => configurable);
impl_ppi_channel!(PPI_CH3, 3 => configurable);
impl_ppi_channel!(PPI_CH4, 4 => configurable);
impl_ppi_channel!(PPI_CH5, 5 => configurable);
impl_ppi_channel!(PPI_CH6, 6 => configurable);
impl_ppi_channel!(PPI_CH7, 7 => configurable);
impl_ppi_channel!(PPI_CH8, 8 => configurable);
impl_ppi_channel!(PPI_CH9, 9 => configurable);
impl_ppi_channel!(PPI_CH10, 10 => configurable);
impl_ppi_channel!(PPI_CH11, 11 => configurable);
impl_ppi_channel!(PPI_CH12, 12 => configurable);
impl_ppi_channel!(PPI_CH13, 13 => configurable);
impl_ppi_channel!(PPI_CH14, 14 => configurable);
impl_ppi_channel!(PPI_CH15, 15 => configurable);

impl_saadc_input!(P0_13, ANALOGINPUT0);
impl_saadc_input!(P0_14, ANALOGINPUT1);
impl_saadc_input!(P0_15, ANALOGINPUT2);
impl_saadc_input!(P0_16, ANALOGINPUT3);
impl_saadc_input!(P0_17, ANALOGINPUT4);
impl_saadc_input!(P0_18, ANALOGINPUT5);
impl_saadc_input!(P0_19, ANALOGINPUT6);
impl_saadc_input!(P0_20, ANALOGINPUT7);

pub mod irqs {
    use crate::pac::Interrupt as InterruptEnum;
    use embassy_macros::interrupt_declare as declare;

    declare!(SPU);
    declare!(CLOCK_POWER);
    declare!(UARTE0_SPIM0_SPIS0_TWIM0_TWIS0);
    declare!(UARTE1_SPIM1_SPIS1_TWIM1_TWIS1);
    declare!(UARTE2_SPIM2_SPIS2_TWIM2_TWIS2);
    declare!(UARTE3_SPIM3_SPIS3_TWIM3_TWIS3);
    declare!(GPIOTE0);
    declare!(SAADC);
    declare!(TIMER0);
    declare!(TIMER1);
    declare!(TIMER2);
    declare!(RTC0);
    declare!(RTC1);
    declare!(WDT);
    declare!(EGU0);
    declare!(EGU1);
    declare!(EGU2);
    declare!(EGU3);
    declare!(EGU4);
    declare!(EGU5);
    declare!(PWM0);
    declare!(PWM1);
    declare!(PWM2);
    declare!(PDM);
    declare!(PWM3);
    declare!(I2S);
    declare!(IPC);
    declare!(FPU);
    declare!(GPIOTE1);
    declare!(KMU);
    declare!(CRYPTOCELL);
}
