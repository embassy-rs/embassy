#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::executor::Spawner;
use embassy_stm32::rcc::{ClockSrc, PLLClkDiv, PLLMul, PLLSource, PLLSrcDiv};
use embassy_stm32::rng::Rng;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

fn config() -> Config {
    let mut config = Config::default();
    config.rcc.mux = ClockSrc::PLL(
        PLLSource::HSI16,
        PLLClkDiv::Div2,
        PLLSrcDiv::Div1,
        PLLMul::Mul8,
        Some(PLLClkDiv::Div2),
    );
    config
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    let mut rng = Rng::new(p.RNG);

    let mut buf = [0u8; 16];
    unwrap!(rng.async_fill_bytes(&mut buf).await);
    info!("random bytes: {:02x}", buf);
}
