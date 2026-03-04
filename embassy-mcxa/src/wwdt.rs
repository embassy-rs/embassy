//! Windowed Watchdog Timer (WWDT) driver for MCXA microcontrollers.
//!
//! The WWDT is a hardware timer that can reset the system or generate an interrupt if the software fails to
//! periodically "feed" the watchdog within a specified time window. This helps detect
//! and recover from software failures or system hangs.
//!
//! The FRO12M provides a 1 MHz clock (clk_1m) used as WWDT0 independant clock source. This clock is / 4 by an internal fixed divider.

use core::marker::PhantomData;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_time::Duration;
use paste::paste;

use crate::clocks::periph_helpers::Clk1MConfig;
use crate::clocks::{ClockError, Gate, WakeGuard, enable_and_reset};
use crate::interrupt::typelevel;
use crate::interrupt::typelevel::{Handler, Interrupt};
use crate::pac;
use crate::pac::wwdt::vals::{Wden, Wdprotect, Wdreset};

/// WWDT0 Error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Clock configuration error.
    ClockSetup(ClockError),
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
    /// * `_irq` - Interrupt binding for WWDT0
    /// * `config - WWDT config with timeout and optional warning value
    pub fn new<T: Instance>(
        _peri: Peri<'d, T>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        let parts = unsafe { enable_and_reset::<T>(&Clk1MConfig).map_err(Error::ClockSetup)? };

        let watchdog = Self {
            info: T::info(),
            _phantom: PhantomData,
            _wg: parts.wake_guard,
        };

        let frequency = parts.freq / 4;
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

        T::Interrupt::unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe {
            T::Interrupt::enable();
        }

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
            self.info.regs().feed().write(|w| w.set_feed(0xAA));
            self.info.regs().feed().write(|w| w.set_feed(0x55));
        });
    }

    /// Enable the watchdog timer.
    /// Function is blocking until the watchdog is actually started.
    fn enable(&self) {
        self.info.regs().mod_().modify(|w| w.set_wden(Wden::RUN));
        while self.info.regs().tc().read().count() == 0xFF {}
    }

    /// Set the watchdog protection mode to flexible.
    fn set_flexible_mode(&self) {
        self.info.regs().mod_().modify(|w| w.set_wdprotect(Wdprotect::FLEXIBLE));
    }

    /// Enable interrupt mode.
    fn enable_interrupt(&self) {
        self.info.regs().mod_().modify(|w| w.set_wdreset(Wdreset::INTERRUPT));
    }

    /// Enable reset mode.
    fn enable_reset(&self) {
        self.info.regs().mod_().modify(|w| w.set_wdreset(Wdreset::RESET));
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

trait SealedInstance: Gate<MrccPeriphConfig = Clk1MConfig> {
    fn info() -> &'static Info;
}

/// WWDT Instance
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this WWDT instance.
    type Interrupt: typelevel::Interrupt;
}

struct Info {
    regs: pac::wwdt::Wwdt,
}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac::wwdt::Wwdt {
        self.regs
    }
}

unsafe impl Sync for Info {}

macro_rules! impl_instance {
    ($($n:literal);*) => {
        $(
            paste!{
                impl SealedInstance for crate::peripherals::[<WWDT $n>] {
                    fn info() -> &'static Info {
                        static INFO: Info = Info {
                            regs: pac::[<WWDT $n>],
                        };
                        &INFO
                    }
                }

                impl Instance for crate::peripherals::[<WWDT $n>] {
                    type Interrupt = crate::interrupt::typelevel::[<WWDT $n>];
                }
            }
        )*
    };
}

impl_instance!(0);

#[cfg(feature = "mcxa5xx")]
impl_instance!(1);
