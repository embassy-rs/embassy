#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

pub(crate) use stm32_metapac as pac;

// This must go FIRST so that all the other modules see its macros.
pub mod fmt;

// Utilities
pub mod interrupt;
pub mod time;

// Always-present hardware
pub mod gpio;
pub mod rcc;

// Sometimes-present hardware
#[cfg(adc)]
pub mod adc;
#[cfg(timer)]
pub mod clock;
#[cfg(dac)]
pub mod dac;
#[cfg(dma)]
pub mod dma;
#[cfg(all(eth, feature = "net"))]
pub mod eth;
#[cfg(exti_v1)]
pub mod exti;
#[cfg(i2c)]
pub mod i2c;
#[cfg(pwr)]
pub mod pwr;
#[cfg(rng)]
pub mod rng;
#[cfg(sdmmc)]
pub mod sdmmc;
#[cfg(spi)]
pub mod spi;
#[cfg(usart)]
pub mod usart;

// This must go last, so that it sees all the impl_foo! macros defined earlier.
mod generated {

    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]

    use crate::interrupt;

    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}
pub use embassy_macros::interrupt;
pub use generated::{peripherals, Peripherals};

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
        #[cfg(dma)]
        dma::init();
        #[cfg(exti_v1)]
        exti::init();
        rcc::init(config.rcc);
    }

    p
}
