//! This example shows how to use the PIO module in the RP235x to read a quadrature rotary encoder.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::rotary_encoder::{Direction, PioEncoder, PioEncoderProgram};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn encoder_0(mut encoder: PioEncoder<'static, PIO0, 0>) {
    let mut count = 0;
    loop {
        info!("Count: {}", count);
        count += match encoder.read().await {
            Direction::Clockwise => 1,
            Direction::CounterClockwise => -1,
        };
    }
}

#[embassy_executor::task]
async fn encoder_1(mut encoder: PioEncoder<'static, PIO0, 1>) {
    let mut count = 0;
    loop {
        info!("Count: {}", count);
        count += match encoder.read().await {
            Direction::Clockwise => 1,
            Direction::CounterClockwise => -1,
        };
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let Pio {
        mut common, sm0, sm1, ..
    } = Pio::new(p.PIO0, Irqs);

    let prg = PioEncoderProgram::new(&mut common);
    let encoder0 = PioEncoder::new(&mut common, sm0, p.PIN_4, p.PIN_5, &prg);
    let encoder1 = PioEncoder::new(&mut common, sm1, p.PIN_6, p.PIN_7, &prg);

    spawner.must_spawn(encoder_0(encoder0));
    spawner.must_spawn(encoder_1(encoder1));
}
