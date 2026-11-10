//! Independent watchdog.
//!
//! Register programming follows the vendor `tremo_iwdg` driver.

use crate::rcc::{self, Peripheral};
use crate::{Peri, pac, peripherals};

const MAX_RELOAD: u32 = 0x0fff;

const CR_RSTEN: u32 = 1 << 5;
const CR_WKEN: u32 = 1 << 4;
const CR_PREDIV_MASK: u32 = 0x0e;
const CR_START: u32 = 1 << 0;

const SR_WRITE_SR2_DONE: u32 = 1 << 3;
const SR_WRITE_CR_DONE: u32 = 1 << 0;
const SR_ALL_DONE: u32 = 0x0f;

const CR1_RESET_REQ_INT_EN: u32 = 1 << 0;
const SR2_RESET_REQ: u32 = 1 << 0;
const RCC_RST_CR_IWDG_RESET_REQ_EN: u32 = 1 << 5;

/// IWDG clock prescaler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum Prescaler {
    Div4 = 0x00,
    Div8 = 0x02,
    Div16 = 0x04,
    Div32 = 0x06,
    Div64 = 0x08,
    Div128 = 0x0a,
    Div256 = 0x0c,
}

/// Independent-watchdog configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    /// Enable clocks, apply `config`, and leave the watchdog stopped.
    pub fn new(peri: Peri<'d, peripherals::IWDG>, config: Config) -> Self {
        let _ = rcc::enable_peripheral(Peripheral::Iwdg);
        let this = Self { _peri: peri };
        this.configure(config);
        this
    }

    /// Apply configuration while the watchdog is stopped.
    pub fn configure(&self, config: Config) {
        let regs = Self::regs();
        wait_sr_done();

        regs.cr().modify(|r, w| unsafe { w.bits(r.bits() & !CR_START) });
        wait_sr_done();

        unsafe {
            regs.sr2().write_with_zero(|w| w.bits(SR2_RESET_REQ));
        }
        wait_sr_done();

        critical_section::with(|_| {
            let rcc = unsafe { pac::Rcc::steal() };
            rcc.rst_cr().modify(|r, w| unsafe {
                let bits = if config.auto_reset {
                    r.bits() | RCC_RST_CR_IWDG_RESET_REQ_EN
                } else {
                    r.bits() & !RCC_RST_CR_IWDG_RESET_REQ_EN
                };
                w.bits(bits)
            });
        });

        regs.cr().modify(|r, w| unsafe {
            let mut bits = (r.bits() & !CR_RSTEN) | (config.prescaler as u32 & CR_PREDIV_MASK);
            if config.auto_reset {
                bits |= CR_RSTEN;
            }
            w.bits(bits)
        });
        wait_sr_done();

        let reload = config.reload.min(MAX_RELOAD);
        unsafe {
            regs.max().write_with_zero(|w| w.bits(reload));
        }
        wait_sr_done();

        if let Some(window) = config.window {
            unsafe {
                regs.win().write_with_zero(|w| w.bits(window.min(MAX_RELOAD)));
            }
            wait_sr_done();
        }
    }

    /// Start the watchdog.
    pub fn start(&self) {
        wait_sr_done();
        Self::regs()
            .cr()
            .modify(|r, w| unsafe { w.bits(r.bits() | CR_START | CR_WKEN) });
        while Self::regs().sr().read().bits() & SR_WRITE_CR_DONE == 0 {}
    }

    /// Stop the watchdog counter.
    pub fn stop(&self) {
        wait_sr_done();
        Self::regs().cr().modify(|r, w| unsafe { w.bits(r.bits() & !CR_START) });
        while Self::regs().sr().read().bits() & SR_WRITE_CR_DONE == 0 {}
    }

    /// Feed the watchdog.
    pub fn pet(&self) {
        let regs = Self::regs();
        if regs.sr2().read().bits() & SR2_RESET_REQ != 0 {
            wait_sr_done();
            unsafe {
                regs.sr2().write_with_zero(|w| w.bits(SR2_RESET_REQ));
            }
        }

        wait_sr_done();
        let reload = regs.max().read().bits();
        unsafe {
            regs.max().write_with_zero(|w| w.bits(reload));
        }
    }

    /// Enable or disable the reset-request interrupt.
    pub fn set_interrupt_enabled(&self, enabled: bool) {
        Self::regs().cr1().modify(|r, w| unsafe {
            let bits = if enabled {
                r.bits() | CR1_RESET_REQ_INT_EN
            } else {
                r.bits() & !CR1_RESET_REQ_INT_EN
            };
            w.bits(bits)
        });
    }

    /// Clear the reset-request interrupt status.
    pub fn clear_interrupt(&self) {
        unsafe {
            Self::regs().sr2().write_with_zero(|w| w.bits(SR2_RESET_REQ));
        }
        while Self::regs().sr().read().bits() & SR_WRITE_SR2_DONE == 0 {}
    }

    #[inline]
    fn regs() -> pac::Iwdg {
        unsafe { pac::Iwdg::steal() }
    }
}

fn wait_sr_done() {
    while IndependentWatchdog::regs().sr().read().bits() & SR_ALL_DONE != SR_ALL_DONE {}
}
