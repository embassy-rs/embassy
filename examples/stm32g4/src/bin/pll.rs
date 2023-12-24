#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{ClockSrc, Pll, PllM, PllN, PllR, PllSource};
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    config.rcc.pll = Some(Pll {
        source: PllSource::HSI,
        prediv_m: PllM::DIV4,
        mul_n: PllN::MUL85,
        div_p: None,
        div_q: None,
        // Main system clock at 170 MHz
        div_r: Some(PllR::DIV2),
    });

    config.rcc.mux = ClockSrc::PLL;

    let _p = embassy_stm32::init(config);
    info!("Hello World!");

    loop {
        Timer::after_millis(1000).await;
        info!("1s elapsed");
    }
}
