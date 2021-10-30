#![macro_use]

//! HAL interface to the CLOCK peripheral.
//!
//! See product specification:
//!
//! - nRF52810: v1.3 Section 6.14
//!
use crate::pac;
use core::future::Future;
use core::marker::PhantomData;
use core::task::Poll;
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unborrow;
use futures::future::poll_fn;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    HfClkError,
    LfClkConfigError,
}

#[derive(Clone, Copy)]
pub enum LfClkSource {
    Rc,
    Xtal,
    Synth,
}

impl From<LfClkSource> for u8 {
    fn from(src: LfClkSource) -> Self {
        match src {
            LfClkSource::Rc => 0,
            LfClkSource::Xtal => 1,
            LfClkSource::Synth => 2,
        }
    }
}

#[derive(Clone, Copy)]
pub enum HfClkSource {
    Rc,
    Xtal,
}

#[non_exhaustive]
pub struct LfClockConfig {
    /// Low-frequency clock source
    pub source: LfClkSource,
    /// Low-frequency clock bypass
    pub bypass: bool,
    /// Low-frequency clock external
    pub external: bool,
    /// Low-frequency clock running
    pub running: bool,
}

impl Default for LfClockConfig {
    fn default() -> Self {
        Self {
            source: LfClkSource::Rc,
            bypass: false,
            external: false,
            running: false,
        }
    }
}

#[non_exhaustive]
pub struct HfClockConfig {
    /// High-frequency clock source
    pub source: HfClkSource,
}

impl Default for HfClockConfig {
    fn default() -> Self {
        Self {
            source: HfClkSource::Rc,
        }
    }
}

/// Interface to the clock instance.
pub struct Clock<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Clock<'d, T> {
    pub fn new(
        _clock: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
    ) -> Self {
        unborrow!(irq);

        let r = T::regs();

        // Disable all events interrupts
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        Self {
            phantom: PhantomData,
        }
    }

    pub fn set_hf_clock_config<'a>(&mut self, config: &'a HfClockConfig) -> impl Future<Output = Result<(), Error>> + 'a {
        async move {
            let r = T::regs();

            match (config.source, r.hfclkstat.read().state().bit()) {
                // source is xtal, but rc is requested => stop hfxo
                (HfClkSource::Rc, true) => {
                    r.tasks_hfclkstop.write(|w| w.tasks_hfclkstop().set_bit());
                    Ok(())
                },
                // source is rc, but xtal is requested => start hfxo
                (HfClkSource::Xtal, false) => {
                    // enable "disabled" interrupt
                    r.intenset.write(|w| w.hfclkstarted().bit(true));

                    // start hfxo
                    r.tasks_hfclkstart.write(|w| w.tasks_hfclkstart().set_bit());

                    // Wait for 'started' event.
                    poll_fn(Self::wait_for_start_event).await;

                    // r.hfclkstat is not immediately updated, so we can´t check it
                    Ok(())
                },
                _ => Ok(())
            }
        }
    }

    pub fn set_lf_clock_config<'a>(&mut self, config: &'a LfClockConfig) -> impl Future<Output = Result<(), Error>> + 'a {
        async move {
            let r = T::regs();

            match (config.source, config.bypass, config.external) {
                // source RC (0), bypass disabled (0), external disabled (0)
                (LfClkSource::Rc, false, false) |
                // source XTAL (1), bypass disabled (0), external disabled (0)
                (LfClkSource::Xtal, false, false) |
                // source XTAL (1), bypass enabled (1), external disabled (0)
                (LfClkSource::Xtal, true, false) |
                // source XTAL (1), bypass enabled (1), external enabled (1)
                (LfClkSource::Xtal, true, true) |
                // source synth (2), bypass disabled (0), external disabled (0)
                (LfClkSource::Synth, false, false) => {
                    // if the desired configuration equals the current configuration, do nothing
                    if config.running == r.lfclkstat.read().state().bit() &&
                        u8::from(config.source) == r.lfclksrc.read().src().bits() &&
                        config.bypass == r.lfclksrc.read().bypass().is_enabled() &&
                        config.external == r.lfclksrc.read().external().is_enabled()
                    {
                        return Ok(());
                    }

                    // if the lfclk is running, stop it
                    if r.lfclkstat.read().state().bit() {
                        r.tasks_lfclkstop.write(|w| w.tasks_lfclkstop().trigger());
                    }

                    // set the new configuration
                    r.lfclksrc.write(|w| unsafe { w.src().bits(u8::from(config.source))
                        .bypass().bit(config.bypass)
                        .external().bit(config.external) });

                    // if lfclk should be running, start it
                    if config.running {
                        r.tasks_lfclkstart.write(|w| w.tasks_lfclkstart().trigger());
                        poll_fn(Self::wait_for_start_event).await;
                    }

                    Ok(())
                },
                _ => Err(Error::LfClkConfigError)
            }
        }
    }

    fn on_interrupt(_: *mut ()) {
        let r = T::regs();
        let s = T::state();

        if r.events_lfclkstarted.read().bits() != 0 {
            s.end_waker.wake();
            r.intenclr.write(|w| w.lfclkstarted().clear());
        }
        
        if r.events_hfclkstarted.read().bits() != 0 {
            s.end_waker.wake();
            r.intenclr.write(|w| w.hfclkstarted().clear());
        }
    }

    fn wait_for_start_event(cx: &mut core::task::Context) -> Poll<()> {
        let r = T::regs();
        let s = T::state();

        s.end_waker.register(cx.waker());

        if r.events_lfclkstarted.read().bits() != 0 {
            r.events_lfclkstarted.reset();

            return Poll::Ready(());
        }

        if r.events_hfclkstarted.read().bits() != 0 {
            r.events_hfclkstarted.reset();

            return Poll::Ready(());
        }

        Poll::Pending
    }
}

pub(crate) mod sealed {
    use super::*;

    pub struct State {
        pub end_waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                end_waker: AtomicWaker::new(),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static pac::clock::RegisterBlock;
        fn state() -> &'static State;
    }
}

pub trait Instance: Unborrow<Target = Self> + sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

macro_rules! impl_clock {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::clock::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::clock::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::clock::sealed::State {
                static STATE: crate::clock::sealed::State = crate::clock::sealed::State::new();
                &STATE
            }
        }
        impl crate::clock::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
}
