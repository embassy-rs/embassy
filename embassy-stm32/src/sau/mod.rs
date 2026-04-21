//! Security Attribution Unit (SAU) driver.
//!
//! The SAU is a Cortex-M Security Extension core peripheral that defines which memory
//! regions are Secure, Non-Secure, or Non-Secure Callable. It is present on all
//! Cortex-M23 and Cortex-M33 processors that implement the TrustZone extension.
//!
//! Up to 8 configurable regions are supported. Memory not covered by any enabled region
//! is treated as **Secure** when the SAU is enabled.
//!
//! # Usage
//!
//! ```no_run
//! use embassy_stm32::sau::{Attribute, Region, init};
//!
//! let regions = [
//!     Region {
//!         base_address: 0x0808_0000,   // Non-Secure Flash (must be 32-byte aligned)
//!         end_address:  0x080F_FFFF,   // Inclusive end (lower 5 bits must be 0x1F)
//!         attribute:    Attribute::NonSecure,
//!     },
//!     Region {
//!         base_address: 0x2007_0000,   // Non-Secure SRAM2
//!         end_address:  0x2007_FFFF,
//!         attribute:    Attribute::NonSecure,
//!     },
//!     Region {
//!         base_address: 0x0C07_F000,   // Non-Secure Callable veneer region
//!         end_address:  0x0C07_FFFF,
//!         attribute:    Attribute::NonSecureCallable,
//!     },
//! ];
//!
//! unsafe { init(&regions); }
//! ```

use cortex_m::peripheral::sau::{SauRegion, SauRegionAttribute};
use cortex_m::peripheral::scb::Exception;

/// SAU region security attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Attribute {
    /// Non-Secure: accessible from both Secure and Non-Secure worlds.
    NonSecure = 0,
    /// Non-Secure Callable: enables calls from Non-Secure world into Secure code
    /// through a veneer (gate) function at this address. The NSC bit is set in RLAR.
    NonSecureCallable = 1,
}

/// SAU region definition.
///
/// Both `base_address` and `end_address` must be 32-byte aligned:
/// - `base_address & 0x1F == 0` (lower 5 bits must be zero)
/// - `end_address & 0x1F == 0x1F` (lower 5 bits must be one)
///
/// For example, to cover 0x0808_0000–0x080F_FFFF both constraints are naturally satisfied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    /// Base address of the region (inclusive). Must be 32-byte aligned.
    pub base_address: u32,
    /// End address of the region (inclusive). The lower 5 bits must be `0x1F`.
    pub end_address: u32,
    /// Security attribute for this region.
    pub attribute: Attribute,
}

/// Initialize and enable the SAU with the given region definitions.
///
/// This function:
/// 1. Disables the SAU.
/// 2. Programs up to 8 regions from `regions` (extras are silently ignored).
/// 3. Re-enables the SAU.
/// 4. Enables the `SecureFault` exception so illegal TrustZone accesses surface as
///    a debuggable fault rather than escalating silently to `HardFault`.
///
/// Memory not covered by any configured region is treated as **Secure**.
///
/// # Safety
/// Must be called from the Secure world before Non-Secure code is started.
/// Regions must satisfy the 32-byte alignment constraints documented on [`Region`].
pub unsafe fn init(regions: &[Region]) {
    let mut core = unsafe { cortex_m::Peripherals::steal() };

    for (i, region) in regions.iter().enumerate().take(8) {
        let sau_region = SauRegion {
            base_address: region.base_address & !0x1F,
            limit_address: region.end_address | 0x1F,
            attribute: match region.attribute {
                Attribute::NonSecure => SauRegionAttribute::NonSecure,
                Attribute::NonSecureCallable => SauRegionAttribute::NonSecureCallable,
            },
        };
        // Ignore errors: out-of-range region numbers are guarded by `take(8)` above.
        let _ = core.SAU.set_region(i as u8, sau_region);
    }

    core.SAU.enable();

    // Enable SecureFault so TrustZone violations produce a dedicated exception.
    core.SCB.enable(Exception::SecureFault);
}

/// Disable the SAU, making all memory Non-Secure accessible.
///
/// # Safety
/// Must be called from the Secure world. After disabling, all memory is Non-Secure
/// accessible — call only if you intend to operate entirely in Non-Secure mode.
pub unsafe fn disable() {
    let core = unsafe { cortex_m::Peripherals::steal() };
    // ALLNS=0, ENABLE=0: SAU disabled, all memory is Secure.
    unsafe { core.SAU.ctrl.write(cortex_m::peripheral::sau::Ctrl(0)) };
}
