#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use embassy_stm32::gpio::{Input, Pull};
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_stm32::init(Default::default());

    let button = Input::new(p.PC13, Pull::Up);

    loop {
        if button.is_high() {
            info!("high");
        } else {
            info!("low");
        }
    }
}
