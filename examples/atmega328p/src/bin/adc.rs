#![no_std]
#![no_main]

use embassy_atmega328p::adc::{Adc, Reference};
use panic_halt as _;

#[avr_device::entry]
fn main() -> ! {
    let p = embassy_atmega328p::init();
    let mut input = p.PC0;
    let mut adc = Adc::new(p.ADC, Reference::Avcc);

    loop {
        let _sample = adc.read(&mut input);
    }
}
