//! AON (Always-On) Timer driver for RP2350
//!
//! The AON Timer is a 64-bit counter that runs at 1 kHz (1ms resolution) and can operate during
//! low-power modes. It's part of the POWMAN peripheral and provides:
//!
//! - Millisecond resolution counter
//! - Alarm support for wakeup from low-power modes (WFI/WFE and DORMANT)
//! - Async alarm waiting with interrupt support (POWMAN_IRQ_TIMER)
//! - Choice of XOSC or LPOSC clock sources
//!
//! # Wake from Low Power
//!
//! The AON Timer supports two wake scenarios:
//!
//! - **WFI/WFE (light sleep)**: Alarm triggers `POWMAN_IRQ_TIMER` interrupt to wake.
//!   Use `wait_for_alarm().await` for async waiting with CPU in low-power mode.
//! - **DORMANT mode (deep sleep)**: Hardware alarm event wakes directly without interrupt.
//!   No CPU clock running, so interrupts cannot fire. This requires using LPOSC as the clock source.
//!
//! # Important Notes
//!
//! - All POWMAN registers require password `0x5AFE` in upper 16 bits for writes
//! - Timer must be stopped before setting the counter value
//! - Resolution is 1ms (1 kHz tick rate)
//!
//! # Example
//!
//! ```no_run
//! use embassy_rp::aon_timer::{AonTimer, Config, ClockSource};
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
//! };
//!
//! let mut timer = AonTimer::new(p.POWMAN, Irqs, config);
//!
//! // Set counter to 0 (or any starting value in milliseconds)
//! timer.set_counter(0);
//!
//! // Start the timer
//! timer.start();
//!
//! // Read current value in milliseconds
//! let ms = timer.now();
//!
//! // Set an alarm and wait asynchronously
//! timer.set_alarm_after(Duration::from_secs(5)).unwrap();
//! timer.wait_for_alarm().await;  // CPU enters low-power mode
//! ```

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::Peri;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;
use embassy_time::Duration;

use crate::{interrupt, pac};

const POWMAN_PASSWORD: u32 = 0x5AFE << 16;

static WAKER: AtomicWaker = AtomicWaker::new();
static ALARM_OCCURRED: AtomicBool = AtomicBool::new(false);

/// AON Timer configuration
#[derive(Clone, Copy)]
pub struct Config {
    /// Clock source for the timer
    pub clock_source: ClockSource,
    /// Clock frequency in kHz
    pub clock_freq_khz: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clock_source: ClockSource::Xosc,
            clock_freq_khz: 12000, // 12 MHz XOSC
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
}

/// AON Timer driver
pub struct AonTimer<'d> {
    _phantom: PhantomData<&'d ()>,
    config: Config,
}

impl<'d> AonTimer<'d> {
    /// Create a new AON Timer instance
    ///
    /// This configures the clock source and frequency but does not start the timer.
    /// Call `start()` to begin counting.
    ///
    /// For async alarm support, you must bind the `POWMAN_IRQ_TIMER` interrupt:
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

        // Configure clock source and frequency
        match config.clock_source {
            ClockSource::Xosc => {
                powman.xosc_freq_khz_int().write(|w| {
                    w.0 = config.clock_freq_khz | POWMAN_PASSWORD;
                    *w
                });
                powman.xosc_freq_khz_frac().write(|w| {
                    w.0 = POWMAN_PASSWORD;
                    *w
                });
            }
            ClockSource::Lposc => {
                powman.lposc_freq_khz_int().write(|w| {
                    w.0 = config.clock_freq_khz | POWMAN_PASSWORD;
                    *w
                });
                powman.lposc_freq_khz_frac().write(|w| {
                    w.0 = POWMAN_PASSWORD;
                    *w
                });
            }
        }

        // Enable the POWMAN_IRQ_TIMER interrupt
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
    /// This allows you to initialize the counter to any value (e.g., Unix timestamp in ms,
    /// or 0 to start counting from boot).
    ///
    /// Note: Timer must be stopped before calling this function.
    pub fn set_counter(&mut self, value_ms: u64) {
        let powman = pac::POWMAN;
        // Write the 64-bit value in 4x 16-bit chunks
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
    pub fn set_alarm(&mut self, alarm_ms: u64) -> Result<(), Error> {
        // Check if alarm is in the past
        let current_ms = self.now();
        if alarm_ms <= current_ms {
            return Err(Error::AlarmInPast);
        }

        // Disable alarm before setting new time
        self.disable_alarm();

        // Set alarm value
        self.set_alarm_value(alarm_ms);

        // Clear any pending alarm flag
        self.clear_alarm();

        // Enable the alarm and interrupt
        self.enable_alarm();
        self.enable_alarm_interrupt();

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

    /// Wait asynchronously for the alarm to fire
    ///
    /// This function will wait until the AON Timer alarm is triggered.
    /// If the alarm is already triggered, it will return immediately.
    /// The CPU will enter WFI (Wait For Interrupt) low-power mode while waiting.
    ///
    /// # Example
    /// ```rust,ignore
    /// // Set alarm for 5 seconds from now
    /// timer.set_alarm_after(Duration::from_secs(5)).unwrap();
    ///
    /// // Wait for the alarm (CPU enters low power mode)
    /// timer.wait_for_alarm().await;
    /// ```
    pub async fn wait_for_alarm(&mut self) {
        poll_fn(|cx| {
            WAKER.register(cx.waker());

            // Atomically check and clear the alarm occurred flag
            if critical_section::with(|_| {
                let occurred = ALARM_OCCURRED.load(Ordering::SeqCst);
                if occurred {
                    ALARM_OCCURRED.store(false, Ordering::SeqCst);
                }
                occurred
            }) {
                // Clear the interrupt flag in hardware
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

        // Disable the alarm interrupt to prevent re-entry
        powman.inte().modify(|w| w.set_timer(false));

        // Disable the alarm
        powman.timer().modify(|w| {
            w.0 = (w.0 & 0x0000FFFF) | POWMAN_PASSWORD;
            w.set_alarm_enab(false);
            *w
        });

        // Clear the interrupt flag in INTR
        powman.intr().modify(|w| w.set_timer(true));

        // Set the alarm occurred flag and wake the waker
        ALARM_OCCURRED.store(true, Ordering::SeqCst);
        WAKER.wake();
    }
}
