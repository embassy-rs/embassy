#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::usart::{BufferedUart, Config};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embedded_io_async::BufRead;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART3 => usart::BufferedInterruptHandler<peripherals::USART3>;
});

/// This example is written for the nucleo-stm32f429zi, with a stm32f429zi chip.
///
/// If you are using a different board or chip, make sure you update the following:
///
/// * [ ] Update .cargo/config.toml with the correct `probe-rs run --chip STM32F429ZITx`chip name.
/// * [ ] Update Cargo.toml to have the correct `embassy-stm32` feature, it is
///       currently `stm32f429zi`.
/// * [ ] If your board has a special clock or power configuration, make sure that it is
///       set up appropriately.
/// * [ ] If your board has different pin mapping, update any pin numbers or peripherals
///       to match your schematic
///
/// If you are unsure, please drop by the Embassy Matrix chat for support, and let us know:
///
/// * Which example you are trying to run
/// * Which chip and board you are using
///
/// Embassy Chat: https://matrix.to/#/#embassy-rs:matrix.org
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let config = Config::default();

    let mut tx_buf = [0u8; 32];
    let mut rx_buf = [0u8; 32];
    let mut buf_usart = BufferedUart::new(p.USART3, Irqs, p.PD9, p.PD8, &mut tx_buf, &mut rx_buf, config).unwrap();

    loop {
        let buf = buf_usart.fill_buf().await.unwrap();
        info!("Received: {}", buf);

        // Read bytes have to be explicitly consumed, otherwise fill_buf() will return them again
        let n = buf.len();
        buf_usart.consume(n);
    }
}
