#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::panic;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::saadc::{ChannelConfig, Config, OneShot};
use embassy_nrf::{interrupt, Peripherals};
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, mut p: Peripherals) {
    let config = Config::default();
    let channel_config = ChannelConfig::single_ended(&mut p.P0_02);
    let mut saadc = OneShot::new(p.SAADC, interrupt::take!(SAADC), config, [channel_config]);

    loop {
        let mut buf = [0; 1];
        saadc.sample(&mut buf).await;
        info!("sample: {=i16}", &buf[0]);
        Timer::after(Duration::from_millis(100)).await;
    }
}
