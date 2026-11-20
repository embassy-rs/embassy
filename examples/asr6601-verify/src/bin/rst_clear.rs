//! Verify RCC.RST_SR write-1-to-clear behavior.

#![no_std]
#![no_main]

use core::ptr::{addr_of_mut, write_volatile};

use cortex_m_rt::entry;
use embassy_asr::pac;
use embassy_asr6601_verify::enable_debug_in_low_power;
use panic_halt as _;

#[unsafe(no_mangle)]
pub static mut RST_BEFORE: u32 = 0;
#[unsafe(no_mangle)]
pub static mut RST_AFTER: u32 = 0;
#[unsafe(no_mangle)]
pub static mut RST_CLEARED: u32 = 0;

#[entry]
fn main() -> ! {
    let rcc = unsafe { pac::Rcc::steal() };
    let before = rcc.rst_sr().read().bits();
    unsafe {
        write_volatile(addr_of_mut!(RST_BEFORE), before);
    }

    if before & (1 << 6) != 0 {
        unsafe {
            rcc.rst_sr().write_with_zero(|w| w.bor_reset_sr().clear_bit_by_one());
        }
    }

    let after = rcc.rst_sr().read().bits();
    unsafe {
        write_volatile(addr_of_mut!(RST_AFTER), after);
        write_volatile(addr_of_mut!(RST_CLEARED), u32::from(after & (1 << 6) == 0));
    }

    enable_debug_in_low_power();

    loop {
        cortex_m::asm::wfi();
    }
}
