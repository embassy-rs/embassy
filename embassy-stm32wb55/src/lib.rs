#![no_std]
#![feature(asm)]
#![feature(type_alias_impl_trait)]

#[cfg(not(any(feature = "55",)))]
compile_error!(
    "No chip feature activated. You must activate exactly one of the following features: 55"
);

#[cfg(feature = "55")]
pub use stm32wb_hal as hal;

#[cfg(feature = "55")]
pub use stm32wb_hal::pac;

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod ble;
pub mod interrupt;
pub use cortex_m_rt::interrupt;
