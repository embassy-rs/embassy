#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use embassy::executor::Spawner;
use embassy_stm32::rcc::{ClockSrc, PLLClkDiv, PLLMul, PLLSource, PLLSrcDiv};
use embassy_stm32::rng::Rng;
use embassy_stm32::{Config, Peripherals};
use example_common::*;

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

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut rng = Rng::new(p.RNG);

    let mut buf = [0u8; 16];
    unwrap!(rng.async_fill_bytes(&mut buf).await);
    info!("random bytes: {:02x}", buf);
}
