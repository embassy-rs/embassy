//! Timer driver.
//!
//! Important note! This driver is very low level. For most time-related use cases, like
//! "sleep for X seconds", "do something every X seconds", or measuring time, you should
//! use [`embassy-time`](https://crates.io/crates/embassy-time) instead!

#![macro_use]

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};

use crate::pac;
use crate::pac::timer::vals;
use crate::ppi::{Event, Task};

pub(crate) trait SealedInstance {
    /// The number of CC registers this instance has.
    const CCS: usize;
    fn regs() -> pac::timer::Timer;
}

/// Basic Timer instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// Extended timer instance.
pub trait ExtendedInstance: Instance {}

macro_rules! impl_timer {
    ($type:ident, $pac_type:ident, $irq:ident, $ccs:literal) => {
        impl crate::timer::SealedInstance for peripherals::$type {
            const CCS: usize = $ccs;
            fn regs() -> pac::timer::Timer {
                unsafe { pac::timer::Timer::from_ptr(pac::$pac_type.as_ptr()) }
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
pub struct Timer<'d> {
    r: pac::timer::Timer,
    ccs: usize,
    _p: PhantomData<&'d ()>,
}

impl<'d> Timer<'d> {
    /// Create a new `Timer` driver.
    ///
    /// This can be useful for triggering tasks via PPI.
    /// `Uarte` uses this internally.
    pub fn new<T: Instance>(timer: Peri<'d, T>) -> Self {
        Self::new_inner(timer, false)
    }

    /// Create a new `Timer` driver in counter mode.
    ///
    /// This can be useful for triggering tasks via PPI.
    /// `Uarte` uses this internally.
    pub fn new_counter<T: Instance>(timer: Peri<'d, T>) -> Self {
        Self::new_inner(timer, true)
    }

    fn new_inner<T: Instance>(_timer: Peri<'d, T>, is_counter: bool) -> Self {
        let regs = T::regs();

        let this = Self {
            r: regs,
            ccs: T::CCS,
            _p: PhantomData,
        };

        // Stop the timer before doing anything else,
        // since changing BITMODE while running can cause 'unpredictable behaviour' according to the specification.
        this.stop();

        regs.mode().write(|w| {
            w.set_mode(match is_counter {
                #[cfg(not(feature = "_nrf51"))]
                true => vals::Mode::LOW_POWER_COUNTER,
                #[cfg(feature = "_nrf51")]
                true => vals::Mode::COUNTER,
                false => vals::Mode::TIMER,
            })
        });

        // Make the counter's max value as high as possible.
        // TODO: is there a reason someone would want to set this lower?
        regs.bitmode().write(|w| w.set_bitmode(vals::Bitmode::_32BIT));

        // Initialize the counter at 0.
        this.clear();

        // Default to the max frequency of the lower power clock
        this.set_frequency(Frequency::F1MHz);

        for n in 0..this.ccs {
            let cc = this.cc(n);
            // Initialize all the shorts as disabled.
            cc.unshort_compare_clear();
            cc.unshort_compare_stop();
            // Initialize the CC registers as 0.
            cc.write(0);
        }

        this
    }

    /// Direct access to the register block.
    #[cfg(feature = "unstable-pac")]
    #[inline]
    pub fn regs(&mut self) -> pac::timer::Timer {
        self.r
    }

    /// Starts the timer.
    pub fn start(&self) {
        self.r.tasks_start().write_value(1)
    }

    /// Stops the timer.
    pub fn stop(&self) {
        self.r.tasks_stop().write_value(1)
    }

    /// Reset the timer's counter to 0.
    pub fn clear(&self) {
        self.r.tasks_clear().write_value(1)
    }

    /// Returns the START task, for use with PPI.
    ///
    /// When triggered, this task starts the timer.
    pub fn task_start(&self) -> Task<'d> {
        Task::from_reg(self.r.tasks_start())
    }

    /// Returns the STOP task, for use with PPI.
    ///
    /// When triggered, this task stops the timer.
    pub fn task_stop(&self) -> Task<'d> {
        Task::from_reg(self.r.tasks_stop())
    }

    /// Returns the CLEAR task, for use with PPI.
    ///
    /// When triggered, this task resets the timer's counter to 0.
    pub fn task_clear(&self) -> Task<'d> {
        Task::from_reg(self.r.tasks_clear())
    }

    /// Returns the COUNT task, for use with PPI.
    ///
    /// When triggered, this task increments the timer's counter by 1.
    /// Only works in counter mode.
    pub fn task_count(&self) -> Task<'d> {
        Task::from_reg(self.r.tasks_count())
    }

    /// Change the timer's frequency.
    ///
    /// This will stop the timer if it isn't already stopped,
    /// because the timer may exhibit 'unpredictable behaviour' if it's frequency is changed while it's running.
    pub fn set_frequency(&self, frequency: Frequency) {
        self.stop();

        self.r
            .prescaler()
            // SAFETY: `frequency` is a variant of `Frequency`,
            // whose values are all in the range of 0-9 (the valid range of `prescaler`).
            .write(|w| w.set_prescaler(frequency as u8))
    }

    /// Returns this timer's `n`th CC register.
    ///
    /// # Panics
    /// Panics if `n` >= the number of CC registers this timer has (4 for a normal timer, 6 for an extended timer).
    pub fn cc(&self, n: usize) -> Cc<'d> {
        if n >= self.ccs {
            panic!("Cannot get CC register {} of timer with {} CC registers.", n, self.ccs);
        }
        Cc {
            n,
            r: self.r,
            _p: PhantomData,
        }
    }
}

impl Timer<'static> {
    /// Persist the timer's configuration for the rest of the program's lifetime. This method
    /// should be preferred over [`core::mem::forget()`] because the `'static` bound prevents
    /// accidental reuse of the underlying peripheral.
    pub fn persist(self) {
        core::mem::forget(self);
    }
}

impl<'d> Drop for Timer<'d> {
    fn drop(&mut self) {
        self.stop();
    }
}

/// A representation of a timer's Capture/Compare (CC) register.
///
/// A CC register holds a 32-bit value.
/// This is used either to store a capture of the timer's current count, or to specify the value for the timer to compare against.
///
/// The timer will fire the register's COMPARE event when its counter reaches the value stored in the register.
/// When the register's CAPTURE task is triggered, the timer will store the current value of its counter in the register
pub struct Cc<'d> {
    n: usize,
    r: pac::timer::Timer,
    _p: PhantomData<&'d ()>,
}

impl<'d> Cc<'d> {
    /// Get the current value stored in the register.
    pub fn read(&self) -> u32 {
        self.r.cc(self.n).read()
    }

    /// Set the value stored in the register.
    ///
    /// `event_compare` will fire when the timer's counter reaches this value.
    pub fn write(&self, value: u32) {
        self.r.cc(self.n).write_value(value);
    }

    /// Capture the current value of the timer's counter in this register, and return it.
    pub fn capture(&self) -> u32 {
        self.r.tasks_capture(self.n).write_value(1);
        self.read()
    }

    /// Returns this CC register's CAPTURE task, for use with PPI.
    ///
    /// When triggered, this task will capture the current value of the timer's counter in this register.
    pub fn task_capture(&self) -> Task<'d> {
        Task::from_reg(self.r.tasks_capture(self.n))
    }

    /// Returns this CC register's COMPARE event, for use with PPI.
    ///
    /// This event will fire when the timer's counter reaches the value in this CC register.
    pub fn event_compare(&self) -> Event<'d> {
        Event::from_reg(self.r.events_compare(self.n))
    }

    /// Clear the COMPARE event for this CC register.
    #[inline]
    pub fn clear_events(&self) {
        self.r.events_compare(self.n).write_value(0);
    }

    /// Enable the shortcut between this CC register's COMPARE event and the timer's CLEAR task.
    ///
    /// This means that when the COMPARE event is fired, the CLEAR task will be triggered.
    ///
    /// So, when the timer's counter reaches the value stored in this register, the timer's counter will be reset to 0.
    pub fn short_compare_clear(&self) {
        self.r.shorts().modify(|w| w.set_compare_clear(self.n, true))
    }

    /// Disable the shortcut between this CC register's COMPARE event and the timer's CLEAR task.
    pub fn unshort_compare_clear(&self) {
        self.r.shorts().modify(|w| w.set_compare_clear(self.n, false))
    }

    /// Enable the shortcut between this CC register's COMPARE event and the timer's STOP task.
    ///
    /// This means that when the COMPARE event is fired, the STOP task will be triggered.
    ///
    /// So, when the timer's counter reaches the value stored in this register, the timer will stop counting up.
    pub fn short_compare_stop(&self) {
        self.r.shorts().modify(|w| w.set_compare_stop(self.n, true))
    }

    /// Disable the shortcut between this CC register's COMPARE event and the timer's STOP task.
    pub fn unshort_compare_stop(&self) {
        self.r.shorts().modify(|w| w.set_compare_stop(self.n, false))
    }
}
