//! Independent watchdog.
//!
//! Register programming follows the vendor `tremo_iwdg` driver.

use crate::rcc::{self, Peripheral};
use crate::{Peri, pac, peripherals};

const MAX_RELOAD: u32 = 0x0fff;

/// IWDG configuration error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ConfigError {
    /// Reload value exceeds the 12-bit `MAX` field (`0..=0x0FFF`).
    InvalidReload,
    /// Window value exceeds the 12-bit `WIN` field (`0..=0x0FFF`).
    InvalidWindow,
    /// Enabling the IWDG clock failed.
    Rcc(rcc::Error),
}

impl From<rcc::Error> for ConfigError {
    fn from(value: rcc::Error) -> Self {
        Self::Rcc(value)
    }
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidReload => f.write_str("IWDG reload out of range"),
            Self::InvalidWindow => f.write_str("IWDG window out of range"),
            Self::Rcc(err) => write!(f, "IWDG RCC error: {err}"),
        }
    }
}

impl core::error::Error for ConfigError {}

/// IWDG clock prescaler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
#[non_exhaustive]
pub enum Prescaler {
    /// Divide the watchdog clock by 4.
    Div4 = 0x00,
    /// Divide the watchdog clock by 8.
    Div8 = 0x02,
    /// Divide the watchdog clock by 16.
    Div16 = 0x04,
    /// Divide the watchdog clock by 32.
    Div32 = 0x06,
    /// Divide the watchdog clock by 64.
    Div64 = 0x08,
    /// Divide the watchdog clock by 128.
    Div128 = 0x0a,
    /// Divide the watchdog clock by 256.
    Div256 = 0x0c,
}

/// Independent-watchdog configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// Clock prescaler.
    pub prescaler: Prescaler,
    /// Reload value in the range `0..=0x0FFF`.
    pub reload: u32,
    /// Optional window value in the range `0..=0x0FFF`.
    pub window: Option<u32>,
    /// When true, timeout asserts a system reset request.
    ///
    /// The vendor notes that auto-reset does not work in STOP3.
    pub auto_reset: bool,
}

impl Config {
    /// Create a default configuration with prescaler 4 and full reload.
    pub const fn new() -> Self {
        Self {
            prescaler: Prescaler::Div4,
            reload: MAX_RELOAD,
            window: None,
            auto_reset: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Owned independent watchdog.
pub struct IndependentWatchdog<'d> {
    _peri: Peri<'d, peripherals::IWDG>,
}

impl<'d> IndependentWatchdog<'d> {
    /// Enable clocks, apply `config`, and start the watchdog.
    ///
    /// Like `embassy-stm32`'s independent watchdog, construction starts
    /// watching: keep the handle alive (and keep petting) to avoid a reset.
    /// [`stop`](Self::stop) halts it; dropping the handle stops it as well.
    pub fn new(peri: Peri<'d, peripherals::IWDG>, config: Config) -> Result<Self, ConfigError> {
        rcc::enable_peripheral(Peripheral::Iwdg)?;
        let mut this = Self { _peri: peri };
        this.configure(config)?;
        this.start();
        Ok(this)
    }

    /// Apply configuration while the watchdog is stopped.
    ///
    /// Rejects reload/window values outside the 12-bit fields instead of
    /// silently clamping: a clamped timeout would fire later than requested.
    pub fn configure(&mut self, config: Config) -> Result<(), ConfigError> {
        if config.reload > MAX_RELOAD {
            return Err(ConfigError::InvalidReload);
        }
        if let Some(window) = config.window
            && window > MAX_RELOAD
        {
            return Err(ConfigError::InvalidWindow);
        }
        let regs = Self::regs();
        wait_sr_done();

        regs.cr().modify(|_, w| w.start().clear_bit());
        wait_sr_done();

        unsafe {
            regs.sr2().write_with_zero(|w| w.reset_req_sr().set_bit());
        }
        wait_sr_done();

        critical_section::with(|_| {
            let rcc = unsafe { pac::Rcc::steal() };
            rcc.rst_cr().modify(|_, w| w.iwdg_reset_req_en().bit(config.auto_reset));
        });

        regs.cr().modify(|_, w| {
            w.rsten().bit(config.auto_reset);
            match config.prescaler {
                Prescaler::Div4 => w.prediv().value_4(),
                Prescaler::Div8 => w.prediv().value_8(),
                Prescaler::Div16 => w.prediv().value_16(),
                Prescaler::Div32 => w.prediv().value_32(),
                Prescaler::Div64 => w.prediv().value_64(),
                Prescaler::Div128 => w.prediv().value_128(),
                Prescaler::Div256 => w.prediv().value_256(),
            }
        });
        wait_sr_done();

        unsafe {
            regs.max().write_with_zero(|w| w.max().bits(config.reload as u16));
        }
        wait_sr_done();

        if let Some(window) = config.window {
            unsafe {
                regs.win().write_with_zero(|w| w.win().bits(window as u16));
            }
            wait_sr_done();
        }
        Ok(())
    }

    /// Start the watchdog.
    pub fn start(&mut self) {
        wait_sr_done();
        Self::regs().cr().modify(|_, w| {
            w.start().set_bit();
            w.wken().set_bit()
        });
        while !Self::regs().sr().read().write_cr_done().bit_is_set() {}
    }

    /// Stop the watchdog counter.
    pub fn stop(&mut self) {
        wait_sr_done();
        Self::regs().cr().modify(|_, w| w.start().clear_bit());
        while !Self::regs().sr().read().write_cr_done().bit_is_set() {}
    }

    /// Feed the watchdog.
    pub fn pet(&mut self) {
        let regs = Self::regs();
        if regs.sr2().read().reset_req_sr().bit_is_set() {
            wait_sr_done();
            unsafe {
                regs.sr2().write_with_zero(|w| w.reset_req_sr().set_bit());
            }
        }

        wait_sr_done();
        let reload = regs.max().read().max().bits();
        unsafe {
            regs.max().write_with_zero(|w| w.max().bits(reload));
        }
    }

    /// Enable or disable the reset-request interrupt.
    pub fn set_interrupt_enabled(&mut self, enabled: bool) {
        Self::regs().cr1().modify(|_, w| w.reset_req_int_en().bit(enabled));
    }

    /// Clear the reset-request interrupt status.
    pub fn clear_interrupt(&mut self) {
        unsafe {
            Self::regs().sr2().write_with_zero(|w| w.reset_req_sr().set_bit());
        }
        while !Self::regs().sr().read().write_sr2_done().bit_is_set() {}
    }

    #[inline]
    fn regs() -> pac::Iwdg {
        unsafe { pac::Iwdg::steal() }
    }
}

fn wait_sr_done() {
    // Fail-stop init handshake: the IWDG runs on its own clock domain and the
    // vendor driver polls these bits without timeout. Bounding this would
    // silently continue with a half-programmed watchdog, which is worse than
    // hanging a wedged chip here before it can run unsupervised.
    let ready = || {
        let sr = IndependentWatchdog::regs().sr().read();
        sr.write_cr_done().bit_is_set()
            && sr.max_set_done().bit_is_set()
            && sr.win_set_done().bit_is_set()
            && sr.write_sr2_done().bit_is_set()
    };
    while !ready() {}
}

impl Drop for IndependentWatchdog<'_> {
    fn drop(&mut self) {
        // Stop on drop so a discarded handle cannot reset the system out
        // from under the new owner; keep the handle alive to keep watching.
        self.stop();
    }
}
