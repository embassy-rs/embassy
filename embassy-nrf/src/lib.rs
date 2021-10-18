#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(type_alias_impl_trait)]

#[cfg(not(any(
    feature = "nrf51",
    feature = "nrf52805",
    feature = "nrf52810",
    feature = "nrf52811",
    feature = "nrf52820",
    feature = "nrf52832",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "nrf5340-app",
    feature = "nrf5340-net",
    feature = "nrf9160",
)))]
compile_error!("No chip feature activated. You must activate exactly one of the following features: nrf52810, nrf52811, nrf52832, nrf52833, nrf52840");

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;
pub(crate) mod util;

#[cfg(feature = "_time-driver")]
mod time_driver;

pub mod buffered_uarte;
pub mod gpio;
#[cfg(feature = "gpiote")]
pub mod gpiote;
#[cfg(not(feature = "nrf9160"))]
pub mod nvmc;
pub mod interconnect;
#[cfg(not(any(feature = "nrf52805", feature = "nrf52820")))]
pub mod pwm;
#[cfg(feature = "nrf52840")]
pub mod qspi;
#[cfg(not(feature = "nrf9160"))]
pub mod rng;
#[cfg(not(feature = "nrf52820"))]
pub mod saadc;
pub mod spim;
#[cfg(not(feature = "nrf9160"))]
pub mod temp;
pub mod timer;
pub mod twim;
pub mod uarte;
pub mod wdt;

// This mod MUST go last, so that it sees all the `impl_foo!` macros
#[cfg(feature = "nrf52805")]
#[path = "chips/nrf52805.rs"]
mod chip;
#[cfg(feature = "nrf52810")]
#[path = "chips/nrf52810.rs"]
mod chip;
#[cfg(feature = "nrf52811")]
#[path = "chips/nrf52811.rs"]
mod chip;
#[cfg(feature = "nrf52820")]
#[path = "chips/nrf52820.rs"]
mod chip;
#[cfg(feature = "nrf52832")]
#[path = "chips/nrf52832.rs"]
mod chip;
#[cfg(feature = "nrf52833")]
#[path = "chips/nrf52833.rs"]
mod chip;
#[cfg(feature = "nrf52840")]
#[path = "chips/nrf52840.rs"]
mod chip;
#[cfg(feature = "nrf9160")]
#[path = "chips/nrf9160.rs"]
mod chip;

pub use chip::EASY_DMA_SIZE;

#[cfg(feature = "unstable-pac")]
pub use chip::pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use chip::pac;

use crate::pac::CLOCK;
pub use chip::{peripherals, Peripherals};

pub mod interrupt {
    pub use crate::chip::irqs::*;
    pub use cortex_m::interrupt::{CriticalSection, Mutex};
    pub use embassy::interrupt::{declare, take, Interrupt};
    pub use embassy_hal_common::interrupt::Priority3 as Priority;
}
pub use embassy_macros::interrupt;

pub mod config {
    pub enum HfclkSource {
        Internal,
        ExternalXtal,
    }

    pub enum LfclkSource {
        InternalRC,
        #[cfg(not(feature = "nrf9160"))]
        Synthesized,
        ExternalXtal,
        #[cfg(not(feature = "nrf9160"))]
        ExternalLowSwing,
        #[cfg(not(feature = "nrf9160"))]
        ExternalFullSwing,
    }

    #[non_exhaustive]
    pub struct Config {
        pub hfclk_source: HfclkSource,
        pub lfclk_source: LfclkSource,
        #[cfg(feature = "gpiote")]
        pub gpiote_interrupt_priority: crate::interrupt::Priority,
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

pub fn init(config: config::Config) -> Peripherals {
    // Do this first, so that it panics if user is calling `init` a second time
    // before doing anything important.
    let peripherals = Peripherals::take();

    let r = unsafe { &*CLOCK::ptr() };

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
    #[cfg(not(feature = "nrf9160"))]
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
    #[cfg(feature = "nrf9160")]
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

    peripherals
}
