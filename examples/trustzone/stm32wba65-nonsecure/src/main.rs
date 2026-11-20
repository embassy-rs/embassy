//! Non-secure image for the STM32WBA65 TrustZone example.
//!
//! The secure image boots this image from `0x0810_0000` after opening bank 2,
//! SRAM1, and the non-secure peripheral alias. [`SECURE_ADD`] is the secure
//! gateway veneer linked into the secure image at `0x0C0F_F000`.
//!
//! Flash this image before running `examples/trustzone/stm32wba65-secure`.
//! With `TZEN` set, reset enters the secure image, which jumps here. Output is
//! semihosting, so run under probe-rs.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

/// Thumb address of the secure `secure_add` veneer (`0x0C0F_F000` with the Thumb bit set).
const SECURE_ADD: usize = 0x0C0F_F001;

#[entry]
fn main() -> ! {
    let secure_add: extern "C" fn(u32) -> u32 = unsafe { core::mem::transmute(SECURE_ADD) };
    let result = secure_add(41);
    let _ = hprintln!("non-secure: secure_add(41) = {}", result);
    loop {
        cortex_m::asm::nop();
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = hprintln!("non-secure panic: {}", info);
    loop {
        cortex_m::asm::bkpt();
    }
}
