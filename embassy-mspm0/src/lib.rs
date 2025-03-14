#![no_std]
// Doc feature labels can be tested locally by running RUSTDOCFLAGS="--cfg=docsrs" cargo +nightly doc
#![cfg_attr(
    docsrs,
    feature(doc_auto_cfg, doc_cfg_hide),
    doc(cfg_hide(doc, docsrs))
)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod gpio;
pub mod timer;

#[cfg(feature = "_time-driver")]
mod time_driver;

// Interrupt group handlers.
#[cfg_attr(feature = "mspm0c110x", path = "int_group/c110x.rs")]
#[cfg_attr(feature = "mspm0g110x", path = "int_group/g110x.rs")]
#[cfg_attr(feature = "mspm0g150x", path = "int_group/g150x.rs")]
#[cfg_attr(feature = "mspm0g151x", path = "int_group/g151x.rs")]
#[cfg_attr(feature = "mspm0g310x", path = "int_group/g310x.rs")]
#[cfg_attr(feature = "mspm0g350x", path = "int_group/g350x.rs")]
#[cfg_attr(feature = "mspm0g351x", path = "int_group/g351x.rs")]
#[cfg_attr(feature = "mspm0l110x", path = "int_group/l110x.rs")]
#[cfg_attr(feature = "mspm0l122x", path = "int_group/l122x.rs")]
#[cfg_attr(feature = "mspm0l130x", path = "int_group/l130x.rs")]
#[cfg_attr(feature = "mspm0l134x", path = "int_group/l134x.rs")]
#[cfg_attr(feature = "mspm0l222x", path = "int_group/l222x.rs")]
mod int_group;

pub(crate) mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]
    #![allow(missing_docs)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}

// Reexports
pub use _generated::{peripherals, Peripherals};
pub use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
#[cfg(feature = "unstable-pac")]
pub use mspm0_metapac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use mspm0_metapac as pac;

pub use crate::_generated::interrupt;
pub(crate) use _generated::gpio_pincm;


/// `embassy-mspm0` global configuration.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    // TODO
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // TODO
        }
    }
}

pub fn init(_config: Config) -> Peripherals {
    critical_section::with(|cs| {
        let peripherals = Peripherals::take_with_cs(cs);

        // TODO: Further clock configuration

        pac::SYSCTL.mclkcfg().modify(|w| {
            // Enable MFCLK
            w.set_usemftick(true);
            // MDIV must be disabled if MFCLK is enabled.
            w.set_mdiv(0);
        });

        // Enable MFCLK for peripheral use
        //
        // TODO: Optional?
        pac::SYSCTL.genclken().modify(|w| {
            w.set_mfpclken(true);
        });

        pac::SYSCTL.borthreshold().modify(|w| {
            w.set_level(0);
        });

        gpio::init(pac::GPIOA);
        #[cfg(gpio_pb)]
        gpio::init(pac::GPIOB);
        #[cfg(gpio_pc)]
        gpio::init(pac::GPIOC);

        _generated::enable_group_interrupts(cs);

        #[cfg(feature = "mspm0c110x")]
        unsafe {
            use crate::_generated::interrupt::typelevel::Interrupt;
            crate::interrupt::typelevel::GPIOA::enable();
        }

        #[cfg(feature = "_time-driver")]
        time_driver::init(cs);

        peripherals
    })
}
