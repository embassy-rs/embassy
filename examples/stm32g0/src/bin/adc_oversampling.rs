//! adc oversampling example
//!
//! This example uses adc oversampling to achieve 16bit data

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcConfig, Clock, Ovsr, Ovss, Presc, SampleTime};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Adc oversample test");

    let mut config = AdcConfig::default();
    config.clock = Some(Clock::Async { div: Presc::DIV1 });
    config.oversampling_ratio = Some(Ovsr::MUL16);
    config.oversampling_shift = Some(Ovss::NO_SHIFT);
    config.oversampling_enable = Some(true);

    let mut adc = Adc::new_with_config(p.ADC1, config);
    let mut pin = p.PA1;

    loop {
        let v = adc.blocking_read(&mut pin, SampleTime::CYCLES1_5);
        info!("--> {} ", v); //max 65520 = 0xFFF0
        Timer::after_millis(100).await;
    }
}
