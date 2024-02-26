#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::rcc::{AdcClockSource, Pll, PllMul, PllPreDiv, PllRDiv, Pllsrc, Sysclk};
use embassy_stm32::Config;
use embassy_time::{Delay, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    config.rcc.pll = Some(Pll {
        source: Pllsrc::HSI,
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL85,
        divp: None,
        divq: None,
        // Main system clock at 170 MHz
        divr: Some(PllRDiv::DIV2),
    });

    config.rcc.adc12_clock_source = AdcClockSource::SYS;
    config.rcc.sys = Sysclk::PLL1_R;

    let mut p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC2, &mut Delay);
    adc.set_sample_time(SampleTime::CYCLES32_5);

    loop {
        let measured = adc.read(&mut p.PA7);
        info!("measured: {}", measured);
        Timer::after_millis(500).await;
    }
}
