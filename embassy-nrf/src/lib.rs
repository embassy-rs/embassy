#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(type_alias_impl_trait)]

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

pub mod gpiote;
pub mod interrupt;
pub mod qspi;
pub mod uarte;

pub use cortex_m_rt::interrupt;
