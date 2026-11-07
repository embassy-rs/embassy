//! Reads a PDM microphone on PDM20 and prints the level of each buffer.
//!
//! Wire the microphone's clock to P1.13 and its data to P1.14, or change the pins below.

#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::pdm::{self, Config, Pdm, SamplerState};
use embassy_nrf::{bind_interrupts, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    PDM20 => pdm::InterruptHandler<peripherals::PDM20>;
});

/// 64 ms of audio at the default 16 kHz.
const SAMPLES: usize = 1024;

/// EasyDMA needs word-aligned buffers, and `[i16; N]` on its own only promises 2 bytes.
#[repr(align(4))]
struct Buffers([[i16; SAMPLES]; 2]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let mut config = Config::default();
    config.gain_left = 24; // +12 dB
    config.high_pass = Some(pdm::HighPassPole::P79);
    let mut pdm = Pdm::new(p.PDM20, p.P1_13, p.P1_14, Irqs, config);

    let mut buffers = Buffers([[0; SAMPLES]; 2]);
    pdm.run_task_sampler(&mut buffers.0, |samples| {
        let peak = samples.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
        info!("{} samples, peak {}", samples.len(), peak);
        SamplerState::Sampled
    })
    .await
    .unwrap();
}
