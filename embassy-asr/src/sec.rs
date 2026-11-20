//! Flash security / access-error status block.
//!
//! The vendor SDK does not provide a dedicated SEC driver. Flash programming
//! checks and clears `SEC.SR.FLASH_ACCESS_ERROR`; this module exposes that
//! documented status surface and leaves undocumented filter/reset fields alone.

use crate::rcc::{self, Peripheral};
use crate::{Peri, pac, peripherals};

/// Owned SEC peripheral.
pub struct Sec<'d> {
    _peri: Peri<'d, peripherals::SEC>,
}

impl<'d> Sec<'d> {
    /// Enable the SEC clock.
    pub fn new(peri: Peri<'d, peripherals::SEC>) -> Self {
        let _ = rcc::enable_peripheral(Peripheral::Sec);
        Self { _peri: peri }
    }

    /// Whether a flash access-policy error is latched.
    pub fn flash_access_error(&self) -> bool {
        Self::regs().sr().read().flash_access_error().bit_is_set()
    }

    /// Clear the flash access-policy error flag.
    pub fn clear_flash_access_error(&self) {
        Self::regs().sr().modify(|_, w| w.flash_access_error().set_bit());
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

/// Crate-private helpers used by the flash driver.
pub(crate) mod flash_access {
    use crate::pac;

    pub(crate) fn clear_error() {
        let sec = unsafe { pac::Sec::steal() };
        if sec.sr().read().flash_access_error().bit_is_set() {
            sec.sr().modify(|_, w| w.flash_access_error().set_bit());
        }
    }

    pub(crate) fn take_error() -> bool {
        let sec = unsafe { pac::Sec::steal() };
        let error = sec.sr().read().flash_access_error().bit_is_set();
        if error {
            sec.sr().modify(|_, w| w.flash_access_error().set_bit());
        }
        error
    }
}
