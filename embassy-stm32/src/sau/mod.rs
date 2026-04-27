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
//!         base_address: 0x0810_0000,   // Non-Secure Flash (must be 32-byte aligned)
//!         end_address:  0x081F_FFFF,   // Inclusive end (lower 5 bits must be 0x1F)
//!         attribute:    Attribute::NonSecure,
//!     },
//!     Region {
//!         base_address: 0x2007_0000,   // Non-Secure SRAM2
//!         end_address:  0x2007_FFFF,
//!         attribute:    Attribute::NonSecure,
//!     },
//!     Region {
//!         base_address: 0x0C0F_E000,   // Non-Secure Callable veneer region
//!         end_address:  0x0C0F_FFFF,
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
/// For example, to cover 0x0810_0000–0x081F_FFFF both constraints are naturally satisfied.
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
/// Sets SAU CTRL.ALLNS=1, ENABLE=0: the SAU is off and all memory is treated as
/// Non-Secure (unless overridden by the IDAU). Use this only when you intend to
/// run entirely in Non-Secure mode with no security boundary enforcement.
///
/// # Safety
/// Must be called from the Secure world.
pub unsafe fn disable() {
    let core = unsafe { cortex_m::Peripherals::steal() };
    // ALLNS=1 (bit 1), ENABLE=0 (bit 0): SAU disabled, all memory Non-Secure.
    unsafe { core.SAU.ctrl.write(cortex_m::peripheral::sau::Ctrl(0b10)) };
}

/// Route an interrupt to the Non-Secure world via the NVIC ITNS registers.
///
/// By default all interrupts target the Secure world. Call this for every interrupt
/// that the Non-Secure application handles (e.g. USART1, TIM2, DMA channels).
///
/// `irq_number` is the peripheral interrupt number as defined in your device's
/// interrupt vector table (0-based, excluding the 16 CPU exceptions).
///
/// # Safety
/// Must be called from the Secure world before jumping to Non-Secure code.
pub unsafe fn route_irq_to_nonsecure(irq_number: u16) {
    // TODO: replace with NVIC::route_to_nonsecure() once cortex-m PR #647 lands
    // (https://github.com/rust-embedded/cortex-m/pull/647).
    //
    // NVIC ITNS registers start at 0xE000_E380 (ARMv8-M only).
    // itns[n] covers interrupts [n*32 .. n*32+31]; setting bit k routes interrupt
    // (n*32 + k) to Non-Secure world.
    const NVIC_ITNS_BASE: *mut u32 = 0xE000_E380 as *mut u32;
    let reg = usize::from(irq_number / 32);
    let bit = irq_number as u32 % 32;
    unsafe {
        let ptr = NVIC_ITNS_BASE.add(reg);
        ptr.write_volatile(ptr.read_volatile() | (1 << bit));
    }
}

/// Enable FPU access from the Non-Secure world.
///
/// Sets SCB->NSACR bits 10–11 (CP10/CP11) so Non-Secure code can use the FPU.
/// Also clears the FPCCR.TS bit so that lazy FP state preservation does not
/// treat FP registers as Secure (prevents accidental FP-register leakage across
/// the security boundary).
///
/// Must be called before jumping to Non-Secure code if the NS application uses
/// floating-point operations. Without this, NS FPU use will raise a UsageFault.
///
/// # Safety
/// Must be called from the Secure world.
pub unsafe fn enable_nonsecure_fpu() {
    // TODO: replace raw SCB NSACR write with SCB::enable_nonsecure_fpu() once
    // cortex-m PR #647 lands (https://github.com/rust-embedded/cortex-m/pull/647).
    //
    // SCB NSACR is at 0xE000_ED8C. Bits 10-11 (CP10/CP11) grant NS access to the FPU.
    const SCB_NSACR: *mut u32 = 0xE000_ED8C as *mut u32;
    const CP10_CP11: u32 = 0b11 << 10;
    unsafe { SCB_NSACR.write_volatile(SCB_NSACR.read_volatile() | CP10_CP11) };

    // FPCCR.TS (bit 26) = 0 means FP state is Non-Secure, preventing Secure FP register
    // contents from leaking to NS code on context switches.
    let core = cortex_m::Peripherals::steal();
    core.FPU.fpccr.modify(|v| v & !(1 << 26));
}

/// Transfer control to the Non-Secure application. Does not return.
///
/// This performs the standard Secure→Non-Secure boot handoff:
/// 1. Sets `SCB_NS->VTOR` to `ns_vtor` so the NS world knows its vector table.
/// 2. Loads `MSP_NS` from the first word of the NS vector table (initial NS stack pointer).
/// 3. Reads the NS reset handler address from the second word of the vector table.
/// 4. Executes `BXNS` to atomically switch to Non-Secure state and jump to the handler.
///
/// # Safety
/// - Must be called from the Secure world after all GTZC/SAU setup is complete.
/// - `ns_vtor` must be a valid Non-Secure vector table address (32-byte aligned per
///   Cortex-M33 requirements; in practice 64-byte or 128-byte alignment is typical).
/// - The Non-Secure reset handler at `*(ns_vtor + 4)` must be a valid Thumb function
///   address (i.e., bit 0 set in the vector table entry, as per ARM ABI convention).
pub unsafe fn jump_to_nonsecure(ns_vtor: u32) -> ! {
    // Configure the Non-Secure vector table.
    // SCB_NS->VTOR is the NS alias of the SCB VTOR register (0xE002_ED08).
    const SCB_NS_VTOR: *mut u32 = 0xE002_ED08 as *mut u32;
    SCB_NS_VTOR.write_volatile(ns_vtor);

    // Load initial NS stack pointer from the first word of the NS vector table.
    let ns_sp = core::ptr::read_volatile(ns_vtor as *const u32);
    core::arch::asm!(
        "msr msp_ns, {sp}",
        sp = in(reg) ns_sp,
        options(nomem, nostack, preserves_flags),
    );

    // Read the NS reset handler address from the second word of the NS vector table.
    // ARM convention: bit 0 = 1 in the vector table (Thumb mode marker).
    // BXNS requires bit 0 = 0 or it raises SecureFault (SFSR.INVTRAN).
    let ns_reset = core::ptr::read_volatile((ns_vtor as *const u32).add(1));

    // BXNS atomically clears bit 0, switches to Non-Secure state, and jumps.
    core::arch::asm!(
        "bxns {entry}",
        entry = in(reg) ns_reset & !1u32,
        options(noreturn),
    );
}
