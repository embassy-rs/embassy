//! Echo bytes received on UART0.
//!
//! UART0 is muxed to PB0 (TX) and PB1 (RX) with alternate function 1.

#![no_std]
#![no_main]

use embassy_asr::bind_interrupts;
use embassy_asr::gpio::AlternateFunction;
use embassy_asr::uart::{Config, InterruptHandler, Uart};
use embassy_executor::Spawner;
use panic_halt as _;

bind_interrupts!(struct Irqs {
    UART0 => InterruptHandler<embassy_asr::peripherals::UART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_asr::init(Default::default());

    let mut uart = Uart::new(
        p.UART0,
        Some((p.PB0, AlternateFunction::Function1)),
        Some((p.PB1, AlternateFunction::Function1)),
        Irqs,
        Config::default(),
    )
    .unwrap();

    uart.write(b"embassy-asr UART echo\r\n").await.unwrap();

    let mut byte = [0u8; 1];
    loop {
        uart.read(&mut byte).await.unwrap();
        uart.write(&byte).await.unwrap();
    }
}
