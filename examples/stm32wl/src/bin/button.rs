#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_stm32::init(Default::default());

    let button = Input::new(p.PA0, Pull::Up);
    let mut led1 = Output::new(p.PB15, Level::High, Speed::Low);
    let mut led2 = Output::new(p.PB9, Level::High, Speed::Low);

    loop {
        if button.is_high() {
            led1.set_high();
            led2.set_low();
        } else {
            led1.set_low();
            led2.set_high();
        }
    }
}
