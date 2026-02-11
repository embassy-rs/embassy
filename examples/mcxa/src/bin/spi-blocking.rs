#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::clocks::config::Div8;
use hal::config::Config;
use hal::spi::controller::{self, Spi};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.clock_cfg.sirc.fro_lf_div = Div8::from_divisor(1);

    let p = hal::init(config);

    info!("SPI example");

    let mut config = controller::Config::default();
    config.frequency = 1_000_000;
    let mut spi = Spi::new_blocking(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, config).unwrap();

    let mut rx_buf = [0u8; 32];
    let tx_buf = [0x55u8; 32];

    loop {
        spi.blocking_transfer(&mut rx_buf, &tx_buf).unwrap();
        assert!(rx_buf.iter().all(|b| *b == 0x55));
        Timer::after_secs(1).await;
    }
}
