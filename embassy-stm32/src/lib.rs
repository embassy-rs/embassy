#![no_std]
#![cfg_attr(
    feature = "nightly",
    feature(generic_associated_types, type_alias_impl_trait)
)]

#[cfg(feature = "unstable-pac")]
pub use stm32_metapac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use stm32_metapac as pac;

// This must go FIRST so that all the other modules see its macros.
pub mod fmt;

// Utilities
pub mod interrupt;
pub mod time;
mod traits;

// Always-present hardware
pub mod dma;
pub mod gpio;
pub mod rcc;
#[cfg(feature = "_time-driver")]
mod time_driver;
pub mod timer;

// Sometimes-present hardware

#[cfg(adc)]
pub mod adc;
#[cfg(can)]
pub mod can;
#[cfg(dac)]
pub mod dac;
#[cfg(dcmi)]
pub mod dcmi;
#[cfg(all(eth, feature = "net"))]
pub mod eth;
#[cfg(feature = "exti")]
pub mod exti;
#[cfg(fmc)]
pub mod fmc;
#[cfg(i2c)]
pub mod i2c;

#[cfg(crc)]
pub mod crc;
pub mod pwm;
#[cfg(rng)]
pub mod rng;
#[cfg(sdmmc)]
pub mod sdmmc;
#[cfg(spi)]
pub mod spi;
#[cfg(usart)]
pub mod usart;
#[cfg(feature = "usb-otg")]
pub mod usb_otg;

#[cfg(feature = "subghz")]
pub mod subghz;

// This must go last, so that it sees all the impl_foo! macros defined earlier.
mod generated {

    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}
pub use embassy_macros::interrupt;
pub use generated::{peripherals, Peripherals};

#[non_exhaustive]
pub struct Config {
    pub rcc: rcc::Config,
    pub enable_debug_during_sleep: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rcc: Default::default(),
            enable_debug_during_sleep: true,
        }
    }
}

/// Initialize embassy.
pub fn init(config: Config) -> Peripherals {
    let p = Peripherals::take();

    unsafe {
        if config.enable_debug_during_sleep {
            crate::pac::DBGMCU.cr().modify(|cr| {
                crate::pac::dbgmcu! {
                    (cr, $fn_name:ident) => {
                        cr.$fn_name(true);
                    };
                }
            });
        }

        gpio::init();
        dma::init();
        #[cfg(feature = "exti")]
        exti::init();

        rcc::init(config.rcc);

        // must be after rcc init
        #[cfg(feature = "_time-driver")]
        time_driver::init();
    }

    p
}
