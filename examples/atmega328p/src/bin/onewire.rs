#![no_std]
#![no_main]

use embassy_atmega328p::delay::Delay;
use embassy_atmega328p::onewire::{OneWire, crc8};
use panic_halt as _;

#[avr_device::entry]
fn main() -> ! {
    let p = embassy_atmega328p::init();
    let mut bus = OneWire::new(p.PD2);
    let mut delay = Delay::new();

    loop {
        if bus.reset() {
            bus.skip_rom();
            bus.write_byte(0x44); // DS18B20: Convert T
            delay.delay_ms(750);

            if bus.reset() {
                bus.skip_rom();
                bus.write_byte(0xbe); // Read Scratchpad
                let mut scratchpad = [0u8; 9];
                for byte in &mut scratchpad {
                    *byte = bus.read_byte();
                }
                let _valid = crc8(&scratchpad[..8]) == scratchpad[8];
            }
        }
    }
}
