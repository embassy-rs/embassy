#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::convert::TryFrom;
use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::{
    rcc::{
        APBPrescaler, ClockSrc, HSEConfig, HSESrc, PLL48Div, PLLConfig, PLLMainDiv, PLLMul,
        PLLPreDiv, PLLSrc,
    },
    time::Hertz,
    Config, Peripherals,
};

use defmt_rtt as _; // global logger
use panic_probe as _;

// Example config for maximum performance on a NUCLEO-F207ZG board
fn config() -> Config {
    let mut config = Config::default();
    // By default, HSE on the board comes from a 8 MHz clock signal (not a crystal)
    config.rcc.hse = Some(HSEConfig {
        frequency: Hertz(8_000_000),
        source: HSESrc::Bypass,
    });
    // PLL uses HSE as the clock source
    config.rcc.pll_mux = PLLSrc::HSE;
    config.rcc.pll = PLLConfig {
        // 8 MHz clock source / 8 = 1 MHz PLL input
        pre_div: unwrap!(PLLPreDiv::try_from(8)),
        // 1 MHz PLL input * 240 = 240 MHz PLL VCO
        mul: unwrap!(PLLMul::try_from(240)),
        // 240 MHz PLL VCO / 2 = 120 MHz main PLL output
        main_div: PLLMainDiv::Div2,
        // 240 MHz PLL VCO / 5 = 48 MHz PLL48 output
        pll48_div: unwrap!(PLL48Div::try_from(5)),
    };
    // System clock comes from PLL (= the 120 MHz main PLL output)
    config.rcc.mux = ClockSrc::PLL;
    // 120 MHz / 4 = 30 MHz APB1 frequency
    config.rcc.apb1_pre = APBPrescaler::Div4;
    // 120 MHz / 2 = 60 MHz APB2 frequency
    config.rcc.apb2_pre = APBPrescaler::Div2;
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, _p: Peripherals) {
    loop {
        Timer::after(Duration::from_millis(1000)).await;
        info!("1s elapsed");
    }
}
