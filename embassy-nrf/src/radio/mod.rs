//! Integrated 2.4 GHz Radio
//!
//! The 2.4 GHz radio transceiver is compatible with multiple radio standards
//! such as 1Mbps, 2Mbps and Long Range Bluetooth Low Energy.

#![macro_use]

/// Bluetooth Low Energy Radio driver.
pub mod ble;

use core::marker::PhantomData;

use crate::{interrupt, pac, Peripheral};

/// Interrupt handler
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();
        let s = T::state();

        if r.events_end.read().events_end().bit_is_set() {
            s.end_waker.wake();
            r.intenclr.write(|w| w.end().clear());
        }
    }
}

pub(crate) mod utils {
    use super::*;

    // Check if the HFCLK is XTAL is enabled
    pub fn check_xtal() {
        // safe: only reading the value
        let is_xtal = unsafe {
            let r = &*pac::CLOCK::ptr();
            r.hfclkstat.read().src().is_xtal()
        };
        assert!(is_xtal, "HFCLK must be XTAL");
    }
}

pub(crate) mod sealed {
    use embassy_sync::waitqueue::AtomicWaker;

    pub struct State {
        /// end packet transmission or reception
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
        fn regs() -> &'static crate::pac::radio::RegisterBlock;
        fn state() -> &'static State;
    }
}

macro_rules! impl_radio {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::radio::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::radio::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }

            fn state() -> &'static crate::radio::sealed::State {
                static STATE: crate::radio::sealed::State = crate::radio::sealed::State::new();
                &STATE
            }
        }
        impl crate::radio::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

/// Radio peripheral instance.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}
