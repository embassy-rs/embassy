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
pub mod gpiote;
pub mod interrupt;
#[cfg(feature = "52840")]
pub mod qspi;
pub mod rtc;
pub mod spim;
pub mod uarte;
