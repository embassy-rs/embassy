#![no_std]
#![no_main]

use embassy_atmega328p::gpio::{Level, Output};
use embassy_atmega328p::spi::{Config, Spi};
use embedded_hal::spi::SpiBus;
use panic_halt as _;

#[avr_device::entry]
fn main() -> ! {
    let p = embassy_atmega328p::init();
    let mut spi = Spi::new(p.SPI, p.PB2, p.PB3, p.PB4, p.PB5, Config::default()).unwrap();
    let mut chip_select = Output::new(p.PD7, Level::High);

    loop {
        chip_select.set_low();
        let _ = spi.write(&[0x9f]);
        chip_select.set_high();
    }
}
