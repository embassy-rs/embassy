#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mspm0::gpio::{Input, Level, Output, Pull};
use embassy_mspm0::peripherals::PB3;
use embassy_mspm0::{Config, bind_interrupts, gpio};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    GROUP1 => gpio::InterruptHandler<PB3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");

    let p = embassy_mspm0::init(Config::default());

    let led1 = p.PA0;
    let s2 = p.PB3;

    let mut led1 = Output::new(led1, Level::Low);
    let mut s2 = Input::new(s2, Irqs, Pull::Up);

    // led1 is active low
    led1.set_high();

    loop {
        s2.wait_for_falling_edge().await;

        info!("Switch 2 was pressed");

        led1.toggle();
    }
}
