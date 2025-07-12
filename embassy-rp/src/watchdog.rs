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
use crate::{pac, Peri};

/// The reason for a system reset from the watchdog.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ResetReason {
    /// The reset was forced.
    Forced,
    /// The watchdog was not fed in time.
    TimedOut,
}

/// Watchdog peripheral
pub struct Watchdog {
    phantom: PhantomData<WATCHDOG>,
    load_value: u32, // decremented by 2 per tick (µs)
}

impl Watchdog {
    /// Create a new `Watchdog`
    pub fn new(_watchdog: Peri<'static, WATCHDOG>) -> Self {
        Self {
            phantom: PhantomData,
            load_value: 0,
        }
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

    // Configure which hardware will be reset by the watchdog
    // (everything except ROSC, XOSC)
    fn configure_wdog_reset_triggers(&self) {
        let psm = pac::PSM;
        psm.wdsel().write_value(pac::psm::regs::Wdsel(
            0x0001ffff & !(0x01 << 0usize) & !(0x01 << 1usize),
        ));
    }

    /// Feed the watchdog timer
    pub fn feed(&mut self) {
        self.load_counter(self.load_value)
    }

    /// Start the watchdog timer
    pub fn start(&mut self, period: Duration) {
        #[cfg(feature = "rp2040")]
        const MAX_PERIOD: u32 = 0xFFFFFF / 2;
        #[cfg(feature = "_rp235x")]
        const MAX_PERIOD: u32 = 0xFFFFFF;

        let delay_us = period.as_micros();
        if delay_us > (MAX_PERIOD) as u64 {
            panic!("Period cannot exceed {} microseconds", MAX_PERIOD);
        }
        let delay_us = delay_us as u32;

        // Due to a logic error, the watchdog decrements by 2 and
        // the load value must be compensated; see RP2040-E1
        // This errata is fixed in the RP235x
        if cfg!(feature = "rp2040") {
            self.load_value = delay_us * 2;
        } else {
            self.load_value = delay_us;
        }

        self.enable(false);
        self.configure_wdog_reset_triggers();
        self.load_counter(self.load_value);
        self.enable(true);
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
    pub fn get_scratch(&mut self, index: usize) -> u32 {
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
