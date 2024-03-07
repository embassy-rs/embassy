#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::opamp::{OpAmp, OpAmpGain};
use embassy_stm32::peripherals::ADC2;
use embassy_stm32::time::mhz;
use embassy_stm32::{adc, bind_interrupts, Config};
use embassy_time::{Delay, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC1_2 => adc::InterruptHandler<ADC2>;
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

    let mut adc = Adc::new(p.ADC2, Irqs, &mut Delay);
    let mut opamp = OpAmp::new(p.OPAMP2);

    adc.set_sample_time(SampleTime::CYCLES601_5);

    info!("enable vrefint...");

    let mut vrefint = adc.enable_vref(&mut Delay);
    let mut temperature = adc.enable_temperature();
    let mut buffer = opamp.buffer_ext(&mut p.PA7, &mut p.PA6, OpAmpGain::Mul1);

    loop {
        let vref = adc.read(&mut vrefint).await;
        info!("read vref: {} (should be {})", vref, vrefint.value());

        let temp = adc.read(&mut temperature).await;
        info!("read temperature: {}", temp);

        let buffer = adc.read(&mut buffer).await;
        info!("read buffer: {}", buffer);

        let pin_mv = (buffer as u32 * vrefint.value() as u32 / vref as u32) * 3300 / 4095;
        info!("computed pin mv: {}", pin_mv);

        Timer::after_millis(500).await;
    }
}
