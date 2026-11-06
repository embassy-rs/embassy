//! SPI internal loopback test (PL022 LBM bit).
//!
//! No external wiring needed. The hardware loops TX back into RX.

#![no_std]
#![no_main]

use core::ptr::{addr_of_mut, write_volatile};

use cortex_m_rt::entry;
use embassy_asr::gpio::AlternateFunction;
use embassy_asr::pac;
use embassy_asr::spi::{Config as SpiConfig, Spi};
use embassy_asr6601_verify::enable_debug_in_low_power;
use panic_halt as _;

const WORDS: usize = 16;
const MAGIC: u32 = 0x5350_4921; // "SPI!"
const DONE: u32 = 0x444f_4e45;

#[unsafe(no_mangle)]
pub static mut SPI_RESULTS: [u32; WORDS] = [0; WORDS];

fn word(index: usize) -> *mut u32 {
    unsafe { addr_of_mut!(SPI_RESULTS).cast::<u32>().add(index) }
}

fn put(index: usize, value: u32) {
    unsafe { write_volatile(word(index), value) }
}

#[entry]
fn main() -> ! {
    put(0, MAGIC);
    put(1, 1);

    let p = embassy_asr::init(Default::default());
    enable_debug_in_low_power();

    let mut spi = Spi::new_blocking_af(
        p.SSP2,
        p.PA8,
        AlternateFunction::Function4,
        p.PA12,
        AlternateFunction::Function4,
        p.PA13,
        AlternateFunction::Function4,
        SpiConfig::default(),
    )
    .unwrap();

    let regs = unsafe { pac::Ssp2::steal() };
    regs.cr1().modify(|_, w| w.lbm().set_bit());

    let expected = [0x55u8, 0xaa, 0x3c, 0xc3];
    let mut buffer = expected;
    let result = spi.blocking_transfer_in_place(&mut buffer);

    put(2, if result.is_ok() { 1 } else { 0 });
    put(3, u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]));
    put(
        4,
        u32::from_le_bytes([expected[0], expected[1], expected[2], expected[3]]),
    );
    put(5, if buffer == expected { 1 } else { 0 });
    put(6, regs.cr1().read().bits());
    put(7, regs.sr().read().bits());
    put(1, DONE);

    loop {
        cortex_m::asm::wfi();
    }
}
