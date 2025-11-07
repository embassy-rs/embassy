#![no_std]
#![no_main]

use crate::hal::lpuart::{Config, Lpuart, lib};
use embassy_executor::Spawner;
use embassy_mcxa276 as hal;

use {defmt_rtt as _, panic_probe as _};

mod common;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = hal::init(hal::config::Config::default());
    let p2 = lib::init();

    defmt::info!("boot");

    // Board-level init for UART2 clocks and pins.
    unsafe {
        common::init_uart2(hal::pac());
    }

    // Create UART configuration
    let config = Config {
        baudrate_bps: 115_200,
        enable_tx: true,
        enable_rx: true,
        ..Default::default()
    };

    // Create UART instance using LPUART2 with PIO2_2 as TX and PIO2_3 as RX
    let lpuart = Lpuart::new_blocking(
        p2.LPUART2, // Peripheral
        p2.PIO2_2,  // TX pin
        p2.PIO2_3,  // RX pin
        config,
    )
    .unwrap();

    // Split into separate TX and RX parts
    let (mut tx, mut rx) = lpuart.split();

    // Write hello messages
    tx.blocking_write(b"Hello world.\r\n").unwrap();
    tx.blocking_write(b"Echoing. Type characters...\r\n")
        .unwrap();

    // Echo loop
    loop {
        let mut buf = [0u8; 1];
        rx.blocking_read(&mut buf).unwrap();
        tx.blocking_write(&buf).unwrap();
    }
}
