#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::f32::consts::PI;

use defmt::{error, info, trace};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Pin, Pull};
use embassy_nrf::i2s::{Channels, MckFreq, Mode, Ratio, SampleWidth, MODE_MASTER_32000};
use embassy_nrf::pac::ficr::info;
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
    config.mode = MODE_MASTER_32000;
    // config.mode = Mode::Master {
    //     freq: MckFreq::_32MDiv10,
    //     ratio: Ratio::_256x,
    // }; // 12500 Hz
    config.channels = Channels::Left;
    config.swidth = SampleWidth::_16bit;
    let sample_rate = config.mode.sample_rate().expect("I2S Master");
    let inv_sample_rate = 1.0 / sample_rate as f32;

    info!("Sample rate: {}", sample_rate);

    // Wait for a button press
    // let mut btn1 = Input::new(p.P1_00.degrade(), Pull::Up);
    // btn1.wait_for_low().await;

    let irq = interrupt::take!(I2S);
    let mut i2s = i2s::I2S::new(p.I2S, irq, p.P0_28, p.P0_29, p.P0_31, p.P0_11, p.P0_30, config).output();

    type Sample = i16;
    const MAX_UNIPOLAR_VALUE: Sample = (1 << 15) as Sample;
    const NUM_SAMPLES: usize = 2000;
    let mut buffers: [AlignedBuffer<[Sample; NUM_SAMPLES]>; 3] = [
        AlignedBuffer([0; NUM_SAMPLES]),
        AlignedBuffer([0; NUM_SAMPLES]),
        AlignedBuffer([0; NUM_SAMPLES]),
    ];

    let mut carrier = SineOsc::new();

    let mut freq_mod = SineOsc::new();
    freq_mod.set_frequency(8.0, inv_sample_rate);
    freq_mod.set_amplitude(1.0);

    let mut amp_mod = SineOsc::new();
    amp_mod.set_frequency(4.0, inv_sample_rate);
    amp_mod.set_amplitude(0.5);

    let mut generate = |buf: &mut [Sample]| {
        let ptr = buf as *const [Sample] as *const Sample as u32;
        trace!("GEN: {}", ptr);

        for sample in &mut buf.as_mut().chunks_mut(1) {
            let signal = carrier.generate();
            let freq_modulation = bipolar_to_unipolar(freq_mod.generate());
            carrier.set_frequency(220.0 + 220.0 * freq_modulation, inv_sample_rate);
            let amp_modulation = bipolar_to_unipolar(amp_mod.generate());
            carrier.set_amplitude(amp_modulation);
            let value = (MAX_UNIPOLAR_VALUE as f32 * signal) as Sample;
            sample[0] = value;
        }
    };

    generate(buffers[0].as_mut().as_mut_slice());
    generate(buffers[1].as_mut().as_mut_slice());

    i2s.start(buffers[0].as_ref().as_slice()).expect("I2S Start");

    let mut index = 1;
    loop {
        if let Err(err) = i2s.send(buffers[index].as_ref().as_slice()).await {
            error!("{}", err);
        }

        index += 1;
        if index >= 3 {
            index = 0;
        }
        generate(buffers[index].as_mut().as_mut_slice());
    }
}

#[derive(Clone)]
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
