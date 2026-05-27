#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::Hertz;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(24_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div3,
            mul: PllMul::Mul150,
            divp: Some(PllDiv::Div2),
            divq: None,
            divr: None,
            divs: None,
            divt: None,
        });
        config.rcc.sys = Sysclk::Pll1P; // 600 Mhz
        config.rcc.ahb_pre = AHBPrescaler::Div2; // 300 Mhz
        config.rcc.apb1_pre = APBPrescaler::Div2; // 150 Mhz
        config.rcc.apb2_pre = APBPrescaler::Div2; // 150 Mhz
        config.rcc.apb4_pre = APBPrescaler::Div2; // 150 Mhz
        config.rcc.apb5_pre = APBPrescaler::Div2; // 150 Mhz
        config.rcc.voltage_scale = VoltageScale::High;
    }
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut led = Output::new(p.PD10, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(500).await;

        info!("low");
        led.set_low();
        Timer::after_millis(500).await;
    }
}
