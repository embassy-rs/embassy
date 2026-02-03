//! Windowed Watchdog Timer (WWDT) driver for MCXA microcontrollers.
//!
//! The WWDT is a hardware timer that can reset the system or generate an interrupt if the software fails to
//! periodically "feed" the watchdog within a specified time window. This helps detect
//! and recover from software failures or system hangs.
//!
//! The FRO12M provides a 1 MHz clock (clk_1m) used as WWDT0 independant clock source. This clock is / 4 by an internal fixed divider.

use embassy_hal_internal::Peri;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_time::Duration;

use crate::interrupt::typelevel;
use crate::interrupt::typelevel::Handler;
use crate::pac;
use crate::pac::wwdt::vals::{Wden, Wdprotect, Wdreset};
use crate::peripherals::WWDT0;

/// WWDT0 Error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    TimeoutTooSmall,
    TimeoutTooLarge,
    WarningTooLarge,
}

/// WWDT configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// The timeout period after which the watchdog will trigger
    pub timeout: Duration,
    pub warning: Option<Duration>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(1),
            warning: None,
        }
    }
}

/// Watchdog peripheral
pub struct Watchdog<'d> {
    _peri: Peri<'d, WWDT0>,
    // The register block of the WWDT instance
    info: pac::wwdt::Wwdt,
}

impl<'d> Watchdog<'d> {
    /// Create a new WWDT instance.
    /// Configure the WWDT, enables the interrupt, set the timeout and or warning value.
    ///
    /// # Arguments
    ///
    /// * `_peri` - The WWDT0 peripheral instance
    /// * `_irq` - Interrupt binding for WWDT0
    /// * `config - WWDT0 config with timeout and optional warning value
    pub fn new(
        _peri: Peri<'d, WWDT0>,
        _irq: impl crate::interrupt::typelevel::Binding<typelevel::WWDT0, InterruptHandler> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        let info = pac::WWDT0;

        let watchdog = Self { _peri, info };

        let base_frequency = crate::clocks::with_clocks(|clocks| {
            // Ensure clk_1m is active at the required power level
            clocks.ensure_clk_1m_active(&crate::clocks::PoweredClock::NormalEnabledDeepSleepDisabled)
        })
        .expect("Clocks not initialized")
        .expect("clk_1m not enabled or not at required power level");

        let frequency = base_frequency / 4;

        // Enable WATCHDOG clock by writing to mrcc register
        // Can't use enable_and_reset API here because WWDT doesn't have a reset signal.
        pac::MRCC0.mrcc_glb_cc0().modify(|w| w.set_wwdt0(true));

        let timeout_cycles = (frequency as u64 * config.timeout.as_micros()) / 1_000_000;

        // Ensure the value fits in u32 and is within valid range
        //
        // Writing a value below FFh causes 00_00FFh to load into the
        // register. Therefore, the minimum timeout interval is TWDCLK
        // X 256 X 4.
        if timeout_cycles > 0xFFFFFF {
            return Err(Error::TimeoutTooLarge);
        }
        if timeout_cycles <= 0xFF {
            return Err(Error::TimeoutTooSmall);
        }

        watchdog.set_timeout_value(timeout_cycles as u32);

        // Windows value is set to max at reset for no effect.

        if let Some(warning_value) = config.warning {
            let warning_cycles = (frequency as u64 * warning_value.as_micros()) / 1_000_000;
            if warning_cycles > 0x3FF {
                return Err(Error::WarningTooLarge);
            }

            watchdog.set_warning_value(warning_cycles as u16);
            watchdog.enable_interrupt();
        } else {
            watchdog.enable_reset();
        }

        watchdog.lock_oscillator();

        crate::pac::Interrupt::WWDT0.unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { crate::pac::Interrupt::WWDT0.enable() };

        Ok(watchdog)
    }

    /// Start the watchdog timer with the specified timeout period.
    pub fn start(&mut self) {
        self.enable();
        self.feed();

        // Set the WDPROTECT bit to false after the Feed Sequence (0xAA, 0x55)
        self.set_flexible_mode();
    }

    /// Feed the watchdog to prevent timeout.
    ///
    /// This must be called periodically before the timeout period expires to prevent
    /// the watchdog from triggering a reset or interrupt.
    pub fn feed(&self) {
        critical_section::with(|_cs| {
            self.info.feed().write(|w| w.set_feed(0xAA));
            self.info.feed().write(|w| w.set_feed(0x55));
        });
    }

    /// Enable the watchdog timer.
    /// Function is blocking until the watchdog is actually started.
    fn enable(&self) {
        self.info.mod_().modify(|w| w.set_wden(Wden::RUN));
        while self.info.tc().read().count() == 0xFF {}
    }

    /// Set the watchdog protection mode to flexible.
    fn set_flexible_mode(&self) {
        self.info.mod_().modify(|w| w.set_wdprotect(Wdprotect::FLEXIBLE));
    }

    /// Enable interrupt mode.
    fn enable_interrupt(&self) {
        self.info.mod_().modify(|w| w.set_wdreset(Wdreset::INTERRUPT));
    }

    /// Enable reset mode.
    fn enable_reset(&self) {
        self.info.mod_().modify(|w| w.set_wdreset(Wdreset::RESET));
    }

    /// Set the timeout value in clock cycles.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Number of clock cycles before timeout.
    fn set_timeout_value(&self, timeout: u32) {
        self.info.tc().write(|w| w.set_count(timeout));
    }

    /// Set the warning interrupt value in clock cycles.
    ///
    /// # Arguments
    ///
    /// * `warning` - Number of clock cycles before warning interrupt.
    fn set_warning_value(&self, warning: u16) {
        self.info.warnint().write(|w| w.set_warnint(warning));
    }

    /// Lock the oscillator to prevent disabling or powering down the watchdog oscillator.
    fn lock_oscillator(&self) {
        self.info.mod_().modify(|w| w.set_lock(true));
    }
}

/// WWDT0 interrupt handler.
///
/// This handler is called when the watchdog warning interrupt fires.
/// When reset happens, the interrupt handler will never be reached.
pub struct InterruptHandler;

impl Handler<typelevel::WWDT0> for InterruptHandler {
    unsafe fn on_interrupt() {
        crate::perf_counters::incr_interrupt_wwdt();
        let wwdt = pac::WWDT0;

        if wwdt.mod_().read().wdtof() {
            #[cfg(feature = "defmt")]
            defmt::trace!("WWDT0: Timeout occurred");

            wwdt.mod_().modify(|w| w.set_wdtof(true));
        }

        if wwdt.mod_().read().wdint() {
            #[cfg(feature = "defmt")]
            defmt::trace!("WWDT0: Warning interrupt");

            wwdt.mod_().modify(|w| w.set_wdint(true));
        }
    }
}
