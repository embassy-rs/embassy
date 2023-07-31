/// Peripheral Access Crate
#[allow(unused_imports)]
#[rustfmt::skip]
pub mod pac {
    // The nRF5340 has a secure and non-secure (NS) mode.
    // To avoid cfg spam, we remove _ns or _s suffixes here.

    pub use nrf5340_app_pac::NVIC_PRIO_BITS;
    
    #[doc(no_inline)]
    pub use nrf5340_app_pac::{
        interrupt,
        Interrupt,
        Peripherals,

        cache_s as cache,
        cachedata_s as cachedata,
        cacheinfo_s as cacheinfo,
        clock_ns as clock,
        comp_ns as comp,
        cryptocell_s as cryptocell,
        cti_s as cti,
        ctrlap_ns as ctrlap,
        dcnf_ns as dcnf,
        dppic_ns as dppic,
        egu0_ns as egu0,
        ficr_s as ficr,
        fpu_ns as fpu,
        gpiote0_s as gpiote,
        i2s0_ns as i2s0,
        ipc_ns as ipc,
        kmu_ns as kmu,
        lpcomp_ns as lpcomp,
        mutex_ns as mutex,
        nfct_ns as nfct,
        nvmc_ns as nvmc,
        oscillators_ns as oscillators,
        p0_ns as p0,
        pdm0_ns as pdm,
        power_ns as power,
        pwm0_ns as pwm0,
        qdec0_ns as qdec,
        qspi_ns as qspi,
        regulators_ns as regulators,
        reset_ns as reset,
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
        usbd_ns as usbd,
        usbregulator_ns as usbregulator,
        vmc_ns as vmc,
        wdt0_ns as wdt0,
    };
    
    #[cfg(feature = "nrf5340-app-ns")]
    #[doc(no_inline)]
    pub use nrf5340_app_pac::{
        CLOCK_NS as CLOCK,
        COMP_NS as COMP,
        CTRLAP_NS as CTRLAP,
        DCNF_NS as DCNF,
        DPPIC_NS as DPPIC,
        EGU0_NS as EGU0,
        EGU1_NS as EGU1,
        EGU2_NS as EGU2,
        EGU3_NS as EGU3,
        EGU4_NS as EGU4,
        EGU5_NS as EGU5,
        FPU_NS as FPU,
        GPIOTE1_NS as GPIOTE1,
        I2S0_NS as I2S0,
        IPC_NS as IPC,
        KMU_NS as KMU,
        LPCOMP_NS as LPCOMP,
        MUTEX_NS as MUTEX,
        NFCT_NS as NFCT,
        NVMC_NS as NVMC,
        OSCILLATORS_NS as OSCILLATORS,
        P0_NS as P0,
        P1_NS as P1,
        PDM0_NS as PDM0,
        POWER_NS as POWER,
        PWM0_NS as PWM0,
        PWM1_NS as PWM1,
        PWM2_NS as PWM2,
        PWM3_NS as PWM3,
        QDEC0_NS as QDEC0,
        QDEC1_NS as QDEC1,
        QSPI_NS as QSPI,
        REGULATORS_NS as REGULATORS,
        RESET_NS as RESET,
        RTC0_NS as RTC0,
        RTC1_NS as RTC1,
        SAADC_NS as SAADC,
        SPIM0_NS as SPIM0,
        SPIM1_NS as SPIM1,
        SPIM2_NS as SPIM2,
        SPIM3_NS as SPIM3,
        SPIM4_NS as SPIM4,
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
        USBD_NS as USBD,
        USBREGULATOR_NS as USBREGULATOR,
        VMC_NS as VMC,
        WDT0_NS as WDT0,
        WDT1_NS as WDT1,
    };

    #[cfg(feature = "nrf5340-app-s")]
    #[doc(no_inline)]
    pub use nrf5340_app_pac::{
        CACHEDATA_S as CACHEDATA,
        CACHEINFO_S as CACHEINFO,
        CACHE_S as CACHE,
        CLOCK_S as CLOCK,
        COMP_S as COMP,
        CRYPTOCELL_S as CRYPTOCELL,
        CTI_S as CTI,
        CTRLAP_S as CTRLAP,
        DCNF_S as DCNF,
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
        I2S0_S as I2S0,
        IPC_S as IPC,
        KMU_S as KMU,
        LPCOMP_S as LPCOMP,
        MUTEX_S as MUTEX,
        NFCT_S as NFCT,
        NVMC_S as NVMC,
        OSCILLATORS_S as OSCILLATORS,
        P0_S as P0,
        P1_S as P1,
        PDM0_S as PDM0,
        POWER_S as POWER,
        PWM0_S as PWM0,
        PWM1_S as PWM1,
        PWM2_S as PWM2,
        PWM3_S as PWM3,
        QDEC0_S as QDEC0,
        QDEC1_S as QDEC1,
        QSPI_S as QSPI,
        REGULATORS_S as REGULATORS,
        RESET_S as RESET,
        RTC0_S as RTC0,
        RTC1_S as RTC1,
        SAADC_S as SAADC,
        SPIM0_S as SPIM0,
        SPIM1_S as SPIM1,
        SPIM2_S as SPIM2,
        SPIM3_S as SPIM3,
        SPIM4_S as SPIM4,
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
        USBD_S as USBD,
        USBREGULATOR_S as USBREGULATOR,
        VMC_S as VMC,
        WDT0_S as WDT0,
        WDT1_S as WDT1,
    };
}

/// The maximum buffer size that the EasyDMA can send/recv in one operation.
pub const EASY_DMA_SIZE: usize = (1 << 16) - 1;
pub const FORCE_COPY_BUFFER_SIZE: usize = 1024;

pub const FLASH_SIZE: usize = 1024 * 1024;

embassy_hal_internal::peripherals! {
    // USB
    USBD,

    // RTC
    RTC0,
    RTC1,

    // WDT
    WDT,

    // NVMC
    NVMC,

    // UARTE, TWI & SPI
    SERIAL0,
    SERIAL1,
    SERIAL2,
    SERIAL3,

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

    // QSPI
    QSPI,

    // PDM
    PDM0,

    // QDEC
    QDEC0,
    QDEC1,

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
    PPI_CH16,
    PPI_CH17,
    PPI_CH18,
    PPI_CH19,
    PPI_CH20,
    PPI_CH21,
    PPI_CH22,
    PPI_CH23,
    PPI_CH24,
    PPI_CH25,
    PPI_CH26,
    PPI_CH27,
    PPI_CH28,
    PPI_CH29,
    PPI_CH30,
    PPI_CH31,

    PPI_GROUP0,
    PPI_GROUP1,
    PPI_GROUP2,
    PPI_GROUP3,
    PPI_GROUP4,
    PPI_GROUP5,

    // GPIO port 0
    P0_00,
    P0_01,
    #[cfg(feature = "nfc-pins-as-gpio")]
    P0_02,
    #[cfg(feature = "nfc-pins-as-gpio")]
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
}

#[cfg(feature = "nightly")]
impl_usb!(USBD, USBD, USBD);

impl_uarte!(SERIAL0, UARTE0, SERIAL0);
impl_uarte!(SERIAL1, UARTE1, SERIAL1);
impl_uarte!(SERIAL2, UARTE2, SERIAL2);
impl_uarte!(SERIAL3, UARTE3, SERIAL3);

impl_spim!(SERIAL0, SPIM0, SERIAL0);
impl_spim!(SERIAL1, SPIM1, SERIAL1);
impl_spim!(SERIAL2, SPIM2, SERIAL2);
impl_spim!(SERIAL3, SPIM3, SERIAL3);

impl_spis!(SERIAL0, SPIS0, SERIAL0);
impl_spis!(SERIAL1, SPIS1, SERIAL1);
impl_spis!(SERIAL2, SPIS2, SERIAL2);
impl_spis!(SERIAL3, SPIS3, SERIAL3);

impl_twim!(SERIAL0, TWIM0, SERIAL0);
impl_twim!(SERIAL1, TWIM1, SERIAL1);
impl_twim!(SERIAL2, TWIM2, SERIAL2);
impl_twim!(SERIAL3, TWIM3, SERIAL3);

impl_twis!(SERIAL0, TWIS0, SERIAL0);
impl_twis!(SERIAL1, TWIS1, SERIAL1);
impl_twis!(SERIAL2, TWIS2, SERIAL2);
impl_twis!(SERIAL3, TWIS3, SERIAL3);

impl_pwm!(PWM0, PWM0, PWM0);
impl_pwm!(PWM1, PWM1, PWM1);
impl_pwm!(PWM2, PWM2, PWM2);
impl_pwm!(PWM3, PWM3, PWM3);

impl_timer!(TIMER0, TIMER0, TIMER0);
impl_timer!(TIMER1, TIMER1, TIMER1);
impl_timer!(TIMER2, TIMER2, TIMER2);

impl_qspi!(QSPI, QSPI, QSPI);

impl_pdm!(PDM0, PDM0, PDM0);

impl_qdec!(QDEC0, QDEC0, QDEC0);
impl_qdec!(QDEC1, QDEC1, QDEC1);

impl_pin!(P0_00, 0, 0);
impl_pin!(P0_01, 0, 1);
#[cfg(feature = "nfc-pins-as-gpio")]
impl_pin!(P0_02, 0, 2);
#[cfg(feature = "nfc-pins-as-gpio")]
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
impl_ppi_channel!(PPI_CH16, 16 => configurable);
impl_ppi_channel!(PPI_CH17, 17 => configurable);
impl_ppi_channel!(PPI_CH18, 18 => configurable);
impl_ppi_channel!(PPI_CH19, 19 => configurable);
impl_ppi_channel!(PPI_CH20, 20 => configurable);
impl_ppi_channel!(PPI_CH21, 21 => configurable);
impl_ppi_channel!(PPI_CH22, 22 => configurable);
impl_ppi_channel!(PPI_CH23, 23 => configurable);
impl_ppi_channel!(PPI_CH24, 24 => configurable);
impl_ppi_channel!(PPI_CH25, 25 => configurable);
impl_ppi_channel!(PPI_CH26, 26 => configurable);
impl_ppi_channel!(PPI_CH27, 27 => configurable);
impl_ppi_channel!(PPI_CH28, 28 => configurable);
impl_ppi_channel!(PPI_CH29, 29 => configurable);
impl_ppi_channel!(PPI_CH30, 30 => configurable);
impl_ppi_channel!(PPI_CH31, 31 => configurable);

impl_saadc_input!(P0_13, ANALOG_INPUT0);
impl_saadc_input!(P0_14, ANALOG_INPUT1);
impl_saadc_input!(P0_15, ANALOG_INPUT2);
impl_saadc_input!(P0_16, ANALOG_INPUT3);
impl_saadc_input!(P0_17, ANALOG_INPUT4);
impl_saadc_input!(P0_18, ANALOG_INPUT5);
impl_saadc_input!(P0_19, ANALOG_INPUT6);
impl_saadc_input!(P0_20, ANALOG_INPUT7);

embassy_hal_internal::interrupt_mod!(
    FPU,
    CACHE,
    SPU,
    CLOCK_POWER,
    SERIAL0,
    SERIAL1,
    SPIM4,
    SERIAL2,
    SERIAL3,
    GPIOTE0,
    SAADC,
    TIMER0,
    TIMER1,
    TIMER2,
    RTC0,
    RTC1,
    WDT0,
    WDT1,
    COMP_LPCOMP,
    EGU0,
    EGU1,
    EGU2,
    EGU3,
    EGU4,
    EGU5,
    PWM0,
    PWM1,
    PWM2,
    PWM3,
    PDM0,
    I2S0,
    IPC,
    QSPI,
    NFCT,
    GPIOTE1,
    QDEC0,
    QDEC1,
    USBD,
    USBREGULATOR,
    KMU,
    CRYPTOCELL,
);
