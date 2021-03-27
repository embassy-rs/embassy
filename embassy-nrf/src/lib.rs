#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[cfg(not(any(
    feature = "52810",
    feature = "52811",
    feature = "52832",
    feature = "52833",
    feature = "52840",
)))]
compile_error!("No chip feature activated. You must activate exactly one of the following features: 52810, 52811, 52832, 52833, 52840");

#[cfg(any(
    all(feature = "52810", feature = "52811"),
    all(feature = "52810", feature = "52832"),
    all(feature = "52810", feature = "52833"),
    all(feature = "52810", feature = "52840"),
    all(feature = "52811", feature = "52832"),
    all(feature = "52811", feature = "52833"),
    all(feature = "52811", feature = "52840"),
    all(feature = "52832", feature = "52833"),
    all(feature = "52832", feature = "52840"),
    all(feature = "52833", feature = "52840"),
))]
compile_error!("Multile chip features activated. You must activate exactly one of the following features: 52810, 52811, 52832, 52833, 52840");

#[cfg(feature = "52810")]
pub use nrf52810_pac as pac;
#[cfg(feature = "52811")]
pub use nrf52811_pac as pac;
#[cfg(feature = "52832")]
pub use nrf52832_pac as pac;
#[cfg(feature = "52833")]
pub use nrf52833_pac as pac;
#[cfg(feature = "52840")]
pub use nrf52840_pac as pac;

#[cfg(feature = "52810")]
pub use nrf52810_hal as hal;
#[cfg(feature = "52811")]
pub use nrf52811_hal as hal;
#[cfg(feature = "52832")]
pub use nrf52832_hal as hal;
#[cfg(feature = "52833")]
pub use nrf52833_hal as hal;
#[cfg(feature = "52840")]
pub use nrf52840_hal as hal;

/// Length of Nordic EasyDMA differs for MCUs
#[cfg(any(
    feature = "52810",
    feature = "52811",
    feature = "52832",
    feature = "51"
))]
pub mod target_constants {
    // NRF52832 8 bits1..0xFF
    pub const EASY_DMA_SIZE: usize = 255;
    // Easy DMA can only read from data ram
    pub const SRAM_LOWER: usize = 0x2000_0000;
    pub const SRAM_UPPER: usize = 0x3000_0000;
}
#[cfg(any(feature = "52840", feature = "52833", feature = "9160"))]
pub mod target_constants {
    // NRF52840 and NRF9160 16 bits 1..0xFFFF
    pub const EASY_DMA_SIZE: usize = 65535;
    // Limits for Easy DMA - it can only read from data ram
    pub const SRAM_LOWER: usize = 0x2000_0000;
    pub const SRAM_UPPER: usize = 0x3000_0000;
}

/// Does this slice reside entirely within RAM?
pub(crate) fn slice_in_ram(slice: &[u8]) -> bool {
    let ptr = slice.as_ptr() as usize;
    ptr >= target_constants::SRAM_LOWER && (ptr + slice.len()) < target_constants::SRAM_UPPER
}

/// Return an error if slice is not in RAM.
#[cfg(not(feature = "51"))]
pub(crate) fn slice_in_ram_or<T>(slice: &[u8], err: T) -> Result<(), T> {
    if slice.len() == 0 || slice_in_ram(slice) {
        Ok(())
    } else {
        Err(err)
    }
}

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod buffered_uarte;
pub mod gpio;
pub mod gpiote;
pub mod interrupt;
pub mod ppi;
#[cfg(feature = "52840")]
pub mod qspi;
pub mod rtc;
pub mod saadc;
pub mod spim;
pub mod uarte;

embassy_extras::peripherals! {
    // RTC
    RTC0,
    RTC1,
    #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
    RTC2,

    // QSPI
    #[cfg(feature = "52840")]
    QSPI,

    // UARTE
    UARTE0,
    #[cfg(any(feature = "52833", feature = "52840", feature = "9160"))]
    UARTE1,

    // SPIM
    // TODO this is actually shared with SPI, SPIM, SPIS, TWI, TWIS, TWIS.
    // When they're all implemented, they should be only one peripheral here.
    SPIM0,
    #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
    SPIM1,
    #[cfg(any(feature = "52832", feature = "52833", feature = "52840"))]
    SPIM2,
    #[cfg(any(feature = "52833", feature = "52840"))]
    SPIM3,

    // SAADC
    SAADC,

    // GPIOTE
    GPIOTE,
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
    #[cfg(not(feature = "51"))]
    PPI_CH16,
    #[cfg(not(feature = "51"))]
    PPI_CH17,
    #[cfg(not(feature = "51"))]
    PPI_CH18,
    #[cfg(not(feature = "51"))]
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
    #[cfg(not(feature = "51"))]
    PPI_GROUP4,
    #[cfg(not(feature = "51"))]
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
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_00,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_01,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_02,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_03,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_04,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_05,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_06,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_07,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_08,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_09,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_10,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_11,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_12,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_13,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_14,
    #[cfg(any(feature = "52833", feature = "52840"))]
    P1_15,
}
