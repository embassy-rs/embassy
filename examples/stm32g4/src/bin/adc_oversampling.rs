//! adc oversampling example
//!
//! This example uses adc oversampling to achieve 16bit data

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::adc::vals::{Rovsm, Trovs};
use embassy_stm32::adc::{Adc, AdcConfig, SampleTime};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL85,
            divp: None,
            divq: None,
            // Main system clock at 170 MHz
            divr: Some(PllRDiv::DIV2),
        });
        config.rcc.mux.adc12sel = mux::Adcsel::SYS;
        config.rcc.sys = Sysclk::PLL1_R;
    }
    let mut p = embassy_stm32::init(config);

    let mut config = AdcConfig::default();

    // From https://www.st.com/resource/en/reference_manual/rm0440-stm32g4-series-advanced-armbased-32bit-mcus-stmicroelectronics.pdf
    // page652 Oversampler
    // Table 172. Maximum output results vs N and M. Grayed values indicates truncation
    // 0x00 oversampling ratio X2
    // 0x01 oversampling ratio X4
    // 0x02 oversampling ratio X8
    // 0x03 oversampling ratio X16
    // 0x04 oversampling ratio X32
    // 0x05 oversampling ratio X64
    // 0x06 oversampling ratio X128
    // 0x07 oversampling ratio X256
    config.oversampling_ratio = Some(0x03); // ratio X3
    config.oversampling_shift = Some(0b0000); // no shift
    config.oversampling_mode = Some((Rovsm::RESUMED, Trovs::AUTOMATIC, true));

    let mut adc = Adc::new(p.ADC1, config);

    loop {
        let measured = adc.blocking_read(&mut p.PA0, SampleTime::CYCLES6_5);
        info!("data: 0x{:X}", measured); //max 0xFFF0 -> 65520
        Timer::after_millis(500).await;
    }
}
