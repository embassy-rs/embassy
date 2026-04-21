//! Global TrustZone Controller (GTZC) driver.
//!
//! The GTZC provides hardware-enforced TrustZone security for STM32 microcontrollers.
//! It consists of three sub-units:
//!
//! - **TZSC** (TrustZone Security Controller): configures which peripherals are accessible
//!   from the Non-Secure world and which require privileged access.
//! - **MPCBB** (Memory Protection Controller Block-Based): configures security and privilege
//!   attributes for individual 512-byte SRAM blocks.
//! - **TZIC** (TrustZone Illegal access Controller): generates interrupts when a Non-Secure
//!   or unprivileged access is made to a Secure or privileged-only resource.
//!
//! # Supported families
//!
//! | Variant     | Family        | TZSC   | MPCBB | TZIC | MPCWM |
//! |-------------|---------------|--------|-------|------|-------|
//! | `gtzc_wba`  | WBA           | Full   | Yes   | Yes  | No    |
//! | `gtzc_v1`   | U5 / H5       | Full   | Yes   | Yes  | Yes   |
//! | `gtzc_h503` | H503          | PrivOnly| Yes  | No   | BKPSRAM|
//!
//! # Usage
//!
//! All GTZC configuration must be performed from the Secure world. **Enable the GTZC
//! clock first** via [`enable_clock()`], then configure MPCBB/TZIC, and finally call
//! [`lock()`] to prevent Non-Secure code from modifying the configuration.
//!
//! MPCBB and TZIC instances are accessed via chip-specific PAC constants
//! (e.g., `pac::GTZC_MPCBB1`, `pac::GTZC_TZIC`). Consult your device reference
//! manual for which instances are present.
//!
//! ```no_run
//! use embassy_stm32::{gtzc, pac};
//!
//! // Enable GTZC clock before any register access.
//! unsafe { gtzc::enable_clock(); }
//!
//! // Configure SRAM1 blocks as Non-Secure (wba or v1)
//! let mpcbb1 = unsafe { gtzc::Mpcbb::new(pac::GTZC_MPCBB1) };
//! mpcbb1.set_all_secure(/*n_regs=*/14, false);  // SRAM1 = 28 blocks × 512 B on WBA65
//!
//! // Enable TZIC to generate IRQs on illegal accesses (wba or v1)
//! let tzic = unsafe { gtzc::Tzic::new(pac::GTZC_TZIC) };
//! tzic.enable_all(/*n_regs=*/4);
//!
//! // Lock TZSC so the Non-Secure world cannot reconfigure it (wba or v1)
//! unsafe { gtzc::lock(); }
//! ```

#[cfg(any(gtzc_wba, gtzc_v1))]
use crate::pac;

// ────────────────────────────────────────────────────────────────────────────
// Clock enable
// ────────────────────────────────────────────────────────────────────────────

/// Enable the GTZC peripheral clock.
///
/// Must be called before accessing any GTZC register (TZSC, MPCBB, TZIC).
/// Failing to enable the clock causes hard faults on GTZC register reads/writes.
///
/// # Safety
/// Must be called from the Secure world.
// WBA and U5 use `gtzc1en`; H5 (rcc_h5) names the same bit `tzsc1en`.
// H503 (rcc_h50) has no software clock gate for GTZC — always-on.
#[cfg(any(gtzc_wba, all(gtzc_v1, not(stm32h5))))]
pub unsafe fn enable_clock() {
    pac::RCC.ahb1enr().modify(|r| r.set_gtzc1en(true));
}

/// Enable the GTZC peripheral clock.
///
/// Must be called before accessing any GTZC register (TZSC, MPCBB, TZIC).
/// Failing to enable the clock causes hard faults on GTZC register reads/writes.
///
/// # Safety
/// Must be called from the Secure world.
#[cfg(all(gtzc_v1, stm32h5))]
pub unsafe fn enable_clock() {
    pac::RCC.ahb1enr().modify(|r| r.set_tzsc1en(true));
}

// ────────────────────────────────────────────────────────────────────────────
// MPCBB — Memory Protection Controller Block-Based
// ────────────────────────────────────────────────────────────────────────────

/// Memory Protection Controller Block-Based (MPCBB) wrapper.
///
/// Controls security and privilege attributes of individual 512-byte SRAM blocks.
/// Each `seccfgr` / `privcfgr` register covers 32 contiguous blocks (= 16 KiB).
/// Every 32 blocks form one **superblock** that can be independently locked via
/// [`lock_superblock`].
///
/// Obtain a wrapper via [`Mpcbb::new`] with the chip-specific PAC constant for
/// the desired SRAM, e.g., `pac::GTZC_MPCBB1`, `pac::GTZC_MPCBB2`, etc.
#[cfg(any(gtzc_wba, gtzc_v1))]
pub struct Mpcbb {
    inner: pac::gtzc::Mpcbb,
}

#[cfg(any(gtzc_wba, gtzc_v1))]
impl Mpcbb {
    /// Wrap a PAC MPCBB peripheral instance.
    ///
    /// # Safety
    /// The caller must ensure no concurrent access to the same MPCBB instance and
    /// that this code runs in the Secure world.
    #[inline]
    pub unsafe fn new(inner: pac::gtzc::Mpcbb) -> Self {
        Self { inner }
    }

    /// Globally lock this MPCBB's configuration.
    ///
    /// Once locked the security and privilege configuration cannot be changed until
    /// the next hardware reset.
    #[inline]
    pub fn lock(&self) {
        self.inner.cr().modify(|r| r.set_glock(true));
    }

    /// Returns `true` if the global lock has been set.
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.inner.cr().read().glock()
    }

    /// Lock a single superblock (32 blocks = 16 KiB).
    ///
    /// `superblock` is a 0-based superblock index within this MPCBB. Each call locks
    /// one bit in the CFGLOCK register; the lock is cleared only by hardware reset.
    ///
    /// Note: each bit in CFGLOCK corresponds to one 16 KiB superblock. On WBA, the
    /// CFGLOCK register holds all superblock lock bits as a packed bitmask.
    #[inline]
    pub fn lock_superblock(&self, superblock: usize) {
        self.inner
            .cfglock()
            .modify(|r| r.set_splck(r.splck() | (1u32 << superblock)));
    }

    /// Lock the superblocks indicated by `mask`.
    ///
    /// Each bit `n` in `mask` corresponds to superblock `n`. Bits already set are
    /// unaffected (lock is one-way).
    #[inline]
    pub fn lock_superblocks(&self, mask: u32) {
        self.inner.cfglock().modify(|r| r.set_splck(r.splck() | mask));
    }

    /// Returns the current superblock lock bitmask from CFGLOCK.
    #[inline]
    pub fn superblock_lock_mask(&self) -> u32 {
        self.inner.cfglock().read().splck()
    }

    /// Configure the Secure Read/Write Illegal Access Disable (SRWILADIS) bit.
    ///
    /// When `true`, illegal accesses **from the Secure world** to Secure MPCBB blocks
    /// do **not** generate a TZIC interrupt or flag. This is useful to suppress
    /// spurious Secure-world self-access violations during initialization.
    ///
    /// Default after reset: `false` (all illegal accesses generate TZIC events).
    #[inline]
    pub fn set_srwiladis(&self, disabled: bool) {
        self.inner.cr().modify(|r| r.set_srwiladis(disabled));
    }

    /// Returns the current SRWILADIS setting.
    #[inline]
    pub fn srwiladis(&self) -> bool {
        self.inner.cr().read().srwiladis()
    }

    /// Configure the Inverted Security State (INVSECSTATE) bit.
    ///
    /// When `true`, the security polarity of all blocks in this MPCBB is inverted:
    /// a `seccfgr` bit of `0` means Secure, and `1` means Non-Secure. This is
    /// an advanced option that is rarely needed; leave `false` for normal operation.
    #[inline]
    pub fn set_invsecstate(&self, inverted: bool) {
        self.inner.cr().modify(|r| r.set_invsecstate(inverted));
    }

    /// Returns the current INVSECSTATE setting.
    #[inline]
    pub fn invsecstate(&self) -> bool {
        self.inner.cr().read().invsecstate()
    }

    /// Set the security attribute of a single 512-byte block.
    ///
    /// `block_idx` is a 0-based index into this SRAM region.
    /// `secure = true` marks the block as Secure (Non-Secure accesses generate an IRQ);
    /// `secure = false` makes it accessible from both worlds.
    #[inline]
    pub fn set_block_secure(&self, block_idx: usize, secure: bool) {
        let reg = block_idx / 32;
        let bit = block_idx % 32;
        self.inner.seccfgr(reg).modify(|r| {
            if secure {
                r.set_sec(r.sec() | (1u32 << bit));
            } else {
                r.set_sec(r.sec() & !(1u32 << bit));
            }
        });
    }

    /// Set the privilege attribute of a single 512-byte block.
    ///
    /// `privileged = true` restricts access to privileged mode only.
    #[inline]
    pub fn set_block_privileged(&self, block_idx: usize, privileged: bool) {
        let reg = block_idx / 32;
        let bit = block_idx % 32;
        self.inner.privcfgr(reg).modify(|r| {
            if privileged {
                r.set_priv_(r.priv_() | (1u32 << bit));
            } else {
                r.set_priv_(r.priv_() & !(1u32 << bit));
            }
        });
    }

    /// Write a raw security bitmap covering 32 contiguous blocks.
    ///
    /// `reg` selects which group of 32 blocks to configure (0-based).
    /// Each bit in `bits` corresponds to one block: `1` = Secure, `0` = Non-Secure.
    #[inline]
    pub fn set_seccfgr(&self, reg: usize, bits: u32) {
        self.inner.seccfgr(reg).write(|r| r.set_sec(bits));
    }

    /// Write a raw privilege bitmap covering 32 contiguous blocks.
    ///
    /// Each bit: `1` = Privileged-only, `0` = Unprivileged-accessible.
    #[inline]
    pub fn set_privcfgr(&self, reg: usize, bits: u32) {
        self.inner.privcfgr(reg).write(|r| r.set_priv_(bits));
    }

    /// Read the raw security bitmap for a group of 32 blocks.
    #[inline]
    pub fn seccfgr(&self, reg: usize) -> u32 {
        self.inner.seccfgr(reg).read().sec()
    }

    /// Read the raw privilege bitmap for a group of 32 blocks.
    #[inline]
    pub fn privcfgr(&self, reg: usize) -> u32 {
        self.inner.privcfgr(reg).read().priv_()
    }

    /// Set all blocks as Secure or Non-Secure in one call.
    ///
    /// `n_regs` is the number of 32-block registers that cover this SRAM.
    /// Calculate as `sram_size_bytes.div_ceil(16 * 1024)`. For example:
    /// - 448 KiB SRAM (WBA65 SRAM1) → `n_regs = 28`
    /// - 64 KiB SRAM (WBA65 SRAM2)  → `n_regs = 4`
    /// - 16 KiB SRAM (WBA65 SRAM6)  → `n_regs = 1`
    pub fn set_all_secure(&self, n_regs: usize, secure: bool) {
        let bits = if secure { 0xFFFF_FFFF } else { 0 };
        for n in 0..n_regs {
            self.inner.seccfgr(n).write(|r| r.set_sec(bits));
        }
    }

    /// Set all blocks as Privileged-only or Unprivileged-accessible in one call.
    ///
    /// See [`set_all_secure`] for the `n_regs` calculation.
    pub fn set_all_privileged(&self, n_regs: usize, privileged: bool) {
        let bits = if privileged { 0xFFFF_FFFF } else { 0 };
        for n in 0..n_regs {
            self.inner.privcfgr(n).write(|r| r.set_priv_(bits));
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// TZIC — TrustZone Illegal access Controller
// ────────────────────────────────────────────────────────────────────────────

/// TrustZone Illegal access Controller (TZIC) wrapper.
///
/// Generates an interrupt when a Non-Secure or unprivileged access is attempted
/// to a resource marked Secure or Privileged-only by TZSC or MPCBB.
///
/// Each register covers a group of peripherals/blocks. Consult your device
/// reference manual for the number of groups available on your chip.
///
/// Obtain a wrapper via [`Tzic::new`] with `pac::GTZC_TZIC`.
#[cfg(any(gtzc_wba, gtzc_v1))]
pub struct Tzic {
    inner: pac::gtzc::Tzic,
}

#[cfg(any(gtzc_wba, gtzc_v1))]
impl Tzic {
    /// Wrap a PAC TZIC peripheral instance.
    ///
    /// # Safety
    /// The caller must ensure no concurrent access and that this runs in Secure world.
    #[inline]
    pub unsafe fn new(inner: pac::gtzc::Tzic) -> Self {
        Self { inner }
    }

    /// Enable all illegal-access IRQs for register group `reg`.
    #[inline]
    pub fn enable_irqs(&self, reg: usize) {
        self.inner.ier(reg).write(|r| r.set_ie(0xFFFF_FFFF));
    }

    /// Disable all illegal-access IRQs for register group `reg`.
    #[inline]
    pub fn disable_irqs(&self, reg: usize) {
        self.inner.ier(reg).write(|r| r.set_ie(0));
    }

    /// Clear all pending illegal-access flags for register group `reg`.
    #[inline]
    pub fn clear_flags(&self, reg: usize) {
        self.inner.fcr(reg).write(|r| r.set_cf(0xFFFF_FFFF));
    }

    /// Read pending illegal-access flags for register group `reg`.
    ///
    /// Each bit corresponds to one peripheral or SRAM block; `1` = access violation pending.
    #[inline]
    pub fn status(&self, reg: usize) -> u32 {
        self.inner.sr(reg).read().f()
    }

    /// Clear any pending flags then enable IRQs for all `n_regs` register groups.
    ///
    /// Typical call:
    /// - WBA65: `tzic.enable_all(4)`
    /// - U585:  `tzic.enable_all(3)`
    pub fn enable_all(&self, n_regs: usize) {
        for reg in 0..n_regs {
            self.clear_flags(reg);
            self.enable_irqs(reg);
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// TZSC — TrustZone Security Controller (lock / raw access helpers)
// ────────────────────────────────────────────────────────────────────────────

/// Lock the TZSC security and privilege configuration until the next reset.
///
/// After locking, no code (Secure or Non-Secure) can modify the TZSC register
/// contents. Typically called at the end of the Secure world's boot setup.
///
/// For per-peripheral security/privilege configuration access the TZSC PAC registers
/// directly via `pac::GTZC_TZSC` — field names are chip-specific:
///
/// - **WBA** (`gtzc_wba`): `pac::GTZC_TZSC.tzsc_seccfgr1().modify(|r| r.set_usart1sec(false))`
/// - **U5 / H5** (`gtzc_v1`): `pac::GTZC_TZSC.seccfgr1().modify(|r| r.set_usart1sec(false))`
///
/// # Safety
/// Must be called from the Secure world.
#[cfg(any(gtzc_wba, gtzc_v1))]
pub unsafe fn lock() {
    #[cfg(gtzc_wba)]
    pac::GTZC_TZSC.tzsc_cr().modify(|r| r.set_lck(true));

    #[cfg(gtzc_v1)]
    pac::GTZC_TZSC.cr().modify(|r| r.set_lck(true));
}

/// Returns `true` if the TZSC configuration is locked.
#[cfg(any(gtzc_wba, gtzc_v1))]
pub fn is_locked() -> bool {
    #[cfg(gtzc_wba)]
    return pac::GTZC_TZSC.tzsc_cr().read().lck();

    #[cfg(gtzc_v1)]
    return pac::GTZC_TZSC.cr().read().lck();
}

// ────────────────────────────────────────────────────────────────────────────
// MPCWM — Memory Protection Controller Watermark (gtzc_v1 only)
// ────────────────────────────────────────────────────────────────────────────
//
// The MPCWM allows configuring security and privilege of sub-regions within
// external memories (FMC, OCTOSPI, BKPSRAM). Each watermark region A/B has:
//   - CFGRx: SREN (sub-region enable), SRLOCK, SEC, PRIV
//   - Rx:    SUBA_START (granularity), SUBA_LENGTH
//
// Direct access via `pac::GTZC_TZSC.mpcwm1acfgr()` etc. is recommended until
// a higher-level MPCWM API is added.
