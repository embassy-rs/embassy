//! Timer driver.
//!
//! Important note! This driver is very low level. For most time-related use cases, like
//! "sleep for X seconds", "do something every X seconds", or measuring time, you should
//! use [`embassy-time`](https://crates.io/crates/embassy-time) instead!

#![macro_use]

use embassy_hal_internal::{into_ref, PeripheralRef};

use crate::ppi::{Event, Task};
use crate::{pac, Peripheral};

pub(crate) mod sealed {

    use super::*;

    pub trait Instance {
        /// The number of CC registers this instance has.
        const CCS: usize;
        fn regs() -> &'static pac::timer0::RegisterBlock;
    }
    pub trait ExtendedInstance {}

    pub trait TimerType {}
}

/// Basic Timer instance.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// Extended timer instance.
pub trait ExtendedInstance: Instance + sealed::ExtendedInstance {}

macro_rules! impl_timer {
    ($type:ident, $pac_type:ident, $irq:ident, $ccs:literal) => {
        impl crate::timer::sealed::Instance for peripherals::$type {
            const CCS: usize = $ccs;
            fn regs() -> &'static pac::timer0::RegisterBlock {
                unsafe { &*(pac::$pac_type::ptr() as *const pac::timer0::RegisterBlock) }
            }
        }
        impl crate::timer::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl_timer!($type, $pac_type, $irq, 4);
    };
    ($type:ident, $pac_type:ident, $irq:ident, extended) => {
        impl_timer!($type, $pac_type, $irq, 6);
        impl crate::timer::sealed::ExtendedInstance for peripherals::$type {}
        impl crate::timer::ExtendedInstance for peripherals::$type {}
    };
}

/// Timer frequency
#[repr(u8)]
pub enum Frequency {
    /// 16MHz
    F16MHz = 0,
    /// 8MHz
    F8MHz = 1,
    /// 4MHz
    F4MHz = 2,
    /// 2MHz
    F2MHz = 3,
    /// 1MHz
    F1MHz = 4,
    /// 500kHz
    F500kHz = 5,
    /// 250kHz
    F250kHz = 6,
    /// 125kHz
    F125kHz = 7,
    /// 62500Hz
    F62500Hz = 8,
    /// 31250Hz
    F31250Hz = 9,
}

/// nRF Timer driver.
///
/// The timer has an internal counter, which is incremented for every tick of the timer.
/// The counter is 32-bit, so it wraps back to 0 when it reaches 2^32.
///
/// It has either 4 or 6 Capture/Compare registers, which can be used to capture the current state of the counter
/// or trigger an event when the counter reaches a certain value.

/// Timer driver.
pub struct Timer<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Timer<'d, T> {
    /// Create a new `Timer` driver.
    ///
    /// This can be useful for triggering tasks via PPI
    /// `Uarte` uses this internally.
    pub fn new(timer: impl Peripheral<P = T> + 'd) -> Self {
        Self::new_inner(timer, false)
    }

    /// Create a new `Timer` driver in counter mode.
    ///
    /// This can be useful for triggering tasks via PPI
    /// `Uarte` uses this internally.
    pub fn new_counter(timer: impl Peripheral<P = T> + 'd) -> Self {
        Self::new_inner(timer, true)
    }

    fn new_inner(timer: impl Peripheral<P = T> + 'd, is_counter: bool) -> Self {
        into_ref!(timer);

        let regs = T::regs();

        let this = Self { _p: timer };

        // Stop the timer before doing anything else,
        // since changing BITMODE while running can cause 'unpredictable behaviour' according to the specification.
        this.stop();

        if is_counter {
            regs.mode.write(|w| w.mode().low_power_counter());
        } else {
            regs.mode.write(|w| w.mode().timer());
        }

        // Make the counter's max value as high as possible.
        // TODO: is there a reason someone would want to set this lower?
        regs.bitmode.write(|w| w.bitmode()._32bit());

        // Initialize the counter at 0.
        this.clear();

        // Default to the max frequency of the lower power clock
        this.set_frequency(Frequency::F1MHz);

        for n in 0..T::CCS {
            let cc = this.cc(n);
            // Initialize all the shorts as disabled.
            cc.unshort_compare_clear();
            cc.unshort_compare_stop();
            // Initialize the CC registers as 0.
            cc.write(0);
        }

        this
    }

    /// Starts the timer.
    pub fn start(&self) {
        T::regs().tasks_start.write(|w| unsafe { w.bits(1) })
    }

    /// Stops the timer.
    pub fn stop(&self) {
        T::regs().tasks_stop.write(|w| unsafe { w.bits(1) })
    }

    /// Reset the timer's counter to 0.
    pub fn clear(&self) {
        T::regs().tasks_clear.write(|w| unsafe { w.bits(1) })
    }

    /// Returns the START task, for use with PPI.
    ///
    /// When triggered, this task starts the timer.
    pub fn task_start(&self) -> Task<'d> {
        Task::from_reg(&T::regs().tasks_start)
    }

    /// Returns the STOP task, for use with PPI.
    ///
    /// When triggered, this task stops the timer.
    pub fn task_stop(&self) -> Task<'d> {
        Task::from_reg(&T::regs().tasks_stop)
    }

    /// Returns the CLEAR task, for use with PPI.
    ///
    /// When triggered, this task resets the timer's counter to 0.
    pub fn task_clear(&self) -> Task<'d> {
        Task::from_reg(&T::regs().tasks_clear)
    }

    /// Returns the COUNT task, for use with PPI.
    ///
    /// When triggered, this task increments the timer's counter by 1.
    /// Only works in counter mode.
    pub fn task_count(&self) -> Task<'d> {
        Task::from_reg(&T::regs().tasks_count)
    }

    /// Change the timer's frequency.
    ///
    /// This will stop the timer if it isn't already stopped,
    /// because the timer may exhibit 'unpredictable behaviour' if it's frequency is changed while it's running.
    pub fn set_frequency(&self, frequency: Frequency) {
        self.stop();

        T::regs()
            .prescaler
            // SAFETY: `frequency` is a variant of `Frequency`,
            // whose values are all in the range of 0-9 (the valid range of `prescaler`).
            .write(|w| unsafe { w.prescaler().bits(frequency as u8) })
    }

    /// Returns this timer's `n`th CC register.
    ///
    /// # Panics
    /// Panics if `n` >= the number of CC registers this timer has (4 for a normal timer, 6 for an extended timer).
    pub fn cc(&self, n: usize) -> Cc<'d, T> {
        if n >= T::CCS {
            panic!("Cannot get CC register {} of timer with {} CC registers.", n, T::CCS);
        }
        Cc {
            n,
            _p: unsafe { self._p.clone_unchecked() },
        }
    }
}

/// A representation of a timer's Capture/Compare (CC) register.
///
/// A CC register holds a 32-bit value.
/// This is used either to store a capture of the timer's current count, or to specify the value for the timer to compare against.
///
/// The timer will fire the register's COMPARE event when its counter reaches the value stored in the register.
/// When the register's CAPTURE task is triggered, the timer will store the current value of its counter in the register
pub struct Cc<'d, T: Instance> {
    n: usize,
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Cc<'d, T> {
    /// Get the current value stored in the register.
    pub fn read(&self) -> u32 {
        T::regs().cc[self.n].read().cc().bits()
    }

    /// Set the value stored in the register.
    ///
    /// `event_compare` will fire when the timer's counter reaches this value.
    pub fn write(&self, value: u32) {
        // SAFETY: there are no invalid values for the CC register.
        T::regs().cc[self.n].write(|w| unsafe { w.cc().bits(value) })
    }

    /// Capture the current value of the timer's counter in this register, and return it.
    pub fn capture(&self) -> u32 {
        T::regs().tasks_capture[self.n].write(|w| unsafe { w.bits(1) });
        self.read()
    }

    /// Returns this CC register's CAPTURE task, for use with PPI.
    ///
    /// When triggered, this task will capture the current value of the timer's counter in this register.
    pub fn task_capture(&self) -> Task<'d> {
        Task::from_reg(&T::regs().tasks_capture)
    }

    /// Returns this CC register's COMPARE event, for use with PPI.
    ///
    /// This event will fire when the timer's counter reaches the value in this CC register.
    pub fn event_compare(&self) -> Event<'d> {
        Event::from_reg(&T::regs().events_compare[self.n])
    }

    /// Enable the shortcut between this CC register's COMPARE event and the timer's CLEAR task.
    ///
    /// This means that when the COMPARE event is fired, the CLEAR task will be triggered.
    ///
    /// So, when the timer's counter reaches the value stored in this register, the timer's counter will be reset to 0.
    pub fn short_compare_clear(&self) {
        T::regs()
            .shorts
            .modify(|r, w| unsafe { w.bits(r.bits() | (1 << self.n)) })
    }

    /// Disable the shortcut between this CC register's COMPARE event and the timer's CLEAR task.
    pub fn unshort_compare_clear(&self) {
        T::regs()
            .shorts
            .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << self.n)) })
    }

    /// Enable the shortcut between this CC register's COMPARE event and the timer's STOP task.
    ///
    /// This means that when the COMPARE event is fired, the STOP task will be triggered.
    ///
    /// So, when the timer's counter reaches the value stored in this register, the timer will stop counting up.
    pub fn short_compare_stop(&self) {
        T::regs()
            .shorts
            .modify(|r, w| unsafe { w.bits(r.bits() | (1 << (8 + self.n))) })
    }

    /// Disable the shortcut between this CC register's COMPARE event and the timer's STOP task.
    pub fn unshort_compare_stop(&self) {
        T::regs()
            .shorts
            .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << (8 + self.n))) })
    }
}
