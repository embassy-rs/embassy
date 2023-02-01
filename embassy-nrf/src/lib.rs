#![no_std]
#![cfg_attr(
    feature = "nightly",
    feature(type_alias_impl_trait, async_fn_in_trait, impl_trait_projections)
)]
#![cfg_attr(feature = "nightly", allow(incomplete_features))]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(not(any(
    feature = "nrf51",
    feature = "nrf52805",
    feature = "nrf52810",
    feature = "nrf52811",
    feature = "nrf52820",
    feature = "nrf52832",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "nrf5340-app-s",
    feature = "nrf5340-app-ns",
    feature = "nrf5340-net",
    feature = "nrf9160-s",
    feature = "nrf9160-ns",
)))]
compile_error!("No chip feature activated. You must activate exactly one of the following features: nrf52810, nrf52811, nrf52832, nrf52833, nrf52840");

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;
pub(crate) mod util;

#[cfg(feature = "_time-driver")]
mod time_driver;

#[cfg(feature = "nightly")]
pub mod buffered_uarte;
pub mod gpio;
#[cfg(feature = "gpiote")]
pub mod gpiote;
#[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
pub mod i2s;
pub mod nvmc;
#[cfg(any(
    feature = "nrf52810",
    feature = "nrf52811",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "_nrf9160"
))]
pub mod pdm;
pub mod ppi;
#[cfg(not(any(feature = "nrf52805", feature = "nrf52820", feature = "_nrf5340-net")))]
pub mod pwm;
#[cfg(not(any(feature = "nrf51", feature = "_nrf9160", feature = "_nrf5340")))]
pub mod qdec;
#[cfg(feature = "nrf52840")]
pub mod qspi;
#[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
pub mod rng;
#[cfg(not(any(feature = "nrf52820", feature = "_nrf5340-net")))]
pub mod saadc;
pub mod spim;
pub mod spis;
#[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
pub mod temp;
pub mod timer;
pub mod twim;
pub mod twis;
pub mod uarte;
#[cfg(any(
    feature = "_nrf5340-app",
    feature = "nrf52820",
    feature = "nrf52833",
    feature = "nrf52840"
))]
#[cfg(feature = "nightly")]
pub mod usb;
#[cfg(not(feature = "_nrf5340"))]
pub mod wdt;

// This mod MUST go last, so that it sees all the `impl_foo!` macros
#[cfg_attr(feature = "nrf52805", path = "chips/nrf52805.rs")]
#[cfg_attr(feature = "nrf52810", path = "chips/nrf52810.rs")]
#[cfg_attr(feature = "nrf52811", path = "chips/nrf52811.rs")]
#[cfg_attr(feature = "nrf52820", path = "chips/nrf52820.rs")]
#[cfg_attr(feature = "nrf52832", path = "chips/nrf52832.rs")]
#[cfg_attr(feature = "nrf52833", path = "chips/nrf52833.rs")]
#[cfg_attr(feature = "nrf52840", path = "chips/nrf52840.rs")]
#[cfg_attr(feature = "_nrf5340-app", path = "chips/nrf5340_app.rs")]
#[cfg_attr(feature = "_nrf5340-net", path = "chips/nrf5340_net.rs")]
#[cfg_attr(feature = "_nrf9160", path = "chips/nrf9160.rs")]
mod chip;

pub use chip::EASY_DMA_SIZE;

pub mod interrupt {
    //! nRF interrupts for cortex-m devices.
    pub use cortex_m::interrupt::{CriticalSection, Mutex};
    pub use embassy_cortex_m::interrupt::*;

    pub use crate::chip::irqs::*;
}

// Reexports

#[cfg(feature = "unstable-pac")]
pub use chip::pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use chip::pac;
pub use chip::{peripherals, Peripherals};
pub use embassy_cortex_m::executor;
pub use embassy_cortex_m::interrupt::_export::interrupt;
pub use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};

pub mod config {
    //! Configuration options used when initializing the HAL.

    /// High frequency clock source.
    pub enum HfclkSource {
        /// Internal source
        Internal,
        /// External source from xtal.
        ExternalXtal,
    }

    /// Low frequency clock source
    pub enum LfclkSource {
        /// Internal RC oscillator
        InternalRC,
        /// Synthesized from the high frequency clock source.
        #[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
        Synthesized,
        /// External source from xtal.
        ExternalXtal,
        /// External source from xtal with low swing applied.
        #[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
        ExternalLowSwing,
        /// External source from xtal with full swing applied.
        #[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
        ExternalFullSwing,
    }

    /// Configuration for peripherals. Default configuration should work on any nRF chip.
    #[non_exhaustive]
    pub struct Config {
        /// High frequency clock source.
        pub hfclk_source: HfclkSource,
        /// Low frequency clock source.
        pub lfclk_source: LfclkSource,
        /// GPIOTE interrupt priority. Should be lower priority than softdevice if used.
        #[cfg(feature = "gpiote")]
        pub gpiote_interrupt_priority: crate::interrupt::Priority,
        /// Time driver interrupt priority. Should be lower priority than softdevice if used.
        #[cfg(feature = "_time-driver")]
        pub time_interrupt_priority: crate::interrupt::Priority,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                // There are hobby nrf52 boards out there without external XTALs...
                // Default everything to internal so it Just Works. User can enable external
                // xtals if they know they have them.
                hfclk_source: HfclkSource::Internal,
                lfclk_source: LfclkSource::InternalRC,
                #[cfg(feature = "gpiote")]
                gpiote_interrupt_priority: crate::interrupt::Priority::P0,
                #[cfg(feature = "_time-driver")]
                time_interrupt_priority: crate::interrupt::Priority::P0,
            }
        }
    }
}

/// Initialize peripherals with the provided configuration. This should only be called once at startup.
pub fn init(config: config::Config) -> Peripherals {
    // Do this first, so that it panics if user is calling `init` a second time
    // before doing anything important.
    let peripherals = Peripherals::take();

    let r = unsafe { &*pac::CLOCK::ptr() };

    // Start HFCLK.
    match config.hfclk_source {
        config::HfclkSource::Internal => {}
        config::HfclkSource::ExternalXtal => {
            // Datasheet says this is likely to take 0.36ms
            r.events_hfclkstarted.write(|w| unsafe { w.bits(0) });
            r.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });
            while r.events_hfclkstarted.read().bits() == 0 {}
        }
    }

    // Configure LFCLK.
    #[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
    match config.lfclk_source {
        config::LfclkSource::InternalRC => r.lfclksrc.write(|w| w.src().rc()),
        config::LfclkSource::Synthesized => r.lfclksrc.write(|w| w.src().synth()),

        config::LfclkSource::ExternalXtal => r.lfclksrc.write(|w| w.src().xtal()),

        config::LfclkSource::ExternalLowSwing => r.lfclksrc.write(|w| {
            w.src().xtal();
            w.external().enabled();
            w.bypass().disabled();
            w
        }),
        config::LfclkSource::ExternalFullSwing => r.lfclksrc.write(|w| {
            w.src().xtal();
            w.external().enabled();
            w.bypass().enabled();
            w
        }),
    }
    #[cfg(feature = "_nrf9160")]
    match config.lfclk_source {
        config::LfclkSource::InternalRC => r.lfclksrc.write(|w| w.src().lfrc()),
        config::LfclkSource::ExternalXtal => r.lfclksrc.write(|w| w.src().lfxo()),
    }

    // Start LFCLK.
    // Datasheet says this could take 100us from synth source
    // 600us from rc source, 0.25s from an external source.
    r.events_lfclkstarted.write(|w| unsafe { w.bits(0) });
    r.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
    while r.events_lfclkstarted.read().bits() == 0 {}

    // Init GPIOTE
    #[cfg(feature = "gpiote")]
    gpiote::init(config.gpiote_interrupt_priority);

    // init RTC time driver
    #[cfg(feature = "_time-driver")]
    time_driver::init(config.time_interrupt_priority);

    // Disable UARTE (enabled by default for some reason)
    #[cfg(feature = "_nrf9160")]
    unsafe {
        (*pac::UARTE0::ptr()).enable.write(|w| w.enable().disabled());
        (*pac::UARTE1::ptr()).enable.write(|w| w.enable().disabled());
    }

    peripherals
}
