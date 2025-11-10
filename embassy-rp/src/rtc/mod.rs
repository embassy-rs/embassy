//! RTC driver.
mod filter;

use core::future::poll_fn;
use core::sync::atomic::{AtomicBool, Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

pub use self::filter::DateTimeFilter;

#[cfg_attr(feature = "chrono", path = "datetime_chrono.rs")]
#[cfg_attr(not(feature = "chrono"), path = "datetime_no_deps.rs")]
mod datetime;

pub use self::datetime::{DateTime, DayOfWeek, Error as DateTimeError};
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
    /// Create a new instance of the real time clock, with the given date as an initial value.
    ///
    /// # Errors
    ///
    /// Will return `RtcError::InvalidDateTime` if the datetime is not a valid range.
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

    /// Set the time from internal format
    pub fn restore(&mut self, ymd: rp_pac::rtc::regs::Rtc1, hms: rp_pac::rtc::regs::Rtc0) {
        // disable RTC while we configure it
        self.inner.regs().ctrl().modify(|w| w.set_rtc_enable(false));
        while self.inner.regs().ctrl().read().rtc_active() {
            core::hint::spin_loop();
        }

        self.inner.regs().setup_0().write(|w| {
            *w = rp_pac::rtc::regs::Setup0(ymd.0);
        });
        self.inner.regs().setup_1().write(|w| {
            *w = rp_pac::rtc::regs::Setup1(hms.0);
        });

        // Load the new datetime and re-enable RTC
        self.inner.regs().ctrl().write(|w| w.set_load(true));
        self.inner.regs().ctrl().write(|w| w.set_rtc_enable(true));
        while !self.inner.regs().ctrl().read().rtc_active() {
            core::hint::spin_loop();
        }
    }

    /// Get the time in internal format
    pub fn save(&mut self) -> (rp_pac::rtc::regs::Rtc1, rp_pac::rtc::regs::Rtc0) {
        let rtc_0: rp_pac::rtc::regs::Rtc0 = self.inner.regs().rtc_0().read();
        let rtc_1 = self.inner.regs().rtc_1().read();
        (rtc_1, rtc_0)
    }

    /// Checks to see if this Rtc is running
    pub fn is_running(&self) -> bool {
        self.inner.regs().ctrl().read().rtc_active()
    }

    /// Set the datetime to a new value.
    ///
    /// # Errors
    ///
    /// Will return `RtcError::InvalidDateTime` if the datetime is not a valid range.
    pub fn set_datetime(&mut self, t: DateTime) -> Result<(), RtcError> {
        self::datetime::validate_datetime(&t).map_err(RtcError::InvalidDateTime)?;

        // disable RTC while we configure it
        self.inner.regs().ctrl().modify(|w| w.set_rtc_enable(false));
        while self.inner.regs().ctrl().read().rtc_active() {
            core::hint::spin_loop();
        }

        self.inner.regs().setup_0().write(|w| {
            self::datetime::write_setup_0(&t, w);
        });
        self.inner.regs().setup_1().write(|w| {
            self::datetime::write_setup_1(&t, w);
        });

        // Load the new datetime and re-enable RTC
        self.inner.regs().ctrl().write(|w| w.set_load(true));
        self.inner.regs().ctrl().write(|w| w.set_rtc_enable(true));
        while !self.inner.regs().ctrl().read().rtc_active() {
            core::hint::spin_loop();
        }
        Ok(())
    }

    /// Return the current datetime.
    ///
    /// # Errors
    ///
    /// Will return an `RtcError::InvalidDateTime` if the stored value in the system is not a valid [`DayOfWeek`].
    pub fn now(&self) -> Result<DateTime, RtcError> {
        if !self.is_running() {
            return Err(RtcError::NotRunning);
        }

        let rtc_0 = self.inner.regs().rtc_0().read();
        let rtc_1 = self.inner.regs().rtc_1().read();

        self::datetime::datetime_from_registers(rtc_0, rtc_1).map_err(RtcError::InvalidDateTime)
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
    /// # #[cfg(feature = "chrono")]
    /// # fn main() { }
    /// # #[cfg(not(feature = "chrono"))]
    /// # fn main() {
    /// # use embassy_rp::rtc::{Rtc, DateTimeFilter};
    /// # let mut real_time_clock: Rtc<embassy_rp::peripherals::RTC> = unsafe { core::mem::zeroed() };
    /// let now = real_time_clock.now().unwrap();
    /// real_time_clock.schedule_alarm(
    ///     DateTimeFilter::default()
    ///         .minute(if now.minute == 59 { 0 } else { now.minute + 1 })
    /// );
    /// # }
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
            // Convert day of week value to DayOfWeek enum
            let day_of_week = match irq_1.dotw() {
                0 => DayOfWeek::Sunday,
                1 => DayOfWeek::Monday,
                2 => DayOfWeek::Tuesday,
                3 => DayOfWeek::Wednesday,
                4 => DayOfWeek::Thursday,
                5 => DayOfWeek::Friday,
                6 => DayOfWeek::Saturday,
                _ => return None, // Invalid day of week
            };
            filter.day_of_week = Some(day_of_week);
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
                return Poll::Ready(());
            } else {
                return Poll::Pending;
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

/// Errors that can occur on methods on [Rtc]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RtcError {
    /// An invalid DateTime was given or stored on the hardware.
    InvalidDateTime(DateTimeError),

    /// The RTC clock is not running
    NotRunning,
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
