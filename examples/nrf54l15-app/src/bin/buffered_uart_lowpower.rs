#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::buffered_uarte::{self, BufferedUarte};
use embassy_nrf::gpio::{Input, Pull};
use embassy_nrf::{bind_interrupts, peripherals, uarte};
use embedded_io_async::Write;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SERIAL30 => buffered_uarte::InterruptHandler<peripherals::SERIAL30>;
});

/// Uses the pins of the VCOM0 and button 0 on the nRF54L15 devkit.
/// Loops the following:
///   - write Hello! to VCOM0
///   - read some bytes from VCOM0
///   - wait for button 0 to be pressed
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_nrf::init(Default::default());
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::Excluded;
    config.baudrate = uarte::Baudrate::Baud115200;

    let mut tx_buffer = [0u8; 12];
    let mut rx_buffer = [0u8; 12];

    loop {
        let mut u = BufferedUarte::new(
            p.SERIAL30.reborrow(),
            p.P0_01.reborrow(),
            p.P0_00.reborrow(),
            Irqs,
            config.clone(),
            &mut rx_buffer,
            &mut tx_buffer,
        );

        info!("uarte initialized!");

        unwrap!(u.write_all(b"Hello!\r\n").await);
        info!("wrote hello in uart!");

        info!("reading...");
        let buf = unwrap!(u.fill_buf().await);
        info!("read done, got {}", buf);

        // Read bytes have to be explicitly consumed, otherwise fill_buf() will return them again
        let n = buf.len();
        u.consume(n);

        // Enter low power state of the UART
        drop(u);

        // Wait for button 0 to be pressed
        Input::new(p.P1_13.reborrow(), Pull::Up).wait_for_low().await;
    }
}
