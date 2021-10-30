//! HAL interface to the WDT peripheral.
//!
//! This HAL implements a basic watchdog timer with 1..=8 handles.
//! Once the watchdog has been started, it cannot be stopped.

use crate::pac::WDT;
use crate::peripherals;

const MIN_TICKS: u32 = 15;

#[non_exhaustive]
pub struct Config {
    /// Number of 32768 Hz ticks in each watchdog period.
    ///
    /// Note: there is a minimum of 15 ticks (458 microseconds). If a lower
    /// number is provided, 15 ticks will be used as the configured value.
    pub timeout_ticks: u32,

    /// Should the watchdog continue to count during sleep modes?
    pub run_during_sleep: bool,

    /// Should the watchdog continue to count when the CPU is halted for debug?
    pub run_during_debug_halt: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout_ticks: 32768, // 1 second
            run_during_debug_halt: true,
            run_during_sleep: true,
        }
    }
}

/// An interface to the Watchdog.
pub struct Watchdog {
    _private: (),
}

impl Watchdog {
    /// Try to create a new watchdog instance from the peripheral.
    ///
    /// This function will return an error if the watchdog is already active
    /// with a `config` different to the requested one, or a different number of
    /// enabled handles.
    ///
    /// `N` must be between 1 and 8, inclusive.
    #[inline]
    pub fn try_new<const N: usize>(
        wdt: peripherals::WDT,
        config: Config,
    ) -> Result<(Self, [WatchdogHandle; N]), peripherals::WDT> {
        assert!(N >= 1 && N <= 8);

        let r = unsafe { &*WDT::ptr() };

        let crv = config.timeout_ticks.max(MIN_TICKS);
        let rren = (1u32 << N) - 1;

        #[cfg(not(feature = "_nrf9160"))]
        let runstatus = r.runstatus.read().runstatus().bit();
        #[cfg(feature = "_nrf9160")]
        let runstatus = r.runstatus.read().runstatuswdt().bit();

        if runstatus {
            let curr_config = r.config.read();
            if curr_config.halt().bit() != config.run_during_debug_halt
                || curr_config.sleep().bit() != config.run_during_sleep
                || r.crv.read().bits() != crv
                || r.rren.read().bits() != rren
            {
                return Err(wdt);
            }
        } else {
            r.config.write(|w| {
                w.sleep().bit(config.run_during_sleep);
                w.halt().bit(config.run_during_debug_halt);
                w
            });
            r.intenset.write(|w| w.timeout().set_bit());

            r.crv.write(|w| unsafe { w.bits(crv) });
            r.rren.write(|w| unsafe { w.bits(rren) });
            r.tasks_start.write(|w| unsafe { w.bits(1) });
        }

        let this = Self { _private: () };

        const DUMMY_HANDLE: WatchdogHandle = WatchdogHandle { index: 0 };
        let mut handles = [DUMMY_HANDLE; N];
        for i in 0..N {
            handles[i] = WatchdogHandle { index: i as u8 };
            handles[i].pet();
        }

        Ok((this, handles))
    }

    /// Enable the watchdog interrupt.
    ///
    /// NOTE: Although the interrupt will occur, there is no way to prevent
    /// the reset from occurring. From the time the event was fired, the
    /// system will reset two LFCLK ticks later (61 microseconds) if the
    /// interrupt has been enabled.
    #[inline(always)]
    pub fn enable_interrupt(&mut self) {
        let r = unsafe { &*WDT::ptr() };
        r.intenset.write(|w| w.timeout().set_bit());
    }

    /// Disable the watchdog interrupt.
    ///
    /// NOTE: This has no effect on the reset caused by the Watchdog.
    #[inline(always)]
    pub fn disable_interrupt(&mut self) {
        let r = unsafe { &*WDT::ptr() };
        r.intenclr.write(|w| w.timeout().set_bit());
    }

    /// Is the watchdog still awaiting pets from any handle?
    ///
    /// This reports whether sufficient pets have been received from all
    /// handles to prevent a reset this time period.
    #[inline(always)]
    pub fn awaiting_pets(&self) -> bool {
        let r = unsafe { &*WDT::ptr() };
        let enabled = r.rren.read().bits();
        let status = r.reqstatus.read().bits();
        (status & enabled) == 0
    }
}

pub struct WatchdogHandle {
    index: u8,
}

impl WatchdogHandle {
    /// Pet the watchdog.
    ///
    /// This function pets the given watchdog handle.
    ///
    /// NOTE: All active handles must be pet within the time interval to
    /// prevent a reset from occurring.
    #[inline]
    pub fn pet(&mut self) {
        let r = unsafe { &*WDT::ptr() };
        r.rr[self.index as usize].write(|w| w.rr().reload());
    }

    /// Has this handle been pet within the current window?
    pub fn is_pet(&self) -> bool {
        let r = unsafe { &*WDT::ptr() };
        let rd = r.reqstatus.read().bits();
        let idx = self.index as usize;
        ((rd >> idx) & 0x1) == 0
    }

    /// Steal a watchdog handle by index.
    ///
    /// Safety: watchdog must be initialized, index must be between 0 and N-1 where
    /// N is the handle count when initializing.
    pub unsafe fn steal(index: u8) -> Self {
        Self { index }
    }
}
