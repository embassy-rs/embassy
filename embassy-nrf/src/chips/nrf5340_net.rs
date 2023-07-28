/// Peripheral Access Crate
#[allow(unused_imports)]
#[rustfmt::skip]
pub mod pac {
    // The nRF5340 has a secure and non-secure (NS) mode.
    // To avoid cfg spam, we remove _ns or _s suffixes here.

    pub use nrf5340_net_pac::NVIC_PRIO_BITS;

    #[doc(no_inline)]
    pub use nrf5340_net_pac::{
        interrupt,
        Interrupt,
        Peripherals,

        aar_ns as aar,
        acl_ns as acl,
        appmutex_ns as appmutex,
        ccm_ns as ccm,
        clock_ns as clock,
        cti_ns as cti,
        ctrlap_ns as ctrlap,
        dcnf_ns as dcnf,
        dppic_ns as dppic,
        ecb_ns as ecb,
        egu0_ns as egu0,
        ficr_ns as ficr,
        gpiote_ns as gpiote,
        ipc_ns as ipc,
        nvmc_ns as nvmc,
        p0_ns as p0,
        power_ns as power,
        radio_ns as radio,
        reset_ns as reset,
        rng_ns as rng,
        rtc0_ns as rtc0,
        spim0_ns as spim0,
        spis0_ns as spis0,
        swi0_ns as swi0,
        temp_ns as temp,
        timer0_ns as timer0,
        twim0_ns as twim0,
        twis0_ns as twis0,
        uarte0_ns as uarte0,
        uicr_ns as uicr,
        vmc_ns as vmc,
        vreqctrl_ns as vreqctrl,
        wdt_ns as wdt,

        AAR_NS as AAR,
        ACL_NS as ACL,
        APPMUTEX_NS as APPMUTEX,
        APPMUTEX_S as APPMUTEX_S,
        CBP as CBP,
        CCM_NS as CCM,
        CLOCK_NS as CLOCK,
        CPUID as CPUID,
        CTI_NS as CTI,
        CTRLAP_NS as CTRLAP,
        DCB as DCB,
        DCNF_NS as DCNF,
        DPPIC_NS as DPPIC,
        DWT as DWT,
        ECB_NS as ECB,
        EGU0_NS as EGU0,
        FICR_NS as FICR,
        FPB as FPB,
        GPIOTE_NS as GPIOTE,
        IPC_NS as IPC,
        ITM as ITM,
        MPU as MPU,
        NVIC as NVIC,
        NVMC_NS as NVMC,
        P0_NS as P0,
        P1_NS as P1,
        POWER_NS as POWER,
        RADIO_NS as RADIO,
        RESET_NS as RESET,
        RNG_NS as RNG,
        RTC0_NS as RTC0,
        RTC1_NS as RTC1,
        SCB as SCB,
        SPIM0_NS as SPIM0,
        SPIS0_NS as SPIS0,
        SWI0_NS as SWI0,
        SWI1_NS as SWI1,
        SWI2_NS as SWI2,
        SWI3_NS as SWI3,
        SYST as SYST,
        TEMP_NS as TEMP,
        TIMER0_NS as TIMER0,
        TIMER1_NS as TIMER1,
        TIMER2_NS as TIMER2,
        TPIU as TPIU,
        TWIM0_NS as TWIM0,
        TWIS0_NS as TWIS0,
        UARTE0_NS as UARTE0,
        UICR_NS as UICR,
        VMC_NS as VMC,
        VREQCTRL_NS as VREQCTRL,
        WDT_NS as WDT,
    };
    
}

/// The maximum buffer size that the EasyDMA can send/recv in one operation.
pub const EASY_DMA_SIZE: usize = (1 << 16) - 1;
pub const FORCE_COPY_BUFFER_SIZE: usize = 1024;

pub const FLASH_SIZE: usize = 256 * 1024;

embassy_hal_internal::peripherals! {
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

    // RNG
    RNG,

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

impl_uarte!(SERIAL0, UARTE0, SERIAL0);
impl_spim!(SERIAL0, SPIM0, SERIAL0);
impl_spis!(SERIAL0, SPIS0, SERIAL0);
impl_twim!(SERIAL0, TWIM0, SERIAL0);
impl_twis!(SERIAL0, TWIS0, SERIAL0);

impl_timer!(TIMER0, TIMER0, TIMER0);
impl_timer!(TIMER1, TIMER1, TIMER1);
impl_timer!(TIMER2, TIMER2, TIMER2);

impl_rng!(RNG, RNG, RNG);

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

embassy_hal_internal::interrupt_mod!(
    CLOCK_POWER,
    RADIO,
    RNG,
    GPIOTE,
    WDT,
    TIMER0,
    ECB,
    AAR_CCM,
    TEMP,
    RTC0,
    IPC,
    SERIAL0,
    EGU0,
    RTC1,
    TIMER1,
    TIMER2,
    SWI0,
    SWI1,
    SWI2,
    SWI3,
);
