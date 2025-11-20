/// Peripheral Access Crate
#[allow(unused_imports)]
#[rustfmt::skip]
pub mod pac {
    pub use nrf_pac::*;

    #[cfg(feature = "_ns")]
    #[doc(no_inline)]
    pub use nrf_pac::{
        FICR_NS as FICR,
        DPPIC00_NS as DPPIC00,
        PPIB00_NS as PPIB00,
        PPIB01_NS as PPIB01,
        AAR00_NS as AAR00,
        CCM00_NS as CCM00,
        ECB00_NS as ECB00,
        SPIM00_NS as SPIM00,
        SPIS00_NS as SPIS00,
        UARTE00_NS as UARTE00,
        VPR00_NS as VPR00,
        P2_NS as P2,
        CTRLAP_NS as CTRLAP,
        TAD_NS as TAD,
        TIMER00_NS as TIMER00,
        DPPIC10_NS as DPPIC10,
        PPIB10_NS as PPIB10,
        PPIB11_NS as PPIB11,
        TIMER10_NS as TIMER10,
        RTC10_NS as RTC10,
        EGU10_NS as EGU10,
        RADIO_NS as RADIO,
        DPPIC20_NS as DPPIC20,
        PPIB20_NS as PPIB20,
        PPIB21_NS as PPIB21,
        PPIB22_NS as PPIB22,
        SPIM20_NS as SPIM20,
        SPIS20_NS as SPIS20,
        TWIM20_NS as TWIM20,
        TWIS20_NS as TWIS20,
        UARTE20_NS as UARTE20,
        SPIM21_NS as SPIM21,
        SPIS21_NS as SPIS21,
        TWIM21_NS as TWIM21,
        TWIS21_NS as TWIS21,
        UARTE21_NS as UARTE21,
        SPIM22_NS as SPIM22,
        SPIS22_NS as SPIS22,
        TWIM22_NS as TWIM22,
        TWIS22_NS as TWIS22,
        UARTE22_NS as UARTE22,
        EGU20_NS as EGU20,
        TIMER20_NS as TIMER20,
        TIMER21_NS as TIMER21,
        TIMER22_NS as TIMER22,
        TIMER23_NS as TIMER23,
        TIMER24_NS as TIMER24,
        MEMCONF_NS as MEMCONF,
        PDM20_NS as PDM20,
        PDM21_NS as PDM21,
        PWM20_NS as PWM20,
        PWM21_NS as PWM21,
        PWM22_NS as PWM22,
        SAADC_NS as SAADC,
        NFCT_NS as NFCT,
        TEMP_NS as TEMP,
        P1_NS as P1,
        GPIOTE20_NS as GPIOTE20,
        I2S20_NS as I2S20,
        QDEC20_NS as QDEC20,
        QDEC21_NS as QDEC21,
        GRTC_NS as GRTC,
        DPPIC30_NS as DPPIC30,
        PPIB30_NS as PPIB30,
        SPIM30_NS as SPIM30,
        SPIS30_NS as SPIS30,
        TWIM30_NS as TWIM30,
        TWIS30_NS as TWIS30,
        UARTE30_NS as UARTE30,
        RTC30_NS as RTC30,
        COMP_NS as COMP,
        LPCOMP_NS as LPCOMP,
        WDT31_NS as WDT31,
        P0_NS as P0,
        GPIOTE30_NS as GPIOTE30,
        CLOCK_NS as CLOCK,
        POWER_NS as POWER,
        RESET_NS as RESET,
        OSCILLATORS_NS as OSCILLATORS,
        REGULATORS_NS as REGULATORS,
        TPIU_NS as TPIU,
        ETM_NS as ETM,
    };

    #[cfg(feature = "_s")]
    #[doc(no_inline)]
    pub use nrf_pac::{
        FICR_NS as FICR,
        SICR_S as SICR,
        ICACHEDATA_S as ICACHEDATA,
        ICACHEINFO_S as ICACHEINFO,
        SWI00_S as SWI00,
        SWI01_S as SWI01,
        SWI02_S as SWI02,
        SWI03_S as SWI03,
        SPU00_S as SPU00,
        MPC00_S as MPC00,
        DPPIC00_S as DPPIC00,
        PPIB00_S as PPIB00,
        PPIB01_S as PPIB01,
        KMU_S as KMU,
        AAR00_S as AAR00,
        CCM00_S as CCM00,
        ECB00_S as ECB00,
        CRACEN_S as CRACEN,
        SPIM00_S as SPIM00,
        SPIS00_S as SPIS00,
        UARTE00_S as UARTE00,
        GLITCHDET_S as GLITCHDET,
        RRAMC_S as RRAMC,
        VPR00_S as VPR00,
        P2_S as P2,
        CTRLAP_S as CTRLAP,
        TAD_S as TAD,
        TIMER00_S as TIMER00,
        SPU10_S as SPU10,
        DPPIC10_S as DPPIC10,
        PPIB10_S as PPIB10,
        PPIB11_S as PPIB11,
        TIMER10_S as TIMER10,
        RTC10_S as RTC10,
        EGU10_S as EGU10,
        RADIO_S as RADIO,
        SPU20_S as SPU20,
        DPPIC20_S as DPPIC20,
        PPIB20_S as PPIB20,
        PPIB21_S as PPIB21,
        PPIB22_S as PPIB22,
        SPIM20_S as SPIM20,
        SPIS20_S as SPIS20,
        TWIM20_S as TWIM20,
        TWIS20_S as TWIS20,
        UARTE20_S as UARTE20,
        SPIM21_S as SPIM21,
        SPIS21_S as SPIS21,
        TWIM21_S as TWIM21,
        TWIS21_S as TWIS21,
        UARTE21_S as UARTE21,
        SPIM22_S as SPIM22,
        SPIS22_S as SPIS22,
        TWIM22_S as TWIM22,
        TWIS22_S as TWIS22,
        UARTE22_S as UARTE22,
        EGU20_S as EGU20,
        TIMER20_S as TIMER20,
        TIMER21_S as TIMER21,
        TIMER22_S as TIMER22,
        TIMER23_S as TIMER23,
        TIMER24_S as TIMER24,
        MEMCONF_S as MEMCONF,
        PDM20_S as PDM20,
        PDM21_S as PDM21,
        PWM20_S as PWM20,
        PWM21_S as PWM21,
        PWM22_S as PWM22,
        SAADC_S as SAADC,
        NFCT_S as NFCT,
        TEMP_S as TEMP,
        P1_S as P1,
        GPIOTE20_S as GPIOTE20,
        TAMPC_S as TAMPC,
        I2S20_S as I2S20,
        QDEC20_S as QDEC20,
        QDEC21_S as QDEC21,
        GRTC_S as GRTC,
        SPU30_S as SPU30,
        DPPIC30_S as DPPIC30,
        PPIB30_S as PPIB30,
        SPIM30_S as SPIM30,
        SPIS30_S as SPIS30,
        TWIM30_S as TWIM30,
        TWIS30_S as TWIS30,
        UARTE30_S as UARTE30,
        RTC30_S as RTC30,
        COMP_S as COMP,
        LPCOMP_S as LPCOMP,
        WDT30_S as WDT30,
        WDT31_S as WDT31,
        P0_S as P0,
        GPIOTE30_S as GPIOTE30,
        CLOCK_S as CLOCK,
        POWER_S as POWER,
        RESET_S as RESET,
        OSCILLATORS_S as OSCILLATORS,
        REGULATORS_S as REGULATORS,
        CRACENCORE_S as CRACENCORE,
        CPUC_S as CPUC,
        ICACHE_S as ICACHE,
    };
}

/// The maximum buffer size that the EasyDMA can send/recv in one operation.
pub const EASY_DMA_SIZE: usize = (1 << 16) - 1;
pub const FORCE_COPY_BUFFER_SIZE: usize = 1024;

// 1.5 MB NVM
#[allow(unused)]
pub const FLASH_SIZE: usize = 1524 * 1024;

embassy_hal_internal::peripherals! {
    // PPI
    PPI00_CH0,
    PPI00_CH1,
    PPI00_CH2,
    PPI00_CH3,
    PPI00_CH4,
    PPI00_CH5,
    PPI00_CH6,
    PPI00_CH7,

    PPI10_CH0,
    PPI10_CH1,
    PPI10_CH2,
    PPI10_CH3,
    PPI10_CH4,
    PPI10_CH5,
    PPI10_CH6,
    PPI10_CH7,
    PPI10_CH8,
    PPI10_CH9,
    PPI10_CH10,
    PPI10_CH11,
    PPI10_CH12,
    PPI10_CH13,
    PPI10_CH14,
    PPI10_CH15,
    PPI10_CH16,
    PPI10_CH17,
    PPI10_CH18,
    PPI10_CH19,
    PPI10_CH20,
    PPI10_CH21,
    PPI10_CH22,
    PPI10_CH23,

    PPI20_CH0,
    PPI20_CH1,
    PPI20_CH2,
    PPI20_CH3,
    PPI20_CH4,
    PPI20_CH5,
    PPI20_CH6,
    PPI20_CH7,
    PPI20_CH8,
    PPI20_CH9,
    PPI20_CH10,
    PPI20_CH11,
    PPI20_CH12,
    PPI20_CH13,
    PPI20_CH14,
    PPI20_CH15,

    PPI30_CH0,
    PPI30_CH1,
    PPI30_CH2,
    PPI30_CH3,

    PPI00_GROUP0,
    PPI00_GROUP1,

    PPI10_GROUP0,
    PPI10_GROUP1,
    PPI10_GROUP2,
    PPI10_GROUP3,
    PPI10_GROUP4,
    PPI10_GROUP5,

    PPI20_GROUP0,
    PPI20_GROUP1,
    PPI20_GROUP2,
    PPI20_GROUP3,
    PPI20_GROUP4,
    PPI20_GROUP5,

    PPI30_GROUP0,
    PPI30_GROUP1,

    // PPI BRIDGE channels
    PPIB00_CH0,
    PPIB00_CH1,
    PPIB00_CH2,
    PPIB00_CH3,
    PPIB00_CH4,
    PPIB00_CH5,
    PPIB00_CH6,
    PPIB00_CH7,

    PPIB01_CH0,
    PPIB01_CH1,
    PPIB01_CH2,
    PPIB01_CH3,
    PPIB01_CH4,
    PPIB01_CH5,
    PPIB01_CH6,
    PPIB01_CH7,

    PPIB10_CH0,
    PPIB10_CH1,
    PPIB10_CH2,
    PPIB10_CH3,
    PPIB10_CH4,
    PPIB10_CH5,
    PPIB10_CH6,
    PPIB10_CH7,

    PPIB11_CH0,
    PPIB11_CH1,
    PPIB11_CH2,
    PPIB11_CH3,
    PPIB11_CH4,
    PPIB11_CH5,
    PPIB11_CH6,
    PPIB11_CH7,
    PPIB11_CH8,
    PPIB11_CH9,
    PPIB11_CH10,
    PPIB11_CH11,
    PPIB11_CH12,
    PPIB11_CH13,
    PPIB11_CH14,
    PPIB11_CH15,

    PPIB20_CH0,
    PPIB20_CH1,
    PPIB20_CH2,
    PPIB20_CH3,
    PPIB20_CH4,
    PPIB20_CH5,
    PPIB20_CH6,
    PPIB20_CH7,

    PPIB21_CH0,
    PPIB21_CH1,
    PPIB21_CH2,
    PPIB21_CH3,
    PPIB21_CH4,
    PPIB21_CH5,
    PPIB21_CH6,
    PPIB21_CH7,
    PPIB21_CH8,
    PPIB21_CH9,
    PPIB21_CH10,
    PPIB21_CH11,
    PPIB21_CH12,
    PPIB21_CH13,
    PPIB21_CH14,
    PPIB21_CH15,

    PPIB22_CH0,
    PPIB22_CH1,
    PPIB22_CH2,
    PPIB22_CH3,

    PPIB30_CH0,
    PPIB30_CH1,
    PPIB30_CH2,
    PPIB30_CH3,

    // Timers
    TIMER00,
    TIMER10,
    TIMER20,
    TIMER21,
    TIMER22,
    TIMER23,
    TIMER24,

    // GPIO port 0
    P0_00,
    P0_01,
    P0_02,
    P0_03,
    P0_04,
    P0_05,
    P0_06,

    // GPIO port 1
    P1_00,
    P1_01,
    P1_02,
    P1_03,
    P1_04,
    P1_05,
    P1_06,
    P1_07,
    P1_08,
    P1_09,
    P1_10,
    P1_11,
    P1_12,
    P1_13,
    P1_14,
    P1_15,
    P1_16,


    // GPIO port 2
    P2_00,
    P2_01,
    P2_02,
    P2_03,
    P2_04,
    P2_05,
    P2_06,
    P2_07,
    P2_08,
    P2_09,
    P2_10,

    // GRTC
    GRTC,

    // RTC
    RTC10,
    RTC30,

    // PWM
    PWM20,
    PWM21,
    PWM22,

    // SERIAL
    SERIAL00,
    SERIAL20,
    SERIAL21,
    SERIAL22,
    SERIAL30,

    // SAADC
    SAADC,

    // RADIO
    RADIO,


    // GPIOTE instances
    GPIOTE20,
    GPIOTE30,

    // GPIOTE channels
    GPIOTE20_CH0,
    GPIOTE20_CH1,
    GPIOTE20_CH2,
    GPIOTE20_CH3,
    GPIOTE20_CH4,
    GPIOTE20_CH5,
    GPIOTE20_CH6,
    GPIOTE20_CH7,
    GPIOTE30_CH0,
    GPIOTE30_CH1,
    GPIOTE30_CH2,
    GPIOTE30_CH3,

    // CRACEN
    #[cfg(feature = "_s")]
    CRACEN,

    #[cfg(feature = "_s")]
    // RRAMC
    RRAMC,

    // TEMP
    TEMP,

    // WDT
    #[cfg(feature = "_ns")]
    WDT,
    #[cfg(feature = "_s")]
    WDT0,
    #[cfg(feature = "_s")]
    WDT1,
}

impl_pin!(P0_00, 0, 0);
impl_pin!(P0_01, 0, 1);
impl_pin!(P0_02, 0, 2);
impl_pin!(P0_03, 0, 3);
impl_pin!(P0_04, 0, 4);
impl_pin!(P0_05, 0, 5);
impl_pin!(P0_06, 0, 6);

impl_pin!(P1_00, 1, 0);
impl_pin!(P1_01, 1, 1);
impl_pin!(P1_02, 1, 2);
impl_pin!(P1_03, 1, 3);
impl_pin!(P1_04, 1, 4);
impl_pin!(P1_05, 1, 5);
impl_pin!(P1_06, 1, 6);
impl_pin!(P1_07, 1, 7);
impl_pin!(P1_08, 1, 8);
impl_pin!(P1_09, 1, 9);
impl_pin!(P1_10, 1, 10);
impl_pin!(P1_11, 1, 11);
impl_pin!(P1_12, 1, 12);
impl_pin!(P1_13, 1, 13);
impl_pin!(P1_14, 1, 14);
impl_pin!(P1_15, 1, 15);
impl_pin!(P1_16, 1, 16);

impl_pin!(P2_00, 2, 0);
impl_pin!(P2_01, 2, 1);
impl_pin!(P2_02, 2, 2);
impl_pin!(P2_03, 2, 3);
impl_pin!(P2_04, 2, 4);
impl_pin!(P2_05, 2, 5);
impl_pin!(P2_06, 2, 6);
impl_pin!(P2_07, 2, 7);
impl_pin!(P2_08, 2, 8);
impl_pin!(P2_09, 2, 9);
impl_pin!(P2_10, 2, 10);

cfg_if::cfg_if! {
    if #[cfg(feature = "gpiote")] {
        impl_gpiote_pin!(P0_00, GPIOTE30);
        impl_gpiote_pin!(P0_01, GPIOTE30);
        impl_gpiote_pin!(P0_02, GPIOTE30);
        impl_gpiote_pin!(P0_03, GPIOTE30);
        impl_gpiote_pin!(P0_04, GPIOTE30);
        impl_gpiote_pin!(P0_05, GPIOTE30);
        impl_gpiote_pin!(P0_06, GPIOTE30);

        impl_gpiote_pin!(P1_00, GPIOTE20);
        impl_gpiote_pin!(P1_01, GPIOTE20);
        impl_gpiote_pin!(P1_02, GPIOTE20);
        impl_gpiote_pin!(P1_03, GPIOTE20);
        impl_gpiote_pin!(P1_04, GPIOTE20);
        impl_gpiote_pin!(P1_05, GPIOTE20);
        impl_gpiote_pin!(P1_06, GPIOTE20);
        impl_gpiote_pin!(P1_07, GPIOTE20);
        impl_gpiote_pin!(P1_08, GPIOTE20);
        impl_gpiote_pin!(P1_09, GPIOTE20);
        impl_gpiote_pin!(P1_10, GPIOTE20);
        impl_gpiote_pin!(P1_11, GPIOTE20);
        impl_gpiote_pin!(P1_12, GPIOTE20);
        impl_gpiote_pin!(P1_13, GPIOTE20);
        impl_gpiote_pin!(P1_14, GPIOTE20);
        impl_gpiote_pin!(P1_15, GPIOTE20);
        impl_gpiote_pin!(P1_16, GPIOTE20);
    }
}

impl_rtc!(RTC10, RTC10, RTC10);
impl_rtc!(RTC30, RTC30, RTC30);

#[cfg(feature = "_ns")]
impl_wdt!(WDT, WDT31, WDT31, 0);
#[cfg(feature = "_s")]
impl_wdt!(WDT0, WDT31, WDT31, 0);
#[cfg(feature = "_s")]
impl_wdt!(WDT1, WDT30, WDT30, 1);
// DPPI00 channels
impl_ppi_channel!(PPI00_CH0, DPPIC00, 0 => configurable);
impl_ppi_channel!(PPI00_CH1, DPPIC00, 1 => configurable);
impl_ppi_channel!(PPI00_CH2, DPPIC00, 2 => configurable);
impl_ppi_channel!(PPI00_CH3, DPPIC00, 3 => configurable);
impl_ppi_channel!(PPI00_CH4, DPPIC00, 4 => configurable);
impl_ppi_channel!(PPI00_CH5, DPPIC00, 5 => configurable);
impl_ppi_channel!(PPI00_CH6, DPPIC00, 6 => configurable);
impl_ppi_channel!(PPI00_CH7, DPPIC00, 7 => configurable);

// DPPI10 channels
impl_ppi_channel!(PPI10_CH0, DPPIC10, 0 => static);

// DPPI20 channels
impl_ppi_channel!(PPI20_CH0, DPPIC20, 0 => configurable);
impl_ppi_channel!(PPI20_CH1, DPPIC20, 1 => configurable);
impl_ppi_channel!(PPI20_CH2, DPPIC20, 2 => configurable);
impl_ppi_channel!(PPI20_CH3, DPPIC20, 3 => configurable);
impl_ppi_channel!(PPI20_CH4, DPPIC20, 4 => configurable);
impl_ppi_channel!(PPI20_CH5, DPPIC20, 5 => configurable);
impl_ppi_channel!(PPI20_CH6, DPPIC20, 6 => configurable);
impl_ppi_channel!(PPI20_CH7, DPPIC20, 7 => configurable);
impl_ppi_channel!(PPI20_CH8, DPPIC20, 8 => configurable);
impl_ppi_channel!(PPI20_CH9, DPPIC20, 9 => configurable);
impl_ppi_channel!(PPI20_CH10, DPPIC20, 10 => configurable);
impl_ppi_channel!(PPI20_CH11, DPPIC20, 11 => configurable);
impl_ppi_channel!(PPI20_CH12, DPPIC20, 12 => configurable);
impl_ppi_channel!(PPI20_CH13, DPPIC20, 13 => configurable);
impl_ppi_channel!(PPI20_CH14, DPPIC20, 14 => configurable);
impl_ppi_channel!(PPI20_CH15, DPPIC20, 15 => configurable);

// DPPI30 channels
impl_ppi_channel!(PPI30_CH0, DPPIC30, 0 => configurable);
impl_ppi_channel!(PPI30_CH1, DPPIC30, 1 => configurable);
impl_ppi_channel!(PPI30_CH2, DPPIC30, 2 => configurable);
impl_ppi_channel!(PPI30_CH3, DPPIC30, 3 => configurable);

// DPPI00 groups
impl_ppi_group!(PPI00_GROUP0, DPPIC00, 0);
impl_ppi_group!(PPI00_GROUP1, DPPIC00, 1);

// DPPI10 groups
impl_ppi_group!(PPI10_GROUP0, DPPIC10, 0);

// DPPI20 groups
impl_ppi_group!(PPI20_GROUP0, DPPIC20, 0);
impl_ppi_group!(PPI20_GROUP1, DPPIC20, 1);
impl_ppi_group!(PPI20_GROUP2, DPPIC20, 2);
impl_ppi_group!(PPI20_GROUP3, DPPIC20, 3);
impl_ppi_group!(PPI20_GROUP4, DPPIC20, 4);
impl_ppi_group!(PPI20_GROUP5, DPPIC20, 5);

// DPPI30 groups
impl_ppi_group!(PPI30_GROUP0, DPPIC30, 0);
impl_ppi_group!(PPI30_GROUP1, DPPIC30, 1);

impl_timer!(TIMER00, TIMER00, TIMER00);
impl_timer!(TIMER10, TIMER10, TIMER10);
impl_timer!(TIMER20, TIMER20, TIMER20);
impl_timer!(TIMER21, TIMER21, TIMER21);
impl_timer!(TIMER22, TIMER22, TIMER22);
impl_timer!(TIMER23, TIMER23, TIMER23);
impl_timer!(TIMER24, TIMER24, TIMER24);

impl_twim!(SERIAL20, TWIM20, SERIAL20);
impl_twim!(SERIAL21, TWIM21, SERIAL21);
impl_twim!(SERIAL22, TWIM22, SERIAL22);
impl_twim!(SERIAL30, TWIM30, SERIAL30);

impl_twis!(SERIAL20, TWIS20, SERIAL20);
impl_twis!(SERIAL21, TWIS21, SERIAL21);
impl_twis!(SERIAL22, TWIS22, SERIAL22);
impl_twis!(SERIAL30, TWIS30, SERIAL30);

impl_pwm!(PWM20, PWM20, PWM20);
impl_pwm!(PWM21, PWM21, PWM21);
impl_pwm!(PWM22, PWM22, PWM22);

#[cfg(feature = "_s")]
impl_spim!(
    SERIAL00,
    SPIM00,
    SERIAL00,
    match pac::OSCILLATORS_S.pll().currentfreq().read().currentfreq() {
        pac::oscillators::vals::Currentfreq::CK128M => 128_000_000,
        pac::oscillators::vals::Currentfreq::CK64M => 64_000_000,
        _ => unreachable!(),
    }
);
#[cfg(feature = "_ns")]
impl_spim!(
    SERIAL00,
    SPIM00,
    SERIAL00,
    match pac::OSCILLATORS_NS.pll().currentfreq().read().currentfreq() {
        pac::oscillators::vals::Currentfreq::CK128M => 128_000_000,
        pac::oscillators::vals::Currentfreq::CK64M => 64_000_000,
        _ => unreachable!(),
    }
);
impl_spim!(SERIAL20, SPIM20, SERIAL20, 16_000_000);
impl_spim!(SERIAL21, SPIM21, SERIAL21, 16_000_000);
impl_spim!(SERIAL22, SPIM22, SERIAL22, 16_000_000);
impl_spim!(SERIAL30, SPIM30, SERIAL30, 16_000_000);

impl_spis!(SERIAL20, SPIS20, SERIAL20);
impl_spis!(SERIAL21, SPIS21, SERIAL21);
impl_spis!(SERIAL22, SPIS22, SERIAL22);
impl_spis!(SERIAL30, SPIS30, SERIAL30);

impl_uarte!(SERIAL00, UARTE00, SERIAL00);
impl_uarte!(SERIAL20, UARTE20, SERIAL20);
impl_uarte!(SERIAL21, UARTE21, SERIAL21);
impl_uarte!(SERIAL22, UARTE22, SERIAL22);
impl_uarte!(SERIAL30, UARTE30, SERIAL30);

// NB: SAADC uses "pin" abstraction, not "AIN"
impl_saadc_input!(P1_04, 1, 4);
impl_saadc_input!(P1_05, 1, 5);
impl_saadc_input!(P1_06, 1, 6);
impl_saadc_input!(P1_07, 1, 7);
impl_saadc_input!(P1_11, 1, 11);
impl_saadc_input!(P1_12, 1, 12);
impl_saadc_input!(P1_13, 1, 13);
impl_saadc_input!(P1_14, 1, 14);

#[cfg(feature = "_s")]
impl_cracen!(CRACEN, CRACEN, CRACEN);

embassy_hal_internal::interrupt_mod!(
    SWI00,
    SWI01,
    SWI02,
    SWI03,
    SPU00,
    MPC00,
    AAR00_CCM00,
    ECB00,
    CRACEN,
    SERIAL00,
    RRAMC,
    VPR00,
    CTRLAP,
    TIMER00,
    SPU10,
    TIMER10,
    RTC10,
    EGU10,
    RADIO_0,
    RADIO_1,
    SPU20,
    SERIAL20,
    SERIAL21,
    SERIAL22,
    EGU20,
    TIMER20,
    TIMER21,
    TIMER22,
    TIMER23,
    TIMER24,
    PDM20,
    PDM21,
    PWM20,
    PWM21,
    PWM22,
    SAADC,
    NFCT,
    TEMP,
    GPIOTE20_0,
    GPIOTE20_1,
    TAMPC,
    I2S20,
    QDEC20,
    QDEC21,
    GRTC_0,
    GRTC_1,
    GRTC_2,
    GRTC_3,
    SPU30,
    SERIAL30,
    RTC30,
    COMP_LPCOMP,
    WDT30,
    WDT31,
    GPIOTE30_0,
    GPIOTE30_1,
    CLOCK_POWER,
);
