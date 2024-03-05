//! Integrated 2.4 GHz Radio
//!
//! The 2.4 GHz radio transceiver is compatible with multiple radio standards
//! such as 1Mbps, 2Mbps and Long Range Bluetooth Low Energy.

#![macro_use]

/// Bluetooth Low Energy Radio driver.
#[cfg(not(feature = "nrf51"))]
pub mod ble;
#[cfg(any(
    feature = "nrf52820",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "_nrf5340-net"
))]
/// IEEE 802.15.4
pub mod ieee802154;

use core::marker::PhantomData;

use pac::radio::state::STATE_A as RadioState;
#[cfg(not(feature = "nrf51"))]
use pac::radio::txpower::TXPOWER_A as TxPower;

use crate::{interrupt, pac, Peripheral};

/// RADIO error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Buffer was too long.
    BufferTooLong,
    /// Buffer was too short.
    BufferTooShort,
    /// The buffer is not in data RAM. It's most likely in flash, and nRF's DMA cannot access flash.
    BufferNotInRAM,
    /// Clear channel assessment reported channel in use
    ChannelInUse,
    /// CRC check failed
    CrcFailed(u16),
}

/// Interrupt handler
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();
        let s = T::state();
        // clear all interrupts
        r.intenclr.write(|w| w.bits(0xffff_ffff));
        s.event_waker.wake();
    }
}

pub(crate) mod sealed {
    use embassy_sync::waitqueue::AtomicWaker;

    pub struct State {
        /// end packet transmission or reception
        pub event_waker: AtomicWaker,
    }
    impl State {
        pub const fn new() -> Self {
            Self {
                event_waker: AtomicWaker::new(),
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

/// Get the state of the radio
pub(crate) fn state(radio: &pac::radio::RegisterBlock) -> RadioState {
    match radio.state.read().state().variant() {
        Some(state) => state,
        None => unreachable!(),
    }
}

#[allow(dead_code)]
pub(crate) fn trace_state(radio: &pac::radio::RegisterBlock) {
    match state(radio) {
        RadioState::DISABLED => trace!("radio:state:DISABLED"),
        RadioState::RX_RU => trace!("radio:state:RX_RU"),
        RadioState::RX_IDLE => trace!("radio:state:RX_IDLE"),
        RadioState::RX => trace!("radio:state:RX"),
        RadioState::RX_DISABLE => trace!("radio:state:RX_DISABLE"),
        RadioState::TX_RU => trace!("radio:state:TX_RU"),
        RadioState::TX_IDLE => trace!("radio:state:TX_IDLE"),
        RadioState::TX => trace!("radio:state:TX"),
        RadioState::TX_DISABLE => trace!("radio:state:TX_DISABLE"),
    }
}
