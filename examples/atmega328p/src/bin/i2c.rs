#![no_std]
#![no_main]

use embassy_atmega328p::i2c::I2c;
use embedded_hal::i2c::I2c as _;
use panic_halt as _;

#[avr_device::entry]
fn main() -> ! {
    let p = embassy_atmega328p::init();
    let mut i2c = I2c::new(p.TWI, p.PC4, p.PC5, 100_000).unwrap();
    let mut byte = [0u8; 1];

    loop {
        // Replace the address/register with those of the connected device.
        let _ = i2c.write_read(0x50, &[0x00], &mut byte);
    }
}
