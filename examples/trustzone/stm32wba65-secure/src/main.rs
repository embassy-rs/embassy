//! Secure image for the STM32WBA65 TrustZone example.
//!
//! This image programs the SAU and the SRAM1 block controller, then transfers
//! control to the non-secure image. The two images are linked separately.
//!
//! # Memory
//!
//! Secure flash is the bank 1 secure alias, `0x0C00_0000` for 1 MiB. Secure RAM
//! is SRAM2, `0x3007_0000` for 64 KiB. Non-secure flash is bank 2, `0x0810_0000`
//! for 1 MiB. Non-secure RAM is SRAM1, `0x2000_0000` for 448 KiB.
//!
//! `secure_add` is the only non-secure callable entry. Its secure-gateway veneer is
//! linked at `0x0C0F_F000`, which is the address the non-secure image calls. The
//! `"cmse-nonsecure-entry"` ABI is still feature-gated, so the veneer is assembly
//! and the body is a normal Rust function.
//!
//! # Option bytes
//!
//! Program these once with STM32CubeProgrammer, then reset the chip. Reset with
//! `TZEN` set boots this image from `0x0C00_0000`.
//!
//! * `FLASH_OPTR.TZEN = 1`
//! * `SECWM1_PSTRT = 0x00` and `SECWM1_PEND = 0x7F`, so bank 1 is entirely secure
//! * `SECWM2_PSTRT = 0x7F` and `SECWM2_PEND = 0x00`, so bank 2 has no secure pages
//!   (`PSTRT > PEND` is an empty secure area)
//!
//! When the option bytes differ, this image reports that over semihosting and
//! waits. Flash the non-secure image, then run this one under probe-rs:
//!
//! ```text
//! cargo run --release --manifest-path examples/trustzone/stm32wba65-nonsecure/Cargo.toml
//! cargo run --release --manifest-path examples/trustzone/stm32wba65-secure/Cargo.toml
//! ```
//!
//! With `TZEN` set, reset always enters this image, which then boots the
//! non-secure image. Both images print over semihosting.

#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m_rt::{entry, exception};
use cortex_m_semihosting::hprintln;
use embassy_stm32::pac::flash::Flash;
use embassy_stm32::pac::gtzc::Mpcbb;
use embassy_stm32::pac::rcc::Rcc;
use embassy_stm32::pac::{self};
use embassy_stm32::trustzone::{self, Region, RegionAttribute};

/// Secure-alias offset for peripherals whose non-secure base is `0x4000_0000`.
const SECURE_ALIAS: usize = 0x1000_0000;

const NS_FLASH: u32 = 0x0810_0000;
const NS_FLASH_LIMIT: u32 = 0x081F_FFFF;
const NS_SRAM: u32 = 0x2000_0000;
const NS_SRAM_LIMIT: u32 = 0x2006_FFFF;
const NS_PERIPH: u32 = 0x4000_0000;
const NS_PERIPH_LIMIT: u32 = 0x4FFF_FFFF;

/// Non-secure callable veneer. Matches `veneer.ld` and the non-secure image.
const NSC_VENEER: u32 = 0x0C0F_F000;
const NSC_VENEER_LIMIT: u32 = 0x0C0F_F01F;

/// Bank 1 pages 0 through 127. Page 127 holds the veneer.
const BANK1_PSTRT: u8 = 0x00;
const BANK1_PEND: u8 = 0x7F;

/// SRAM1 is 448 KiB. Each MPCBB superblock register covers 32 blocks of 512 bytes.
const SRAM1_SUPERBLOCKS: usize = 28;

#[entry]
fn main() -> ! {
    let flash = secure_flash();
    require_option_bytes(flash);

    let rcc = secure_rcc();
    rcc.ahb1enr().modify(|w| w.set_gtzc1en(true));

    make_sram1_nonsecure(secure_mpcbb1());
    asm::dsb();
    asm::isb();

    let mut cp = cortex_m::Peripherals::take().unwrap();
    cp.SCB.enable_nonsecure_fpu();
    if let Err(err) = trustzone::configure(
        &mut cp,
        &[
            Region {
                base_address: NS_FLASH,
                limit_address: NS_FLASH_LIMIT,
                attribute: RegionAttribute::NonSecure,
            },
            Region {
                base_address: NS_SRAM,
                limit_address: NS_SRAM_LIMIT,
                attribute: RegionAttribute::NonSecure,
            },
            Region {
                base_address: NS_PERIPH,
                limit_address: NS_PERIPH_LIMIT,
                attribute: RegionAttribute::NonSecure,
            },
            Region {
                base_address: NSC_VENEER,
                limit_address: NSC_VENEER_LIMIT,
                attribute: RegionAttribute::NonSecureCallable,
            },
        ],
    ) {
        let _ = hprintln!("SAU configuration failed: {:?}", err);
        loop {
            asm::bkpt();
        }
    }
    asm::dsb();
    asm::isb();

    let _ = hprintln!("Booting non-secure image at {:#010x}", NS_FLASH);
    unsafe { trustzone::boot_nonsecure(NS_FLASH as *const u32, cp.SCBNS) }
}

/// Body of the non-secure callable `secure_add` entry.
///
/// The symbol `secure_add` is the assembly veneer at [`NSC_VENEER`]. It executes
/// `sg`, calls this function, clears the registers that could leak secure state,
/// and returns to non-secure code with `bxns`.
#[unsafe(no_mangle)]
extern "C" fn secure_add_body(value: u32) -> u32 {
    let result = value.wrapping_add(1);
    let _ = hprintln!("secure_add({}) = {}", value, result);
    result
}

// The veneer is 8 bytes. `veneer.ld` requires the padded NSC region to stay 32 bytes.
core::arch::global_asm!(
    r#"
    .section .gnu.sgstubs, "ax"
    .arch armv8-m.main
    .thumb
    .syntax unified
    .global secure_add
    .type secure_add, %function
    .thumb_func
secure_add:
    sg
    b.w secure_add_entry
    .size secure_add, . - secure_add

    .section .text.secure_add_entry, "ax"
    .arch armv8-m.main
    .fpu fpv5-sp-d16
    .thumb
    .syntax unified
    .global secure_add_entry
    .type secure_add_entry, %function
    .thumb_func
secure_add_entry:
    stmdb sp!, {{r4-r12, lr}}
    bl secure_add_body
    mov r1, #0
    mov r2, #0
    mov r3, #0
    vmsr fpscr, r1
    vmov d0, r1, r1
    vmov d1, r1, r1
    vmov d2, r1, r1
    vmov d3, r1, r1
    vmov d4, r1, r1
    vmov d5, r1, r1
    vmov d6, r1, r1
    vmov d7, r1, r1
    vmov d8, r1, r1
    vmov d9, r1, r1
    vmov d10, r1, r1
    vmov d11, r1, r1
    vmov d12, r1, r1
    vmov d13, r1, r1
    vmov d14, r1, r1
    vmov d15, r1, r1
    ldmia sp!, {{r4-r12, lr}}
    bxns lr
    .size secure_add_entry, . - secure_add_entry
    "#
);

#[exception]
fn SecureFault() {
    let _ = hprintln!("SecureFault");
    loop {
        asm::bkpt();
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = hprintln!("secure panic: {}", info);
    loop {
        asm::bkpt();
    }
}

/// A secure CPU access through the non-secure peripheral alias is a non-secure
/// transaction. Secure-only registers are reached through the secure alias.
fn secure_peripheral(ns_base: *mut ()) -> *mut () {
    ns_base.wrapping_byte_add(SECURE_ALIAS)
}

fn secure_flash() -> Flash {
    unsafe { Flash::from_ptr(secure_peripheral(pac::FLASH.as_ptr())) }
}

fn secure_rcc() -> Rcc {
    unsafe { Rcc::from_ptr(secure_peripheral(pac::RCC.as_ptr())) }
}

fn secure_mpcbb1() -> Mpcbb {
    unsafe { Mpcbb::from_ptr(secure_peripheral(pac::GTZC_MPCBB1.as_ptr())) }
}

fn require_option_bytes(flash: Flash) {
    if !flash.optr().read().tzen() {
        halt("Set FLASH_OPTR.TZEN and reset before running this example.");
    }

    let bank1 = flash.secwm1r1().read();
    if bank1.secwm1_pstrt() != BANK1_PSTRT || bank1.secwm1_pend() != BANK1_PEND {
        let _ = hprintln!(
            "Bank 1 watermark is PSTRT={:#04x} PEND={:#04x}.",
            bank1.secwm1_pstrt(),
            bank1.secwm1_pend()
        );
        halt("Set SECWM1_PSTRT=0x00 and SECWM1_PEND=0x7F so bank 1 is entirely secure.");
    }

    let bank2 = flash.secwm2r1().read();
    if bank2.secwm2_pstrt() <= bank2.secwm2_pend() {
        let _ = hprintln!(
            "Bank 2 watermark is PSTRT={:#04x} PEND={:#04x}.",
            bank2.secwm2_pstrt(),
            bank2.secwm2_pend()
        );
        halt("Set SECWM2_PSTRT=0x7F and SECWM2_PEND=0x00 so bank 2 has no secure pages.");
    }
}

fn make_sram1_nonsecure(mpcbb1: Mpcbb) {
    // Secure data accesses to non-secure SRAM blocks are allowed. Instruction
    // fetches from those blocks stay illegal.
    mpcbb1.cr().modify(|w| w.set_srwiladis(true));

    // `Mpcbb::seccfgr` only accepts the first four superblocks. SRAM1 has 28.
    let base = mpcbb1.as_ptr() as *mut u8;
    unsafe {
        for superblock in 0..SRAM1_SUPERBLOCKS {
            core::ptr::write_volatile(base.add(0x100 + superblock * 4).cast(), 0);
            core::ptr::write_volatile(base.add(0x200 + superblock * 4).cast(), 0);
        }
    }
}

fn halt(message: &str) -> ! {
    let _ = hprintln!("{}", message);
    loop {
        asm::bkpt();
    }
}
