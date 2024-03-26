#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::time::mhz;
use embassy_stm32::{adc, bind_interrupts, Config};
use embassy_time::{Delay, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC1_2 => adc::InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: mhz(8),
            mode: HseMode::Bypass,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL9,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
        config.rcc.adc = AdcClockSource::Pll(AdcPllPrescaler::DIV1);
    }
    let mut p = embassy_stm32::init(config);

    info!("create adc...");

    let mut adc = Adc::new(p.ADC1, Irqs, &mut Delay);

    adc.set_sample_time(SampleTime::CYCLES601_5);

    info!("enable vrefint...");

    let mut vrefint = adc.enable_vref(&mut Delay);
    let mut temperature = adc.enable_temperature();

    loop {
        let vref = adc.read(&mut vrefint).await;
        info!("read vref: {} (should be {})", vref, vrefint.value());

        let temp = adc.read(&mut temperature).await;
        info!("read temperature: {}", temp);

        let pin = adc.read(&mut p.PA0).await;
        info!("read pin: {}", pin);

        let pin_mv = (pin as u32 * vrefint.value() as u32 / vref as u32) * 3300 / 4095;
        info!("computed pin mv: {}", pin_mv);

        Timer::after_millis(500).await;
    }
}
