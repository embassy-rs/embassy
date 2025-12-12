//! Windowed Watchdog Timer (WWDT) driver for MCXA microcontrollers.
//!
//! The WWDT is a hardware timer that can reset the system or generate an interrupt if the software fails to
//! periodically "feed" the watchdog within a specified time window. This helps detect
//! and recover from software failures or system hangs.
//! The FRO12M provides a 1 MHz clock (clk_1m) used as WWDT0 independant clock source. This clock is / 4 by an internal fixed divider.
//!

use crate::interrupt::typelevel;
use crate::interrupt::typelevel::Handler;
use crate::pac;
use crate::peripherals::WWDT0;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_hal_internal::Peri;
use embassy_time::Duration;

/// Watchdog peripheral
pub struct Watchdog<'d> {
    _peri: Peri<'d, WWDT0>,
    load_value: u32,
    frequency: u32,
    // The register block of the WWDT instance
    info: &'static pac::wwdt0::RegisterBlock,
}

impl<'d> Watchdog<'d> {
    /// Create a new WWDT instance.
    ///
    /// # Arguments
    ///
    /// * `_peri` - The WWDT0 peripheral instance
    /// * `_irq` - Interrupt binding for WWDT0
    pub fn new(
        _peri: Peri<'d, WWDT0>,
        _irq: impl crate::interrupt::typelevel::Binding<typelevel::WWDT0, InterruptHandler> + 'd,
    ) -> Self {
        let _frequency = crate::clocks::with_clocks(|clocks| {
            // Ensure clk_1m is active at the required power level
            clocks.ensure_clk_1m_active(&crate::clocks::PoweredClock::NormalEnabledDeepSleepDisabled)
        })
        .expect("Clocks not initialized")
        .expect("clk_1m not enabled or not at required power level");

        // Enable WATCHDOG clock by writing to mrcc register
        // Can't use enable_and_reset API here because WWDT doesn't have a reset signal.
        let mrcc = unsafe { pac::Mrcc0::steal() };
        mrcc.mrcc_glb_cc0().modify(|_, w| w.wwdt0().enabled());

        crate::pac::Interrupt::WWDT0.unpend();
        unsafe {
            crate::pac::Interrupt::WWDT0.enable();
        }
        Self {
            _peri,
            load_value: 0,
            frequency: _frequency / 4,
            info: unsafe { &*pac::Wwdt0::ptr() },
        }
    }

    // Start the watchdog timer with the specified timeout period.
    ///
    /// # Arguments
    ///
    /// * `period` - The timeout period after which the watchdog will trigger
    /// * `warning` - Optional warning interrupt period before timeout. If `Some`, an interrupt
    ///               will be generated at this time before the actual timeout. If `None`, the
    ///               watchdog will directly reset the system on timeout.
    ///
    pub fn start(&mut self, period: Duration, warning: Option<Duration>) {
        let timeout_cycles = (self.frequency as u64 * period.as_micros()) / 1_000_000;

        // Ensure the value fits in u32 and is within valid range
        // Writing a value below FFh causes 00_00FFh to load into the register. Therefore, the minimum timeout interval is TWDCLK X 256 X 4.
        assert!(timeout_cycles <= 0xFFFFFF, "Timeout too large for watchdog counter");
        assert!(timeout_cycles > 0xFF, "Timeout too small for watchdog counter");

        self.load_value = timeout_cycles as u32;
        self.set_timeout_value(self.load_value);

        //Windows value is set to max at reset for no effect.

        if let Some(warning_value) = warning {
            let warning_cycles = (self.frequency as u64 * warning_value.as_micros()) / 1_000_000;
            assert!(
                warning_cycles <= 0x3FF,
                "Warning value too large. At 1MHz, warning value must be <= 4.092ms."
            );

            self.set_warning_value(warning_cycles as u16);
            self.enable_interrupt();
        } else {
            self.enable_reset();
        }

        self.lock_oscillator();
        self.enable(true);
        self.feed();

        // Set the WDPROTECT bit to false after the Feed Sequence (0xAA, 0x55)
        self.set_protect_mode(false);
    }

    /// Feed the watchdog to prevent timeout.
    ///
    /// This must be called periodically before the timeout period expires to prevent
    /// the watchdog from triggering a reset or interrupt.
    pub fn feed(&self) {
        //TBD To avoid an unintended interrupt, it is a good practice to disable interrupts around a feed sequence
        self.info.feed().write(|w| unsafe { w.feed().bits(0xAA) });
        self.info.feed().write(|w| unsafe { w.feed().bits(0x55) });
    }

    /// Enable or disable the watchdog timer.
    ///
    /// # Arguments
    ///
    /// * `bit` - `true` to enable (run), `false` to disable (stop).
    fn enable(&self, bit: bool) {
        self.info
            .mod_()
            .modify(|_, w| if bit { w.wden().run() } else { w.wden().stop() });
        while self.info.tc().read().count() == 0xFF {}
    }

    /// Set the watchdog protection mode.
    ///
    /// # Arguments
    ///
    /// * `bit` - `true` for threshold mode (protected), `false` for flexible mode.
    fn set_protect_mode(&self, bit: bool) {
        self.info.mod_().modify(|_, w| {
            if bit {
                w.wdprotect().threshold()
            } else {
                w.wdprotect().flexible()
            }
        });
    }

    /// Enable interrupt mode.
    fn enable_interrupt(&self) {
        self.info.mod_().modify(|_, w| w.wdreset().interrupt());
    }

    /// Enable reset mode.
    fn enable_reset(&self) {
        self.info.mod_().modify(|_, w| w.wdreset().reset());
    }

    /// Set the timeout value in clock cycles.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Number of clock cycles before timeout.
    fn set_timeout_value(&self, timeout: u32) {
        self.info.tc().write(|w| unsafe { w.count().bits(timeout) });
    }

    /// Set the warning interrupt value in clock cycles.
    ///
    /// # Arguments
    ///
    /// * `warning` - Number of clock cycles before warning interrupt.
    fn set_warning_value(&self, warning: u16) {
        self.info.warnint().write(|w| unsafe { w.warnint().bits(warning) });
    }

    /// Set the window value in clock cycles.
    ///
    /// # Arguments
    ///
    /// * `window` - Minimum number of clock cycles before feeding is allowed.
    fn set_window_value(&self, window: u32) {
        self.info.window().write(|w| unsafe { w.window().bits(window) });
    }

    /// Lock the oscillator to prevent disabling or powering down the watchdog oscillator.
    fn lock_oscillator(&self) {
        self.info.mod_().modify(|_, w| w.lock().lock());
    }
}

// Get the watchdog status flags.
///
/// # Arguments
///
/// * `regs` - Reference to the WWDT register block
///
/// # Returns
///
/// Bitmask containing WDTOF (bit 2) and WDINT (bit 3) flags.
unsafe fn get_status_flag(regs: &pac::wwdt0::RegisterBlock) -> u32 {
    regs.mod_().read().bits() & (0xC)
}

/// Clear the watchdog status flags.
///
/// # Arguments
///
/// * `regs` - Reference to the WWDT register block
/// * `flag` - Bitmask of flags to clear
unsafe fn clear_status_flag(regs: &pac::wwdt0::RegisterBlock, flag: u32) {
    regs.mod_().modify(|_, w| unsafe { w.bits(flag) });
}

/// WWDT0 interrupt handler.
///
/// This handler is called when the watchdog warning interrupt fires.
/// When reset happens, the interrupt handler will never be reached.
pub struct InterruptHandler;

impl Handler<typelevel::WWDT0> for InterruptHandler {
    unsafe fn on_interrupt() {
        let wwdt = &*pac::Wwdt0::ptr();
        let flag = get_status_flag(wwdt);
        defmt::info!("WWDT0 Interrupt Handler : Timeout happened");
        clear_status_flag(wwdt, flag);
    }
}
