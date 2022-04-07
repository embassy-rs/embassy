//! ## EasyDMA considerations
//!
//! On nRF chips, peripherals can use the so called EasyDMA feature to offload the task of interacting
//! with peripherals. It takes care of sending/receiving data over a variety of bus protocols (TWI/I2C, UART, SPI).
//! However, EasyDMA requires the buffers used to transmit and receive data to reside in RAM. Unfortunately, Rust
//! slices will not always do so. The following example using the SPI peripheral shows a common situation where this might happen:
//!
//! ```no_run
//! // As we pass a slice to the function whose contents will not ever change,
//! // the compiler writes it into the flash and thus the pointer to it will
//! // reference static memory. Since EasyDMA requires slices to reside in RAM,
//! // this function call will fail.
//! let result = spim.write_from_ram(&[1, 2, 3]);
//! assert_eq!(result, Err(Error::DMABufferNotInDataMemory));
//!
//! // The data is still static and located in flash. However, since we are assigning
//! // it to a variable, the compiler will load it into memory. Passing a reference to the
//! // variable will yield a pointer that references dynamic memory, thus making EasyDMA happy.
//! // This function call succeeds.
//! let data = [1, 2, 3];
//! let result = spim.write_from_ram(&data);
//! assert!(result.is_ok());
//! ```
//!
//! Each peripheral struct which uses EasyDMA ([`Spim`](spim::Spim), [`Uarte`](uarte::Uarte), [`Twim`](twim::Twim)) has two variants of their mutating functions:
//! - Functions with the suffix (e.g. [`write_from_ram`](Spim::write_from_ram), [`transfer_from_ram`](Spim::transfer_from_ram)) will return an error if the passed slice does not reside in RAM.
//! - Functions without the suffix (e.g. [`write`](Spim::write), [`transfer`](Spim::transfer)) will check whether the data is in RAM and copy it into memory prior to transmission.
//!
//! Since copying incurs a overhead, you are given the option to choose from `_from_ram` variants which will
//! fail and notify you, or the more convenient versions without the suffix which are potentially a little bit
//! more inefficient. Be aware that this overhead is not only in terms of instruction count but also in terms of memory usage
//! as the methods without the suffix will be allocating a statically sized buffer (up to 512 bytes for the nRF52840).
//!
//! Note that the methods that read data like [`read`](spim::Spim::read) and [`transfer_in_place`](spim::Spim::transfer_in_place) do not have the corresponding `_from_ram` variants as
//! mutable slices always reside in RAM.

#![no_std]
#![cfg_attr(
    feature = "nightly",
    feature(generic_associated_types, type_alias_impl_trait)
)]

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

pub mod buffered_uarte;
pub mod gpio;
#[cfg(feature = "gpiote")]
pub mod gpiote;
#[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
pub mod nvmc;
pub mod ppi;
#[cfg(not(any(feature = "nrf52805", feature = "nrf52820", feature = "_nrf5340-net")))]
pub mod pwm;
#[cfg(feature = "nrf52840")]
pub mod qspi;
#[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
pub mod rng;
#[cfg(not(any(feature = "nrf52820", feature = "_nrf5340-net")))]
pub mod saadc;
pub mod spim;
#[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
pub mod temp;
pub mod timer;
pub mod twim;
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

#[cfg(feature = "unstable-pac")]
pub use chip::pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use chip::pac;

pub use embassy::util::Unborrow;
pub use embassy_hal_common::unborrow;

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
        #[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
        Synthesized,
        ExternalXtal,
        #[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
        ExternalLowSwing,
        #[cfg(not(any(feature = "_nrf5340", feature = "_nrf9160")))]
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

    peripherals
}
