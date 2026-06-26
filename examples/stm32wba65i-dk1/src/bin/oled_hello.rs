//! Minimal OLED bring-up: retry SPI init until "Hello" appears.
//!
//! ```bash
//! cargo run --release --bin oled_hello
//! ```

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::Config;
use embassy_stm32wba65i_dk1_examples::oled::{init_loop, OledBus};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());
    info!("oled_hello: SPI3 OLED bring-up (PA0/PB8, CS=PE1, DC=PE0, RST=PE3)");

    let led = Output::new(p.PD8, Level::High, Speed::Low);
    let oled = OledBus::new(p.SPI3, p.PA0, p.PB8, p.PE1, p.PE0, p.PE3);

    let _oled = init_loop(oled, led).await;
    info!("oled_hello: display ready — holding");

    loop {
        Timer::after_secs(10).await;
    }
}
