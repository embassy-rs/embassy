#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::Div1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul25,
            divp: Some(PllDiv::Div2),
            divq: Some(PllDiv::Div4), // SPI1 cksel defaults to pll1_q
            divr: None,
        });
        config.rcc.pll2 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul25,
            divp: None,
            divq: None,
            divr: Some(PllDiv::Div4), // 100mhz
        });
        config.rcc.sys = Sysclk::Pll1P; // 200 Mhz
        config.rcc.ahb_pre = AHBPrescaler::Div1; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::Div2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.mux.adcdacsel = mux::Adcdacsel::Pll2R;
    }
    let mut p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1);

    let mut vrefint_channel = adc.enable_vrefint();

    loop {
        let vrefint = adc.blocking_read(&mut vrefint_channel, SampleTime::Cycles245);
        info!("vrefint: {}", vrefint);
        let measured = adc.blocking_read(&mut p.PA0, SampleTime::Cycles245);
        info!("measured: {}", measured);
        Timer::after_millis(500).await;
    }
}
