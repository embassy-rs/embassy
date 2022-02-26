#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use embassy::executor::Spawner;
use embassy::time::Duration;
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

    // We want the task start to effectively short with the last one ending so
    // we don't miss any samples. The Saadc will trigger the initial TASKS_START.
    let mut start_ppi = Ppi::new_one_to_one(p.PPI_CH0, saadc.event_end(), saadc.task_start());
    start_ppi.enable();

    let mut timer = Timer::new(p.TIMER0);
    timer.set_frequency(Frequency::F1MHz);
    timer.cc(0).write(1000); // We want to sample at 1KHz
    timer.cc(0).short_compare_clear();

    let mut sample_ppi =
        Ppi::new_one_to_one(p.PPI_CH1, timer.cc(0).event_compare(), saadc.task_sample());

    timer.start();

    // This delay demonstrates that starting the timer prior to running
    // the task sampler is benign given the calibration that follows.
    embassy::time::Timer::after(Duration::from_millis(500)).await;
    saadc.calibrate().await;

    let mut bufs = [[[0; 3]; 500]; 2];

    let mut c = 0;
    let mut a: i32 = 0;

    saadc
        .run_task_sampler(
            &mut bufs,
            || {
                sample_ppi.enable();
            },
            move |buf| {
                // NOTE: It is important that the time spent within this callback
                // does not exceed the time taken to acquire the 1500 samples we
                // have in this example, which would be 10us + 2us per
                // sample * 1500 = 18ms. You need to measure the time taken here
                // and set the sample buffer size accordingly. Exceeding this
                // time can lead to the peripheral re-writing the other buffer.
                for b in buf {
                    a += b[0] as i32;
                }
                c += buf.len();
                if c > 1000 {
                    a = a / c as i32;
                    info!("channel 1: {=i32}", a);
                    c = 0;
                    a = 0;
                }
                SamplerState::Sampled
            },
        )
        .await;
}
