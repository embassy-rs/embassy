#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use defmt_rtt as _; // global logger
use embassy::executor::Spawner;
use embassy_stm32::rcc::{ClockSrc, PLLClkDiv, PLLMul, PLLSource, PLLSrcDiv};
use embassy_stm32::rng::Rng;
use embassy_stm32::{Config, Peripherals};
use panic_probe as _;

fn config() -> Config {
    let mut config = Config::default();
    // 72Mhz clock (16 / 1 * 18 / 4)
    config.rcc.mux = ClockSrc::PLL(
        PLLSource::HSI16,
        PLLClkDiv::Div4,
        PLLSrcDiv::Div1,
        PLLMul::Mul18,
        Some(PLLClkDiv::Div6), // 48Mhz (16 / 1 * 18 / 6)
    );
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut rng = Rng::new(p.RNG);

    let mut buf = [0u8; 16];
    unwrap!(rng.async_fill_bytes(&mut buf).await);
    info!("random bytes: {:02x}", buf);
}
