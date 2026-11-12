//! Window / system watchdog (ARM SP805-style WDG block).
//!
//! Register programming follows the vendor `tremo_wdg` driver.
//!
//! PAC field accessors are used for LOAD, VALUE and CONTROL below.
//! LOCK (magic-word write), INTCLR (any-write-clears) and RIS/MIS have no
//! PAC fields and stay raw, matching the vendor header and SP805 behavior.

use crate::rcc::{self, Peripheral};
use crate::{Peri, pac, peripherals};

const LOCK_TOKEN: u32 = 0x1acc_e551;

/// Owned system watchdog.
pub struct Watchdog<'d> {
    _peri: Peri<'d, peripherals::WDG>,
}

impl<'d> Watchdog<'d> {
    /// Enable the WDG clocks and leave the watchdog stopped.
    pub fn new(peri: Peri<'d, peripherals::WDG>) -> Self {
        let _ = rcc::enable_peripheral(Peripheral::Wdg);
        // Reset so a second construction never inherits a running counter
        // or armed reset request from a previous owner.
        let _ = rcc::reset_peripheral(Peripheral::Wdg);
        Self { _peri: peri }
    }

    /// Start the watchdog with the given reload value (raw counter ticks).
    ///
    /// The tick rate is the WDG block clock; there is no published divider to
    /// convert wall time, so callers tune this against the vendor example.
    /// Per the vendor driver, the first timeout asserts the WDG interrupt and
    /// reloads the counter; a second timeout without a feed resets the system.
    pub fn start(&mut self, reload_value: u32) {
        unlock();
        let regs = Self::regs();
        unsafe { regs.load().write_with_zero(|w| w.load().bits(reload_value)) };
        regs.control().modify(|_, w| {
            w.resen().set_bit();
            w.inten().set_bit();
            w
        });
        lock();

        critical_section::with(|_| {
            let rcc = unsafe { pac::Rcc::steal() };
            rcc.rst_cr().modify(|_, w| w.wwdg_reset_req_en().set_bit());
        });
    }

    /// Feed the watchdog and clear its interrupt.
    pub fn pet(&mut self) {
        unlock();
        unsafe { Self::regs().intclr().write_with_zero(|w| w.bits(1)) };
        lock();
    }

    /// Stop the watchdog and disable its reset request.
    pub fn stop(&mut self) {
        critical_section::with(|_| {
            let rcc = unsafe { pac::Rcc::steal() };
            rcc.rst_cr().modify(|_, w| w.wwdg_reset_req_en().clear_bit());
        });

        unlock();
        let regs = Self::regs();
        regs.control().modify(|_, w| {
            w.resen().clear_bit();
            w.inten().clear_bit();
            w
        });
        unsafe { regs.load().write_with_zero(|w| w.load().bits(0xffff_ffff)) };
        lock();
    }

    /// Current down-counter value.
    pub fn value(&self) -> u32 {
        Self::regs().value().read().value().bits()
    }

    #[inline]
    fn regs() -> pac::Wwdg {
        unsafe { pac::Wwdg::steal() }
    }
}

fn unlock() {
    unsafe {
        Watchdog::regs().lock().write_with_zero(|w| w.bits(LOCK_TOKEN));
    }
}

fn lock() {
    unsafe {
        Watchdog::regs().lock().write_with_zero(|w| w.bits(!LOCK_TOKEN));
    }
}

impl Drop for Watchdog<'_> {
    fn drop(&mut self) {
        // Stop on drop so a discarded handle cannot reset the system out
        // from under the new owner; keep the handle alive to keep watching.
        self.stop();
        let _ = rcc::disable_peripheral(Peripheral::Wdg);
    }
}
