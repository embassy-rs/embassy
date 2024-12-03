#![no_std]
#![no_main]

use defmt::{debug, error, info};
use embassy_executor::Spawner;
use embassy_nrf::i2s::{self, Channels, Config, DoubleBuffering, MasterClock, Sample as _, SampleWidth, I2S};
use embassy_nrf::pwm::{Prescaler, SimplePwm};
use embassy_nrf::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

type Sample = i16;

const NUM_SAMPLES: usize = 500;

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
    let mut input_stream =
        I2S::new_master(p.I2S, Irqs, p.P0_25, p.P0_26, p.P0_27, master_clock, config).input(p.P0_29, buffers);

    // Configure the PWM to use the pins corresponding to the RGB leds
    let mut pwm = SimplePwm::new_3ch(p.PWM0, p.P0_23, p.P0_22, p.P0_24);
    pwm.set_prescaler(Prescaler::Div1);
    pwm.set_max_duty(255);

    let mut rms_online = RmsOnline::<NUM_SAMPLES>::default();

    input_stream.start().await.expect("I2S Start");

    loop {
        let rms = rms_online.process(input_stream.buffer());
        let rgb = rgb_from_rms(rms);

        debug!("RMS: {}, RGB: {:?}", rms, rgb);
        for i in 0..3 {
            pwm.set_duty(i, rgb[i].into());
        }

        if let Err(err) = input_stream.receive().await {
            error!("{}", err);
        }
    }
}

/// RMS from 0.0 until 0.75 will give green with a proportional intensity
/// RMS from 0.75 until 0.9 will give a blend between orange and red proportionally to the intensity
/// RMS above 0.9 will give a red with a proportional intensity
fn rgb_from_rms(rms: f32) -> [u8; 3] {
    if rms < 0.75 {
        let intensity = rms / 0.75;
        [0, (intensity * 165.0) as u8, 0]
    } else if rms < 0.9 {
        let intensity = (rms - 0.75) / 0.15;
        [200, 165 - (165.0 * intensity) as u8, 0]
    } else {
        let intensity = (rms - 0.9) / 0.1;
        [200 + (55.0 * intensity) as u8, 0, 0]
    }
}

pub struct RmsOnline<const N: usize> {
    pub squares: [f32; N],
    pub head: usize,
}

impl<const N: usize> Default for RmsOnline<N> {
    fn default() -> Self {
        RmsOnline {
            squares: [0.0; N],
            head: 0,
        }
    }
}

impl<const N: usize> RmsOnline<N> {
    pub fn reset(&mut self) {
        self.squares = [0.0; N];
        self.head = 0;
    }

    pub fn process(&mut self, buf: &[Sample]) -> f32 {
        buf.iter()
            .for_each(|sample| self.push(*sample as f32 / Sample::SCALE as f32));

        let sum_of_squares = self.squares.iter().fold(0.0, |acc, v| acc + *v);
        Self::approx_sqrt(sum_of_squares / N as f32)
    }

    pub fn push(&mut self, signal: f32) {
        let square = signal * signal;
        self.squares[self.head] = square;
        self.head = (self.head + 1) % N;
    }

    /// Approximated sqrt taken from [micromath]
    ///
    /// [micromath]: https://docs.rs/micromath/latest/src/micromath/float/sqrt.rs.html#11-17
    ///
    fn approx_sqrt(value: f32) -> f32 {
        f32::from_bits((value.to_bits() + 0x3f80_0000) >> 1)
    }
}
