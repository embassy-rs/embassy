#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[cfg(not(any(
    feature = "nrf51",
    feature = "nrf52805",
    feature = "nrf52810",
    feature = "nrf52811",
    feature = "nrf52820",
    feature = "nrf52832",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "nrf5340-app",
    feature = "nrf5340-net",
    feature = "nrf9160",
)))]
compile_error!("No chip feature activated. You must activate exactly one of the following features: nrf52810, nrf52811, nrf52832, nrf52833, nrf52840");

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;
pub(crate) mod util;

pub mod buffered_uarte;
pub mod gpio;
pub mod gpiote;
pub mod ppi;
#[cfg(feature = "nrf52840")]
pub mod qspi;
pub mod rtc;
#[cfg(not(feature = "nrf52820"))]
pub mod saadc;
pub mod spim;
pub mod system;
pub mod timer;
pub mod twim;
pub mod uarte;

// This mod MUST go last, so that it sees all the `impl_foo!` macros
#[cfg(feature = "nrf52805")]
#[path = "chips/nrf52805.rs"]
mod chip;
#[cfg(feature = "nrf52810")]
#[path = "chips/nrf52810.rs"]
mod chip;
#[cfg(feature = "nrf52811")]
#[path = "chips/nrf52811.rs"]
mod chip;
#[cfg(feature = "nrf52820")]
#[path = "chips/nrf52820.rs"]
mod chip;
#[cfg(feature = "nrf52832")]
#[path = "chips/nrf52832.rs"]
mod chip;
#[cfg(feature = "nrf52833")]
#[path = "chips/nrf52833.rs"]
mod chip;
#[cfg(feature = "nrf52840")]
#[path = "chips/nrf52840.rs"]
mod chip;

pub(crate) use chip::pac;
pub use chip::{peripherals, Peripherals};

pub mod interrupt {
    pub use crate::chip::irqs::*;
    pub use cortex_m::interrupt::{CriticalSection, Mutex};
    pub use embassy::interrupt::{declare, take, Interrupt};
    pub use embassy_extras::interrupt::Priority3 as Priority;
}
pub use embassy_macros::interrupt;
