#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

// This must go FIRST so that all the other modules see its macros.
pub mod fmt;

#[cfg(feature = "_timer")]
pub mod clock;
#[cfg(feature = "_dma")]
pub mod dma;
pub mod exti;
pub mod gpio;
#[cfg(feature = "_i2c")]
pub mod i2c;
pub mod pwr;
pub mod rcc;
#[cfg(feature = "_rng")]
pub mod rng;
#[cfg(feature = "_sdmmc")]
pub mod sdmmc;
#[cfg(feature = "_spi")]
pub mod spi;
#[cfg(feature = "_usart")]
pub mod usart;

// This must go LAST so that it sees the `impl_foo!` macros
#[cfg(feature = "pac")]
pub mod pac;

#[cfg(not(feature = "pac"))]
mod pac;
pub mod time;

pub use embassy_macros::interrupt;
pub use pac::{interrupt, peripherals, Peripherals};

// workaround for svd2rust-generated code using `use crate::generic::*;`
pub(crate) use pac::regs::generic;

#[non_exhaustive]
pub struct Config {
    rcc: rcc::Config,
}

impl Config {
    pub fn rcc(mut self, rcc: rcc::Config) -> Self {
        self.rcc = rcc;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rcc: Default::default(),
        }
    }
}

/// Initialize embassy.
pub fn init(config: Config) -> Peripherals {
    let p = Peripherals::take();

    unsafe {
        dma::init();
        pac::init_exti();
        rcc::init(config.rcc);
    }

    p
}
