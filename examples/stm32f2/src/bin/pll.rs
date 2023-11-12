#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::convert::TryFrom;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{
    APBPrescaler, ClockSrc, HSEConfig, HSESrc, Pll, PllMul, PllPDiv, PllPreDiv, PllQDiv, PllSource,
};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Example config for maximum performance on a NUCLEO-F207ZG board

    let mut config = Config::default();
    // By default, HSE on the board comes from a 8 MHz clock signal (not a crystal)
    config.rcc.hse = Some(HSEConfig {
        frequency: Hertz(8_000_000),
        source: HSESrc::Bypass,
    });
    // PLL uses HSE as the clock source
    config.rcc.pll_mux = PllSource::HSE;
    config.rcc.pll = Pll {
        // 8 MHz clock source / 8 = 1 MHz PLL input
        pre_div: unwrap!(PllPreDiv::try_from(8)),
        // 1 MHz PLL input * 240 = 240 MHz PLL VCO
        mul: unwrap!(PllMul::try_from(240)),
        // 240 MHz PLL VCO / 2 = 120 MHz main PLL output
        divp: PllPDiv::DIV2,
        // 240 MHz PLL VCO / 5 = 48 MHz PLL48 output
        divq: PllQDiv::DIV5,
    };
    // System clock comes from PLL (= the 120 MHz main PLL output)
    config.rcc.mux = ClockSrc::PLL;
    // 120 MHz / 4 = 30 MHz APB1 frequency
    config.rcc.apb1_pre = APBPrescaler::DIV4;
    // 120 MHz / 2 = 60 MHz APB2 frequency
    config.rcc.apb2_pre = APBPrescaler::DIV2;

    let _p = embassy_stm32::init(config);

    loop {
        Timer::after_millis(1000).await;
        info!("1s elapsed");
    }
}
