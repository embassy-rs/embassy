#![macro_use]

use core::marker::PhantomData;
use core::task::Poll;

use embassy::interrupt::Interrupt;
use embassy::interrupt::InterruptExt;
use embassy::util::OnDrop;
use embassy::util::Unborrow;
use embassy_extras::unborrow;
use futures::future::poll_fn;

use crate::pac;
use crate::ppi::Event;
use crate::ppi::Task;

pub(crate) mod sealed {
    use embassy::util::AtomicWaker;

    use super::*;

    pub trait Instance {
        /// The number of CC registers this instance has.
        const CCS: usize;
        fn regs() -> &'static pac::timer0::RegisterBlock;
        /// Storage for the waker for CC register `n`.
        fn waker(n: usize) -> &'static AtomicWaker;
    }
    pub trait ExtendedInstance {}
}

pub trait Instance: Unborrow<Target = Self> + sealed::Instance + 'static {
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
            fn waker(n: usize) -> &'static ::embassy::util::AtomicWaker {
                use ::embassy::util::AtomicWaker;
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
pub struct Timer<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Timer<'d, T> {
    pub fn new(
        timer: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
    ) -> Self {
        unborrow!(irq);

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        Self::new_irqless(timer)
    }

    /// Create a `Timer` without an interrupt, meaning `Cc::wait` won't work.
    ///
    /// This is used by `Uarte` internally.
    pub(crate) fn new_irqless(_timer: impl Unborrow<Target = T> + 'd) -> Self {
        let regs = T::regs();

        let mut this = Self {
            phantom: PhantomData,
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
        T::regs().tasks_start.write(|w| w.tasks_start().trigger())
    }

    /// Stops the timer.
    pub fn stop(&self) {
        T::regs().tasks_stop.write(|w| w.tasks_stop().trigger())
    }

    /// Reset the timer's counter to 0.
    pub fn clear(&self) {
        T::regs().tasks_clear.write(|w| w.tasks_clear().trigger())
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
            if regs.events_compare[n]
                .read()
                .events_compare()
                .is_generated()
            {
                // Clear the interrupt, otherwise the interrupt will be repeatedly raised as soon as the interrupt handler exits.
                // We can't clear the event, because it's used to poll whether the future is done or still pending.
                regs.intenclr.write(|w| match n {
                    0 => w.compare0().clear(),
                    1 => w.compare1().clear(),
                    2 => w.compare2().clear(),
                    3 => w.compare3().clear(),
                    4 => w.compare4().clear(),
                    5 => w.compare5().clear(),
                    _ => unreachable!("No timers have more than 6 CC registers"),
                });
                T::waker(n).wake();
            }
        }
    }

    /// Returns this timer's `n`th CC register.
    ///
    /// # Panics
    /// Panics if `n` >= the number of CC registers this timer has (4 for a normal timer, 6 for an extended timer).
    pub fn cc(&mut self, n: usize) -> Cc<T> {
        if n >= T::CCS {
            panic!(
                "Cannot get CC register {} of timer with {} CC registers.",
                n,
                T::CCS
            );
        }
        Cc {
            n,
            phantom: PhantomData,
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
pub struct Cc<'a, T: Instance> {
    n: usize,
    phantom: PhantomData<&'a mut T>,
}

impl<'a, T: Instance> Cc<'a, T> {
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
        T::regs().tasks_capture[self.n].write(|w| w.tasks_capture().trigger());
        self.read()
    }

    /// Returns this CC register's CAPTURE task, for use with PPI.
    ///
    /// When triggered, this task will capture the current value of the timer's counter in this register.
    pub fn task_capture(&self) -> Task {
        Task::from_reg(&T::regs().tasks_capture[self.n])
    }

    /// Returns this CC register's COMPARE event, for use with PPI.
    ///
    /// This event will fire when the timer's counter reaches the value in this CC register.
    pub fn event_compare(&self) -> Event {
        Event::from_reg(&T::regs().events_compare)
    }

    /// Enable the shortcut between this CC register's COMPARE event and the timer's CLEAR task.
    ///
    /// This means that when the COMPARE event is fired, the CLEAR task will be triggered.
    ///
    /// So, when the timer's counter reaches the value stored in this register, the timer's counter will be reset to 0.
    pub fn short_compare_clear(&self) {
        T::regs().shorts.write(|w| match self.n {
            0 => w.compare0_clear().enabled(),
            1 => w.compare1_clear().enabled(),
            2 => w.compare2_clear().enabled(),
            3 => w.compare3_clear().enabled(),
            4 => w.compare4_clear().enabled(),
            5 => w.compare5_clear().enabled(),
            _ => unreachable!("a `Cc` cannot be created with `n > 5`"),
        })
    }

    /// Disable the shortcut between this CC register's COMPARE event and the timer's CLEAR task.
    pub fn unshort_compare_clear(&self) {
        T::regs().shorts.write(|w| match self.n {
            0 => w.compare0_clear().disabled(),
            1 => w.compare1_clear().disabled(),
            2 => w.compare2_clear().disabled(),
            3 => w.compare3_clear().disabled(),
            4 => w.compare4_clear().disabled(),
            5 => w.compare5_clear().disabled(),
            _ => unreachable!("a `Cc` cannot be created with `n > 5`"),
        })
    }

    /// Enable the shortcut between this CC register's COMPARE event and the timer's STOP task.
    ///
    /// This means that when the COMPARE event is fired, the STOP task will be triggered.
    ///
    /// So, when the timer's counter reaches the value stored in this register, the timer will stop counting up.
    pub fn short_compare_stop(&self) {
        T::regs().shorts.write(|w| match self.n {
            0 => w.compare0_stop().enabled(),
            1 => w.compare1_stop().enabled(),
            2 => w.compare2_stop().enabled(),
            3 => w.compare3_stop().enabled(),
            4 => w.compare4_stop().enabled(),
            5 => w.compare5_stop().enabled(),
            _ => unreachable!("a `Cc` cannot be created with `n > 5`"),
        })
    }

    /// Disable the shortcut between this CC register's COMPARE event and the timer's STOP task.
    pub fn unshort_compare_stop(&self) {
        T::regs().shorts.write(|w| match self.n {
            0 => w.compare0_stop().disabled(),
            1 => w.compare1_stop().disabled(),
            2 => w.compare2_stop().disabled(),
            3 => w.compare3_stop().disabled(),
            4 => w.compare4_stop().disabled(),
            5 => w.compare5_stop().disabled(),
            _ => unreachable!("a `Cc` cannot be created with `n > 5`"),
        })
    }

    /// Wait until the timer's counter reaches the value stored in this register.
    ///
    /// This requires a mutable reference so that this task's waker cannot be overwritten by a second call to `wait`.
    pub async fn wait(&mut self) {
        let regs = T::regs();

        // Enable the interrupt for this CC's COMPARE event.
        regs.intenset.write(|w| match self.n {
            0 => w.compare0().set(),
            1 => w.compare1().set(),
            2 => w.compare2().set(),
            3 => w.compare3().set(),
            4 => w.compare4().set(),
            5 => w.compare5().set(),
            _ => unreachable!("a `Cc` cannot be created with `n > 5`"),
        });

        // Disable the interrupt if the future is dropped.
        let on_drop = OnDrop::new(|| {
            regs.intenclr.write(|w| match self.n {
                0 => w.compare0().clear(),
                1 => w.compare1().clear(),
                2 => w.compare2().clear(),
                3 => w.compare3().clear(),
                4 => w.compare4().clear(),
                5 => w.compare5().clear(),
                _ => unreachable!("a `Cc` cannot be created with `n > 5`"),
            });
        });

        poll_fn(|cx| {
            T::waker(self.n).register(cx.waker());

            if regs.events_compare[self.n]
                .read()
                .events_compare()
                .is_generated()
            {
                // Reset the register for next time
                regs.events_compare[self.n].write(|w| w.events_compare().not_generated());
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
