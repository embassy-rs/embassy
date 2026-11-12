//! Reset cause flags.
//!
//! RCC records what caused the most recent reset in a set of sticky flags
//! (`RCC_CSR`, `RCC_CSR2` or `RCC_RSR` depending on the family). The flags are
//! only cleared by software (`RMVF`) or by a power-on reset, so without clearing
//! them they accumulate across resets.

use core::sync::atomic::{AtomicU32, Ordering};

use crate::pac::RCC;
use crate::pac::common::{RW, Reg};
#[cfg(not(any(stm32c0, stm32c5, stm32g5, stm32h5, stm32h7, stm32h7rs, stm32n6)))]
use crate::pac::rcc::regs::Csr as Raw;
#[cfg(stm32c0)]
use crate::pac::rcc::regs::Csr2 as Raw;
#[cfg(any(stm32c5, stm32g5, stm32h5, stm32h7, stm32h7rs, stm32n6))]
use crate::pac::rcc::regs::Rsr as Raw;

#[cfg(stm32c0)]
fn reg() -> Reg<Raw, RW> {
    RCC.csr2()
}

#[cfg(any(stm32c5, stm32g5, stm32h5, stm32h7, stm32h7rs, stm32n6))]
fn reg() -> Reg<Raw, RW> {
    RCC.rsr()
}

#[cfg(not(any(stm32c0, stm32c5, stm32g5, stm32h5, stm32h7, stm32h7rs, stm32n6)))]
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
    pub fn nrst_pin(&self) -> bool {
        Raw(self.0).pinrstf()
    }

    /// A software reset was requested, for example with `SCB::sys_reset()`.
    pub fn software(&self) -> bool {
        Raw(self.0).sftrstf()
    }

    /// The independent watchdog expired.
    pub fn independent_watchdog(&self) -> bool {
        #[cfg(any(rcc_h7, rcc_h7rm0433, rcc_h7rm0468))]
        return Raw(self.0).iwdg1rstf();
        #[cfg(not(any(rcc_h7, rcc_h7rm0433, rcc_h7rm0468)))]
        return Raw(self.0).iwdgrstf();
    }

    /// The window watchdog expired.
    pub fn window_watchdog(&self) -> bool {
        #[cfg(any(rcc_h7, rcc_h7rm0433, rcc_h7rm0468))]
        return Raw(self.0).wwdg1rstf();
        #[cfg(not(any(rcc_h7, rcc_h7rm0433, rcc_h7rm0468)))]
        return Raw(self.0).wwdgrstf();
    }

    /// Illegal entry into a low-power mode.
    ///
    /// This happens when Stop or Standby mode is entered while the corresponding
    /// nRST option bits are configured to generate a reset instead.
    pub fn low_power(&self) -> bool {
        Raw(self.0).lpwrrstf()
    }

    /// A power-on or power-down reset (POR/PDR).
    ///
    /// Other flags are usually set along with it, so check this first.
    #[cfg(any(
        stm32f0, stm32f1, stm32f2, stm32f3, stm32f4, stm32f7, stm32h7, stm32h7rs, stm32l0, stm32l1, stm32n6
    ))]
    pub fn power_on(&self) -> bool {
        Raw(self.0).porrstf()
    }

    /// A brownout reset (BOR).
    ///
    /// On many families a power-on reset also sets this flag. Where `power_on()` is
    /// available, check it first.
    #[cfg(any(
        stm32c5, stm32f2, stm32f4, stm32f7, stm32g4, stm32h5, stm32h7, stm32h7rs, stm32l4, stm32l5, stm32n6, stm32u3,
        stm32u5, stm32wb, stm32wba, stm32wl
    ))]
    pub fn brownout(&self) -> bool {
        Raw(self.0).borrstf()
    }

    /// A power reset: power-on, power-down or brownout (POR/PDR or BOR).
    #[cfg(any(stm32c0, stm32g0, stm32u0))]
    pub fn power_reset(&self) -> bool {
        Raw(self.0).pwrrstf()
    }

    /// The option bytes were reloaded, for example by setting `OBL_LAUNCH` in the
    /// flash interface.
    #[cfg(any(
        stm32c0, stm32f0, stm32f3, stm32g0, stm32g4, stm32h7rs, stm32l0, stm32l4, stm32l5, stm32u0, stm32u3, stm32u5,
        stm32wb, stm32wba, stm32wl
    ))]
    pub fn option_load(&self) -> bool {
        Raw(self.0).oblrstf()
    }

    /// The firewall detected an illegal access and reset the chip.
    #[cfg(any(stm32l0, stm32l4))]
    pub fn firewall(&self) -> bool {
        Raw(self.0).fwrstf()
    }

    // The flag combinations described below follow RM0433, Table 56.

    /// The CPU was reset. On dual-core chips, this is the Cortex-M7.
    ///
    /// Most resets set this along with other flags. It is set on its own when only
    /// the CPU was reset.
    #[cfg(any(rcc_h7, rcc_h7rm0433, rcc_h7rm0468))]
    pub fn cpu(&self) -> bool {
        Raw(self.0).cpurstf()
    }

    /// The D1 domain was reset.
    ///
    /// A power-on reset sets this along with other flags. It is set on its own when
    /// the D1 domain exits DStandby mode.
    #[cfg(any(rcc_h7, rcc_h7rm0433, rcc_h7rm0468))]
    pub fn d1_domain(&self) -> bool {
        Raw(self.0).d1rstf()
    }

    /// The D2 domain was reset.
    ///
    /// A power-on reset sets this along with other flags. It is set on its own when
    /// the D2 domain exits DStandby mode.
    #[cfg(any(rcc_h7, rcc_h7rm0433, rcc_h7rm0468))]
    pub fn d2_domain(&self) -> bool {
        Raw(self.0).d2rstf()
    }
}
