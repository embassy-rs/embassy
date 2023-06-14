#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{ClockSrc, Pll, PllM, PllN, PllR, PllSrc};
use embassy_stm32::Config;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    config.rcc.pll = Some(Pll {
        source: PllSrc::HSI16,
        prediv_m: PllM::Div4,
        mul_n: PllN::Mul85,
        div_p: None,
        div_q: None,
        // Main system clock at 170 MHz
        div_r: Some(PllR::Div2),
    });

    config.rcc.mux = ClockSrc::PLL;

    let _p = embassy_stm32::init(config);
    info!("Hello World!");

    loop {
        Timer::after(Duration::from_millis(1000)).await;
        info!("1s elapsed");
    }
}
