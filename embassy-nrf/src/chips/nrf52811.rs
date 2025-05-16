pub use nrf_pac as pac;

/// The maximum buffer size that the EasyDMA can send/recv in one operation.
pub const EASY_DMA_SIZE: usize = (1 << 14) - 1;
pub const FORCE_COPY_BUFFER_SIZE: usize = 256;

pub const FLASH_SIZE: usize = 192 * 1024;

pub const RESET_PIN: u32 = 21;
pub const APPROTECT_MIN_BUILD_CODE: u8 = b'B';

embassy_hal_internal::peripherals! {
    // RTC
    RTC0,
    RTC1,

    // WDT
    WDT,

    // NVMC
    NVMC,

    // RNG
    RNG,

    // UARTE
    UARTE0,

    // SPI/TWI
    TWI0_SPI1,
    SPI0,

    // SAADC
    SAADC,

    // PWM
    PWM0,

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
    #[cfg(feature="reset-pin-as-gpio")]
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

    // TEMP
    TEMP,

    // QDEC
    QDEC,

    // PDM
    PDM,

    // Radio
    RADIO,

    // EGU
    EGU0,
    EGU1,
}

impl_uarte!(UARTE0, UARTE0, UARTE0);

impl_spim!(SPI0, SPIM0, SPI0);
impl_spim!(TWI0_SPI1, SPIM1, TWI0_SPI1);

impl_spis!(SPI0, SPIS0, SPI0);
impl_spis!(TWI0_SPI1, SPIS1, TWI0_SPI1);

impl_twim!(TWI0_SPI1, TWIM0, TWI0_SPI1);

impl_twis!(TWI0_SPI1, TWIS0, TWI0_SPI1);

impl_pwm!(PWM0, PWM0, PWM0);

impl_pdm!(PDM, PDM, PDM);

impl_qdec!(QDEC, QDEC, QDEC);

impl_rng!(RNG, RNG, RNG);

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
#[cfg(feature = "reset-pin-as-gpio")]
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
impl_ppi_channel!(PPI_CH16, 16 => configurable);
impl_ppi_channel!(PPI_CH17, 17 => configurable);
impl_ppi_channel!(PPI_CH18, 18 => configurable);
impl_ppi_channel!(PPI_CH19, 19 => configurable);
impl_ppi_channel!(PPI_CH20, 20 => static);
impl_ppi_channel!(PPI_CH21, 21 => static);
impl_ppi_channel!(PPI_CH22, 22 => static);
impl_ppi_channel!(PPI_CH23, 23 => static);
impl_ppi_channel!(PPI_CH24, 24 => static);
impl_ppi_channel!(PPI_CH25, 25 => static);
impl_ppi_channel!(PPI_CH26, 26 => static);
impl_ppi_channel!(PPI_CH27, 27 => static);
impl_ppi_channel!(PPI_CH28, 28 => static);
impl_ppi_channel!(PPI_CH29, 29 => static);
impl_ppi_channel!(PPI_CH30, 30 => static);
impl_ppi_channel!(PPI_CH31, 31 => static);

impl_saadc_input!(P0_02, ANALOG_INPUT0);
impl_saadc_input!(P0_03, ANALOG_INPUT1);
impl_saadc_input!(P0_04, ANALOG_INPUT2);
impl_saadc_input!(P0_05, ANALOG_INPUT3);
impl_saadc_input!(P0_28, ANALOG_INPUT4);
impl_saadc_input!(P0_29, ANALOG_INPUT5);
impl_saadc_input!(P0_30, ANALOG_INPUT6);
impl_saadc_input!(P0_31, ANALOG_INPUT7);

impl_radio!(RADIO, RADIO, RADIO);

impl_egu!(EGU0, EGU0, EGU0_SWI0);
impl_egu!(EGU1, EGU1, EGU1_SWI1);

impl_wdt!(WDT, WDT, WDT, 0);

embassy_hal_internal::interrupt_mod!(
    CLOCK_POWER,
    RADIO,
    UARTE0,
    TWI0_SPI1,
    SPI0,
    GPIOTE,
    SAADC,
    TIMER0,
    TIMER1,
    TIMER2,
    RTC0,
    TEMP,
    RNG,
    ECB,
    AAR_CCM,
    WDT,
    RTC1,
    QDEC,
    COMP,
    EGU0_SWI0,
    EGU1_SWI1,
    SWI2,
    SWI3,
    SWI4,
    SWI5,
    PWM0,
    PDM,
);
