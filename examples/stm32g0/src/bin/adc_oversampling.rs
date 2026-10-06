//! adc oversampling example
//!
//! This example uses adc oversampling to achieve 16bit data

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, Clock, Config, Oversampling, OversamplingRatio, Prescaler, SampleTime};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Adc oversample test");

    let mut config = Config::default();
    config.clock = Clock::Async(Prescaler::Div1);
    // Accumulate 16 samples without shifting the sum: 12-bit samples become 16-bit results.
    config.oversampling = Some(Oversampling::new(OversamplingRatio::X16, 0));

    let mut adc = Adc::new_blocking(p.ADC1, config);
    let mut pin = p.PA1;

    loop {
        let v = adc.blocking_read(&mut pin, SampleTime::Cycles15);
        info!("--> {} ", v); //max 65520 = 0xFFF0
        Timer::after_millis(100).await;
    }
}
