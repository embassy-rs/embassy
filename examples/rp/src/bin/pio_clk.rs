//! This example shows how to output a clock signal on an output pin using the PIO module in the RP2040 chip.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::clk::{PioClk, PioClkProgram};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let prg = PioClkProgram::new(&mut common);
    let mut clk = PioClk::new(&mut common, sm0, p.PIN_18, &prg, 10_000);

    loop {
        clk.start();
        Timer::after_millis(5000).await;
        clk.stop();
        Timer::after_millis(5000).await;
    }
}
