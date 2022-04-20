#![no_std]
#![no_main]
#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use embassy::time::{Duration, Timer};
use embassy_nrf::{
    gpio::{Level, Output, OutputDrive},
    Peripherals,
};

use panic_reset as _;

#[embassy::main]
async fn main(_s: embassy::executor::Spawner, p: Peripherals) {
    let mut led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);
    //let mut led = Output::new(p.P1_10, Level::Low, OutputDrive::Standard);

    loop {
        led.set_high();
        Timer::after(Duration::from_millis(300)).await;
        led.set_low();
        Timer::after(Duration::from_millis(300)).await;
    }
}
