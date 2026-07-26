//! # Embassy time driver for the [ARM CMSDK timer](https://developer.arm.com/documentation/101107/0000/Programmers-model/Base-element/CMSDK-timer)
//!
//! The actual system might have a variable amount of timers at different base addresses.
//! This driver provides an own declaration of the register block in the [regs] module.
//!
//! It provides intialization methods which consume an instance of a [regs::MmioRegisters]
//! register block to allow resource management.
//!
//! If you use this driver, you generally have to call [init] or [init_cortex_m] on `cortex-m`
//! systems, and then set up interrupts handlers which call [on_interrupt_timekeeping] and
//! [on_interrupt_alarm] respectively.
#![deny(missing_docs)]
pub mod regs;

use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering};

#[cfg(feature = "cortex-m")]
use cortex_m::interrupt::InterruptNumber;
use critical_section::{CriticalSection, Mutex};
use embassy_time_driver::{Driver, TICK_HZ, time_driver_impl};
use embassy_time_queue_utils::Queue;
use once_cell::sync::OnceCell;
use regs::{Control, Interrupt};

time_driver_impl!(
    static TIME_DRIVER: TimerDriver = TimerDriver {
        periods: AtomicU32::new(0),
        timekeeper: Mutex::new(RefCell::new(None)),
        alarm_timer: Mutex::new(RefCell::new(None)),
        alarms: Mutex::new(AlarmState::new()),
        queue: Mutex::new(RefCell::new(Queue::new())),
});

/// Initialization method for cortex-m systems.
///
/// This initialization method calls [init] and also unmasks the interrupt.
#[cfg(feature = "cortex-m")]
pub fn init_cortex_m<I: InterruptNumber + Copy>(
    sysclk_hz: u32,
    timekeeper_interrupt: I,
    timekeeper: regs::MmioRegisters<'static>,
    alarm_timer_interrupt: I,
    alarm_timer: regs::MmioRegisters<'static>,
) {
    TIME_DRIVER.init_cortex_m(
        sysclk_hz,
        timekeeper_interrupt,
        timekeeper,
        alarm_timer_interrupt,
        alarm_timer,
    );
}

/// Initialization method.
pub fn init(sysclk_hz: u32, timekeeper: regs::MmioRegisters<'static>, alarm_timer: regs::MmioRegisters<'static>) {
    TIME_DRIVER.init(sysclk_hz, timekeeper, alarm_timer);
}

/// Should be called inside the IRQ of the timekeeper timer.
///
/// # Safety
///
/// This function has to be called once by the TIM IRQ used for the timekeeping.
pub unsafe fn on_interrupt_timekeeping() {
    unsafe {
        TIME_DRIVER.on_interrupt_timekeeping();
    }
}

/// Should be called inside the IRQ of the alarm timer.
///
/// # Safety
///
/// This function has to be called once by the ALARM IRQ used for the alarm handling.
pub unsafe fn on_interrupt_alarm() {
    unsafe {
        TIME_DRIVER.on_interrupt_alarm();
    }
}

#[derive(Debug)]
struct AlarmState {
    timestamp: Cell<u64>,
}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
        }
    }
}

unsafe impl Send for AlarmState {}

static SCALE: OnceCell<u64> = OnceCell::new();

/// Embassy time driver.
pub struct TimerDriver {
    periods: AtomicU32,
    timekeeper: Mutex<RefCell<Option<regs::MmioRegisters<'static>>>>,
    alarm_timer: Mutex<RefCell<Option<regs::MmioRegisters<'static>>>>,
    /// Timestamp at which to fire alarm. u64::MAX if no alarm is scheduled.
    alarms: Mutex<AlarmState>,
    queue: Mutex<RefCell<Queue>>,
}

impl core::fmt::Debug for TimerDriver {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TimerDriver")
            .field("periods", &self.periods)
            .field("alarms", &self.alarms)
            .field("queue", &self.queue)
            .finish()
    }
}

impl TimerDriver {
    /// Initialize the time driver.
    #[cfg(feature = "cortex-m")]
    pub fn init_cortex_m<I: InterruptNumber + Copy>(
        &self,
        sysclk_hz: u32,
        timekeeper_interrupt: I,
        timekeeper: regs::MmioRegisters<'static>,
        alarm_timer_interrupt: I,
        alarm_timer: regs::MmioRegisters<'static>,
    ) {
        unsafe {
            cortex_m::peripheral::NVIC::unmask(timekeeper_interrupt);
            cortex_m::peripheral::NVIC::unmask(alarm_timer_interrupt);
        }
        self.init(sysclk_hz, timekeeper, alarm_timer);
    }

    fn init(
        &self,
        sysclk_hz: u32,
        mut timekeeper: regs::MmioRegisters<'static>,
        alarm_timer: regs::MmioRegisters<'static>,
    ) {
        // Initiate scale value here. This is required to convert timer ticks back to a timestamp.
        SCALE.set((sysclk_hz / TICK_HZ as u32) as u64).unwrap();
        timekeeper.write_control(Control::ZERO);
        timekeeper.write_reload(u32::MAX);
        timekeeper.write_value(u32::MAX);
        timekeeper.write_control(
            Control::builder()
                .with_interrupt_enable(true)
                .with_external_input_as_clock(false)
                .with_external_input_as_enable(false)
                .with_enable(true)
                .build(),
        );

        critical_section::with(|cs| {
            self.timekeeper.replace(cs, Some(timekeeper));
            self.alarm_timer.replace(cs, Some(alarm_timer));
        })
    }

    /// Should be called inside the IRQ of the timekeeper timer.
    ///
    /// # Safety
    ///
    /// This function has to be called once by the TIM IRQ used for the timekeeping.
    pub unsafe fn on_interrupt_timekeeping(&self) {
        self.next_period();
        critical_section::with(|cs| {
            let mut timekeeper_mut = self.timekeeper.borrow(cs).borrow_mut();
            timekeeper_mut
                .as_mut()
                .unwrap()
                .write_interrupt(Interrupt::builder().with_interrupt_bit(true).build());
        });
    }

    /// Should be called inside the IRQ of the alarm timer.
    ///
    /// # Safety
    ///
    ///This function has to be called once by the Alarm IRQ used for the alarm handling.
    pub unsafe fn on_interrupt_alarm(&self) {
        critical_section::with(|cs| {
            if self.alarms.borrow(cs).timestamp.get() <= self.now() {
                self.trigger_alarm(cs)
            }
            let mut alarm_timer_mut = self.alarm_timer.borrow(cs).borrow_mut();
            alarm_timer_mut
                .as_mut()
                .unwrap()
                .write_interrupt(Interrupt::builder().with_interrupt_bit(true).build());
        })
    }

    fn next_period(&self) {
        let period = self.periods.fetch_add(1, Ordering::AcqRel) + 1;
        let t = (period as u64) << 32;
        critical_section::with(|cs| {
            let alarm = &self.alarms.borrow(cs);
            let at = alarm.timestamp.get();
            if at < t {
                self.trigger_alarm(cs);
            } else {
                let remaining_ticks = (at - t).checked_mul(*SCALE.get().unwrap());
                if let Some(ticks) = remaining_ticks
                    && ticks <= u32::MAX as u64
                {
                    let mut timer_mut = self.alarm_timer.borrow(cs).borrow_mut();
                    let timer = timer_mut.as_mut().unwrap();
                    timer.write_control(Control::ZERO);
                    timer.write_value(ticks as u32);
                    timer.write_control(
                        Control::builder()
                            .with_interrupt_enable(true)
                            .with_external_input_as_clock(false)
                            .with_external_input_as_enable(false)
                            .with_enable(true)
                            .build(),
                    );
                }
            }
        })
    }

    fn trigger_alarm(&self, cs: CriticalSection) {
        {
            let mut timer_mut = self.alarm_timer.borrow(cs).borrow_mut();
            let timer = timer_mut.as_mut().unwrap();
            timer.write_control(Control::ZERO);
        }

        let alarm = &self.alarms.borrow(cs);
        // Setting the maximum value disables the alarm.
        alarm.timestamp.set(u64::MAX);

        // Call after clearing alarm, so the callback can set another alarm.
        let mut next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        while !self.set_alarm(cs, next) {
            next = self.queue.borrow(cs).borrow_mut().next_expiration(self.now());
        }
    }

    fn set_alarm(&self, cs: CriticalSection, timestamp: u64) -> bool {
        if SCALE.get().is_none() {
            return false;
        }
        let mut timer_mut = self.alarm_timer.borrow(cs).borrow_mut();
        let timer = timer_mut.as_mut().unwrap();
        timer.write_control(Control::ZERO);

        let alarm = self.alarms.borrow(cs);
        alarm.timestamp.set(timestamp);

        let t = self.now();
        if timestamp <= t {
            alarm.timestamp.set(u64::MAX);
            return false;
        }

        // If it hasn't triggered yet, setup the relevant reset value, regardless of whether
        // the interrupts are enabled or not. When they are enabled at a later point, the
        // right value is already set.

        // If the timestamp is in the next few ticks, add a bit of buffer to be sure the alarm
        // is not missed.
        //
        // This means that an alarm can be delayed for up to 2 ticks (from t+1 to t+3), but this is allowed
        // by the Alarm trait contract. What's not allowed is triggering alarms *before* their scheduled time,
        // and we don't do that here.
        let safe_timestamp = timestamp.max(t + 3);
        let timer_ticks = (safe_timestamp - t).checked_mul(*SCALE.get().unwrap());
        timer.write_reload(u32::MAX);
        if let Some(ticks) = timer_ticks
            && ticks < u32::MAX as u64
        {
            timer.write_value(ticks as u32);
            timer.write_control(
                Control::builder()
                    .with_interrupt_enable(true)
                    .with_external_input_as_clock(false)
                    .with_external_input_as_enable(false)
                    .with_enable(true)
                    .build(),
            );
        }
        // If it's too far in the future, don't enable timer yet.
        // It will be enabled later by `next_period`.

        true
    }
}

impl Driver for TimerDriver {
    fn now(&self) -> u64 {
        if SCALE.get().is_none() {
            return 0;
        }
        let mut period1: u32;
        let mut period2: u32;
        let mut counter_val: u32;

        loop {
            // Acquire ensures that we get the latest value of `periods` and
            // no instructions can be reordered before the load.
            period1 = self.periods.load(Ordering::Acquire);

            let timer_count = critical_section::with(|cs| {
                let timer = TIME_DRIVER.timekeeper.borrow(cs).borrow();
                timer.as_ref().unwrap().read_value()
            });
            counter_val = u32::MAX - timer_count;

            // Double read to protect against race conditions when the counter is overflowing.
            period2 = self.periods.load(Ordering::Relaxed);
            if period1 == period2 {
                let now = (((period1 as u64) << 32) | counter_val as u64) / *SCALE.get().unwrap();
                return now;
            }
        }
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();

            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now());
                while !self.set_alarm(cs, next) {
                    next = queue.next_expiration(self.now());
                }
            }
        })
    }
}
