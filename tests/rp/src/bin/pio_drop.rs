#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{PIO0, PIO1};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(pub struct IrqsPio0{PIO0_IRQ_0=>InterruptHandler<PIO0>;});
bind_interrupts!(pub struct IrqsPio1{PIO1_IRQ_0=>InterruptHandler<PIO1>;});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    // add some delay to give an attached debug probe time to parse the
    // defmt RTT header. Reading that header might touch flash memory, which
    // interferes with flash write operations.
    // https://github.com/knurling-rs/defmt/pull/683
    Timer::after_millis(10).await;

    let pio_0 = Pio::new(p.PIO0, IrqsPio0);
    let pio_1 = Pio::new(p.PIO1, IrqsPio1);

    drop(pio_0);
    drop(pio_1);

    info!("Test OK");
    cortex_m::asm::bkpt();
}
