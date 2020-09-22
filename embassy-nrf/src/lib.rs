#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(type_alias_impl_trait)]

#[cfg(not(any(
    feature = "nrf52810",
    feature = "nrf52811",
    feature = "nrf52832",
    feature = "nrf52833",
    feature = "nrf52840",
)))]
compile_error!("No chip feature activated. You must activate exactly one of the following features: nrf52810, nrf52811, nrf52832, nrf52833, nrf52840");

#[cfg(any(
    all(feature = "nrf52810", feature = "nrf52811"),
    all(feature = "nrf52810", feature = "nrf52832"),
    all(feature = "nrf52810", feature = "nrf52833"),
    all(feature = "nrf52810", feature = "nrf52840"),
    all(feature = "nrf52811", feature = "nrf52832"),
    all(feature = "nrf52811", feature = "nrf52833"),
    all(feature = "nrf52811", feature = "nrf52840"),
    all(feature = "nrf52832", feature = "nrf52833"),
    all(feature = "nrf52832", feature = "nrf52840"),
    all(feature = "nrf52833", feature = "nrf52840"),
))]
compile_error!("Multile chip features activated. You must activate exactly one of the following features: nrf52810, nrf52811, nrf52832, nrf52833, nrf52840");

#[cfg(feature = "nrf52810")]
pub use nrf52810_pac as pac;
#[cfg(feature = "nrf52811")]
pub use nrf52811_pac as pac;
#[cfg(feature = "nrf52832")]
pub use nrf52832_pac as pac;
#[cfg(feature = "nrf52833")]
pub use nrf52833_pac as pac;
#[cfg(feature = "nrf52840")]
pub use nrf52840_pac as pac;

pub mod interrupt;
pub mod qspi;
pub mod uarte;
pub use cortex_m_rt::interrupt;
