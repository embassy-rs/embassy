#![no_std]
#![no_main]

use core::f32::consts::PI;

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_nrf::i2s::{self, Channels, Config, DoubleBuffering, MasterClock, Sample as _, SampleWidth, I2S};
use embassy_nrf::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

type Sample = i16;

const NUM_SAMPLES: usize = 50;

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

    let buffers = DoubleBuffering::<Sample, NUM_SAMPLES>::new();
    let mut output_stream =
        I2S::new_master(p.I2S, Irqs, p.P0_25, p.P0_26, p.P0_27, master_clock, config).output(p.P0_28, buffers);

    let mut waveform = Waveform::new(1.0 / sample_rate as f32);

    waveform.process(output_stream.buffer());

    output_stream.start().await.expect("I2S Start");

    loop {
        waveform.process(output_stream.buffer());

        if let Err(err) = output_stream.send().await {
            error!("{}", err);
        }
    }
}

struct Waveform {
    inv_sample_rate: f32,
    carrier: SineOsc,
    freq_mod: SineOsc,
    amp_mod: SineOsc,
}

impl Waveform {
    fn new(inv_sample_rate: f32) -> Self {
        let mut carrier = SineOsc::new();
        carrier.set_frequency(110.0, inv_sample_rate);

        let mut freq_mod = SineOsc::new();
        freq_mod.set_frequency(1.0, inv_sample_rate);
        freq_mod.set_amplitude(1.0);

        let mut amp_mod = SineOsc::new();
        amp_mod.set_frequency(16.0, inv_sample_rate);
        amp_mod.set_amplitude(0.5);

        Self {
            inv_sample_rate,
            carrier,
            freq_mod,
            amp_mod,
        }
    }

    fn process(&mut self, buf: &mut [Sample]) {
        for sample in buf.chunks_mut(1) {
            let freq_modulation = bipolar_to_unipolar(self.freq_mod.generate());
            self.carrier
                .set_frequency(110.0 + 440.0 * freq_modulation, self.inv_sample_rate);

            let amp_modulation = bipolar_to_unipolar(self.amp_mod.generate());
            self.carrier.set_amplitude(amp_modulation);

            let signal = self.carrier.generate();

            sample[0] = (Sample::SCALE as f32 * signal) as Sample;
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
