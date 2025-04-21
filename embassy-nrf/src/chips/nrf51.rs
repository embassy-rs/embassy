pub use nrf_pac as pac;

/// The maximum buffer size that the EasyDMA can send/recv in one operation.
pub const EASY_DMA_SIZE: usize = (1 << 14) - 1;

pub const FLASH_SIZE: usize = 128 * 1024;

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
    UART0,

    // SPI/TWI
    TWI0,
    SPI0,

    // ADC
    ADC,

    // TIMER
    TIMER0,
    TIMER1,
    TIMER2,

    // GPIOTE
    GPIOTE_CH0,
    GPIOTE_CH1,
    GPIOTE_CH2,
    GPIOTE_CH3,

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

    // TEMP
    TEMP,

    // Radio
    RADIO,
}

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

impl_radio!(RADIO, RADIO, RADIO);

impl_wdt!(WDT, WDT, WDT, 0);

embassy_hal_internal::interrupt_mod!(
    CLOCK_POWER,
    RADIO,
    UART0,
    TWISPI0,
    TWISPI1,
    GPIOTE,
    ADC,
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
    LPCOMP,
    SWI0,
    SWI1,
    SWI2,
    SWI3,
    SWI4,
    SWI5,
);
