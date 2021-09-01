#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::panic;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::saadc::{Config, OneShot, Sample};
use embassy_nrf::{interrupt, Peripherals};
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, mut p: Peripherals) {
    let config = Config::default();
    let mut saadc = OneShot::new(p.SAADC, interrupt::take!(SAADC), config);

    loop {
        let sample = saadc.sample(&mut p.P0_02).await;
        info!("sample: {=i16}", sample);
        Timer::after(Duration::from_millis(100)).await;
    }
}
