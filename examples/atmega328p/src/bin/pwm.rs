#![no_std]
#![no_main]

use embassy_atmega328p::delay::Delay;
use embassy_atmega328p::pwm::{Pwm0, Timer0Prescaler};
use panic_halt as _;

#[avr_device::entry]
fn main() -> ! {
    let p = embassy_atmega328p::init();
    let mut pwm = Pwm0::new(p.TIMER0, p.PD6, p.PD5, Timer0Prescaler::Div64);
    let mut delay = Delay::new();

    loop {
        for duty in 0..=u8::MAX {
            pwm.set_duty_a(duty);
            pwm.set_duty_b(u8::MAX - duty);
            delay.delay_ms(4);
        }
    }
}
