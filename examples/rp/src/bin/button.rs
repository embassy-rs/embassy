#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy::executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::Peripherals;

use defmt_rtt as _; // global logger
use panic_probe as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let button = Input::new(p.PIN_28, Pull::Up);
    let mut led = Output::new(p.PIN_25, Level::Low);

    loop {
        if button.is_high() {
            led.set_high();
        } else {
            led.set_low();
        }
    }
}
