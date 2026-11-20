//! Real Time Clock (RTC)
//!
//! The STM32F1 RTC is a 32-bit counter that increments once per second. It has no calendar: what the
//! count means (for example, seconds since the Unix epoch) is up to the application.
//!
//! The counter is in the backup domain, so it keeps counting across resets, and while VDD is off if
//! VBAT is powered and the RTC runs from the LSE. [`init`](crate::init) sets the prescaler to 1 Hz
//! when it configures the RTC clock (`config.rcc.ls`), which only happens after a backup domain reset
//! or when the RTC clock source changes.

use crate::pac::RTC;
use crate::pac::rtc::vals::Rtoff;
use crate::time::Hertz;
use crate::{Peri, peripherals};

/// RTC driver.
pub struct Rtc<'d> {
    _peri: Peri<'d, peripherals::RTC>,
}

impl<'d> Rtc<'d> {
    /// Create a new RTC driver.
    ///
    /// The counter keeps running and keeps its value.
    ///
    /// # Panics
    ///
    /// Panics if the RTC clock is disabled in `config.rcc.ls`.
    pub fn new(peri: Peri<'d, peripherals::RTC>) -> Self {
        assert!(
            unsafe { crate::rcc::get_freqs() }.rtc.to_hertz().is_some(),
            "RTC clock is not enabled"
        );

        // The counter registers read as garbage until they have been resynchronized with the RTC core
        // after a reset (RM0008 18.3.3).
        wait_write_done();
        RTC.crl().modify(|w| w.set_rsf(false));
        while !RTC.crl().read().rsf() {}

        Self { _peri: peri }
    }

    /// Read the counter.
    pub fn counter(&self) -> u32 {
        loop {
            let high = RTC.cnth().read().cnth();
            let low = RTC.cntl().read().cntl();
            // Retry if the low half wrapped during the read.
            if RTC.cnth().read().cnth() == high {
                return (high as u32) << 16 | low as u32;
            }
        }
    }

    /// Set the counter.
    ///
    /// This restarts the current second: the counter next increments one second after this call.
    pub fn set_counter(&mut self, value: u32) {
        configure(|| {
            RTC.cnth().write(|w| w.set_cnth((value >> 16) as u16));
            RTC.cntl().write(|w| w.set_cntl(value as u16));
        });
    }
}

/// Set the prescaler so the counter increments once per second with an RTC clock of `clk`.
///
/// Writing the prescaler restarts the current second even if the value doesn't change, so this is only
/// called when the backup domain is (re)configured, not on every boot.
pub(crate) fn set_prescaler(clk: Hertz) {
    let prl = clk.0 - 1;
    assert!(prl < 1 << 20, "RTC clock too fast for the RTC prescaler");
    configure(|| {
        RTC.prlh().write(|w| w.set_prlh((prl >> 16) as u8));
        RTC.prll().write(|w| w.set_prll(prl as u16));
    });
}

fn wait_write_done() {
    while RTC.crl().read().rtoff() != Rtoff::Terminated {}
}

/// Write the counter, prescaler or alarm registers in `f` (RM0008 18.3.4).
fn configure(f: impl FnOnce()) {
    wait_write_done();
    RTC.crl().modify(|w| w.set_cnf(true));
    f();
    // The writes take effect when leaving configuration mode.
    RTC.crl().modify(|w| w.set_cnf(false));
    wait_write_done();
}
