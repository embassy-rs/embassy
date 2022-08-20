#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::interrupt;
use embassy_nrf::pdm::{Config, Channels, Pdm};
use embassy_time::{Duration, Timer};
use fixed::types::I7F1;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_p: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    let mut config = Config::default();
    // Pins are correct for the onboard microphone on the Feather nRF52840 Sense.
    config.channels = Channels::Mono;
    config.gain_left = I7F1::from_bits(5); // 2.5 dB
    let mut pdm = Pdm::new(p.PDM, interrupt::take!(PDM), &mut p.P0_00, &mut p.P0_01, config);

    loop {
        let mut buf = [0; 128];
        pdm.sample(&mut buf).await;
        info!(
            "{} samples, min {=i16}, max {=i16}, mean {=i16}",
            buf.len(),
            buf.iter().min().unwrap(),
            buf.iter().max().unwrap(),
            (buf.iter().map(|v| i32::from(*v)).sum::<i32>() / buf.len() as i32) as i16,
        );
        Timer::after(Duration::from_millis(100)).await;
    }
}
