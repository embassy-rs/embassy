#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[cfg(feature = "unstable-pac")]
pub use stm32_metapac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use stm32_metapac as pac;

// This must go FIRST so that all the other modules see its macros.
pub mod fmt;

// Utilities
pub mod interrupt;
pub mod time;

// Always-present hardware
pub mod dma;
pub mod gpio;
pub mod rcc;
#[cfg(feature = "_time-driver")]
mod time_driver;

// Sometimes-present hardware

#[cfg(adc)]
pub mod adc;
#[cfg(dac)]
pub mod dac;
#[cfg(dbgmcu)]
pub mod dbgmcu;
#[cfg(all(eth, feature = "net"))]
pub mod eth;
#[cfg(exti)]
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
    pub rcc: rcc::Config,
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
        gpio::init();
        dma::init();
        #[cfg(exti)]
        exti::init();

        rcc::init(config.rcc);

        // must be after rcc init
        #[cfg(feature = "_time-driver")]
        time_driver::init();
    }

    p
}
