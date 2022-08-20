#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::interrupt;
use embassy_nrf::pdm::{Config, Channels, Pdm, SamplerState};
use embassy_nrf::timer::Frequency;
use fixed::types::I7F1;
use num_integer::Roots;
use {defmt_rtt as _, panic_probe as _};

// Demonstrates both continuous sampling and scanning multiple channels driven by a PPI linked timer

#[embassy_executor::main]
async fn main(_p: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    let mut config = Config::default();
    // Pins are correct for the onboard microphone on the Feather nRF52840 Sense.
    config.channels = Channels::Mono;
    config.gain_left = I7F1::from_bits(5); // 2.5 dB
    let mut pdm = Pdm::new(p.PDM, interrupt::take!(PDM), &mut p.P0_00, &mut p.P0_01, config);

    let mut bufs = [[0; 500]; 2];

    pdm
        .run_task_sampler(
            &mut bufs,
            move |buf| {
                // NOTE: It is important that the time spent within this callback
                // does not exceed the time taken to acquire the 1500 samples we
                // have in this example, which would be 10us + 2us per
                // sample * 1500 = 18ms. You need to measure the time taken here
                // and set the sample buffer size accordingly. Exceeding this
                // time can lead to the peripheral re-writing the other buffer.
                info!(
                    "{} samples, min {=i16}, max {=i16}, RMS {=i16}",
                    buf.len(),
                    buf.iter().min().unwrap(),
                    buf.iter().max().unwrap(),
                    (
                        buf.iter().map(|v| i32::from(*v).pow(2)).fold(0i32, |a,b| a.saturating_add(b))
                    / buf.len() as i32).sqrt() as i16,
                );
                SamplerState::Sampled
            },
        )
        .await;
}
