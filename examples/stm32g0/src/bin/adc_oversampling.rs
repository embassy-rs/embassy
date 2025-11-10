//! adc oversampling example
//!
//! This example uses adc oversampling to achieve 16bit data

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, Clock, Ovsr, Ovss, Presc, SampleTime};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Adc oversample test");

    let mut adc = Adc::new_with_clock(p.ADC1, Clock::Async { div: Presc::DIV1 });
    let mut pin = p.PA1;

    adc.set_oversampling_ratio(Ovsr::MUL16);
    adc.set_oversampling_shift(Ovss::NO_SHIFT);
    adc.oversampling_enable(true);

    loop {
        let v = adc.blocking_read(&mut pin, SampleTime::CYCLES1_5);
        info!("--> {} ", v); //max 65520 = 0xFFF0
        Timer::after_millis(100).await;
    }
}
