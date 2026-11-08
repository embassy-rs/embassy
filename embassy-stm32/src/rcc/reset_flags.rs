//! Reset cause flags.
//!
//! RCC records what caused the most recent reset in a set of sticky flags
//! (`RCC_CSR`, `RCC_CSR2` or `RCC_RSR` depending on the family). The flags are
//! only cleared by software (`RMVF`) or by a power-on reset, so without clearing
//! them they accumulate across resets.

use core::sync::atomic::{AtomicU32, Ordering};

use crate::pac::RCC;
use crate::pac::common::{RW, Reg};
#[cfg(not(any(stm32c0, stm32c5, stm32h5, stm32h7, stm32h7rs)))]
use crate::pac::rcc::regs::Csr as Raw;
#[cfg(stm32c0)]
use crate::pac::rcc::regs::Csr2 as Raw;
#[cfg(any(stm32c5, stm32h5, stm32h7, stm32h7rs))]
use crate::pac::rcc::regs::Rsr as Raw;

#[cfg(stm32c0)]
fn reg() -> Reg<Raw, RW> {
    RCC.csr2()
}

#[cfg(any(stm32c5, stm32h5, stm32h7, stm32h7rs))]
fn reg() -> Reg<Raw, RW> {
    RCC.rsr()
}

#[cfg(not(any(stm32c0, stm32c5, stm32h5, stm32h7, stm32h7rs)))]
fn reg() -> Reg<Raw, RW> {
    RCC.csr()
}

/// Register value read by the first call to [`reset_flags`].
///
/// `u32::MAX` instead of `u32::MIN` because the registers reserved bits read as zero.
/// It is a valid reading when no flag is set.
static FLAGS: AtomicU32 = AtomicU32::new(u32::MAX);

/// Returns the flags describing what caused the most recent reset.
///
/// The first call reads the flags from RCC and clears them in hardware, so the
/// next reset starts from a clean state. The value is cached, and later calls
/// return the same flags for the rest of this boot.
///
/// Until this is called, the flags are left untouched in hardware. If something
/// that ran before the application (for example a bootloader) calls this function
/// or otherwise clears the flags, the application will see them empty.
///
/// More than one flag is often set for a single reset. See the individual methods
/// on [`ResetFlags`] for details.
///
/// On dual-core chips, call this from one core only.
pub fn reset_flags() -> ResetFlags {
    critical_section::with(|_| {
        let mut bits = FLAGS.load(Ordering::Relaxed);
        if bits == u32::MAX {
            bits = reg().read().0;
            reg().modify(|w| w.set_rmvf(true));
            FLAGS.store(bits, Ordering::Relaxed);
        }
        ResetFlags(bits)
    })
}

/// The flags describing what caused the most recent reset, returned by [`reset_flags`].
///
/// Which flags are available depends on the chip family.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ResetFlags(pub u32);

impl ResetFlags {
    /// The NRST pin was pulled low.
    ///
    /// On many STM32 chips, internal reset sources can also drive NRST low,
    /// so this flag is commonly set together with other flags.
    /// Check the more specific flags first.
    pub fn nrst(&self) -> bool {
        #[cfg(any(stm32f2, stm32f4, stm32f7))]
        return Raw(self.0).padrstf();
        #[cfg(not(any(stm32f2, stm32f4, stm32f7)))]
        return Raw(self.0).pinrstf();
    }

    /// A software reset was requested, for example with `SCB::sys_reset()`.
    pub fn sw_reset(&self) -> bool {
        Raw(self.0).sftrstf()
    }

    /// The independent watchdog expired.
    pub fn iwdg(&self) -> bool {
        #[cfg(any(stm32f2, stm32f4, stm32f7))]
        return Raw(self.0).wdgrstf();
        #[cfg(stm32h7)]
        return Raw(self.0).iwdg1rstf();
        #[cfg(not(any(stm32f2, stm32f4, stm32f7, stm32h7)))]
        return Raw(self.0).iwdgrstf();
    }

    /// The window watchdog expired.
    pub fn wwdg(&self) -> bool {
        #[cfg(stm32h7)]
        return Raw(self.0).wwdg1rstf();
        #[cfg(not(stm32h7))]
        return Raw(self.0).wwdgrstf();
    }

    /// Illegal entry into a low-power mode.
    ///
    /// This happens when Stop or Standby mode is entered while the corresponding
    /// nRST option bits are configured to generate a reset instead.
    pub fn low_power(&self) -> bool {
        Raw(self.0).lpwrrstf()
    }
}
