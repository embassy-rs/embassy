#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::cmp::Ordering;

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::pdm::{self, Config, Frequency, OperationMode, Pdm, Ratio, SamplerState};
use embassy_nrf::{bind_interrupts, peripherals};
use fixed::types::I7F1;
use microfft::real::rfft_1024;
use num_integer::Roots;
use {defmt_rtt as _, panic_probe as _};

// Demonstrates both continuous sampling and scanning multiple channels driven by a PPI linked timer

bind_interrupts!(struct Irqs {
    PDM => pdm::InterruptHandler<peripherals::PDM>;
});

#[embassy_executor::main]
async fn main(_p: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    let mut config = Config::default();
    // Pins are correct for the onboard microphone on the Feather nRF52840 Sense.
    config.frequency = Frequency::_1280K; // 16 kHz sample rate
    config.ratio = Ratio::RATIO80;
    config.operation_mode = OperationMode::Mono;
    config.gain_left = I7F1::from_bits(5); // 2.5 dB
    let mut pdm = Pdm::new(p.PDM, Irqs, &mut p.P0_00, &mut p.P0_01, config);

    let mut bufs = [[0; 1024]; 2];

    pdm.run_task_sampler(&mut bufs, move |buf| {
        // NOTE: It is important that the time spent within this callback
        // does not exceed the time taken to acquire the 1500 samples we
        // have in this example, which would be 10us + 2us per
        // sample * 1500 = 18ms. You need to measure the time taken here
        // and set the sample buffer size accordingly. Exceeding this
        // time can lead to the peripheral re-writing the other buffer.
        let mean = (buf.iter().map(|v| i32::from(*v)).sum::<i32>() / buf.len() as i32) as i16;
        let (peak_freq_index, peak_mag) = fft_peak_freq(&buf);
        let peak_freq = peak_freq_index * 16000 / buf.len();
        info!(
            "{} samples, min {=i16}, max {=i16}, mean {=i16}, AC RMS {=i16}, peak {} @ {} Hz",
            buf.len(),
            buf.iter().min().unwrap(),
            buf.iter().max().unwrap(),
            mean,
            (buf.iter()
                .map(|v| i32::from(*v - mean).pow(2))
                .fold(0i32, |a, b| a.saturating_add(b))
                / buf.len() as i32)
                .sqrt() as i16,
            peak_mag,
            peak_freq,
        );
        SamplerState::Sampled
    })
    .await
    .unwrap();
}

fn fft_peak_freq(input: &[i16; 1024]) -> (usize, u32) {
    let mut f = [0f32; 1024];
    for i in 0..input.len() {
        f[i] = (input[i] as f32) / 32768.0;
    }
    // N.B. rfft_1024 does the FFT in-place so result is actually also a reference to f.
    let result = rfft_1024(&mut f);
    result[0].im = 0.0;

    result
        .iter()
        .map(|c| c.norm_sqr())
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .map(|(i, v)| (i, ((v * 32768.0) as u32).sqrt()))
        .unwrap()
}
