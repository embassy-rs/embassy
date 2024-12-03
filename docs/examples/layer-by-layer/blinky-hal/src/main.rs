#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    let p = embassy_stm32::init(Default::default());
    let mut led = Output::new(p.PB14, Level::High, Speed::VeryHigh);
    let button = Input::new(p.PC13, Pull::Up);

    loop {
        if button.is_low() {
            led.set_high();
        } else {
            led.set_low();
        }
    }
}
