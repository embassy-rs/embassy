#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

// This must go FIRST so that all the other modules see its macros.
pub mod fmt;

use embassy::interrupt::{Interrupt, InterruptExt};

#[cfg(feature = "_dma")]
pub mod dma;
pub mod exti;
pub mod gpio;
#[cfg(feature = "_rng")]
pub mod rng;
#[cfg(feature = "_spi")]
pub mod spi;
#[cfg(feature = "_usart")]
pub mod usart;

// This must go LAST so that it sees the `impl_foo!` macros
mod pac;
pub mod time;

pub use embassy_macros::interrupt;
pub use pac::{interrupt, peripherals, Peripherals};

// workaround for svd2rust-generated code using `use crate::generic::*;`
pub(crate) use pac::generic;

#[non_exhaustive]
pub struct Config {
    _private: (),
}

impl Default for Config {
    fn default() -> Self {
        Self { _private: () }
    }
}

/// Initialize embassy.
pub fn init(_config: Config) -> Peripherals {
    let p = Peripherals::take();

    unsafe {
        interrupt::EXTI0::steal().enable();
        interrupt::EXTI1::steal().enable();
        interrupt::EXTI2::steal().enable();
        interrupt::EXTI3::steal().enable();
        interrupt::EXTI4::steal().enable();
        interrupt::EXTI9_5::steal().enable();
        interrupt::EXTI15_10::steal().enable();
    }

    p
}
