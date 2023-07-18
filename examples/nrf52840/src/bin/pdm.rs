#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::pdm::{self, Config, Pdm};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::{Duration, Timer};
use fixed::types::I7F1;
use num_integer::Roots;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PDM => pdm::InterruptHandler<peripherals::PDM>;
});

#[embassy_executor::main]
async fn main(_p: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let config = Config::default();
    let mut pdm = Pdm::new(p.PDM, Irqs, p.P0_01, p.P0_00, config);

    loop {
        for gain in [I7F1::from_num(-20), I7F1::from_num(0), I7F1::from_num(20)] {
            pdm.set_gain(gain, gain);
            info!("Gain = {} dB", defmt::Debug2Format(&gain));
            pdm.start().await;

            // wait some time till the microphon settled
            Timer::after(Duration::from_millis(1000)).await;

            const SAMPLES: usize = 2048;
            let mut buf = [0i16; SAMPLES];
            pdm.sample(&mut buf).await.unwrap();

            let mean = (buf.iter().map(|v| i32::from(*v)).sum::<i32>() / buf.len() as i32) as i16;
            info!(
                "{} samples, min {=i16}, max {=i16}, mean {=i16}, AC RMS {=i16}",
                buf.len(),
                buf.iter().min().unwrap(),
                buf.iter().max().unwrap(),
                mean,
                (buf.iter()
                    .map(|v| i32::from(*v - mean).pow(2))
                    .fold(0i32, |a, b| a.saturating_add(b))
                    / buf.len() as i32)
                    .sqrt() as i16,
            );

            info!("samples: {:?}", &buf);

            pdm.stop().await;
            Timer::after(Duration::from_millis(100)).await;
        }
    }
}
