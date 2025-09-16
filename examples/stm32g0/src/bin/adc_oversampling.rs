//! adc oversampling example
//!
//! This example uses adc oversampling to achieve 16bit data

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, Clock, Presc, SampleTime};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Adc oversample test");

    let mut adc = Adc::new_with_clock(p.ADC1, Clock::Async { div: Presc::DIV1 });
    adc.set_sample_time(SampleTime::CYCLES1_5);
    let mut pin = p.PA1;

    // From https://www.st.com/resource/en/reference_manual/rm0444-stm32g0x1-advanced-armbased-32bit-mcus-stmicroelectronics.pdf
    // page373 15.8 Oversampler
    // Table 76. Maximum output results vs N and M. Grayed values indicates truncation
    // 0x00 oversampling ratio X2
    // 0x01 oversampling ratio X4
    // 0x02 oversampling ratio X8
    // 0x03 oversampling ratio X16
    // 0x04 oversampling ratio X32
    // 0x05 oversampling ratio X64
    // 0x06 oversampling ratio X128
    // 0x07 oversampling ratio X256
    adc.set_oversampling_ratio(0x03);
    adc.set_oversampling_shift(0b0000);
    adc.oversampling_enable(true);

    loop {
        let v = adc.blocking_read(&mut pin);
        info!("--> {} ", v); //max 65520 = 0xFFF0
        Timer::after_millis(100).await;
    }
}
