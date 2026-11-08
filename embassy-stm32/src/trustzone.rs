//! Helpers for building a Secure TrustZone image.
//!
//! TrustZone applications consist of two independently linked images. The Secure image configures
//! the device security attribution, then transfers control to the Non-Secure image. This module
//! provides the Armv8-M part of that setup. Device-specific attribution, such as STM32 GTZC
//! peripheral and MPCBB SRAM configuration, deliberately remains in the PAC because it differs
//! between STM32 families.
//!
//! Enable this module with the [`trustzone-secure`](crate#feature-flags) feature. The feature must
//! only be enabled for the Secure image; the Non-Secure image should use the normal HAL feature
//! set.
//!
//! # Setup order
//!
//! 1. Link the Secure and Non-Secure images into distinct flash and RAM ranges.
//! 2. Configure STM32-specific GTZC/MPCBB attribution through [`crate::pac`].
//! 3. Route each peripheral interrupt the Non-Secure image handles with
//!    [`NVIC::route_to_nonsecure`](cortex_m::peripheral::NVIC::route_to_nonsecure).
//!    Interrupts left alone stay Secure, which is the reset default.
//! 4. Call [`configure`] with the Non-Secure flash/RAM regions and the Secure image's
//!    Non-Secure Callable (NSC) region.
//! 5. Call [`boot_nonsecure`] with the base address of the Non-Secure vector table.
//!
//! These helpers require `cortex-m` 0.7.9 or newer. [`boot_nonsecure`] (`asm::bootload_ns`),
//! [`SCB::enable_nonsecure_fpu`](cortex_m::peripheral::SCB::enable_nonsecure_fpu), and
//! [`NVIC::route_to_nonsecure`](cortex_m::peripheral::NVIC::route_to_nonsecure) come from
//! that line of releases.
//!
//! Regions not covered by an SAU entry are Secure. SAU regions must be 32-byte aligned; this is
//! checked by [`configure`]. The linker must place compiler-generated CMSE veneers in the same NSC
//! region declared to the SAU.
//!
//! # Secure entry functions
//!
//! Cross-world entry functions use Rust's `"cmse-nonsecure-entry"` ABI, which is still
//! feature-gated (`#![feature(cmse_nonsecure_entry)]`). They must validate all pointers
//! received from Non-Secure code (for example with [`cortex_m::cmse::TestTarget`]) and
//! must not expose secrets through return values or shared memory.
//!
//! `examples/trustzone/stm32wba65-secure` and `examples/trustzone/stm32wba65-nonsecure` are a
//! complete pair of images for STM32WBA65. That example supplies the secure-gateway veneer
//! in assembly so it builds on the stable toolchain.

use cortex_m::peripheral::sau::{SauError, SauRegion};
pub use cortex_m::peripheral::sau::{SauRegion as Region, SauRegionAttribute as RegionAttribute};
use cortex_m::peripheral::scb::Exception;
use cortex_m::peripheral::{Peripherals, SCBNS};

/// Configure the Arm Security Attribution Unit (SAU) for a Secure application.
///
/// This enables `SecureFault`, programs `regions`, and enables the SAU. Configure STM32-specific
/// GTZC peripheral and MPCBB SRAM attribution before calling this function.
///
/// The region order is their SAU priority order: a higher numbered matching region takes
/// precedence over a lower numbered one.
pub fn configure(core: &mut Peripherals, regions: &[SauRegion]) -> Result<(), SauError> {
    core.SCB.enable(Exception::SecureFault);
    core.SAU.init(regions)
}

/// Transfer execution from the Secure image to a Non-Secure image.
///
/// `vector_table` must point to a valid, suitably aligned Non-Secure vector table in memory
/// configured as Non-Secure by the SAU and the STM32 implementation-defined attribution unit.
/// The caller must have completed all device-specific security configuration before this call.
///
/// This never returns.
///
/// # Safety
///
/// Calling this with an invalid vector table, or before establishing the Secure/Non-Secure
/// boundary, can transfer control with an invalid stack pointer or weaken the intended security
/// boundary.
pub unsafe fn boot_nonsecure(vector_table: *const u32, scb_ns: SCBNS) -> ! {
    unsafe { cortex_m::asm::bootload_ns(vector_table, scb_ns) }
}
