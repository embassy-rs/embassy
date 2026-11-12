//! DAC linearity helper.
//!
//! The host can change `DAC_REQUEST` via `probe-rs write`; the firmware applies
//! the new 12-bit code and publishes it in `DAC_ACTIVE`.
//!
//! GPIO09 (PA9) is the DAC output.

#![no_std]
#![no_main]

use core::ptr::{addr_of_mut, read_volatile, write_volatile};

use cortex_m_rt::entry;
use embassy_asr::dac::{Config as DacConfig, Dac};
use embassy_asr6601_verify::enable_debug_in_low_power;
use panic_halt as _;

#[unsafe(no_mangle)]
pub static mut DAC_REQUEST: u32 = 2048;
#[unsafe(no_mangle)]
pub static mut DAC_ACTIVE: u32 = u32::MAX;
#[unsafe(no_mangle)]
pub static mut DAC_UPDATES: u32 = 0;

fn write_word(ptr: *mut u32, value: u32) {
    unsafe { write_volatile(ptr, value) }
}

fn read_word(ptr: *const u32) -> u32 {
    unsafe { read_volatile(ptr) }
}

#[entry]
fn main() -> ! {
    let p = embassy_asr::init(Default::default());
    enable_debug_in_low_power();
    let mut dac = Dac::new(p.DAC, p.PA9, DacConfig::default());

    let request = addr_of_mut!(DAC_REQUEST).cast::<u32>();
    let active = addr_of_mut!(DAC_ACTIVE).cast::<u32>();
    let updates = addr_of_mut!(DAC_UPDATES).cast::<u32>();

    let mut last = u32::MAX;
    loop {
        let code = read_word(request);
        if code <= 4095 && code != last {
            if dac.set(code as u16).is_ok() {
                write_word(active, code);
            } else {
                write_word(active, 0x8000_0000 | code);
            }
            write_word(updates, read_word(updates).wrapping_add(1));
            last = code;
        }
    }
}
