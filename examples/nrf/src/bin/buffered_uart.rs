#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy::executor::Spawner;
use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy_nrf::buffered_uarte::State;
use embassy_nrf::{buffered_uarte::BufferedUarte, interrupt, uarte, Peripherals};
use futures::pin_mut;

use defmt_rtt as _; // global logger
use panic_probe as _;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;

    let mut tx_buffer = [0u8; 4096];
    let mut rx_buffer = [0u8; 4096];

    let irq = interrupt::take!(UARTE0_UART0);
    let mut state = State::new();
    // Please note - important to have hardware flow control (https://github.com/embassy-rs/embassy/issues/536)
    let u = BufferedUarte::new(
        &mut state,
        p.UARTE0,
        p.TIMER0,
        p.PPI_CH0,
        p.PPI_CH1,
        irq,
        p.P0_08,
        p.P0_06,
        p.P0_07,
        p.P0_05,
        config,
        &mut rx_buffer,
        &mut tx_buffer,
    );
    pin_mut!(u);

    info!("uarte initialized!");

    unwrap!(u.write_all(b"Hello!\r\n").await);
    info!("wrote hello in uart!");

    // Simple demo, reading 8-char chunks and echoing them back reversed.
    loop {
        info!("reading...");
        let mut buf = [0u8; 8];
        unwrap!(u.read_exact(&mut buf).await);
        info!("read done, got {}", buf);

        // Reverse buf
        for i in 0..4 {
            buf.swap(i, 7 - i);
        }

        info!("writing...");
        unwrap!(u.write_all(&buf).await);
        info!("write done");

        // Wait until the bytes are actually finished being transmitted
        unwrap!(u.flush().await);
    }
}
