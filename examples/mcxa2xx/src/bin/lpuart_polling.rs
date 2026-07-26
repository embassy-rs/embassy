#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

use crate::hal::lpuart::{Config, Lpuart};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    defmt::info!("boot");

    // Create UART configuration
    let config = Config {
        baudrate_bps: 115_200,
        ..Default::default()
    };

    // Create UART instance using LPUART2 with P2_2 as TX and P2_3 as RX
    let lpuart = Lpuart::new_blocking(
        p.LPUART2, // Peripheral
        p.P2_2,    // TX pin
        p.P2_3,    // RX pin
        config,
    )
    .unwrap();

    // Split into separate TX and RX parts
    let (mut tx, mut rx) = lpuart.split();

    // Write hello messages
    tx.blocking_write(b"Hello world.\r\n").unwrap();
    tx.blocking_write(b"Echoing. Type characters...\r\n").unwrap();

    // Echo loop
    loop {
        let mut buf = [0u8; 1];
        rx.blocking_read(&mut buf).unwrap();
        tx.blocking_write(&buf).unwrap();
    }
}
