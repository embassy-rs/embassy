#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let p = embassy_stm32::init(Default::default());

    let mut out1 = Output::new(p.PA8, Level::Low, Speed::High);

    out1.set_high();
    Timer::after(Duration::from_millis(500)).await;
    out1.set_low();

    Timer::after(Duration::from_millis(500)).await;
    info!("end program");

    cortex_m::asm::bkpt();
}
