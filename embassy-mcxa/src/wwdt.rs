//! Windowed Watchdog Timer (WWDT) driver for MCXA microcontrollers.
//!
//! The WWDT is a hardware timer that can reset the system or generate an interrupt if the software fails to
//! periodically "feed" the watchdog within a specified time window. This helps detect
//! and recover from software failures or system hangs.
//!
//! WWDT0 is hardwired to `clk_1m`, the 1 MHz clock derived from FRO12M. On MCXA5xx, WWDT1
//! instead has a source mux and can additionally be driven from `clk_16k` or `fro_hf_div`; see
//! [`ClockConfig`]. Both instances have a `CLKDIV` pre-divider, and both then divide by a further
//! fixed factor of 4 inside the peripheral.
//!
//! An armed watchdog cannot be stopped by dropping its handle. Its active-only
//! clock requirement therefore remains until reset, even if the handle is
//! dropped. There is deliberately no watchdog suspension API.
//!
//! Reset-mode construction masks the selected instance's interrupt so its
//! handler cannot interfere with reset. Warning mode enables that interrupt.

#[cfg(feature = "embedded-mcu-hal")]
use core::convert::Infallible;
use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_time::Duration;

use crate::clocks::periph_helpers::{Div4, WwdtClockSel, WwdtConfig, WwdtInstance};
use crate::clocks::{ClockError, Gate, PoweredClock, WakeGuard, enable_and_reset};
use crate::interrupt::typelevel;
use crate::interrupt::typelevel::{Handler, Interrupt};
use crate::pac;
use crate::pac::wwdt::{Wden, Wdprotect, Wdreset};

/// WWDT error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Clock configuration error.
    ClockSetup(ClockError),
    TimeoutTooSmall,
    TimeoutTooLarge,
    WarningTooLarge,
}

/// WWDT clock configuration
///
/// The resolved frequency is divided by the WWDT's own fixed divide-by-4
/// prescaler (MCXA5xx RM Rev 1 34.2.2, Figure 172) before it reaches the
/// counter, so the tick rate is `source / div / 4`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockConfig {
    /// Powered clock configuration
    pub power: PoweredClock,
    /// WWDT clock source
    ///
    /// WWDT0 is hardwired to `clk_1m` and accepts only
    /// [`WwdtClockSel::Clk1M`]; anything else is rejected with
    /// [`Error::ClockSetup`].
    pub source: WwdtClockSel,
    /// WWDT pre-divider
    pub div: Div4,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            // `clk_1m` is WWDT0's only source, and is the frequency the driver
            // assumed for both instances before the source became selectable.
            source: WwdtClockSel::Clk1M,
            div: const { Div4::no_div() },
        }
    }
}

/// WWDT configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// The timeout period after which the watchdog will trigger
    pub timeout: Duration,
    pub warning: Option<Duration>,
    /// Clock configuration
    pub clock: ClockConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(1),
            warning: None,
            clock: ClockConfig::default(),
        }
    }
}

/// Watchdog peripheral
pub struct Watchdog<'d> {
    info: &'static Info,
    _phantom: PhantomData<&'d mut ()>,
    _wg: Option<WakeGuard>,
}

impl<'d> Watchdog<'d> {
    /// Create a new WWDT instance.
    ///
    /// Configure the WWDT, enables the interrupt, set the timeout and or warning value.
    ///
    /// # Arguments
    ///
    /// * `_peri` - The WWDT peripheral instance
    /// * `_irq` - Interrupt binding for the selected WWDT instance
    /// * `config` - WWDT config with timeout and optional warning value
    pub fn new<T: Instance>(
        _peri: Peri<'d, T>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        let ClockConfig { power, source, div } = config.clock;

        // Enable clocks
        let conf = WwdtConfig {
            power,
            source,
            div,
            instance: T::CLOCK_INSTANCE,
        };

        let parts = unsafe { enable_and_reset::<T>(&conf).map_err(Error::ClockSetup)? };

        let watchdog = Self {
            info: T::info(),
            _phantom: PhantomData,
            _wg: parts.wake_guard,
        };

        let timeout_cycles = counter_ticks(parts.freq, config.timeout);

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
            let warning_cycles = counter_ticks(parts.freq, warning_value);
            if warning_cycles > 0x3FF {
                return Err(Error::WarningTooLarge);
            }

            watchdog.set_warning_value(warning_cycles as u16);
            watchdog.enable_interrupt();
        } else {
            watchdog.enable_reset();
        }

        T::Interrupt::unpend();

        if config.warning.is_some() {
            // SAFETY: `_irq` ensures an interrupt handler exists.
            unsafe { T::Interrupt::enable() };
        } else {
            T::Interrupt::disable();
        }

        Ok(watchdog)
    }

    /// Start the watchdog timer with the specified timeout period.
    pub fn start(&mut self) {
        self.lock_oscillator();
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
            self.info.regs().feed().write(|w| w.set_feed(0xAA));
            self.info.regs().feed().write(|w| w.set_feed(0x55));
        });
    }

    /// Enable the watchdog timer.
    /// Function is blocking until the watchdog is actually started.
    fn enable(&self) {
        self.info.regs().mod_().modify(|w| w.set_wden(Wden::Run));
        while self.info.regs().tc().read().count() == 0xFF {}
    }

    /// Set the watchdog protection mode to flexible.
    fn set_flexible_mode(&self) {
        self.info.regs().mod_().modify(|w| w.set_wdprotect(Wdprotect::Flexible));
    }

    /// Enable interrupt mode.
    fn enable_interrupt(&self) {
        self.info.regs().mod_().modify(|w| w.set_wdreset(Wdreset::Interrupt));
    }

    /// Enable reset mode.
    fn enable_reset(&self) {
        self.info.regs().mod_().modify(|w| w.set_wdreset(Wdreset::Reset));
    }

    /// Set the timeout value in clock cycles.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Number of clock cycles before timeout.
    fn set_timeout_value(&self, timeout: u32) {
        self.info.regs().tc().write(|w| w.set_count(timeout));
    }

    /// Set the warning interrupt value in clock cycles.
    ///
    /// # Arguments
    ///
    /// * `warning` - Number of clock cycles before warning interrupt.
    fn set_warning_value(&self, warning: u16) {
        self.info.regs().warnint().write(|w| w.set_warnint(warning));
    }

    /// Lock the oscillator to prevent disabling or powering down the watchdog oscillator.
    fn lock_oscillator(&self) {
        self.info.regs().mod_().modify(|w| w.set_lock(true));
    }
}

fn counter_ticks(clock_hz: u32, duration: Duration) -> u64 {
    (u64::from(clock_hz / 4) * duration.as_micros()) / 1_000_000
}

impl Drop for Watchdog<'_> {
    fn drop(&mut self) {
        if self.info.regs().mod_().read().wden() == Wden::Run {
            // Hardware outlives the handle and cannot be stopped until reset.
            core::mem::forget(self._wg.take());
        }
    }
}

#[cfg(test)]
mod power_tests {
    extern crate std;

    use super::*;
    use std::boxed::Box;

    fn test_watchdog(power: PoweredClock) -> Watchdog<'static> {
        let memory = Box::leak(Box::new([0u32; 64]));
        // SAFETY: this isolated aligned allocation covers the WWDT registers.
        let regs = unsafe { crate::pac::wwdt::Wwdt::from_ptr(memory.as_mut_ptr().cast()) };
        Watchdog {
            info: Box::leak(Box::new(Info { regs })),
            _phantom: PhantomData,
            _wg: WakeGuard::for_power(&power),
        }
    }

    #[test]
    fn unarmed_watchdog_releases_its_requirement_on_drop() {
        critical_section::with(|cs| {
            let watchdog = test_watchdog(PoweredClock::NormalEnabledDeepSleepDisabled);
            assert!(crate::clocks::active_wake_guards(&cs));
            drop(watchdog);
            assert!(!crate::clocks::active_wake_guards(&cs));
        });
    }

    #[test]
    fn armed_watchdog_keeps_its_requirement_after_handle_drop() {
        critical_section::with(|cs| {
            let watchdog = test_watchdog(PoweredClock::NormalEnabledDeepSleepDisabled);
            watchdog.info.regs().mod_().modify(|w| w.set_wden(Wden::Run));
            // SAFETY: the armed watchdog's Drop forgets the original guard.
            // This copy is dropped only afterward to balance the test counter.
            let cleanup = unsafe { core::ptr::read(watchdog._wg.as_ref().unwrap()) };
            drop(watchdog);
            assert!(crate::clocks::active_wake_guards(&cs));
            drop(cleanup);
            assert!(!crate::clocks::active_wake_guards(&cs));
        });
    }

    #[test]
    fn retained_watchdog_can_be_fed_without_inhibiting_deep_sleep() {
        critical_section::with(|cs| {
            let watchdog = test_watchdog(PoweredClock::AlwaysEnabled);
            watchdog.info.regs().mod_().modify(|w| {
                w.set_wden(Wden::Run);
                w.set_wdreset(Wdreset::Reset);
            });
            watchdog.feed();
            assert_eq!(watchdog.info.regs().mod_().read().wden(), Wden::Run);
            assert_eq!(watchdog.info.regs().mod_().read().wdreset(), Wdreset::Reset);
            assert!(!crate::clocks::active_wake_guards(&cs));
            drop(watchdog);
            assert!(!crate::clocks::active_wake_guards(&cs));
        });
    }

    #[test]
    fn fro16k_preserves_sixty_second_timeout_and_thirty_second_feed_margin() {
        assert_eq!(counter_ticks(16_384, Duration::from_secs(60)), 245_760);
        assert_eq!(counter_ticks(16_384, Duration::from_secs(30)), 122_880);
    }

    #[test]
    fn clk1m_timeout_scaling_is_unchanged() {
        assert_eq!(counter_ticks(1_000_000, Duration::from_secs(60)), 15_000_000);
    }
}

/// WWDT interrupt handler.
///
/// This handler is called when the watchdog warning interrupt fires.
/// When reset happens, the interrupt handler will never be reached.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        crate::perf_counters::incr_interrupt_wwdt();
        if T::info().regs().mod_().read().wdtof() {
            #[cfg(feature = "defmt")]
            defmt::trace!("WWDT0: Timeout occurred");

            T::info().regs().mod_().modify(|w| w.set_wdtof(true));
        }

        if T::info().regs().mod_().read().wdint() {
            #[cfg(feature = "defmt")]
            defmt::trace!("T::INFO().REGS()0: Warning interrupt");

            T::info().regs().mod_().modify(|w| w.set_wdint(true));
        }
    }
}

pub(crate) trait SealedInstance: Gate<MrccPeriphConfig = WwdtConfig> {
    /// Clock instance
    const CLOCK_INSTANCE: WwdtInstance;

    fn info() -> &'static Info;
}

/// WWDT Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this WWDT instance.
    type Interrupt: typelevel::Interrupt;
}

pub(crate) struct Info {
    pub(crate) regs: pac::wwdt::Wwdt,
}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::wwdt::Wwdt {
        self.regs
    }
}

unsafe impl Sync for Info {}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_wwdt_instance {
    ($n:literal) => {
        paste::paste! {
            impl $crate::wwdt::SealedInstance for $crate::peripherals::[<WWDT $n>] {
                const CLOCK_INSTANCE: $crate::clocks::periph_helpers::WwdtInstance =
                    $crate::clocks::periph_helpers::WwdtInstance::[<Wwdt $n>];

                fn info() -> &'static $crate::wwdt::Info {
                    static INFO: $crate::wwdt::Info = $crate::wwdt::Info {
                        regs: $crate::pac::[<WWDT $n>],
                    };
                    &INFO
                }
            }

            impl $crate::wwdt::Instance for $crate::peripherals::[<WWDT $n>] {
                type Interrupt = $crate::interrupt::typelevel::[<WWDT $n>];
            }
        }
    };
}

#[cfg(feature = "embedded-mcu-hal")]
impl embedded_mcu_hal::watchdog::Watchdog for Watchdog<'_> {
    type Error = Infallible;

    fn feed(&mut self) -> Result<(), Self::Error> {
        Self::feed(self);
        Ok(())
    }
}
