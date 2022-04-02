#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use defmt_rtt as _; // global logger
use embassy_stm32::gpio::{Input, Pull};
use panic_probe as _;

#[cortex_m_rt::entry]
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
