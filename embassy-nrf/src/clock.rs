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
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};
use core::task::Poll;
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unborrow;
use futures::future::poll_fn;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Error,
}

pub enum LfClkSource {
    Rc,
    Xtal,
    Sync,
}

pub enum HfClkSource {
    Rc,
    Xtal,
}

#[non_exhaustive]
pub struct Config {
    /// Low-frequency clock source
    lf_clk_source: LfClkSource,
    /// Low-frequency clock bypass
    lf_clk_bypass: bool,
    /// Low-frequency clock external
    lf_clk_external: bool,
    /// High-frequency clock source
    hf_clk_source: HfClkSource,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lf_clk_source: LfClkSource::Rc,
            lf_clk_bypass: false,
            lf_clk_external: false,
            hf_clk_source: HfClkSource::Rc,
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
        config: Config,
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

    fn wait_for_disabled_event(cx: &mut core::task::Context) -> Poll<()> {
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

impl<'a, T: Instance> Drop for Clock<'a, T> {
    fn drop(&mut self) {
        info!("clock drop");

        // TODO when implementing async here, check for abort

        // disable TODO check
        // let r = T::regs();
        // r.power.write(|w| w.power().disabled());

        info!("clock drop: done");
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
