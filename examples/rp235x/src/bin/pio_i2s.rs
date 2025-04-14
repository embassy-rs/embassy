//! This example shows generating audio and sending it to a connected i2s DAC using the PIO
//! module of the RP235x.
//!
//! Connect the i2s DAC as follows:
//!   bclk : GPIO 18
//!   lrc  : GPIO 19
//!   din  : GPIO 20
//! Then hold down the boot select button to trigger a rising triangle waveform.

#![no_std]
#![no_main]

use core::mem;

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::i2s::{PioI2sOut, PioI2sOutProgram};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

const SAMPLE_RATE: u32 = 48_000;
const BIT_DEPTH: u32 = 16;
const CHANNELS: u32 = 2;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Setup pio state machine for i2s output
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let bit_clock_pin = p.PIN_18;
    let left_right_clock_pin = p.PIN_19;
    let data_pin = p.PIN_20;

    let program = PioI2sOutProgram::new(&mut common);
    let mut i2s = PioI2sOut::new(
        &mut common,
        sm0,
        p.DMA_CH0,
        data_pin,
        bit_clock_pin,
        left_right_clock_pin,
        SAMPLE_RATE,
        BIT_DEPTH,
        CHANNELS,
        &program,
    );

    let fade_input = Input::new(p.PIN_0, Pull::Up);

    // create two audio buffers (back and front) which will take turns being
    // filled with new audio data and being sent to the pio fifo using dma
    const BUFFER_SIZE: usize = 960;
    static DMA_BUFFER: StaticCell<[u32; BUFFER_SIZE * 2]> = StaticCell::new();
    let dma_buffer = DMA_BUFFER.init_with(|| [0u32; BUFFER_SIZE * 2]);
    let (mut back_buffer, mut front_buffer) = dma_buffer.split_at_mut(BUFFER_SIZE);

    // start pio state machine
    let mut fade_value: i32 = 0;
    let mut phase: i32 = 0;

    loop {
        // trigger transfer of front buffer data to the pio fifo
        // but don't await the returned future, yet
        let dma_future = i2s.write(front_buffer);

        // fade in audio when bootsel is pressed
        let fade_target = if fade_input.is_low() { i32::MAX } else { 0 };

        // fill back buffer with fresh audio samples before awaiting the dma future
        for s in back_buffer.iter_mut() {
            // exponential approach of fade_value => fade_target
            fade_value += (fade_target - fade_value) >> 14;
            // generate triangle wave with amplitude and frequency based on fade value
            phase = (phase + (fade_value >> 22)) & 0xffff;
            let triangle_sample = (phase as i16 as i32).abs() - 16384;
            let sample = (triangle_sample * (fade_value >> 15)) >> 16;
            // duplicate mono sample into lower and upper half of dma word
            *s = (sample as u16 as u32) * 0x10001;
        }

        // now await the dma future. once the dma finishes, the next buffer needs to be queued
        // within DMA_DEPTH / SAMPLE_RATE = 8 / 48000 seconds = 166us
        dma_future.await;
        mem::swap(&mut back_buffer, &mut front_buffer);
    }
}
