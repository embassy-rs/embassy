//! Improved inter-integrated circuit (I3C)
//!
//! N6 exposes I3C1/I3C2 with event and error interrupts. This module provides
//! instance wiring, RCC enable/reset, and direct register access. A full
//! controller/target protocol driver is not yet implemented — use [`I3c::regs`]
//! for register-level access until higher-level APIs land.

use embassy_hal_internal::{Peri, PeripheralType};

use crate::pac::i3c::I3c as Regs;
use crate::peripherals;
use crate::rcc::{self, RccPeripheral};

trait SealedInstance: RccPeripheral {
    fn regs() -> Regs;
}

/// I3C instance trait.
#[allow(private_bounds)]
pub trait Instance: PeripheralType + SealedInstance + 'static {
    /// Event interrupt for this instance.
    type EventInterrupt: crate::interrupt::typelevel::Interrupt;
    /// Error interrupt for this instance.
    type ErrorInterrupt: crate::interrupt::typelevel::Interrupt;
}

/// I3C peripheral handle.
pub struct I3c<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> I3c<'d, T> {
    /// Create a new I3C handle and enable/reset the peripheral.
    pub fn new(peri: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();
        Self { _peri: peri }
    }

    /// Direct access to the underlying PAC registers.
    pub fn regs() -> Regs {
        T::regs()
    }
}

foreach_peripheral!(
    (i3c, $inst:ident) => {
        impl SealedInstance for peripherals::$inst {
            fn regs() -> Regs {
                crate::pac::$inst
            }
        }

        impl Instance for peripherals::$inst {
            type EventInterrupt = crate::_generated::peripheral_interrupts::$inst::EV;
            type ErrorInterrupt = crate::_generated::peripheral_interrupts::$inst::ER;
        }
    };
);
