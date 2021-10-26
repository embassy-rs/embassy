#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy::traits::rng::Random;
use embassy_stm32::rcc::{ClockSrc, PLLClkDiv, PLLMul, PLLSource, PLLSrcDiv};
use embassy_stm32::rng::Rng;
use embassy_stm32::{Config, Peripherals};
use example_common::*;

fn config() -> Config {
    let mut config = Config::default();
    config.rcc = config.rcc.clock_src(ClockSrc::PLL(
        PLLSource::HSI16,
        PLLClkDiv::Div2,
        PLLSrcDiv::Div1,
        PLLMul::Mul8,
        Some(PLLClkDiv::Div2),
    ));
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let mut rng = Random::new(Rng::new(p.RNG));

    loop {
        info!("random {}", unwrap!(rng.next_u8(16).await));
        Timer::after(Duration::from_secs(1)).await;
    }
}
