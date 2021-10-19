#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::panic;
use embassy::executor::Spawner;
use embassy_nrf::ppi::Ppi;
use embassy_nrf::saadc::{ChannelConfig, Config, Saadc, SamplerState};
use embassy_nrf::timer::{Frequency, Timer};
use embassy_nrf::{interrupt, Peripherals};
use example_common::*;

// Demonstrates both continuous sampling and scanning multiple channels driven by a PPI linked timer

#[embassy::main]
async fn main(_spawner: Spawner, mut p: Peripherals) {
    let config = Config::default();
    let channel_1_config = ChannelConfig::single_ended(&mut p.P0_02);
    let channel_2_config = ChannelConfig::single_ended(&mut p.P0_03);
    let channel_3_config = ChannelConfig::single_ended(&mut p.P0_04);
    let mut saadc = Saadc::new(
        p.SAADC,
        interrupt::take!(SAADC),
        config,
        [channel_1_config, channel_2_config, channel_3_config],
    );

    let mut timer = Timer::new(p.TIMER0);
    timer.set_frequency(Frequency::F1MHz);
    timer.cc(0).write(100); // We want to sample at 10KHz
    timer.cc(0).short_compare_clear();

    let mut ppi = Ppi::new_one_to_one(p.PPI_CH0, timer.cc(0).event_compare(), saadc.task_sample());
    ppi.enable();

    timer.start();

    let mut bufs = [[[0; 3]; 50]; 2];

    let mut c = 0;
    let mut a: i32 = 0;

    saadc
        .run_task_sampler(&mut bufs, move |buf| {
            for b in buf {
                a += b[0] as i32;
            }
            c += buf.len();
            if c > 10000 {
                a = a / c as i32;
                info!("channel 1: {=i32}", a);
                c = 0;
                a = 0;
            }
            SamplerState::Sampled
        })
        .await;
}
