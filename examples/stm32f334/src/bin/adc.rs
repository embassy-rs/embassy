#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::time::mhz;
use embassy_stm32::{Config, adc, bind_interrupts};
use embassy_time::Timer;
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
            prediv: PllPreDiv::Div1,
            mul: PllMul::Mul9,
        });
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div1;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div1;
        config.rcc.adc = AdcClockSource::Pll(AdcPllPrescaler::Div1);
    }
    let mut p = embassy_stm32::init(config);

    info!("create adc...");

    let mut adc = Adc::new(p.ADC1, Irqs);

    info!("enable vrefint...");

    let mut vrefint = adc.enable_vref();
    let mut temperature = adc.enable_temperature();

    loop {
        let vref = adc.irq_read(&mut vrefint, SampleTime::Cycles6015).await;
        info!("read vref: {} (should be {})", vref, vrefint.calibrated_value());

        let temp = adc.irq_read(&mut temperature, SampleTime::Cycles6015).await;
        info!("read temperature: {}", temp);

        let pin = adc.irq_read(&mut p.PA0, SampleTime::Cycles6015).await;
        info!("read pin: {}", pin);

        let pin_mv = (pin as u32 * vrefint.calibrated_value() as u32 / vref as u32) * 3300 / 4095;
        info!("computed pin mv: {}", pin_mv);

        Timer::after_millis(500).await;
    }
}
