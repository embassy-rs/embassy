//! Low-power timer in time-counter mode.
//!
//! The 32-bit counter can reset on compare or free-run until overflow. Reading
//! it latches a hardware snapshot. Reconfiguring a period stops/resets the
//! counter; async wait cancellation also stops it. Pulse counting and DMA are
//! not supported. LPTMR stops counting while the core is halted by a debugger.

use embassy_hal_internal::drop::OnDrop;
use embassy_time::Duration;
use maitake_sync::WaitCell;

use crate::clocks::periph_helpers::{Div4, LptmrClockSel, LptmrConfig, SPConfHelper};
use crate::clocks::{ClockError, PoweredClock, WakeGuard, with_clocks};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::lptmr::{Pbyp, Pcs, Prescale, Ten, Tfc, Tie};
use crate::{Peri, interrupt, pac, peripherals};

#[cfg(feature = "time-driver-lptmr")]
pub(crate) mod time_driver;

/// Counter prescaler: bypass, or division by 2^exponent (exponent 1..=16).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Prescaler(u8);

impl Prescaler {
    /// Count each functional input clock edge.
    pub const fn bypass() -> Self {
        Self(0)
    }

    /// Divide by 2^exponent. Returns None outside 1..=16.
    pub const fn divide(exponent: u8) -> Option<Self> {
        if exponent >= 1 && exponent <= 16 {
            Some(Self(exponent))
        } else {
            None
        }
    }

    /// Counter clock divisor.
    pub const fn divisor(self) -> u32 {
        1 << self.0
    }
}

/// Timer configuration.
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub struct Config {
    /// Required clock power state.
    pub power: PoweredClock,
    /// Functional input clock.
    pub source: LptmrClockSel,
    /// MRCC clock divider (not used by direct FRO16K).
    pub div: Div4,
    /// Counter clock prescaler.
    /// The complete divider chain must produce an integral-Hz counter clock.
    pub prescaler: Prescaler,
    /// Keep counting after compare instead of resetting on compare.
    pub free_running: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: LptmrClockSel::Clk1M,
            div: Div4::no_div(),
            prescaler: Prescaler::bypass(),
            free_running: false,
        }
    }
}

/// Timer error.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Clock setup failed.
    ClockSetup(ClockError),
    /// Period is zero or exceeds the supported u32 tick range.
    InvalidPeriod,
    /// Timer has not been started.
    NotRunning,
}

static WAIT_CELL: WaitCell = WaitCell::new();

/// LPTMR compare interrupt handler. Bind it to LPTMR0.
pub struct InterruptHandler;

impl interrupt::typelevel::Handler<interrupt::typelevel::LPTMR0> for InterruptHandler {
    unsafe fn on_interrupt() {
        if pac::LPTMR0.csr().read().tcf() {
            // Leave TCF latched for the waiting future. Never acknowledge it
            // accidentally through a read-modify-write of CSR.
            pac::LPTMR0.csr().modify(|w| {
                w.set_tcf(false);
                w.set_tie(Tie::Tie0);
            });
            WAIT_CELL.wake();
        }
    }
}

/// Owned low-power timer.
pub struct Lptmr<'d> {
    _peri: Peri<'d, peripherals::LPTMR0>,
    _wake_guard: Option<WakeGuard>,
    freq: u32,
    free_running: bool,
}

pub(crate) fn configure(config: Config) -> Result<crate::clocks::periph_helpers::PreEnableParts, Error> {
    // No MRCC reset exists; TEN=0 must precede PSR/CMR configuration even
    // after a warm reset (MCXA2 35.3.3, MCXA5 40.3.3).
    pac::LPTMR0.csr().write(|_| {});
    let clock = LptmrConfig {
        power: config.power,
        source: config.source,
        div: config.div,
        prescaler: config.prescaler,
    };
    let parts = with_clocks(move |clocks| clock.pre_enable_config(clocks))
        .ok_or(Error::ClockSetup(ClockError::NeverInitialized))?
        .map_err(Error::ClockSetup)?;
    pac::LPTMR0.psr().write(|w| {
        w.set_pcs(if config.source == LptmrClockSel::Clk16K {
            Pcs::Pcs01
        } else {
            Pcs::Pcs11
        });
        w.set_pbyp(if config.prescaler.0 == 0 {
            Pbyp::Pbyp1
        } else {
            Pbyp::Pbyp0
        });
        w.set_prescale(Prescale::from_bits(config.prescaler.0.saturating_sub(1)));
    });
    Ok(parts)
}

/// Latch and read the hardware counter. Call under a critical section to
/// prevent an ISR from replacing the snapshot between the write and read.
pub(crate) fn read_counter() -> u32 {
    pac::LPTMR0.cnr().write(|w| w.set_counter(0));
    pac::LPTMR0.cnr().read().counter()
}

impl<'d> Lptmr<'d> {
    /// Create a timer and enable its NVIC interrupt. The peripheral interrupt
    /// itself remains masked until an async wait is polled.
    pub fn new(
        peri: Peri<'d, peripherals::LPTMR0>,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::LPTMR0, InterruptHandler>,
        config: Config,
    ) -> Result<Self, Error> {
        let parts = configure(config)?;
        interrupt::typelevel::LPTMR0::disable();
        interrupt::typelevel::LPTMR0::unpend();
        unsafe {
            interrupt::typelevel::LPTMR0::enable();
        }
        Ok(Self {
            _peri: peri,
            _wake_guard: parts.wake_guard,
            freq: parts.freq,
            free_running: config.free_running,
        })
    }

    /// Counter frequency in Hz, after all dividers.
    pub fn frequency(&self) -> u32 {
        self.freq
    }

    /// Is the counter enabled?
    pub fn is_running(&self) -> bool {
        pac::LPTMR0.csr().read().ten() == Ten::Ten1
    }

    /// Latched compare status (not automatically cleared by this method).
    pub fn has_elapsed(&self) -> bool {
        pac::LPTMR0.csr().read().tcf()
    }

    /// Current counter snapshot.
    pub fn counter(&self) -> u32 {
        critical_section::with(|_| read_counter())
    }

    /// Acknowledge compare without altering the counter configuration.
    pub fn clear_compare_flag(&mut self) {
        pac::LPTMR0.csr().modify(|w| w.set_tcf(true));
    }

    /// Stop, reset the counter and clear/mask compare status and interrupt.
    pub fn stop(&mut self) {
        pac::LPTMR0.csr().write(|_| {});
    }

    /// Start a period of 1..=u32::MAX counter ticks, resetting any old period.
    /// Compare occurs after CMR+1 increments, so CMR is programmed to ticks-1.
    pub fn start(&mut self, ticks: u32) -> Result<(), Error> {
        if ticks == 0 {
            return Err(Error::InvalidPeriod);
        }
        self.stop();
        pac::LPTMR0.cmr().write(|w| w.set_compare(ticks - 1));
        pac::LPTMR0.csr().write(|w| {
            w.set_tfc(if self.free_running { Tfc::Tfc1 } else { Tfc::Tfc0 });
            w.set_ten(Ten::Ten1);
        });
        Ok(())
    }

    /// Wait for a started timer's compare. Completion and cancellation both
    /// stop/reset the timer, making the next wait independent of stale events.
    pub async fn wait_for_compare(&mut self) -> Result<(), Error> {
        if !self.is_running() {
            return Err(Error::NotRunning);
        }
        let _stop = OnDrop::new(|| pac::LPTMR0.csr().write(|_| {}));
        WAIT_CELL
            .wait_for(|| {
                critical_section::with(|_| {
                    if pac::LPTMR0.csr().read().tcf() {
                        return true;
                    }
                    pac::LPTMR0.csr().modify(|w| {
                        w.set_tcf(false);
                        w.set_tie(Tie::Tie1);
                    });
                    false
                })
            })
            .await
            .expect("LPTMR wait cell is never closed");
        Ok(())
    }

    /// Start and asynchronously wait for a period of counter ticks.
    pub async fn wait_ticks(&mut self, ticks: u32) -> Result<(), Error> {
        self.start(ticks)?;
        self.wait_for_compare().await
    }

    /// Start and busy-wait for a period of counter ticks.
    pub fn blocking_wait_ticks(&mut self, ticks: u32) -> Result<(), Error> {
        self.start(ticks)?;
        while !self.has_elapsed() {
            cortex_m::asm::nop();
        }
        self.stop();
        Ok(())
    }

    fn from_micros(&self, micros: u64, freq: u32) -> Option<u32> {
        let ticks = micros.checked_mul(freq as u64)?.div_ceil(1_000_000);
        ticks.try_into().ok()
    }

    /// Wait for a duration rounded up to counter ticks. Zero returns immediately.
    pub async fn delay(&mut self, duration: Duration) -> Result<(), Error> {
        let ticks = self
            .from_micros(duration.as_micros(), self.freq)
            .ok_or(Error::InvalidPeriod)?;
        if ticks == 0 {
            return Ok(());
        }
        self.wait_ticks(ticks).await
    }
}

impl Drop for Lptmr<'_> {
    fn drop(&mut self) {
        self.stop();
        interrupt::typelevel::LPTMR0::disable();
        interrupt::typelevel::LPTMR0::unpend();
    }
}
