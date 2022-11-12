// Example inspired by RTIC's I2S demo: https://github.com/nrf-rs/nrf-hal/blob/master/examples/i2s-controller-demo/src/main.rs

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::f32::consts::PI;

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_nrf::i2s::{MckFreq, Mode, Ratio, MODE_MASTER_16000, MODE_MASTER_8000};
use embassy_nrf::{i2s, interrupt};
use {defmt_rtt as _, panic_probe as _};

#[repr(align(4))]
pub struct AlignedBuffer<T: ?Sized>(T);

impl<T> AsRef<T> for AlignedBuffer<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for AlignedBuffer<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut config = i2s::Config::default();
    // config.mode = MODE_MASTER_16000;
    config.mode = Mode::Master {
        freq: MckFreq::_32MDiv10,
        ratio: Ratio::_256x,
    }; // 12500 Hz
    let sample_rate = config.mode.sample_rate().expect("I2S Master");
    let inv_sample_rate = 1.0 / sample_rate as f32;

    info!("Sample rate: {}", sample_rate);

    let irq = interrupt::take!(I2S);
    let mut i2s = i2s::I2S::new(p.I2S, irq, p.P0_28, p.P0_29, p.P0_31, p.P0_11, p.P0_30, config);

    const BUF_SAMPLES: usize = 250;
    const BUF_SIZE: usize = BUF_SAMPLES * 2;
    let mut buf = AlignedBuffer([0i16; BUF_SIZE]);

    let mut carrier = SineOsc::new();
    carrier.set_frequency(300.0, inv_sample_rate);

    let mut modulator = SineOsc::new();
    modulator.set_frequency(0.01, inv_sample_rate);
    modulator.set_amplitude(0.2);

    i2s.set_tx_enabled(true);
    i2s.start();

    loop {
        for sample in buf.as_mut().chunks_mut(2) {
            let signal = carrier.generate();
            // let modulation = bipolar_to_unipolar(modulator.generate());
            // carrier.set_frequency(200.0 + 100.0 * modulation, inv_sample_rate);
            // carrier.set_amplitude((modulation);
            let value = (i16::MAX as f32 * signal) as i16;
            sample[0] = value;
            sample[1] = value;
            // info!("{}", signal);
        }

        if let Err(err) = i2s.tx(buf.as_ref().as_slice()).await {
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
