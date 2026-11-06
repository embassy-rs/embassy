//! UART0 loopback verification on GPIO16/GPIO17.
//!
//! Connect GPIO16 (PB0) and GPIO17 (PB1) together.

#![no_std]
#![no_main]

use core::ptr::{addr_of_mut, write_volatile};

use cortex_m_rt::entry;
use embassy_asr::gpio::AlternateFunction;
use embassy_asr::pac;
use embassy_asr::uart::{Config as UartConfig, Uart};
use embassy_asr6601_verify::enable_debug_in_low_power;
use panic_halt as _;

const WORDS: usize = 16;
const MAGIC: u32 = 0x5541_5254; // "UART"
const DONE: u32 = 0x444f_4e45;

#[unsafe(no_mangle)]
pub static mut UART_RESULTS: [u32; WORDS] = [0; WORDS];

fn word(index: usize) -> *mut u32 {
    unsafe { addr_of_mut!(UART_RESULTS).cast::<u32>().add(index) }
}

fn put(index: usize, value: u32) {
    unsafe { write_volatile(word(index), value) }
}

fn read_loopback() -> (u32, u32) {
    let regs = unsafe { pac::Uart0::steal() };
    let mut packed = 0;
    for index in 0..4 {
        let mut byte = 0;
        let mut found = false;
        for _ in 0..2_000_000 {
            if !regs.fr().read().rxfe().bit_is_set() {
                byte = regs.dr().read().data().bits();
                found = true;
                break;
            }
            core::hint::spin_loop();
        }
        if !found {
            return (packed, 0xffff_ffff);
        }
        packed |= u32::from(byte) << (index * 8);
    }
    (packed, 0)
}

#[entry]
fn main() -> ! {
    put(0, MAGIC);
    put(1, 1);

    let p = embassy_asr::init(Default::default());
    enable_debug_in_low_power();

    let pattern = [0x55u8, 0xa5, 0x3c, 0xc3];

    let mut uart = Uart::new_blocking(
        p.UART0,
        Some((p.PB1, AlternateFunction::Function1)),
        Some((p.PB0, AlternateFunction::Function1)),
        UartConfig::default(),
    )
    .unwrap();
    let _ = uart.blocking_write(&pattern);
    let _ = uart.blocking_flush();
    let (byte_a, raw_a) = read_loopback();
    drop(uart);

    let p2 = unsafe { embassy_asr::Peripherals::steal() };
    let mut uart = Uart::new_blocking(
        p2.UART0,
        Some((p2.PB0, AlternateFunction::Function1)),
        Some((p2.PB1, AlternateFunction::Function1)),
        UartConfig::default(),
    )
    .unwrap();
    let _ = uart.blocking_write(&pattern);
    let _ = uart.blocking_flush();
    let (byte_b, raw_b) = read_loopback();

    put(2, 0x100);
    put(3, byte_a);
    put(4, raw_a);
    put(5, byte_b);
    put(6, raw_b);
    put(7, if byte_a == 0xc33c_a555 { 1 } else { 0 });
    put(8, if byte_b == 0xc33c_a555 { 1 } else { 0 });
    put(9, 0xc33c_a555);
    put(1, DONE);

    loop {
        cortex_m::asm::wfi();
    }
}
