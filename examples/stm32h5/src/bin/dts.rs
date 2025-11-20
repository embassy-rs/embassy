#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dts::{Dts, InterruptHandler, SampleTime};
use embassy_stm32::peripherals::DTS;
use embassy_stm32::rcc::frequency;
use embassy_stm32::{Config, bind_interrupts, dts};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    DTS => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL25,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV4), // SPI1 cksel defaults to pll1_q
            divr: None,
        });
        config.rcc.pll2 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL25,
            divp: None,
            divq: None,
            divr: Some(PllDiv::DIV4), // 100mhz
        });
        config.rcc.sys = Sysclk::PLL1_P; // 200 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV1; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.mux.adcdacsel = mux::Adcdacsel::PLL2_R;
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut config = dts::Config::default();
    config.sample_time = SampleTime::ClockCycles15;
    let mut dts = Dts::new(p.DTS, Irqs, config);

    let cal = Dts::factory_calibration();
    let convert_to_celsius = |raw_temp: u16| {
        let raw_temp = raw_temp as f32;
        let sample_time = (config.sample_time as u8) as f32;

        let f = frequency::<DTS>().0 as f32;

        let t0 = cal.t0 as f32;
        let fmt0 = cal.fmt0.0 as f32;
        let ramp_coeff = cal.ramp_coeff as f32;

        ((f * sample_time / raw_temp) - fmt0) / ramp_coeff + t0
    };

    loop {
        let temp = dts.read().await;
        info!("Temp: {} degrees", convert_to_celsius(temp));
        Timer::after_millis(500).await;
    }
}
