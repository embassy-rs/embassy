#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::lpuart::{Config, Lpuart};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    defmt::info!("boot");

    let config = Config {
        baudrate_bps: 115_200,
        ..Default::default()
    };

    let lpuart = Lpuart::new_blocking(p.LPUART1, p.P1_9, p.P1_8, config).unwrap();
    let (mut tx, mut rx) = lpuart.split();

    tx.blocking_write(b"Hello world.\r\n").unwrap();
    tx.blocking_write(b"Echoing. Type characters...\r\n").unwrap();

    loop {
        let mut buf = [0u8; 1];
        rx.blocking_read(&mut buf).unwrap();
        tx.blocking_write(&buf).unwrap();
    }
}
