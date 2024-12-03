#![no_std]
#![no_main]

use core::f32::consts::PI;

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_nrf::i2s::{self, Channels, Config, MasterClock, MultiBuffering, Sample as _, SampleWidth, I2S};
use embassy_nrf::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

type Sample = i16;

const NUM_BUFFERS: usize = 2;
const NUM_SAMPLES: usize = 4;

bind_interrupts!(struct Irqs {
    I2S => i2s::InterruptHandler<peripherals::I2S>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let master_clock: MasterClock = i2s::ExactSampleRate::_50000.into();

    let sample_rate = master_clock.sample_rate();
    info!("Sample rate: {}", sample_rate);

    let mut config = Config::default();
    config.sample_width = SampleWidth::_16bit;
    config.channels = Channels::MonoLeft;

    let buffers_out = MultiBuffering::<Sample, NUM_BUFFERS, NUM_SAMPLES>::new();
    let buffers_in = MultiBuffering::<Sample, NUM_BUFFERS, NUM_SAMPLES>::new();
    let mut full_duplex_stream = I2S::new_master(p.I2S, Irqs, p.P0_25, p.P0_26, p.P0_27, master_clock, config)
        .full_duplex(p.P0_29, p.P0_28, buffers_out, buffers_in);

    let mut modulator = SineOsc::new();
    modulator.set_frequency(8.0, 1.0 / sample_rate as f32);
    modulator.set_amplitude(1.0);

    full_duplex_stream.start().await.expect("I2S Start");

    loop {
        let (buff_out, buff_in) = full_duplex_stream.buffers();
        for i in 0..NUM_SAMPLES {
            let modulation = (Sample::SCALE as f32 * bipolar_to_unipolar(modulator.generate())) as Sample;
            buff_out[i] = buff_in[i] * modulation;
        }

        if let Err(err) = full_duplex_stream.send_and_receive().await {
            error!("{}", err);
        }
    }
}

struct SineOsc {
    amplitude: f32,
    modulo: f32,
    phase_inc: f32,
}

impl SineOsc {
    const B: f32 = 4.0 / PI;
    const C: f32 = -4.0 / (PI * PI);
    const P: f32 = 0.225;

    pub fn new() -> Self {
        Self {
            amplitude: 1.0,
            modulo: 0.0,
            phase_inc: 0.0,
        }
    }

    pub fn set_frequency(&mut self, freq: f32, inv_sample_rate: f32) {
        self.phase_inc = freq * inv_sample_rate;
    }

    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }

    pub fn generate(&mut self) -> f32 {
        let signal = self.parabolic_sin(self.modulo);
        self.modulo += self.phase_inc;
        if self.modulo < 0.0 {
            self.modulo += 1.0;
        } else if self.modulo > 1.0 {
            self.modulo -= 1.0;
        }
        signal * self.amplitude
    }

    fn parabolic_sin(&mut self, modulo: f32) -> f32 {
        let angle = PI - modulo * 2.0 * PI;
        let y = Self::B * angle + Self::C * angle * abs(angle);
        Self::P * (y * abs(y) - y) + y
    }
}

#[inline]
fn abs(value: f32) -> f32 {
    if value < 0.0 {
        -value
    } else {
        value
    }
}

#[inline]
fn bipolar_to_unipolar(value: f32) -> f32 {
    (value + 1.0) / 2.0
}
