//! Watchdog Timer (WDT) driver.
//!
//! This HAL implements a basic watchdog timer with 1..=8 handles.
//! Once the watchdog has been started, it cannot be stopped.

#![macro_use]

use core::hint::unreachable_unchecked;

use embassy_hal_internal::PeripheralType;

use crate::pac::wdt::vals;
pub use crate::pac::wdt::vals::{Halt as HaltConfig, Sleep as SleepConfig};
use crate::{interrupt, pac, peripherals, Peri};

const MIN_TICKS: u32 = 15;

/// WDT configuration.
#[non_exhaustive]
pub struct Config {
    /// Number of 32768 Hz ticks in each watchdog period.
    ///
    /// Note: there is a minimum of 15 ticks (458 microseconds). If a lower
    /// number is provided, 15 ticks will be used as the configured value.
    pub timeout_ticks: u32,

    /// Should the watchdog continue to count during sleep modes?
    pub action_during_sleep: SleepConfig,

    /// Should the watchdog continue to count when the CPU is halted for debug?
    pub action_during_debug_halt: HaltConfig,
}

impl Config {
    /// Create a config structure from the current configuration of the WDT
    /// peripheral.
    pub fn try_new<T: Instance>(_wdt: &Peri<'_, T>) -> Option<Self> {
        let r = T::REGS;

        #[cfg(not(any(feature = "_nrf91", feature = "_nrf5340")))]
        let runstatus = r.runstatus().read().runstatus();
        #[cfg(any(feature = "_nrf91", feature = "_nrf5340"))]
        let runstatus = r.runstatus().read().runstatuswdt();

        if runstatus {
            let config = r.config().read();
            Some(Self {
                timeout_ticks: r.crv().read(),
                action_during_sleep: config.sleep(),
                action_during_debug_halt: config.halt(),
            })
        } else {
            None
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout_ticks: 32768, // 1 second
            action_during_debug_halt: HaltConfig::RUN,
            action_during_sleep: SleepConfig::RUN,
        }
    }
}

/// Watchdog driver.
pub struct Watchdog<T: Instance> {
    _wdt: Peri<'static, T>,
}

impl<T: Instance> Watchdog<T> {
    /// Try to create a new watchdog driver.
    ///
    /// This function will return an error if the watchdog is already active
    /// with a `config` different to the requested one, or a different number of
    /// enabled handles.
    ///
    /// `N` must be between 1 and 8, inclusive.
    #[inline]
    pub fn try_new<const N: usize>(
        wdt: Peri<'static, T>,
        config: Config,
    ) -> Result<(Self, [WatchdogHandle; N]), Peri<'static, T>> {
        assert!(N >= 1 && N <= 8);

        let r = T::REGS;

        let crv = config.timeout_ticks.max(MIN_TICKS);
        let rren = crate::pac::wdt::regs::Rren((1u32 << N) - 1);

        #[cfg(not(any(feature = "_nrf91", feature = "_nrf5340")))]
        let runstatus = r.runstatus().read().runstatus();
        #[cfg(any(feature = "_nrf91", feature = "_nrf5340"))]
        let runstatus = r.runstatus().read().runstatuswdt();

        if runstatus {
            let curr_config = r.config().read();
            if curr_config.halt() != config.action_during_debug_halt
                || curr_config.sleep() != config.action_during_sleep
                || r.crv().read() != crv
                || r.rren().read() != rren
            {
                return Err(wdt);
            }
        } else {
            r.config().write(|w| {
                w.set_sleep(config.action_during_sleep);
                w.set_halt(config.action_during_debug_halt);
            });
            r.intenset().write(|w| w.set_timeout(true));

            r.crv().write_value(crv);
            r.rren().write_value(rren);
            r.tasks_start().write_value(1);
        }

        let this = Self { _wdt: wdt };

        let mut handles = [const { WatchdogHandle { index: 0 } }; N];
        for i in 0..N {
            handles[i] = unsafe { WatchdogHandle::steal::<T>(i as u8) };
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
        T::REGS.intenset().write(|w| w.set_timeout(true));
    }

    /// Disable the watchdog interrupt.
    ///
    /// NOTE: This has no effect on the reset caused by the Watchdog.
    #[inline(always)]
    pub fn disable_interrupt(&mut self) {
        T::REGS.intenclr().write(|w| w.set_timeout(true));
    }

    /// Is the watchdog still awaiting pets from any handle?
    ///
    /// This reports whether sufficient pets have been received from all
    /// handles to prevent a reset this time period.
    #[inline(always)]
    pub fn awaiting_pets(&self) -> bool {
        let r = T::REGS;
        let enabled = r.rren().read().0;
        let status = r.reqstatus().read().0;
        (status & enabled) == 0
    }
}

/// Watchdog handle.
pub struct WatchdogHandle {
    index: u8,
}

impl WatchdogHandle {
    fn regs(&self) -> pac::wdt::Wdt {
        match self.index / 8 {
            #[cfg(not(feature = "_multi_wdt"))]
            peripherals::WDT::INDEX => peripherals::WDT::REGS,
            #[cfg(feature = "_multi_wdt")]
            peripherals::WDT0::INDEX => peripherals::WDT0::REGS,
            #[cfg(feature = "_multi_wdt")]
            peripherals::WDT1::INDEX => peripherals::WDT1::REGS,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    fn rr_index(&self) -> usize {
        usize::from(self.index % 8)
    }

    /// Pet the watchdog.
    ///
    /// This function pets the given watchdog handle.
    ///
    /// NOTE: All active handles must be pet within the time interval to
    /// prevent a reset from occurring.
    #[inline]
    pub fn pet(&mut self) {
        let r = self.regs();
        r.rr(self.rr_index()).write(|w| w.set_rr(vals::Rr::RELOAD));
    }

    /// Has this handle been pet within the current window?
    pub fn is_pet(&self) -> bool {
        let r = self.regs();
        !r.reqstatus().read().rr(self.rr_index())
    }

    /// Steal a watchdog handle by index.
    ///
    /// # Safety
    /// Watchdog must be initialized and `index` must be between `0` and `N-1`
    /// where `N` is the handle count when initializing.
    pub unsafe fn steal<T: Instance>(index: u8) -> Self {
        Self {
            index: T::INDEX * 8 + index,
        }
    }
}

pub(crate) trait SealedInstance {
    const REGS: pac::wdt::Wdt;
    const INDEX: u8;
}

/// WDT instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_wdt {
    ($type:ident, $pac_type:ident, $irq:ident, $index:literal) => {
        impl crate::wdt::SealedInstance for peripherals::$type {
            const REGS: pac::wdt::Wdt = pac::$pac_type;
            const INDEX: u8 = $index;
        }
        impl crate::wdt::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
