//! EGU driver.
//!
//! The event generator driver provides a higher level API for task triggering
//! and events to use with PPI.

#![macro_use]

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;

use crate::ppi::{Event, Task};
use crate::{interrupt, pac, Peri};

/// An instance of the EGU.
pub struct Egu<'d> {
    r: pac::egu::Egu,
    _phantom: PhantomData<&'d ()>,
}

impl<'d> Egu<'d> {
    /// Create a new EGU instance.
    pub fn new<T: Instance>(_p: Peri<'d, T>) -> Self {
        Self {
            r: T::regs(),
            _phantom: PhantomData,
        }
    }

    /// Get a handle to a trigger for the EGU.
    pub fn trigger(&mut self, number: TriggerNumber) -> Trigger<'d> {
        Trigger {
            number,
            r: self.r,
            _phantom: PhantomData,
        }
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> pac::egu::Egu;
}

/// Basic Egu instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_egu {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::egu::SealedInstance for peripherals::$type {
            fn regs() -> pac::egu::Egu {
                pac::$pac_type
            }
        }
        impl crate::egu::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

/// Represents a trigger within the EGU.
pub struct Trigger<'d> {
    number: TriggerNumber,
    r: pac::egu::Egu,
    _phantom: PhantomData<&'d ()>,
}

impl<'d> Trigger<'d> {
    /// Get task for this trigger to use with PPI.
    pub fn task(&self) -> Task<'d> {
        let nr = self.number as usize;
        Task::from_reg(self.r.tasks_trigger(nr))
    }

    /// Get event for this trigger to use with PPI.
    pub fn event(&self) -> Event<'d> {
        let nr = self.number as usize;
        Event::from_reg(self.r.events_triggered(nr))
    }

    /// Enable interrupts for this trigger
    pub fn enable_interrupt(&mut self) {
        self.r
            .intenset()
            .modify(|w| w.set_triggered(self.number as usize, true));
    }

    /// Enable interrupts for this trigger
    pub fn disable_interrupt(&mut self) {
        self.r
            .intenset()
            .modify(|w| w.set_triggered(self.number as usize, false));
    }
}

/// Represents a trigger within an EGU.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum TriggerNumber {
    Trigger0 = 0,
    Trigger1 = 1,
    Trigger2 = 2,
    Trigger3 = 3,
    Trigger4 = 4,
    Trigger5 = 5,
    Trigger6 = 6,
    Trigger7 = 7,
    Trigger8 = 8,
    Trigger9 = 9,
    Trigger10 = 10,
    Trigger11 = 11,
    Trigger12 = 12,
    Trigger13 = 13,
    Trigger14 = 14,
    Trigger15 = 15,
}
