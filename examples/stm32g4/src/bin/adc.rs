#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::rcc::{AdcClockSource, ClockSrc, Pll, PllM, PllN, PllR, PllSrc};
use embassy_stm32::Config;
use embassy_time::{Delay, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    config.rcc.pll = Some(Pll {
        source: PllSrc::HSI16,
        prediv_m: PllM::DIV4,
        mul_n: PllN::MUL85,
        div_p: None,
        div_q: None,
        // Main system clock at 170 MHz
        div_r: Some(PllR::DIV2),
    });

    config.rcc.adc12_clock_source = AdcClockSource::SYSCLK;
    config.rcc.mux = ClockSrc::PLL;

    let mut p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC2, &mut Delay);
    adc.set_sample_time(SampleTime::Cycles32_5);

    loop {
        let measured = adc.read(&mut p.PA7);
        info!("measured: {}", measured);
        Timer::after(Duration::from_millis(500)).await;
    }
}
