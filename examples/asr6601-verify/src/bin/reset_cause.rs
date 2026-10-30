//! Record RCC.RST_SR before doing anything else.
//!
//! Flash this, then power-cycle the board. The firmware stores the reset
//! status at `RST_SR_AT_BOOT`, then sleeps.

#![no_std]
#![no_main]

use core::ptr::{addr_of_mut, write_volatile};

use cortex_m_rt::entry;
use embassy_asr::pac;
use embassy_asr6601_verify::enable_debug_in_low_power;
use panic_halt as _;

const MAGIC: u32 = 0x5253_5453; // "RSTS"
const DONE: u32 = 0x444f_4e45;

#[unsafe(no_mangle)]
pub static mut RST_SR_AT_BOOT: u32 = 0;
#[unsafe(no_mangle)]
pub static mut RST_STATUS: u32 = 0;
#[unsafe(no_mangle)]
pub static mut RST_DONE: u32 = 0;

#[entry]
fn main() -> ! {
    let rcc = unsafe { pac::Rcc::steal() };
    let rst_sr = rcc.rst_sr().read().bits();
    unsafe {
        write_volatile(addr_of_mut!(RST_SR_AT_BOOT), rst_sr);
        write_volatile(addr_of_mut!(RST_STATUS), MAGIC);
    }

    enable_debug_in_low_power();

    unsafe {
        write_volatile(addr_of_mut!(RST_DONE), DONE);
    }

    loop {
        cortex_m::asm::wfi();
    }
}
