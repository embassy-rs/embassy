//! Flash security / access-error status block.
//!
//! The vendor SDK does not provide a dedicated SEC driver. Flash programming
//! checks and clears `SEC.SR.FLASH_ACCESS_ERROR`; this module exposes that
//! documented status surface and leaves undocumented filter/reset fields alone.

use core::sync::atomic::{AtomicU8, Ordering};

use crate::rcc::{self, Peripheral};
use crate::{Peri, pac, peripherals};

static CLOCK_USERS: AtomicU8 = AtomicU8::new(0);

/// Acquire the shared SEC clock.
///
/// Both the security block driver and the flash driver need the SEC register
/// window alive. The clock is gated only when the last user releases it.
pub(crate) fn acquire_clock() {
    critical_section::with(|_| {
        let users = CLOCK_USERS.load(Ordering::Relaxed);
        if users == 0 {
            let _ = rcc::enable_peripheral(Peripheral::Sec);
        }
        CLOCK_USERS.store(users.saturating_add(1), Ordering::Relaxed);
    });
}

/// Release the shared SEC clock.
pub(crate) fn release_clock() {
    critical_section::with(|_| {
        let users = CLOCK_USERS.load(Ordering::Relaxed);
        if users <= 1 {
            CLOCK_USERS.store(0, Ordering::Relaxed);
            let _ = rcc::disable_peripheral(Peripheral::Sec);
        } else {
            CLOCK_USERS.store(users - 1, Ordering::Relaxed);
        }
    });
}

/// Owned SEC peripheral.
pub struct Sec<'d> {
    _peri: Peri<'d, peripherals::SEC>,
}

impl<'d> Sec<'d> {
    /// Enable the SEC clock.
    pub fn new(peri: Peri<'d, peripherals::SEC>) -> Self {
        acquire_clock();
        Self { _peri: peri }
    }

    /// Whether a flash access-policy error is latched.
    pub fn flash_access_error(&self) -> bool {
        Self::regs().sr().read().flash_access_error().bit_is_set()
    }

    /// Clear the flash access-policy error flag.
    pub fn clear_flash_access_error(&mut self) {
        clear_error();
    }

    /// Raw status register value.
    pub fn status_bits(&self) -> u32 {
        Self::regs().sr().read().bits()
    }

    #[inline]
    fn regs() -> pac::Sec {
        unsafe { pac::Sec::steal() }
    }
}

impl Drop for Sec<'_> {
    fn drop(&mut self) {
        release_clock();
    }
}

/// Clear the latched flash access error through the W1C status register.
pub(crate) fn clear_error() {
    let sec = unsafe { pac::Sec::steal() };
    if sec.sr().read().flash_access_error().bit_is_set() {
        // Writing only the W1C bit avoids clearing unrelated SEC flags.
        unsafe {
            sec.sr().write_with_zero(|w| w.flash_access_error().set_bit());
        }
    }
}

/// Return whether a flash access error is latched, clearing it if so.
pub(crate) fn take_error() -> bool {
    let sec = unsafe { pac::Sec::steal() };
    let error = sec.sr().read().flash_access_error().bit_is_set();
    if error {
        unsafe {
            sec.sr().write_with_zero(|w| w.flash_access_error().set_bit());
        }
    }
    error
}
