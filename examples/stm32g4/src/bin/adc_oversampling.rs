//! adc oversampling example
//!
//! This example uses adc oversampling to achieve 16bit data

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::adc::{Adc, AdcChannel, Config as AdcConfig, Oversampling, OversamplingRatio, SampleTime};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.pll = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul85,
            divp: None,
            divq: None,
            // Main system clock at 170 MHz
            divr: Some(PllRDiv::Div2),
        });
        config.rcc.mux.adc12sel = mux::Adcsel::Sys;
        config.rcc.sys = Sysclk::Pll1R;
    }
    let mut p = embassy_stm32::init(config);

    let mut config = AdcConfig::default();

    // From https://www.st.com/resource/en/reference_manual/rm0440-stm32g4-series-advanced-armbased-32bit-mcus-stmicroelectronics.pdf
    // page652 Oversampler
    // Table 172. Maximum output results vs N and M. Grayed values indicates truncation
    //
    // Accumulate 16 samples (ratio X16) without shifting the sum, so a 12-bit conversion
    // yields a 16-bit result.
    let mut oversampling = Oversampling::new(OversamplingRatio::X16, 0); // no shift
    oversampling.resumed = true; // resume the sequence after an injected conversion
    config.oversampling = Some(oversampling);

    let mut adc = Adc::new_blocking(p.ADC1, config);

    loop {
        let measured = adc.blocking_read(p.PA0.reborrow_adc(), SampleTime::Cycles65);
        info!("data: 0x{:X}", measured); //max 0xFFF0 -> 65520
        Timer::after_millis(500).await;
    }
}
