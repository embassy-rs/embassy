#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

use embedded_hal::delay::DelayNs as _;
use nrf_pac as pac;
use pac::gpio::vals;
use riscv::{self as _, delay::McycleDelay};

#[entry]
fn main() -> ! {
    let port1 = pac::P1_S;
    port1.pin_cnf(10).write(|w| {
        w.set_dir(vals::Dir::OUTPUT);
        w.set_input(vals::Input::DISCONNECT);
        w.set_pull(vals::Pull::DISABLED);
        w.set_drive0(vals::Drive::S);
        w.set_drive1(vals::Drive::S);
        w.set_sense(vals::Sense::DISABLED);
    });

    // 32 MHz seems to be the correct frequency for the RISCV core,
    // but it's not documented in the datasheet.
    const TICKS_PER_SECOND: u32 = 32_000_000;

    let mut delay = McycleDelay::new(TICKS_PER_SECOND);

    // Enable cycle counter, by clearing the CY inhibit bit
    unsafe {
        riscv::register::mcountinhibit::clear_cy();
    }

    loop {
        delay.delay_ms(1_000);
        port1.outclr().write(|w| w.set_pin(10, true));
        delay.delay_ms(1_000);
        port1.outset().write(|w| w.set_pin(10, true));
    }
}
