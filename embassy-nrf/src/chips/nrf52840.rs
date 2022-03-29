pub use nrf52840_pac as pac;

/// The maximum buffer size that the EasyDMA can send/recv in one operation.
pub const EASY_DMA_SIZE: usize = (1 << 16) - 1;
pub const FORCE_COPY_BUFFER_SIZE: usize = 512;

pub const FLASH_SIZE: usize = 1024 * 1024;

embassy_hal_common::peripherals! {
    // USB
    USBD,

    // RTC
    RTC0,
    RTC1,
    RTC2,

    // WDT
    WDT,

    // NVMC
    NVMC,

    // RNG
    RNG,

    // QSPI
    QSPI,

    // UARTE
    UARTE0,
    UARTE1,

    // SPI/TWI
    TWISPI0,
    TWISPI1,
    SPI2,
    SPI3,

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
    TIMER3,
    TIMER4,

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

    // TEMP
    TEMP,
}

#[cfg(feature = "nightly")]
impl_usb!(USBD, USBD, USBD);

impl_uarte!(UARTE0, UARTE0, UARTE0_UART0);
impl_uarte!(UARTE1, UARTE1, UARTE1);

impl_spim!(TWISPI0, SPIM0, SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
impl_spim!(TWISPI1, SPIM1, SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);
impl_spim!(SPI2, SPIM2, SPIM2_SPIS2_SPI2);
impl_spim!(SPI3, SPIM3, SPIM3);

impl_twim!(TWISPI0, TWIM0, SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
impl_twim!(TWISPI1, TWIM1, SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);

impl_pwm!(PWM0, PWM0, PWM0);
impl_pwm!(PWM1, PWM1, PWM1);
impl_pwm!(PWM2, PWM2, PWM2);
impl_pwm!(PWM3, PWM3, PWM3);

impl_timer!(TIMER0, TIMER0, TIMER0);
impl_timer!(TIMER1, TIMER1, TIMER1);
impl_timer!(TIMER2, TIMER2, TIMER2);
impl_timer!(TIMER3, TIMER3, TIMER3, extended);
impl_timer!(TIMER4, TIMER4, TIMER4, extended);

impl_qspi!(QSPI, QSPI, QSPI);

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

impl_saadc_input!(P0_02, ANALOGINPUT0);
impl_saadc_input!(P0_03, ANALOGINPUT1);
impl_saadc_input!(P0_04, ANALOGINPUT2);
impl_saadc_input!(P0_05, ANALOGINPUT3);
impl_saadc_input!(P0_28, ANALOGINPUT4);
impl_saadc_input!(P0_29, ANALOGINPUT5);
impl_saadc_input!(P0_30, ANALOGINPUT6);
impl_saadc_input!(P0_31, ANALOGINPUT7);

pub mod irqs {
    use crate::pac::Interrupt as InterruptEnum;
    use embassy_macros::interrupt_declare as declare;

    declare!(POWER_CLOCK);
    declare!(RADIO);
    declare!(UARTE0_UART0);
    declare!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    declare!(SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);
    declare!(NFCT);
    declare!(GPIOTE);
    declare!(SAADC);
    declare!(TIMER0);
    declare!(TIMER1);
    declare!(TIMER2);
    declare!(RTC0);
    declare!(TEMP);
    declare!(RNG);
    declare!(ECB);
    declare!(CCM_AAR);
    declare!(WDT);
    declare!(RTC1);
    declare!(QDEC);
    declare!(COMP_LPCOMP);
    declare!(SWI0_EGU0);
    declare!(SWI1_EGU1);
    declare!(SWI2_EGU2);
    declare!(SWI3_EGU3);
    declare!(SWI4_EGU4);
    declare!(SWI5_EGU5);
    declare!(TIMER3);
    declare!(TIMER4);
    declare!(PWM0);
    declare!(PDM);
    declare!(MWU);
    declare!(PWM1);
    declare!(PWM2);
    declare!(SPIM2_SPIS2_SPI2);
    declare!(RTC2);
    declare!(I2S);
    declare!(FPU);
    declare!(USBD);
    declare!(UARTE1);
    declare!(QSPI);
    declare!(CRYPTOCELL);
    declare!(PWM3);
    declare!(SPIM3);
}
