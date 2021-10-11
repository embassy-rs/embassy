#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::panic;
use embassy::executor::Spawner;
use embassy_nrf::saadc::{ChannelConfig, Config, Continuous};
use embassy_nrf::{interrupt, Peripherals};
use example_common::*;
use futures::StreamExt;

#[embassy::main]
async fn main(_spawner: Spawner, mut p: Peripherals) {
    let config = Config::default();
    let channel_config = ChannelConfig::single_ended(&mut p.P0_02);
    let mut saadc = Continuous::new(p.SAADC, interrupt::take!(SAADC), config, [channel_config]);

    let mut double_buf = [[0; 500]; 2]; // A double buffer for 10_000 samples / 1/20th second
    let mut samples = saadc.sample(&mut double_buf, 1600); // 10_000 samples per second
    while let Some(buf) = samples.next().await {
        info!(
            "samples: {=i16}, {=i16}, {=i16}...",
            &buf[0], &buf[1], &buf[2]
        );
    }
}
