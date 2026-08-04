#![no_std]
#![no_main]

use core::fmt::Write;

use embassy_atmega328p::usart::Usart;
use panic_halt as _;

#[avr_device::entry]
fn main() -> ! {
    let p = embassy_atmega328p::init();
    let mut usart = Usart::new(p.USART0, p.PD0, p.PD1, 115_200).unwrap();
    writeln!(usart, "Hello from Embassy on ATmega328P!").unwrap();

    loop {
        if let Ok(byte) = usart.read_byte() {
            usart.write_byte(byte);
        }
    }
}
