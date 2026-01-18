//! AON (Always-On) Timer driver for RP2350
//!
//! The AON Timer is a 64-bit counter that typically runs at 1 kHz (1ms resolution) and can operate during
//! low-power modes. It's part of the POWMAN peripheral and provides:
//!
//! - Millisecond resolution counter
//! - Alarm support for wakeup from low-power modes (WFI/WFE and DORMANT)
//! - Async alarm waiting with interrupt support (POWMAN_IRQ_TIMER)
//! - Choice of XOSC or LPOSC clock sources
//!
//! # Alarm Wake Modes
//!
//! The AON Timer supports multiple wake modes via the [`AlarmWakeMode`] enum:
//!
//! - **`WfiOnly` (default)**: Alarm triggers `POWMAN_IRQ_TIMER` interrupt to wake from
//!   WFI/WFE (light sleep). Use `wait_for_alarm().await` for async waiting. Works with
//!   both XOSC and LPOSC clock sources.
//!
//! - **`DormantOnly`**: Hardware power-up wake from DORMANT (deep sleep). Sets the
//!   `PWRUP_ON_ALARM` bit to trigger hardware power-up event (no interrupt, since CPU
//!   clock is stopped). **Requirements:**
//!   - Must use LPOSC clock source (XOSC is powered down in DORMANT)
//!   - Requires Secure privilege level (TIMER register is Secure-only)
//!
//! - **`Both`**: Enables both interrupt wake (WFI/WFE) and hardware power-up wake (DORMANT).
//!   Subject to the same requirements as `DormantOnly` for DORMANT support.
//!
//! - **`Disabled`**: Alarm flag is set but no wake mechanisms are enabled. Use `alarm_fired()`
//!   to manually poll the alarm status.
//!
//! You can set the wake mode either in [`Config`] at initialization, or at runtime via
//! [`AonTimer::set_wake_mode()`].
//!
//! # Security Considerations
//!
//! The TIMER register (including the `PWRUP_ON_ALARM` bit) is **Secure-only** per the
//! RP2350 datasheet. Setting wake modes that involve DORMANT wake (`DormantOnly` or `Both`)
//! may fail silently or have no effect when running in Non-secure contexts. Methods
//! `enable_dormant_wake()`, `disable_dormant_wake()`, and `set_wake_mode()` that configure
//! DORMANT wake are subject to this restriction.
//!
//! # Important Notes
//!
//! - All POWMAN registers require password `0x5AFE` in upper 16 bits for writes
//! - Timer must be stopped before setting the counter value
//! - Resolution is 1ms (1 kHz tick rate)
//!
//! # Example - WFI/WFE Wake (Default)
//!
//! ```rust,ignore
//! use embassy_rp::aon_timer::{AonTimer, Config, ClockSource, AlarmWakeMode};
//! use embassy_rp::bind_interrupts;
//! use embassy_time::Duration;
//!
//! // Bind the interrupt handler
//! bind_interrupts!(struct Irqs {
//!     POWMAN_IRQ_TIMER => embassy_rp::aon_timer::InterruptHandler;
//! });
//!
//! let config = Config {
//!     clock_source: ClockSource::Xosc,
//!     clock_freq_khz: 12000, // 12 MHz
//!     alarm_wake_mode: AlarmWakeMode::WfiOnly, // Default
//! };
//!
//! let mut timer = AonTimer::new(p.POWMAN, Irqs, config);
//! timer.set_counter(0);
//! timer.start();
//!
//! // Set an alarm and wait asynchronously (interrupt-based wake)
//! timer.set_alarm_after(Duration::from_secs(5)).unwrap();
//! timer.wait_for_alarm().await;  // CPU enters WFI low-power mode
//! ```
//!
//! # Example - DORMANT Wake
//!
//! ```rust,ignore
//! use embassy_rp::aon_timer::{AonTimer, Config, ClockSource, AlarmWakeMode};
//! use embassy_rp::bind_interrupts;
//! use embassy_time::Duration;
//!
//! bind_interrupts!(struct Irqs {
//!     POWMAN_IRQ_TIMER => embassy_rp::aon_timer::InterruptHandler;
//! });
//!
//! let config = Config {
//!     clock_source: ClockSource::Lposc,  // Required for DORMANT
//!     clock_freq_khz: 32,                // ~32 kHz LPOSC
//!     alarm_wake_mode: AlarmWakeMode::DormantOnly,
//! };
//!
//! let mut timer = AonTimer::new(p.POWMAN, Irqs, config);
//! timer.set_counter(0);
//! timer.start();
//!
//! // Set alarm for DORMANT wake (hardware power-up)
//! timer.set_alarm_after(Duration::from_secs(10)).unwrap();
//! // Enter DORMANT mode here - alarm will wake via power-up event
//! ```
//!
//! # Example - Runtime Wake Mode Change
//!
//! ```rust,ignore
//! use embassy_rp::aon_timer::{AonTimer, Config, ClockSource, AlarmWakeMode};
//! use embassy_rp::bind_interrupts;
//! use embassy_time::Duration;
//!
//! bind_interrupts!(struct Irqs {
//!     POWMAN_IRQ_TIMER => embassy_rp::aon_timer::InterruptHandler;
//! });
//!
//! let mut timer = AonTimer::new(p.POWMAN, Irqs, Config::default());
//! timer.set_counter(0);
//! timer.start();
//!
//! // Use WFI wake initially
//! timer.set_alarm_after(Duration::from_secs(5)).unwrap();
//! timer.wait_for_alarm().await;
//!
//! // Switch to both wake modes at runtime
//! timer.set_wake_mode(AlarmWakeMode::Both);
//! timer.set_alarm_after(Duration::from_secs(10)).unwrap();
//! // Now supports both WFI and DORMANT wake
//! ```
//!
//! # Example - Using DateTime with AON Timer
//!
//! ```rust,ignore
//! use embassy_rp::aon_timer::{AonTimer, Config, DateTime, DayOfWeek};
//! use embassy_rp::bind_interrupts;
//!
//! bind_interrupts!(struct Irqs {
//!     POWMAN_IRQ_TIMER => embassy_rp::aon_timer::InterruptHandler;
//! });
//!
//! let mut timer = AonTimer::new(p.POWMAN, Irqs, Config::default());
//!
//! // Set timer to a specific DateTime (e.g., 2024-06-15 12:30:00 UTC)
//! let start_time = DateTime {
//!     year: 2024,
//!     month: 6,
//!     day: 15,
//!     day_of_week: DayOfWeek::Saturday,
//!     hour: 12,
//!     minute: 30,
//!     second: 0,
//! };
//! timer.set_datetime(start_time).unwrap();
//! timer.start();
//!
//! // Later, read current DateTime
//! let current = timer.now_as_datetime().unwrap();
//! info!("Current time: {}-{:02}-{:02} {:02}:{:02}:{:02}",
//!       current.year, current.month, current.day,
//!       current.hour, current.minute, current.second);
//!
//! // Set alarm for specific DateTime (1 hour later)
//! let alarm_time = DateTime {
//!     year: 2024,
//!     month: 6,
//!     day: 15,
//!     day_of_week: DayOfWeek::Saturday,
//!     hour: 13,
//!     minute: 30,
//!     second: 0,
//! };
//! timer.set_alarm_at_datetime(alarm_time).unwrap();
//! timer.wait_for_alarm().await;
//! ```

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::Peri;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;
use embassy_time::Duration;

pub use crate::datetime::{DateTime, DayOfWeek, Error as DateTimeError};
use crate::{interrupt, pac};

const POWMAN_PASSWORD: u32 = 0x5AFE << 16;

static WAKER: AtomicWaker = AtomicWaker::new();
static ALARM_OCCURRED: AtomicBool = AtomicBool::new(false);

/// Alarm wake mode configuration
///
/// Controls which low-power wake mechanisms are enabled for the alarm.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AlarmWakeMode {
    /// Wake from WFI/WFE only (interrupt-based via POWMAN_IRQ_TIMER)
    ///
    /// This is the default and most common mode. The alarm triggers an interrupt
    /// that wakes the CPU from light sleep (WFI/WFE). Works with both XOSC and LPOSC.
    WfiOnly,

    /// Wake from DORMANT mode only (hardware power-up via PWRUP_ON_ALARM)
    ///
    /// The alarm wakes the chip from deep sleep (DORMANT) by triggering a hardware
    /// power-up event. No interrupt is used since the CPU clock is stopped in DORMANT.
    ///
    /// **Requirements:**
    /// - Must use LPOSC clock source (XOSC is powered down in DORMANT)
    /// - Requires Secure privilege level (TIMER register is Secure-only)
    /// - May fail silently in Non-secure contexts
    DormantOnly,

    /// Wake from both WFI/WFE and DORMANT modes
    ///
    /// Enables both interrupt-based wake (WFI/WFE) and hardware power-up wake (DORMANT).
    /// Subject to the same requirements as DormantOnly for DORMANT wake support.
    Both,

    /// Alarm fires but doesn't wake (manual polling only)
    ///
    /// The alarm flag is set in hardware but no wake mechanisms are enabled.
    /// Use `alarm_fired()` to manually poll the alarm status.
    Disabled,
}

/// AON Timer configuration
#[derive(Clone, Copy)]
pub struct Config {
    /// Clock source for the timer
    pub clock_source: ClockSource,
    /// Clock frequency in kHz
    pub clock_freq_khz: u32,
    /// Alarm wake mode
    ///
    /// Controls which low-power wake mechanisms are enabled for alarms.
    /// See [`AlarmWakeMode`] for details on each mode.
    pub alarm_wake_mode: AlarmWakeMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clock_source: ClockSource::Xosc,
            clock_freq_khz: 12000, // 12 MHz XOSC
            alarm_wake_mode: AlarmWakeMode::WfiOnly,
        }
    }
}

/// Clock source for the AON Timer
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClockSource {
    /// Crystal oscillator (more accurate, requires external crystal)
    Xosc,
    /// Low-power oscillator (less accurate, ~32 kHz, available in all power modes)
    Lposc,
}

/// AON Timer errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The alarm time is in the past
    AlarmInPast,
    /// DateTime conversion error
    DateTime(DateTimeError),
}

/// AON Timer driver
pub struct AonTimer<'d> {
    _phantom: PhantomData<&'d ()>,
    config: Config,
}

impl<'d> AonTimer<'d> {
    /// Create a new AON Timer instance
    ///
    /// This configures the clock source, frequency, and alarm wake mode but does not
    /// start the timer. Call `start()` to begin counting.
    ///
    /// The wake mode in `config.alarm_wake_mode` determines how alarms wake the CPU:
    /// - `WfiOnly` (default): Interrupt-based wake from WFI/WFE
    /// - `DormantOnly`: Hardware power-up wake from DORMANT (requires LPOSC + Secure mode)
    /// - `Both`: Both interrupt and hardware power-up wake
    /// - `Disabled`: No automatic wake (manual polling only)
    ///
    /// For interrupt-based wake modes (`WfiOnly` or `Both`), you must bind the
    /// `POWMAN_IRQ_TIMER` interrupt:
    /// ```rust,ignore
    /// bind_interrupts!(struct Irqs {
    ///     POWMAN_IRQ_TIMER => aon_timer::InterruptHandler;
    /// });
    /// let timer = AonTimer::new(p.POWMAN, Irqs, config);
    /// ```
    pub fn new(
        _inner: Peri<'d, crate::peripherals::POWMAN>,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::POWMAN_IRQ_TIMER, InterruptHandler> + 'd,
        config: Config,
    ) -> Self {
        let powman = pac::POWMAN;

        match config.clock_source {
            ClockSource::Xosc => {
                powman.xosc_freq_khz_int().write(|w| {
                    w.0 = (config.clock_freq_khz & 0xFFFF) | POWMAN_PASSWORD;
                    *w
                });
                powman.xosc_freq_khz_frac().write(|w| {
                    w.0 = POWMAN_PASSWORD;
                    *w
                });
            }
            ClockSource::Lposc => {
                powman.lposc_freq_khz_int().write(|w| {
                    w.0 = (config.clock_freq_khz & 0xFFFF) | POWMAN_PASSWORD;
                    *w
                });
                powman.lposc_freq_khz_frac().write(|w| {
                    w.0 = POWMAN_PASSWORD;
                    *w
                });
            }
        }

        interrupt::POWMAN_IRQ_TIMER.unpend();
        unsafe { interrupt::POWMAN_IRQ_TIMER.enable() };

        Self {
            _phantom: PhantomData,
            config,
        }
    }

    /// Start the timer
    ///
    /// The timer will begin counting from its current value.
    pub fn start(&mut self) {
        let powman = pac::POWMAN;
        powman.timer().modify(|w| {
            w.0 = (w.0 & 0x0000FFFF) | POWMAN_PASSWORD;
            match self.config.clock_source {
                ClockSource::Lposc => w.set_use_lposc(true),
                ClockSource::Xosc => w.set_use_xosc(true),
            }
            w.set_run(true);
            *w
        });
    }

    /// Stop the timer
    ///
    /// The timer will stop counting but retain its current value.
    pub fn stop(&mut self) {
        let powman = pac::POWMAN;
        powman.timer().modify(|w| {
            w.0 = (w.0 & 0x0000FFFF) | POWMAN_PASSWORD;
            w.set_run(false);
            *w
        });
    }

    /// Check if the timer is currently running
    pub fn is_running(&self) -> bool {
        let powman = pac::POWMAN;
        powman.timer().read().run()
    }

    /// Read the current counter value in milliseconds
    ///
    /// This reads the 64-bit counter value with rollover protection.
    /// The value represents milliseconds since the counter was last set.
    pub fn now(&self) -> u64 {
        let powman = pac::POWMAN;
        // Read with rollover protection: read upper, lower, upper again
        loop {
            let upper1 = powman.read_time_upper().read();
            let lower = powman.read_time_lower().read();
            let upper2 = powman.read_time_upper().read();

            // If upper didn't change, we got a consistent read
            if upper1 == upper2 {
                return ((upper1 as u64) << 32) | (lower as u64);
            }
            // Otherwise retry (rollover occurred)
        }
    }

    /// Set the counter value in milliseconds
    ///
    /// This allows you to initialize the counter to any value (e.g., milliseconds since epoch,
    /// or 0 to start counting from boot).
    ///
    /// Note: Timer must be stopped before calling this function.
    pub fn set_counter(&mut self, value_ms: u64) {
        if self.is_running() {
            panic!("timer must be stopped before setting counter");
        }
        let powman = pac::POWMAN;
        powman.set_time_15to0().write(|w| {
            w.0 = ((value_ms & 0xFFFF) as u32) | POWMAN_PASSWORD;
            *w
        });
        powman.set_time_31to16().write(|w| {
            w.0 = (((value_ms >> 16) & 0xFFFF) as u32) | POWMAN_PASSWORD;
            *w
        });
        powman.set_time_47to32().write(|w| {
            w.0 = (((value_ms >> 32) & 0xFFFF) as u32) | POWMAN_PASSWORD;
            *w
        });
        powman.set_time_63to48().write(|w| {
            w.0 = (((value_ms >> 48) & 0xFFFF) as u32) | POWMAN_PASSWORD;
            *w
        });
    }

    /// Set an alarm for a specific counter value (in milliseconds)
    ///
    /// The alarm will fire when the counter reaches this value.
    /// Returns an error if the alarm time is in the past.
    ///
    /// The wake behavior depends on the configured `alarm_wake_mode`:
    /// - `WfiOnly`: Alarm triggers interrupt wake from WFI/WFE
    /// - `DormantOnly`: Alarm triggers power-up from DORMANT mode
    /// - `Both`: Alarm triggers both interrupt and power-up wake
    /// - `Disabled`: Alarm flag is set but no wake occurs
    pub fn set_alarm(&mut self, alarm_ms: u64) -> Result<(), Error> {
        let current_ms = self.now();
        if alarm_ms <= current_ms {
            return Err(Error::AlarmInPast);
        }

        self.disable_alarm();
        self.set_alarm_value(alarm_ms);
        self.clear_alarm();

        match self.config.alarm_wake_mode {
            AlarmWakeMode::WfiOnly => {
                self.disable_dormant_wake();
                self.enable_alarm_interrupt();
            }
            AlarmWakeMode::DormantOnly => {
                self.enable_dormant_wake();
                self.disable_alarm_interrupt();
            }
            AlarmWakeMode::Both => {
                self.enable_dormant_wake();
                self.enable_alarm_interrupt();
            }
            AlarmWakeMode::Disabled => {
                self.disable_dormant_wake();
                self.disable_alarm_interrupt();
            }
        }

        self.enable_alarm();

        Ok(())
    }

    /// Enable the alarm interrupt (INTE.TIMER)
    ///
    /// This allows the alarm to trigger POWMAN_IRQ_TIMER and wake from WFI/WFE.
    fn enable_alarm_interrupt(&mut self) {
        let powman = pac::POWMAN;
        powman.inte().modify(|w| w.set_timer(true));
    }

    /// Disable the alarm interrupt (INTE.TIMER)
    pub fn disable_alarm_interrupt(&mut self) {
        let powman = pac::POWMAN;
        powman.inte().modify(|w| w.set_timer(false));
    }

    /// Set the internal alarm value in milliseconds
    #[inline(always)]
    fn set_alarm_value(&mut self, value: u64) {
        let powman = pac::POWMAN;
        powman.alarm_time_15to0().write(|w| {
            w.0 = ((value & 0xFFFF) as u32) | POWMAN_PASSWORD;
            *w
        });
        powman.alarm_time_31to16().write(|w| {
            w.0 = (((value >> 16) & 0xFFFF) as u32) | POWMAN_PASSWORD;
            *w
        });
        powman.alarm_time_47to32().write(|w| {
            w.0 = (((value >> 32) & 0xFFFF) as u32) | POWMAN_PASSWORD;
            *w
        });
        powman.alarm_time_63to48().write(|w| {
            w.0 = (((value >> 48) & 0xFFFF) as u32) | POWMAN_PASSWORD;
            *w
        });
    }

    /// Check if the alarm has fired
    pub fn alarm_fired(&self) -> bool {
        let powman = pac::POWMAN;
        powman.timer().read().alarm()
    }

    /// Clear the alarm flag
    pub fn clear_alarm(&mut self) {
        let powman = pac::POWMAN;
        powman.timer().modify(|w| {
            w.0 = (w.0 & 0x0000FFFF) | POWMAN_PASSWORD;
            w.set_alarm(true); // Write 1 to clear
            *w
        });
    }

    /// Disable the alarm
    pub fn disable_alarm(&mut self) {
        let powman = pac::POWMAN;
        powman.timer().modify(|w| {
            w.0 = (w.0 & 0x0000FFFF) | POWMAN_PASSWORD;
            w.set_alarm_enab(false);
            *w
        });
    }

    /// Enable the alarm
    pub fn enable_alarm(&mut self) {
        let powman = pac::POWMAN;
        powman.timer().modify(|w| {
            w.0 = (w.0 & 0x0000FFFF) | POWMAN_PASSWORD;
            w.set_alarm_enab(true);
            *w
        });
    }

    /// Enable DORMANT mode wake on alarm
    ///
    /// Sets the TIMER.PWRUP_ON_ALARM bit to allow the alarm to wake the chip from
    /// DORMANT (deep sleep) mode. This is a hardware power-up event, distinct from
    /// interrupt-based WFI/WFE wake.
    ///
    /// **Security Note**: The TIMER register is Secure-only per the RP2350 datasheet.
    /// This method may fail silently or have no effect when called from Non-secure contexts.
    ///
    /// **Clock Source**: DORMANT wake requires LPOSC as the clock source, since XOSC
    /// is powered down in DORMANT mode.
    pub fn enable_dormant_wake(&mut self) {
        let powman = pac::POWMAN;
        powman.timer().modify(|w| {
            w.0 = (w.0 & 0x0000FFFF) | POWMAN_PASSWORD;
            w.set_pwrup_on_alarm(true);
            *w
        });
    }

    /// Disable DORMANT mode wake on alarm
    ///
    /// Clears the TIMER.PWRUP_ON_ALARM bit. The alarm will no longer wake the chip
    /// from DORMANT mode, but can still wake from WFI/WFE via interrupts.
    ///
    /// **Security Note**: The TIMER register is Secure-only per the RP2350 datasheet.
    /// This method may fail silently or have no effect when called from Non-secure contexts.
    pub fn disable_dormant_wake(&mut self) {
        let powman = pac::POWMAN;
        powman.timer().modify(|w| {
            w.0 = (w.0 & 0x0000FFFF) | POWMAN_PASSWORD;
            w.set_pwrup_on_alarm(false);
            *w
        });
    }

    /// Set the alarm wake mode
    ///
    /// Configures which low-power wake mechanisms are enabled for the alarm.
    /// This immediately updates the hardware configuration.
    ///
    /// # Arguments
    /// * `mode` - The desired wake mode (see [`AlarmWakeMode`])
    ///
    /// # Security Note
    /// Setting modes that involve DORMANT wake (DormantOnly or Both) requires
    /// Secure privilege level. These modes may fail silently in Non-secure contexts.
    pub fn set_wake_mode(&mut self, mode: AlarmWakeMode) {
        match mode {
            AlarmWakeMode::WfiOnly => {
                self.enable_alarm_interrupt();
                self.disable_dormant_wake();
            }
            AlarmWakeMode::DormantOnly => {
                self.disable_alarm_interrupt();
                self.enable_dormant_wake();
            }
            AlarmWakeMode::Both => {
                self.enable_alarm_interrupt();
                self.enable_dormant_wake();
            }
            AlarmWakeMode::Disabled => {
                self.disable_alarm_interrupt();
                self.disable_dormant_wake();
            }
        }
        self.config.alarm_wake_mode = mode;
    }

    /// Set an alarm to fire after a duration from now
    ///
    /// This is a convenience method that sets the alarm to `now() + duration`.
    pub fn set_alarm_after(&mut self, duration: Duration) -> Result<(), Error> {
        let current_ms = self.now();
        let alarm_ms = current_ms + duration.as_millis();
        self.set_alarm(alarm_ms)
    }

    /// Get the current counter value as a Duration
    ///
    /// This is useful for measuring time spans. The duration represents
    /// the time since the counter was last set to 0.
    pub fn elapsed(&self) -> Duration {
        Duration::from_millis(self.now())
    }

    /// Set the counter from a DateTime (Unix epoch)
    ///
    /// # Errors
    /// Returns error if DateTime is before 1970-01-01.
    ///
    /// # Panics
    /// Panics if timer is running.
    pub fn set_datetime(&mut self, dt: DateTime) -> Result<(), DateTimeError> {
        #[cfg(feature = "chrono")]
        let millis = crate::datetime::timestamp_millis(&dt)?;
        #[cfg(not(feature = "chrono"))]
        let millis = dt.timestamp_millis()?;

        self.set_counter(millis);
        Ok(())
    }

    /// Get the current counter value as a DateTime (Unix epoch)
    ///
    /// # Errors
    /// Returns error if counter value cannot be represented as valid DateTime.
    pub fn now_as_datetime(&self) -> Result<DateTime, DateTimeError> {
        let millis = self.now();

        #[cfg(feature = "chrono")]
        return crate::datetime::from_timestamp_millis(millis);
        #[cfg(not(feature = "chrono"))]
        return DateTime::from_timestamp_millis(millis);
    }

    /// Set an alarm for a specific DateTime
    ///
    /// # Errors
    /// Returns error if DateTime conversion fails or alarm time is in the past.
    pub fn set_alarm_at_datetime(&mut self, dt: DateTime) -> Result<(), Error> {
        #[cfg(feature = "chrono")]
        let alarm_ms = crate::datetime::timestamp_millis(&dt).map_err(Error::DateTime)?;
        #[cfg(not(feature = "chrono"))]
        let alarm_ms = dt.timestamp_millis().map_err(Error::DateTime)?;

        self.set_alarm(alarm_ms)
    }

    /// Wait asynchronously for the alarm to fire
    ///
    /// This function will wait until the AON Timer alarm is triggered.
    /// If the alarm is already triggered, it will return immediately.
    ///
    /// **Wake Mode Behavior:**
    /// - `WfiOnly` or `Both`: CPU enters WFI (Wait For Interrupt) low-power mode while
    ///   waiting. The alarm interrupt will wake the CPU and this function will return.
    /// - `DormantOnly` or `Disabled`: This function will NOT automatically wake from
    ///   DORMANT mode. For DORMANT wake, the hardware power-up event will restart the
    ///   chip, not resume this async function.
    ///
    /// This method is primarily intended for `WfiOnly` and `Both` wake modes where
    /// interrupt-based wake is available.
    ///
    /// # Example
    /// ```rust,ignore
    /// // Set alarm for 5 seconds from now
    /// timer.set_alarm_after(Duration::from_secs(5)).unwrap();
    ///
    /// // Wait for the alarm (CPU enters WFI low-power mode)
    /// timer.wait_for_alarm().await;
    /// ```
    pub async fn wait_for_alarm(&mut self) {
        poll_fn(|cx| {
            WAKER.register(cx.waker());

            if ALARM_OCCURRED.swap(false, Ordering::SeqCst) {
                self.clear_alarm();

                compiler_fence(Ordering::SeqCst);
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }
}

/// Interrupt handler for AON Timer alarms
pub struct InterruptHandler {
    _empty: (),
}

impl interrupt::typelevel::Handler<interrupt::typelevel::POWMAN_IRQ_TIMER> for InterruptHandler {
    #[inline(always)]
    unsafe fn on_interrupt() {
        let powman = crate::pac::POWMAN;

        powman.inte().modify(|w| w.set_timer(false));

        powman.timer().modify(|w| {
            w.0 = (w.0 & 0x0000FFFF) | POWMAN_PASSWORD;
            w.set_alarm_enab(false);
            *w
        });

        powman.intr().modify(|w| w.set_timer(true));

        ALARM_OCCURRED.store(true, Ordering::SeqCst);
        WAKER.wake();
    }
}
