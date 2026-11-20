//! RTC driver.
mod datetime;
mod filter;

use core::future::poll_fn;
use core::sync::atomic::{AtomicBool, Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

pub use self::datetime::RawDateTime;
#[cfg(feature = "jiff")]
pub use self::datetime::{InvalidDateTimeError, OutOfRangeError};
pub use self::filter::DateTimeFilter;
use crate::clocks::clk_rtc_freq;
use crate::interrupt::typelevel::Binding;
use crate::interrupt::{self, InterruptExt};

// Static waker for the interrupt handler
static WAKER: AtomicWaker = AtomicWaker::new();
// Static flag to indicate if an alarm has occurred
static ALARM_OCCURRED: AtomicBool = AtomicBool::new(false);

/// A reference to the real time clock of the system
pub struct Rtc<'d, T: Instance> {
    inner: Peri<'d, T>,
}

impl<'d, T: Instance> Rtc<'d, T> {
    /// Create a new instance of the real time clock.
    pub fn new(inner: Peri<'d, T>, _irq: impl Binding<interrupt::typelevel::RTC_IRQ, InterruptHandler>) -> Self {
        // Set the RTC divider
        inner.regs().clkdiv_m1().write(|w| w.set_clkdiv_m1(clk_rtc_freq() - 1));

        // Setup the IRQ
        // Clear any pending interrupts from the RTC_IRQ interrupt and enable it, so we do not have unexpected interrupts after initialization
        interrupt::RTC_IRQ.unpend();
        unsafe { interrupt::RTC_IRQ.enable() };

        Self { inner }
    }

    /// Enable or disable the leap year check. The rp2040 chip will always add a Feb 29th on every year that is divisible by 4, but this may be incorrect (e.g. on century years). This function allows you to disable this check.
    ///
    /// Leap year checking is enabled by default.
    pub fn set_leap_year_check(&mut self, leap_year_check_enabled: bool) {
        self.inner.regs().ctrl().modify(|w| {
            w.set_force_notleapyear(!leap_year_check_enabled);
        });
    }

    /// Checks to see if this Rtc is running
    pub fn is_running(&self) -> bool {
        self.inner.regs().ctrl().read().rtc_active()
    }

    /// Set the date and time.
    ///
    /// Writes a dummy day of week (Sunday) to the RTC, which [`read`](Self::read) ignores. Use
    /// [`set_raw`](Self::set_raw) if you need the hardware day of week, e.g. for alarms that
    /// match on it.
    ///
    /// # Errors
    ///
    /// Returns [`OutOfRangeError`] if the year is not in `0..=4095`.
    #[cfg(feature = "jiff")]
    pub fn set(&mut self, t: jiff::civil::DateTime) -> Result<(), OutOfRangeError> {
        self.set_raw(datetime::from_jiff(t)?);
        Ok(())
    }

    /// Write raw values to the RTC calendar registers, and start the RTC.
    ///
    /// See [`RawDateTime`]: the values are not validated.
    pub fn set_raw(&mut self, t: RawDateTime) {
        // disable RTC while we configure it
        self.inner.regs().ctrl().modify(|w| w.set_rtc_enable(false));
        while self.inner.regs().ctrl().read().rtc_active() {
            core::hint::spin_loop();
        }

        self.inner.regs().setup_0().write(|w| {
            w.set_year(t.year);
            w.set_month(t.month);
            w.set_day(t.day);
        });
        self.inner.regs().setup_1().write(|w| {
            w.set_dotw(t.day_of_week);
            w.set_hour(t.hour);
            w.set_min(t.minute);
            w.set_sec(t.second);
        });

        // Load the new datetime and re-enable RTC
        self.inner.regs().ctrl().write(|w| w.set_load(true));
        self.inner.regs().ctrl().write(|w| w.set_rtc_enable(true));
        while !self.inner.regs().ctrl().read().rtc_active() {
            core::hint::spin_loop();
        }
    }

    /// Read the current date and time.
    ///
    /// Ignores the day of week stored in the RTC.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidDateTimeError`] if the RTC holds a date and time that does not exist, such as
    /// February 29 of 2100 (see [`set_leap_year_check`](Self::set_leap_year_check)), or the
    /// registers of an RTC that was never set.
    #[cfg(feature = "jiff")]
    pub fn read(&self) -> Result<jiff::civil::DateTime, InvalidDateTimeError> {
        datetime::to_jiff(self.read_raw())
    }

    /// Read the raw contents of the RTC calendar registers.
    ///
    /// See [`RawDateTime`]: the values are not validated.
    pub fn read_raw(&self) -> RawDateTime {
        // Reading RTC_0 latches RTC_1, so it must be read first.
        let rtc_0 = self.inner.regs().rtc_0().read();
        let rtc_1 = self.inner.regs().rtc_1().read();
        RawDateTime {
            year: rtc_1.year(),
            month: rtc_1.month(),
            day: rtc_1.day(),
            day_of_week: rtc_0.dotw(),
            hour: rtc_0.hour(),
            minute: rtc_0.min(),
            second: rtc_0.sec(),
        }
    }

    /// Disable the alarm that was scheduled with [`schedule_alarm`].
    ///
    /// [`schedule_alarm`]: #method.schedule_alarm
    pub fn disable_alarm(&mut self) {
        self.inner.regs().irq_setup_0().modify(|s| s.set_match_ena(false));

        while self.inner.regs().irq_setup_0().read().match_active() {
            core::hint::spin_loop();
        }
    }

    /// Schedule an alarm. The `filter` determines at which point in time this alarm is set.
    ///
    /// Keep in mind that the filter only triggers on the specified time. If you want to schedule this alarm every minute, you have to call:
    /// ```no_run
    /// # use embassy_rp::rtc::{Rtc, DateTimeFilter};
    /// # let mut real_time_clock: Rtc<embassy_rp::peripherals::RTC> = unsafe { core::mem::zeroed() };
    /// let now = real_time_clock.read_raw();
    /// real_time_clock.schedule_alarm(
    ///     DateTimeFilter::default()
    ///         .minute(if now.minute == 59 { 0 } else { now.minute + 1 })
    /// );
    /// ```
    pub fn schedule_alarm(&mut self, filter: DateTimeFilter) {
        self.disable_alarm();

        self.inner.regs().irq_setup_0().write(|w| {
            filter.write_setup_0(w);
        });
        self.inner.regs().irq_setup_1().write(|w| {
            filter.write_setup_1(w);
        });

        self.inner.regs().inte().modify(|w| w.set_rtc(true));

        // Set the enable bit and check if it is set
        self.inner.regs().irq_setup_0().modify(|w| w.set_match_ena(true));
        while !self.inner.regs().irq_setup_0().read().match_active() {
            core::hint::spin_loop();
        }
    }

    /// Clear the interrupt. This should be called every time the `RTC_IRQ` interrupt is triggered,
    /// or the next [`schedule_alarm`] will never fire.
    ///
    /// [`schedule_alarm`]: #method.schedule_alarm
    pub fn clear_interrupt(&mut self) {
        self.disable_alarm();
    }

    /// Check if an alarm is scheduled.
    ///
    /// This function checks if the RTC alarm is currently active. If it is, it returns the alarm configuration
    /// as a [`DateTimeFilter`]. Otherwise, it returns `None`.
    pub fn alarm_scheduled(&self) -> Option<DateTimeFilter> {
        // Check if alarm is active
        if !self.inner.regs().irq_setup_0().read().match_active() {
            return None;
        }

        // Get values from both alarm registers
        let irq_0 = self.inner.regs().irq_setup_0().read();
        let irq_1 = self.inner.regs().irq_setup_1().read();

        // Create a DateTimeFilter and populate it based on which fields are enabled
        let mut filter = DateTimeFilter::default();

        if irq_0.year_ena() {
            filter.year = Some(irq_0.year());
        }

        if irq_0.month_ena() {
            filter.month = Some(irq_0.month());
        }

        if irq_0.day_ena() {
            filter.day = Some(irq_0.day());
        }

        if irq_1.dotw_ena() {
            filter.day_of_week = Some(irq_1.dotw());
        }

        if irq_1.hour_ena() {
            filter.hour = Some(irq_1.hour());
        }

        if irq_1.min_ena() {
            filter.minute = Some(irq_1.min());
        }

        if irq_1.sec_ena() {
            filter.second = Some(irq_1.sec());
        }

        Some(filter)
    }

    /// Wait for an RTC alarm to trigger.
    ///
    /// This function will wait until the RTC alarm is triggered. If the alarm is already triggered, it will return immediately.
    /// If no alarm is scheduled, it will wait indefinitely until one is scheduled and triggered.
    pub async fn wait_for_alarm(&mut self) {
        poll_fn(|cx| {
            WAKER.register(cx.waker());

            // Atomically check and clear the alarm occurred flag to prevent race conditions
            if critical_section::with(|_| {
                let occurred = ALARM_OCCURRED.load(Ordering::SeqCst);
                if occurred {
                    ALARM_OCCURRED.store(false, Ordering::SeqCst);
                }
                occurred
            }) {
                // Clear the interrupt and disable the alarm
                self.clear_interrupt();

                compiler_fence(Ordering::SeqCst);
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }
}

/// Interrupt handler.
pub struct InterruptHandler {
    _empty: (),
}

impl crate::interrupt::typelevel::Handler<crate::interrupt::typelevel::RTC_IRQ> for InterruptHandler {
    unsafe fn on_interrupt() {
        // Disable the alarm first thing, to prevent unexpected re-entry
        let rtc = crate::pac::RTC;
        rtc.irq_setup_0().modify(|w| w.set_match_ena(false));

        // Set the alarm occurred flag and wake the waker
        ALARM_OCCURRED.store(true, Ordering::SeqCst);
        WAKER.wake();
    }
}

trait SealedInstance {
    fn regs(&self) -> crate::pac::rtc::Rtc;
}

/// RTC peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {}

impl SealedInstance for crate::peripherals::RTC {
    fn regs(&self) -> crate::pac::rtc::Rtc {
        crate::pac::RTC
    }
}
impl Instance for crate::peripherals::RTC {}
