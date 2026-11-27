//! Watchdog
//!
//! The watchdog is a countdown timer that can restart parts of the chip if it reaches zero. This can be used to restart the
//! processor if software gets stuck in an infinite loop. The programmer must periodically write a value to the watchdog to
//! stop it from reaching zero.
//!
//! Credit: based on `rp-hal` implementation (also licensed Apache+MIT)

use core::marker::PhantomData;

use embassy_time::Duration;

use crate::peripherals::WATCHDOG;
use crate::{Peri, pac};

// Every stage of the power-on state machine (PSM), as laid out in `PSM.WDSEL`.
#[cfg(feature = "rp2040")]
const PSM_ALL_STAGES: pac::psm::regs::Wdsel = pac::psm::regs::Wdsel(0x0001_ffff);
#[cfg(feature = "_rp235x")]
const PSM_ALL_STAGES: pac::psm::regs::Wdsel = pac::psm::regs::Wdsel(0x01ff_ffff);

/// The reason for a system reset from the watchdog.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResetReason {
    /// The reset was forced.
    Forced,
    /// The watchdog was not fed in time.
    TimedOut,
}

/// Watchdog peripheral
pub struct Watchdog<'d> {
    phantom: PhantomData<&'d mut WATCHDOG>,
}

impl<'d> Watchdog<'d> {
    /// Create a new `Watchdog`
    pub fn new(_watchdog: Peri<'d, WATCHDOG>) -> Self {
        Self { phantom: PhantomData }
    }

    /// Start tick generation on clk_tick which is driven from clk_ref.
    ///
    /// # Arguments
    ///
    /// * `cycles` - Total number of tick cycles before the next tick is generated.
    ///   It is expected to be the frequency in MHz of clk_ref.
    #[cfg(feature = "rp2040")]
    pub fn enable_tick_generation(&mut self, cycles: u8) {
        let watchdog = pac::WATCHDOG;
        watchdog.tick().write(|w| {
            w.set_enable(true);
            w.set_cycles(cycles.into())
        });
    }

    /// Defines whether or not the watchdog timer should be paused when processor(s) are in debug mode
    /// or when JTAG is accessing bus fabric
    pub fn pause_on_debug(&mut self, pause: bool) {
        let watchdog = pac::WATCHDOG;
        watchdog.ctrl().modify(|w| {
            w.set_pause_dbg0(pause);
            w.set_pause_dbg1(pause);
            w.set_pause_jtag(pause);
        })
    }

    fn load_counter(&self, counter: u32) {
        let watchdog = pac::WATCHDOG;
        watchdog.load().write_value(pac::watchdog::regs::Load(counter));
    }

    fn enable(&self, bit: bool) {
        let watchdog = pac::WATCHDOG;
        watchdog.ctrl().modify(|w| w.set_enable(bit))
    }

    // Configure which hardware will be reset by the watchdog.
    //
    // The PSM restarts its sequence at the earliest stage selected in WDSEL and resets every
    // later stage with it, so the earliest selected stage decides how much of the chip resets.
    fn configure_wdog_reset_triggers(&self) {
        let mut wdsel = PSM_ALL_STAGES;
        // Everything except ROSC and XOSC: the sequence restarts at CLOCKS.
        #[cfg(feature = "rp2040")]
        {
            wdsel.set_rosc(false);
            wdsel.set_xosc(false);
        }
        // Everything except the processor cold reset domain, which holds the debug state: the
        // sequence restarts at OTP. The bootrom's `reboot()` makes the same selection.
        #[cfg(feature = "_rp235x")]
        wdsel.set_proc_cold(false);
        pac::PSM.wdsel().write_value(wdsel);
    }

    /// Feed the watchdog timer
    pub fn feed(&mut self, timeout: Duration) {
        #[cfg(feature = "rp2040")]
        const MAX_PERIOD: u32 = 0xFFFFFF / 2;
        #[cfg(feature = "_rp235x")]
        const MAX_PERIOD: u32 = 0xFFFFFF;

        let timeout_us = timeout.as_micros();
        if timeout_us > (MAX_PERIOD) as u64 {
            panic!("Period cannot exceed {} microseconds", MAX_PERIOD);
        }
        let timeout_us = timeout_us as u32;

        // Due to a logic error, the watchdog decrements by 2 and
        // the load value must be compensated; see RP2040-E1
        // This errata is fixed in the RP235x
        let load_value = if cfg!(feature = "rp2040") {
            timeout_us * 2
        } else {
            timeout_us
        };

        self.load_counter(load_value)
    }

    /// Start the watchdog timer
    pub fn start(&mut self, initial_timeout: Duration) {
        self.enable(false);
        self.configure_wdog_reset_triggers();
        self.feed(initial_timeout);
        self.enable(true);
    }

    /// Stop the watchdog timer
    pub fn stop(&mut self) {
        self.enable(false);
    }

    /// Trigger a system reset
    pub fn trigger_reset(&mut self) {
        self.configure_wdog_reset_triggers();
        self.pause_on_debug(false);
        self.enable(true);
        let watchdog = pac::WATCHDOG;
        watchdog.ctrl().write(|w| {
            w.set_trigger(true);
        })
    }

    /// Store data in scratch register
    pub fn set_scratch(&mut self, index: usize, value: u32) {
        let watchdog = pac::WATCHDOG;
        match index {
            0 => watchdog.scratch0().write(|w| *w = value),
            1 => watchdog.scratch1().write(|w| *w = value),
            2 => watchdog.scratch2().write(|w| *w = value),
            3 => watchdog.scratch3().write(|w| *w = value),
            4 => watchdog.scratch4().write(|w| *w = value),
            5 => watchdog.scratch5().write(|w| *w = value),
            6 => watchdog.scratch6().write(|w| *w = value),
            7 => watchdog.scratch7().write(|w| *w = value),
            _ => panic!("Invalid watchdog scratch index"),
        }
    }

    /// Read data from scratch register
    pub fn scratch(&mut self, index: usize) -> u32 {
        let watchdog = pac::WATCHDOG;
        match index {
            0 => watchdog.scratch0().read(),
            1 => watchdog.scratch1().read(),
            2 => watchdog.scratch2().read(),
            3 => watchdog.scratch3().read(),
            4 => watchdog.scratch4().read(),
            5 => watchdog.scratch5().read(),
            6 => watchdog.scratch6().read(),
            7 => watchdog.scratch7().read(),
            _ => panic!("Invalid watchdog scratch index"),
        }
    }

    /// Get the reason for the last system reset, if it was caused by the watchdog.
    pub fn reset_reason(&self) -> Option<ResetReason> {
        let watchdog = pac::WATCHDOG;
        let reason = watchdog.reason().read();
        if reason.force() {
            Some(ResetReason::Forced)
        } else if reason.timer() {
            Some(ResetReason::TimedOut)
        } else {
            None
        }
    }
}
