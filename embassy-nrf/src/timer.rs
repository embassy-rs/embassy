#![macro_use]

use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use futures::future::poll_fn;

use crate::interrupt::{Interrupt, InterruptExt};
use crate::ppi::{Event, Task};
use crate::{pac, Peripheral};

pub(crate) mod sealed {

    use super::*;

    pub trait Instance {
        /// The number of CC registers this instance has.
        const CCS: usize;
        fn regs() -> &'static pac::timer0::RegisterBlock;
        /// Storage for the waker for CC register `n`.
        fn waker(n: usize) -> &'static AtomicWaker;
    }
    pub trait ExtendedInstance {}

    pub trait TimerType {}
}

pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {
    type Interrupt: Interrupt;
}
pub trait ExtendedInstance: Instance + sealed::ExtendedInstance {}

macro_rules! impl_timer {
    ($type:ident, $pac_type:ident, $irq:ident, $ccs:literal) => {
        impl crate::timer::sealed::Instance for peripherals::$type {
            const CCS: usize = $ccs;
            fn regs() -> &'static pac::timer0::RegisterBlock {
                unsafe { &*(pac::$pac_type::ptr() as *const pac::timer0::RegisterBlock) }
            }
            fn waker(n: usize) -> &'static ::embassy_sync::waitqueue::AtomicWaker {
                use ::embassy_sync::waitqueue::AtomicWaker;
                const NEW_AW: AtomicWaker = AtomicWaker::new();
                static WAKERS: [AtomicWaker; $ccs] = [NEW_AW; $ccs];
                &WAKERS[n]
            }
        }
        impl crate::timer::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
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

#[repr(u8)]
pub enum Frequency {
    // I'd prefer not to prefix these with `F`, but Rust identifiers can't start with digits.
    F16MHz = 0,
    F8MHz = 1,
    F4MHz = 2,
    F2MHz = 3,
    F1MHz = 4,
    F500kHz = 5,
    F250kHz = 6,
    F125kHz = 7,
    F62500Hz = 8,
    F31250Hz = 9,
}

/// nRF Timer driver.
///
/// The timer has an internal counter, which is incremented for every tick of the timer.
/// The counter is 32-bit, so it wraps back to 0 at 4294967296.
///
/// It has either 4 or 6 Capture/Compare registers, which can be used to capture the current state of the counter
/// or trigger an event when the counter reaches a certain value.

pub trait TimerType: sealed::TimerType {}

pub enum Awaitable {}
pub enum NotAwaitable {}

impl sealed::TimerType for Awaitable {}
impl sealed::TimerType for NotAwaitable {}
impl TimerType for Awaitable {}
impl TimerType for NotAwaitable {}

pub struct Timer<'d, T: Instance, I: TimerType = NotAwaitable> {
    _p: PeripheralRef<'d, T>,
    _i: PhantomData<I>,
}

impl<'d, T: Instance> Timer<'d, T, Awaitable> {
    pub fn new_awaitable(timer: impl Peripheral<P = T> + 'd, irq: impl Peripheral<P = T::Interrupt> + 'd) -> Self {
        into_ref!(irq);

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        Self::new_irqless(timer)
    }
}
impl<'d, T: Instance> Timer<'d, T, NotAwaitable> {
    /// Create a `Timer` without an interrupt, meaning `Cc::wait` won't work.
    ///
    /// This can be useful for triggering tasks via PPI
    /// `Uarte` uses this internally.
    pub fn new(timer: impl Peripheral<P = T> + 'd) -> Self {
        Self::new_irqless(timer)
    }
}

impl<'d, T: Instance, I: TimerType> Timer<'d, T, I> {
    /// Create a `Timer` without an interrupt, meaning `Cc::wait` won't work.
    ///
    /// This is used by the public constructors.
    fn new_irqless(timer: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(timer);

        let regs = T::regs();

        let mut this = Self {
            _p: timer,
            _i: PhantomData,
        };

        // Stop the timer before doing anything else,
        // since changing BITMODE while running can cause 'unpredictable behaviour' according to the specification.
        this.stop();

        // Set the instance to timer mode.
        regs.mode.write(|w| w.mode().timer());

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
    pub fn task_start(&self) -> Task {
        Task::from_reg(&T::regs().tasks_start)
    }

    /// Returns the STOP task, for use with PPI.
    ///
    /// When triggered, this task stops the timer.
    pub fn task_stop(&self) -> Task {
        Task::from_reg(&T::regs().tasks_stop)
    }

    /// Returns the CLEAR task, for use with PPI.
    ///
    /// When triggered, this task resets the timer's counter to 0.
    pub fn task_clear(&self) -> Task {
        Task::from_reg(&T::regs().tasks_clear)
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

    fn on_interrupt(_: *mut ()) {
        let regs = T::regs();
        for n in 0..T::CCS {
            if regs.events_compare[n].read().bits() != 0 {
                // Clear the interrupt, otherwise the interrupt will be repeatedly raised as soon as the interrupt handler exits.
                // We can't clear the event, because it's used to poll whether the future is done or still pending.
                regs.intenclr
                    .modify(|r, w| unsafe { w.bits(r.bits() | (1 << (16 + n))) });
                T::waker(n).wake();
            }
        }
    }

    /// Returns this timer's `n`th CC register.
    ///
    /// # Panics
    /// Panics if `n` >= the number of CC registers this timer has (4 for a normal timer, 6 for an extended timer).
    pub fn cc(&mut self, n: usize) -> Cc<T, I> {
        if n >= T::CCS {
            panic!("Cannot get CC register {} of timer with {} CC registers.", n, T::CCS);
        }
        Cc {
            n,
            _p: self._p.reborrow(),
            _i: PhantomData,
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
pub struct Cc<'d, T: Instance, I: TimerType = NotAwaitable> {
    n: usize,
    _p: PeripheralRef<'d, T>,
    _i: PhantomData<I>,
}

impl<'d, T: Instance> Cc<'d, T, Awaitable> {
    /// Wait until the timer's counter reaches the value stored in this register.
    ///
    /// This requires a mutable reference so that this task's waker cannot be overwritten by a second call to `wait`.
    pub async fn wait(&mut self) {
        let regs = T::regs();

        // Enable the interrupt for this CC's COMPARE event.
        regs.intenset
            .modify(|r, w| unsafe { w.bits(r.bits() | (1 << (16 + self.n))) });

        // Disable the interrupt if the future is dropped.
        let on_drop = OnDrop::new(|| {
            regs.intenclr
                .modify(|r, w| unsafe { w.bits(r.bits() | (1 << (16 + self.n))) });
        });

        poll_fn(|cx| {
            T::waker(self.n).register(cx.waker());

            if regs.events_compare[self.n].read().bits() != 0 {
                // Reset the register for next time
                regs.events_compare[self.n].reset();
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        // The interrupt was already disabled in the interrupt handler, so there's no need to disable it again.
        on_drop.defuse();
    }
}
impl<'d, T: Instance> Cc<'d, T, NotAwaitable> {}

impl<'d, T: Instance, I: TimerType> Cc<'d, T, I> {
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
    pub fn task_capture(&self) -> Task {
        Task::from_reg(&T::regs().tasks_capture)
    }

    /// Returns this CC register's COMPARE event, for use with PPI.
    ///
    /// This event will fire when the timer's counter reaches the value in this CC register.
    pub fn event_compare(&self) -> Event {
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
