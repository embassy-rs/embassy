#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

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

pub mod buffered_uarte;
pub mod gpio;
pub mod gpiote;
pub mod ppi;
#[cfg(not(any(feature = "nrf52805", feature = "nrf52820")))]
pub mod pwm;
#[cfg(feature = "nrf52840")]
pub mod qspi;
pub mod rng;
pub mod rtc;
#[cfg(not(feature = "nrf52820"))]
pub mod saadc;
pub mod spim;
pub mod timer;
pub mod twim;
pub mod uarte;

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

#[cfg(feature = "unstable-pac")]
pub use chip::pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use chip::pac;

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
        Synthesized,
        ExternalXtal,
        ExternalLowSwing,
        ExternalFullSwing,
    }

    #[non_exhaustive]
    pub struct Config {
        pub hfclk_source: HfclkSource,
        pub lfclk_source: LfclkSource,
        pub gpiote_interrupt_priority: crate::interrupt::Priority,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                // There are hobby nrf52 boards out there without external XTALs...
                // Default everything to internal so it Just Works. User can enable external
                // xtals if they know they have them.
                hfclk_source: HfclkSource::Internal,
                lfclk_source: LfclkSource::InternalRC,
                gpiote_interrupt_priority: crate::interrupt::Priority::P0,
            }
        }
    }
}

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

    // Start LFCLK.
    // Datasheet says this could take 100us from synth source
    // 600us from rc source, 0.25s from an external source.
    r.events_lfclkstarted.write(|w| unsafe { w.bits(0) });
    r.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
    while r.events_lfclkstarted.read().bits() == 0 {}

    // Init GPIOTE
    crate::gpiote::init(config.gpiote_interrupt_priority);

    peripherals
}
