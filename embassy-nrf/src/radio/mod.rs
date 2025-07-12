//! Integrated 2.4 GHz Radio
//!
//! The 2.4 GHz radio transceiver is compatible with multiple radio standards
//! such as 1Mbps, 2Mbps and Long Range Bluetooth Low Energy.

#![macro_use]

/// Bluetooth Low Energy Radio driver.
#[cfg(any(
    feature = "nrf52811",
    feature = "nrf52820",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "_nrf5340-net"
))]
/// IEEE 802.15.4
pub mod ieee802154;

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
pub use pac::radio::vals::Txpower as TxPower;

use crate::{interrupt, pac};

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
        r.intenclr().write(|w| w.0 = 0xffff_ffff);
        s.event_waker.wake();
    }
}

pub(crate) struct State {
    /// end packet transmission or reception
    event_waker: AtomicWaker,
}
impl State {
    pub(crate) const fn new() -> Self {
        Self {
            event_waker: AtomicWaker::new(),
        }
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> crate::pac::radio::Radio;
    fn state() -> &'static State;
}

macro_rules! impl_radio {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::radio::SealedInstance for peripherals::$type {
            fn regs() -> crate::pac::radio::Radio {
                pac::$pac_type
            }

            #[allow(unused)]
            fn state() -> &'static crate::radio::State {
                static STATE: crate::radio::State = crate::radio::State::new();
                &STATE
            }
        }
        impl crate::radio::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

/// Radio peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}
