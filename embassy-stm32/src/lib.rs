#![no_std]
#![cfg_attr(
    feature = "nightly",
    feature(generic_associated_types, type_alias_impl_trait)
)]

// This must go FIRST so that all the other modules see its macros.
pub mod fmt;
include!(concat!(env!("OUT_DIR"), "/_macros.rs"));

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
#[cfg(eth)]
pub mod eth;
#[cfg(feature = "exti")]
pub mod exti;
#[cfg(fmc)]
pub mod fmc;
#[cfg(i2c)]
pub mod i2c;

#[cfg(crc)]
pub mod crc;
#[cfg(any(
    flash_l0, flash_l1, flash_wl, flash_wb, flash_l4, flash_f3, flash_f7, flash_h7
))]
pub mod flash;
pub mod pwm;
#[cfg(rng)]
pub mod rng;
#[cfg(sdmmc)]
pub mod sdmmc;
#[cfg(spi)]
pub mod spi;
#[cfg(usart)]
pub mod usart;
#[cfg(usb)]
pub mod usb;
#[cfg(any(otgfs, otghs))]
pub mod usb_otg;

#[cfg(feature = "subghz")]
pub mod subghz;

// This must go last, so that it sees all the impl_foo! macros defined earlier.
pub(crate) mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}

// Reexports
pub use _generated::{peripherals, Peripherals};
pub use embassy_cortex_m::executor;
pub use embassy_hal_common::{unborrow, Unborrow};
pub use embassy_macros::cortex_m_interrupt as interrupt;

#[cfg(feature = "unstable-pac")]
pub use stm32_metapac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use stm32_metapac as pac;

#[non_exhaustive]
pub struct Config {
    pub rcc: rcc::Config,
    #[cfg(dbgmcu)]
    pub enable_debug_during_sleep: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rcc: Default::default(),
            #[cfg(dbgmcu)]
            enable_debug_during_sleep: true,
        }
    }
}

/// Initialize embassy.
pub fn init(config: Config) -> Peripherals {
    let p = Peripherals::take();

    unsafe {
        #[cfg(dbgmcu)]
        if config.enable_debug_during_sleep {
            crate::pac::DBGMCU.cr().modify(|cr| {
                #[cfg(any(dbgmcu_f0, dbgmcu_g0, dbgmcu_u5))]
                {
                    cr.set_dbg_stop(true);
                    cr.set_dbg_standby(true);
                }
                #[cfg(any(
                    dbgmcu_f1, dbgmcu_f2, dbgmcu_f3, dbgmcu_f4, dbgmcu_f7, dbgmcu_g4, dbgmcu_f7,
                    dbgmcu_l0, dbgmcu_l1, dbgmcu_l4, dbgmcu_wb, dbgmcu_wl
                ))]
                {
                    cr.set_dbg_sleep(true);
                    cr.set_dbg_stop(true);
                    cr.set_dbg_standby(true);
                }
                #[cfg(dbgmcu_h7)]
                {
                    cr.set_d1dbgcken(true);
                    cr.set_d3dbgcken(true);
                    cr.set_dbgsleep_d1(true);
                    cr.set_dbgstby_d1(true);
                    cr.set_dbgstop_d1(true);
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
