#![no_std]
// Doc feature labels can be tested locally by running RUSTDOCFLAGS="--cfg=docsrs" cargo +nightly doc
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg_hide), doc(cfg_hide(doc, docsrs)))]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

// This must be declared early as well for
mod macros;

pub mod gpio;
pub mod timer;
pub mod uart;

/// Operating modes for peripherals.
pub mod mode {
    trait SealedMode {}

    /// Operating mode for a peripheral.
    #[allow(private_bounds)]
    pub trait Mode: SealedMode {}

    /// Blocking mode.
    pub struct Blocking;
    impl SealedMode for Blocking {}
    impl Mode for Blocking {}

    /// Async mode.
    pub struct Async;
    impl SealedMode for Async {}
    impl Mode for Async {}
}

#[cfg(feature = "_time-driver")]
mod time_driver;

// Interrupt group handlers.
#[cfg_attr(feature = "mspm0c110x", path = "int_group/c110x.rs")]
#[cfg_attr(feature = "mspm0g350x", path = "int_group/g350x.rs")]
#[cfg_attr(feature = "mspm0g351x", path = "int_group/g351x.rs")]
#[cfg_attr(feature = "mspm0l130x", path = "int_group/l130x.rs")]
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
pub(crate) use _generated::gpio_pincm;
pub use _generated::{peripherals, Peripherals};
pub use embassy_hal_internal::Peri;
#[cfg(feature = "unstable-pac")]
pub use mspm0_metapac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use mspm0_metapac as pac;

pub use crate::_generated::interrupt;

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
